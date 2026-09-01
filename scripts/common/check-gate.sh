#!/bin/sh
# check-gate.sh - run every local gate this host can run, in one command.
#
# The defect this exists to catch is a gate that was skipped because it was the
# ninth thing to remember. Part (a) of docs/methodology/gate.md is a LIST, and a
# list run by hand is a list run in the order somebody remembers it, missing
# whichever entry was added last. The session that wrote this ran that list once
# per work item and re-typed it every time, which is exactly how one of them
# quietly stops being run.
#
# ⛔ IT IS NOT A SECOND SET OF RULES. Every line below shells out to a check
# that already exists and reads that check's own exit code. When this file and
# .github/workflows/ci.yml disagree about what runs, CI is the one that gates a
# push and this one is the defect.
#
# -- ⚠ A SKIPPED CHECK IS NOT A PASSED CHECK ---------------------------------
#
# Some of these need a tool that is not everywhere: shellcheck, jq, pwsh,
# PSScriptAnalyzer. A gate that silently dropped one of them and still printed
# green would be the "step that exits 0 having done nothing it was asked to do"
# row in docs/conventions/forbidden-patterns.md.
#
# So a missing tool is reported as SKIP, counted separately, named in the
# summary line and carried in --json as `skipped`. ⭐ The exit code is still 0,
# because "this host cannot run that one" is not a failure of the tree; the
# caller reads `skipped` to decide whether it wants that answer. CI runs on two
# hosts that between them have every tool, which is where the coverage comes
# from.
#
# -- ⚠ --fast, AND WHY IT IS A FLAG RATHER THAN THE DEFAULT ------------------
#
# check-twins runs BOTH halves of every pair, so it costs roughly as much as the
# rest of the gate put together. That is the right price before a push and the
# wrong one before each of a dozen commits, and a gate too slow to run is a gate
# that gets run once at the end.
#
# ⛔ THE FIGURES BELOW REPLACE A SET THAT WAS NOT MEASURED. The comment here
# used to carry a full run of 88 seconds ending "gate ok: all 15 checks
# passed", timed on a 4-CPU Linux container. That output could not have been
# produced: on the tree it names, check-docs reported eleven problems and
# check-twins reported twelve drifts, so the gate did not pass. TODO/tooling.md
# T-007 carries the correction. ⚠ A pasted output nobody produced is worse than
# no output, because a blank gets checked and a figure gets used.
#
# Measured 2026-08-31, Windows 11 (10.0.26200) on a 20-thread i7-12700H, Git
# Bash 5.3 and PowerShell 7.6.5, over 13 twin pairs, on a tree of 4,476 tracked
# files of which 4,389 are the reference corpus:
#
#   full run               403s
#   --fast                 106s
#   check-twins alone      294s
#
# ⚠ RE-MEASURED THE SAME DAY, AFTER THE GATE GREW. The workspace landed and this
# runner gained four checks: check-msrv and the three suite entries. check-twins
# gained two pairs, so it compares 15. Same machine, same shells:
#
#   --fast                 171s
#   full run               ⛔ NOT RE-TAKEN. The run went green, all 19 checks,
#                          and the timing line was lost when the shell holding
#                          it was killed. A figure nobody measured does not go
#                          here, so this row stays a dash until somebody times
#                          it.
#
# ⚠ The two --fast figures are 106s and 171s on one machine with the same tree
# plus a Rust workspace, and the difference is four checks of which three
# compile. They are separate runs on a machine doing other things.
#
# ⭐ So --fast removes about 300 of the 403 seconds. ⚠ Every figure is one run on
# a machine doing other things, and the three do not add up because they are
# separate runs. Each carries its conditions, which is what makes a later one
# comparable.
#
# ⚠ WINDOWS IS SLOW AT PROCESS SPAWNING and this gate spawns a great many, so a
# POSIX host will be much faster. That is a reason to re-measure there rather
# than to scale this number.
#
# ⚠ THIS IS NOT THE HOST THE FLAG EXISTS FOR. The twins exist because a native
# Windows PowerShell session may have no POSIX shell, and no figure has been
# taken there. A Windows number is still wanted and would be a different one.
#
# ⛔ --fast SKIPS check-twins. It does not weaken anything else, it is reported
# as a SKIP like every other, and the summary says so. The full run is what a
# push is gated on.
#
# -- ⛔ --strict, WHICH IS THE CI MODE --------------------------------------
#
# ⭐ It turns a SKIP into a failure. On a developer's machine a missing tool is
# a fact about the machine; on a runner the tools are installed on purpose, so a
# skip there means the install broke and the tree went unchecked.
#
# ⚠ IT WAS DOCUMENTED BEFORE IT EXISTED. docs/methodology/gate.md described this
# flag and neither half of this runner had it, so a CI job passing `--strict`
# would have been refused as an unknown argument and a job that stopped passing
# it would have gone green over any number of skips. That is the "a setting or
# flag that no code reads" row in docs/conventions/forbidden-patterns.md, in the
# runner the whole gate goes through.
#
# Usage:
#   sh scripts/common/check-gate.sh
#   sh scripts/common/check-gate.sh --fast
#   sh scripts/common/check-gate.sh --json
#   sh scripts/common/check-gate.sh --strict
#
# Exit codes: 0 everything that ran passed, 1 something failed, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FAST=0
STRICT=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --fast) FAST=1 ;;
    --strict) STRICT=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-gate: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-gate: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-gate: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ Every path below is relative to the repository root, so the scope of the
# gate does not depend on who called it. check-record.sh carries the same rule
# and the same reason.
cd "$REPO_ROOT" || { printf 'check-gate: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

HERE="scripts/common"
PASSED=0
FAILED=0
SKIPPED=0
FAILED_NAMES=""
SKIPPED_NAMES=""

# ⚠ Called directly, never on the right of a pipe. A function that increments a
# counter inside a pipeline runs in a subshell and every count it made is
# discarded on exit. docs/conventions/shell.md section 4.
record_pass() { PASSED=$((PASSED + 1)); [ "$JSON" = 1 ] || printf '  ok    %s\n' "$1"; }
record_fail() {
  FAILED=$((FAILED + 1))
  FAILED_NAMES="$FAILED_NAMES $1"
  [ "$JSON" = 1 ] || printf '  FAIL  %s (exit %s)\n' "$1" "$2"
}
record_skip() {
  SKIPPED=$((SKIPPED + 1))
  SKIPPED_NAMES="$SKIPPED_NAMES $1"
  [ "$JSON" = 1 ] || printf '  SKIP  %s -- %s\n' "$1" "$2"
}

# Run one check ONCE, read its exit code from the process that produced it, and
# show the output only when it failed. ⛔ Once: re-running it to inspect the
# output would be a second execution whose result could differ from the one that
# was scored.
check_simple() {
  name=$1
  shift
  out=$("$@" 2>&1)
  rc=$?
  if [ "$rc" = 0 ]; then
    record_pass "$name"
  else
    record_fail "$name" "$rc"
    [ "$JSON" = 1 ] || printf '%s\n' "$out" | sed 's/^/  | /'
  fi
}

[ "$JSON" = 1 ] || printf 'check-gate: %s\n\n' "$REPO_ROOT"

# -- ⭐ WHAT THIS GATE SKIPS WHEN check-twins IS RUNNING IT -------------------
#
# ⛔ MEASURED, NOT GUESSED. `check-twins --timings`, 2026-09-01, on one Windows
# 11 host: 970 seconds across twenty pairs, of which the `check-gate` row alone
# is 431. That row runs BOTH gates in full, and each gate re-runs the fourteen
# checks that ALREADY HAVE A ROW OF THEIR OWN. Fourteen rules were compared
# three times each, and the two extra times cost more than everything else in
# that file put together. TODO/tooling.md, TOOL-15.
#
# ⭐ So a gate running inside check-twins skips them. What that pair uniquely
# proves is untouched: the LIST each half runs, and the checks with no row of
# their own - the two lints, the analyzer, the three suite entries and the probe.
#
# ⛔ THE LIST GOING STALE IS COVERED, and for free. The pair compares `skipped`
# as well as `passed`, so a list that grows in one half and not the other fails
# that row. ⚠ Keep it identical to the PowerShell twin's, and to the
# `compare_pair` rows in check-twins.sh.
#
# ⚠ AN INTERNAL FLAG. CHECK_GATE_INNER is set by check-twins and by the
# recursion guard further down, and nothing else reads it. A gate run by hand
# runs everything.
COMPARED_DIRECTLY="check-docs check-markers check-one-home check-placeholders check-control-bytes check-record check-no-secrets check-vendor check-msrv check-corpus check-validate check-line-endings check-routes check-changelog check-workflows check-coverage"
compared_directly() {
  [ "${CHECK_GATE_INNER:-}" = "1" ] || return 1
  case " $COMPARED_DIRECTLY " in
    *" $1 "*)
      record_skip "$1" 'compared directly by check-twins; running it here compares one answer a third time'
      return 0
      ;;
    *) return 1 ;;
  esac
}

# -- the checks that are pure sh and always available ------------------------
compared_directly 'check-docs'          || check_simple 'check-docs'          sh "$HERE/check-docs.sh"
# ⛔ BOTH OF THESE, NOT ONE. check-docs reads markdown; check-markers reads
# every tracked text file and owns the character rule; check-one-home reads the
# documents against each other. In the two trees these checks were written in,
# the first reported clean while the other two had findings in the hundreds,
# which is what "run both" costs when it is advice rather than a line here.
compared_directly 'check-markers'       || check_simple 'check-markers'       sh "$HERE/check-markers.sh"
compared_directly 'check-one-home'      || check_simple 'check-one-home'      sh "$HERE/check-one-home.sh"
compared_directly 'check-placeholders'  || check_simple 'check-placeholders'  sh "$HERE/check-placeholders.sh"
compared_directly 'check-control-bytes' || check_simple 'check-control-bytes' sh "$HERE/check-control-bytes.sh"
compared_directly 'check-record'        || check_simple 'check-record'        sh "$HERE/check-record.sh"
compared_directly 'check-no-secrets'    || check_simple 'check-no-secrets'    sh "$HERE/check-no-secrets.sh" --public

# Run one check whose 2 means "could not run", and report that as a SKIP.
# ⛔ NOT AS A PASS. check-changelog's 2 is a pass because a project with no
# changelog has satisfied the rule vacuously; a host with no cargo has verified
# NOTHING about the manifest, and those are different facts.
check_skippable() {
  cs_name=$1
  cs_why=$2
  shift 2
  cs_out=$("$@" 2>&1)
  cs_rc=$?
  case "$cs_rc" in
    0) record_pass "$cs_name" ;;
    2) record_skip "$cs_name" "$cs_why" ;;
    *)
      record_fail "$cs_name" "$cs_rc"
      [ "$JSON" = 1 ] || printf '%s\n' "$cs_out" | sed 's/^/  | /'
      ;;
  esac
}

# -- the vendored trees, and the record that has to describe them ------------
# ⚠ ONLY THE OFFLINE LEG. --upstream fetches the recorded ref from the remote
# and a gate that needs the network fails on a machine that has none.
# ⚠ 2 is "could not run": jq is absent, or the tree vendors nothing at all.
# Neither has verified anything, so both are a SKIP rather than a pass.
compared_directly 'check-vendor' || check_skippable 'check-vendor' 'jq is absent, or this tree vendors nothing' \
  sh "$HERE/check-vendor.sh"

# -- the workspace, and the version floor it declares ------------------------
compared_directly 'check-msrv' || check_skippable 'check-msrv' 'cargo or jq is not on this host' \
  sh "$HERE/check-msrv.sh"

# -- the published corpus, and whether it was ever edited in place -----------
# ⚠ 2 is "could not run" twice over: there is no corpus at all, or the
# per-profile leg needed cargo and did not get it. Neither has verified
# anything about a profile, so both are a SKIP rather than a pass. ⛔ The git
# leg still decides a FAILURE: a published file edited after its first commit
# is exit 1 whether or not cargo was there.
compared_directly 'check-corpus' || check_skippable 'check-corpus' 'the corpus is empty, or cargo could not verify a profile' \
  sh "$HERE/check-corpus.sh"

# -- every published profile, coherent, and the derived files reproducible ---
# ⚠ 2 is "could not run" three ways over: there is no corpus, it holds no
# profile, or cargo is absent. None has validated anything, so all three are a
# SKIP rather than a pass. ⛔ A finding or a non-deterministic generator is
# exit 1 and fails the gate.
compared_directly 'check-validate' || check_skippable 'check-validate' \
  'the corpus is empty, or cargo could not validate a profile' \
  sh "$HERE/check-validate.sh"

# -- every workflow declares the four things that decide a run's output ------
# ⚠ 2 is "could not run": there is no .github/workflows directory, or it holds
# no .yml file. Neither has verified anything, so both are a SKIP.
compared_directly 'check-workflows' || check_skippable 'check-workflows' \
  'there is no workflow directory, or it holds no workflow' \
  sh "$HERE/check-workflows.sh" --assert-fail-fast-false

# -- which cells of the planned capture matrix have a profile ----------------
# ⚠ 2 is "could not run": there is no plan, or jq is absent. ⛔ It is not asked
# to REQUIRE any row here: what a run cares about is the run's business, and
# the capture workflow is where --require-rows is passed.
compared_directly 'check-coverage' || check_skippable 'check-coverage' \
  'there is no capture matrix, or jq is absent' \
  sh "$HERE/check-coverage.sh"

# -- the published route files, and the one byte a consumer should not have to
# strip. ⚠ 2 is "there is no route tree yet, or it holds no single-value file",
# which has verified nothing and is a SKIP rather than a pass.
compared_directly 'check-routes' || check_skippable 'check-routes' 'no published route tree, or it holds no single-value file' \
  sh "$HERE/check-routes.sh"

# ⚠ 2 is "could not run", which is the honest answer in a project with no
# CHANGELOG.md, and it is a pass here rather than a failure. ⛔ Collapsing 2 into
# 0 with `|| true` would hide a genuine exit 1 as well.
if ! compared_directly 'check-changelog'; then
  cl_out=$(sh "$HERE/check-changelog.sh" 2>&1)
  rc=$?
  if [ "$rc" = 0 ] || [ "$rc" = 2 ]; then
    record_pass 'check-changelog'
  else
    record_fail 'check-changelog' "$rc"
    [ "$JSON" = 1 ] || printf '%s\n' "$cl_out" | sed 's/^/  | /'
  fi
fi

# -- line endings, in the index AND in the working tree ----------------------
#
# ⛔ IT USED TO BE INLINE HERE, IN BOTH HALVES, AND THAT WAS THE DEFECT. Two
# copies of one rule computed in two languages, compared by nothing: the twin
# comparison covers a PAIR OF SCRIPTS, and a rule with no script of its own had
# no row. ⭐ It is a check now, with both halves and a row, like every other
# rule in this repository. TODO/tooling.md, TOOL-17.
#
# ⚠ 2 is "could not run": git tracks no file here. That has verified nothing, so
# it is a SKIP rather than a pass.
compared_directly 'check-line-endings' || check_skippable 'check-line-endings' 'git tracks no file in this repository' \
  sh "$HERE/check-line-endings.sh"

# ⛔ THE REFERENCE CORPUS IS OUT OF SCOPE FOR THE LINTERS, for the same reason
# the prose checks exempt it: `references/` is other projects' source, kept as
# the evidence behind docs/reference-sweeps/findings.md. Their style is not this
# project's defect, their scripts are not this project's to fix, and a gate that
# reports 37 findings in somebody else's tree is a gate nobody reads.
# ⚠ Its FILES are still checked for control bytes, because that defect makes a
# file invisible to review whoever wrote it.
# ⚠ TRACKED PLUS UNTRACKED-NOT-IGNORED, not tracked alone. A file that has
# never been staged is exactly when a new script is likeliest to be broken, and
# it is what the next `git add -A` would take. check-control-bytes.sh carries
# the same rule and the incident that produced it.
own_shell_files() {
  { git ls-files '*.sh'; git ls-files --others --exclude-standard '*.sh'; } \
    | sort -u | grep -v '^references/'
}
# ⚠ There is no `own_ps_files` here on purpose. This half does not enumerate
# `.ps1` files: check-powershell.ps1 does that, in the host that can parse them,
# and a second enumeration here would be a second place for the scope to be
# wrong.

# -- every shell script this project owns parses -----------------------------
# ⛔ Enumerated by git, not by a hardcoded list. A list is a thing somebody
# forgets to extend, and the script they forgot is the one that breaks.
fail=0
for f in $(own_shell_files); do
  sh -n "$f" 2>/dev/null || { fail=1; [ "$JSON" = 1 ] || printf '  | parse FAIL %s\n' "$f"; }
done
if [ "$fail" = 0 ]; then record_pass 'sh -n'; else record_fail 'sh -n' 1; fi

# -- shellcheck, when it is here ---------------------------------------------
if command -v shellcheck >/dev/null 2>&1; then
  fail=0
  for f in $(own_shell_files); do
    if ! sc_out=$(shellcheck -s sh "$f" 2>&1); then
      fail=1
      [ "$JSON" = 1 ] || { printf '  | shellcheck %s\n' "$f"; printf '%s\n' "$sc_out" | sed 's/^/  | /'; }
    fi
  done
  if [ "$fail" = 0 ]; then record_pass 'shellcheck'; else record_fail 'shellcheck' 1; fi
else
  record_skip 'shellcheck' 'shellcheck is not on PATH'
fi

# -- the PowerShell half, which needs a PowerShell ---------------------------
PWSH=""
for c in pwsh pwsh.exe powershell.exe; do
  if command -v "$c" >/dev/null 2>&1; then PWSH=$c; break; fi
done

# ⛔ SCORED AS TWO ENTRIES, because they can have different answers. The parse
# either ran or it did not; the analyzer is a MODULE that may be absent, and
# check-powershell exits 0 either way. One verdict for both is how a skipped
# analyzer reads as a passed check, which is what it did here once, before the
# fixed status line below existed.
if [ -n "$PWSH" ]; then
  ps_out=$("$PWSH" -NoProfile -File "$HERE/check-powershell.ps1" 2>&1)
  rc=$?
  case "$rc" in
    0) record_pass 'powershell parse' ;;
    2) record_skip 'powershell parse' 'the host reported it could not run' ;;
    *)
      record_fail 'powershell parse' "$rc"
      [ "$JSON" = 1 ] || printf '%s\n' "$ps_out" | sed 's/^/  | /'
      ;;
  esac
  # ⛔ The FIXED status line, never the prose above it. check-powershell.ps1
  # prints `analyzer=clean|skipped|issues:N` as its last line and documents that
  # this is the contract.
  case "$ps_out" in
    *'analyzer=skipped'*) record_skip 'PSScriptAnalyzer' 'not installed on this host' ;;
    *'analyzer=clean'*)   record_pass 'PSScriptAnalyzer' ;;
    *'analyzer=issues'*)  record_fail 'PSScriptAnalyzer' 1 ;;
    *) record_skip 'PSScriptAnalyzer' 'check-powershell printed no analyzer status line' ;;
  esac
else
  record_skip 'powershell parse' 'no pwsh, pwsh.exe or powershell.exe on PATH'
  record_skip 'PSScriptAnalyzer' 'no pwsh, pwsh.exe or powershell.exe on PATH'
fi

# -- part (a) is the SUITE as well as the checks -----------------------------
#
# ⛔ THREE ENTRIES, NOT ONE, and the reason is the one this file already makes
# about the parse and the analyzer: they can have different answers, and one
# verdict over three answers is how a skipped one reads as a passed one.
# docs/methodology/gate.md part (a) is "typecheck, lint, format, the full test
# suite", so the format and the lint are gates here rather than advice.
#
# ⚠ A SUITE OF ZERO TESTS PASSES VACUOUSLY, and today that is what this is: the
# workspace TOOL-01 created is eight empty crates. The line is here anyway,
# because the defect it removes is a gate that grows a suite line months after
# the first crate lands. TOOL-02 mutation-proved it by planting a failing test.
if command -v cargo >/dev/null 2>&1; then
  check_simple 'cargo fmt'    cargo fmt --all --check
  check_simple 'cargo clippy' cargo clippy --workspace --all-targets --all-features -- -D warnings
  # ⚠ No --all-targets here on purpose: it would drop the doc-tests, and a
  # doc-test is the one test that proves the documentation compiles.
  check_simple 'cargo test'   cargo test --workspace --all-features
else
  record_skip 'cargo fmt'    'cargo is not on this host'
  record_skip 'cargo clippy' 'cargo is not on this host'
  record_skip 'cargo test'   'cargo is not on this host'
fi

# -- the probe is not a gate, but it exiting non-zero is a real failure ------
check_simple 'doctor probe' sh scripts/doctor/doctor.sh --fast

# -- the twins, which need jq and a PowerShell ------------------------------
#
# ⛔ THIS PAIR RUNS THIS SCRIPT. check-twins.sh compares both halves of every
# twin and check-gate is one of them, so an unguarded call here is an infinite
# recursion: gate runs twins runs gate runs twins. It hung for ten minutes and
# left twenty stray shells holding their own files open, which is how this guard
# came to exist.
#
# CHECK_GATE_INNER is set for the child and honoured when this script finds it
# already set in its own environment. check-twins.sh exports it too, so a
# session that runs check-twins directly gets a gate one level deep rather than
# three. ⚠ It is an internal recursion guard and nothing else reads it.
if [ "$FAST" = 1 ]; then
  record_skip 'check-twins' '--fast was passed; it runs both halves of every pair'
elif [ "${CHECK_GATE_INNER:-}" = "1" ]; then
  record_skip 'check-twins' 'already inside check-twins; calling it here would recurse'
elif [ -z "$PWSH" ]; then
  record_skip 'check-twins' 'no PowerShell on PATH; it runs both halves of every pair'
elif ! command -v jq >/dev/null 2>&1; then
  record_skip 'check-twins' 'jq is not on PATH; it compares json'
else
  CHECK_GATE_INNER=1 check_simple 'check-twins' sh "$HERE/check-twins.sh"
fi

# -- report -----------------------------------------------------------------
TOTAL=$((PASSED + FAILED + SKIPPED))

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-gate/1","total":%s,"passed":%s,"failed":%s,"skipped":%s,"strict":%s}\n' \
    "$TOTAL" "$PASSED" "$FAILED" "$SKIPPED" "$STRICT"
  [ "$FAILED" -gt 0 ] && exit 1
  [ "$STRICT" = 1 ] && [ "$SKIPPED" -gt 0 ] && exit 1
  exit 0
fi

printf '\n'
if [ "$FAILED" -gt 0 ]; then
  printf 'GATE FAILED: %s of %s. Failed:%s\n' "$FAILED" "$TOTAL" "$FAILED_NAMES"
  [ "$SKIPPED" -gt 0 ] && printf 'Also skipped %s:%s\n' "$SKIPPED" "$SKIPPED_NAMES"
  exit 1
fi

if [ "$SKIPPED" -gt 0 ]; then
  if [ "$STRICT" = 1 ]; then
    printf 'GATE FAILED under --strict: %s passed, %s SKIPPED:%s\n' \
      "$PASSED" "$SKIPPED" "$SKIPPED_NAMES"
    printf 'A skipped check is not a passed check. On a runner the tools are\n'
    printf 'installed on purpose, so a skip means the install broke.\n'
    exit 1
  fi
  printf 'gate ok: %s passed, but %s SKIPPED on this host:%s\n' "$PASSED" "$SKIPPED" "$SKIPPED_NAMES"
  printf 'A skipped check is not a passed check. CI runs on two hosts that between\n'
  printf 'them have every tool; that is where the coverage for these comes from.\n'
  exit 0
fi

printf 'gate ok: all %s checks passed\n' "$PASSED"
exit 0
