# provision-browser.ps1 - purge every browser of one family from this machine,
# install the build that was asked for, and prove both.
#
# ⭐ THE TWIN OF provision-browser.sh. TODO/driver.md, DRIVER-08 and DRIVER-09.
#
# ⛔ ON A MACHINE THIS PROJECT CONTROLS COMPLETELY, IT MEASURED WHATEVER
# SOMEBODY ELSE'S IMAGE INSTALLED. A capture lane called `b-ids-driver resolve`,
# which by design finds what is already there, so the corpus recorded a build
# nobody chose from a source nobody named, and every profile carried
# `captured.acquisition: null`.
#
# ⛔ MEASURED, AND THE COST WAS ALREADY PAID: on 2026-09-02 ubuntu-latest served
# Chrome 151.0.7922.173 and windows-latest served 151.0.7922.174, so the single
# highest-value capture available, one build on two platforms, was unobtainable.
#
# -- ⛔ FOUR STEPS, AND EVERY ONE OF THEM IS CONFIRMED -----------------------
#
#   1. purge, by every route an image might have used;
#   2. CONFIRM the purge: `resolve` must exit 2, meaning it found nothing. A
#      purge that reported success while a browser remained is the "reporting a
#      result the code never read" row of
#      docs/conventions/forbidden-patterns.md;
#   3. install the build asked for, from the route asked for;
#   4. CONFIRM the install: `resolve` must report exactly the version asked for.
#      A lane that installed one build and captured another is the same defect
#      one step along.
#
# -- ⭐ TWO ROUTES, BECAUSE THEY ARE TWO PRODUCTS ----------------------------
#
#   vendor       branded Chrome from the vendor's own channel. CURRENT BUILD
#                ONLY: the channel serves what is current and nothing else.
#                ⭐ Both platforms provisioned on one day therefore get the SAME
#                build, which is the whole point.
#   for-testing  an exact build, any version, every platform, from the
#                automation-build index. ⛔ UNBRANDED: a different brand list
#                and a different sec-ch-ua, and a profile taken through it
#                records `branded: false`. DRIVER-06 measures the difference.
#
# ⛔ THIS NEVER REDISTRIBUTES A BROWSER. It prints the URL it fetched and the
# sha256 of what arrived; the artefact is the vendor's to serve.
#
# -- ⛔ IT REFUSES TO RUN ON A MACHINE THAT IS NOT DISPOSABLE ----------------
#
# Purging a browser is a change to somebody's machine. B_IDS_DISPOSABLE=1 says
# the machine is thrown away afterwards, and only a workflow sets it. A
# developer's laptop must not lose its browser to a capture.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/provision-browser.ps1 -Browser chrome -Route vendor
#   pwsh -NoProfile -File scripts/common/provision-browser.ps1 -Browser chrome -Route for-testing -Version 151.0.7922.173
#   pwsh -NoProfile -File scripts/common/provision-browser.ps1 -Plan -Browser chrome -Route vendor
#
# Exit codes: 0 provisioned and confirmed,
#             1 a step ran and failed, or the version is not the one asked for,
#             2 could not run, which includes a machine that is not disposable.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [string]$Browser = '',
    [string]$Route = 'vendor',
    [string]$Version = '',
    [switch]$Plan,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('provision-browser: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $Browser) {
    [Console]::Error.WriteLine('provision-browser: -Browser is required')
    exit 2
}
if ($Route -ne 'vendor' -and $Route -ne 'for-testing') {
    [Console]::Error.WriteLine('provision-browser: -Route is vendor or for-testing, not ' + $Route)
    exit 2
}

# ⛔ THE ROUTE DECIDES WHETHER A VERSION MEANS ANYTHING, and saying so is better
# than accepting one and ignoring it. The vendor channel serves what is current;
# asking it for a build is asking for something it cannot answer.
if ($Route -eq 'vendor' -and $Version) {
    [Console]::Error.WriteLine('provision-browser: -Route vendor serves the CURRENT build only, so -Version ' + $Version)
    [Console]::Error.WriteLine('  cannot be honoured. Use -Route for-testing for an exact build, and read that it')
    [Console]::Error.WriteLine('  is UNBRANDED. TODO/driver.md, DRIVER-08.')
    exit 2
}
if ($Route -eq 'for-testing' -and -not $Version) {
    [Console]::Error.WriteLine('provision-browser: -Route for-testing needs -Version')
    exit 2
}

# ⛔ Resolved from the script's own location, never from the working directory.
$here = Split-Path -Parent $PSCommandPath
$root = (Resolve-Path -LiteralPath (Join-Path $here '..' | Join-Path -ChildPath '..')).Path
Set-Location -LiteralPath $root

# ⚠ PowerShell 5.1 has no $IsWindows, so the absence of the variable is itself
# the answer: 5.1 runs on Windows and nowhere else.
if (Test-Path 'variable:IsWindows') {
    if ($IsWindows) { $os = 'windows' }
    elseif ($IsMacOS) { $os = 'mac' }
    elseif ($IsLinux) { $os = 'linux' }
    else { $os = 'unknown' }
} else {
    $os = 'windows'
}

# -- what each route and platform would do, printed rather than assumed -------
#
# ⭐ -Plan RUNS NOTHING. It is what a person reads before letting this near a
# machine, and it is what the acceptance check can assert on a host that is not
# disposable.
function Write-Plan {
    $key = $os + '/' + $Route
    switch ($key) {
        'linux/vendor' {
            Write-Output 'purge   apt-get remove --purge, then /opt/google/chrome and the /usr/bin links'
            Write-Output 'fetch   https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb'
            Write-Output 'install dpkg -i, then apt-get -f install for anything it needs'
        }
        'windows/vendor' {
            Write-Output 'purge   the vendor uninstaller for every install found, then the program directory'
            Write-Output 'fetch   https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi'
            Write-Output 'install msiexec /qn, which is the silent unattended mode'
        }
        default {
            if ($Route -eq 'for-testing') {
                Write-Output 'purge   as for the vendor route on this platform'
                Write-Output 'fetch   the zip the automation-build index names for this exact build'
                Write-Output 'install unzip into a directory this project owns, and link it where resolve looks'
            } else {
                Write-Output ('no plan is recorded for ' + $Route + ' on this platform')
            }
        }
    }
    Write-Output 'confirm resolve exits 2 after the purge, and reports the version after the install'
}

if ($Plan) {
    Write-Output ('provision-browser plan: ' + $Browser + ' via ' + $Route + ' on ' + $os)
    Write-Output ''
    Write-Plan
    exit 0
}

# ⛔ TWO INDEPENDENT REFUSALS, AND THEY ARE THE FIRST THING AFTER PARSING.
#
# ⚠ MEASURED, ON THIS PROJECT'S OWN OPERATOR MACHINE, 2026-09-02. A session
# testing that the guard could fail mutated the single condition and ran the
# tool on a developer laptop. The purge path executed. Nothing was removed,
# because the Windows uninstaller match did not fire, and the confirm step then
# refused correctly. ⛔ It should not have been reachable at all, and "it
# happened not to match" is not a safety margin.
#
# ⭐ SO THERE ARE TWO CONDITIONS FROM TWO SOURCES, and one edit cannot lift
# both: a variable this project sets only inside a workflow, and the marker the
# platform sets on every hosted runner. A person who genuinely wants this on a
# disposable machine of their own sets both, deliberately.
#
# ⛔ AND A TEST THAT HAS TO BYPASS A GUARD RUNS AGAINST A COPY, never against
# this file on a machine the guard protects.
$disposable = $env:B_IDS_DISPOSABLE
$onARunner = $env:CI
if ($disposable -ne '1' -or [string]::IsNullOrEmpty($onARunner)) {
    $shownDisposable = if ([string]::IsNullOrEmpty($disposable)) { 'unset' } else { $disposable }
    $shownRunner = if ([string]::IsNullOrEmpty($onARunner)) { 'unset' } else { $onARunner }
    [Console]::Error.WriteLine('provision-browser: this machine is not marked disposable, so nothing was purged.')
    [Console]::Error.WriteLine('  B_IDS_DISPOSABLE=' + $shownDisposable + ' and CI=' + $shownRunner + ', and BOTH are required.')
    [Console]::Error.WriteLine('  Set them only on a machine that is thrown away afterwards.')
    [Console]::Error.WriteLine('  Run with -Plan to read what it would do. TODO/driver.md, DRIVER-08.')
    exit 2
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('provision-browser: cargo not found')
    exit 2
}
if ($os -eq 'unknown') {
    [Console]::Error.WriteLine('provision-browser: no plan for this platform')
    exit 2
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'provision-browser'
New-Item -ItemType Directory -Force -Path $out | Out-Null

Write-Output 'building the driver'
& cargo build -q -p b-ids-driver
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('provision-browser: the driver did not build')
    exit 2
}
$driver = Join-Path $root 'target' | Join-Path -ChildPath 'debug' | Join-Path -ChildPath 'b-ids-driver'
if (-not (Test-Path -LiteralPath $driver)) { $driver = $driver + '.exe' }
if (-not (Test-Path -LiteralPath $driver)) {
    [Console]::Error.WriteLine('provision-browser: ' + $driver + ' is not there')
    exit 2
}

# ⛔ ONE READER OF "WHAT IS INSTALLED", and it is the driver rather than a second
# search written here. A script that looked for a browser its own way would be a
# second answer to the question `resolve` exists to answer, and the two would
# disagree the first time a path moved.
function Get-ResolvedVersion {
    $text = (& $driver resolve --browser $Browser --json 2>$null | Select-Object -First 1)
    if (-not $text) { return '' }
    $m = [regex]::Match([string]$text, '"version":"([^"]*)"')
    if ($m.Success) { return $m.Groups[1].Value }
    return ''
}
function Get-ResolveExitCode {
    & $driver resolve --browser $Browser --json > $null 2>&1
    return $LASTEXITCODE
}

# -- 1. purge -----------------------------------------------------------------
Write-Output ''
Write-Output ('-- purging every ' + $Browser + ' on this machine --')
$before = Get-ResolvedVersion
if ($before) { Write-Output ('before  ' + $before) } else { Write-Output 'before  nothing resolved' }

function Invoke-PurgeWindows {
    # ⚠ EVERY INSTALL, not the first one found: an image may carry a machine-wide
    # install and a per-user one, and removing one leaves the other for `resolve`.
    $keep = $ErrorActionPreference
    $ErrorActionPreference = 'SilentlyContinue'
    $roots = @(
        'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*'
    )
    foreach ($regRoot in $roots) {
        foreach ($key in (Get-ItemProperty $regRoot)) {
            $name = $key.DisplayName
            if ($name -match '^Google Chrome' -or $name -match '^Microsoft Edge$') {
                $uninstall = $key.UninstallString
                if ($uninstall -match 'setup\.exe') {
                    $exe = ($uninstall -split '" ')[0].Trim('"')
                    & $exe --uninstall --system-level --force-uninstall 2>$null
                    & $exe --uninstall --force-uninstall 2>$null
                }
            }
        }
    }
    Remove-Item -Recurse -Force (Join-Path $env:ProgramFiles 'Google\Chrome') 2>$null
    Remove-Item -Recurse -Force (Join-Path ${env:ProgramFiles(x86)} 'Google\Chrome') 2>$null
    Remove-Item -Recurse -Force (Join-Path $env:LOCALAPPDATA 'Google\Chrome') 2>$null
    $ErrorActionPreference = $keep
}

function Invoke-PurgeLinux {
    foreach ($pkg in @('google-chrome-stable', 'google-chrome-beta', 'google-chrome-unstable', 'microsoft-edge-stable')) {
        & sudo apt-get remove --purge -y $pkg > $null 2>&1
    }
    & sudo rm -rf /opt/google/chrome /opt/google/chrome-beta /opt/google/chrome-unstable /opt/microsoft/msedge > $null 2>&1
    & sudo rm -f /usr/bin/google-chrome /usr/bin/google-chrome-stable /usr/bin/microsoft-edge /usr/bin/microsoft-edge-stable > $null 2>&1
}

switch ($os) {
    'windows' { Invoke-PurgeWindows }
    'linux' { Invoke-PurgeLinux }
    'mac' { & sudo rm -rf '/Applications/Google Chrome.app' > $null 2>&1 }
}

# -- 2. confirm the purge -----------------------------------------------------
#
# ⛔ READ FROM THE RESOLVER, not from the exit code of a package manager. A
# remove that reported success over a browser somewhere else on the machine is
# exactly what this step is for. 2 is "resolve found nothing", which is CI-07's
# meaning of could-not-run and here is the success condition.
$rc = Get-ResolveExitCode
if ($rc -ne 2) {
    [Console]::Error.WriteLine('provision-browser: the purge left a browser behind. resolve exited ' + $rc + ' and reports ' + (Get-ResolvedVersion))
    exit 1
}
Write-Output 'after   nothing resolves, resolve exits 2'

# -- 3. install ---------------------------------------------------------------
Write-Output ''
Write-Output ('-- installing ' + $Browser + ' via ' + $Route + ' --')
$url = ''
$archive = ''

$key = $os + '/' + $Route
if ($key -eq 'linux/vendor') {
    $url = 'https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb'
    $archive = Join-Path $out 'google-chrome-stable_current_amd64.deb'
    try { Invoke-WebRequest -Uri $url -OutFile $archive -UseBasicParsing } catch {
        [Console]::Error.WriteLine('provision-browser: fetch failed')
        exit 1
    }
    & sudo apt-get install -y $archive > $null 2>&1
    if ($LASTEXITCODE -ne 0) {
        & sudo dpkg -i $archive > $null 2>&1
        & sudo apt-get -f install -y > $null 2>&1
    }
} elseif ($key -eq 'windows/vendor') {
    $url = 'https://dl.google.com/dl/chrome/install/googlechromestandaloneenterprise64.msi'
    $archive = Join-Path $out 'googlechromestandaloneenterprise64.msi'
    try { Invoke-WebRequest -Uri $url -OutFile $archive -UseBasicParsing } catch {
        [Console]::Error.WriteLine('provision-browser: fetch failed')
        exit 1
    }
    & msiexec.exe /i $archive /qn /norestart | Out-Null
} else {
    [Console]::Error.WriteLine('provision-browser: the ' + $Route + ' route on ' + $os + ' is not implemented yet')
    [Console]::Error.WriteLine('  Run with -Plan to read what it would do. TODO/driver.md, DRIVER-08.')
    exit 2
}

# ⛔ THE DIGEST OF WHAT ARRIVED, printed rather than inferred. Every download URL
# will one day 404, and a later reader still has to be able to say whether two
# captures used the same bytes. DRIVER-05 is where that rule comes from.
if (Test-Path -LiteralPath $archive) {
    $sha = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    $bytes = (Get-Item -LiteralPath $archive).Length
    Write-Output ('url     ' + $url)
    Write-Output ('sha256  ' + $sha)
    Write-Output ('bytes   ' + $bytes)
}

# -- 4. confirm the install ---------------------------------------------------
$got = Get-ResolvedVersion
if (-not $got) {
    [Console]::Error.WriteLine('provision-browser: nothing resolves after the install')
    exit 1
}
Write-Output ('version ' + $got)

if ($Version -and $got -ne $Version) {
    [Console]::Error.WriteLine('provision-browser: asked for ' + $Version + ' and got ' + $got)
    exit 1
}

Write-Output ''
Write-Output ('provisioned  ' + $Browser + ' ' + $got + ' via ' + $Route + ' on ' + $os)
exit 0
