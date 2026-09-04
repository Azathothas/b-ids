# check-pcap.ps1 - is every published packet capture the profile's own bytes, and
# does it say it was synthesised?
#
# ⭐ THE TWIN OF check-pcap.sh. Same schema, same exit codes, same legs.
# check-twins.sh is what stops the two drifting.
#
# ⛔ A SYNTHESISED CAPTURE THAT IS INDISTINGUISHABLE FROM A REAL ONE is the one
# thing TODO/publish.md, PUB-06, forbids.
#
# ⚠ THE DISSECTION LEG IS A SKIP WHERE THERE IS NO TOOL, and a skip is reported
# as a skip: this check does NOT claim a standard tool opened the file when none
# was there to try.
#
# ⚠ NO xxd HERE. A native PowerShell session has none, and it does not need one:
# the bytes are read with Get-Content -AsByteStream and hexed in-process, which
# is what scripts/README.md means by the job existing on both platforms even
# where the tool does not.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-pcap.ps1
#   pwsh -NoProfile -File scripts/common/check-pcap.ps1 -Json
#
# Exit codes: 0 every capture is what it should be, 1 one is not, 2 could not run.
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
    [Console]::Error.WriteLine('check-pcap: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-pcap: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-pcap: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-pcap: cargo not found')
    exit 2
}

$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-pcap: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
$env:B_IDS_CORPUS_ROOT = $corpusRoot

$suite = Join-Path $root 'crates/b-ids-corpus/tests/pcap.rs'
if (-not (Test-Path -LiteralPath $suite)) {
    [Console]::Error.WriteLine('check-pcap: no suite at ' + $suite)
    exit 2
}

# ⛔ THE CASES ARE NAMED HERE AND ASSERTED THERE.
$want = @(
    'pcap_the_client_hello_is_the_profiles_own_bytes',
    'pcap_every_block_declares_its_length_at_both_ends',
    'pcap_the_file_says_it_was_synthesised',
    'pcap_a_profile_with_no_raw_hello_produces_nothing',
    'pcap_the_header_checksums_are_computed_rather_than_zero'
)

$problems = @()
$suiteText = Get-Content -LiteralPath $suite -Raw
foreach ($w in $want) {
    if ($suiteText -notmatch [regex]::Escape("fn $w")) {
        $problems += "$w is not in the suite"
    }
}

$out = Join-Path (Join-Path $root '.tmp') 'check-pcap-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Recurse -Force }
New-Item -ItemType Directory -Path $out -Force | Out-Null

$testsLog = Join-Path $out 'tests.log'
& cargo test -q -p b-ids-corpus --test pcap *> $testsLog
$rcT = $LASTEXITCODE
$cases = 0
foreach ($line in (Get-Content -LiteralPath $testsLog -ErrorAction SilentlyContinue)) {
    if ($line -match '^running (\d+) tests') { $cases = [int]$Matches[1]; break }
}
if ($rcT -ne 0) { $problems += 'the suite failed. Its output is in .tmp/check-pcap-ps/tests.log' }
if ($cases -lt $want.Count) {
    $problems += "the suite ran $cases case(s) where at least $($want.Count) were expected"
}

$tree = Join-Path $out 'tree'
$publishLog = Join-Path $out 'publish.log'
& cargo run -q -p b-ids-corpus -- publish --root $corpusRoot --out $tree *> $publishLog
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-pcap: the assembler did not build the tree')
    Get-Content -LiteralPath $publishLog | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 2
}

$rawDir = Join-Path (Join-Path $corpusRoot 'raw') 'v1'
$sidecars = @(Get-ChildItem -LiteralPath $rawDir -Recurse -File -Filter '*.hello.hex' -ErrorAction SilentlyContinue).Count
$pcapDir = Join-Path $tree 'pcap'
$captures = @(Get-ChildItem -LiteralPath $pcapDir -Recurse -File -Filter '*.pcapng' -ErrorAction SilentlyContinue)
$files = $captures.Count
if ($sidecars -ne $files) {
    $problems += "$sidecars raw hello(s) produced $files capture(s), and it is one each"
}
if ($files -lt 1) {
    $problems += 'no capture was written at all, so nothing below checked anything'
}

$marker = 'SYNTHESISED BY b-ids'
$checked = 0
foreach ($f in ($captures | Sort-Object -Property FullName -CaseSensitive)) {
    $rel = $f.FullName.Substring((Join-Path $pcapDir 'v1').Length + 1).Replace('\', '/')
    $route = Split-Path -Parent $rel
    $route = "$route".Replace('\', '/')
    $version = [System.IO.Path]::GetFileNameWithoutExtension($rel)
    $sidecar = Join-Path (Join-Path $rawDir $route) "$version.hello.hex"
    if (-not (Test-Path -LiteralPath $sidecar)) {
        $problems += "$rel has no sidecar at raw/v1/$route/$version.hello.hex to compare against"
        continue
    }

    # ⭐ THE INDEPENDENT COMPARISON. The file is hexed and the corpus's own
    # recorded hex has to appear in it as a contiguous run.
    $bytes = [System.IO.File]::ReadAllBytes($f.FullName)
    $dump = [System.BitConverter]::ToString($bytes).Replace('-', '').ToLowerInvariant()
    $wantHex = ((Get-Content -LiteralPath $sidecar -Raw) -replace '\s', '').ToLowerInvariant()
    if (-not $dump.Contains($wantHex)) {
        $problems += "$rel does not carry the ClientHello raw/v1/$route/$version.hello.hex records"
    }

    $text = [System.Text.Encoding]::ASCII.GetString($bytes)
    if (-not $text.Contains($marker)) {
        $problems += "$rel does not say it was synthesised"
    }

    # ⚠ THE FORMAT'S OWN TWO MAGIC NUMBERS, at the two places the format puts
    # them.
    $head = $dump.Substring(0, [Math]::Min(24, $dump.Length))
    if (-not ($head.StartsWith('0a0d0d0a') -and $head.EndsWith('4d3c2b1a'))) {
        $problems += "$rel does not open with a pcapng section header: $head"
    }
    $checked++
}

# ⚠ THE DISSECTION LEG, WHICH IS A SKIP RATHER THAN A PASS WHERE THERE IS NO TOOL.
$dissected = 'skipped'
$tshark = Get-Command tshark -CommandType Application -ErrorAction SilentlyContinue
if ($tshark -and $files -ge 1) {
    $one = ($captures | Sort-Object -Property FullName -CaseSensitive | Select-Object -First 1).FullName
    & $tshark.Source -r $one -T fields -e frame.number *> (Join-Path $out 'tshark.log')
    if ($LASTEXITCODE -eq 0) {
        $dissected = 'ok'
    }
    else {
        $dissected = 'failed'
        $problems += "tshark could not read $one. Its output is in .tmp/check-pcap-ps/tshark.log"
    }
}

Remove-Item Env:B_IDS_CORPUS_ROOT -ErrorAction SilentlyContinue

if ($Json) {
    Write-Output ('{"schema":"check-pcap/1","captures":' + $files +
                  ',"sidecars":' + $sidecars +
                  ',"checked":' + $checked +
                  ',"cases":' + $cases +
                  ',"dissected":"' + $dissected +
                  '","problems":' + $problems.Count + '}')
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -eq 0) {
    Write-Output "pcap ok: $files capture(s) over $sidecars raw hello(s), every one carrying the"
    Write-Output '  ClientHello its profile recorded, byte for byte, and every one saying'
    Write-Output "  it was synthesised. $cases suite case(s)."
    if ($dissected -eq 'ok') {
        Write-Output '  ⭐ A standard tool read it: tshark opened the first one.'
    }
    else {
        Write-Output '  ⚠ SKIP the dissection leg: no tshark on this host, so nothing here'
        Write-Output '  says a standard tool can open the file. Install tshark and it runs.'
    }
    exit 0
}

[Console]::Error.WriteLine("pcap check failed, $($problems.Count) problem(s):")
[Console]::Error.WriteLine('')
foreach ($p in $problems) { [Console]::Error.WriteLine('  ' + $p) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A synthesised capture that is indistinguishable from a real one is the')
[Console]::Error.WriteLine('one thing this entry forbids. TODO/publish.md, PUB-06.')
exit 1
