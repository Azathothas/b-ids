# check-pr-body.ps1 - would a scheduled run that found a change open a pull
# request a reviewer can act on, and would a run that found nothing stay silent?
#
# ⭐ THE TWIN OF check-pr-body.sh. TODO/ci.md, CI-04, and TODO/driver.md,
# DRIVER-09, is why a script in this directory does not land without one.
#
# ⛔ AN ISSUE IS A REQUEST FOR SOMEBODY ELSE TO DO WORK. A pull request with the
# work already in it is the deliverable.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the suite that owns the body's contents is present, case by case, and
#      passes. ⚠ THE ASSERTIONS ARE THE CRATE'S;
#   2. ⭐ END TO END OVER THE REAL CORPUS, the generator opens ONE request for
#      the run, carrying every route that moved, and its body carries every
#      section, the validator's output and a named list of what the run could
#      not do. ⛔ One branch per ROUTE was withdrawn on 2026-09-04;
#   3. ⛔ A NO-OP CHANGE OPENS NOTHING AT ALL;
#   4. a run file that does not parse is a refusal rather than a body with a
#      blank in it.
#
# ⛔ -Fixture IS REQUIRED, for the reason `latest` requires --assert-stable.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-pr-body.ps1 -Fixture
#   pwsh -NoProfile -File scripts/common/check-pr-body.ps1 -Fixture -Json
#
# Exit codes: 0 the body is what CI-04 asks for, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [switch]$Fixture,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-pr-body: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $Fixture) {
    [Console]::Error.WriteLine('check-pr-body: -Fixture is required. There is no pull request to check:')
    [Console]::Error.WriteLine('  this checks a generator against a fixture, and a run with no argument')
    [Console]::Error.WriteLine('  would read as though it had checked a real one.')
    exit 2
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-pr-body: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-pr-body: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-pr-body: cargo not found')
    exit 2
}

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED TO BE THIS TREE. Until
# PUB-13 this passed the repository root as `--after`, which stopped holding a
# corpus the day corpus/ left the default branch. TODO/publish.md, PUB-13.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-pr-body: no corpus is reachable, so the generator has nothing to run over')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()

$suite = Join-Path $root 'crates/b-ids-corpus/tests/pull_request.rs'
if (-not (Test-Path -LiteralPath $suite)) {
    [Console]::Error.WriteLine('check-pr-body: no suite at ' + $suite)
    exit 2
}

# ⛔ THE CASES ARE NAMED HERE AND ASSERTED THERE, so a suite that lost one is
# caught by this check rather than by nobody.
$want = @(
    'pull_request_a_body_carries_every_fact_the_model_holds',
    'pull_request_a_no_op_change_opens_nothing_at_all',
    'pull_request_a_body_names_what_the_run_could_not_do',
    'pull_request_two_runs_over_one_change_produce_identical_text',
    'pull_request_one_branch_per_run_carries_every_route_that_moved',
    'pull_request_a_run_identifier_that_is_not_a_branch_name_is_made_into_one',
    'pull_request_the_merge_conditions_can_fail_and_say_which',
    'pull_request_every_condition_holding_is_reachable_rather_than_impossible',
    'pull_request_the_labels_carry_the_class_the_confidence_and_the_subject',
    'pull_request_a_new_route_says_it_has_nothing_to_diff_against'
)

$problems = @()
$suiteText = Get-Content -LiteralPath $suite -Raw
foreach ($name in $want) {
    if ($suiteText -notmatch [regex]::Escape('fn ' + $name)) {
        $problems += ('  ' + $name + ' is not in the suite')
    }
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-pr-body-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -Recurse -Force -LiteralPath $out }
New-Item -ItemType Directory -Force -Path (Join-Path $out 'empty') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $out 'requests') | Out-Null

$testLog = Join-Path $out 'tests.log'
& cargo test -q -p b-ids-corpus --test pull_request > $testLog 2>&1
$rcT = $LASTEXITCODE
$cases = 0
$running = Get-Content -LiteralPath $testLog | Where-Object { $_ -match '^running (\d+) tests' } | Select-Object -First 1
if ($running -and $running -match '^running (\d+) tests') { $cases = [int]$Matches[1] }
if ($rcT -ne 0) { $problems += '  the suite failed. Its output is in .tmp/check-pr-body-ps/tests.log' }
if ($cases -lt $want.Count) {
    $problems += ('  the suite ran ' + $cases + ' case(s) where at least ' + $want.Count + ' were expected')
}

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-pr-body: the corpus crate did not build')
    exit 2
}
$bin = Join-Path $root 'target' | Join-Path -ChildPath 'debug' | Join-Path -ChildPath 'b-ids-corpus'
if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine('check-pr-body: ' + $bin + ' is not there')
    exit 2
}

# ⚠ THE RUN FACTS ARE A FIXTURE, and every field is filled: the generator
# refuses a file missing one.
$unavailable = 'the macos lane has no runner in this plan'
$runJson = @'
{
  "workflow": "capture.yml",
  "run_id": "a fixture run",
  "images": [["linux64", "a fixture image"]],
  "harness": "a fixture harness",
  "command": "sh experiments/10-first-profile.sh --headless --browser chrome",
  "unavailable": ["the macos lane has no runner in this plan"],
  "validator_output": "a fixture validator line",
  "validator_findings": 0,
  "formats_round_trip": true
}
'@
$runPath = Join-Path $out 'run.json'
Set-Content -LiteralPath $runPath -Value $runJson -NoNewline

# -- 2: end to end, over the real corpus ------------------------------------
$generateLog = Join-Path $out 'generate.log'
& $bin pull-request --before (Join-Path $out 'empty') --after $corpusRoot --run $runPath --out (Join-Path $out 'requests') > $generateLog 2>&1
$rcG = $LASTEXITCODE
if ($rcG -ne 0) {
    $problems += ('  the generator exited ' + $rcG + '. Its output is in .tmp/check-pr-body-ps/generate.log')
}
$status = Get-Content -LiteralPath $generateLog | Where-Object { $_ -like 'corpus=pull-request *' } | Select-Object -Last 1
if (-not $status) {
    [Console]::Error.WriteLine('check-pr-body: the generator printed no status line')
    exit 1
}
$requests = [int][regex]::Match($status, 'requests:(\d+)').Groups[1].Value
$auto = [int][regex]::Match($status, 'auto:(\d+)').Groups[1].Value
if ($requests -lt 1) { $problems += '  a corpus with profiles produced no request at all' }

$headings = @(
    '## What changed',
    '## The fields that differ',
    '## Where this capture came from',
    '## The validator',
    '## What this run could not do',
    '## Reproducing this',
    '## Merging'
)
foreach ($dir in (Get-ChildItem -LiteralPath (Join-Path $out 'requests') -Directory)) {
    foreach ($file in @('branch', 'title', 'body.md', 'labels', 'mergeable')) {
        $path = Join-Path $dir.FullName $file
        if (-not (Test-Path -LiteralPath $path) -or (Get-Item -LiteralPath $path).Length -eq 0) {
            $problems += ('  ' + $dir.Name + ': ' + $file + ' was not written, or is empty')
        }
    }
    $bodyPath = Join-Path $dir.FullName 'body.md'
    if (-not (Test-Path -LiteralPath $bodyPath)) { continue }
    $body = Get-Content -LiteralPath $bodyPath -Raw
    foreach ($heading in $headings) {
        if (-not $body.Contains($heading)) {
            $problems += ('  ' + $dir.Name + ': the body has no ' + $heading + ' section')
        }
    }
    if (-not $body.Contains('a fixture validator line')) {
        $problems += ('  ' + $dir.Name + ": the body does not carry the validator's output")
    }
    if (-not $body.Contains($unavailable)) {
        $problems += ('  ' + $dir.Name + ': the body does not name what the run could not do')
    }
    if (-not $body.Contains('a fixture run')) {
        $problems += ('  ' + $dir.Name + ': the body does not carry the run identifier')
    }
    $labels = Get-Content -LiteralPath (Join-Path $dir.FullName 'labels')
    foreach ($prefix in @('confidence:', 'class:', 'subject:')) {
        if (-not ($labels | Where-Object { $_ -like ($prefix + '*') })) {
            $problems += ('  ' + $dir.Name + ': no ' + $prefix.TrimEnd(':') + ' label')
        }
    }
}

# -- 3: the no-op, end to end -----------------------------------------------
$none = Join-Path $out 'none'
New-Item -ItemType Directory -Force -Path $none | Out-Null
$noopLog = Join-Path $out 'noop.log'
& $bin pull-request --before $corpusRoot --after $corpusRoot --run $runPath --out $none > $noopLog 2>&1
$rcN = $LASTEXITCODE
if ($rcN -ne 0) { $problems += ('  the no-op run exited ' + $rcN) }
$noopStatus = Get-Content -LiteralPath $noopLog | Where-Object { $_ -like 'corpus=pull-request *' } | Select-Object -Last 1
$noop = [int][regex]::Match("$noopStatus", 'requests:(\d+)').Groups[1].Value
if ($noop -ne 0) {
    $problems += ('  a no-op change produced ' + $noop + ' request(s), and it must produce none')
}
if ((Get-ChildItem -LiteralPath $none -Force | Measure-Object).Count -gt 0) {
    $problems += '  a no-op change wrote files into the output directory'
}

# -- 4: a run file that does not parse is a refusal --------------------------
$half = Join-Path $out 'half.json'
Set-Content -LiteralPath $half -Value '{"workflow":"capture.yml"}' -NoNewline
& $bin pull-request --before (Join-Path $out 'empty') --after $corpusRoot --run $half --out (Join-Path $out 'half') > (Join-Path $out 'half.log') 2>&1
$rcH = $LASTEXITCODE
if ($rcH -ne 2) {
    $problems += ('  a run file missing fields exited ' + $rcH + ' where 2 was expected')
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-pr-body/1","cases":' + $cases + ',"requests":' + $requests + ',"auto":' + $auto + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output ('pr body ok: ' + $cases + ' suite case(s), ' + $requests + ' request(s) generated from the corpus,')
    Write-Output ('  ' + $auto + ' of them mergeable without a human, every body carrying its seven')
    Write-Output '  sections, and a no-op change opening nothing at all.'
    exit 0
}

[Console]::Error.WriteLine('pr body check failed, ' + $count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A pull request with the work in it is the deliverable, and one that')
[Console]::Error.WriteLine('silently omits a field is worse than one that says it could not capture')
[Console]::Error.WriteLine('it. TODO/ci.md, CI-04.')
exit 1
