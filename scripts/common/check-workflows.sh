#!/bin/sh
# check-workflows.sh - does every workflow declare the the things that decide
# whether a run produces data or nothing?
#
# The defect this exists to catch is a matrix that cancels every lane when one
# browser fails to download. That is the DEFAULT behaviour of a matrix, so a
# workflow acquires it by saying nothing, and the failure is invisible until the
# night a run that captured twenty-seven profiles publishes none of them.
#
# -- FOUR RULES, EACH WITH A FAILURE MODE THAT DECIDES A RUN'S OUTPUT --------
#
#   fail-fast: false   on every job that declares a matrix. Without it one
#                      lane's failure cancels its siblings, and the lanes here
#                      are independent by construction.
#   timeout-minutes    on every job. A hung browser holds a runner, and browsers
#                      hang. A job with no limit inherits the platform's, which
#                      is six hours.
#   if: always()       on a job whose `needs` names a job that FANS OUT. That is
#                      the collect job, and one that runs only when every lane
#                      passed publishes nothing on the nights it is most wanted.
#                      A job that needs an ordinary job is left alone: a lane
#                      that runs after a failed plan step is a lane with no plan.
#   a pinned action    every `uses:` names a 40-character commit, never a tag. A
#                      moved tag runs code nobody reviewed with the permissions
#                      granted to the job.
#
# Plus one that is not about matrices: a workflow with no top-level
# `permissions:` inherits whatever the repository grants, which is how a lane
# that downloads a browser ends up holding a write token.
#
# And one about what a step READS: every `.tmp/NAME` a workflow names outside a
# comment must be written by something. That is a tracked file under scripts/
# or crates/ naming the same `.tmp/NAME`, or a line of the same workflow that
# creates it (mkdir, --out, a redirect, tee, cp, mv, rm -rf before a rebuild).
# ⛔ MEASURED, NOT PREDICTED: capture.yml read `.tmp/10-first-profile/` for a
# month after the capture script moved to `.tmp/capture-profile/`, and every
# scheduled capture captured six browsers and published nothing.
#
# -- WHY THIS PARSES RATHER THAN GREPS --------------------------------------
#
# `fail-fast: false` appearing anywhere in a file says nothing about WHICH job
# carries it, and a grep would report a workflow green because a different job
# had the line. So this reads the indentation: jobs at two spaces under `jobs:`,
# their keys at four, `strategy:`'s keys at six. And the `always()` rule needs
# to know which jobs fan out before it can judge which jobs need one, so every
# job is collected first and the verdicts are reached at the end.
#
# IT IS NOT A YAML PARSER AND DOES NOT PRETEND TO BE. It reads the block
# structure these files actually use. The CI step that runs a real YAML library
# over every workflow is what proves they parse at all: two questions, two
# tools.
#
# -- AN EMPTY SCOPE IS EXIT 2 -----------------------------------------------
#
# `git ls-files` answers a path outside the repository with an EMPTY LIST and a
# fatal on stderr, so a fixture directory is walked with `find` instead. That is
# not a precaution: check-routes reported "ok, 0 files" over the fixture written
# to prove it could refuse, in both halves.
#
# Usage:
#   sh scripts/common/check-workflows.sh
#   sh scripts/common/check-workflows.sh --json
#   sh scripts/common/check-workflows.sh --assert-fail-fast-false
#   sh scripts/common/check-workflows.sh --fixtures DIR
#
# Exit codes: 0 clean, 1 a workflow is missing one of them, 2 could not run.
#
# Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURES=""
ASSERT_FF=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --assert-fail-fast-false) ASSERT_FF=1 ;;
    --fixtures) shift; FIXTURES="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-workflows: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-workflows: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-workflows: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-workflows: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

WORKFLOW_DIR=".github/workflows"

if [ -n "$FIXTURES" ]; then
  [ -d "$FIXTURES" ] || { printf 'check-workflows: no directory at %s\n' "$FIXTURES" >&2; exit 2; }
  FILES=$(find "$FIXTURES" -type f -name '*.yml' 2>/dev/null | LC_ALL=C sort)
elif [ ! -d "$WORKFLOW_DIR" ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-workflows/1","workflows":0,"jobs":0,"problems":0}\n'
  else
    printf 'check-workflows: there is no %s directory, so nothing was checked.\n' "$WORKFLOW_DIR" >&2
  fi
  exit 2
else
  FILES=$({ git ls-files -- "$WORKFLOW_DIR/*.yml"; git ls-files --others --exclude-standard -- "$WORKFLOW_DIR/*.yml"; } | LC_ALL=C sort -u)
fi

NFILES=$(printf '%s\n' "$FILES" | awk 'NF' | wc -l | tr -d ' ')

# A SCOPE THAT YIELDED NO FILE HAS VERIFIED NOTHING.
if [ "$NFILES" = 0 ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-workflows/1","workflows":0,"jobs":0,"problems":0}\n'
  else
    printf 'check-workflows: no workflow file in scope, so nothing was checked.\n' >&2
  fi
  exit 2
fi

# Keep this program identical in rule to the PowerShell twin's.
REPORT=$(for wf in $FILES; do
  awk -v file="$wf" -v assert_ff="$ASSERT_FF" '
    # An uninitialised awk variable used as a SUBSCRIPT is the empty string,
    # not zero, so names[njobs] on the first job wrote names[""] and left
    # names[0] unset. The END loop then read an empty name and reported a job
    # that does not exist, once per file.
    BEGIN { njobs = 0 }
    function record() {
      if (job == "") return
      names[njobs] = job
      timeout[job] = has_timeout
      calls[job] = calls_workflow
      matrix[job] = has_matrix
      failfast[job] = ff
      needs[job] = need_list
      cond[job] = ifline
      njobs++
      job = ""
    }
    /^permissions:/ { permissions = 1 }
    # every action is pinned to a commit, never to a tag
    /uses:[ \t]*[^ \t]+@/ {
      line = $0
      sub(/.*uses:[ \t]*/, "", line)
      sub(/[ \t]+#.*$/, "", line)
      sub(/\r$/, "", line)
      ref = line; sub(/^[^@]*@/, "", ref)
      if (ref !~ /^[0-9a-f]+$/ || length(ref) != 40)
        print "P" file ": uses " line ", which is not a 40-character commit. A moved tag runs code nobody reviewed."
    }
    /^jobs:[ \t]*$/ { in_jobs = 1; next }
    in_jobs && /^[a-zA-Z]/ { record(); in_jobs = 0 }
    in_jobs && /^  [A-Za-z0-9_.-]+:[ \t]*$/ {
      record()
      job = $1; sub(/:$/, "", job)
      has_timeout = 0; has_matrix = 0; ff = ""; need_list = ""; ifline = ""
      calls_workflow = 0
      in_strategy = 0
      next
    }
    job != "" && /^    timeout-minutes:/ { has_timeout = 1; in_strategy = 0; next }
    # ⚠ A JOB THAT CALLS A REUSABLE WORKFLOW may not declare a timeout, which
    # the platform refuses; the jobs of the called workflow carry theirs, and
    # this check reads that file too.
    job != "" && /^    uses:/ { calls_workflow = 1 }
    job != "" && /^    needs:/ { need_list = $0; in_strategy = 0; next }
    job != "" && /^    if:/ { ifline = $0; in_strategy = 0; next }
    job != "" && /^    strategy:/ { in_strategy = 1; next }
    job != "" && /^    [A-Za-z0-9_-]+:/ { in_strategy = 0 }
    in_strategy && /^      fail-fast:/ { ff = $2; next }
    in_strategy && /^      matrix:/ { has_matrix = 1; next }
    END {
      record()
      for (i = 0; i < njobs; i++) {
        j = names[i]
        if (!timeout[j] && !calls[j])
          print "P" file ": job " j ": no timeout-minutes. A hung step holds a runner for the platform default."
        if (matrix[j] && failfast[j] != "false" && assert_ff == 1)
          print "P" file ": job " j ": declares a matrix and does not declare fail-fast: false. One lane failing cancels its siblings."
        # ⚠ THE always() RULE IS ABOUT COLLECTING, NOT ABOUT NEEDING. It fires
        # only where a job depends on one that FANS OUT, because that is the
        # job whose whole value is publishing what the lanes managed.
        if (needs[j] != "" && cond[j] !~ /always\(\)/) {
          for (k = 0; k < njobs; k++) {
            u = names[k]
            if (matrix[u] && index(needs[j], u) > 0) {
              print "P" file ": job " j ": needs the fan-out job " u " and does not run regardless. A collect job that only runs when every lane passed publishes nothing on the nights it matters."
              break
            }
          }
        }
      }
      if (!permissions)
        print "P" file ": declares no top-level permissions. The default is whatever the repository grants."
      printf "J%d\n", njobs
    }
  ' "$wf"
done)

# -- every .tmp path a workflow reads has a writer ---------------------------
# ⚠ The writers are searched for in the REPOSITORY'S scripts and crates even
# when --fixtures names another directory, because that is where a fixture's
# steps would find them too.
TMP_REPORT=$(for wf in $FILES; do
  names=$(awk '
    /^[ \t]*#/ { next }
    {
      line = $0
      while (match(line, /\.tmp\/[A-Za-z0-9_.-]+/)) {
        name = substr(line, RSTART + 5, RLENGTH - 5)
        sub(/\.+$/, "", name)
        if (name != "") print name
        line = substr(line, RSTART + RLENGTH)
      }
    }
  ' "$wf" | LC_ALL=C sort -u)
  for name in $names; do
    pattern=$(printf '%s' "$name" | sed 's/[.]/[.]/g')
    pattern="[.]tmp/$pattern([^A-Za-z0-9_.-]|\$)"
    # ⚠ A COMMENT IS NOT A WRITER, and this file's own header names the path
    # that went stale, so comment lines are dropped before the match counts.
    if git grep -hE "$pattern" -- scripts crates 2>/dev/null |
      grep -vE '^[[:space:]]*(#|//|<#)' | grep -q .; then
      continue
    fi
    if awk -v re="$pattern" '
      /^[ \t]*#/ { next }
      $0 ~ re && $0 ~ /(mkdir|--out|>|tee |cp |mv |rm -rf)/ { found = 1 }
      END { exit found ? 0 : 1 }
    ' "$wf"; then
      continue
    fi
    printf 'P%s: reads .tmp/%s, which nothing in scripts/ or crates/ writes and this workflow does not create. A step reading a path its producer stopped writing fails, or uploads nothing, on every run.\n' "$wf" "$name"
  done
done)
REPORT=$(printf '%s\n%s\n' "$REPORT" "$TMP_REPORT")

NJOBS=$(printf '%s\n' "$REPORT" | awk '/^J/ { n += substr($0, 2) } END { print n + 0 }')
DETAIL=$(printf '%s\n' "$REPORT" | awk '/^P/ { print substr($0, 2) }')
PROBLEMS=$(printf '%s\n' "$DETAIL" | awk 'NF' | wc -l | tr -d ' ')

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-workflows/1","workflows":%s,"jobs":%s,"problems":%s}\n' \
    "$NFILES" "$NJOBS" "$PROBLEMS"
  [ "$PROBLEMS" -gt 0 ] && exit 1
  exit 0
fi

if [ "$PROBLEMS" -gt 0 ]; then
  printf 'workflow check failed, %s problem(s) over %s workflow(s) and %s job(s):\n\n' \
    "$PROBLEMS" "$NFILES" "$NJOBS"
  printf '%s\n' "$DETAIL" | sed 's/^/  /'
  exit 1
fi

printf 'workflows ok: %s file(s), %s job(s)' "$NFILES" "$NJOBS"
if [ "$ASSERT_FF" = 1 ]; then
  printf ', every matrix declares fail-fast: false'
fi
printf '\n'
exit 0
