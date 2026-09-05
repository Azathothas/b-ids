#!/bin/sh
# check-msrv.sh - is the declared minimum supported Rust version derived from
# the dependency graph, or is it a number somebody typed?
#
# The defect this exists to catch is a `rust-version` field that nobody
# measured. docs/history/todo/tooling.md TOOL-01 states the rule: the dependency graph says
# what the workspace actually requires, and a number chosen by hand goes stale
# the first time a dependency raises its own floor. What is left behind then is
# a CLAIM that reads like a CONSTRAINT: consumers on the declared version get a
# compile error the manifest promised they would not.
#
# -- WHAT IT CHECKS ----------------------------------------------------------
#   1. ⛔ The workspace declares a `rust-version` at all. An absent one is not
#      "any version": it is a promise nobody made.
#   2. ⛔ The declared value is not BELOW the floor the resolved dependency
#      graph imposes, which is the highest `rust-version` any package outside
#      this workspace declares.
#
# ⚠ THE GRAPH IS ONE OF TWO LEGS AND IT IS THE WEAKER ONE. A graph with no
# dependencies imposes no floor at all, which is this tree's state today, and a
# check that reported a floor there would be inventing one. The other leg is
# --verify, which COMPILES the workspace with the declared toolchain and is the
# only thing that can say the declared value is reachable. Neither leg alone is
# a measurement of the true minimum: the graph cannot see the language features
# the code uses, and --verify proves the declared version WORKS without proving
# nothing older would.
#
# ⛔ WORKSPACE MEMBERS ARE EXCLUDED FROM THE FLOOR, and that exclusion is the
# whole reason this check can fail. Every member inherits `rust-version` from
# the workspace, so a floor computed over all packages would read back the
# value it is checking and agree with itself forever. That is the "acceptance
# command that cannot fail" row in docs/conventions/forbidden-patterns.md.
#
# ⚠ --write IS THE FIX FLAG and it REFUSES when the graph imposes no floor.
# A helper that guessed a version there would be writing the exact fabricated
# number this check exists to find.
#
# Usage:
#   sh scripts/common/check-msrv.sh
#   sh scripts/common/check-msrv.sh --json
#   sh scripts/common/check-msrv.sh --verify
#   sh scripts/common/check-msrv.sh --write
#
# Exit codes: 0 clean, 1 the declared value is missing or too low, 2 could not
# run (no cargo, no jq, or --verify with the declared toolchain not installed).
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
WRITE=0
VERIFY=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --write) WRITE=1 ;;
    --verify) VERIFY=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-msrv: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-msrv: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-msrv: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-msrv: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

[ -f Cargo.toml ] || {
  printf 'check-msrv: no Cargo.toml in this repository.\n' >&2
  printf '  That is "could not run", not "passed".\n' >&2
  exit 2
}
command -v cargo >/dev/null 2>&1 || { printf 'check-msrv: cargo is not on PATH\n' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { printf 'check-msrv: jq is not on PATH; it reads cargo metadata\n' >&2; exit 2; }
command -v awk >/dev/null 2>&1 || { printf 'check-msrv: awk not found\n' >&2; exit 2; }

# -- the declared value ------------------------------------------------------
# ⚠ Scoped to the [workspace.package] table. A `rust-version` under some other
# table is a different field, and matching it anywhere in the file is how a
# check reads the wrong one.
DECLARED=$(awk '
  /^\[/ { in_wp = ($0 == "[workspace.package]") ; next }
  in_wp && /^[ \t]*rust-version[ \t]*=/ {
    line = $0
    sub(/^[^=]*=[ \t]*/, "", line)
    gsub(/["'"'"']/, "", line)
    sub(/[ \t]*(#.*)?$/, "", line)
    print line
    exit
  }
' Cargo.toml)

# -- the floor the resolved graph imposes ------------------------------------
# ⛔ Resolution can reach the network the first time. It is still the right
# command: --no-deps would answer with the workspace alone, which is the set
# this check has to exclude.
META=$(cargo metadata --format-version 1 2>/dev/null) || {
  printf 'check-msrv: cargo metadata failed. Run it directly to see why:\n' >&2
  printf '  cargo metadata --format-version 1\n' >&2
  exit 2
}

PKG_TOTAL=$(printf '%s' "$META" | jq -r '.packages | length')
# `[.id] - $m` is empty exactly when the package IS a workspace member, so
# `length > 0` keeps the dependencies and drops this project's own crates.
FLOOR_LINES=$(printf '%s' "$META" | jq -r '
  .workspace_members as $m
  | .packages[]
  | select(([.id] - $m) | length > 0)
  | select(.rust_version != null)
  | "\(.rust_version)\t\(.name)"
')
DEP_TOTAL=$(printf '%s' "$META" | jq -r '
  .workspace_members as $m
  | [.packages[] | select(([.id] - $m) | length > 0)] | length
')

# ⚠ The maximum is taken by awk rather than by `sort`, because `sort` on a
# native PowerShell host is an alias for Sort-Object and answers differently.
# scripts/README.md carries the measurement.
FLOOR=$(printf '%s\n' "$FLOOR_LINES" | awk -F'\t' '
  function num(v,   a) { split(v, a, "."); return (a[1] + 0) * 1000000 + (a[2] + 0) * 1000 + (a[3] + 0) }
  $1 != "" { n = num($1); if (n > best) { best = n; bv = $1; bn = $2 } }
  END { if (best > 0) printf "%s %s\n", bv, bn }
')
FLOOR_VERSION=$(printf '%s' "$FLOOR" | awk '{ print $1 }')
FLOOR_PACKAGE=$(printf '%s' "$FLOOR" | awk '{ print $2 }')

# -- the fix flag, which refuses rather than guessing -------------------------
if [ "$WRITE" = 1 ]; then
  if [ -z "$FLOOR_VERSION" ]; then
    printf 'check-msrv: REFUSED. The dependency graph imposes no floor, so there is\n' >&2
    printf '  nothing to derive and nothing to write. %s package(s) resolved, none\n' "$DEP_TOTAL" >&2
    printf '  of them outside this workspace declares a rust-version.\n' >&2
    printf '  ⛔ A version invented here would be the defect this check exists to find.\n' >&2
    exit 2
  fi
  command -v node >/dev/null 2>&1 || { printf 'check-msrv: --write needs node for write-file.mjs\n' >&2; exit 2; }
  [ -n "$DECLARED" ] || { printf 'check-msrv: --write cannot patch a field that is absent. Add it first.\n' >&2; exit 2; }
  find_b64=$(printf 'rust-version = "%s"' "$DECLARED" | base64 | tr -d '\n')
  repl_b64=$(printf 'rust-version = "%s"' "$FLOOR_VERSION" | base64 | tr -d '\n')
  # ⛔ One write path. write-file.mjs refuses a match count that differs from
  # what was declared and leaves the file untouched, which is what a silent
  # no-op reporting success would not do.
  node scripts/common/write-file.mjs replace Cargo.toml \
    --find-b64 "$find_b64" --replace-b64 "$repl_b64" --expect 1 || exit 2
  printf 'check-msrv: rust-version %s -> %s, derived from %s\n' "$DECLARED" "$FLOOR_VERSION" "$FLOOR_PACKAGE"
  printf '  Now read it back: sh scripts/common/check-msrv.sh\n'
  exit 0
fi

# -- the verify leg, which compiles ------------------------------------------
VERIFIED=0
if [ "$VERIFY" = 1 ]; then
  if [ -z "$DECLARED" ]; then
    printf 'check-msrv: --verify has nothing to verify: no rust-version is declared.\n' >&2
    exit 1
  fi
  command -v rustup >/dev/null 2>&1 || { printf 'check-msrv: --verify needs rustup to select a toolchain\n' >&2; exit 2; }
  # ⛔ BOTH BINARIES, NOT CARGO ALONE. Measured here on 2026-08-31: an install
  # killed part-way registers the toolchain and leaves a working `cargo` beside
  # a rustc with no manifest. A guard that probed cargo alone let that through,
  # and `cargo check` then failed on `rustc -vV`, which this script reported as
  # "the workspace does NOT compile". ⚠ That is a broken host accusing the tree,
  # which is the exact confusion between "failed" and "could not run" that the
  # three exit codes exist to keep apart.
  if ! rustup run "$DECLARED" cargo --version >/dev/null 2>&1 ||
     ! rustup run "$DECLARED" rustc --version >/dev/null 2>&1; then
    printf 'check-msrv: toolchain %s is not installed, or is installed incompletely.\n' "$DECLARED" >&2
    printf '  That is "could not run", not "failed". Install it and re-run:\n' >&2
    printf '    rustup toolchain install %s\n' "$DECLARED" >&2
    exit 2
  fi
  if ! v_out=$(rustup run "$DECLARED" cargo check --workspace --all-targets 2>&1); then
    printf 'check-msrv: the workspace does NOT compile on the declared %s.\n' "$DECLARED" >&2
    printf '%s\n' "$v_out" | sed 's/^/  | /' >&2
    exit 1
  fi
  VERIFIED=1
fi

# -- the verdict -------------------------------------------------------------
PROBLEMS=0
REPORT=""
add() { REPORT="$REPORT$1
"; PROBLEMS=$((PROBLEMS + 1)); }

if [ -z "$DECLARED" ]; then
  add "  Cargo.toml: [workspace.package] declares no rust-version. An absent one is not 'any version': it is a promise nobody made."
elif [ -n "$FLOOR_VERSION" ]; then
  cmp=$(awk -v a="$DECLARED" -v b="$FLOOR_VERSION" '
    function num(v,   p) { split(v, p, "."); return (p[1] + 0) * 1000000 + (p[2] + 0) * 1000 + (p[3] + 0) }
    BEGIN { print (num(a) < num(b)) ? "low" : "ok" }
  ')
  if [ "$cmp" = low ]; then
    add "  Cargo.toml: rust-version is $DECLARED, and the dependency graph needs $FLOOR_VERSION ($FLOOR_PACKAGE). Derive it: sh scripts/common/check-msrv.sh --write"
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-msrv/1","declared":"%s","graph_floor":"%s","packages":%s,"dependencies":%s,"verified":%s,"problems":%s}\n' \
    "$DECLARED" "$FLOOR_VERSION" "$PKG_TOTAL" "$DEP_TOTAL" "$VERIFIED" "$PROBLEMS"
  [ "$PROBLEMS" -gt 0 ] && exit 1
  exit 0
fi

if [ "$PROBLEMS" -gt 0 ]; then
  printf 'msrv check failed, %s problem(s):\n\n' "$PROBLEMS"
  printf '%s' "$REPORT"
  exit 1
fi

printf 'msrv ok: declared %s' "$DECLARED"
if [ -n "$FLOOR_VERSION" ]; then
  printf ', graph floor %s from %s' "$FLOOR_VERSION" "$FLOOR_PACKAGE"
else
  printf ', graph floor none (%s dependency package(s) resolved)' "$DEP_TOTAL"
fi
[ "$VERIFIED" = 1 ] && printf ', compiles on %s' "$DECLARED"
printf '\n'
exit 0
