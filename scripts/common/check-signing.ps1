# check-signing.ps1 - can a consumer tell a capture from an assertion without
# trusting a file that travelled with the artefact?
#
# ⭐ THE TWIN OF check-signing.sh. Same schema, same exit codes, same legs.
# check-twins.sh is what stops the two drifting.
#
# ⛔ A CHECKSUMS FILE PUBLISHED IN THE SAME RELEASE AS THE ARTEFACT PROVES
# TRANSPORT, NOT AUTHORSHIP. docs/history/todo/publish.md, PUB-09.
#
# ⭐ THE ANSWER IS KEYLESS: the runner's own OIDC identity signs, so no
# long-lived key exists and no workflow names a secret.
#
# ⚠ THE LIVE LEG IS A SKIP AND SAYS WHY. Verifying needs a release to verify,
# and a pushed tag is the only thing that cuts one.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-signing.ps1
#   pwsh -NoProfile -File scripts/common/check-signing.ps1 -Json
#
# Exit codes: 0 the surface is what PUB-09 asks for, 1 it is not, 2 could not run.
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
    [Console]::Error.WriteLine('check-signing: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-signing: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-signing: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root

$wf = '.github/workflows/publish.yml'
if (-not (Test-Path -LiteralPath $wf)) {
    [Console]::Error.WriteLine("check-signing: no $wf")
    exit 2
}

# ⭐ THE ONE PLACE THE VERIFICATION COMMAND IS WRITTEN.
$verifyCommand = 'gh attestation verify'

$problems = @()
$lines = @(Get-Content -LiteralPath $wf)

# -- 1: the two writes, on the job ------------------------------------------
$jobLines = @()
$inside = $false
foreach ($line in $lines) {
    if ($line -match '^  release:') { $inside = $true }
    elseif ($inside -and $line -match '^  [a-z-]+:') { $inside = $false }
    if ($inside) { $jobLines += $line }
}
# ⛔ THE COMMENTS ARE STRIPPED, WHICH THE MUTATION PASS FOUND. With
# `id-token: write` REMOVED from the release job, both halves of this check
# reported `signing ok`: the job's own comment explains what that permission is
# for and spells it exactly, so the search matched the PROSE. ⭐ This leg had
# never been seen to refuse until it was made to.
$jobText = ($jobLines | Where-Object { $_ -notmatch '^\s*#' }) -join "`n"
foreach ($want in @('id-token: write', 'attestations: write')) {
    if (-not $jobText.Contains($want)) {
        $problems += "the release job does not declare $want, and keyless attestation needs it"
    }
}
$topLines = @()
$inside = $false
foreach ($line in $lines) {
    if ($line -match '^permissions:\s*$') { $inside = $true; continue }
    if ($inside -and $line -match '^[a-zA-Z]') { $inside = $false }
    if ($inside) { $topLines += $line }
}
$topText = ($topLines | Where-Object { $_ -notmatch '^\s*#' }) -join "`n"
foreach ($forbidden in @('id-token', 'attestations')) {
    if ($topText.Contains($forbidden)) {
        $problems += "$forbidden is granted at the top of the file, and it belongs to one job"
    }
}

# -- 2: no key, no secret ----------------------------------------------------
if (($lines -join "`n") -match 'secrets\.') {
    $problems += "$wf names a secret, and keyless attestation needs none"
}
$tracked = @(& git ls-files)
$keys = @($tracked | Where-Object {
        $_ -match '\.(pem|key|p12|pfx|jks|gpg|asc)$' -or $_ -match '(^|/)id_(rsa|ed25519|ecdsa)$'
    })
if ($keys.Count -gt 0) {
    $problems += 'the tree carries key-shaped file(s): ' + ($keys -join ' ')
}

# -- 3: pinned to a commit ---------------------------------------------------
$pin = 'none'
foreach ($line in $lines) {
    if ($line -match 'attest-build-provenance@([0-9a-f]+)') { $pin = $Matches[1]; break }
}
if ($pin.Length -ne 40) {
    $problems += "the attestation action is not pinned to a 40-character commit: $pin"
}

# -- 4 and 5: it attests, before it releases, and over the archive -----------
#
# ⚠ COMMENT LINES ARE EXCLUDED. publish.yml explains the release step in a
# comment above the step, and a search that matched prose reported the release
# happening before the attestation. The sh half found that on its first run.
function Find-Uncommented {
    param([string]$Needle)
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*#') { continue }
        if ($lines[$i].Contains($Needle)) { return $i + 1 }
    }
    return 0
}
$attestAt = Find-Uncommented -Needle 'uses: actions/attest-build-provenance@'
$releaseAt = Find-Uncommented -Needle 'gh release create'
if ($attestAt -eq 0) {
    $problems += "nothing in $wf attests anything"
}
elseif ($releaseAt -eq 0) {
    $problems += "nothing in $wf creates a release, so the ordering could not be checked"
}
elseif ($attestAt -ge $releaseAt) {
    $problems += "the attestation is at line $attestAt and the release is created at line $releaseAt, so a release would exist before anything attested it"
}

$subjects = @()
$inside = $false
foreach ($line in $lines) {
    if ($line -match 'uses: actions/attest-build-provenance@') { $inside = $true }
    elseif ($inside -and $line -match '^      - name:' -and $line -notmatch 'attest') { $inside = $false }
    if ($inside) { $subjects += $line }
}
$subjectText = $subjects -join "`n"
if (-not $subjectText.Contains('tar.gz')) {
    $problems += 'the attestation does not name the release archive, so a consumer could verify the list and not the thing it describes'
}
if (-not $subjectText.Contains('SHA256SUMS')) {
    $problems += 'the attestation does not name SHA256SUMS'
}

# -- 6: the published command is this one ------------------------------------
$published = @(Get-ChildItem -Path 'README.md', 'docs', 'TODO' -Recurse -File -Filter '*.md' -ErrorAction SilentlyContinue |
        Where-Object { (Get-Content -LiteralPath $_.FullName -Raw) -match [regex]::Escape($verifyCommand) })
if ($published.Count -eq 0) {
    $problems += "no document publishes '$verifyCommand', so a consumer is not told how to verify"
}

# ⚠ THE LIVE LEG.
$verified = 'skipped'
$releases = 0
if (Get-Command gh -CommandType Application -ErrorAction SilentlyContinue) {
    $listed = (& gh release list --limit 1 --json tagName --jq 'length' 2>$null)
    if ($LASTEXITCODE -eq 0 -and $listed -match '^\d+$') { $releases = [int]$listed }
}

if ($Json) {
    Write-Output ('{"schema":"check-signing/1","pinned":"' + $pin +
                  '","releases":' + $releases +
                  ',"verified":"' + $verified +
                  '","problems":' + $problems.Count + '}')
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -eq 0) {
    Write-Output "signing ok: the release job signs with the runner's own identity, over the"
    Write-Output '  archive and the two files a consumer fetches, before the release exists.'
    Write-Output "  ⛔ No key, no secret, and the action is pinned to $pin."
    Write-Output "  ⚠ SKIP the live leg: $releases release(s) exist, and verifying needs one. A"
    Write-Output '  pushed tag is the only thing that cuts a release, and that is the'
    Write-Output "  operator's own act. Nothing here says an attestation was verified."
    exit 0
}

[Console]::Error.WriteLine("signing check failed, $($problems.Count) problem(s):")
[Console]::Error.WriteLine('')
foreach ($p in $problems) { [Console]::Error.WriteLine('  ' + $p) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A checksums file published beside the artefact proves transport rather')
[Console]::Error.WriteLine('than authorship. docs/history/todo/publish.md, PUB-09.')
exit 1
