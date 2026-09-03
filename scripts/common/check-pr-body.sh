#!/bin/sh
# check-pr-body.sh - would a scheduled run that found a change open a pull
# request a reviewer can act on, and would a run that found nothing stay silent?
#
# ⛔ AN ISSUE IS A REQUEST FOR SOMEBODY ELSE TO DO WORK. A pull request with the
# work already in it is the deliverable. TODO/ci.md, CI-04.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the suite that owns the body's contents is present, case by case, and
#      passes. ⚠ THE ASSERTIONS ARE THE CRATE'S: a second idea of what a body
#      must carry, written here, would disagree with the crate's the first time
#      either moved;
#   2. ⭐ END TO END OVER THE REAL CORPUS, the generator opens one request per
#      route, and each body carries every section, the validator's output and a
#      named list of what the run could not do;
#   3. ⛔ A NO-OP CHANGE OPENS NOTHING AT ALL. Silence is the correct output for
#      a browser that did not change, and a bot that writes on a schedule trains
#      people to ignore it;
#   4. a run file that does not parse is a refusal rather than a body with a
#      blank in it.
#
# ⚠ THE FIELD-LEVEL DIFF IS THE SUITE'S HALF, and saying so is the point. An
# `Advanced` movement needs two published builds at ONE route, and which two
# depends on what the corpus holds today; the suite renders that pair from a
# fixture, and the end-to-end leg here drives the case that is deterministic
# whatever the corpus holds: every route is new.
#
# ⛔ --fixture IS REQUIRED, for the reason `latest` requires --assert-stable.
# There is no pull request to check: this checks a GENERATOR against a fixture,
# and a run with no argument would read as though it had checked a real one.
#
# Usage:
#   sh scripts/common/check-pr-body.sh --fixture
#   sh scripts/common/check-pr-body.sh --fixture --json
#
# Exit codes: 0 the body is what CI-04 asks for, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURE=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --fixture) FIXTURE=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-pr-body: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

[ "$FIXTURE" = 1 ] || {
  printf 'check-pr-body: --fixture is required. There is no pull request to check:\n' >&2
  printf '  this checks a generator against a fixture, and a run with no argument\n' >&2
  printf '  would read as though it had checked a real one.\n' >&2
  exit 2
}

command -v git >/dev/null 2>&1 || { printf 'check-pr-body: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-pr-body: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-pr-body: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { printf 'check-pr-body: cargo not found\n' >&2; exit 2; }

SUITE="$REPO_ROOT/crates/b-ids-corpus/tests/pull_request.rs"
[ -f "$SUITE" ] || { printf 'check-pr-body: no suite at %s\n' "$SUITE" >&2; exit 2; }

# ⛔ THE CASES ARE NAMED HERE AND ASSERTED THERE, so a suite that lost one is
# caught by this check rather than by nobody. A check that ran a suite without
# saying which cases it expects passes over a deleted test.
WANT='pull_request_a_body_carries_every_fact_the_model_holds
pull_request_a_no_op_change_opens_nothing_at_all
pull_request_a_body_names_what_the_run_could_not_do
pull_request_two_runs_over_one_change_produce_identical_text
pull_request_a_branch_name_is_one_per_route_and_schema_major
pull_request_the_merge_conditions_can_fail_and_say_which
pull_request_every_condition_holding_is_reachable_rather_than_impossible
pull_request_the_labels_carry_the_class_the_confidence_and_the_subject
pull_request_a_new_route_says_it_has_nothing_to_diff_against'

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

CASES_WANTED=0
for want in $WANT; do
  CASES_WANTED=$((CASES_WANTED + 1))
  grep -q "fn $want" "$SUITE" || note "$want is not in the suite"
done

OUT="$REPO_ROOT/.tmp/check-pr-body"
rm -rf "$OUT"
mkdir -p "$OUT/empty" "$OUT/requests" || {
  printf 'check-pr-body: cannot create %s\n' "$OUT" >&2
  exit 2
}

# ⛔ READ FROM THE PROCESS, UNPIPED.
cargo test -q -p b-ids-corpus --test pull_request > "$OUT/tests.log" 2>&1
rc_t=$?
CASES=$(awk '/^running [0-9]+ tests/ { print $2; exit }' "$OUT/tests.log")
[ "$rc_t" = 0 ] || note "the suite failed. Its output is in .tmp/check-pr-body/tests.log"
[ "${CASES:-0}" -ge "$CASES_WANTED" ] 2>/dev/null || \
  note "the suite ran ${CASES:-0} case(s) where at least $CASES_WANTED were expected"

cargo build -q -p b-ids-corpus || {
  printf 'check-pr-body: the corpus crate did not build\n' >&2
  exit 2
}
BIN="$REPO_ROOT/target/debug/b-ids-corpus"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-pr-body: %s is not executable\n' "$BIN" >&2; exit 2; }

# ⚠ THE RUN FACTS ARE A FIXTURE, and every field is filled: the generator
# refuses a file missing one, which is what keeps a body from carrying a blank
# where a run identifier belongs.
UNAVAILABLE='the macos lane has no runner in this plan'
cat > "$OUT/run.json" <<JSON
{
  "workflow": "capture.yml",
  "run_id": "a fixture run",
  "images": [["linux64", "a fixture image"]],
  "harness": "a fixture harness",
  "command": "sh experiments/10-first-profile.sh --headless --browser chrome",
  "unavailable": ["$UNAVAILABLE"],
  "validator_output": "a fixture validator line",
  "validator_findings": 0,
  "formats_round_trip": true
}
JSON

# -- 2: end to end, over the real corpus ------------------------------------
#
# ⭐ EVERY ROUTE IS NEW, which is the case that is deterministic whatever the
# corpus holds. `--before` is an empty directory rather than a state this check
# invented, so nothing here fabricates a profile.
"$BIN" pull-request --before "$OUT/empty" --after "$REPO_ROOT" \
  --run "$OUT/run.json" --out "$OUT/requests" > "$OUT/generate.log" 2>&1
rc_g=$?
[ "$rc_g" = 0 ] || note "the generator exited $rc_g. Its output is in .tmp/check-pr-body/generate.log"

STATUS=$(awk '/^corpus=pull-request /{ line = $0 } END { print line }' "$OUT/generate.log")
REQUESTS=$(printf '%s' "$STATUS" | awk -F'requests:' '{ split($2, a, / /); print a[1] }')
AUTO=$(printf '%s' "$STATUS" | awk -F'auto:' '{ split($2, a, / /); print a[1] }')
[ -n "${REQUESTS:-}" ] || { printf 'check-pr-body: the generator printed no status line\n' >&2; exit 1; }
[ "${REQUESTS:-0}" -ge 1 ] 2>/dev/null || note "a corpus with profiles produced no request at all"

for dir in "$OUT/requests"/*/; do
  [ -d "$dir" ] || continue
  name=$(basename "$dir")
  for want in branch title body.md labels mergeable; do
    [ -s "$dir$want" ] || note "$name: $want was not written, or is empty"
  done
  [ -s "$dir/body.md" ] || continue
  for heading in '## What changed' '## The fields that differ' \
                 '## Where this capture came from' '## The validator' \
                 '## What this run could not do' '## Reproducing this' '## Merging'; do
    grep -qF "$heading" "$dir/body.md" || note "$name: the body has no $heading section"
  done
  grep -qF 'a fixture validator line' "$dir/body.md" || \
    note "$name: the body does not carry the validator's output"
  grep -qF "$UNAVAILABLE" "$dir/body.md" || \
    note "$name: the body does not name what the run could not do"
  grep -qF 'a fixture run' "$dir/body.md" || \
    note "$name: the body does not carry the run identifier"
  grep -q '^confidence:' "$dir/labels" || note "$name: no confidence label"
  grep -q '^class:' "$dir/labels" || note "$name: no class label"
  grep -q '^subject:' "$dir/labels" || note "$name: no subject label"
done

# -- 3: the no-op, end to end -----------------------------------------------
#
# ⛔ NOT A FIXTURE. The real corpus against itself is the truest no-op there is,
# and it needs nothing invented.
rm -rf "$OUT/none" && mkdir -p "$OUT/none"
"$BIN" pull-request --before "$REPO_ROOT" --after "$REPO_ROOT" \
  --run "$OUT/run.json" --out "$OUT/none" > "$OUT/noop.log" 2>&1
rc_n=$?
[ "$rc_n" = 0 ] || note "the no-op run exited $rc_n"
NOOP=$(awk -F'requests:' '/^corpus=pull-request /{ split($2, a, / /); n = a[1] } END { print n }' "$OUT/noop.log")
[ "${NOOP:-1}" = 0 ] || note "a no-op change produced ${NOOP:-?} request(s), and it must produce none"
if [ -n "$(ls -A "$OUT/none" 2>/dev/null)" ]; then
  note "a no-op change wrote files into the output directory"
fi

# -- 4: a run file that does not parse is a refusal --------------------------
#
# ⛔ A BODY WITH A BLANK WHERE A RUN IDENTIFIER BELONGS is a fabricated
# provenance block in a project whose product is provenance.
printf '{"workflow":"capture.yml"}\n' > "$OUT/half.json"
"$BIN" pull-request --before "$OUT/empty" --after "$REPO_ROOT" \
  --run "$OUT/half.json" --out "$OUT/half" > "$OUT/half.log" 2>&1
rc_h=$?
[ "$rc_h" = 2 ] || note "a run file missing fields exited $rc_h where 2 was expected"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-pr-body/1","cases":%s,"requests":%s,"auto":%s,"problems":%s}\n' \
    "${CASES:-0}" "${REQUESTS:-0}" "${AUTO:-0}" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'pr body ok: %s suite case(s), %s request(s) generated from the corpus,\n' \
    "$CASES" "$REQUESTS"
  printf '  %s of them mergeable without a human, every body carrying its seven\n' "$AUTO"
  printf '  sections, and a no-op change opening nothing at all.\n'
  exit 0
fi

printf 'pr body check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A pull request with the work in it is the deliverable, and one that\n' >&2
printf 'silently omits a field is worse than one that says it could not capture\n' >&2
printf 'it. TODO/ci.md, CI-04.\n' >&2
exit 1
