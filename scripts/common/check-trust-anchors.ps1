# check-trust-anchors.ps1 - does every profile carrying the trust-anchor
# extension have a published list with a capture date, and does the
# recommendation state all three options?
#
# ⭐ THE TWIN OF check-trust-anchors.sh. TODO/corpus.md, CORPUS-04, and
# TODO/driver.md, DRIVER-09, is why a script in this directory does not land
# without one.
#
# ⛔ ONE EXTENSION CARRIES A SNAPSHOT OF THE BROWSER'S OWN ROOT STORE, so a
# client copying one build's list is advertising which build it copied. It
# changes on a different schedule from everything else a profile carries, which
# is why it is published beside the corpus.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. every profile that CARRIES the extension has a published list;
#   2. every published list names its capture instant and at least one
#      identifier, because a list with no date is a list nobody can place;
#   3. the recommendation states all THREE options, each with its cost, and
#      asserts no preference;
#   4. ⚠ AND IT REFUSES A VACUOUS PASS. A corpus in which no profile carries the
#      extension would satisfy rule 1 by having nothing to check.
#
# ⚠ THE NAME OF THE EXTENSION IS INFERRED AND THIS DOES NOT SETTLE IT.
# docs/inherited-claims.md section 3 carries that split.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-trust-anchors.ps1
#   pwsh -NoProfile -File scripts/common/check-trust-anchors.ps1 -Json
#
# Exit codes: 0 every carrier is published, 1 one is not, 2 could not run.
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
    [Console]::Error.WriteLine('check-trust-anchors: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-trust-anchors: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-trust-anchors: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-trust-anchors: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-trust-anchors: cargo not found')
    exit 2
}

$doc = Join-Path $root 'docs' | Join-Path -ChildPath 'trust-anchors.md'
if (-not (Test-Path -LiteralPath $doc)) {
    [Console]::Error.WriteLine('check-trust-anchors: no recommendation at ' + $doc)
    exit 2
}
if (-not (Test-Path -LiteralPath (Join-Path $corpusRoot 'corpus'))) {
    [Console]::Error.WriteLine('check-trust-anchors: there is no corpus, so no list can be published')
    exit 2
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-trust-anchors-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -Recurse -Force -LiteralPath $out }
New-Item -ItemType Directory -Force -Path $out | Out-Null

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-trust-anchors: the corpus crate did not build')
    exit 2
}
$bin = Join-Path $root 'target' | Join-Path -ChildPath 'debug' | Join-Path -ChildPath 'b-ids-corpus'
if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine('check-trust-anchors: ' + $bin + ' is not there')
    exit 2
}

$log = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-trust-anchors-ps.log'
& $bin anchors --root $corpusRoot --out $out > $log 2>&1
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-trust-anchors: publishing the lists exited ' + $LASTEXITCODE)
    Get-Content -LiteralPath $log | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}
$status = (Get-Content -LiteralPath $log | Where-Object { $_ -like 'corpus=anchors *' } | Select-Object -Last 1)
$lists = [int][regex]::Match($status, 'lists:(\d+)').Groups[1].Value
$profiles = [int][regex]::Match($status, 'profiles:(\d+)').Groups[1].Value

# 1. ⛔ HOW MANY PROFILES CARRY IT, counted from the corpus rather than from the
# publisher's own answer. A publisher that skipped a carrier would otherwise
# agree with itself.
$carriers = 0
foreach ($file in (Get-ChildItem -LiteralPath (Join-Path $corpusRoot 'corpus') -Recurse -Filter '*.json')) {
    if ($file.Name -eq 'index.json' -or $file.Name -eq 'latest.json') { continue }
    # ⚠ NOT $profile: it is an automatic variable in PowerShell, and assigning
    # to it is the same trap docs/conventions/shell.md names for $args.
    $candidate = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
    if (@($candidate.tls.extensions | Where-Object { $_.codepoint -eq 51764 }).Count -gt 0) {
        $carriers = $carriers + 1
    }
}

$problems = @()

# 4. ⚠ THE VACUOUS PASS, REFUSED FIRST.
if ($carriers -eq 0) {
    [Console]::Error.WriteLine('check-trust-anchors: no profile in this corpus carries codepoint 0xca34, so')
    [Console]::Error.WriteLine('  there is nothing to publish and nothing this check can verify. That is a')
    [Console]::Error.WriteLine('  fact about the builds captured, not a pass. TODO/corpus.md, CORPUS-04.')
    exit 2
}

if ($lists -ne $carriers) {
    $problems += ('  ' + $carriers + ' profile(s) carry the extension and ' + $lists + ' list(s) were published')
}

# 2. every published list names its date and at least one identifier
foreach ($file in (Get-ChildItem -LiteralPath $out -Filter '*.json')) {
    $list = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
    if (-not $list.captured_at) {
        $problems += ('  ' + $file.Name + ': no capture instant, so nobody can place this list in time')
    }
    if (@($list.identifiers).Count -eq 0) {
        $problems += ('  ' + $file.Name + ': no identifiers')
    }
}

# 3. ⛔ ALL THREE OPTIONS, each with its cost.
$text = Get-Content -LiteralPath $doc -Raw
foreach ($phrase in @('Omit the extension', 'Carry a captured list', 'Send it empty')) {
    if (-not $text.Contains($phrase)) {
        $problems += ("  the recommendation does not state the option '" + $phrase + "'")
    }
}
if (-not $text.Contains('asserts no preference')) {
    $problems += '  the recommendation does not say that it asserts no preference'
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-trust-anchors/1","carriers":' + $carriers + ',"lists":' + $lists + ',"profiles":' + $profiles + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output ('trust anchors ok: ' + $carriers + ' of ' + $profiles + ' profile(s) carry codepoint 0xca34, and every one has a')
    Write-Output '  published list with its capture instant. The recommendation states all three'
    Write-Output '  options and asserts no preference.'
    exit 0
}

[Console]::Error.WriteLine('trust-anchor check failed, ' + $count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('The list is a snapshot of a root store and it changes per build.')
[Console]::Error.WriteLine('docs/trust-anchors.md is the recommendation. TODO/corpus.md, CORPUS-04.')
exit 1
