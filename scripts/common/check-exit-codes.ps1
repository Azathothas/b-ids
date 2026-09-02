# check-exit-codes.ps1 - does every script in this tree report "could not run"
# as 2, on both halves of every pair?
#
# ⭐ THE TWIN OF check-exit-codes.sh, and the half that matters most, because
# the PowerShell side is where the defect was. TODO/ci.md, CI-07.
#
# The defect this exists to catch is a check that fails because a machine cannot
# run it. A capture job on a runner with no browser must not fail the build, and
# a check that returns 1 for "I could not run" is a check somebody disables.
#
# -- ⛔ WHAT IT ACTUALLY MEASURES, AND WHY THIS INPUT ------------------------
#
# Every script here is invoked with an argument no script accepts. That is the
# one state EVERY script in the tree can be put into from outside, without a
# missing tool, a missing browser or a network.
#
# ⭐ MEASURED 2026-09-02 AND IT IS WHY THIS CHECK EXISTS. Every POSIX half
# returned 2 and every PowerShell half returned 1, 22 pairs of 22: `pwsh -File`
# reports a parameter-binding failure as 1, and 1 is this project's code for
# "it ran and the thing failed". ⛔ The fix is a remaining-arguments parameter
# in every param() block, which catches what would otherwise fail to bind.
#
# ⚠ AND THE PARAMETER IS NAMED $UnboundArguments RATHER THAN $Rest. PowerShell
# variables are case-insensitive, and check-markers.ps1 already used a local
# $rest: the first spelling shadowed it and took that script from a clean run to
# "Cannot convert value { to type System.Int32".
#
# ⛔ IT DOES NOT ACCEPT 0. A script that ignored an argument it does not
# understand and ran anyway is worse than one that refused.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-exit-codes.ps1
#   pwsh -NoProfile -File scripts/common/check-exit-codes.ps1 -Json
#
# Exit codes: 0 clean, 1 a script did not answer 2, 2 could not run.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. This script holds the rule it checks.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-exit-codes: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-exit-codes: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-exit-codes: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root

if (-not (Get-Command pwsh -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-exit-codes: pwsh not found')
    exit 2
}

# ⛔ THE ARGUMENT NO SCRIPT ACCEPTS, spelled so it cannot become one by
# accident. ⚠ A PowerShell parameter name rather than a POSIX flag, because
# that is the shape that used to bind and fail with 1.
$unknown = '-BIdsCheckExitCodesNotARealArgument'

# ⚠ TRACKED PLUS UNTRACKED-NOT-IGNORED, because a script that has never been
# staged is exactly the one somebody has just written.
$tracked = & git ls-files -- 'scripts/*.ps1'
$untracked = & git ls-files --others --exclude-standard -- 'scripts/*.ps1'
$scripts = @($tracked) + @($untracked) | Where-Object { $_ } | Sort-Object -Unique
if ($LASTEXITCODE -ne 0 -or -not $scripts) {
    [Console]::Error.WriteLine('check-exit-codes: no scripts found')
    exit 2
}

$problems = @()
$checked = 0
foreach ($script in ($scripts | Sort-Object)) {
    # ⛔ THIS SCRIPT IS NOT RUN BY ITSELF. Invoking it here would recurse.
    if ($script -like '*/check-exit-codes.ps1') { continue }
    $checked++
    # ⛔ The output is discarded rather than read: what is measured is the code,
    # and a script that printed its usage is behaving correctly.
    $null = & pwsh -NoProfile -File $script $unknown 2>&1
    if ($LASTEXITCODE -ne 2) {
        $problems += ('  {0}: exit {1}, and could-not-run is 2' -f $script, $LASTEXITCODE)
    }
}

# ⭐ THE FIXTURE LEG, so this check has been seen to refuse.
$fixtureDir = Join-Path $root '.tmp/check-exit-codes'
$null = New-Item -ItemType Directory -Force -Path $fixtureDir
$fixture = Join-Path $fixtureDir 'refuses-with-one.ps1'
Set-Content -LiteralPath $fixture -Value 'exit 1' -Encoding utf8
$null = & pwsh -NoProfile -File $fixture $unknown 2>&1
$fixtureCode = $LASTEXITCODE
Remove-Item -LiteralPath $fixture -Force -ErrorAction SilentlyContinue
if ($fixtureCode -eq 2) {
    [Console]::Error.WriteLine('check-exit-codes: the fixture that exits 1 was read as 2, so this check cannot refuse')
    exit 2
}

if ($Json) {
    # ⚠ CONCATENATED rather than formatted. A -f template carries brace
    # placeholders, and scripts/common/check-placeholders reads those as a
    # placeholder that survived into a real file. It is right to: the two are
    # indistinguishable from outside.
    Write-Output ('{"schema":"check-exit-codes/1","scripts":' + $checked +
        ',"problems":' + $problems.Count + '}')
} elseif ($problems.Count -eq 0) {
    Write-Output ('exit codes ok: {0} script(s), each answers 2 for an argument it cannot act on' -f $checked)
} else {
    [Console]::Error.WriteLine(('exit code check failed, {0} script(s) did not answer 2:' -f $problems.Count))
    [Console]::Error.WriteLine('')
    foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('Exit 2 is could-not-run. 1 is it ran and the thing failed, and 0 is it ran')
    [Console]::Error.WriteLine('and passed. TODO/ci.md, CI-07.')
}

if ($problems.Count -ne 0) { exit 1 }
exit 0
