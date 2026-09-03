# check-cold-start.ps1 - is the cold-start job still cold, and does everything a
# cold pipeline names still resolve on this host?
#
# ⭐ THE TWIN OF check-cold-start.sh. TODO/driver.md, DRIVER-09, is why a script
# in this directory does not land without one, and this pair earns it for a
# second reason: the Windows half of the cold-start workflow RUNS this half, so
# a twin that drifted would be probing a different list from the one the job
# depends on.
#
# ⛔ EVERY WARM RUN PASSES OVER A BROKEN COLD PATH. TODO/ci.md, CI-05.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the workflow exists, runs on a schedule, and can be dispatched by hand;
#   2. ⛔ NO CACHE OF ANY KIND;
#   3. its concurrency group is its own;
#   4. every stage carries an `id`, and the report step names every one of them
#      and runs `if: always()`;
#   5. ⭐ THE RESOLUTION PROBE, over the same list the twin carries.
#
# ⚠ THE READING IS THIS HALF'S OWN: -match over the file's lines where the twin
# uses awk, sed and grep.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-cold-start.ps1
#   pwsh -NoProfile -File scripts/common/check-cold-start.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-cold-start.ps1 -Resolve
#   pwsh -NoProfile -File scripts/common/check-cold-start.ps1 -Resolve -RequireTools
#
# Exit codes: 0 the job is still cold and the probe found what it names,
# 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [switch]$Resolve,
    [switch]$RequireTools,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-cold-start: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-cold-start: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-cold-start: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root

$workflow = '.github/workflows/cold-start.yml'

# ⛔ THE PROGRAMS A COLD PIPELINE NEEDS, IN ONE PLACE and identical to the twin's.
# ⚠ A browser is deliberately NOT here: `b-ids-driver resolve` exits 2 on a host
# with none, and that is a fact about the host rather than a broken cold path.
$tools = @('git', 'cargo', 'rustc', 'rustup', 'jq', 'awk', 'sed', 'grep', 'tar')

$problems = New-Object System.Collections.ArrayList
$stages = 0
$found = 0
$missing = 0
$firstMissing = ''
$report = New-Object System.Collections.ArrayList

foreach ($tool in $tools) {
    if (Get-Command $tool -ErrorAction SilentlyContinue) {
        $found++
        [void]$report.Add("  ok    $tool")
    }
    else {
        $missing++
        if (-not $firstMissing) { $firstMissing = $tool }
        [void]$report.Add("  ABSENT $tool")
    }
}

if ($Resolve) {
    if ($Json) {
        $bad = 0
        if ($RequireTools) { $bad = $missing }
        Write-Output ('{"schema":"check-cold-start/1","tools":' + $tools.Count +
                      ',"found":' + $found + ',"missing":' + $missing +
                      ',"stages":0,"problems":' + $bad + '}')
        if ($RequireTools -and $missing -gt 0) { exit 1 }
        exit 0
    }
    Write-Output "cold start probe over $($tools.Count) program(s):"
    Write-Output ''
    $report | ForEach-Object { Write-Output $_ }
    Write-Output ''
    if ($missing -eq 0) {
        Write-Output 'every program a cold pipeline names is on this host.'
        exit 0
    }
    if ($RequireTools) {
        [Console]::Error.WriteLine("`u{26D4} the cold path breaks at the first absent program: $firstMissing")
        [Console]::Error.WriteLine('Every warm run passes over a broken cold path. TODO/ci.md, CI-05.')
        exit 1
    }
    Write-Output "`u{26A0} $missing absent, first $firstMissing. On this host that is a fact about the host;"
    Write-Output '  --require-tools is what makes it a failure, and the runner passes it.'
    exit 0
}

function Show-Failure {
    [Console]::Error.WriteLine("cold start check failed, $($problems.Count) problem(s):")
    [Console]::Error.WriteLine('')
    $problems | ForEach-Object { [Console]::Error.WriteLine($_) }
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('Every warm run passes over a broken cold path. TODO/ci.md, CI-05.')
}

if (-not (Test-Path -LiteralPath $workflow -PathType Leaf)) {
    [void]$problems.Add("  there is no $workflow, so nothing ever runs this pipeline from cold")
    Show-Failure
    exit 1
}

$lines = @(Get-Content -LiteralPath $workflow)

function Get-TopBlock {
    param([string[]]$Text, [string]$Key)
    $out = New-Object System.Collections.ArrayList
    $inside = $false
    foreach ($line in $Text) {
        if ($line -match "^$Key`:\s*$") { $inside = $true; continue }
        if ($inside -and $line -match '^[a-zA-Z]') { $inside = $false }
        if ($inside) { [void]$out.Add($line) }
    }
    return , $out.ToArray()
}

# -- 1: the workflow, and its triggers ---------------------------------------
$onBlock = Get-TopBlock -Text $lines -Key 'on'
if (-not ($onBlock -match '^  schedule:')) {
    [void]$problems.Add('  the cold-start workflow is not on a schedule, and a cold path nobody runs is one nobody checks')
}
if (-not ($onBlock -match '^  workflow_dispatch:')) {
    [void]$problems.Add('  the cold-start workflow cannot be dispatched by hand')
}

# -- 2: no cache of any kind -------------------------------------------------
$live = @($lines | Where-Object { $_ -notmatch '^\s*#' })
$caches = @($live | Where-Object {
        $_ -match 'actions/cache' -or $_ -match 'rust-cache' -or $_ -match 'sccache' -or
        $_ -match 'RUSTC_WRAPPER' -or $_ -match '^\s*cache:'
    }).Count
if ($caches -ne 0) {
    [void]$problems.Add("  $caches line(s) name a cache, and a cold-start job that shares one has stopped being one")
}

# -- 3: its own concurrency group --------------------------------------------
$group = ''
foreach ($line in (Get-TopBlock -Text $lines -Key 'concurrency')) {
    if ($line -match '^  group:\s*(.+)$') { $group = $Matches[1]; break }
}
if ($group -notmatch 'cold-start') {
    $shown = if ($group) { $group } else { 'absent' }
    [void]$problems.Add("  the concurrency group is $shown, which is not this workflow's own")
}

# -- 4: every stage has an id, and the report names every one ----------------
$ids = New-Object System.Collections.ArrayList
foreach ($line in $lines) {
    if ($line -match '^        id:\s*(\S+)\s*$') { [void]$ids.Add($Matches[1]) }
}
$stages = $ids.Count
if ($stages -lt 6) {
    [void]$problems.Add("  $stages stage(s) carry an id, and a pipeline reported at that resolution names nothing useful")
}

$reportStep = New-Object System.Collections.ArrayList
$inReport = $false
foreach ($line in $lines) {
    if ($line -match '^      - name: what this cold start reached') { $inReport = $true }
    if ($inReport) { [void]$reportStep.Add($line) }
}
if ($reportStep.Count -eq 0) {
    [void]$problems.Add('  there is no report step, so a failed run does not name the stage that broke')
}
if (-not ($reportStep -match 'if: always\(\)')) {
    [void]$problems.Add('  the report step does not run if: always(), so a red job says nothing about which stage went red')
}
foreach ($id in $ids) {
    if ($id -eq 'report') { continue }
    if (-not ($reportStep -match [regex]::Escape("steps.$id.outcome"))) {
        [void]$problems.Add("  the report step does not name the $id stage, so a failure there is not reported by name")
    }
}

# -- 5: the probe, folded into the verdict -----------------------------------
if ($RequireTools -and $missing -gt 0) {
    [void]$problems.Add("  the cold path breaks at the first absent program: $firstMissing")
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-cold-start/1","tools":' + $tools.Count +
                  ',"found":' + $found + ',"missing":' + $missing +
                  ',"stages":' + $stages + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output "cold start ok: $stages stage(s), each named by the report step, no cache of any kind,"
    Write-Output "  and $found of $($tools.Count) program(s) present on this host."
    if ($missing -ne 0) {
        Write-Output "  `u{26A0} A SKIP IS NOT A PASS: $missing absent here, first $firstMissing. The runner passes --require-tools."
    }
    Write-Output "  `u{26D4} Nothing was built, captured or published by this check."
    exit 0
}

Show-Failure
exit 1
