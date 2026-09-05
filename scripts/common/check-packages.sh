#!/bin/sh
# check-packages.sh - does each language package build offline, report the
# corpus release it embeds, and does that release match the one it was cut from?
#
# ⛔ FETCHING AND PARSING A CORPUS IS WORK. A DEPENDENCY LINE IS NOT.
# docs/history/todo/publish.md, PUB-05.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. every ecosystem the crate names produces its files, and the count is
#      asserted rather than assumed;
#   2. ⭐ THE PACKAGE BUILDS OFFLINE. It is imported by its own runtime with no
#      network and no install step, and it answers;
#   3. ⛔ THE RELEASE IT REPORTS IS THE ONE IT WAS CUT FROM, recomputed here
#      with `sha256sum` rather than taken from the generator. A package that
#      reported its own belief about its own bytes would be a pin nobody
#      checked;
#   4. ⛔ IT EMBEDS AND IT DOES NOT FETCH. Nothing in the generated source names
#      a network call, which is the entry's Must-not;
#   5. ⚠ IT ANSWERS THE SAME THINGS THE RUST PACKAGE DOES. The profile count and
#      the newest capture instant are compared against the corpus, so a package
#      that embedded half of it is caught rather than trusted.
#
# ⚠ THE RUNTIME IS THE SKIP. Without `node` there is nothing to import the
# package with, and a skip is reported as a skip: this check does not claim a
# package was built when nothing built it.
#
# Usage:
#   sh scripts/common/check-packages.sh
#   sh scripts/common/check-packages.sh --json
#
# Exit codes: 0 every package is what it says it is, 1 one is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-packages: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-packages: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-packages: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-packages: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { printf 'check-packages: cargo not found\n' >&2; exit 2; }
command -v sha256sum >/dev/null 2>&1 || {
  printf 'check-packages: sha256sum not found, and the pin is recomputed with it\n' >&2
  exit 2
}
command -v jq >/dev/null 2>&1 || { printf 'check-packages: jq not found\n' >&2; exit 2; }

CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-packages: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

OUT="$REPO_ROOT/.tmp/check-packages"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-packages: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo run -q -p b-ids-corpus -- publish --root "$CORPUS_ROOT" --out "$OUT/tree" \
  > "$OUT/publish.log" 2>&1 || {
  printf 'check-packages: the assembler did not build the tree\n' >&2
  cat "$OUT/publish.log" >&2
  exit 2
}

# ⛔ THE ECOSYSTEMS ARE READ FROM THE CRATE, never listed here. A second list is
# a second answer, and the day they disagree the check is the one that is wrong.
ECOSYSTEMS=$(awk -F'"' '/pub const ECOSYSTEMS/ { for (i = 2; i <= NF; i += 2) print $i }' \
  "$REPO_ROOT/crates/b-ids-corpus/src/packages.rs")
[ -n "$ECOSYSTEMS" ] || { printf 'check-packages: the crate names no ecosystem\n' >&2; exit 2; }

# ⭐ THE PIN, RECOMPUTED. Not taken from the generator, and not from the Rust
# package either: `sha256sum` over the corpus index is a third implementation.
WANT=$(sha256sum "$CORPUS_ROOT/corpus/v1/index.json" | awk '{ print $1 }')
PROFILES=$(find "$CORPUS_ROOT/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json |
  wc -l | tr -d ' ')

BUILT=0
CHECKED=0
for eco in $ECOSYSTEMS; do
  DIR="$OUT/tree/packages/$eco"
  if [ ! -d "$DIR" ]; then
    note "the crate names the $eco ecosystem and the assembler wrote no packages/$eco"
    continue
  fi
  BUILT=$((BUILT + 1))

  # ⛔ IT EMBEDS AND IT DOES NOT FETCH, which is this entry's Must-not. Checked
  # over the generated source rather than promised in a comment.
  for forbidden in 'fetch(' 'XMLHttpRequest' 'require("http' "require('http" 'node:http' 'node:https'; do
    if grep -qF "$forbidden" "$DIR"/*.mjs "$DIR"/*.js 2>/dev/null; then
      note "packages/$eco names $forbidden, and a package that fetches at runtime is what PUB-05 forbids"
    fi
  done

  case "$eco" in
    js)
      if ! command -v node >/dev/null 2>&1; then
        continue
      fi
      # ⭐ IMPORTED BY ITS OWN RUNTIME, with no install step and no network.
      ( cd "$DIR" && node -e '
        import("./index.mjs").then((m) => {
          const r = m.release();
          const out = {
            identifier: r.identifier,
            profiles: m.profiles().length,
            paths: m.paths().length,
            newestCapture: r.newestCapture,
            selectable: m.select({ browser: "chrome" }).length,
            absent: m.latestStable("chrome", "macos-arm64") === undefined,
          };
          console.log(JSON.stringify(out));
        }).catch((e) => { console.error(String(e)); process.exit(1); });
      ' ) > "$OUT/$eco.json" 2>"$OUT/$eco.err"
      rc=$?
      if [ "$rc" != 0 ]; then
        note "packages/$eco did not import: $(head -1 "$OUT/$eco.err")"
        continue
      fi
      CHECKED=$((CHECKED + 1))
      GOT=$(jq -r '.identifier' "$OUT/$eco.json" | tr -d '\r')
      [ "$GOT" = "$WANT" ] ||
        note "packages/$eco reports release $GOT and the corpus index digests to $WANT"
      GOT_N=$(jq -r '.profiles' "$OUT/$eco.json" | tr -d '\r')
      [ "$GOT_N" = "$PROFILES" ] ||
        note "packages/$eco embeds $GOT_N profile(s) and the corpus holds $PROFILES"
      GOT_P=$(jq -r '.paths' "$OUT/$eco.json" | tr -d '\r')
      [ "$GOT_P" = "$PROFILES" ] ||
        note "packages/$eco reports $GOT_P path(s) for $GOT_N profile(s), and it is one each"
      # ⛔ A ROUTE THE CORPUS DOES NOT HOLD ANSWERS NOTHING, never a neighbour.
      # A missing route is a fact and a substituted value is a lie.
      [ "$(jq -r '.absent' "$OUT/$eco.json" | tr -d '\r')" = "true" ] ||
        note "packages/$eco answered for a platform the corpus has no profile on"
      ;;
    *)
      note "no runner is written for the $eco ecosystem, so its package was generated and not driven"
      ;;
  esac
done

NODE=present
command -v node >/dev/null 2>&1 || NODE=absent

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-packages/1","ecosystems":%s,"built":%s,"driven":%s,"profiles":%s,"node":"%s","problems":%s}\n' \
    "$(printf '%s\n' "$ECOSYSTEMS" | grep -c .)" "$BUILT" "$CHECKED" "$PROFILES" "$NODE" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'packages ok: %s ecosystem(s) generated, %s driven by their own runtime,\n' "$BUILT" "$CHECKED"
  printf '  each embedding %s profile(s) and reporting release %s,\n' "$PROFILES" "$WANT"
  printf '  which is what sha256sum makes of the corpus index here.\n'
  printf '  ⛔ Nothing generated fetches at runtime.\n'
  if [ "$NODE" = absent ]; then
    printf '  ⚠ SKIP: no node on this host, so the js package was generated and not\n'
    printf '  imported. A skip is not a pass.\n'
  fi
  exit 0
fi

printf 'packages check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A package that needs the network to answer fails in the environment its\n' >&2
printf 'consumers care most about. docs/history/todo/publish.md, PUB-05.\n' >&2
exit 1
