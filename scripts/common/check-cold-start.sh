#!/bin/sh
# check-cold-start.sh - is the cold-start job still cold, and does everything a
# cold pipeline names still resolve on this host?
#
# ⛔ EVERY WARM RUN PASSES OVER A BROKEN COLD PATH. A dead URL, a removed field
# or a renamed flag is invisible until the day somebody needs a capture, and
# nothing else in this tree catches it. docs/history/todo/ci.md, CI-05.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the workflow exists, runs on a schedule, and can be dispatched by hand;
#   2. ⛔ NO CACHE OF ANY KIND. No actions/cache, no rust-cache, no sccache, no
#      `cache:` key. A cold-start job that shares a cache has stopped being one
#      while continuing to report as one, which is the worst outcome available;
#   3. its concurrency group is its own, so it does not queue behind or cancel
#      another workflow;
#   4. every stage carries an `id`, and the report step names every one of them
#      and runs `if: always()`. "Fails loudly naming the first step that could
#      not resolve" is the acceptance, and a report that skips a stage or is
#      skipped itself is not that;
#   5. ⭐ THE RESOLUTION PROBE. Every program a cold pipeline needs, reported by
#      name. The workflow runs this same probe as its first step, so the list
#      lives here rather than in two places.
#
# -- ⚠ WHY A MISSING TOOL IS A REPORT AND NOT A FAILURE ----------------------
#
# On a laptop a missing tool is a fact about the laptop, and a gate that failed
# over it would be a gate somebody disables. On a runner the tools are installed
# on purpose, so the workflow passes --require-tools and the first absent one is
# named and fails the job. That is the same split --strict makes in the gate.
#
# ⛔ IT RUNS NO PIPELINE. Building the workspace, taking a capture and
# assembling a publish are what the WORKFLOW does on a fresh runner; a check
# that did them here would take an hour and would prove nothing about a cold
# machine, because this one is warm.
#
# Usage:
#   sh scripts/common/check-cold-start.sh
#   sh scripts/common/check-cold-start.sh --json
#   sh scripts/common/check-cold-start.sh --resolve
#   sh scripts/common/check-cold-start.sh --resolve --require-tools
#
# Exit codes: 0 the job is still cold and the probe found what it names,
# 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
RESOLVE=0
REQUIRE=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --resolve) RESOLVE=1 ;;
    --require-tools) REQUIRE=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-cold-start: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-cold-start: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-cold-start: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-cold-start: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

WF=".github/workflows/cold-start.yml"

# ⛔ THE PROGRAMS A COLD PIPELINE NEEDS, IN ONE PLACE. The workflow runs this
# probe rather than carrying its own list, so this cannot go stale against it.
# ⚠ A browser is deliberately NOT here: `b-ids-driver resolve` exits 2 on a host
# with none, and that is a fact about the host rather than a broken cold path.
TOOLS="git cargo rustc rustup jq awk sed grep tar"

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

FOUND=0
MISSING=0
FIRST_MISSING=""
REPORT=""
for tool in $TOOLS; do
  if command -v "$tool" >/dev/null 2>&1; then
    FOUND=$((FOUND + 1))
    REPORT="$REPORT  ok    $tool
"
  else
    MISSING=$((MISSING + 1))
    [ -n "$FIRST_MISSING" ] || FIRST_MISSING="$tool"
    REPORT="$REPORT  ABSENT $tool
"
  fi
done

# ⭐ --resolve IS THE PROBE ON ITS OWN, which is what the workflow's first step
# runs. It asserts nothing about the workflow file, because on a fresh runner
# the interesting question is whether the machine has what the pipeline names.
if [ "$RESOLVE" = 1 ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-cold-start/1","tools":%s,"found":%s,"missing":%s,"stages":0,"problems":%s}\n' \
      "$(printf '%s\n' "$TOOLS" | wc -w | tr -d ' ')" "$FOUND" "$MISSING" \
      "$([ "$REQUIRE" = 1 ] && echo "$MISSING" || echo 0)"
    { [ "$REQUIRE" = 1 ] && [ "$MISSING" -gt 0 ]; } && exit 1
    exit 0
  fi
  printf 'cold start probe over %s program(s):\n\n' "$(printf '%s\n' "$TOOLS" | wc -w | tr -d ' ')"
  printf '%s' "$REPORT"
  printf '\n'
  if [ "$MISSING" = 0 ]; then
    printf 'every program a cold pipeline names is on this host.\n'
    exit 0
  fi
  if [ "$REQUIRE" = 1 ]; then
    printf '⛔ the cold path breaks at the first absent program: %s\n' "$FIRST_MISSING" >&2
    printf 'Every warm run passes over a broken cold path. docs/history/todo/ci.md, CI-05.\n' >&2
    exit 1
  fi
  printf '⚠ %s absent, first %s. On this host that is a fact about the host;\n' "$MISSING" "$FIRST_MISSING"
  printf '  --require-tools is what makes it a failure, and the runner passes it.\n'
  exit 0
fi

# -- 1: the workflow, and its triggers ---------------------------------------
STAGES=0
if [ ! -f "$WF" ]; then
  note "there is no $WF, so nothing ever runs this pipeline from cold"
  printf 'cold start check failed, %s problem(s):\n\n' "$COUNT" >&2
  printf '%s\n' "$PROBLEMS" >&2
  exit 1
fi

ON=$(awk '/^on:[ \t]*$/ { inside = 1; next } inside && /^[a-zA-Z]/ { inside = 0 } inside { print }' "$WF")
printf '%s\n' "$ON" | grep -q '^  schedule:' ||
  note "the cold-start workflow is not on a schedule, and a cold path nobody runs is one nobody checks"
printf '%s\n' "$ON" | grep -q '^  workflow_dispatch:' ||
  note "the cold-start workflow cannot be dispatched by hand"

# -- 2: no cache of any kind -------------------------------------------------
#
# ⛔ A COMMENT MAY SAY THE WORDS; A STEP MAY NOT CARRY THEM.
LIVE=$(sed 's/^[ \t]*#.*$//' "$WF")
CACHES=$(printf '%s\n' "$LIVE" | grep -cE 'actions/cache|rust-cache|sccache|RUSTC_WRAPPER|^[ \t]*cache:')
[ "$CACHES" = 0 ] ||
  note "$CACHES line(s) name a cache, and a cold-start job that shares one has stopped being one"

# -- 3: its own concurrency group --------------------------------------------
GROUP=$(awk '/^concurrency:[ \t]*$/ { inside = 1; next } inside && /^[a-zA-Z]/ { inside = 0 }
  inside && /^  group:/ { sub(/^  group:[ \t]*/, ""); print; exit }' "$WF")
case "${GROUP:-}" in
  *cold-start*) ;;
  *) note "the concurrency group is ${GROUP:-absent}, which is not this workflow's own" ;;
esac

# -- 4: every stage has an id, and the report names every one ----------------
#
# ⚠ READ FROM THE STEP BLOCKS. An `id:` anywhere in a file says nothing about
# which step carries it.
IDS=$(awk '/^        id:[ \t]*/ { sub(/^        id:[ \t]*/, ""); print }' "$WF")
STAGES=$(printf '%s\n' "$IDS" | awk 'NF' | wc -l | tr -d ' ')
[ "$STAGES" -ge 6 ] ||
  note "$STAGES stage(s) carry an id, and a pipeline reported at that resolution names nothing useful"

REPORT_STEP=$(awk '/^      - name: what this cold start reached/ { inside = 1 }
  inside { print }' "$WF")
[ -n "$REPORT_STEP" ] ||
  note "there is no report step, so a failed run does not name the stage that broke"
printf '%s\n' "$REPORT_STEP" | grep -q 'if: always()' ||
  note "the report step does not run if: always(), so a red job says nothing about which stage went red"

for id in $IDS; do
  case "$id" in
    report) continue ;;
  esac
  printf '%s\n' "$REPORT_STEP" | grep -q "steps.$id.outcome" ||
    note "the report step does not name the $id stage, so a failure there is not reported by name"
done

# -- 5: the probe, folded into the verdict -----------------------------------
[ "$REQUIRE" = 0 ] || [ "$MISSING" = 0 ] ||
  note "the cold path breaks at the first absent program: $FIRST_MISSING"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-cold-start/1","tools":%s,"found":%s,"missing":%s,"stages":%s,"problems":%s}\n' \
    "$(printf '%s\n' "$TOOLS" | wc -w | tr -d ' ')" "$FOUND" "$MISSING" "$STAGES" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'cold start ok: %s stage(s), each named by the report step, no cache of any kind,\n' "$STAGES"
  printf '  and %s of %s program(s) present on this host.\n' "$FOUND" \
    "$(printf '%s\n' "$TOOLS" | wc -w | tr -d ' ')"
  [ "$MISSING" = 0 ] ||
    printf '  ⚠ A SKIP IS NOT A PASS: %s absent here, first %s. The runner passes --require-tools.\n' \
      "$MISSING" "$FIRST_MISSING"
  printf '  ⛔ Nothing was built, captured or published by this check.\n'
  exit 0
fi

printf 'cold start check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'Every warm run passes over a broken cold path. docs/history/todo/ci.md, CI-05.\n' >&2
exit 1
