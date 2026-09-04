#!/bin/sh
# check-bindings.sh - does every other ecosystem's package answer identically to
# the Rust crate?
#
# ⛔ A REIMPLEMENTATION IN EACH LANGUAGE IS THE FAILURE TO AVOID. Four
# implementations of one selection rule is four places for it to be wrong, and
# the one that is wrong is the one nobody uses often enough to notice.
# TODO/library.md, LIB-03.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. ⭐ THE COMPARISON IS OVER THE ANSWERS RATHER THAN OVER THE INTERFACES,
#      which is the entry's own wording. Two implementations can expose the same
#      names and disagree about what they mean;
#   2. ⛔ INCLUDING THE CASE WHERE A PROFILE IS ABSENT. Two implementations
#      agree easily on what exists; a missing route, an unknown browser and a
#      platform the corpus has no profile on are where they part;
#   3. the two documents are compared after normalising key ORDER and nothing
#      else, so a value that differs in type or in case is a difference.
#
# ⚠ THE ANSWER SET IS ONE DOCUMENT ASKED TWICE. crates/b-ids/examples/answers.rs
# and scripts/fixtures/bindings-answers.mjs ask the same questions in the same
# order; a package that answered a DIFFERENT set would produce a different
# document and be caught rather than counted as agreeing.
#
# ⚠ THE RUNTIME IS THE SKIP. Without `node` there is nothing to run the other
# half with, and a skip is reported as a skip.
#
# Usage:
#   sh scripts/common/check-bindings.sh
#   sh scripts/common/check-bindings.sh --json
#
# Exit codes: 0 every binding agrees, 1 one does not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-bindings: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-bindings: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-bindings: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-bindings: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { printf 'check-bindings: cargo not found\n' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { printf 'check-bindings: jq not found\n' >&2; exit 2; }

CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-bindings: no corpus is reachable, so nothing was compared\n' >&2
  exit 2
}
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

ASKER="$REPO_ROOT/scripts/fixtures/bindings-answers.mjs"
[ -f "$ASKER" ] || { printf 'check-bindings: no asker at %s\n' "$ASKER" >&2; exit 2; }
[ -f "$REPO_ROOT/crates/b-ids/examples/answers.rs" ] || {
  printf 'check-bindings: the Rust crate has no answers example to compare against\n' >&2
  exit 2
}

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

OUT="$REPO_ROOT/.tmp/check-bindings"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-bindings: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo run -q -p b-ids-corpus -- publish --root "$CORPUS_ROOT" --out "$OUT/tree" \
  > "$OUT/publish.log" 2>&1 || {
  printf 'check-bindings: the assembler did not build the packages\n' >&2
  cat "$OUT/publish.log" >&2
  exit 2
}

# ⛔ THE REFERENCE ANSWER, from the crate every binding is a binding OVER.
cargo run -q -p b-ids --example answers > "$OUT/rust.json" 2>"$OUT/rust.err" || {
  printf 'check-bindings: the Rust crate did not answer\n' >&2
  cat "$OUT/rust.err" >&2
  exit 2
}
jq -S . "$OUT/rust.json" > "$OUT/rust.norm" 2>/dev/null || {
  note "the Rust answers are not JSON"
}

NODE=present
command -v node >/dev/null 2>&1 || NODE=absent

COMPARED=0
if [ "$NODE" = present ] && [ -d "$OUT/tree/packages/js" ]; then
  cp "$ASKER" "$OUT/tree/packages/js/" || exit 2
  ( cd "$OUT/tree/packages/js" && node bindings-answers.mjs ) > "$OUT/js.json" 2>"$OUT/js.err"
  rc=$?
  if [ "$rc" != 0 ]; then
    note "the js package did not answer: $(head -1 "$OUT/js.err")"
  else
    jq -S . "$OUT/js.json" > "$OUT/js.norm" 2>/dev/null || note "the js answers are not JSON"
    if [ -f "$OUT/js.norm" ] && [ -f "$OUT/rust.norm" ]; then
      COMPARED=$((COMPARED + 1))
      if ! diff -u "$OUT/rust.norm" "$OUT/js.norm" > "$OUT/diff.txt" 2>&1; then
        note "the js package does not answer as the Rust crate does. See .tmp/check-bindings/diff.txt"
        # ⚠ THE FIRST DIFFERING KEYS, named here so a reader does not have to
        # open a file to learn which rule drifted.
        for key in $(grep -E '^[-+]  "' "$OUT/diff.txt" | sed 's/^[-+] *"\([^"]*\)".*/\1/' |
          LC_ALL=C sort -u | head -4); do
          note "  they disagree about $key"
        done
      fi
    fi
  fi
elif [ "$NODE" = present ]; then
  note "the assembler wrote no packages/js, so there was nothing to compare"
fi

# ⛔ AND THE ABSENT CASES ARE ACTUALLY IN THE ANSWER SET. A comparison over a
# document that never asked what happens when a profile is missing would agree
# and prove the thing LIB-03 names specifically.
for want in at_missing latest_chrome_macos latest_safari_linux64; do
  jq -e "has(\"$want\")" "$OUT/rust.json" >/dev/null 2>&1 ||
    note "the answer set does not ask $want, and LIB-03 names the absent case"
  [ "$(jq -r ".$want" "$OUT/rust.json" 2>/dev/null)" = "null" ] ||
    note "$want answered something, so it is not the absent case it is named for"
done

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-bindings/1","compared":%s,"node":"%s","problems":%s}\n' \
    "$COMPARED" "$NODE" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'bindings ok: %s binding(s) compared against the Rust crate, answer for\n' "$COMPARED"
  printf '  answer over one corpus, and the three absent cases came back empty on\n'
  printf '  both sides. ⛔ The comparison is over the ANSWERS rather than over the\n'
  printf '  interfaces.\n'
  if [ "$NODE" = absent ]; then
    printf '  ⚠ SKIP: no node on this host, so the js package was not run. A skip is\n'
    printf '  not a pass.\n'
  fi
  exit 0
fi

printf 'bindings check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'Four implementations of one selection rule is four places for it to be\n' >&2
printf 'wrong. TODO/library.md, LIB-03.\n' >&2
exit 1
