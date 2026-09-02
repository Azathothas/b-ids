# check-staleness.ps1 - is the corpus behind the build the vendor is serving,
# and what would replace it?
#
# ⭐ THE TWIN OF check-staleness.sh. TODO/ci.md, CI-02.
#
# ⛔ A BROWSER SHIPPING A NEW VERSION IS NOT A DEFECT IN A COMMIT. Asserting
# current versions on push makes every unrelated change fail on the day a
# browser ships, so this runs on a SCHEDULE and never on a push.
#
# -- ⭐ WHEN IT GOES RED ITS OUTPUT CARRIES THE REPLACEMENT VALUES -----------
#
# A check that only says a fingerprint changed is half a tool. Every stale row
# names the route that is behind, the build it holds, the build the vendor is
# serving, that build's rollout fraction, and every source that answered.
#
# -- ⛔ ONE SOURCE BEING UNREACHABLE IS NOT A FAILURE ------------------------
#
# `b-ids-driver versions` fetches each source separately and reports which
# answered. ⚠ This script reads that report and never fetches anything itself.
#
# -- ⛔ THE ORDERING IS NUMERIC PER COMPONENT -------------------------------
#
# `151.0.7922.9` is BEHIND `151.0.7922.76` and a string comparison says the
# opposite. ⚠ Both halves implement the comparison themselves rather than
# sharing a binary, so this pair is a genuine two-implementation comparison.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-staleness.ps1
#   pwsh -NoProfile -File scripts/common/check-staleness.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-staleness.ps1 -Versions FILE
#
# Exit codes: 0 the corpus holds the serving build, 1 it is behind,
#             2 it could not run.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [string]$Corpus = '',
    [string]$Versions = '',
    [string]$Channel = 'stable',
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. TODO/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-staleness: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-staleness: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-staleness: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root

if (-not $Corpus) { $Corpus = 'corpus/v1' }
$pointerPath = Join-Path $Corpus 'latest.json'
if (-not (Test-Path -LiteralPath $pointerPath)) {
    [Console]::Error.WriteLine('check-staleness: no pointer file at ' + $pointerPath)
    exit 2
}

# ⛔ READ FROM `b-ids-driver versions`, never fetched here. ⚠ Without -Versions
# this reaches the NETWORK, which is why this check is not in the gate.
if ($Versions) {
    if (-not (Test-Path -LiteralPath $Versions)) {
        [Console]::Error.WriteLine('check-staleness: no file at ' + $Versions)
        exit 2
    }
    $reportPath = $Versions
} else {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        [Console]::Error.WriteLine('check-staleness: cargo not found')
        exit 2
    }
    $reportDir = Join-Path $root '.tmp/check-staleness'
    $null = New-Item -ItemType Directory -Force -Path $reportDir
    $reportPath = Join-Path $reportDir 'report.json'
    $payload = & cargo run -q -p b-ids-driver -- versions --channel $Channel --json 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $payload) {
        [Console]::Error.WriteLine('check-staleness: no source answered, so nothing could be compared')
        exit 2
    }
    Set-Content -LiteralPath $reportPath -Value $payload -Encoding utf8
}

$pointer = Get-Content -LiteralPath $pointerPath -Raw | ConvertFrom-Json
$report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json

$chosen = $report.chosen
if (-not $chosen -or -not $chosen.version) {
    [Console]::Error.WriteLine('check-staleness: the report names no serving build')
    exit 2
}

$answered = @()
$silent = @()
foreach ($answer in $report.answers) {
    if ($answer.version) {
        $answered += ($answer.source + '=' + $answer.version)
    } else {
        $why = if ($answer.error) { $answer.error } else { 'no answer' }
        $silent += ($answer.source + '=' + $why)
    }
}

# ⛔ Numeric per component. A build string is dot-separated numbers and a
# lexical comparison puts 9 after 76.
function Compare-Build {
    param([string]$Left, [string]$Right)
    $l = @($Left -split '\.' | ForEach-Object { [int]$_ })
    $r = @($Right -split '\.' | ForEach-Object { [int]$_ })
    $len = [math]::Max($l.Count, $r.Count)
    for ($i = 0; $i -lt $len; $i++) {
        $a = if ($i -lt $l.Count) { $l[$i] } else { 0 }
        $b = if ($i -lt $r.Count) { $r[$i] } else { 0 }
        if ($a -ne $b) { if ($a -lt $b) { return -1 } else { return 1 } }
    }
    return 0
}

$rows = @()
foreach ($property in $pointer.per_channel.PSObject.Properties) {
    $key = $property.Name
    $parts = $key -split '/'
    # ⚠ ONLY THE FAMILY THE REPORT IS ABOUT. Comparing a Firefox route against a
    # Chrome answer would report every non-Chrome route as behind forever.
    if ($parts[0] -ne 'chrome') { continue }
    if ($parts[1] -ne $Channel) { continue }
    $match = [regex]::Match($property.Value, '([^/]+)\.json$')
    if (-not $match.Success) { continue }
    $held = $match.Groups[1].Value
    $rows += [pscustomobject]@{
        Route  = $key
        Held   = $held
        Behind = (Compare-Build -Left $held -Right $chosen.version) -lt 0
    }
}

$stale = @($rows | Where-Object { $_.Behind })

if ($Json) {
    Write-Output ('{"schema":"check-staleness/1","routes":' + $rows.Count +
        ',"stale":' + $stale.Count +
        ',"serving":"' + $chosen.version + '"' +
        ',"fraction":' + $chosen.fraction +
        ',"highest_known":"' + $chosen.highest_known + '"' +
        ',"highest_fraction":' + $chosen.highest_fraction +
        ',"answered":' + $answered.Count +
        ',"silent":' + $silent.Count + '}')
    # ⛔ THE SAME EXIT CODE AS THE HUMAN FORM. A -Json run that reported stale:2
    # and exited 0 would be the "step that exits 0 having done nothing it was
    # asked to do" row of docs/conventions/forbidden-patterns.md.
    if ($stale.Count -gt 0) { exit 1 }
    exit 0
}

if ($rows.Count -eq 0) {
    [Console]::Error.WriteLine('check-staleness: the pointer names no route for this vendor and channel')
    exit 2
}

if ($stale.Count -eq 0) {
    Write-Output ('staleness ok: ' + $rows.Count + ' route(s) hold ' + $chosen.version +
        ', which is what is serving at fraction ' + $chosen.fraction)
    exit 0
}

[Console]::Error.WriteLine('staleness: ' + $stale.Count + ' of ' + $rows.Count + ' route(s) are behind')
[Console]::Error.WriteLine('')
foreach ($row in $stale) {
    [Console]::Error.WriteLine('  ' + $row.Route)
    [Console]::Error.WriteLine('    holds    ' + $row.Held)
    [Console]::Error.WriteLine('    serving  ' + $chosen.version + ' at fraction ' + $chosen.fraction)
    [Console]::Error.WriteLine('    highest  ' + $chosen.highest_known + ' at fraction ' + $chosen.highest_fraction)
}
$answeredText = if ($answered.Count) { $answered -join ', ' } else { 'none' }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('  sources that answered: ' + $answeredText)
if ($silent.Count) {
    [Console]::Error.WriteLine('  sources that did not:  ' + ($silent -join ', '))
}
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('  the replacement is a CAPTURE of ' + $chosen.version + ', not an edit: the corpus')
[Console]::Error.WriteLine('  is append-only and a correction is a new profile. TODO/ci.md, CI-02.')
exit 1
