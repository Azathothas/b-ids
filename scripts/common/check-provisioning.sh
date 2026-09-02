#!/bin/sh
# check-provisioning.sh - does the provisioning tool refuse what it must, and
# does it provision what it promises?
#
# ⛔ THE TOOL PURGES BROWSERS. Every refusal it makes is what stands between a
# developer's machine and losing one, so the refusals are checked on EVERY host
# and the provisioning itself only where the machine is thrown away.
# TODO/driver.md, DRIVER-08.
#
# -- ⛔ WHAT IS CHECKED EVERYWHERE -------------------------------------------
#
#   1. a machine missing EITHER of the two conditions is refused, exit 2, and
#      neither variable alone arms it;
#   2. a route that is not one of the two is refused;
#   3. --route vendor with --version is refused, because the vendor channel
#      serves the current build and cannot honour one;
#   4. --route for-testing with no --version is refused, because the index is
#      keyed by build;
#   5. --plan names a purge, a fetch, an install and a confirm for this platform
#      and RUNS NOTHING;
#   6. --plan for the for-testing route names an index step as well, because
#      that route reads one and the vendor route does not.
#
# -- ⛔ WHAT IS CHECKED ONLY ON A DISPOSABLE MACHINE -------------------------
#
#   7. the tool purges, `resolve` then exits 2, it installs, and `resolve` then
#      reports a version. ⚠ Skipped loudly elsewhere, never silently: a check
#      that quietly passed where it could not run is the shape that makes a
#      green suite mean nothing.
#   8. with --build, the same for the for-testing route at an EXACT build, and
#      `resolve` must then report that build and no other. ⛔ Skipped loudly
#      without --build rather than run against a version spelled here: a build
#      hardcoded in a check goes stale, and the matrix cell is where a build is
#      named. TODO/driver.md, DRIVER-08.
#
# Usage:
#   sh scripts/common/check-provisioning.sh
#   sh scripts/common/check-provisioning.sh --json
#   sh scripts/common/check-provisioning.sh --build 151.0.7922.76
#
# Exit codes: 0 every refusal held, 1 one did not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
BUILD=""
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --build) shift; BUILD="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-provisioning: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-provisioning: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-provisioning: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-provisioning: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

TOOL="$REPO_ROOT/scripts/common/provision-browser.sh"
[ -f "$TOOL" ] || { printf 'check-provisioning: no tool at %s\n' "$TOOL" >&2; exit 2; }

PROBLEMS=""
COUNT=0
CHECKED=0

# ⛔ Each refusal is run with the environment set exactly as the case needs, and
# the exit code is read from the process, unpiped.
#
# ⛔ NOTHING HERE EVER BYPASSES A GUARD. A test that has to disable one runs
# against a COPY of the tool under the ignored scratch directory, never against
# the file on a machine the guard protects. ⚠ That rule is written down
# because it was broken here on 2026-09-02 and the purge path ran on a
# developer laptop. docs/HISTORY/README.md carries the incident.
refuses() {
  r_why=$1
  r_expect=$2
  r_disposable=$3
  shift 3
  CHECKED=$((CHECKED + 1))
  case "$r_disposable" in
    # ⚠ BOTH set is the only environment in which the tool would act, and it is
    # used only for the argument refusals, which exit before anything runs.
    both) r_out=$(B_IDS_DISPOSABLE=1 CI=true sh "$TOOL" "$@" 2>&1) ;;
    ci) r_out=$(B_IDS_DISPOSABLE='' CI=true sh "$TOOL" "$@" 2>&1) ;;
    disposable) r_out=$(B_IDS_DISPOSABLE=1 CI='' sh "$TOOL" "$@" 2>&1) ;;
    *) r_out=$(B_IDS_DISPOSABLE='' CI='' sh "$TOOL" "$@" 2>&1) ;;
  esac
  r_rc=$?
  if [ "$r_rc" != "2" ]; then
    PROBLEMS="$PROBLEMS  $r_why: exit $r_rc, expected 2
"
    COUNT=$((COUNT + 1))
    return
  fi
  case "$r_out" in
    *"$r_expect"*) ;;
    *)
      PROBLEMS="$PROBLEMS  $r_why: refused without saying '$r_expect'
"
      COUNT=$((COUNT + 1))
      ;;
  esac
}

# 1. ⛔ THE THREE THAT PROTECT A LAPTOP, and all three are checked because one
# condition holding is not the same fact as both being required.
refuses 'neither condition set' 'BOTH are required' none \
  --browser chrome --route vendor
refuses 'the runner marker alone' 'BOTH are required' ci \
  --browser chrome --route vendor
refuses 'the disposable flag alone' 'BOTH are required' disposable \
  --browser chrome --route vendor

# 2. a route that does not exist
refuses 'a route that is not one of the two' 'vendor or for-testing' both \
  --browser chrome --route apt

# 3. ⛔ A VERSION THE CHANNEL CANNOT HONOUR. Accepting it and ignoring it would
# install the current build while a caller believed it had pinned one.
refuses 'a version on the vendor route' 'CURRENT build only' both \
  --browser chrome --route vendor --version 151.0.7922.173

# 4. the index is keyed by build, so a route with no build cannot use it
refuses 'no version on the for-testing route' 'needs --version' both \
  --browser chrome --route for-testing

# 5. ⛔ --plan RUNS NOTHING, and it is what a person reads before letting this
# near a machine. It is checked on a host that is NOT disposable, so a plan that
# had started purging would be caught by the guard rather than by this line.
CHECKED=$((CHECKED + 1))
plan=$(B_IDS_DISPOSABLE='' CI='' sh "$TOOL" --plan --browser chrome --route vendor 2>&1)
plan_rc=$?
if [ "$plan_rc" != "0" ]; then
  PROBLEMS="$PROBLEMS  --plan: exit $plan_rc, expected 0
"
  COUNT=$((COUNT + 1))
else
  for word in purge fetch install confirm; do
    case "$plan" in
      *"$word"*) ;;
      *)
        PROBLEMS="$PROBLEMS  --plan: names no $word step for this platform
"
        COUNT=$((COUNT + 1))
        ;;
    esac
  done
fi

# 6. ⛔ THE for-testing PLAN NAMES AN INDEX STEP, and the vendor plan does not.
# The two routes differ by where the build comes from, so a plan that described
# them identically would be a plan nobody could use to tell them apart.
CHECKED=$((CHECKED + 1))
ftplan=$(B_IDS_DISPOSABLE='' CI='' sh "$TOOL" --plan --browser chrome --route for-testing \
  --version 151.0.7922.76 2>&1)
ftplan_rc=$?
if [ "$ftplan_rc" != "0" ]; then
  PROBLEMS="$PROBLEMS  --plan for-testing: exit $ftplan_rc, expected 0
"
  COUNT=$((COUNT + 1))
else
  for word in purge index fetch install confirm; do
    case "$ftplan" in
      *"$word"*) ;;
      *)
        PROBLEMS="$PROBLEMS  --plan for-testing: names no $word step for this platform
"
        COUNT=$((COUNT + 1))
        ;;
    esac
  done
fi

# 7. ⚠ THE PROVISIONING ITSELF, only where the machine is thrown away.
PROVISIONED="skipped"
if [ "${B_IDS_DISPOSABLE:-}" = "1" ] && [ -n "${CI:-}" ]; then
  CHECKED=$((CHECKED + 1))
  if B_IDS_DISPOSABLE=1 sh "$TOOL" --browser chrome --route vendor > "$REPO_ROOT/.tmp/provisioned.txt" 2>&1
  then
    PROVISIONED="ok"
  else
    PROVISIONED="failed"
    PROBLEMS="$PROBLEMS  provisioning: the tool did not purge and install cleanly
"
    COUNT=$((COUNT + 1))
  fi
fi

# 8. ⛔ THE EXACT-BUILD ROUTE, which is the one that can be asked for a build
# and answer with a different one. The confirm step inside the tool is what
# refuses that, and this is where it is exercised.
ACQUIRED="skipped"
if [ -n "$BUILD" ] && [ "${B_IDS_DISPOSABLE:-}" = "1" ] && [ -n "${CI:-}" ]; then
  CHECKED=$((CHECKED + 1))
  if B_IDS_DISPOSABLE=1 sh "$TOOL" --browser chrome --route for-testing --version "$BUILD" \
    > "$REPO_ROOT/.tmp/acquired.txt" 2>&1
  then
    ACQUIRED="ok"
  else
    ACQUIRED="failed"
    PROBLEMS="$PROBLEMS  for-testing: the tool did not acquire and confirm $BUILD
"
    COUNT=$((COUNT + 1))
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-provisioning/2","checks":%s,"problems":%s,"provisioned":"%s","acquired":"%s"}\n' \
    "$CHECKED" "$COUNT" "$PROVISIONED" "$ACQUIRED"
elif [ "$COUNT" = 0 ]; then
  printf 'provisioning ok: %s check(s), every refusal held, provisioning %s\n' \
    "$CHECKED" "$PROVISIONED"
  if [ "$PROVISIONED" = "skipped" ]; then
    printf '  SKIP the provisioning itself: this machine is not disposable, so nothing\n'
    printf '  was purged. A workflow on a disposable runner is where that leg runs,\n'
    printf '  and .github/workflows/provision.yml is what runs it.\n'
  fi
  if [ "$ACQUIRED" = "skipped" ]; then
    printf '  SKIP the exact-build route: it runs with --build on a disposable\n'
    printf '  machine, and the build is named by the matrix cell rather than here.\n'
  fi
else
  printf 'provisioning check failed, %s problem(s):\n\n' "$COUNT" >&2
  printf '%s\n' "$PROBLEMS" >&2
  printf 'Every refusal here stands between a machine and losing its browser.\n' >&2
  printf 'TODO/driver.md, DRIVER-08.\n' >&2
fi

[ "$COUNT" = 0 ] || exit 1
exit 0
