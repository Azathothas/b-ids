#!/bin/sh
# check-notes-generator.sh - do the release body and the changelog entry come out
# of one generator, and do they agree fact for fact?
#
# ⛔ RELEASE NOTES AND A CHANGELOG WRITTEN SEPARATELY DRIFT, and the reader who
# trusts the wrong one is the one who was doing something careful. TODO/publish.md,
# PUB-08.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. over one corpus change, the two outputs carry every fact the model holds;
#   2. two runs over that change produce identical text;
#   3. a NO-OP change produces nothing at all, because silence is the correct
#      output for "the browser did not change";
#   4. ⛔ AND THE COMPARISON CAN FAIL. A fixture whose two outputs are generated
#      from DIFFERENT inputs is asserted NOT to agree. A check that only ever
#      sees agreement is one nobody knows works.
#
# ⚠ THE ASSERTIONS ARE THE CRATE'S. A second comparison written here would be a
# second idea of what "agree" means, disagreeing with the one the crate ships the
# first time either moved. This runs that suite and reads its exit code.
#
# Usage:
#   sh scripts/common/check-notes-generator.sh
#   sh scripts/common/check-notes-generator.sh --json
#
# Exit codes: 0 they agree, 1 they do not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-notes-generator: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-notes-generator: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-notes-generator: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-notes-generator: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { printf 'check-notes-generator: cargo not found\n' >&2; exit 2; }

SUITE="$REPO_ROOT/crates/b-ids-corpus/tests/notes.rs"
[ -f "$SUITE" ] || {
  printf 'check-notes-generator: no suite at %s\n' "$SUITE" >&2
  exit 2
}

# ⛔ THE FOUR ASSERTIONS ARE NAMED HERE AND ASSERTED THERE, so a suite that lost
# one is caught by this check rather than by nobody. A check that ran a suite
# without saying which cases it expects passes over a deleted test.
WANT='notes_the_two_outputs_agree_fact_for_fact
notes_a_no_op_change_renders_nothing_at_all
notes_two_runs_over_one_change_produce_identical_text
notes_two_outputs_generated_from_different_inputs_do_not_agree'

MISSING=""
COUNT=0
for want in $WANT; do
  grep -q "fn $want" "$SUITE" || {
    MISSING="$MISSING  $want is not in the suite
"
    COUNT=$((COUNT + 1))
  }
done

OUT="$REPO_ROOT/.tmp/check-notes-generator.log"
# ⛔ READ FROM THE PROCESS, UNPIPED.
cargo test -q -p b-ids-corpus --test notes > "$OUT" 2>&1
RC=$?
CASES=$(awk '/^running [0-9]+ tests/ { print $2; exit }' "$OUT")
if [ "$RC" != 0 ]; then
  MISSING="$MISSING  the suite failed. Its output is in .tmp/check-notes-generator.log
"
  COUNT=$((COUNT + 1))
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-notes-generator/1","cases":%s,"problems":%s}\n' "${CASES:-0}" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'notes generator ok: %s case(s). The release body and the changelog entry\n' "${CASES:-0}"
  printf '  are rendered from one model, they carry every fact it holds, a no-op\n'
  printf '  change renders nothing, and two outputs from different inputs are\n'
  printf '  asserted NOT to agree.\n'
  exit 0
fi

printf 'notes generator check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$MISSING" >&2
printf 'One generator, two outputs, so the two cannot disagree by construction\n' >&2
printf 'rather than by discipline. TODO/publish.md, PUB-08.\n' >&2
exit 1
