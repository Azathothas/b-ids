# check-generated-configs.ps1 - the PowerShell half of check-generated-configs.sh.
#
# ⛔ MOST OF THE DAY-TO-DAY VALUE OF A CORPUS IS THE ARTEFACT SOMEBODY PASTES
# INTO THEIR OWN TOOL, and a snippet that silently approximates is worse than no
# snippet: it produces a client that is almost right, which is more
# distinguishing than an honestly old one. TODO/publish.md, PUB-04.
#
# ⛔ THE SH HALF CARRIES THE FULL CONTRACT AND THIS ONE FOLLOWS IT. What each
# assertion is and why is written there; check-twins runs both halves over one
# tree and compares the JSON and the exit code, so a change here that is not
# made there is drift the comparison catches.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-generated-configs.ps1
#   pwsh -NoProfile -File scripts/common/check-generated-configs.ps1 -Json
#
# Exit codes: 0 every snippet is gated, 1 one is not, 2 could not run.

param(
    [switch]$Json,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

$ErrorActionPreference = 'Stop'

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine("check-generated-configs: unknown argument: $($UnboundArguments[0])")
    exit 2
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-generated-configs: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-generated-configs: not a git repository')
    exit 2
}
$root = "$root".Trim()
Set-Location $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. TODO/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-generated-configs: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
$env:B_IDS_CORPUS_ROOT = $corpusRoot

if (-not (Test-Path -LiteralPath (Join-Path $corpusRoot 'corpus'))) {
    [Console]::Error.WriteLine('check-generated-configs: there is no corpus, so nothing was generated')
    exit 2
}
foreach ($tool in @('cargo', 'jq')) {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        [Console]::Error.WriteLine("check-generated-configs: $tool not found")
        exit 2
    }
}

$problems = New-Object System.Collections.ArrayList
function Add-Problem([string]$Text) { [void]$problems.Add($Text) }

$out = Join-Path $root '.tmp/check-generated-configs-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Recurse -Force -Confirm:$false }
$null = New-Item -ItemType Directory -Path $out -Force

$tree = Join-Path $out 'tree'
$publishLog = Join-Path $out 'publish.log'
& cargo run -q -p b-ids-corpus -- publish --root $corpusRoot --out $tree *> $publishLog
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine("check-generated-configs: the assembler exited $LASTEXITCODE")
    Get-Content $publishLog | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}

$configs = Join-Path $tree 'configs'
if (-not (Test-Path -LiteralPath $configs)) {
    [Console]::Error.WriteLine('check-generated-configs: the assembler produced no configs/ directory')
    exit 1
}

& cargo build -q -p b-ids-cli
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-generated-configs: the client did not build')
    exit 2
}
$bin = Join-Path $root 'target/debug/b-ids-cli'
if (-not (Test-Path -LiteralPath $bin)) { $bin = "$bin.exe" }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine("check-generated-configs: $bin is not executable")
    exit 2
}

$matrixPath = Join-Path $out 'matrix.json'
& $bin --matrix > $matrixPath 2> (Join-Path $out 'matrix.err')
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine("check-generated-configs: the matrix generator exited $LASTEXITCODE")
    exit 1
}
$matrix = Get-Content $matrixPath -Raw | ConvertFrom-Json
$holeStacks = @($matrix.holes | ForEach-Object { $_.stack } | Sort-Object -Unique)
$runnable = if ($matrix.cells.Count -gt 0) { $matrix.cells[0].stack } else { '' }
if (-not $runnable) {
    [Console]::Error.WriteLine('check-generated-configs: the matrix has no cell to read a stack from')
    exit 2
}

$snippets = 0
$refusals = 0
$detect = 0

foreach ($file in (Get-ChildItem -LiteralPath $configs -Recurse -File | Sort-Object FullName)) {
    $base = $file.Name
    if ($base -eq 'README.md') { continue }

    # ⚠ The relative path is built with forward slashes so both halves report
    # the same string. $_.FullName is backslash-separated on Windows.
    $relative = $file.FullName.Substring($configs.Length + 1) -replace '\\', '/'

    if ($base -eq 'detect.conf') {
        $detect++
        $body = Get-Content -LiteralPath $file.FullName -Raw
        if ($body -imatch '\bja4|\bja3|sha256:|digest') {
            Add-Problem "$base names a digest, and the corpus holds none: $relative"
        }
        continue
    }

    $stack = [System.IO.Path]::GetFileNameWithoutExtension($base)
    $ext = $file.Extension.TrimStart('.')

    if ($holeStacks -contains $stack) {
        $refusals++
        if ($ext -eq 'rs') {
            Add-Problem "$stack has a hole in the matrix and a snippet in the tree: $relative"
            continue
        }
        $body = Get-Content -LiteralPath $file.FullName -Raw
        if ($body -notmatch 'NO SNIPPET IS GENERATED') {
            Add-Problem "$stack's refusal does not say it is one: $relative"
        }
        if ($body -notmatch 'read at references/[^:]+:[0-9]+') {
            Add-Problem "$stack's refusal names no file and line: $relative"
        }
    } elseif ($stack -eq $runnable) {
        $snippets++
        # ⛔ THE PAIR, NOT THE STACK. A cell for one profile does not license a
        # snippet for another, and the profile is what the file names.
        $id = ''
        $head = Get-Content -LiteralPath $file.FullName -TotalCount 8
        foreach ($line in $head) {
            if ($line -match 'profile\s+(\S+)') { $id = $Matches[1]; break }
        }
        if (-not $id) {
            Add-Problem "$stack's snippet names no profile: $relative"
        } else {
            $emitting = @($matrix.cells | Where-Object { $_.stack -eq $stack -and $_.profile -eq $id -and $_.emits })
            if ($emitting.Count -ne 1) {
                Add-Problem "$stack has a snippet for $id and the matrix has no emitting cell for that pair"
            }
        }
    } else {
        Add-Problem "there is a file for stack $stack, which the matrix has neither a cell nor a hole for"
    }
}

$profiles = @(Get-ChildItem -LiteralPath (Join-Path $corpusRoot 'corpus') -Recurse -File -Filter '*.json' |
    Where-Object { $_.Name -ne 'index.json' -and $_.Name -ne 'latest.json' }).Count
$dirs = @(Get-ChildItem -LiteralPath $configs -Recurse -Directory |
    Where-Object { @(Get-ChildItem -LiteralPath $_.FullName -File).Count -gt 0 }).Count
if ($profiles -ne $dirs) {
    Add-Problem "the corpus has $profiles profile(s) and the tree has $dirs configuration director(y/ies)"
}

if ($refusals -le 0) {
    Add-Problem 'no stack was refused, so nothing here was gated on anything'
}

if ($Json) {
    Write-Output ('{"schema":"check-generated-configs/1","snippets":' + $snippets +
                  ',"refusals":' + $refusals +
                  ',"detection":' + $detect +
                  ',"profiles":' + $profiles +
                  ',"problems":' + $problems.Count + '}')
    if ($problems.Count -ne 0) { exit 1 }
    exit 0
}

if ($problems.Count -ne 0) {
    [Console]::Error.WriteLine("generated configs check failed, $($problems.Count) problem(s):")
    [Console]::Error.WriteLine('')
    foreach ($p in $problems) { [Console]::Error.WriteLine("  $p") }
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('A snippet is generated only where the support matrix says the pair can')
    [Console]::Error.WriteLine('emit. Fix the generator, never this check.')
    exit 1
}

Write-Output "generated configs ok: $snippets snippet(s) over $profiles profile(s), each for a pair the"
Write-Output "  matrix marks emittable, and $refusals refusal(s) naming a hole at a file and a line."
Write-Output "  $detect detection rule(s), none naming a digest the corpus does not hold."
exit 0
