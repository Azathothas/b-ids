# derive-ja4-vector.ps1 - derive one JA4 test vector from a published profile,
# with tools that are NOT this project's code.
#
# The twin of scripts/common/derive-ja4-vector.sh. Read that file's header for
# the defect this exists to catch and for why automating the derivation does not
# weaken the vector: the arithmetic is still jq's and a general-purpose SHA-256
# rather than b_ids_harness::sha256, and the test compares this answer against
# the Rust one.
#
# ⛔ THE DIGEST IS NOT THIS PROJECT'S. The sh half uses sha256sum and this half
# uses the platform's own SHA-256. Both are third-party arithmetic, which is the
# property VALID-04 asks for.
#
# ⚠ ONE MODE WRITES AND IT SAYS SO: -Fill ROOT rewrites that root's
# vectors/ja4/v1.json with the vectors that were MISSING from it. ⛔ It never
# edits a vector that is already there.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 PROFILE
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 -Json PROFILE
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 -Selftest
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 -Fill ROOT
#
# Exit codes: 0 derived, 1 the profile could not be read, 2 could not run.
#
# ⛔ Read $LASTEXITCODE from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [switch]$Selftest,
    # ⚠ -Fill TAKES ITS ROOT AS ITS OWN VALUE rather than reusing the
    # positional, because the positional is a PROFILE and a root is not one.
    [string]$Fill,
    # PSAvoidAssignmentToAutomaticVariable: $Profile is a PowerShell
    # automatic variable and assigning to it has side effects, which is the
    # same class of trap docs/conventions/shell.md section 8 records about
    # $args. The sh half calls the same thing PROFILE; only this half cannot.
    [Parameter(Position = 0)][string]$ProfilePath,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2
    # rather than 1. Without it PowerShell's own binder refuses the argument
    # before this script runs and answers 1, which is "it ran and the thing
    # failed". check-exit-codes caught exactly that on this file.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

$ErrorActionPreference = 'Stop'

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('derive-ja4-vector: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

function Get-Truncated {
    param([string]$Text)
    # ⛔ AN EMPTY LIST HASHES TO TWELVE ZEROS, not to the digest of an empty
    # string. That is the specification's rule and it is the half an
    # implementation gets wrong.
    if ([string]::IsNullOrEmpty($Text)) { return '000000000000' }
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Text))
    } finally {
        $sha.Dispose()
    }
    $hex = ($bytes | ForEach-Object { $_.ToString('x2') }) -join ''
    return $hex.Substring(0, 12)
}

if ($Selftest) {
    $empty = Get-Truncated ''
    $known = Get-Truncated '002f,0035'
    $problems = 0
    if ($empty -ne '000000000000') {
        [Console]::Error.WriteLine("derive-ja4-vector: an empty list hashed to $empty")
        $problems++
    }
    if ($known.Length -ne 12) {
        [Console]::Error.WriteLine("derive-ja4-vector: a digest was $($known.Length) characters")
        $problems++
    }
    if ($known -cnotmatch '^[0-9a-f]+$') {
        [Console]::Error.WriteLine('derive-ja4-vector: a digest was not lower-case hexadecimal')
        $problems++
    }
    if ($Json) {
        Write-Output ('{"schema":"derive-ja4-vector/1","selftest":true,"empty":"' + $empty +
                      '","width":' + $known.Length + ',"problems":' + $problems + '}')
    } else {
        Write-Output "derive-ja4-vector selftest: empty=$empty width=$($known.Length) problems=$problems"
    }
    if ($problems -gt 0) { exit 1 }
    exit 0
}

$jq = Get-Command jq -CommandType Application, ExternalScript -ErrorAction SilentlyContinue
if (-not $jq) {
    [Console]::Error.WriteLine('derive-ja4-vector: jq not found')
    exit 2
}
$top = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $top) {
    [Console]::Error.WriteLine('derive-ja4-vector: not a git repository')
    exit 2
}
$repoRoot = $top.Trim()
$derive = Join-Path $repoRoot 'scripts/fixtures/ja4-derive.jq'
if (-not (Test-Path -LiteralPath $derive)) {
    [Console]::Error.WriteLine("derive-ja4-vector: no $derive")
    exit 2
}

# ⭐ THE ONE DERIVATION, so the single-profile mode and -Fill cannot answer
# differently. It was inline until -Fill needed it in a loop.
function Get-Ja4 {
    param([string]$Path)
    $raw = & jq -r -f $derive $Path 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $raw) {
        [Console]::Error.WriteLine("derive-ja4-vector: $Path is not a profile this can read")
        return $null
    }
    $d = ($raw -join "`n") | ConvertFrom-Json
    $nc = [int]$d.ncipher
    $nx = [int]$d.next
    # ⚠ TWO DIGITS EACH, SATURATING AT 99, which is the specification's rule.
    if ($nc -gt 99) { $nc = 99 }
    if ($nx -gt 99) { $nx = 99 }
    $prefix = 't13{0}{1:d2}{2:d2}{3}' -f $d.sni, $nc, $nx, $d.alpn
    return @{
        ja4    = $prefix + '_' + (Get-Truncated $d.ciphers_sorted) + '_' + (Get-Truncated $d.extensions_sorted)
        ja4_r  = $prefix + '_' + $d.ciphers_sorted + '_' + $d.extensions_sorted
        ja4_ro = $prefix + '_' + $d.ciphers_original + '_' + $d.extensions_original
    }
}

# ⚠ THE RAW SIDECAR'S PATH IS DERIVED FROM THE PROFILE'S OWN, the way the
# publisher derives it, and RELATIVE TO THE BASE BEING ASKED ABOUT rather than
# to this repository: a merged tree under .tmp is a real base.
function Get-HelloPath {
    param([string]$Path, [string]$Base)
    $full = (Resolve-Path -LiteralPath $Path).Path.Replace('\', '/')
    $baseFull = (Resolve-Path -LiteralPath $Base).Path.Replace('\', '/')
    $rel = $full.Replace($baseFull + '/', '')
    return ($rel -replace '^corpus/', 'raw/') -replace '\.json$', '.hello.hex'
}

# -- ⭐ FILL: EVERY VECTOR THE TREE IS MISSING, AND NOT ONE IT ALREADY HAS ----
if ($Fill) {
    $corpusDir = Join-Path (Join-Path $Fill 'corpus') 'v1'
    if (-not (Test-Path -LiteralPath $corpusDir)) {
        [Console]::Error.WriteLine("derive-ja4-vector: $Fill holds no corpus/v1")
        exit 2
    }
    $vectors = Join-Path (Join-Path (Join-Path $Fill 'vectors') 'ja4') 'v1.json'
    if (-not (Test-Path -LiteralPath $vectors -PathType Leaf)) {
        # ⛔ REFUSED RATHER THAN CREATED. The file carries the specification
        # vectors and the provenance block as well as the captures.
        [Console]::Error.WriteLine("derive-ja4-vector: no $vectors to fill")
        exit 2
    }
    $file = Get-Content -LiteralPath $vectors -Raw | ConvertFrom-Json
    $presentIds = @($file.vectors | Where-Object { $_.kind -eq 'capture' } | ForEach-Object { $_.id })
    $derived = 0
    $present = 0
    $total = 0
    $added = @()
    # ⚠ SORTED, so two runs over one tree produce one file.
    $found = @(Get-ChildItem -LiteralPath $corpusDir -Recurse -File -Filter '*.json' |
        Where-Object { $_.Name -ne 'index.json' -and $_.Name -ne 'latest.json' } |
        Sort-Object -Property FullName -CaseSensitive)
    foreach ($entry in $found) {
        $total++
        $id = ((& jq -r '.id' $entry.FullName) -replace "`r", '')
        if ($presentIds -contains $id) {
            $present++
            continue
        }
        $one = Get-Ja4 -Path $entry.FullName
        if ($null -eq $one) { exit 1 }
        $added += [ordered]@{
            kind   = 'capture'
            id     = $id
            hello  = (Get-HelloPath -Path $entry.FullName -Base $Fill)
            ja4    = $one.ja4
            ja4_r  = $one.ja4_r
            ja4_ro = $one.ja4_ro
        }
        $derived++
        Write-Output "derived $id"
    }
    if ($derived -gt 0) {
        # ⛔ WRITTEN THROUGH jq, NOT ConvertTo-Json. The sh half writes this file
        # with jq's own two-space layout, and the two halves have to produce the
        # same bytes: this file is compared byte for byte by check-data-branch
        # once it is published.
        $tmp = Join-Path $Fill '.tmp-ja4-fill.json'
        ($added | ConvertTo-Json -Depth 4 -AsArray) | Set-Content -LiteralPath $tmp -Encoding utf8NoBOM
        $next = & jq --slurpfile add $tmp '.vectors += $add[0]' $vectors
        if ($LASTEXITCODE -ne 0) {
            Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
            [Console]::Error.WriteLine('derive-ja4-vector: the vector file could not be rewritten')
            exit 1
        }
        Set-Content -LiteralPath $vectors -Value (($next -join "`n") + "`n") -NoNewline -Encoding utf8NoBOM
        Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
    }
    if ($Json) {
        Write-Output ('{"schema":"derive-ja4-vector/2","fill":true,"profiles":' + $total +
                      ',"derived":' + $derived + ',"present":' + $present + '}')
    } else {
        Write-Output "derive-ja4-vector fill: $total profile(s), $derived vector(s) derived, $present already present."
        Write-Output '⛔ Nothing already in the file was rewritten.'
    }
    exit 0
}

if (-not $ProfilePath) {
    [Console]::Error.WriteLine('derive-ja4-vector: name a profile')
    exit 2
}
if (-not (Test-Path -LiteralPath $ProfilePath)) {
    [Console]::Error.WriteLine("derive-ja4-vector: no $ProfilePath")
    exit 1
}

$one = Get-Ja4 -Path $ProfilePath
if ($null -eq $one) { exit 1 }

if ($Json) {
    $id = (& jq -r '.id' $ProfilePath) -replace "`r", ''
    $out = [ordered]@{
        kind   = 'capture'
        id     = $id
        hello  = (Get-HelloPath -Path $ProfilePath -Base $repoRoot)
        ja4    = $one.ja4
        ja4_r  = $one.ja4_r
        ja4_ro = $one.ja4_ro
    }
    Write-Output (ConvertTo-Json $out -Depth 4)
} else {
    Write-Output "ja4    $($one.ja4)"
    Write-Output "ja4_r  $($one.ja4_r)"
    Write-Output "ja4_ro $($one.ja4_ro)"
}
exit 0
