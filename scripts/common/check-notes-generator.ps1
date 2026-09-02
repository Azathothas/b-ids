# check-notes-generator.ps1 - do the release body and the changelog entry come
# out of one generator, and do they agree fact for fact?
#
# ⭐ THE TWIN OF check-notes-generator.sh. TODO/publish.md, PUB-08, and
# TODO/driver.md, DRIVER-09, is why a script in this directory does not land
# without one.
#
# ⛔ RELEASE NOTES AND A CHANGELOG WRITTEN SEPARATELY DRIFT, and the reader who
# trusts the wrong one is the one who was doing something careful.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. over one corpus change, the two outputs carry every fact the model holds;
#   2. two runs over that change produce identical text;
#   3. a NO-OP change produces nothing at all;
#   4. ⛔ AND THE COMPARISON CAN FAIL. A fixture whose two outputs come from
#      DIFFERENT inputs is asserted NOT to agree.
#
# ⚠ THE ASSERTIONS ARE THE CRATE'S. This runs that suite and reads its exit code.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-notes-generator.ps1
#   pwsh -NoProfile -File scripts/common/check-notes-generator.ps1 -Json
#
# Exit codes: 0 they agree, 1 they do not, 2 could not run.
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
    [Console]::Error.WriteLine('check-notes-generator: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-notes-generator: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-notes-generator: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-notes-generator: cargo not found')
    exit 2
}

$suite = Join-Path $root 'crates' | Join-Path -ChildPath 'b-ids-corpus' | Join-Path -ChildPath 'tests' | Join-Path -ChildPath 'notes.rs'
if (-not (Test-Path -LiteralPath $suite)) {
    [Console]::Error.WriteLine('check-notes-generator: no suite at ' + $suite)
    exit 2
}

# ⛔ THE FOUR ASSERTIONS ARE NAMED HERE AND ASSERTED THERE, so a suite that lost
# one is caught by this check rather than by nobody.
$want = @(
    'notes_the_two_outputs_agree_fact_for_fact',
    'notes_a_no_op_change_renders_nothing_at_all',
    'notes_two_runs_over_one_change_produce_identical_text',
    'notes_two_outputs_generated_from_different_inputs_do_not_agree'
)
$text = Get-Content -LiteralPath $suite -Raw
$problems = @()
foreach ($name in $want) {
    if (-not $text.Contains('fn ' + $name)) {
        $problems += ('  ' + $name + ' is not in the suite')
    }
}

$log = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-notes-generator-ps.log'
& cargo test -q -p b-ids-corpus --test notes > $log 2>&1
$rc = $LASTEXITCODE
$cases = 0
$line = (Get-Content -LiteralPath $log | Where-Object { $_ -match '^running (\d+) tests' } | Select-Object -First 1)
if ($line) { $cases = [int][regex]::Match($line, '^running (\d+) tests').Groups[1].Value }
if ($rc -ne 0) {
    $problems += '  the suite failed. Its output is in .tmp/check-notes-generator-ps.log'
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-notes-generator/1","cases":' + $cases + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output ('notes generator ok: ' + $cases + ' case(s). The release body and the changelog entry')
    Write-Output '  are rendered from one model, they carry every fact it holds, a no-op'
    Write-Output '  change renders nothing, and two outputs from different inputs are'
    Write-Output '  asserted NOT to agree.'
    exit 0
}

[Console]::Error.WriteLine('notes generator check failed, ' + $count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('One generator, two outputs, so the two cannot disagree by construction')
[Console]::Error.WriteLine('rather than by discipline. TODO/publish.md, PUB-08.')
exit 1
