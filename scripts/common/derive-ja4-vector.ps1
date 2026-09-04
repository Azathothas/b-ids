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
# Usage:
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 PROFILE
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 -Json PROFILE
#   pwsh -NoProfile -File scripts/common/derive-ja4-vector.ps1 -Selftest
#
# Exit codes: 0 derived, 1 the profile could not be read, 2 could not run.
#
# ⛔ Read $LASTEXITCODE from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [switch]$Selftest,
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
if (-not $ProfilePath) {
    [Console]::Error.WriteLine('derive-ja4-vector: name a profile')
    exit 2
}
if (-not (Test-Path -LiteralPath $ProfilePath)) {
    [Console]::Error.WriteLine("derive-ja4-vector: no $ProfilePath")
    exit 1
}

$raw = & jq -r -f $derive $ProfilePath 2>$null
if ($LASTEXITCODE -ne 0 -or -not $raw) {
    [Console]::Error.WriteLine("derive-ja4-vector: $ProfilePath is not a profile this can read")
    exit 1
}
$d = ($raw -join "`n") | ConvertFrom-Json

$nc = [int]$d.ncipher
$nx = [int]$d.next
# ⚠ TWO DIGITS EACH, SATURATING AT 99, which is the specification's rule.
if ($nc -gt 99) { $nc = 99 }
if ($nx -gt 99) { $nx = 99 }
$prefix = 't13{0}{1:d2}{2:d2}{3}' -f $d.sni, $nc, $nx, $d.alpn

$ja4 = $prefix + '_' + (Get-Truncated $d.ciphers_sorted) + '_' + (Get-Truncated $d.extensions_sorted)
$ja4r = $prefix + '_' + $d.ciphers_sorted + '_' + $d.extensions_sorted
$ja4ro = $prefix + '_' + $d.ciphers_original + '_' + $d.extensions_original

if ($Json) {
    $id = (& jq -r '.id' $ProfilePath) -replace "`r", ''
    # ⚠ The raw sidecar's path is DERIVED from the profile's own path, the way
    # the publisher derives it, rather than named separately.
    $rel = (Resolve-Path -LiteralPath $ProfilePath).Path.Replace('\', '/')
    $rel = $rel.Replace($repoRoot.Replace('\', '/') + '/', '')
    $hello = ($rel -replace '^corpus/', 'raw/') -replace '\.json$', '.hello.hex'
    $out = [ordered]@{
        kind   = 'capture'
        id     = $id
        hello  = $hello
        ja4    = $ja4
        ja4_r  = $ja4r
        ja4_ro = $ja4ro
    }
    Write-Output (ConvertTo-Json $out -Depth 4)
} else {
    Write-Output "ja4    $ja4"
    Write-Output "ja4_r  $ja4r"
    Write-Output "ja4_ro $ja4ro"
}
exit 0
