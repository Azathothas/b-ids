#!/bin/sh
# check-exit-codes.sh - does every script in this tree report "could not run" as
# 2, on both halves of every pair?
#
# The defect this exists to catch is a check that fails because a machine cannot
# run it. A capture job on a runner with no browser must not fail the build, and
# a check that returns 1 for "I could not run" is a check somebody disables.
# docs/history/todo/ci.md, CI-07.
#
# -- ⛔ WHAT IT ACTUALLY MEASURES, AND WHY THIS INPUT ------------------------
#
# Every script here is invoked with an argument no script accepts. That is the
# one state EVERY script in the tree can be put into from outside, without a
# missing tool, a missing browser or a network. ⚠ A check that needed a real
# unrunnable condition per script would have as many special cases as scripts,
# and the ones it could not construct would go unchecked.
#
# ⭐ MEASURED 2026-09-02 AND IT IS WHY THIS CHECK EXISTS. Every POSIX half
# returned 2 and every PowerShell half returned 1, 22 pairs of 22: `pwsh -File`
# reports a parameter-binding failure as 1, and 1 is this project's code for
# "it ran and the thing failed". Both halves of one pair disagreeing about
# whether a state is a failure is the exact defect CI-07 is about, and the twin
# comparison could not see it because it compares runs that succeed.
#
# ⛔ IT DOES NOT ACCEPT 0. A script that ignored an argument it does not
# understand and ran anyway is worse than one that refused: it did something
# other than what it was asked to do and reported success.
#
# Usage:
#   sh scripts/common/check-exit-codes.sh
#   sh scripts/common/check-exit-codes.sh --json
#
# Exit codes: 0 clean, 1 a script did not answer 2, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-exit-codes: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-exit-codes: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-exit-codes: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-exit-codes: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⛔ THE ARGUMENT NO SCRIPT ACCEPTS, spelled so it cannot become one by
# accident. A short name like --xyz is a name somebody might add.
UNKNOWN='--b-ids-check-exit-codes-not-a-real-argument'

# ⚠ THE SHELL HALVES ARE RUN AND THE POWERSHELL HALVES ARE RUN BY THE TWIN.
# This half cannot assume pwsh exists, and a check that SKIPPED the PowerShell
# side silently would report a green half of a pair as the whole pair.
# scripts/README.md carries the contract; check-twins runs both.
# ⚠ TRACKED PLUS UNTRACKED-NOT-IGNORED, because a script that has never been
# staged is exactly the one somebody has just written, and the one most likely
# to have got this wrong. check-routes takes the same view of a route file for
# the same reason.
SCRIPTS=$({ git ls-files -- 'scripts/*.sh'; git ls-files --others --exclude-standard -- 'scripts/*.sh'; } | LC_ALL=C sort -u)
[ -n "$SCRIPTS" ] || { printf 'check-exit-codes: no scripts found\n' >&2; exit 2; }

PROBLEMS=""
COUNT=0
CHECKED=0
for script in $SCRIPTS; do
  # ⛔ THIS SCRIPT IS NOT RUN BY ITSELF. Invoking it here would recurse: the
  # inner run would invoke the outer one again, and so on until the process
  # table says stop. Its own answer is asserted by the twin comparison and by
  # the fixture leg below.
  case "$script" in
    */check-exit-codes.sh) continue ;;
  esac
  CHECKED=$((CHECKED + 1))
  # ⛔ Unpiped, and the output is discarded rather than read: what is being
  # measured is the code, and a script that printed its usage is behaving
  # correctly. docs/conventions/shell.md section 2.
  sh "$script" "$UNKNOWN" >/dev/null 2>&1
  rc=$?
  if [ "$rc" != 2 ]; then
    PROBLEMS="$PROBLEMS  $script: exit $rc, and could-not-run is 2
"
    COUNT=$((COUNT + 1))
  fi
done

# ⭐ THE FIXTURE LEG, so this check has been seen to refuse. A guard whose test
# has never failed is theatre, and this one is cheap to plant: a script that
# exits 1 for an unknown argument is the exact defect, written into a scratch
# file that is removed afterwards.
FIXTURE_DIR="$REPO_ROOT/.tmp/check-exit-codes"
mkdir -p "$FIXTURE_DIR" 2>/dev/null || {
  printf 'check-exit-codes: cannot write %s\n' "$FIXTURE_DIR" >&2
  exit 2
}
FIXTURE="$FIXTURE_DIR/refuses-with-one.sh"
printf '#!/bin/sh\nexit 1\n' > "$FIXTURE"
sh "$FIXTURE" "$UNKNOWN" >/dev/null 2>&1
FIXTURE_RC=$?
rm -f "$FIXTURE"
if [ "$FIXTURE_RC" = 2 ]; then
  printf 'check-exit-codes: the fixture that exits 1 was read as 2, so this check cannot refuse\n' >&2
  exit 2
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-exit-codes/1","scripts":%s,"problems":%s}\n' "$CHECKED" "$COUNT"
elif [ "$COUNT" = 0 ]; then
  printf 'exit codes ok: %s script(s), each answers 2 for an argument it cannot act on\n' "$CHECKED"
else
  printf 'exit code check failed, %s script(s) did not answer 2:\n\n' "$COUNT" >&2
  printf '%s\n' "$PROBLEMS" >&2
  printf 'Exit 2 is could-not-run. 1 is it ran and the thing failed, and 0 is it ran\n' >&2
  printf 'and passed. docs/history/todo/ci.md, CI-07.\n' >&2
fi

[ "$COUNT" = 0 ] || exit 1
exit 0
