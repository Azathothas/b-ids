# check-provisioning.ps1 - does the provisioning tool refuse what it must, and
# does it provision what it promises?
#
# ⭐ THE TWIN OF check-provisioning.sh. docs/history/todo/driver.md, DRIVER-08 and DRIVER-09.
#
# ⛔ THE TOOL PURGES BROWSERS. Every refusal it makes is what stands between a
# developer's machine and losing one, so the refusals are checked on EVERY host
# and the provisioning itself only where the machine is thrown away.
#
# ⚠ THIS HALF DRIVES provision-browser.ps1 AND NOTHING ELSE. A half that
# shelled out to the other language would report a green half of a pair as the
# whole pair on the host that most needs this one. scripts/README.md carries
# that as the check contract.
#
# -- ⛔ WHAT IS CHECKED EVERYWHERE -------------------------------------------
#
#   1. a machine missing EITHER of the two conditions is refused, exit 2, and
#      neither variable alone arms it;
#   2. a route that is not one of the two is refused;
#   3. -Route vendor with -Version is refused, because the vendor channel
#      serves the current build and cannot honour one;
#   4. -Route for-testing with no -Version is refused, because the index is
#      keyed by build;
#   5. -Plan names a purge, a fetch, an install and a confirm for this platform
#      and RUNS NOTHING;
#   6. -Plan for the for-testing route names an index step as well, because
#      that route reads one and the vendor route does not.
#
# -- ⛔ WHAT IS CHECKED ONLY ON A DISPOSABLE MACHINE -------------------------
#
#   7. the tool purges, `resolve` then exits 2, it installs, and `resolve` then
#      reports a version. ⚠ Skipped loudly elsewhere, never silently: a check
#      that quietly passed where it could not run is the shape that makes a
#      green suite mean nothing.
#   8. with -Build, the same for the for-testing route at an EXACT build, and
#      `resolve` must then report that build and no other. ⛔ Skipped loudly
#      without -Build rather than run against a version spelled here: a build
#      hardcoded in a check goes stale, and the matrix cell is where a build is
#      named.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-provisioning.ps1
#   pwsh -NoProfile -File scripts/common/check-provisioning.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-provisioning.ps1 -Build 151.0.7922.76
#
# Exit codes: 0 every refusal held, 1 one did not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [string]$Build = '',
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-provisioning: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-provisioning: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-provisioning: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root

$tool = Join-Path $root 'scripts' | Join-Path -ChildPath 'common' | Join-Path -ChildPath 'provision-browser.ps1'
if (-not (Test-Path -LiteralPath $tool)) {
    [Console]::Error.WriteLine('check-provisioning: no tool at ' + $tool)
    exit 2
}

$pwshExe = (Get-Process -Id $PID).Path
if (-not $pwshExe) {
    [Console]::Error.WriteLine('check-provisioning: cannot find the shell that is running this')
    exit 2
}

$problems = @()
$checked = 0

# ⛔ Each refusal is run with the environment set exactly as the case needs, and
# the exit code is read from the process, unpiped.
#
# ⛔ NOTHING HERE EVER BYPASSES A GUARD. A test that has to disable one runs
# against a COPY of the tool under the ignored scratch directory, never against
# the file on a machine the guard protects. ⚠ That rule is written down
# because it was broken here on 2026-09-02 and the purge path ran on a
# developer laptop. docs/history/README.md carries the incident.
function Test-Refusal {
    param(
        [string]$Why,
        [string]$Expect,
        [string]$Environment,
        [string[]]$Arguments
    )

    $keptDisposable = $env:B_IDS_DISPOSABLE
    $keptCi = $env:CI
    switch ($Environment) {
        # ⚠ BOTH set is the only environment in which the tool would act, and it
        # is used only for the argument refusals, which exit before anything runs.
        'both' { $env:B_IDS_DISPOSABLE = '1'; $env:CI = 'true' }
        'ci' { $env:B_IDS_DISPOSABLE = ''; $env:CI = 'true' }
        'disposable' { $env:B_IDS_DISPOSABLE = '1'; $env:CI = '' }
        default { $env:B_IDS_DISPOSABLE = ''; $env:CI = '' }
    }

    $out = ''
    try {
        $out = (& $pwshExe -NoProfile -File $tool @Arguments 2>&1 | Out-String)
        $rc = $LASTEXITCODE
    } finally {
        $env:B_IDS_DISPOSABLE = $keptDisposable
        $env:CI = $keptCi
    }

    $script:checked = $script:checked + 1
    if ($rc -ne 2) {
        $script:problems += ('  ' + $Why + ': exit ' + $rc + ', expected 2')
        return
    }
    if ($out -notlike ('*' + $Expect + '*')) {
        $script:problems += ('  ' + $Why + ": refused without saying '" + $Expect + "'")
    }
}

# 1. ⛔ THE THREE THAT PROTECT A LAPTOP, and all three are checked because one
# condition holding is not the same fact as both being required.
Test-Refusal 'neither condition set' 'BOTH are required' 'none' @('-Browser', 'chrome', '-Route', 'vendor')
Test-Refusal 'the runner marker alone' 'BOTH are required' 'ci' @('-Browser', 'chrome', '-Route', 'vendor')
Test-Refusal 'the disposable flag alone' 'BOTH are required' 'disposable' @('-Browser', 'chrome', '-Route', 'vendor')

# 2. a route that does not exist
Test-Refusal 'a route that is not one of the two' 'vendor or for-testing' 'both' @('-Browser', 'chrome', '-Route', 'apt')

# 3. ⛔ A VERSION THE CHANNEL CANNOT HONOUR. Accepting it and ignoring it would
# install the current build while a caller believed it had pinned one.
Test-Refusal 'a version on the vendor route' 'CURRENT build only' 'both' @('-Browser', 'chrome', '-Route', 'vendor', '-Version', '151.0.7922.173')

# 4. the index is keyed by build, so a route with no build cannot use it
Test-Refusal 'no version on the for-testing route' 'needs -Version' 'both' @('-Browser', 'chrome', '-Route', 'for-testing')

# 5. ⛔ -Plan RUNS NOTHING, and it is what a person reads before letting this
# near a machine. It is checked on a host that is NOT disposable, so a plan that
# had started purging would be caught by the guard rather than by this line.
$checked = $checked + 1
$keptDisposable = $env:B_IDS_DISPOSABLE
$keptCi = $env:CI
$env:B_IDS_DISPOSABLE = ''
$env:CI = ''
try {
    $plan = (& $pwshExe -NoProfile -File $tool -Plan -Browser chrome -Route vendor 2>&1 | Out-String)
    $planRc = $LASTEXITCODE
} finally {
    $env:B_IDS_DISPOSABLE = $keptDisposable
    $env:CI = $keptCi
}
if ($planRc -ne 0) {
    $problems += ('  -Plan: exit ' + $planRc + ', expected 0')
} else {
    # ⛔ A LINE WHOSE FIRST FIELD IS THE STEP, never the word anywhere in the
    # output. ⚠ Measured 2026-09-02 by planting the defect: with every index
    # line removed from a COPY of the tool, a substring search still passed,
    # because the fetch line reads "the zip that index names for the build".
    $planLines = $plan -split "`r?`n"
    foreach ($word in @('purge', 'fetch', 'install', 'confirm')) {
        if (-not ($planLines | Where-Object { ($_ -split '\s+')[0] -ceq $word })) {
            $problems += ('  -Plan: names no ' + $word + ' step for this platform')
        }
    }
}

# 6. ⛔ THE for-testing PLAN NAMES AN INDEX STEP, and the vendor plan does not.
# The two routes differ by where the build comes from, so a plan that described
# them identically would be a plan nobody could use to tell them apart.
$checked = $checked + 1
$keptDisposable = $env:B_IDS_DISPOSABLE
$keptCi = $env:CI
$env:B_IDS_DISPOSABLE = ''
$env:CI = ''
try {
    $ftPlan = (& $pwshExe -NoProfile -File $tool -Plan -Browser chrome -Route for-testing `
        -Version 151.0.7922.76 2>&1 | Out-String)
    $ftPlanRc = $LASTEXITCODE
} finally {
    $env:B_IDS_DISPOSABLE = $keptDisposable
    $env:CI = $keptCi
}
if ($ftPlanRc -ne 0) {
    $problems += ('  -Plan for-testing: exit ' + $ftPlanRc + ', expected 0')
} else {
    $ftPlanLines = $ftPlan -split "`r?`n"
    foreach ($word in @('purge', 'index', 'fetch', 'install', 'confirm')) {
        if (-not ($ftPlanLines | Where-Object { ($_ -split '\s+')[0] -ceq $word })) {
            $problems += ('  -Plan for-testing: names no ' + $word + ' step for this platform')
        }
    }
}

# 7. ⚠ THE PROVISIONING ITSELF, only where the machine is thrown away.
# ⛔ THE SCRATCH DIRECTORY IS CREATED, NEVER ASSUMED. It is ignored by git, so a
# fresh checkout does not have it, and a redirect into a directory that is not
# there fails before the command on its left ever runs.
#
# ⚠ MEASURED 2026-09-02, provision.yml run 33627230086, the first time this ran
# on a runner at all. ⛔ This half printed NOTHING and exited 1: the redirect
# threw under $ErrorActionPreference = 'Stop' and the summary below is written
# at the end, so the throw skipped it. A failing acceptance that says nothing is
# not diagnosable, which is why each leg is wrapped now.
$scratch = Join-Path $root '.tmp'
New-Item -ItemType Directory -Force -Path $scratch | Out-Null

$provisioned = 'skipped'
if ($env:B_IDS_DISPOSABLE -eq '1' -and -not [string]::IsNullOrEmpty($env:CI)) {
    $checked = $checked + 1
    try {
        & $pwshExe -NoProfile -File $tool -Browser chrome -Route vendor `
            > (Join-Path $scratch 'provisioned.txt') 2>&1
        $vendorRc = $LASTEXITCODE
    } catch {
        $vendorRc = -1
        $problems += ('  provisioning: the vendor route could not be started: ' + $_.Exception.Message)
    }
    if ($vendorRc -eq 0) {
        $provisioned = 'ok'
    } else {
        $provisioned = 'failed'
        $problems += ('  provisioning: the vendor route exited ' + $vendorRc + '. Its output is in .tmp/provisioned.txt')
    }
}

# 8. ⛔ THE EXACT-BUILD ROUTE, which is the one that can be asked for a build
# and answer with a different one. The confirm step inside the tool is what
# refuses that, and this is where it is exercised.
$acquired = 'skipped'
if ($Build -and $env:B_IDS_DISPOSABLE -eq '1' -and -not [string]::IsNullOrEmpty($env:CI)) {
    $checked = $checked + 1
    try {
        & $pwshExe -NoProfile -File $tool -Browser chrome -Route for-testing -Version $Build `
            > (Join-Path $scratch 'acquired.txt') 2>&1
        $ftRc = $LASTEXITCODE
    } catch {
        $ftRc = -1
        $problems += ('  for-testing: ' + $Build + ' could not be started: ' + $_.Exception.Message)
    }
    if ($ftRc -eq 0) {
        $acquired = 'ok'
    } else {
        $acquired = 'failed'
        $problems += ('  for-testing: ' + $Build + ' exited ' + $ftRc + '. Its output is in .tmp/acquired.txt')
    }
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-provisioning/2","checks":' + $checked + ',"problems":' + $count + ',"provisioned":"' + $provisioned + '","acquired":"' + $acquired + '"}')
} elseif ($count -eq 0) {
    Write-Output ('provisioning ok: ' + $checked + ' check(s), every refusal held, provisioning ' + $provisioned)
    if ($provisioned -eq 'skipped') {
        Write-Output '  SKIP the provisioning itself: this machine is not disposable, so nothing'
        Write-Output '  was purged. A workflow on a disposable runner is where that leg runs,'
        Write-Output '  and .github/workflows/provision.yml is what runs it.'
    }
    if ($acquired -eq 'skipped') {
        Write-Output '  SKIP the exact-build route: it runs with -Build on a disposable'
        Write-Output '  machine, and the build is named by the matrix cell rather than here.'
    }
} else {
    [Console]::Error.WriteLine('provisioning check failed, ' + $count + ' problem(s):')
    [Console]::Error.WriteLine('')
    foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('Every refusal here stands between a machine and losing its browser.')
    [Console]::Error.WriteLine('docs/history/todo/driver.md, DRIVER-08.')
}

if ($count -ne 0) { exit 1 }
exit 0
