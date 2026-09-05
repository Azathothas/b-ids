#!/bin/sh
# check-support-matrix.sh - is every cell in the support matrix produced by a
# run, and does every hole still point at something?
#
# ⛔ A CLIENT AUTHOR CURRENTLY FINDS OUT WHICH STACK CAN EMIT WHICH PROFILE BY
# BUILDING IT. A published table would answer that, and a published table
# somebody maintains by hand goes stale the day a hole closes and nobody
# notices. docs/history/todo/emitters.md, EMIT-01.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the matrix is GENERATED, by running the generator here rather than by
#      reading a committed file. There is no committed matrix to go stale;
#   2. ⛔ EVERY CELL IS EVIDENCE `run`, and carries the command that reproduces
#      it. A cell filled any other way is a hole wearing a cell's clothes;
#   3. ⛔ EVERY HOLE IS EVIDENCE `read`, names a path under references/ and a
#      line, AND THAT PATH AND LINE STILL RESOLVE. A citation nobody resolves is
#      the defect TOOL-10 exists for, and a reference tree moves when it is
#      re-mined;
#   4. every published profile has a cell, so a profile the generator quietly
#      skipped is a finding rather than an absence nobody counted;
#   5. ⭐ THERE IS AT LEAST ONE HOLE. "Let it have holes" is the entry's rule,
#      and a matrix with none is one nobody filled honestly.
#
# -- ⚠ WHY A HOLE IS NOT A CELL ----------------------------------------------
#
# This tree can RUN exactly one emitter: its own. Every other stack was READ, at
# a file and a line, in a tree this repository holds at a named commit. Those
# are different kinds of knowledge and the matrix keeps them apart, because a
# row that said "cannot" with no way to re-check it is a claim rather than a
# finding.
#
# Usage:
#   sh scripts/common/check-support-matrix.sh
#   sh scripts/common/check-support-matrix.sh --json
#
# Exit codes: 0 the matrix is what it claims, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-support-matrix: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-support-matrix: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-support-matrix: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-support-matrix: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. docs/history/todo/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-support-matrix: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"
command -v cargo >/dev/null 2>&1 || { printf 'check-support-matrix: cargo not found\n' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { printf 'check-support-matrix: jq not found\n' >&2; exit 2; }

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

OUT="$REPO_ROOT/.tmp/check-support-matrix"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-support-matrix: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo build -q -p b-ids-cli || {
  printf 'check-support-matrix: the client did not build\n' >&2
  exit 2
}
TARGET_DIR=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
BIN="$TARGET_DIR/debug/b-ids-cli"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-support-matrix: %s is not executable\n' "$BIN" >&2; exit 2; }

# ⛔ GENERATED, NEVER READ FROM A COMMITTED FILE. READ FROM THE PROCESS, UNPIPED.
"$BIN" --matrix > "$OUT/matrix.json" 2> "$OUT/matrix.err"
rc=$?
if [ "$rc" != 0 ]; then
  printf 'check-support-matrix: the generator exited %s\n' "$rc" >&2
  cat "$OUT/matrix.err" >&2
  exit 1
fi
jq -e . "$OUT/matrix.json" > /dev/null 2>&1 ||
  { printf 'check-support-matrix: the generator did not emit json\n' >&2; exit 1; }

SCHEMA=$(jq -r '.schema' "$OUT/matrix.json")
[ "$SCHEMA" = "emit-support-matrix/1" ] ||
  note "the matrix names schema $SCHEMA"

CELLS=$(jq '.cells | length' "$OUT/matrix.json")
HOLES=$(jq '.holes | length' "$OUT/matrix.json")

# -- 2: every cell is a run, with a command ----------------------------------
TYPED=$(jq '[.cells[] | select(.evidence != "run")] | length' "$OUT/matrix.json")
[ "$TYPED" = 0 ] ||
  note "$TYPED cell(s) are not evidence run, and a cell filled any other way is a hole wearing a cell's clothes"
NOCMD=$(jq '[.cells[] | select((.reproduce // "") == "")] | length' "$OUT/matrix.json")
[ "$NOCMD" = 0 ] ||
  note "$NOCMD cell(s) name no command that reproduces them"

# -- 3: every hole is a reading whose citation still resolves ----------------
#
# ⛔ THE PATH AND THE LINE, both. A file that shrank below the line it is cited
# at is a citation that has stopped pointing at anything.
[ "$HOLES" -ge 1 ] ||
  note "the matrix declares no hole at all, and a matrix with none is one nobody filled honestly"
RESOLVED=0
jq -r '.holes[] | [.stack, .evidence, .file, (.line|tostring)] | @tsv' "$OUT/matrix.json" > "$OUT/holes.tsv"
while IFS="$(printf '\t')" read -r stack evidence file line; do
  [ -n "${stack:-}" ] || continue
  [ "$evidence" = read ] ||
    note "$stack: a hole is evidence $evidence, and a hole is a reading"
  case "$file" in
    references/*) ;;
    *) note "$stack: $file is not under references/, so nothing holds it at a named commit" ;;
  esac
  if [ ! -f "$file" ]; then
    note "$stack: $file does not exist, so the evidence for this hole no longer resolves"
    continue
  fi
  have=$(wc -l < "$file" | tr -d ' ')
  if [ "$have" -lt "$line" ]; then
    note "$stack: $file has $have line(s) and the hole cites line $line"
    continue
  fi
  RESOLVED=$((RESOLVED + 1))
done < "$OUT/holes.tsv"

# -- 4: every published profile has a cell -----------------------------------
PROFILES=$(find "$CORPUS_ROOT/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')
[ "$CELLS" = "$PROFILES" ] ||
  note "the matrix carries $CELLS cell(s) over $PROFILES published profile(s)"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-support-matrix/1","cells":%s,"holes":%s,"resolved":%s,"profiles":%s,"problems":%s}\n' \
    "$CELLS" "$HOLES" "$RESOLVED" "$PROFILES" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'support matrix ok: %s cell(s) over %s profile(s), every one produced by a run,\n' \
    "$CELLS" "$PROFILES"
  printf '  and %s of %s hole(s) still resolving to a file and a line under references/.\n' \
    "$RESOLVED" "$HOLES"
  printf '  ⛔ A cell is a run and a hole is a reading, and this check keeps them apart.\n'
  exit 0
fi

printf 'support matrix check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A cell that says "approximately" is worse than one that says "cannot".\n' >&2
printf 'docs/history/todo/emitters.md, EMIT-01.\n' >&2
exit 1
