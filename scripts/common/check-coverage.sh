#!/bin/sh
# check-coverage.sh - which cells of the planned capture matrix have a profile,
# and which have none?
#
# The defect this exists to catch is a corpus that looks full because nobody
# wrote down what is missing. Coverage decides whether this project is useful at
# all, and it also decides whether automated merging is possible: agreement
# across two independent sources is only satisfiable when one build is captured
# on more than one host.
#
# -- ⭐ ONE MATRIX, TWO READERS, AND THAT IS THE POINT ------------------------
#
# .github/capture-matrix.json is the plan. .github/workflows/capture.yml builds
# its job matrix from it and this reads it to say what landed, so the plan and
# the report cannot disagree. A matrix written into a workflow and a coverage
# report written from somewhere else is a value in two places with no check that
# they agree.
#
# -- ⛔ A PLANNED CELL THAT WAS NOT ATTEMPTED IS REPORTED, NEVER DROPPED ------
#
# A report that lists only what was tried cannot show what is missing, which is
# the one thing a coverage report is for. Every cell in the plan gets a row:
# `captured`, `absent`, or `not-attempted` where the plan itself says the cell
# is not enabled yet.
#
# -- ⚠ WHAT IT READS, AND WHY IT IS NOT A SECOND ANSWER ----------------------
#
# The corpus side comes from corpus/v1/index.json, which is DERIVED from the
# tree by `b-ids-corpus index --write` and asserted against it by
# `b-ids-corpus verify`. A shell walk of the corpus directory would be a second
# implementation of the layout rule, which is the thing check-corpus already
# refuses to do.
#
# --require-rows NAMES a browser that must have at least one capture SOMEWHERE.
# It is a caller's assertion rather than a property of the plan: nothing is
# required by default, because what a run cares about is the run's business.
#
# Usage:
#   sh scripts/common/check-coverage.sh
#   sh scripts/common/check-coverage.sh --json
#   sh scripts/common/check-coverage.sh --require-rows chrome,edge
#
# Exit codes: 0 every required row has a capture, 1 one does not,
#             2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

# ⛔ ONE SUBSTITUTION, NOT ONE PER LINE READ. An assignment prefix on a
# `while ... read` is re-evaluated on EVERY iteration, so `IFS="$(printf
# '\t')" read ...` forks once per line. Measured 2026-09-02: a command
# substitution costs 35 ms on this host, and check-docs.sh reads about 1100
# lines that way. TODO/tooling.md, TOOL-18.
TAB=$(printf '\t')

JSON=0
REQUIRE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --require-rows) shift; REQUIRE="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-coverage: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-coverage: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-coverage: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-coverage: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-coverage: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

command -v jq >/dev/null 2>&1 || { printf 'check-coverage: jq not found; it reads two json files\n' >&2; exit 2; }

PLAN=".github/capture-matrix.json"
INDEX="$CORPUS_ROOT/corpus/v1/index.json"

[ -f "$PLAN" ] || { printf 'check-coverage: there is no %s, so there is no plan to report against\n' "$PLAN" >&2; exit 2; }

# An absent index is a corpus with nothing in it, which is a real state and not
# an error: every cell is then absent. A MALFORMED one is exit 2.
CAPTURED=""
if [ -f "$INDEX" ]; then
  # ⛔ THE CARRIAGE RETURN IS STRIPPED, and leaving it out was a defect that
  # shipped for one run. jq on this Windows host writes CRLF, so the third field of
  # every plan row arrived as `true` followed by a carriage return, which is not
  # `true`: the report dropped the word `required` from every required row while
  # the JSON, which does not carry that field, matched its twin exactly.
  # ⚠ A drift the comparison could not see, found by reading the two human outputs
  # side by side.
    CAPTURED=$(jq -r '.profiles[] | "\(.browser|ascii_downcase)/\(.channel)/\(.platform)"' "$INDEX" 2>/dev/null | tr -d '\r') || {
    printf 'check-coverage: %s is not readable as an index\n' "$INDEX" >&2
    exit 2
  }
fi

CELLS=$(jq -r '.cells[] | "\(.browser)/\(.channel)/\(.platform)\t\(.enabled)\t\(.required)"' "$PLAN" 2>/dev/null | tr -d '\r') || {
  printf 'check-coverage: %s is not readable as a plan\n' "$PLAN" >&2
  exit 2
}

# ⛔ A PLAN WITH NO CELLS HAS REPORTED NOTHING.
NCELLS=$(printf '%s\n' "$CELLS" | awk 'NF' | wc -l | tr -d ' ')
if [ "$NCELLS" = 0 ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-coverage/2","cells":0,"captured":0,"absent":0,"not_attempted":0,"unplanned":0,"missing_required":0}\n'
  else
    printf 'check-coverage: the plan holds no cell, so nothing was reported.\n' >&2
  fi
  exit 2
fi

ROWS=""
NCAPTURED=0
NABSENT=0
NNOTATTEMPTED=0
NL="
"
printf '%s\n' "$CAPTURED" > "${TMPDIR:-/tmp}/.coverage.$$"
cp "${TMPDIR:-/tmp}/.coverage.$$" "${TMPDIR:-/tmp}/.coverage.unplanned.$$" 2>/dev/null || :
# shellcheck disable=SC2016
while IFS="$TAB" read -r key enabled required; do
  [ -n "$key" ] || continue
  n=$(awk -v k="$key" '$0 == k { c++ } END { print c + 0 }' "${TMPDIR:-/tmp}/.coverage.$$")
  if [ "$n" -gt 0 ]; then
    state="captured"
    NCAPTURED=$((NCAPTURED + 1))
  elif [ "$enabled" = "true" ]; then
    state="absent"
    NABSENT=$((NABSENT + 1))
  else
    state="not-attempted"
    NNOTATTEMPTED=$((NNOTATTEMPTED + 1))
  fi
  req=""
  [ "$required" = "true" ] && req=" required"
  ROWS="$ROWS$(printf '  %-14s %-34s %s profile(s)%s' "$state" "$key" "$n" "$req")$NL"
done <<CELLS_EOF
$CELLS
CELLS_EOF
rm -f "${TMPDIR:-/tmp}/.coverage.$$"

# ⛔ A CAPTURE NO PLANNED CELL COVERS IS REPORTED, NEVER DROPPED. The rule
# above says a planned cell that was not attempted is reported; this is the
# same rule from the other side, and it was missing. Found 2026-09-04, when
# DRIVER-11 added a firefox/stable/win64 profile the plan does not carry: the
# corpus held seven profiles and the report accounted for six, with nothing
# saying so. A report that can only see the plan cannot show what the plan is
# missing.
UNPLANNED=""
NUNPLANNED=0
PLANNED_KEYS=$(printf '%s\n' "$CELLS" | cut -f1)
for key in $(printf '%s\n' "$CAPTURED" | sort -u); do
  [ -n "$key" ] || continue
  if ! printf '%s\n' "$PLANNED_KEYS" | grep -qx -- "$key"; then
    n=$(awk -v k="$key" '$0 == k { c++ } END { print c + 0 }' "${TMPDIR:-/tmp}/.coverage.unplanned.$$" 2>/dev/null || printf 0)
    UNPLANNED="$UNPLANNED$(printf '  %-14s %-34s %s profile(s)' "unplanned" "$key" "$n")$NL"
    NUNPLANNED=$((NUNPLANNED + 1))
  fi
done
rm -f "${TMPDIR:-/tmp}/.coverage.unplanned.$$"

# The caller's assertion: a named browser must have a capture SOMEWHERE.
MISSING=""
NMISSING=0
if [ -n "$REQUIRE" ]; then
  for want in $(printf '%s' "$REQUIRE" | tr ',' ' '); do
    [ -n "$want" ] || continue
    hit=$(printf '%s\n' "$CAPTURED" | awk -v b="$want" -F/ '$1 == b { c++ } END { print c + 0 }')
    if [ "$hit" = 0 ]; then
      MISSING="$MISSING  $want: no capture at all, on any channel or platform$NL"
      NMISSING=$((NMISSING + 1))
    fi
  done
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-coverage/2","cells":%s,"captured":%s,"absent":%s,"not_attempted":%s,"unplanned":%s,"missing_required":%s}\n' \
    "$NCELLS" "$NCAPTURED" "$NABSENT" "$NNOTATTEMPTED" "$NUNPLANNED" "$NMISSING"
  [ "$NMISSING" -gt 0 ] && exit 1
  exit 0
fi

printf 'coverage over %s planned cell(s):\n\n' "$NCELLS"
printf '%s' "$ROWS"
if [ "$NUNPLANNED" -gt 0 ]; then
  printf '%s' "$UNPLANNED"
fi
printf '\n%s captured, %s absent, %s not attempted, %s outside the plan.\n' \
  "$NCAPTURED" "$NABSENT" "$NNOTATTEMPTED" "$NUNPLANNED"

if [ "$NMISSING" -gt 0 ]; then
  printf '\ncoverage check failed, %s required row(s) with no capture:\n\n' "$NMISSING"
  printf '%s' "$MISSING"
  exit 1
fi
exit 0
