#!/bin/sh
# check-vendor.sh - does vendor/upstream.json still describe the vendored trees, and has
# upstream moved past what it records?
#
# The defect this exists to catch is a vendored tree nobody can reconcile. A tree with no recorded
# commit is a fork whose base is lost: the next release cannot be merged onto it, no patch can be
# said to be a diff from anything, and "is this still upstream's code" has no answer. That is the
# state every vendored directory drifts into unless something asserts the record.
#
# -- WHAT IT ASSERTS, OFFLINE ------------------------------------------------
#   - the manifest parses and declares a schema version;
#   - every upstream names a repository, a ref, a 40-hex base and an ISO 8601 UTC instant;
#   - every upstream's directory exists and is not empty;
#   - every EXCLUDED path is absent from that directory, so the exclude list stays true;
#   - every crate the manifest names resolves to a Cargo.toml declaring that name;
#   - every directory under vendor/ has a manifest entry, so a tree cannot be added silently;
#   - every patches/NAME directory names an upstream, every patch in it names a file the tree
#     still has, and patches/README.md carries a section naming that patch.
#
# -- ⚠ TWO LEGS, AND ONLY ONE OF THEM IS IN THE GATE -------------------------
# --upstream fetches the recorded ref from the remote and reports whether it still resolves to the
# recorded base, and which newer release tags exist. It needs the network, and a gate that needs
# the network fails on a machine that has none. check-msrv.sh has the same shape and the same
# reason.
#
# ⛔ A MOVED REF IS REPORTED, NOT FOLLOWED. Reconciling a release is a reading, and
# docs/methodology/vendoring.md says what it owes.
#
# Usage:
#   sh scripts/common/check-vendor.sh
#   sh scripts/common/check-vendor.sh --json
#   sh scripts/common/check-vendor.sh --upstream
#
# Exit codes: 0 consistent, 1 inconsistent, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

# ⛔ ONE SUBSTITUTION, NOT ONE PER LINE READ. An assignment prefix on a
# `while ... read` is re-evaluated on EVERY iteration, so `IFS="$(printf
# '\t')" read ...` forks once per line. Measured 2026-09-02: a command
# substitution costs 35 ms on this host, and check-docs.sh reads about 1100
# lines that way. docs/history/todo/tooling.md, TOOL-18.
TAB=$(printf '\t')

JSON=0
UPSTREAM=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --upstream) UPSTREAM=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-vendor: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-vendor: git not found\n' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { printf 'check-vendor: jq not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-vendor: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ Every path below is relative to the repository root. A run from a subdirectory would otherwise
# scope itself to that subtree and report clean over everything else.
cd "$REPO_ROOT" || { printf 'check-vendor: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

MANIFEST="vendor/upstream.json"

# ⚠ NO MANIFEST IS EXIT 2, NOT EXIT 0. A tree that vendors nothing has neither broken these rules
# nor satisfied them, and reporting green over an absent file is how a check quietly stops
# applying. check-changelog.sh carries the same distinction.
[ -f "$MANIFEST" ] || { printf 'check-vendor: no manifest at %s\n' "$MANIFEST" >&2; exit 2; }
jq -e . "$MANIFEST" >/dev/null 2>&1 || { printf 'check-vendor: %s does not parse\n' "$MANIFEST" >&2; exit 2; }

PROBLEMS=""
COUNT=0
report() { PROBLEMS="$PROBLEMS  $1
"; COUNT=$((COUNT + 1)); }

SCHEMA=$(jq -r '.schema_version // empty' "$MANIFEST")
[ -n "$SCHEMA" ] || report "$MANIFEST: no schema_version. A positional format with no version mis-reads silently."

# ⛔ THE CARRIAGE RETURN IS STRIPPED, AND A SECOND UPSTREAM IS WHAT EXPOSED THAT.
# jq on Windows writes CRLF, which this project has now been bitten by four
# times. ⚠ THE SHAPE HERE IS THE NASTY ONE: a command substitution strips the
# trailing line ending, so the LAST name in the list comes out clean and every
# name before it carries a `\r`. With one upstream in the manifest this loop was
# correct for eighteen months of sessions; the day a second one landed, the
# FIRST reported five problems it does not have and the second reported none.
# ⭐ A latent defect that only a second element could ever show.
# docs/history/todo/emitters.md, EMIT-03, and docs/history/todo/vendor.md, VENDOR-01.
NAMES=$(jq -r '.upstreams[]?.name' "$MANIFEST" | tr -d '\r')
NUPSTREAMS=0
NCRATES=0
NPATCHES=0

for name in $NAMES; do
  NUPSTREAMS=$((NUPSTREAMS + 1))
  repo=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .repository // empty' "$MANIFEST")
  dir=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .directory // empty' "$MANIFEST")
  ref=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .ref // empty' "$MANIFEST")
  base=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .base // empty' "$MANIFEST")
  at=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .vendored_at // empty' "$MANIFEST")

  case "$repo" in
    https://*) ;;
    *) report "$name: repository is not an https URL: ${repo:-none}" ;;
  esac
  [ -n "$ref" ] || report "$name: no ref. Without one nothing can fetch the tree again."
  printf '%s' "$base" | grep -qE '^[0-9a-f]{40}$' \
    || report "$name: base is not a 40-character commit: ${base:-none}"
  printf '%s' "$at" | grep -qE '^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$' \
    || report "$name: vendored_at is not ISO 8601 UTC: ${at:-none}"

  if [ -z "$dir" ]; then
    report "$name: no directory"
    continue
  fi
  if [ ! -d "$dir" ]; then
    report "$name: the manifest names $dir and it does not exist"
    continue
  fi
  if [ -z "$(find "$dir" -type f -print -quit 2>/dev/null)" ]; then
    report "$name: $dir holds no file"
  fi

  # ⛔ An excluded path that is PRESENT means the tree is not what the manifest says it is, and
  # every reconciliation from here compares against the wrong set.
  # ⚠ CARRIAGE RETURNS STRIPPED. jq on this host writes CRLF, and a path read through a
  # pipe keeps the CR: vendor/rustls/rustls\r/Cargo.toml does not exist and the check reported a
  # missing crate over a tree that was correct. Command substitution hides it and a pipe does not.
  jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .exclude[]?' "$MANIFEST" | tr -d '\r' \
  | while IFS= read -r ex; do
      [ -n "$ex" ] || continue
      [ -e "$dir/$ex" ] && printf 'EXCLUDED\t%s\t%s\n' "$name" "$ex"
      :
    done > .check-vendor-excluded.$$
  while IFS="$TAB" read -r _kind n ex; do
    [ -n "${ex:-}" ] || continue
    report "$n: $ex is listed as excluded and is present in $dir"
  done < .check-vendor-excluded.$$
  rm -f .check-vendor-excluded.$$

  # A crate the manifest names has to be a real package with that name, or the record points at
  # something nobody can depend on.
  jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .crates // {} | to_entries[] | .key + "\t" + .value' "$MANIFEST" | tr -d '\r' \
    > .check-vendor-crates.$$
  while IFS="$TAB" read -r crate path; do
    [ -n "${crate:-}" ] || continue
    NCRATES=$((NCRATES + 1))
    manifest_path="$dir/$path/Cargo.toml"
    if [ ! -f "$manifest_path" ]; then
      report "$name: crate $crate is recorded at $path and there is no Cargo.toml there"
      continue
    fi
    grep -qE "^name[[:space:]]*=[[:space:]]*\"$crate\"" "$manifest_path" \
      || report "$name: $manifest_path does not declare name = \"$crate\""
  done < .check-vendor-crates.$$
  rm -f .check-vendor-crates.$$
done

# ⛔ Every tree under vendor/ has a record. A directory added without one is a fork with no base,
# which is the state this check exists to make impossible.
if [ -d vendor ]; then
  for d in vendor/*/; do
    [ -d "$d" ] || continue
    got=$(printf '%s' "$d" | sed 's:/*$::')
    jq -e --arg d "$got" '.upstreams[] | select(.directory == $d)' "$MANIFEST" >/dev/null 2>&1 \
      || report "$got exists and no upstream in $MANIFEST names it"
  done
fi

# The patch series is derived, so a patch naming a file the tree no longer has is a claim about
# the tree that is not true. ⚠ This is the OFFLINE half of that question; whether the series still
# regenerates identically needs a pristine copy and belongs to vendor-diff.mjs --check.
if [ -d patches ]; then
  for d in patches/*/; do
    [ -d "$d" ] || continue
    pname=$(basename "$d")
    dir=$(jq -r --arg n "$pname" '.upstreams[] | select(.name == $n) | .directory // empty' "$MANIFEST")
    if [ -z "$dir" ]; then
      report "patches/$pname has no upstream named $pname in $MANIFEST"
      continue
    fi
    for p in "$d"*.patch; do
      [ -f "$p" ] || continue
      NPATCHES=$((NPATCHES + 1))
      target=$(awk '/^\+\+\+ b\// { sub(/^\+\+\+ b\//, ""); print; exit }' "$p")
      if [ -z "$target" ]; then
        report "$p names no target file"
      elif [ "$target" != "/dev/null" ] && [ ! -e "$dir/$target" ]; then
        report "$p patches $target and $dir/$target does not exist"
      fi
      if [ -f patches/README.md ]; then
        grep -qF "$(basename "$p")" patches/README.md \
          || report "$p has no section in patches/README.md saying what it is for"
      else
        report "patches/README.md does not exist, so no local change has a reason recorded"
      fi
    done
  done
fi

# -- ⚠ the network leg, which is not in the gate -----------------------------
MOVED=0
if [ "$UPSTREAM" = "1" ]; then
  for name in $NAMES; do
    repo=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .repository' "$MANIFEST")
    ref=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .ref' "$MANIFEST")
    base=$(jq -r --arg n "$name" '.upstreams[] | select(.name == $n) | .base' "$MANIFEST")
    # ⛔ Dereferenced with ^{}, because an annotated tag resolves to the TAG OBJECT and comparing
    # that to a commit reports a move that did not happen.
    now=$(git ls-remote "$repo" "refs/tags/$ref^{}" "refs/tags/$ref" "refs/heads/$ref" 2>/dev/null \
      | awk 'NR==1 { print $1 }')
    if [ -z "$now" ]; then
      printf 'upstream %s: ref %s does not resolve at %s\n' "$name" "$ref" "$repo"
      MOVED=$((MOVED + 1))
      continue
    fi
    if [ "$now" != "$base" ]; then
      printf 'upstream %s: ref %s now resolves to %s, recorded %s\n' "$name" "$ref" "$now" "$base"
      MOVED=$((MOVED + 1))
    else
      printf 'upstream %s: ref %s still resolves to the recorded base\n' "$name" "$ref"
    fi
    mine=$(printf '%s' "$ref" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | tail -1)
    if [ -n "$mine" ]; then
      newer=$(git ls-remote --tags "$repo" 2>/dev/null \
        | awk '{ print $2 }' | sed 's:refs/tags/::; s:\^{}$::' | sort -u \
        | grep -E '^[^ ]*[0-9]+\.[0-9]+\.[0-9]+$' \
        | while IFS= read -r t; do
            v=$(printf '%s' "$t" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | tail -1)
            [ -n "$v" ] || continue
            if [ "$v" != "$mine" ] && [ "$(printf '%s\n%s\n' "$mine" "$v" | sort -V | tail -1)" = "$v" ]; then
              printf '%s\n' "$t"
            fi
          done | sort -V)
      if [ -n "$newer" ]; then
        printf 'upstream %s: %s newer release tag(s), newest %s\n' \
          "$name" "$(printf '%s\n' "$newer" | wc -l | tr -d ' ')" "$(printf '%s\n' "$newer" | tail -1)"
        MOVED=$((MOVED + 1))
      else
        printf 'upstream %s: no newer release tag\n' "$name"
      fi
    fi
  done
  printf '\n'
fi

# -- report ------------------------------------------------------------------
if [ "$JSON" = "1" ]; then
  printf '{"schema":"check-vendor/1","problems":%s,"upstreams":%s,"crates":%s,"patches":%s,"moved":%s}\n' \
    "$COUNT" "$NUPSTREAMS" "$NCRATES" "$NPATCHES" "$MOVED"
  [ "$COUNT" -gt 0 ] && exit 1
  exit 0
fi

if [ "$COUNT" -gt 0 ]; then
  printf 'vendor check failed, %s problem(s):\n\n' "$COUNT"
  printf '%s\n' "$PROBLEMS"
  printf 'The rules are in docs/methodology/vendoring.md. The manifest is\n'
  printf 'vendor/upstream.json and the patch record is patches/README.md.\n'
  exit 1
fi

printf 'vendor ok: %s upstream(s), %s crate(s), %s patch(es), manifest agrees with the tree\n' \
  "$NUPSTREAMS" "$NCRATES" "$NPATCHES"
exit 0
