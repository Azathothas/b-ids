# check-gate.ps1 - run every local gate this host can run, in one command.
#
# ⭐ THE TWIN OF check-gate.sh, and the one to prefer on Windows. It earns a
# twin by the rule in check-twins.sh: a native PowerShell session may have no
# POSIX shell at all, and "run the whole gate" is exactly the command somebody
# reaches for on a machine where that is true. Everything else under common/
# runs after the probe has reported and can assume sh; this cannot, because it
# is what a session runs first.
#
# The defect this exists to catch is a gate that was skipped because it was the
# ninth thing to remember. Part (a) of docs/methodology/gate.md is a LIST, and a
# list run by hand is run in the order somebody recalls it, missing whichever
# entry was added last.
#
# ⛔ IT IS NOT A SECOND SET OF RULES. Every line below shells out to a check
# that already exists and reads that check's own exit code. When this file and
# .github/workflows/ci.yml disagree about what runs, CI is the one that gates a
# push and this one is the defect.
#
# ⛔ IT RUNS EACH CHECK'S POWERSHELL TWIN, NOT ITS sh HALF. It used to run the
# sh half of all six twinned checks and skip every one of them when no POSIX
# shell was found, which is the host this file exists for. What still needs sh
# is what has no twin: `sh -n`, `shellcheck`, and check-twins itself.
#
# -- ⚠ A SKIPPED CHECK IS NOT A PASSED CHECK ---------------------------------
#
# Some of these need a tool that is not everywhere: sh, jq, shellcheck,
# PSScriptAnalyzer. A gate that silently dropped one and still printed green
# would be the "step that exits 0 having done nothing it was asked to do" row in
# docs/conventions/forbidden-patterns.md. So a missing tool is SKIP, counted
# separately, named in the summary and carried in -Json as `skipped`.
#
# -- ⚠ -Fast, AND WHY IT IS A FLAG RATHER THAN THE DEFAULT -------------------
#
# check-twins runs BOTH halves of every pair, so it costs roughly as much as the
# rest of the gate put together. That is the right price before a push and the
# wrong one before each of a dozen commits, and a gate too slow to run is a gate
# that gets run once at the end.
#
# ⛔ THE FIGURES BELOW REPLACE A SET THAT WAS NOT MEASURED. This comment used to
# carry a full run of 88 seconds, timed on a 4-CPU Linux container, beside a
# claim that the gate passed. On the tree it named, check-docs reported eleven
# problems and check-twins reported twelve drifts, so it did not.
# TODO/tooling.md T-007 carries the correction.
#
# Measured 2026-08-31, Windows 11 (10.0.26200) on a 20-thread i7-12700H, Git
# Bash 5.3 and PowerShell 7.6.5, over 13 twin pairs, on a tree of 4,476 tracked
# files of which 4,389 are the reference corpus:
#
#   full sh run            403s
#   sh --fast              106s
#   this half, -Fast        31s
#   check-twins alone      294s
#
# ⚠ RE-MEASURED THE SAME DAY, AFTER THE GATE GREW. The workspace landed and this
# runner gained four checks: check-msrv and the three suite entries. check-twins
# gained two pairs, so it compares 15. Same machine, same shells:
#
#   sh --fast              171s
#   this half, -Fast        65s
#   full sh run            ⛔ NOT RE-TAKEN. The run went green, all 19 checks,
#                          and the timing line was lost when the shell holding
#                          it was killed. A figure nobody measured does not go
#                          here.
#
# ⚠ This half went from 31s to 65s and the sh half from 106s to 171s, and the
# difference in both is four checks of which three compile.
#
# ⚠ Each figure is one run on a machine doing other things, and they do not add
# up because they are separate runs.
#
# ⚠ THIS IS NOT THE HOST THIS FILE EXISTS FOR. It earns a twin because a native
# Windows PowerShell session may have no POSIX shell, and no figure has been
# taken there. A Windows number is still wanted and would be a different one.
#
# ⛔ -Fast SKIPS check-twins. It does not weaken anything else, it is reported
# as a SKIP like every other, and the summary says so. The full run is what a
# push is gated on.
#
# -- ⛔ -Strict, WHICH IS THE CI MODE ---------------------------------------
#
# ⭐ It turns a SKIP into a failure. On a developer's machine a missing tool is a
# fact about the machine; on a runner the tools are installed on purpose, so a
# skip there means the install broke and the tree went unchecked.
#
# ⚠ IT WAS DOCUMENTED BEFORE IT EXISTED, in docs/methodology/gate.md, and
# neither half of this runner had it. ⛔ Keep this identical to the sh twin.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-gate.ps1
#   pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
#   pwsh -NoProfile -File scripts/common/check-gate.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-gate.ps1 -Strict
#
#   pwsh -NoProfile -File scripts/common/git-sync.ps1 -Message "..." -BodyFile msg.txt `
#        -Gate "pwsh -NoProfile -File scripts/common/check-gate.ps1"
#
# Exit codes: 0 everything that ran passed, 1 something failed, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

# -- PSScriptAnalyzer, suppressed per rule with the reason --------------------
[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingWriteHost', '',
    Justification = 'Not used here. Declared so a future edit that reaches for Write-Host has to delete this line and think about it; every line of output below goes through Write-Output so -Json stays parseable.')]
[CmdletBinding()]
param(
    [switch]$Json,
    [switch]$Fast,
    [switch]$Strict,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. `pwsh -File` reports a parameter-binding failure as 1, which is
    # this project's code for "it ran and the thing failed"; the POSIX twin
    # exits 2 for the same input. Measured across every pair 2026-09-02:
    # 22 of 22 disagreed. TODO/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-gate: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Exit-With {
    param([Parameter(Mandatory = $true)][int]$Code, [Parameter(Mandatory = $true)][string]$Text)
    [Console]::Error.WriteLine("check-gate: $Text")
    exit $Code
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) { Exit-With 2 'git not found' }
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) { Exit-With 2 'not a git repository' }
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root

$script:Passed = 0
$script:Failed = 0
$script:Skipped = 0
$script:FailedNames = @()
$script:SkippedNames = @()

function Write-Line { param([string]$Text) if (-not $Json) { Write-Output $Text } }

function Add-Pass { param([string]$Name) $script:Passed++; Write-Line "  ok    $Name" }
function Add-Fail {
    param([string]$Name, [int]$Code)
    $script:Failed++
    $script:FailedNames += $Name
    Write-Line "  FAIL  $Name (exit $Code)"
}
function Add-Skip {
    param([string]$Name, [string]$Reason)
    $script:Skipped++
    $script:SkippedNames += $Name
    Write-Line "  SKIP  $Name -- $Reason"
}

function Invoke-Check {
    <#
      Run one check, read its exit code from the process that produced it, and
      show its output only when it failed.

      -PassCodes exists for check-changelog, whose 2 means "could not run" and
      is the honest answer in a project with no CHANGELOG.md. Collapsing that
      into 0 with a blanket ignore would hide a genuine 1 as well.

      ⛔ -SkipCodes IS THE OTHER HALF OF THAT, AND IT IS NOT THE SAME THING.
      check-changelog's 2 is a PASS because a project with no changelog has
      satisfied the rule vacuously. check-msrv's 2 is a SKIP because a host
      with no cargo has verified NOTHING about the manifest. Collapsing the
      second into the first is how a check quietly stops applying.
    #>
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$FilePath,
        [string[]]$Arguments = @(),
        [int[]]$PassCodes = @(0),
        [int[]]$SkipCodes = @(),
        [string]$SkipReason = 'the check reported it could not run'
    )
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $out = & $FilePath @Arguments 2>&1
        $code = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $prev }

    if ($null -eq $code) { $code = 1 }
    if ($PassCodes -contains $code) { Add-Pass $Name; return }
    if ($SkipCodes -contains $code) { Add-Skip $Name $SkipReason; return }

    Add-Fail $Name $code
    if (-not $Json) {
        foreach ($l in ($out | Out-String) -split "`r?`n") {
            if ($l.Trim()) { Write-Output "  | $l" }
        }
    }
}

# -- ⭐ WHAT THIS GATE SKIPS WHEN check-twins IS RUNNING IT -------------------
#
# ⛔ MEASURED, NOT GUESSED. `check-twins --timings`, 2026-09-01, on one Windows
# 11 host: 970 seconds across twenty pairs, of which the `check-gate` row alone
# is 431. That row runs BOTH gates in full, and each gate re-runs the fourteen
# checks that ALREADY HAVE A ROW OF THEIR OWN. TODO/tooling.md, TOOL-15.
#
# ⭐ So a gate running inside check-twins skips them. What that pair uniquely
# proves is untouched: the LIST each half runs, and the checks with no row of
# their own.
#
# ⛔ THE LIST GOING STALE IS COVERED, and for free: the pair compares `skipped`
# as well as `passed`. ⚠ Keep it identical to the sh twin's.
$ComparedDirectly = @(
    'check-docs',
    'check-markers',
    'check-one-home',
    'check-placeholders',
    'check-control-bytes',
    'check-record',
    'check-no-secrets',
    'check-vendor',
    'check-msrv',
    'check-corpus',
    'check-validate',
    'check-line-endings',
    'check-routes',
    'check-exit-codes',
    'check-manual-path',
    'check-changelog',
    'check-workflows',
    'check-coverage'
)

function Test-ComparedDirectly {
    param([string]$Name)
    if ($env:CHECK_GATE_INNER -ne '1') { return $false }
    if ($ComparedDirectly -notcontains $Name) { return $false }
    Add-Skip $Name 'compared directly by check-twins; running it here compares one answer a third time'
    return $true
}

function Invoke-PsCheck {
    <#
      Run a check's POWERSHELL TWIN, through this same host.

      ⛔ THE TWIN IS PREFERRED HERE, NOT THE sh HALF, and getting that wrong is
      what this function exists to stop. This file used to shell out to the .sh
      half of every check and SKIP six of them outright when no POSIX shell was
      found, on precisely the machine the twins were written for. Its own header
      says it earns a twin because a native PowerShell session may have no sh;
      scripts/README.md says to run the .ps1 half on Windows. The runner was the
      one place not doing it.

      ⚠ THE HOST IS RE-ENTERED BY PATH, never by the name `pwsh`. A Windows
      PowerShell 5.1 session may have no `pwsh` on PATH at all, and a 5.1 caller
      has to get 5.1 back rather than whichever host happens to answer.
    #>
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Script,
        [string[]]$Arguments = @(),
        [int[]]$PassCodes = @(0),
        [int[]]$SkipCodes = @(),
        [string]$SkipReason = 'the check reported it could not run'
    )
    # ⭐ THE ONE PLACE THE SKIP LIVES. Every check compared directly by
    # check-twins goes through this function, so the guard is here rather than
    # repeated at fourteen call sites.
    if (Test-ComparedDirectly $Name) { return }
    $full = Join-Path $root $Script
    if (-not (Test-Path -LiteralPath $full)) {
        # ⛔ Named, not dropped. A check whose file is gone is a finding.
        Add-Skip $Name "$Script is missing"
        return
    }
    Invoke-Check -Name $Name -FilePath (Get-Process -Id $PID).Path `
        -Arguments (@('-NoProfile', '-File', $full) + $Arguments) -PassCodes $PassCodes `
        -SkipCodes $SkipCodes -SkipReason $SkipReason
}

function Get-PosixShell {
    # ⚠ Get-Command finds cmdlets, functions and aliases too, so it is filtered
    # to a real executable. docs/conventions/shell.md section 8.
    foreach ($n in @('sh', 'sh.exe', 'bash', 'bash.exe')) {
        $c = Get-Command $n -CommandType Application -ErrorAction SilentlyContinue |
             Select-Object -First 1
        if ($c) { return $c.Source }
    }
    # Git for Windows ships one and does not always put it on PATH.
    foreach ($p in @("$env:ProgramFiles\Git\bin\sh.exe", "$env:ProgramFiles\Git\usr\bin\sh.exe")) {
        if ($p -and (Test-Path -LiteralPath $p)) { return $p }
    }
    return $null
}

Write-Line "check-gate: $root"
Write-Line ''

$sh = Get-PosixShell
$common = 'scripts/common'

# -- the PowerShell half runs first, because it needs no sh -----------------
# ⛔ SCORED AS TWO ENTRIES, because they can have different answers. The parse
# either ran or it did not; the analyzer is a module that may be absent, and
# check-powershell exits 0 either way. One verdict for both is how a skipped
# analyzer reads as a passed check, which it did once before the fixed status
# line existed.
$psCheck = Join-Path $root 'scripts/common/check-powershell.ps1'
if (-not (Test-Path -LiteralPath $psCheck)) {
    Add-Skip 'powershell parse' 'scripts/common/check-powershell.ps1 is missing'
    Add-Skip 'PSScriptAnalyzer'  'scripts/common/check-powershell.ps1 is missing'
}
else {
    $self = (Get-Process -Id $PID).Path
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $psOut = & $self -NoProfile -File $psCheck 2>&1
        $psRc = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $prev }
    if ($null -eq $psRc) { $psRc = 1 }

    $psText = ($psOut | Out-String)
    if ($psRc -eq 0) { Add-Pass 'powershell parse' }
    elseif ($psRc -eq 2) { Add-Skip 'powershell parse' 'the host reported it could not run' }
    else {
        Add-Fail 'powershell parse' $psRc
        if (-not $Json) {
            foreach ($l in $psText -split "`r?`n") { if ($l.Trim()) { Write-Output "  | $l" } }
        }
    }

    # The fixed last line, not the prose. check-powershell.ps1 documents it.
    if ($psText -match 'analyzer=skipped') { Add-Skip 'PSScriptAnalyzer' 'not installed on this host' }
    elseif ($psText -match 'analyzer=clean') { Add-Pass 'PSScriptAnalyzer' }
    elseif ($psText -match 'analyzer=issues') { Add-Fail 'PSScriptAnalyzer' 1 }
    else { Add-Skip 'PSScriptAnalyzer' 'check-powershell printed no analyzer status line' }
}

# -- every check that has a twin, run through the twin -----------------------
# ⛔ BOTH check-docs AND check-markers. The first reads markdown; the second
# owns the character rule over every tracked text file. In the trees these were
# written in, check-docs reported clean while check-markers had findings in the
# hundreds, every one of them in a script rather than a document.
Invoke-PsCheck -Name 'check-docs'          -Script 'scripts/common/check-docs.ps1'
Invoke-PsCheck -Name 'check-markers'       -Script 'scripts/common/check-markers.ps1'
Invoke-PsCheck -Name 'check-one-home'      -Script 'scripts/common/check-one-home.ps1'
Invoke-PsCheck -Name 'check-placeholders'  -Script 'scripts/common/check-placeholders.ps1'
Invoke-PsCheck -Name 'check-control-bytes' -Script 'scripts/common/check-control-bytes.ps1'
Invoke-PsCheck -Name 'check-record'        -Script 'scripts/common/check-record.ps1'
Invoke-PsCheck -Name 'check-no-secrets'    -Script 'scripts/common/check-no-secrets.ps1' -Arguments @('-Public')
Invoke-PsCheck -Name 'check-changelog'     -Script 'scripts/common/check-changelog.ps1' -PassCodes @(0, 2)

# -- the vendored trees, and the record that has to describe them ------------
# ONLY THE OFFLINE LEG. -Upstream fetches the recorded ref from the remote and
# a gate that needs the network fails on a machine that has none. 2 is "could
# not run", which here means the tree vendors nothing, and that is a SKIP.
Invoke-PsCheck -Name 'check-vendor' -Script 'scripts/common/check-vendor.ps1' `
    -SkipCodes @(2) -SkipReason 'this tree vendors nothing'

# -- the workspace, and the version floor it declares ------------------------
Invoke-PsCheck -Name 'check-msrv' -Script 'scripts/common/check-msrv.ps1' `
    -SkipCodes @(2) -SkipReason 'cargo is not on this host'

# -- the published corpus, and whether it was ever edited in place -----------
# 2 is "could not run" twice over: there is no corpus at all, or the
# per-profile leg needed cargo and did not get it. Neither has verified
# anything about a profile, so both are a SKIP rather than a pass. The git leg
# still decides a FAILURE: a published file edited after its first commit is
# exit 1 whether or not cargo was there.
Invoke-PsCheck -Name 'check-corpus' -Script 'scripts/common/check-corpus.ps1' `
    -SkipCodes @(2) -SkipReason 'the corpus is empty, or cargo could not verify a profile'

# -- every published profile, coherent, and the derived files reproducible ---
# ⚠ 2 is "could not run" three ways over: there is no corpus, it holds no
# profile, or cargo is absent. None has validated anything, so all three are a
# SKIP rather than a pass. ⛔ A finding or a non-deterministic generator is exit
# 1 and fails the gate.
Invoke-PsCheck -Name 'check-validate' -Script 'scripts/common/check-validate.ps1' `
    -SkipCodes @(2) -SkipReason 'the corpus is empty, or cargo could not validate a profile'

# -- every workflow declares the four things that decide a run's output ------
# ⚠ 2 is "could not run": there is no .github/workflows directory, or it holds
# no .yml file.
Invoke-PsCheck -Name 'check-workflows' -Script 'scripts/common/check-workflows.ps1' `
    -Arguments @('-AssertFailFastFalse') -SkipCodes @(2) `
    -SkipReason 'there is no workflow directory, or it holds no workflow'

# -- which cells of the planned capture matrix have a profile ----------------
# ⚠ 2 is "could not run": there is no plan. ⛔ No row is REQUIRED here; the
# capture workflow is where --require-rows is passed.
Invoke-PsCheck -Name 'check-coverage' -Script 'scripts/common/check-coverage.ps1' `
    -SkipCodes @(2) -SkipReason 'there is no capture matrix'

# -- every script answers 2 for a state it cannot act on ----------------------
#
# ⛔ 1 is "it ran and the thing failed" and 2 is "it could not run", and a
# script that returned 1 for the second is one somebody disables the day a
# runner has no browser. TODO/ci.md, CI-07.
Invoke-PsCheck -Name 'check-exit-codes' -Script 'scripts/common/check-exit-codes.ps1'

# ⛔ An automated step nobody can do by hand is a step that stops existing
# when the platform does. TODO/ci.md, CI-08.
Invoke-PsCheck -Name 'check-manual-path' -Script 'scripts/common/check-manual-path.ps1'

# -- the published route files, and the one byte a consumer should not have to
# strip. 2 is "there is no route tree yet, or it holds no single-value file",
# which has verified nothing and is a SKIP rather than a pass.
Invoke-PsCheck -Name 'check-routes' -Script 'scripts/common/check-routes.ps1' `
    -SkipCodes @(2) -SkipReason 'no published route tree, or it holds no single-value file'

# -- line endings, in the index AND in the working tree ----------------------
#
# ⛔ IT USED TO BE INLINE HERE, IN BOTH HALVES, AND THAT WAS THE DEFECT. Two
# copies of one rule computed in two languages, compared by nothing: the twin
# comparison covers a PAIR OF SCRIPTS, and a rule with no script of its own had
# no row. ⭐ It is a check now, with both halves and a row. TODO/tooling.md,
# TOOL-17.
Invoke-PsCheck -Name 'check-line-endings' -Script 'scripts/common/check-line-endings.ps1' `
    -SkipCodes @(2) -SkipReason 'git tracks no file in this repository'

# -- the probe, through its own twin -----------------------------------------
Invoke-PsCheck -Name 'doctor probe' -Script 'scripts/doctor/doctor.ps1' -Arguments @('-Fast')

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
$cargo = Get-Command 'cargo' -CommandType Application -ErrorAction SilentlyContinue |
         Select-Object -First 1
if ($cargo) {
    Invoke-Check -Name 'cargo fmt'    -FilePath $cargo.Source -Arguments @('fmt', '--all', '--check')
    Invoke-Check -Name 'cargo clippy' -FilePath $cargo.Source `
        -Arguments @('clippy', '--workspace', '--all-targets', '--all-features', '--', '-D', 'warnings')
    # ⚠ No --all-targets here on purpose: it would drop the doc-tests, and a
    # doc-test is the one test that proves the documentation compiles.
    Invoke-Check -Name 'cargo test'   -FilePath $cargo.Source -Arguments @('test', '--workspace', '--all-features')
}
else {
    Add-Skip 'cargo fmt'    'cargo is not on this host'
    Add-Skip 'cargo clippy' 'cargo is not on this host'
    Add-Skip 'cargo test'   'cargo is not on this host'
}

if (-not $sh) {
    # ⛔ Not a silent degrade. What is left below genuinely needs a POSIX shell,
    # and saying which ones did not run is the difference between a gate and a
    # green badge.
    Add-Skip 'sh -n'               'no POSIX shell on this host'
    Add-Skip 'shellcheck'          'no POSIX shell on this host'
    Add-Skip 'check-twins'         'no POSIX shell on this host; it runs both halves of every pair'
}
else {
    # Every tracked .sh parses.
    # ⛔ THE REFERENCE CORPUS IS OUT OF SCOPE FOR THE LINTERS. `references/` is
    # other projects' source, kept as evidence; their style is not this
    # project's defect. ⛔ Keep this identical to the sh twin.
    # ⚠ Tracked plus untracked-not-ignored, matching the sh twin.
    $shFiles = @(@(& git ls-files '*.sh') + @(& git ls-files --others --exclude-standard '*.sh') |
        ForEach-Object { $_.Trim() } |
        Where-Object { $_ -and $_ -cnotmatch '^references/' } | Sort-Object -Unique)
    $bad = @()
    foreach ($f in $shFiles) {
        $prev = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        try { & $sh -n $f 2>&1 | Out-Null; $code = $LASTEXITCODE } finally { $ErrorActionPreference = $prev }
        if ($code -ne 0) { $bad += $f }
    }
    if ($bad.Count -eq 0) { Add-Pass 'sh -n' }
    else {
        Add-Fail 'sh -n' 1
        if (-not $Json) { foreach ($f in $bad) { Write-Output "  | parse FAIL $f" } }
    }

    $sc = Get-Command shellcheck -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $sc) { Add-Skip 'shellcheck' 'shellcheck is not on PATH' }
    else {
        $bad = @()
        foreach ($f in $shFiles) {
            $prev = $ErrorActionPreference
            $ErrorActionPreference = 'Continue'
            try { & $sc.Source -s sh $f 2>&1 | Out-Null; $code = $LASTEXITCODE } finally { $ErrorActionPreference = $prev }
            if ($code -ne 0) { $bad += $f }
        }
        if ($bad.Count -eq 0) { Add-Pass 'shellcheck' }
        else {
            Add-Fail 'shellcheck' 1
            if (-not $Json) { foreach ($f in $bad) { Write-Output "  | shellcheck $f" } }
        }
    }

    # ⛔ THIS PAIR RUNS THIS SCRIPT. check-twins.sh compares both halves of
    # every twin and check-gate is one of them, so an unguarded call here is an
    # infinite recursion: gate runs twins runs gate runs twins. It hung for ten
    # minutes before this guard existed, which is how the guard came to exist.
    # check-twins.sh exports the same variable, so a session that starts from
    # there gets a gate one level deep rather than three.
    $jq = Get-Command jq -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($Fast) {
        Add-Skip 'check-twins' '-Fast was passed; it runs both halves of every pair'
    }
    elseif ($env:CHECK_GATE_INNER -eq '1') {
        Add-Skip 'check-twins' 'already running inside check-twins; calling it here would recurse'
    }
    elseif (-not $jq) { Add-Skip 'check-twins' 'jq is not on PATH; it compares json' }
    else {
        $env:CHECK_GATE_INNER = '1'
        try { Invoke-Check -Name 'check-twins' -FilePath $sh -Arguments @("$common/check-twins.sh") }
        finally { $env:CHECK_GATE_INNER = $null }
    }
}

# -- report ----------------------------------------------------------------
$total = $script:Passed + $script:Failed + $script:Skipped

if ($Json) {
    $payload = [ordered]@{
        schema  = 'check-gate/1'
        total   = $total
        passed  = $script:Passed
        failed  = $script:Failed
        skipped = $script:Skipped
        strict  = [int][bool]$Strict
    }
    Write-Output ($payload | ConvertTo-Json -Compress -Depth 4)
    if ($script:Failed -gt 0) { exit 1 }
    if ($Strict -and $script:Skipped -gt 0) { exit 1 }
    exit 0
}

Write-Output ''
if ($script:Failed -gt 0) {
    Write-Output "GATE FAILED: $($script:Failed) of $total. Failed: $($script:FailedNames -join ' ')"
    if ($script:Skipped -gt 0) { Write-Output "Also skipped $($script:Skipped): $($script:SkippedNames -join ' ')" }
    exit 1
}

if ($script:Skipped -gt 0) {
    if ($Strict) {
        Write-Output "GATE FAILED under -Strict: $($script:Passed) passed, $($script:Skipped) SKIPPED: $($script:SkippedNames -join ' ')"
        Write-Output 'A skipped check is not a passed check. On a runner the tools are'
        Write-Output 'installed on purpose, so a skip means the install broke.'
        exit 1
    }
    Write-Output "gate ok: $($script:Passed) passed, but $($script:Skipped) SKIPPED on this host: $($script:SkippedNames -join ' ')"
    Write-Output 'A skipped check is not a passed check. CI runs on two hosts that between'
    Write-Output 'them have every tool; that is where the coverage for these comes from.'
    exit 0
}

Write-Output "gate ok: all $($script:Passed) checks passed"
exit 0
