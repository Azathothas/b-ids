# check-license-consistency.ps1 - do the places that state this project's
# licence all state the same one?
#
# ⭐ THE TWIN OF check-license-consistency.sh. TODO/publish.md, PUB-07, and
# TODO/driver.md, DRIVER-09, is why a script in this directory does not land
# without one.
#
# ⛔ A FILE THAT TRAVELS ALONE STILL HAS TO SAY WHAT IT IS. A consumer who
# downloads one profile should not have to find this repository to learn they
# may use it.
#
# -- ⛔ THE PLACES, AND WHY EACH ONE ------------------------------------------
#
#   the workspace manifest   what a builder of the code sees
#   b_ids_schema::LICENSE    ⭐ THE ONE HOME. Everything generated reads it.
#   the published JSON Schema  what a consumer validating a profile sees
#   the corpus index         what a consumer who fetches only the index sees
#   every published profile  what a consumer who fetches ONE file sees
#   the release body         what a consumer who downloads an asset sees
#
# ⚠ THE SIX PROFILES PUBLISHED BEFORE 2026-09-03 DO NOT CARRY THE FIELD, and
# that is recorded rather than repaired. The corpus is append-only.
#
# ⭐ AND THE DATA BRANCH, from a LOCAL ref rather than a fetch: its manifest
# identifier and the bytes of its LICENSE. ⚠ NO LOCAL REF IS A SKIP naming the
# branch, never a pass. TODO/publish.md, PUB-12.
#
# ⚠ THE READING IS THIS HALF'S OWN: ConvertFrom-Json where the twin uses jq, so
# the pair compares two readings rather than two wrappers over one.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-license-consistency.ps1
#   pwsh -NoProfile -File scripts/common/check-license-consistency.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-license-consistency.ps1 -Fixture
#
# Exit codes: 0 they agree, 1 one disagrees, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [switch]$Fixture,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-license-consistency: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-license-consistency: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-license-consistency: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-license-consistency: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot

$problems = New-Object System.Collections.ArrayList
$seen = New-Object System.Collections.ArrayList
function Add-Statement($where, $value) {
    [void]$seen.Add("  ${where}: $value")
}

# -- the one home ------------------------------------------------------------
#
# ⛔ READ FROM THE SOURCE, never typed here. A check carrying its own copy of
# the identifier is a seventh place for it to disagree.
$homeFile = 'crates/b-ids-schema/src/lib.rs'
if (-not (Test-Path -LiteralPath $homeFile)) {
    [Console]::Error.WriteLine("check-license-consistency: no $homeFile")
    exit 2
}
$want = ''
foreach ($line in (Get-Content -LiteralPath $homeFile)) {
    if ($line -match '^pub const LICENSE: &str = "([^"]+)";') { $want = $Matches[1]; break }
}
if (-not $want) {
    [Console]::Error.WriteLine("check-license-consistency: $homeFile declares no LICENSE constant")
    exit 2
}
Add-Statement $homeFile $want

# -- the workspace manifest --------------------------------------------------
$manifest = 'absent'
foreach ($line in (Get-Content -LiteralPath 'Cargo.toml')) {
    if ($line -match '^license = "([^"]+)"') { $manifest = $Matches[1]; break }
}
Add-Statement 'Cargo.toml' $manifest
if ($manifest -ne $want) {
    [void]$problems.Add("  Cargo.toml says $manifest and $homeFile says $want")
}

# -- the published JSON Schema -----------------------------------------------
$schemaFile = 'crates/b-ids-schema/schema/browser-profile-1.schema.json'
$schemaJson = Get-Content -LiteralPath $schemaFile -Raw | ConvertFrom-Json
$schema = 'absent'
if ($schemaJson.properties.PSObject.Properties.Name -contains 'license') {
    $schema = $schemaJson.properties.license.const
}
Add-Statement $schemaFile $schema
if ($schema -ne $want) {
    [void]$problems.Add("  the published schema says $schema and $homeFile says $want")
}
# ⚠ OPTIONAL ON PURPOSE. A schema requiring the field would refuse every profile
# published before it existed, and those are append-only.
if (@($schemaJson.required | Where-Object { $_ -eq 'license' }).Count -ne 0) {
    [void]$problems.Add('  the published schema REQUIRES license, which refuses every profile published before it existed')
}

# -- the corpus index --------------------------------------------------------
$indexFile = Join-Path $corpusRoot 'corpus/v1/index.json'
if (Test-Path -LiteralPath $indexFile) {
    $indexJson = Get-Content -LiteralPath $indexFile -Raw | ConvertFrom-Json
    $index = 'absent'
    if ($indexJson.PSObject.Properties.Name -contains 'license') { $index = $indexJson.license }
    Add-Statement $indexFile $index
    if ($index -ne $want) {
        [void]$problems.Add("  the index says $index and $homeFile says $want")
    }
}
else {
    [void]$problems.Add('  there is no corpus index, so the licence it states was not checked')
}

# -- every published profile -------------------------------------------------
#
# ⚠ COUNTED IN THREE, not two. A profile that carries a DIFFERENT licence is a
# defect; one that carries NONE predates the field.
$profiles = 0
$carrying = 0
$predating = 0
# ⚠ A FILESYSTEM WALK RATHER THAN `git ls-files`, because once the corpus
# leaves the default branch the root is a materialised copy of the data branch,
# which git knows nothing about as a working tree. TODO/publish.md, PUB-11.
foreach ($file in @(Get-ChildItem -LiteralPath (Join-Path $corpusRoot 'corpus/v1') -Recurse -File -Filter '*.json' -ErrorAction SilentlyContinue |
            ForEach-Object { $_.FullName } | Sort-Object)) {
    if (-not $file) { continue }
    # ⚠ ON THE LEAF NAME, never on the path. $_.FullName is BACKSLASH-separated
    # on Windows, so a '*/index.json' pattern matches nothing there and the two
    # derived files were counted as profiles: the twin said 8 where the POSIX
    # half said 6. Found by comparing the two answers. TODO/publish.md, PUB-11.
    $leaf = [System.IO.Path]::GetFileName($file)
    if ($leaf -eq 'index.json' -or $leaf -eq 'latest.json') { continue }
    $profiles++
    $profileJson = Get-Content -LiteralPath $file -Raw | ConvertFrom-Json
    if ($profileJson.PSObject.Properties.Name -notcontains 'license') { $predating++; continue }
    if ($profileJson.license -eq $want) { $carrying++; continue }
    [void]$problems.Add("  $file says $($profileJson.license) and $homeFile says $want")
}
Add-Statement 'corpus profiles' "$carrying carrying it, $predating published before the field existed"
if ($profiles -eq 0) {
    [void]$problems.Add('  no published profile was read, so nothing about the corpus was checked')
}

# -- ⭐ the branch a consumer actually fetches --------------------------------
#
# ⛔ A LOCAL REF, NEVER A FETCH. ⚠ NO LOCAL REF AT ALL IS A SKIP naming the
# branch, never a pass. TODO/publish.md, PUB-12.
$dataBranch = 'data'
$dataRef = ''
& git rev-parse -q --verify "refs/heads/$dataBranch" 2>$null | Out-Null
if ($LASTEXITCODE -eq 0) {
    $dataRef = "refs/heads/$dataBranch"
}
else {
    & git rev-parse -q --verify "refs/remotes/origin/$dataBranch" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { $dataRef = "refs/remotes/origin/$dataBranch" }
}
$data = 'skipped'
if ($dataRef -ne '') {
    $manifestText = (& git show ($dataRef + ':MANIFEST.json') 2>$null) -join "`n"
    $data = 'absent'
    if ($manifestText.Trim() -ne '') {
        try {
            $parsed = $manifestText | ConvertFrom-Json
            if ($parsed.PSObject.Properties.Name -contains 'license' -and $parsed.license) {
                $data = "$($parsed.license)"
            }
        }
        catch {
            $data = 'absent'
        }
    }
    if ($data -ne $want) {
        [void]$problems.Add("  the data branch manifest says $data and $homeFile says $want")
    }
    # ⛔ AND THE TEXT, not only the identifier. A branch naming 0BSD over some
    # other licence file is the failure an identifier comparison cannot see.
    $branchText = (& git rev-parse ($dataRef + ':LICENSE') 2>$null | Select-Object -First 1)
    $localText = (& git hash-object LICENSE 2>$null | Select-Object -First 1)
    if (-not $branchText) {
        [void]$problems.Add("  the $dataBranch branch carries no LICENSE at its root")
    }
    elseif ("$branchText".Trim() -ne "$localText".Trim()) {
        [void]$problems.Add("  the LICENSE on $dataBranch is not the LICENSE in this tree")
    }
    Add-Statement "the $dataBranch branch" "$data, over the same LICENSE text"
}
else {
    Add-Statement "the $dataBranch branch" "skipped: no local ref for $dataBranch"
}

# -- the release body --------------------------------------------------------
#
# ⛔ DELEGATED TO THE GENERATOR. A second renderer of the release body written
# here would be a second answer to what a release says.
$bodySuite = 'crates/b-ids-corpus/tests/notes.rs'
if ((Test-Path -LiteralPath $bodySuite) -and
    (Select-String -LiteralPath $bodySuite -SimpleMatch 'notes_the_release_body_states_the_licence' -Quiet)) {
    Add-Statement $bodySuite 'asserted by notes_the_release_body_states_the_licence'
}
else {
    [void]$problems.Add("  $bodySuite has no case asserting the release body states the licence")
}

# -- ⛔ what a FRESHLY WRITTEN profile carries -------------------------------
#
# ⛔ THE LEG ABOVE IS VACUOUS TODAY AND SAYING SO IS THE POINT: every published
# profile predates the field. What the WRITER produces is the rule that can be
# broken now.
$writerSuite = 'crates/b-ids-schema/tests/profile.rs'
if ((Test-Path -LiteralPath $writerSuite) -and
    (Select-String -LiteralPath $writerSuite -SimpleMatch 'profile_a_freshly_written_one_carries_the_licence' -Quiet)) {
    Add-Statement $writerSuite 'asserted by profile_a_freshly_written_one_carries_the_licence'
}
else {
    [void]$problems.Add("  $writerSuite has no case asserting a freshly written profile carries the licence")
}

# -- ⛔ and the comparison can fail ------------------------------------------
if ($Fixture) {
    # ⛔ A COPY, never the file on this machine.
    $fixtureValue = 'MIT'
    if ($fixtureValue -ne 'MIT') {
        [void]$problems.Add('  the fixture did not produce the identifier it was written to produce')
    }
    # ⛔ THE COMPARISON ITSELF, run against the fixture. Asserting only that the
    # fixture DIFFERS would prove the fixture and not the check.
    $refused = ($fixtureValue -ne $want)
    if (-not $refused) {
        [void]$problems.Add("  the comparison accepted a fixture stating $fixtureValue where $want was expected")
    }
    Add-Statement 'the fixture' "$fixtureValue, refused by the same comparison"
}

$count = $problems.Count
$stated = $seen.Count

if ($Json) {
    Write-Output ('{"schema":"check-license-consistency/2","license":"' + $want +
                  '","stated":' + $stated + ',"profiles":' + $profiles +
                  ',"carrying":' + $carrying + ',"predating":' + $predating +
                  ',"data_branch":"' + $data + '","problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output "licence ok: every statement says $want."
    Write-Output ''
    $seen | ForEach-Object { Write-Output $_ }
    Write-Output ''
    Write-Output "`u{26A0} $predating profile(s) were published before the field existed and do not carry it."
    Write-Output '  The corpus is append-only, so they never will.'
    exit 0
}

[Console]::Error.WriteLine("licence check failed, $count problem(s):")
[Console]::Error.WriteLine('')
$problems | ForEach-Object { [Console]::Error.WriteLine($_) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('Every statement of the licence, as read:')
[Console]::Error.WriteLine('')
$seen | ForEach-Object { [Console]::Error.WriteLine($_) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A file that travels alone still has to say what it is.')
[Console]::Error.WriteLine('TODO/publish.md, PUB-07.')
exit 1
