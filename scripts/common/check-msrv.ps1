#Requires -Version 5.1
<#
.SYNOPSIS
  check-msrv.ps1 - is the declared minimum supported Rust version derived from
  the dependency graph, or is it a number somebody typed?

.DESCRIPTION
  ⛔ THE POWERSHELL TWIN OF scripts/common/check-msrv.sh. One rule, two
  implementations, and check-twins.sh compares their --json answers on one
  tree. TODO/RULES.md section 4 is why a twin exists at all: a POSIX shell
  check cannot be assumed to run on Windows, and the reverse is equally true.

  The defect this exists to catch is a `rust-version` field that nobody
  measured. TODO/tooling.md TOOL-01 states the rule: the dependency graph says
  what the workspace actually requires, and a number chosen by hand goes stale
  the first time a dependency raises its own floor. What is left behind then is
  a CLAIM that reads like a CONSTRAINT: consumers on the declared version get a
  compile error the manifest promised they would not.

  WHAT IT CHECKS
    1. ⛔ The workspace declares a `rust-version` at all. An absent one is not
       "any version": it is a promise nobody made.
    2. ⛔ The declared value is not BELOW the floor the resolved dependency
       graph imposes, which is the highest `rust-version` any package outside
       this workspace declares.

  ⚠ THE GRAPH IS ONE OF TWO LEGS AND IT IS THE WEAKER ONE. A graph with no
  dependencies imposes no floor at all, which is this tree's state today, and a
  check that reported a floor there would be inventing one. The other leg is
  -Verify, which COMPILES the workspace with the declared toolchain and is the
  only thing that can say the declared value is reachable. Neither leg alone is
  a measurement of the true minimum: the graph cannot see the language features
  the code uses, and -Verify proves the declared version WORKS without proving
  nothing older would.

  ⛔ WORKSPACE MEMBERS ARE EXCLUDED FROM THE FLOOR, and that exclusion is the
  whole reason this check can fail. Every member inherits `rust-version` from
  the workspace, so a floor computed over all packages would read back the
  value it is checking and agree with itself forever. That is the "acceptance
  command that cannot fail" row in docs/conventions/forbidden-patterns.md.

  ⚠ -Write IS THE FIX FLAG and it REFUSES when the graph imposes no floor.
  A helper that guessed a version there would be writing the exact fabricated
  number this check exists to find.

  Exit codes: 0 clean, 1 the declared value is missing or too low, 2 could not
  run (no cargo, or -Verify with the declared toolchain not installed).

  ⛔ Read the exit code from this process, unpiped.

.EXAMPLE
  pwsh -NoProfile -File scripts/common/check-msrv.ps1
.EXAMPLE
  pwsh -NoProfile -File scripts/common/check-msrv.ps1 -Json
.EXAMPLE
  pwsh -NoProfile -File scripts/common/check-msrv.ps1 -Verify
.EXAMPLE
  pwsh -NoProfile -File scripts/common/check-msrv.ps1 -Write
#>
[CmdletBinding()]
param(
    [switch]$Json,
    [switch]$Write,
    [switch]$Verify,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. `pwsh -File` reports a parameter-binding failure as 1, which is
    # this project's code for "it ran and the thing failed"; the POSIX twin
    # exits 2 for the same input. Measured across every pair 2026-09-02:
    # 22 of 22 disagreed. TODO/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-msrv: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Exit-With {
    param([int]$Code, [string]$Message)
    if ($Message) { [Console]::Error.WriteLine($Message) }
    exit $Code
}

# ⚠ Get-Command finds cmdlets, functions and aliases too, so it is filtered to
# a real executable. docs/conventions/shell.md section 8.
function Get-Exe {
    param([string]$Name)
    $c = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue |
         Select-Object -First 1
    if ($c) { return $c.Source }
    return $null
}

# ⛔ Resolved from this script's own location, never from the caller's working
# directory. scripts/README.md, contract point 4.
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location -LiteralPath $root

$manifest = Join-Path $root 'Cargo.toml'
if (-not (Test-Path -LiteralPath $manifest)) {
    Exit-With 2 "check-msrv: no Cargo.toml in this repository.`n  That is `"could not run`", not `"passed`"."
}

$cargo = Get-Exe 'cargo'
if (-not $cargo) { Exit-With 2 'check-msrv: cargo is not on PATH' }

# -- the declared value ------------------------------------------------------
# ⚠ Scoped to the [workspace.package] table. A `rust-version` under some other
# table is a different field, and matching it anywhere in the file is how a
# check reads the wrong one.
$declared = ''
$inTable = $false
foreach ($line in [System.IO.File]::ReadAllLines($manifest)) {
    if ($line -match '^\[') { $inTable = ($line.Trim() -ceq '[workspace.package]'); continue }
    if ($inTable -and $line -match '^\s*rust-version\s*=') {
        $value = $line -replace '^[^=]*=\s*', ''
        $value = $value -replace '#.*$', ''
        $value = $value -replace '["'']', ''
        $declared = $value.Trim()
        break
    }
}

# -- the floor the resolved graph imposes ------------------------------------
# ⛔ Resolution can reach the network the first time. It is still the right
# command: --no-deps would answer with the workspace alone, which is the set
# this check has to exclude.
$prev = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
    $metaRaw = & $cargo metadata --format-version 1 2>$null
    $metaRc = $LASTEXITCODE
}
finally { $ErrorActionPreference = $prev }
if ($metaRc -ne 0) {
    Exit-With 2 "check-msrv: cargo metadata failed. Run it directly to see why:`n  cargo metadata --format-version 1"
}

$meta = ($metaRaw | Out-String) | ConvertFrom-Json
$members = @($meta.workspace_members)
$packages = @($meta.packages)
$deps = @($packages | Where-Object { $members -notcontains $_.id })

function Get-VersionNumber {
    param([string]$Version)
    $p = ($Version -split '\.')
    $a = 0; $b = 0; $c = 0
    if ($p.Count -ge 1) { [void][int]::TryParse($p[0], [ref]$a) }
    if ($p.Count -ge 2) { [void][int]::TryParse($p[1], [ref]$b) }
    if ($p.Count -ge 3) { [void][int]::TryParse($p[2], [ref]$c) }
    return ($a * 1000000) + ($b * 1000) + $c
}

$floorVersion = ''
$floorPackage = ''
$best = 0
foreach ($p in $deps) {
    $rv = $null
    if ($p.PSObject.Properties.Name -contains 'rust_version') { $rv = $p.rust_version }
    if (-not $rv) { continue }
    $n = Get-VersionNumber $rv
    if ($n -gt $best) { $best = $n; $floorVersion = $rv; $floorPackage = $p.name }
}

# -- the fix flag, which refuses rather than guessing -------------------------
if ($Write) {
    if (-not $floorVersion) {
        $m = "check-msrv: REFUSED. The dependency graph imposes no floor, so there is`n"
        $m += "  nothing to derive and nothing to write. $($deps.Count) package(s) resolved, none`n"
        $m += "  of them outside this workspace declares a rust-version.`n"
        $m += '  ⛔ A version invented here would be the defect this check exists to find.'
        Exit-With 2 $m
    }
    $node = Get-Exe 'node'
    if (-not $node) { Exit-With 2 'check-msrv: -Write needs node for write-file.mjs' }
    if (-not $declared) { Exit-With 2 'check-msrv: -Write cannot patch a field that is absent. Add it first.' }
    $enc = [System.Text.Encoding]::UTF8
    $findB64 = [Convert]::ToBase64String($enc.GetBytes("rust-version = `"$declared`""))
    $replB64 = [Convert]::ToBase64String($enc.GetBytes("rust-version = `"$floorVersion`""))
    # ⛔ One write path. write-file.mjs refuses a match count that differs from
    # what was declared and leaves the file untouched, which is what a silent
    # no-op reporting success would not do.
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & $node (Join-Path $root 'scripts/common/write-file.mjs') replace 'Cargo.toml' `
            '--find-b64' $findB64 '--replace-b64' $replB64 '--expect' '1'
        $wrc = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $prev }
    if ($wrc -ne 0) { Exit-With 2 'check-msrv: write-file.mjs refused the substitution; Cargo.toml is untouched.' }
    Write-Output "check-msrv: rust-version $declared -> $floorVersion, derived from $floorPackage"
    Write-Output '  Now read it back: pwsh -NoProfile -File scripts/common/check-msrv.ps1'
    exit 0
}

# -- the verify leg, which compiles ------------------------------------------
$verified = 0
if ($Verify) {
    if (-not $declared) { Exit-With 1 'check-msrv: -Verify has nothing to verify: no rust-version is declared.' }
    $rustup = Get-Exe 'rustup'
    if (-not $rustup) { Exit-With 2 'check-msrv: -Verify needs rustup to select a toolchain' }
    # ⛔ BOTH BINARIES, NOT CARGO ALONE. Measured here on 2026-08-31: an install
    # killed part-way registers the toolchain and leaves a working `cargo`
    # beside a rustc with no manifest. A guard that probed cargo alone let that
    # through, and `cargo check` then failed on `rustc -vV`, which this script
    # reported as "the workspace does NOT compile". ⚠ That is a broken host
    # accusing the tree, which is the exact confusion between "failed" and
    # "could not run" that the three exit codes exist to keep apart.
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & $rustup run $declared cargo --version *> $null
        $probeCargo = $LASTEXITCODE
        & $rustup run $declared rustc --version *> $null
        $probeRustc = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $prev }
    if ($probeCargo -ne 0 -or $probeRustc -ne 0) {
        $m = "check-msrv: toolchain $declared is not installed, or is installed incompletely.`n"
        $m += "  That is `"could not run`", not `"failed`". Install it and re-run:`n"
        $m += "    rustup toolchain install $declared"
        Exit-With 2 $m
    }
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $vOut = & $rustup run $declared cargo check --workspace --all-targets 2>&1
        $vRc = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $prev }
    if ($vRc -ne 0) {
        [Console]::Error.WriteLine("check-msrv: the workspace does NOT compile on the declared $declared.")
        foreach ($l in ($vOut | Out-String) -split "`r?`n") {
            if ($l.Trim()) { [Console]::Error.WriteLine("  | $l") }
        }
        exit 1
    }
    $verified = 1
}

# -- the verdict -------------------------------------------------------------
$problems = @()
if (-not $declared) {
    $problems += "  Cargo.toml: [workspace.package] declares no rust-version. An absent one is not 'any version': it is a promise nobody made."
}
elseif ($floorVersion) {
    if ((Get-VersionNumber $declared) -lt (Get-VersionNumber $floorVersion)) {
        $problems += "  Cargo.toml: rust-version is $declared, and the dependency graph needs $floorVersion ($floorPackage). Derive it: pwsh -NoProfile -File scripts/common/check-msrv.ps1 -Write"
    }
}

if ($Json) {
    $line = '{"schema":"check-msrv/1","declared":"' + $declared + '","graph_floor":"' + $floorVersion +
            '","packages":' + $packages.Count + ',"dependencies":' + $deps.Count +
            ',"verified":' + $verified + ',"problems":' + $problems.Count + '}'
    Write-Output $line
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -gt 0) {
    Write-Output "msrv check failed, $($problems.Count) problem(s):"
    Write-Output ''
    foreach ($p in $problems) { Write-Output $p }
    exit 1
}

$summary = "msrv ok: declared $declared"
if ($floorVersion) { $summary += ", graph floor $floorVersion from $floorPackage" }
else { $summary += ", graph floor none ($($deps.Count) dependency package(s) resolved)" }
if ($verified -eq 1) { $summary += ", compiles on $declared" }
Write-Output $summary
exit 0
