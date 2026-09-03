# check-catalogues.ps1 - is every script and every document named by the
# catalogue that claims to list it?
#
# ⭐ THE TWIN OF check-catalogues.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# ⛔ A CATALOGUE NOTHING CHECKS STOPS BEING A CATALOGUE. docs/AGENTS.md calls
# scripts/README.md the contract every script is held to, and carries its own
# table of what each document owns. Neither was compared against the tree:
# measured 2026-09-03, THIRTEEN of the checks the gate runs had no section in
# scripts/README.md at all, and the gate was green throughout.
# TODO/tooling.md, TOOL-19.
#
# ⛔ THE TWO RULES, AND THEY ARE THE ONLY TWO. Every script under scripts/ is
# named by scripts/README.md, with twins collapsed to one base name because a
# pair is one contract. Every document under docs/ is named by its index:
# docs/AGENTS.md for the tree, docs/HISTORY/README.md for the history directory.
#
# ⛔ IT DOES NOT READ THE PROSE. Whether a section is any good is a review.
# ⚠ THE OTHER DIRECTION IS ALREADY HELD by check-docs, which resolves every
# cited path in every markdown file here.
#
# ⛔ AN EMPTY SCOPE IS EXIT 2, because a check reporting clean over nothing is
# how it quietly stops applying.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-catalogues.ps1
#   pwsh -NoProfile -File scripts/common/check-catalogues.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-catalogues.ps1 -Fixture
#   pwsh -NoProfile -File scripts/common/check-catalogues.ps1 -Fixtures DIR
#
# Exit codes: 0 every one is named, 1 one is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    [switch]$Fixture,
    [string]$Fixtures = '',
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1, which is what the POSIX twin does for the same input.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-catalogues: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-catalogues: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-catalogues: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

$script:problems = New-Object System.Collections.Generic.List[string]

# ⭐ THE BASE NAME OF EVERY SCRIPT, twins collapsed. Keep this projection
# identical to the sh twin's sed program.
function Get-ScriptCatalogue {
    param([string]$Mode)
    $paths = if ($Mode -eq 'git') {
        @(& git ls-files -- 'scripts/*') + @(& git ls-files --others --exclude-standard -- 'scripts/*')
    }
    else {
        @(Get-ChildItem -LiteralPath 'scripts' -Recurse -File -ErrorAction SilentlyContinue |
            ForEach-Object { $_.FullName })
    }
    @($paths |
        Where-Object { "$_" -match '\.(sh|ps1|mjs)$' } |
        ForEach-Object { [System.IO.Path]::GetFileNameWithoutExtension("$_") } |
        Sort-Object -Unique -CaseSensitive)
}

# ⭐ EVERY DOCUMENT WITH THE INDEX THAT OWNS IT. ⛔ The two indexes name
# themselves and are skipped: a file listing itself proves nothing about whether
# anything routes to it.
function Get-DocCatalogue {
    param([string]$Mode)
    $paths = if ($Mode -eq 'git') {
        @(& git ls-files -- 'docs/*.md') + @(& git ls-files --others --exclude-standard -- 'docs/*.md')
    }
    else {
        @(Get-ChildItem -LiteralPath 'docs' -Recurse -File -Filter '*.md' -ErrorAction SilentlyContinue |
            ForEach-Object { $_.FullName.Substring((Get-Location).Path.Length + 1) -replace '\\', '/' })
    }
    $out = New-Object System.Collections.Generic.List[object]
    foreach ($p in ($paths | Sort-Object)) {
        $rel = "$p"
        if ($rel -eq 'docs/AGENTS.md' -or $rel -eq 'docs/HISTORY/README.md') { continue }
        if ($rel.StartsWith('docs/HISTORY/')) {
            $out.Add([pscustomobject]@{ Index = 'docs/HISTORY/README.md'; Name = $rel.Substring('docs/HISTORY/'.Length) })
        }
        else {
            $out.Add([pscustomobject]@{ Index = 'docs/AGENTS.md'; Name = $rel.Substring('docs/'.Length) })
        }
    }
    , $out
}

# ⛔ ONE SCAN, USED BY THE REAL RUN AND BY THE FIXTURE, so what the fixture
# proves is what the real run does.
function Invoke-Scan {
    param([string]$Mode)

    $scriptIndex = 'scripts/README.md'
    if (-not (Test-Path -LiteralPath $scriptIndex)) {
        [Console]::Error.WriteLine("check-catalogues: $scriptIndex is missing")
        return @{ Rc = 2; Scripts = 0; Documents = 0 }
    }
    $scriptText = Get-Content -LiteralPath $scriptIndex -Raw

    $names = Get-ScriptCatalogue -Mode $Mode
    foreach ($n in $names) {
        if (-not $scriptText.Contains($n)) {
            $script:problems.Add("$n has no mention in $scriptIndex")
        }
    }

    $pairs = Get-DocCatalogue -Mode $Mode
    $cache = @{}
    foreach ($pair in $pairs) {
        if (-not (Test-Path -LiteralPath $pair.Index)) {
            $script:problems.Add(($pair.Name + ' is owned by ' + $pair.Index + ', which is missing'))
            continue
        }
        if (-not $cache.ContainsKey($pair.Index)) {
            $cache[$pair.Index] = Get-Content -LiteralPath $pair.Index -Raw
        }
        if (-not $cache[$pair.Index].Contains($pair.Name)) {
            $script:problems.Add(($pair.Name + ' has no mention in ' + $pair.Index))
        }
    }

    if ($names.Count -eq 0 -or $pairs.Count -eq 0) {
        [Console]::Error.WriteLine('check-catalogues: scope is empty (' + $names.Count +
            ' script(s), ' + $pairs.Count + ' document(s))')
        return @{ Rc = 2; Scripts = $names.Count; Documents = $pairs.Count }
    }
    return @{ Rc = 0; Scripts = $names.Count; Documents = $pairs.Count }
}

if ($Fixture) {
    $fix = Join-Path ([System.IO.Path]::GetTempPath()) ('checkcatalogues-' + [System.Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path (Join-Path $fix 'scripts/common') -Force | Out-Null
    New-Item -ItemType Directory -Path (Join-Path $fix 'docs/methodology') -Force | Out-Null
    Set-Content -LiteralPath (Join-Path $fix 'scripts/README.md') -Value 'catalogue naming alpha and nothing else'
    Set-Content -LiteralPath (Join-Path $fix 'scripts/common/alpha.sh') -Value '#!/bin/sh'
    Set-Content -LiteralPath (Join-Path $fix 'scripts/common/beta.sh') -Value '#!/bin/sh'
    Set-Content -LiteralPath (Join-Path $fix 'docs/AGENTS.md') -Value 'router naming methodology/one.md and nothing else'
    Set-Content -LiteralPath (Join-Path $fix 'docs/methodology/one.md') -Value '# one'
    Set-Content -LiteralPath (Join-Path $fix 'docs/methodology/two.md') -Value '# two'

    Push-Location -LiteralPath $fix
    try {
        $r = Invoke-Scan -Mode 'find'
    }
    finally {
        Pop-Location
        Remove-Item -LiteralPath $fix -Recurse -Force -ErrorAction SilentlyContinue
    }
    if ($r.Rc -eq 2) {
        [Console]::Error.WriteLine('check-catalogues: the fixture scan could not run')
        exit 2
    }
    if ($script:problems.Count -ne 2) {
        [Console]::Error.WriteLine('check-catalogues: the fixture expected 2 refusals and got ' + $script:problems.Count + '.')
        $script:problems | ForEach-Object { [Console]::Error.WriteLine('  ' + $_) }
        exit 2
    }
    Write-Output 'check-catalogues fixture ok: an unlisted script and an unlisted document'
    Write-Output 'are both refused.'
    exit 0
}

# ⚠ -Fixtures WALKS THE FILESYSTEM instead of asking git, because a tree that is
# not this repository has no index to ask.
$target = $root
$mode = 'git'
if ($Fixtures -ne '') {
    $target = if ([System.IO.Path]::IsPathRooted($Fixtures)) { $Fixtures } else { Join-Path $root $Fixtures }
    if (-not (Test-Path -LiteralPath $target -PathType Container)) {
        [Console]::Error.WriteLine('check-catalogues: no such directory: ' + $Fixtures)
        exit 2
    }
    $mode = 'find'
}

Push-Location -LiteralPath $target
try {
    $r = Invoke-Scan -Mode $mode
}
finally {
    Pop-Location
}
if ($r.Rc -eq 2) { exit 2 }

$count = $script:problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-catalogues/1","scripts":' + $r.Scripts +
                  ',"documents":' + $r.Documents + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -gt 0) {
    Write-Output ('catalogue check failed, ' + $count + ' of ' + $r.Scripts +
                  ' script(s) and ' + $r.Documents + ' document(s):')
    Write-Output ''
    $script:problems | ForEach-Object { Write-Output ('  ' + $_) }
    Write-Output ''
    Write-Output 'A script gets a section in scripts/README.md and a document gets a row'
    Write-Output 'in the index that routes to it. docs/conventions/docs.md.'
    exit 1
}

Write-Output ('catalogues ok: ' + $r.Scripts + ' script(s) named by scripts/README.md, ' +
              $r.Documents + ' document(s)')
Write-Output 'named by the index that owns each one.'
exit 0
