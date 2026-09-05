# check-sources.ps1 - does every external question get asked more than one way,
# and is a disagreement reported rather than resolved?
#
# ⭐ THE TWIN OF check-sources.sh. docs/history/todo/ci.md, CI-06.
#
# ⛔ EVERY EXTERNAL DEPENDENCY WILL ONE DAY ANSWER DIFFERENTLY, and a corpus
# that stopped updating in year two was not worth building.
#
# ⭐ TWO SOURCES THAT DISAGREE ARE THE MOST VALUABLE SIGNAL HERE. One instance
# is already measured: two first-party version sources disagreed and the
# disagreement WAS the defect. docs/inherited-claims.md section 7.
#
# -- ⛔ THE THREE THINGS IT ASSERTS ------------------------------------------
#
#   1. PER-SOURCE ISOLATION. Every source appears with its own answer or its own
#      error. A run that dropped a source that failed would report a smaller
#      sample than it took.
#   2. A SILENT SOURCE IS NOT A FAILURE. A vendor endpoint being down degrades
#      the run rather than ending it.
#   3. A DISAGREEMENT IS FLAGGED, NEVER RESOLVED SILENTLY.
#
# ⚠ IT NEVER FETCHES ANYTHING, and -Report takes the same JSON from a file.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-sources.ps1
#   pwsh -NoProfile -File scripts/common/check-sources.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-sources.ps1 -Report FILE
#
# Exit codes: 0 the contract holds, 1 it does not, 2 could not run.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [string]$Report = '',
    [string]$Channel = 'stable',
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-sources: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-sources: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-sources: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root

$fixtures = Join-Path $root '.tmp/check-sources'
$null = New-Item -ItemType Directory -Force -Path $fixtures

if (-not $Report) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        [Console]::Error.WriteLine('check-sources: cargo not found')
        exit 2
    }
    $payload = & cargo run -q -p b-ids-driver -- versions --channel $Channel --json 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $payload) {
        [Console]::Error.WriteLine('check-sources: no source answered at all, so nothing could be checked')
        exit 2
    }
    $Report = Join-Path $fixtures 'report.json'
    Set-Content -LiteralPath $Report -Value $payload -Encoding utf8
}
if (-not (Test-Path -LiteralPath $Report)) {
    [Console]::Error.WriteLine('check-sources: no file at ' + $Report)
    exit 2
}

# ⛔ THE FIXTURE LEG RUNS ON EVERY INVOCATION, so this check has been seen to
# refuse. Two fixtures, one per clause.
Set-Content -LiteralPath (Join-Path $fixtures 'silent-without-a-reason.json') -Encoding utf8 -Value (@'
{"answers":[{"source":"releases","version":"9.0.0.1","error":null},{"source":"chrome-for-testing","version":null,"error":null}],"chosen":{"version":"9.0.0.1","fraction":1,"highest_known":"9.0.0.1","highest_fraction":1},"disagreement":false}
'@)
Set-Content -LiteralPath (Join-Path $fixtures 'disagreement-unflagged.json') -Encoding utf8 -Value (@'
{"answers":[{"source":"releases","version":"9.0.0.1","error":null},{"source":"chrome-for-testing","version":"9.0.0.2","error":null}],"chosen":{"version":"9.0.0.1","fraction":1,"highest_known":"9.0.0.2","highest_fraction":1},"disagreement":false}
'@)

function Get-SourceReport {
    param([string]$Path)
    $report = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    $answers = @($report.answers)
    $problems = @()

    if ($answers.Count -lt 2) {
        $problems += ('only ' + $answers.Count +
            ' source(s) in the report, and one source is a single point of failure')
    }
    # ⛔ PER-SOURCE ISOLATION. A source carries an answer or a reason, never
    # neither: a source that vanished from the report is a sample nobody counted.
    foreach ($answer in $answers) {
        if (-not $answer.source) { $problems += 'an answer with no source name'; continue }
        if (-not $answer.version -and -not $answer.error) {
            $problems += ($answer.source + ' reported neither a version nor a reason')
        }
    }
    # ⛔ A DISAGREEMENT IS FLAGGED. Two different versions with the flag off is
    # one source silently preferred, which this entry forbids by name.
    $versions = @($answers | Where-Object { $_.version } | ForEach-Object { $_.version } | Sort-Object -Unique)
    if ($versions.Count -gt 1 -and -not $report.disagreement) {
        $problems += ('sources answered ' + ($versions -join ' and ') +
            ' and disagreement is false, which is one source silently preferred')
    }
    if ($versions.Count -le 1 -and $report.disagreement) {
        $answeredText = if ($versions.Count) { $versions[0] } else { 'nothing' }
        $problems += ('disagreement is true and the sources answered ' + $answeredText)
    }
    $answered = @($answers | Where-Object { $_.version }).Count
    [pscustomobject]@{
        Answers      = $answers.Count
        Answered     = $answered
        Silent       = $answers.Count - $answered
        Disagreement = [bool]$report.disagreement
        Problems     = $problems
    }
}

# ⭐ THE FIXTURE LEG FIRST. A check that cannot refuse must not report a pass.
foreach ($pair in @(
    @{ Name = 'silent-without-a-reason'; Expect = 'neither a version nor a reason' },
    @{ Name = 'disagreement-unflagged'; Expect = 'silently preferred' })) {
    $seen = Get-SourceReport -Path (Join-Path $fixtures ($pair.Name + '.json'))
    if (-not ($seen.Problems | Where-Object { $_ -like ('*' + $pair.Expect + '*') })) {
        [Console]::Error.WriteLine('check-sources: the ' + $pair.Name +
            ' fixture was read as clean, so this check cannot refuse')
        exit 2
    }
}

$real = Get-SourceReport -Path $Report

if ($Json) {
    Write-Output ('{"schema":"check-sources/1","sources":' + $real.Answers +
        ',"answered":' + $real.Answered +
        ',"silent":' + $real.Silent +
        ',"disagreement":' + $real.Disagreement.ToString().ToLower() +
        ',"problems":' + $real.Problems.Count + '}')
    if ($real.Problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($real.Problems.Count -eq 0) {
    Write-Output ('sources ok: ' + $real.Answers + ' source(s), ' + $real.Answered +
        ' answered, ' + $real.Silent + ' did not, disagreement=' +
        $real.Disagreement.ToString().ToLower())
    exit 0
}

[Console]::Error.WriteLine('source contract failed, ' + $real.Problems.Count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $real.Problems) { [Console]::Error.WriteLine('  ' + $problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('  Two sources that disagree are the most valuable signal this')
[Console]::Error.WriteLine('  project produces. Record both, publish both, never pick.')
[Console]::Error.WriteLine('  docs/history/todo/ci.md, CI-06.')
exit 1
