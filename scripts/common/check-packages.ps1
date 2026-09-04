# check-packages.ps1 - does each language package build offline, report the
# corpus release it embeds, and does that release match the one it was cut from?
#
# ⭐ THE TWIN OF check-packages.sh. Same schema, same exit codes, same legs.
# check-twins.sh is what stops the two drifting.
#
# ⛔ FETCHING AND PARSING A CORPUS IS WORK. A DEPENDENCY LINE IS NOT.
# TODO/publish.md, PUB-05.
#
# ⚠ NO sha256sum HERE. A native PowerShell session has none, and the job exists
# on both platforms even where the tool does not: Get-FileHash is the platform's
# own implementation and it is equally not this project's code, which is the
# property that makes recomputing the pin worth anything.
#
# ⚠ THE RUNTIME IS THE SKIP. Without node there is nothing to import the package
# with, and a skip is reported as a skip.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-packages.ps1
#   pwsh -NoProfile -File scripts/common/check-packages.ps1 -Json
#
# Exit codes: 0 every package is what it says it is, 1 one is not, 2 could not run.
#
# ⛔ Read $LASTEXITCODE from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1, which is what the POSIX twin does for the same input.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-packages: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-packages: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-packages: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-packages: cargo not found')
    exit 2
}

$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-packages: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
$env:B_IDS_CORPUS_ROOT = $corpusRoot

$problems = @()
$out = Join-Path (Join-Path $root '.tmp') 'check-packages-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Recurse -Force }
New-Item -ItemType Directory -Path $out -Force | Out-Null

$tree = Join-Path $out 'tree'
& cargo run -q -p b-ids-corpus -- publish --root $corpusRoot --out $tree *> (Join-Path $out 'publish.log')
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-packages: the assembler did not build the tree')
    Get-Content -LiteralPath (Join-Path $out 'publish.log') | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 2
}

# ⛔ THE ECOSYSTEMS ARE READ FROM THE CRATE, never listed here.
$source = Get-Content -LiteralPath (Join-Path $root 'crates/b-ids-corpus/src/packages.rs') -Raw
$ecosystems = @()
if ($source -match 'pub const ECOSYSTEMS[^=]*=\s*\[([^\]]*)\]') {
    $ecosystems = @([regex]::Matches($Matches[1], '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value })
}
if ($ecosystems.Count -eq 0) {
    [Console]::Error.WriteLine('check-packages: the crate names no ecosystem')
    exit 2
}

# ⭐ THE PIN, RECOMPUTED with the platform's own hash rather than taken from the
# generator.
$indexPath = Join-Path (Join-Path (Join-Path $corpusRoot 'corpus') 'v1') 'index.json'
$want = (Get-FileHash -LiteralPath $indexPath -Algorithm SHA256).Hash.ToLowerInvariant()
$corpusDir = Join-Path (Join-Path $corpusRoot 'corpus') 'v1'
$profileCount = @(Get-ChildItem -LiteralPath $corpusDir -Recurse -File -Filter '*.json' |
        Where-Object { $_.Name -ne 'index.json' -and $_.Name -ne 'latest.json' }).Count

$node = Get-Command node -CommandType Application -ErrorAction SilentlyContinue
$built = 0
$driven = 0
foreach ($eco in $ecosystems) {
    $dir = Join-Path (Join-Path $tree 'packages') $eco
    if (-not (Test-Path -LiteralPath $dir)) {
        $problems += "the crate names the $eco ecosystem and the assembler wrote no packages/$eco"
        continue
    }
    $built++

    # ⛔ IT EMBEDS AND IT DOES NOT FETCH.
    $sources = @(Get-ChildItem -LiteralPath $dir -File | Where-Object { $_.Extension -in '.mjs', '.js' })
    foreach ($file in $sources) {
        $text = Get-Content -LiteralPath $file.FullName -Raw
        foreach ($forbidden in @('fetch(', 'XMLHttpRequest', 'require("http', "require('http", 'node:http', 'node:https')) {
            if ($text.Contains($forbidden)) {
                $problems += "packages/$eco names $forbidden, and a package that fetches at runtime is what PUB-05 forbids"
            }
        }
    }

    if ($eco -ne 'js') {
        $problems += "no runner is written for the $eco ecosystem, so its package was generated and not driven"
        continue
    }
    if (-not $node) { continue }

    $script = 'import("./index.mjs").then((m) => { const r = m.release(); ' +
              'console.log(JSON.stringify({ identifier: r.identifier, profiles: m.profiles().length, ' +
              'paths: m.paths().length, newestCapture: r.newestCapture, ' +
              'selectable: m.select({ browser: "chrome" }).length, ' +
              'absent: m.latestStable("chrome", "macos-arm64") === undefined })); ' +
              '}).catch((e) => { console.error(String(e)); process.exit(1); });'
    Push-Location -LiteralPath $dir
    $answer = & $node.Source -e $script 2>(Join-Path $out "$eco.err")
    $rc = $LASTEXITCODE
    Pop-Location
    if ($rc -ne 0) {
        $problems += "packages/$eco did not import"
        continue
    }
    $driven++
    $got = ($answer | Select-Object -First 1) | ConvertFrom-Json
    if ($got.identifier -ne $want) {
        $problems += "packages/$eco reports release $($got.identifier) and the corpus index digests to $want"
    }
    if ($got.profiles -ne $profileCount) {
        $problems += "packages/$eco embeds $($got.profiles) profile(s) and the corpus holds $profileCount"
    }
    if ($got.paths -ne $got.profiles) {
        $problems += "packages/$eco reports $($got.paths) path(s) for $($got.profiles) profile(s), and it is one each"
    }
    if (-not $got.absent) {
        $problems += "packages/$eco answered for a platform the corpus has no profile on"
    }
}

Remove-Item Env:B_IDS_CORPUS_ROOT -ErrorAction SilentlyContinue
$nodeState = if ($node) { 'present' } else { 'absent' }

if ($Json) {
    Write-Output ('{"schema":"check-packages/1","ecosystems":' + $ecosystems.Count +
                  ',"built":' + $built +
                  ',"driven":' + $driven +
                  ',"profiles":' + $profileCount +
                  ',"node":"' + $nodeState +
                  '","problems":' + $problems.Count + '}')
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -eq 0) {
    Write-Output "packages ok: $built ecosystem(s) generated, $driven driven by their own runtime,"
    Write-Output "  each embedding $profileCount profile(s) and reporting release $want,"
    Write-Output '  which is what the platform hash makes of the corpus index here.'
    Write-Output '  ⛔ Nothing generated fetches at runtime.'
    if ($nodeState -eq 'absent') {
        Write-Output '  ⚠ SKIP: no node on this host, so the js package was generated and not'
        Write-Output '  imported. A skip is not a pass.'
    }
    exit 0
}

[Console]::Error.WriteLine("packages check failed, $($problems.Count) problem(s):")
[Console]::Error.WriteLine('')
foreach ($p in $problems) { [Console]::Error.WriteLine('  ' + $p) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A package that needs the network to answer fails in the environment its')
[Console]::Error.WriteLine('consumers care most about. TODO/publish.md, PUB-05.')
exit 1
