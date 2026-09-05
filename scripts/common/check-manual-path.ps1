# check-manual-path.ps1 - does every automated job name the command a person
# runs instead, and does that command resolve on this host?
#
# ⭐ THE TWIN OF check-manual-path.sh. docs/history/todo/ci.md, CI-08.
#
# ⛔ A PROJECT WHOSE ONLY PATH TO A CAPTURE IS ONE PROVIDER'S AUTOMATION
# DEGRADES TO NOTHING WHEN THAT PROVIDER DOES. The test is one sentence: if the
# provider disappeared, the project degrades to "somebody runs one command".
#
# ⛔ EVERY JOB DECLARES ITS OWN MANUAL EQUIVALENT, as a `# manual: <command>`
# comment inside the job block. ⚠ It lives beside the job rather than in a table
# somewhere else: a list of equivalents in a second file is a value in two places
# with no check that they agree.
#
# ⚠ WHAT "RESOLVES" MEANS, AND WHY IT IS NOT "RUNS": a script in this tree must
# exist, and the program a command starts with must be on PATH. ⛔ One job of
# the nine is a fuzz lane that runs a hundred thousand cases and another launches
# a browser, so a check that executed them is a check nobody runs.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-manual-path.ps1
#   pwsh -NoProfile -File scripts/common/check-manual-path.ps1 -Json
#
# Exit codes: 0 every job names a command that resolves, 1 one does not,
#             2 could not run.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-manual-path: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-manual-path: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-manual-path: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root

# ⛔ TRACKED AND UNTRACKED BOTH. A workflow that was written and never staged
# escaped this check entirely, so the one moment a new job's manual line is
# missing is the one moment nothing looked. Measured 2026-09-02: this reported
# 9 jobs over a tree carrying 10.
$tracked = & git ls-files -- '.github/workflows/*.yml' '.github/workflows/*.yaml'
$untracked = & git ls-files --others --exclude-standard -- '.github/workflows/*.yml' '.github/workflows/*.yaml'
$workflows = @(@($tracked) + @($untracked) | Where-Object { $_ } | Sort-Object -Unique)
if ($workflows.Count -eq 0) {
    [Console]::Error.WriteLine('check-manual-path: no workflows, so nothing was checked')
    exit 2
}

$problems = @()
$jobs = 0
$named = 0

foreach ($workflow in $workflows) {
    # ⛔ IT READS THE INDENTATION rather than grepping. `# manual:` anywhere in a
    # file says nothing about WHICH job carries it.
    $inJobs = $false
    $job = ''
    $manual = ''
    $pairs = @()
    foreach ($line in (Get-Content -LiteralPath $workflow)) {
        if ($line -match '^jobs:') { $inJobs = $true; continue }
        if ($inJobs -and $line -match '^[^\s]') { $inJobs = $false }
        if (-not $inJobs) { continue }
        if ($line -match '^  ([A-Za-z0-9_-]+):') {
            if ($job) { $pairs += [pscustomobject]@{ Job = $job; Manual = $manual } }
            $job = $Matches[1]
            $manual = ''
            continue
        }
        if ($job -and $line -match '^\s*# manual:\s*(.+)$' -and -not $manual) {
            $manual = $Matches[1].Trim()
        }
    }
    if ($job) { $pairs += [pscustomobject]@{ Job = $job; Manual = $manual } }

    foreach ($pair in $pairs) {
        $jobs++
        if (-not $pair.Manual) {
            $problems += ('  ' + $workflow + ": job '" + $pair.Job + "' names no manual equivalent")
            continue
        }
        $named++
        $words = $pair.Manual -split '\s+'
        $program = $words[0]
        $script = $words | Where-Object { $_ -like '*/*' } | Select-Object -First 1
        if (-not (Get-Command $program -ErrorAction SilentlyContinue)) {
            $problems += ('  ' + $workflow + ": job '" + $pair.Job + "' names '" + $program +
                "', which is not on PATH here")
            continue
        }
        if ($script) {
            if (-not (Test-Path -LiteralPath $script)) {
                $problems += ('  ' + $workflow + ": job '" + $pair.Job + "' names " + $script +
                    ', which this tree does not have')
                continue
            }
            if ($script -like '*.ps1') {
                $errors = $null
                $null = [System.Management.Automation.Language.Parser]::ParseFile(
                    (Resolve-Path -LiteralPath $script), [ref]$null, [ref]$errors)
                if ($errors -and $errors.Count -gt 0) {
                    $problems += ('  ' + $workflow + ": job '" + $pair.Job + "' names " + $script +
                        ', which does not parse')
                }
            }
        }
    }
}

if ($Json) {
    Write-Output ('{"schema":"check-manual-path/1","jobs":' + $jobs +
        ',"named":' + $named + ',"problems":' + $problems.Count + '}')
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -eq 0) {
    Write-Output ('manual path ok: ' + $jobs + ' job(s), each names a command that resolves here')
    exit 0
}

[Console]::Error.WriteLine('manual path check failed, ' + $problems.Count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('Every automated job names the command a person runs instead, as a')
[Console]::Error.WriteLine('manual comment inside the job. docs/history/todo/ci.md, CI-08.')
exit 1
