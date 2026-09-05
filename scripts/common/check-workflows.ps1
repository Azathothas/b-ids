# check-workflows.ps1 - does every workflow declare the four things that decide
# whether a run produces data or nothing?
#
# ⭐ THE TWIN OF check-workflows.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a matrix that cancels every lane when one
# browser fails to download. That is the DEFAULT behaviour of a matrix, so a
# workflow acquires it by saying nothing, and the failure is invisible until the
# night a run that captured twenty-seven profiles publishes none of them.
#
# ⛔ The four rules: fail-fast: false on every job with a matrix; timeout-minutes
# on every job; if: always() on a job whose needs name a job that FANS OUT; and
# every `uses:` pinned to a 40-character commit rather than a tag. Plus a
# top-level `permissions:`, without which a lane inherits whatever the
# repository grants.
#
# ⚠ WHY THIS PARSES RATHER THAN GREPS. `fail-fast: false` appearing anywhere in a
# file says nothing about WHICH job carries it. And the always() rule needs to
# know which jobs fan out before it can judge which jobs need one, so every job
# is collected first and the verdicts are reached at the end.
#
# ⛔ IT IS NOT A YAML PARSER AND DOES NOT PRETEND TO BE.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-workflows.ps1
#   pwsh -NoProfile -File scripts/common/check-workflows.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-workflows.ps1 -AssertFailFastFalse
#   pwsh -NoProfile -File scripts/common/check-workflows.ps1 -Fixtures DIR
#
# Exit codes: 0 clean, 1 a workflow is missing one of them, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    [switch]$AssertFailFastFalse,
    [string]$Fixtures = '',
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. `pwsh -File` reports a parameter-binding failure as 1, which is
    # this project's code for "it ran and the thing failed"; the POSIX twin
    # exits 2 for the same input. Measured across every pair 2026-09-02:
    # 22 of 22 disagreed. docs/history/todo/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-workflows: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-workflows: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-workflows: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location -LiteralPath $root
try {
    $workflowDir = '.github/workflows'

    if ($Fixtures) {
        if (-not (Test-Path -LiteralPath $Fixtures -PathType Container)) {
            [Console]::Error.WriteLine("check-workflows: no directory at $Fixtures")
            exit 2
        }
        $files = @(Get-ChildItem -LiteralPath $Fixtures -Recurse -File -Filter '*.yml' |
                Sort-Object FullName | ForEach-Object { $_.FullName })
    }
    elseif (-not (Test-Path -LiteralPath $workflowDir -PathType Container)) {
        if ($Json) {
            Write-Output '{"schema":"check-workflows/1","workflows":0,"jobs":0,"problems":0}'
        }
        else {
            [Console]::Error.WriteLine("check-workflows: there is no $workflowDir directory, so nothing was checked.")
        }
        exit 2
    }
    else {
        $tracked = @(& git ls-files -- "$workflowDir/*.yml")
        $untracked = @(& git ls-files --others --exclude-standard -- "$workflowDir/*.yml")
        $files = @(($tracked + $untracked) | Where-Object { $_ } | Sort-Object -Unique)
    }

    # ⛔ A SCOPE THAT YIELDED NO FILE HAS VERIFIED NOTHING.
    if ($files.Count -eq 0) {
        if ($Json) {
            Write-Output '{"schema":"check-workflows/1","workflows":0,"jobs":0,"problems":0}'
        }
        else {
            [Console]::Error.WriteLine('check-workflows: no workflow file in scope, so nothing was checked.')
        }
        exit 2
    }

    $problems = [System.Collections.Generic.List[string]]::new()
    $jobCount = 0

    foreach ($file in $files) {
        $lines = @(Get-Content -LiteralPath $file)
        $permissions = $false
        $inJobs = $false
        $inStrategy = $false
        $jobs = [System.Collections.Generic.List[hashtable]]::new()
        $current = $null

        foreach ($line in $lines) {
            if ($line -match '^permissions:') { $permissions = $true }
            if ($line -match 'uses:\s*(\S+)@(\S+)') {
                $ref = $Matches[2]
                if ($ref -notmatch '^[0-9a-f]{40}$') {
                    $problems.Add("${file}: uses $($Matches[1])@$ref, which is not a 40-character commit. A moved tag runs code nobody reviewed.")
                }
            }
            if ($line -match '^jobs:\s*$') { $inJobs = $true; continue }
            if ($inJobs -and $line -match '^[a-zA-Z]') { $inJobs = $false; continue }
            if ($inJobs -and $line -match '^  ([A-Za-z0-9_.-]+):\s*$') {
                $current = @{
                    name = $Matches[1]; timeout = $false; matrix = $false
                    failFast = ''; needs = ''; cond = ''
                }
                $jobs.Add($current)
                $jobCount++
                $inStrategy = $false
                continue
            }
            if ($null -ne $current) {
                if ($line -match '^    timeout-minutes:') { $current.timeout = $true; $inStrategy = $false; continue }
                if ($line -match '^    needs:') { $current.needs = $line; $inStrategy = $false; continue }
                if ($line -match '^    if:') { $current.cond = $line; $inStrategy = $false; continue }
                if ($line -match '^    strategy:') { $inStrategy = $true; continue }
                if ($line -match '^    [A-Za-z0-9_-]+:') { $inStrategy = $false }
                if ($inStrategy -and $line -match '^      fail-fast:\s*(\S+)') { $current.failFast = $Matches[1]; continue }
                if ($inStrategy -and $line -match '^      matrix:') { $current.matrix = $true; continue }
            }
        }

        foreach ($j in $jobs) {
            if (-not $j.timeout) {
                $problems.Add("${file}: job $($j.name): no timeout-minutes. A hung step holds a runner for the platform default.")
            }
            if ($j.matrix -and $j.failFast -ne 'false' -and $AssertFailFastFalse) {
                $problems.Add("${file}: job $($j.name): declares a matrix and does not declare fail-fast: false. One lane failing cancels its siblings.")
            }
            # ⛔ THE always() RULE IS ABOUT COLLECTING, NOT ABOUT NEEDING. It fires
            # only where a job depends on one that FANS OUT.
            if ($j.needs -ne '' -and $j.cond -notmatch 'always\(\)') {
                foreach ($u in $jobs) {
                    if ($u.matrix -and $j.needs.Contains($u.name)) {
                        $problems.Add("${file}: job $($j.name): needs the fan-out job $($u.name) and does not run regardless. A collect job that only runs when every lane passed publishes nothing on the nights it matters.")
                        break
                    }
                }
            }
        }
        if (-not $permissions) {
            $problems.Add("${file}: declares no top-level permissions. The default is whatever the repository grants.")
        }
    }

    if ($Json) {
        Write-Output ('{"schema":"check-workflows/1","workflows":' + $files.Count +
                      ',"jobs":' + $jobCount + ',"problems":' + $problems.Count + '}')
        if ($problems.Count -gt 0) { exit 1 }
        exit 0
    }

    if ($problems.Count -gt 0) {
        Write-Output ("workflow check failed, " + $problems.Count + " problem(s) over " +
                      $files.Count + " workflow(s) and $jobCount job(s):")
        Write-Output ''
        $problems | ForEach-Object { Write-Output "  $_" }
        exit 1
    }

    $suffix = if ($AssertFailFastFalse) { ', every matrix declares fail-fast: false' } else { '' }
    Write-Output ("workflows ok: " + $files.Count + " file(s), $jobCount job(s)$suffix")
    exit 0
}
finally {
    Pop-Location
}
