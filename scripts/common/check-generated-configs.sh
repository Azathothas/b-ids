#!/bin/sh
# check-generated-configs.sh - is every generated snippet for a pair the support
# matrix says can emit, and does every hole get a named refusal instead?
#
# ⛔ MOST OF THE DAY-TO-DAY VALUE OF A CORPUS IS THE ARTEFACT SOMEBODY PASTES
# INTO THEIR OWN TOOL, and a snippet that silently approximates is worse than no
# snippet: it produces a client that is almost right, which is more
# distinguishing than an honestly old one. docs/history/todo/publish.md, PUB-04.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the tree is GENERATED here, by running the assembler, rather than read
#      from a committed copy that could go stale;
#   2. ⛔ EVERY SNIPPET IS FOR A PAIR THE MATRIX MARKS EMITTABLE. A snippet for
#      a stack with a hole is the defect this check exists for;
#   3. ⛔ EVERY HOLE STACK GETS A REFUSAL AND NO SNIPPET, and the refusal names
#      the stack, what it cannot do, and the file and line that was read at;
#   4. every published profile has a directory, so a profile the generator
#      quietly skipped is a finding rather than an absence nobody counted;
#   5. ⭐ THERE IS AT LEAST ONE REFUSAL. A configs tree with none is one that
#      generated a snippet for every stack, which is the thing being refused.
#
# -- ⚠ WHY THE DETECTION FILE IS NOT MATRIX-GATED -----------------------------
#
# A detection rule is built from values the corpus HOLDS: the header names and
# their order, and the ALPN list. It emits nothing, so no stack has to be able
# to emit it and the matrix has nothing to say about it. ⛔ It is still checked,
# for the opposite property: that it names no digest, because the corpus holds
# none and a value computed at generation time is what PUB-03's ruling declined.
#
# Usage:
#   sh scripts/common/check-generated-configs.sh
#   sh scripts/common/check-generated-configs.sh --json
#
# Exit codes: 0 every snippet is gated, 1 one is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-generated-configs: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-generated-configs: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-generated-configs: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-generated-configs: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED, like every other check that
# reads it. docs/history/todo/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-generated-configs: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

[ -d "$CORPUS_ROOT/corpus" ] || {
  printf 'check-generated-configs: there is no corpus, so nothing was generated\n' >&2
  exit 2
}
command -v cargo >/dev/null 2>&1 || { printf 'check-generated-configs: cargo not found\n' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { printf 'check-generated-configs: jq not found\n' >&2; exit 2; }

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

OUT="$REPO_ROOT/.tmp/check-generated-configs"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-generated-configs: cannot create %s\n' "$OUT" >&2; exit 2; }

# -- generate, rather than read a committed copy ------------------------------
cargo run -q -p b-ids-corpus -- publish --root "$CORPUS_ROOT" --out "$OUT/tree" \
  > "$OUT/publish.log" 2>&1
rc=$?
if [ "$rc" != 0 ]; then
  printf 'check-generated-configs: the assembler exited %s\n' "$rc" >&2
  cat "$OUT/publish.log" >&2
  exit 1
fi

CONFIGS="$OUT/tree/configs"
[ -d "$CONFIGS" ] || {
  printf 'check-generated-configs: the assembler produced no configs/ directory\n' >&2
  exit 1
}

# -- the matrix, generated the same way check-support-matrix generates it -----
cargo build -q -p b-ids-cli || {
  printf 'check-generated-configs: the client did not build\n' >&2
  exit 2
}
TARGET_DIR=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
BIN="$TARGET_DIR/debug/b-ids-cli"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-generated-configs: %s is not executable\n' "$BIN" >&2; exit 2; }

"$BIN" --matrix > "$OUT/matrix.json" 2> "$OUT/matrix.err"
rc=$?
if [ "$rc" != 0 ]; then
  printf 'check-generated-configs: the matrix generator exited %s\n' "$rc" >&2
  cat "$OUT/matrix.err" >&2
  exit 1
fi

# The stacks the matrix records a hole against, and the one it can run.
jq -r '.holes[].stack' "$OUT/matrix.json" | sort -u > "$OUT/hole-stacks.txt"
RUNNABLE=$(jq -r '.cells[0].stack // empty' "$OUT/matrix.json")
[ -n "$RUNNABLE" ] || { printf 'check-generated-configs: the matrix has no cell to read a stack from\n' >&2; exit 2; }

SNIPPETS=0
REFUSALS=0
DETECT=0

# -- every generated file, classified by its own name -------------------------
#
# ⚠ find rather than a glob: a recursive glob is not POSIX and the tree is four
# directories deep.
for file in $(find "$CONFIGS" -type f | sort); do
  base=$(basename "$file")
  case "$base" in
    README.md) continue ;;
    detect.conf)
      DETECT=$((DETECT + 1))
      # ⛔ THE ONE THING A DETECTION RULE MUST NOT CARRY. The corpus holds no
      # digest, and PUB-03's ruling declined a route that resolved to one.
      if grep -qiE '\bja4|\bja3|sha256:|digest' "$file"; then
        note "$base names a digest, and the corpus holds none: ${file#"$CONFIGS"/}"
      fi
      continue
      ;;
  esac

  stack="${base%.*}"
  ext="${base##*.}"

  if grep -qx "$stack" "$OUT/hole-stacks.txt"; then
    # ⛔ A HOLE STACK GETS A REFUSAL AND NEVER A SNIPPET.
    REFUSALS=$((REFUSALS + 1))
    if [ "$ext" = "rs" ]; then
      note "$stack has a hole in the matrix and a snippet in the tree: ${file#"$CONFIGS"/}"
      continue
    fi
    grep -q 'NO SNIPPET IS GENERATED' "$file" ||
      note "$stack's refusal does not say it is one: ${file#"$CONFIGS"/}"
    # ⛔ The file and the line, because a refusal a reader cannot check is one
    # they will assume is out of date.
    grep -qE 'read at references/[^:]+:[0-9]+' "$file" ||
      note "$stack's refusal names no file and line: ${file#"$CONFIGS"/}"
  elif [ "$stack" = "$RUNNABLE" ]; then
    SNIPPETS=$((SNIPPETS + 1))
    # ⛔ THE PAIR, NOT THE STACK. A cell for one profile does not license a
    # snippet for another, and the profile is what the file names.
    id=$(awk 'NR<=8 && /profile  */ { print $3; exit }' "$file")
    if [ -z "$id" ]; then
      note "$stack's snippet names no profile: ${file#"$CONFIGS"/}"
    else
      emits=$(jq -r --arg s "$stack" --arg p "$id" \
        '[.cells[] | select(.stack == $s and .profile == $p and .emits)] | length' \
        "$OUT/matrix.json")
      [ "$emits" = "1" ] ||
        note "$stack has a snippet for $id and the matrix has no emitting cell for that pair"
    fi
  else
    note "there is a file for stack $stack, which the matrix has neither a cell nor a hole for"
  fi
done

# -- every published profile has a directory ----------------------------------
PROFILES=$(find "$CORPUS_ROOT/corpus" -name '*.json' \
  ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')
DIRS=$(find "$CONFIGS" -mindepth 4 -maxdepth 4 -type d 2>/dev/null | wc -l | tr -d ' ')
[ "$PROFILES" = "$DIRS" ] ||
  note "the corpus has $PROFILES profile(s) and the tree has $DIRS configuration director(y/ies)"

# ⭐ AT LEAST ONE REFUSAL. A tree with none generated a snippet for every stack.
[ "$REFUSALS" -gt 0 ] ||
  note "no stack was refused, so nothing here was gated on anything"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-generated-configs/1","snippets":%s,"refusals":%s,"detection":%s,"profiles":%s,"problems":%s}\n' \
    "$SNIPPETS" "$REFUSALS" "$DETECT" "$PROFILES" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" != 0 ]; then
  printf 'generated configs check failed, %s problem(s):\n\n' "$COUNT" >&2
  printf '%s\n' "$PROBLEMS" >&2
  printf 'A snippet is generated only where the support matrix says the pair can\n' >&2
  printf 'emit. Fix the generator, never this check.\n' >&2
  exit 1
fi

printf 'generated configs ok: %s snippet(s) over %s profile(s), each for a pair the\n' \
  "$SNIPPETS" "$PROFILES"
printf '  matrix marks emittable, and %s refusal(s) naming a hole at a file and a line.\n' "$REFUSALS"
printf '  %s detection rule(s), none naming a digest the corpus does not hold.\n' "$DETECT"
exit 0
