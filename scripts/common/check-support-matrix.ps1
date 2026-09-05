# check-support-matrix.ps1 - is every cell in the support matrix produced by a
# run, and does every hole still point at something?
#
# ⭐ THE TWIN OF check-support-matrix.sh. docs/history/todo/driver.md, DRIVER-09, is why a
# script in this directory does not land without one.
#
# ⛔ A CLIENT AUTHOR CURRENTLY FINDS OUT WHICH STACK CAN EMIT WHICH PROFILE BY
# BUILDING IT. docs/history/todo/emitters.md, EMIT-01.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the matrix is GENERATED here rather than read from a committed file;
#   2. ⛔ EVERY CELL IS EVIDENCE `run` and names the command that reproduces it;
#   3. ⛔ EVERY HOLE IS EVIDENCE `read`, cites a path under references/ and a
#      line, and that path and line still resolve;
#   4. every published profile has a cell;
#   5. ⭐ there is at least one hole.
#
# ⚠ THE READING IS THIS HALF'S OWN: ConvertFrom-Json where the twin uses jq, so
# the pair compares two readings rather than two wrappers over one.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-support-matrix.ps1
#   pwsh -NoProfile -File scripts/common/check-support-matrix.ps1 -Json
#
# Exit codes: 0 the matrix is what it claims, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-support-matrix: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-support-matrix: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-support-matrix: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. docs/history/todo/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-support-matrix: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-support-matrix: cargo not found')
    exit 2
}

$problems = New-Object System.Collections.ArrayList
$out = Join-Path $root '.tmp/check-support-matrix-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Recurse -Force }
New-Item -ItemType Directory -Path $out -Force | Out-Null

& cargo build -q -p b-ids-cli
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-support-matrix: the client did not build')
    exit 2
}
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
$bin = Join-Path $targetDir 'debug/b-ids-cli.exe'
if (-not (Test-Path -LiteralPath $bin -PathType Leaf)) {
    $bin = Join-Path $targetDir 'debug/b-ids-cli'
}
if (-not (Test-Path -LiteralPath $bin -PathType Leaf)) {
    [Console]::Error.WriteLine("check-support-matrix: $bin is not executable")
    exit 2
}

# ⛔ GENERATED, NEVER READ FROM A COMMITTED FILE.
$matrixPath = Join-Path $out 'matrix.json'
& $bin --matrix > $matrixPath
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine("check-support-matrix: the generator exited $LASTEXITCODE")
    exit 1
}
try {
    $built = Get-Content -LiteralPath $matrixPath -Raw | ConvertFrom-Json
}
catch {
    [Console]::Error.WriteLine('check-support-matrix: the generator did not emit json')
    exit 1
}

if ($built.schema -ne 'emit-support-matrix/1') {
    [void]$problems.Add("  the matrix names schema $($built.schema)")
}

$cells = @($built.cells)
$holes = @($built.holes)

# -- 2: every cell is a run, with a command ----------------------------------
$typed = @($cells | Where-Object { $_.evidence -ne 'run' }).Count
if ($typed -ne 0) {
    [void]$problems.Add("  $typed cell(s) are not evidence run, and a cell filled any other way is a hole wearing a cell's clothes")
}
$noCommand = @($cells | Where-Object { -not $_.reproduce }).Count
if ($noCommand -ne 0) {
    [void]$problems.Add("  $noCommand cell(s) name no command that reproduces them")
}

# -- 3: every hole is a reading whose citation still resolves ----------------
if ($holes.Count -lt 1) {
    [void]$problems.Add('  the matrix declares no hole at all, and a matrix with none is one nobody filled honestly')
}
$resolved = 0
foreach ($hole in $holes) {
    if ($hole.evidence -ne 'read') {
        [void]$problems.Add("  $($hole.stack): a hole is evidence $($hole.evidence), and a hole is a reading")
    }
    if ($hole.file -notlike 'references/*') {
        [void]$problems.Add("  $($hole.stack): $($hole.file) is not under references/, so nothing holds it at a named commit")
    }
    if (-not (Test-Path -LiteralPath $hole.file -PathType Leaf)) {
        [void]$problems.Add("  $($hole.stack): $($hole.file) does not exist, so the evidence for this hole no longer resolves")
        continue
    }
    $have = @(Get-Content -LiteralPath $hole.file).Count
    if ($have -lt $hole.line) {
        [void]$problems.Add("  $($hole.stack): $($hole.file) has $have line(s) and the hole cites line $($hole.line)")
        continue
    }
    $resolved++
}

# -- 4: every published profile has a cell -----------------------------------
$profileCount = @(Get-ChildItem -LiteralPath (Join-Path $corpusRoot 'corpus/v1') -Recurse -File -Filter '*.json' |
        Where-Object { $_.Name -ne 'index.json' -and $_.Name -ne 'latest.json' }).Count
if ($cells.Count -ne $profileCount) {
    [void]$problems.Add("  the matrix carries $($cells.Count) cell(s) over $profileCount published profile(s)")
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-support-matrix/1","cells":' + $cells.Count +
                  ',"holes":' + $holes.Count + ',"resolved":' + $resolved +
                  ',"profiles":' + $profileCount + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output "support matrix ok: $($cells.Count) cell(s) over $profileCount profile(s), every one produced by a run,"
    Write-Output "  and $resolved of $($holes.Count) hole(s) still resolving to a file and a line under references/."
    Write-Output "  `u{26D4} A cell is a run and a hole is a reading, and this check keeps them apart."
    exit 0
}

[Console]::Error.WriteLine("support matrix check failed, $count problem(s):")
[Console]::Error.WriteLine('')
$problems | ForEach-Object { [Console]::Error.WriteLine($_) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A cell that says "approximately" is worse than one that says "cannot".')
[Console]::Error.WriteLine('docs/history/todo/emitters.md, EMIT-01.')
exit 1
