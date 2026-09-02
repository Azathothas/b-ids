# check-formats.ps1 - does every published format come out of the one generator,
# round-trip, and produce the same bytes twice?
#
# ⭐ THE TWIN OF check-formats.sh. TODO/schema.md, SCHEMA-08, and TODO/driver.md,
# DRIVER-09, is why a script in this directory does not land without one.
#
# ⛔ JSON IS ONE CONSUMER, NOT THE CONSUMER. A corpus reachable only by writing a
# JSON walker is a corpus most people copy values out of by hand, and a value
# copied by hand stops matching the day the build moves.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. every format regenerates from the canonical corpus;
#   2. TWO RUNS ARE BYTE-IDENTICAL. A generator that read a clock or a hash seed
#      would produce a diff on every run, and a published artefact that diffs on
#      every run is one nobody can tell a real change from;
#   3. the lossless formats round-trip to byte-identical canonical JSON, which
#      is the half a writer alone cannot prove;
#   4. the lossy ones carry the documented subset and say in their own header
#      what they leave out.
#
# ⛔ NEVER HAND-EDIT A GENERATED FORMAT. If one is ever edited directly the
# generator has lost, and this is what says so.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-formats.ps1
#   pwsh -NoProfile -File scripts/common/check-formats.ps1 -Json
#
# Exit codes: 0 every format round-trips, 1 one did not, 2 could not run.
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
    [Console]::Error.WriteLine('check-formats: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-formats: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-formats: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-formats: cargo not found')
    exit 2
}

# ⛔ 2, not 1. A tree with no corpus has verified nothing about the generator.
if (-not (Test-Path -LiteralPath (Join-Path $root 'corpus'))) {
    [Console]::Error.WriteLine('check-formats: there is no corpus under ' + $root + ', so there is nothing to generate')
    exit 2
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-formats-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -Recurse -Force -LiteralPath $out }
New-Item -ItemType Directory -Force -Path (Join-Path $out 'a') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $out 'b') | Out-Null

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-formats: the corpus crate did not build')
    exit 2
}
$bin = Join-Path $root 'target' | Join-Path -ChildPath 'debug' | Join-Path -ChildPath 'b-ids-corpus'
if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine('check-formats: ' + $bin + ' is not there')
    exit 2
}

$problems = @()

# -- 1 and 2: generate twice, and compare the bytes --------------------------
$logA = Join-Path $out 'a.log'
$logB = Join-Path $out 'b.log'
& $bin formats --root $root --out (Join-Path $out 'a') > $logA 2>&1
$rcA = $LASTEXITCODE
& $bin formats --root $root --out (Join-Path $out 'b') > $logB 2>&1
$rcB = $LASTEXITCODE
if ($rcA -ne 0 -or $rcB -ne 0) {
    [Console]::Error.WriteLine('check-formats: the generator exited ' + $rcA + ' then ' + $rcB)
    Get-Content -LiteralPath $logA | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}

# ⛔ THE FIXED LAST LINE, never the prose above it.
$status = (Get-Content -LiteralPath $logA | Where-Object { $_ -like 'corpus=formats *' } | Select-Object -Last 1)
if (-not $status) {
    [Console]::Error.WriteLine('check-formats: the generator printed no status line')
    exit 1
}
$files = [regex]::Match($status, 'files:(\d+)').Groups[1].Value
$profiles = [regex]::Match($status, 'profiles:(\d+)').Groups[1].Value

foreach ($file in (Get-ChildItem -LiteralPath (Join-Path $out 'a') -File)) {
    $other = Join-Path $out 'b' | Join-Path -ChildPath $file.Name
    $left = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    $right = (Get-FileHash -LiteralPath $other -Algorithm SHA256).Hash
    if ($left -ne $right) {
        $problems += ('  ' + $file.Name + ': two runs of the generator differ, so it is not deterministic')
    }
}

# -- 3 and 4: the round trips, which are the suite's ------------------------
#
# ⛔ THE READERS ARE THE CRATE'S AND SO ARE THE ASSERTIONS. A round trip written
# here would be a second reader of five formats, disagreeing with the one the
# crate publishes the first time either moved.
& cargo test -q -p b-ids-corpus --test formats > (Join-Path $out 'tests.log') 2>&1
if ($LASTEXITCODE -ne 0) {
    $problems += '  the round-trip suite failed. Its output is in .tmp/check-formats-ps/tests.log'
}

foreach ($want in @('corpus.json', 'corpus.ndjson', 'corpus.csv', 'corpus.tsv', 'corpus.md')) {
    $path = Join-Path $out 'a' | Join-Path -ChildPath $want
    if (-not (Test-Path -LiteralPath $path) -or (Get-Item -LiteralPath $path).Length -eq 0) {
        $problems += ('  ' + $want + ' was not generated, or is empty')
    }
}
$md = Join-Path $out 'a' | Join-Path -ChildPath 'corpus.md'
if ((Test-Path -LiteralPath $md) -and -not (Select-String -LiteralPath $md -SimpleMatch 'Do not edit' -Quiet)) {
    $problems += '  corpus.md does not say in its own header that it is generated'
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-formats/1","files":' + $files + ',"profiles":' + $profiles + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output ('formats ok: ' + $files + ' file(s) from ' + $profiles + ' profile(s), byte-identical over two runs,')
    Write-Output '  every lossless format round-trips to canonical JSON and every lossy one'
    Write-Output '  carries the documented subset.'
    exit 0
}

[Console]::Error.WriteLine('formats check failed, ' + $count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('One generator, canonical JSON in, every format out. Never hand-edit a')
[Console]::Error.WriteLine('generated file. TODO/schema.md, SCHEMA-08.')
exit 1
