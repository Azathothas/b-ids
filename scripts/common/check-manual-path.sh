#!/bin/sh
# check-manual-path.sh - does every automated job name the command a person runs
# instead, and does that command resolve on this host?
#
# ⛔ A PROJECT WHOSE ONLY PATH TO A CAPTURE IS ONE PROVIDER'S AUTOMATION
# DEGRADES TO NOTHING WHEN THAT PROVIDER DOES. TODO/ci.md, CI-08.
#
# ⭐ THE TEST IS ONE SENTENCE: if the provider disappeared, the project degrades
# to "somebody runs one command" rather than to nothing. This check is what says
# the sentence is true rather than hoped for.
#
# -- ⛔ EVERY JOB DECLARES ITS OWN MANUAL EQUIVALENT --------------------------
#
# A `# manual: <command>` comment inside a job block names what a person runs to
# do that job's work by hand. ⚠ IT LIVES IN THE WORKFLOW, beside the job, rather
# than in a table somewhere else: a list of equivalents in a second file is a
# value in two places with no check that they agree, and the copy that goes
# stale is the one nobody is reading when the provider is down.
#
# ⛔ A JOB WITH NO MANUAL LINE FAILS. That is the whole check: an automated step
# nobody can do by hand is a step that stops existing when the platform does.
#
# -- ⚠ WHAT "RESOLVES" MEANS, AND WHY IT IS NOT "RUNS" -----------------------
#
# Each named command is checked to the point where the host can say it exists:
# a script in this tree must exist and parse, and the program a command starts
# with must be on PATH. ⛔ IT IS NOT EXECUTED, and the reason is in the tree
# rather than convenience: one job of the nine is a fuzz lane that runs a
# hundred thousand cases, and another launches a browser for ninety seconds. A
# check that ran them is a check nobody runs, and a check nobody runs is worth
# nothing at three in the morning.
#
# ⛔ A COMMAND THAT NAMES A SCRIPT THIS TREE DOES NOT HAVE IS A FAILURE, not a
# skip: that is exactly the rot this entry exists to catch.
#
# Usage:
#   sh scripts/common/check-manual-path.sh
#   sh scripts/common/check-manual-path.sh --json
#
# Exit codes: 0 every job names a command that resolves, 1 one does not,
#             2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-manual-path: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-manual-path: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-manual-path: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-manual-path: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

WORKFLOWS=$(git ls-files -- '.github/workflows/*.yml' '.github/workflows/*.yaml' | LC_ALL=C sort)
[ -n "$WORKFLOWS" ] || {
  printf 'check-manual-path: no workflows, so nothing was checked\n' >&2
  exit 2
}

# ⛔ IT READS THE INDENTATION rather than grepping. `# manual:` appearing
# anywhere in a file says nothing about WHICH job carries it, and a grep would
# report a workflow green because a different job had the line.
# scripts/common/check-workflows.sh reads the same shape for the same reason.
PROBLEMS=""
COUNT=0
JOBS=0
NAMED=0

for workflow in $WORKFLOWS; do
  # Each line: <job>\t<manual command or the empty string>
  pairs=$(awk '
    /^jobs:/ { injobs = 1; next }
    injobs && /^[^[:space:]]/ { injobs = 0 }
    injobs && /^  [A-Za-z0-9_-]+:/ {
      if (job != "") { print job "\t" manual }
      job = $0
      sub(/^  /, "", job)
      sub(/:.*$/, "", job)
      manual = ""
      next
    }
    injobs && job != "" && /^[[:space:]]*# manual:/ {
      line = $0
      sub(/^[[:space:]]*# manual:[[:space:]]*/, "", line)
      if (manual == "") { manual = line }
    }
    END { if (job != "") { print job "\t" manual } }
  ' "$workflow")

  # ⚠ A here-string is not portable; the loop reads the variable through a pipe
  # into a subshell, so the counters are accumulated in a temporary file rather
  # than in the subshell that would discard them.
  printf '%s\n' "$pairs" | while IFS= read -r pair; do
    [ -n "$pair" ] || continue
    printf '%s\n' "$pair"
  done > "$REPO_ROOT/.tmp/manual-pairs.txt"

  while IFS="$(printf '\t')" read -r job manual; do
    [ -n "$job" ] || continue
    JOBS=$((JOBS + 1))
    if [ -z "$manual" ]; then
      PROBLEMS="$PROBLEMS  $workflow: job '$job' names no manual equivalent
"
      COUNT=$((COUNT + 1))
      continue
    fi
    NAMED=$((NAMED + 1))
    # ⛔ The first word is the program; the first path-shaped word is the script.
    program=$(printf '%s' "$manual" | awk '{ print $1 }')
    script=$(printf '%s' "$manual" | awk '{ for (i = 1; i <= NF; i++) if ($i ~ /\//) { print $i; exit } }')
    if ! command -v "$program" >/dev/null 2>&1; then
      PROBLEMS="$PROBLEMS  $workflow: job '$job' names '$program', which is not on PATH here
"
      COUNT=$((COUNT + 1))
      continue
    fi
    if [ -n "$script" ]; then
      if [ ! -f "$script" ]; then
        PROBLEMS="$PROBLEMS  $workflow: job '$job' names $script, which this tree does not have
"
        COUNT=$((COUNT + 1))
        continue
      fi
      case "$script" in
        *.sh)
          if ! sh -n "$script" 2>/dev/null; then
            PROBLEMS="$PROBLEMS  $workflow: job '$job' names $script, which does not parse
"
            COUNT=$((COUNT + 1))
          fi
          ;;
      esac
    fi
  done < "$REPO_ROOT/.tmp/manual-pairs.txt"
done
rm -f "$REPO_ROOT/.tmp/manual-pairs.txt"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-manual-path/1","jobs":%s,"named":%s,"problems":%s}\n' \
    "$JOBS" "$NAMED" "$COUNT"
elif [ "$COUNT" = 0 ]; then
  printf 'manual path ok: %s job(s), each names a command that resolves here\n' "$JOBS"
else
  printf 'manual path check failed, %s problem(s):\n\n' "$COUNT" >&2
  printf '%s\n' "$PROBLEMS" >&2
  printf 'Every automated job names the command a person runs instead, as a\n' >&2
  # shellcheck disable=SC2016 # the text names a literal comment marker
  printf '`# manual:` comment inside the job. TODO/ci.md, CI-08.\n' >&2
fi

[ "$COUNT" = 0 ] || exit 1
exit 0
