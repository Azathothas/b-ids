# check-coverage.ps1 - which cells of the planned capture matrix have a profile,
# and which have none?
#
# ⭐ THE TWIN OF check-coverage.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a corpus that looks full because nobody
# wrote down what is missing. Coverage decides whether this project is useful,
# and it decides whether automated merging is possible at all: agreement across
# two independent sources is only satisfiable when one build is captured on more
# than one host.
#
# ⭐ ONE MATRIX, TWO READERS. .github/capture-matrix.json is the plan;
# .github/workflows/capture.yml builds its job matrix from it and this reads it
# to say what landed, so the plan and the report cannot disagree.
#
# ⛔ A PLANNED CELL THAT WAS NOT ATTEMPTED IS REPORTED, NEVER DROPPED. A report
# listing only what was tried cannot show what is missing.
#
# ⚠ THE CORPUS SIDE COMES FROM corpus/v1/index.json, which is derived from the
# tree and asserted against it by `b-ids-corpus verify`. Walking the corpus
# directory here would be a second implementation of the layout rule.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-coverage.ps1
#   pwsh -NoProfile -File scripts/common/check-coverage.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-coverage.ps1 -RequireRows chrome,edge
#
# Exit codes: 0 every required row has a capture, 1 one does not,
#             2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    [string]$RequireRows = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-coverage: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-coverage: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location -LiteralPath $root
try {
    $planPath = '.github/capture-matrix.json'
    $indexPath = 'corpus/v1/index.json'

    if (-not (Test-Path -LiteralPath $planPath -PathType Leaf)) {
        [Console]::Error.WriteLine("check-coverage: there is no $planPath, so there is no plan to report against")
        exit 2
    }
    try { $plan = Get-Content -LiteralPath $planPath -Raw | ConvertFrom-Json }
    catch {
        [Console]::Error.WriteLine("check-coverage: $planPath is not readable as a plan")
        exit 2
    }

    # An absent index is a corpus with nothing in it, which is a real state and
    # not an error. A MALFORMED one is exit 2.
    $captured = @()
    if (Test-Path -LiteralPath $indexPath -PathType Leaf) {
        try { $index = Get-Content -LiteralPath $indexPath -Raw | ConvertFrom-Json }
        catch {
            [Console]::Error.WriteLine("check-coverage: $indexPath is not readable as an index")
            exit 2
        }
        $captured = @($index.profiles | ForEach-Object {
                "$($_.browser.ToLowerInvariant())/$($_.channel)/$($_.platform)"
            })
    }

    $cells = @($plan.cells)
    # ⛔ A PLAN WITH NO CELLS HAS REPORTED NOTHING.
    if ($cells.Count -eq 0) {
        if ($Json) {
            Write-Output '{"schema":"check-coverage/1","cells":0,"captured":0,"absent":0,"not_attempted":0,"missing_required":0}'
        }
        else {
            [Console]::Error.WriteLine('check-coverage: the plan holds no cell, so nothing was reported.')
        }
        exit 2
    }

    $rows = @()
    $nCaptured = 0
    $nAbsent = 0
    $nNotAttempted = 0
    foreach ($cell in $cells) {
        $key = "$($cell.browser)/$($cell.channel)/$($cell.platform)"
        $n = @($captured | Where-Object { $_ -eq $key }).Count
        if ($n -gt 0) { $state = 'captured'; $nCaptured++ }
        elseif ($cell.enabled) { $state = 'absent'; $nAbsent++ }
        else { $state = 'not-attempted'; $nNotAttempted++ }
        $req = if ($cell.required) { ' required' } else { '' }
        $rows += ('  {0,-14} {1,-34} {2} profile(s){3}' -f $state, $key, $n, $req)
    }

    $missing = @()
    if ($RequireRows) {
        foreach ($want in ($RequireRows -split ',')) {
            $w = $want.Trim()
            if (-not $w) { continue }
            $hit = @($captured | Where-Object { $_.Split('/')[0] -eq $w }).Count
            if ($hit -eq 0) {
                $missing += "  ${w}: no capture at all, on any channel or platform"
            }
        }
    }

    if ($Json) {
        Write-Output ('{"schema":"check-coverage/1","cells":' + $cells.Count +
                      ',"captured":' + $nCaptured + ',"absent":' + $nAbsent +
                      ',"not_attempted":' + $nNotAttempted +
                      ',"missing_required":' + $missing.Count + '}')
        if ($missing.Count -gt 0) { exit 1 }
        exit 0
    }

    Write-Output ("coverage over " + $cells.Count + " planned cell(s):")
    Write-Output ''
    $rows | ForEach-Object { Write-Output $_ }
    Write-Output ''
    Write-Output "$nCaptured captured, $nAbsent absent, $nNotAttempted not attempted."

    if ($missing.Count -gt 0) {
        Write-Output ''
        Write-Output ("coverage check failed, " + $missing.Count + " required row(s) with no capture:")
        Write-Output ''
        $missing | ForEach-Object { Write-Output $_ }
        exit 1
    }
    exit 0
}
finally {
    Pop-Location
}
