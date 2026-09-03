# check-publish.ps1 - does the workflow that publishes this project declare what
# it must, and do the rules it defers to actually refuse?
#
# ⭐ THE TWIN OF check-publish.sh. TODO/driver.md, DRIVER-09, is why a script in
# this directory does not land without one.
#
# ⛔ NOTHING IN THIS REPOSITORY WAS EVER PUBLISHED UNTIL A TRIGGER EXISTED, and
# the first thing a trigger can get wrong is irreversible: a force push over the
# data branch discards every commit a consumer pinned. TODO/publish.md, PUB-10.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the workflow exists and declares all THREE triggers the operator ruled;
#   2. ⛔ THE WRITE IS JOB-SCOPED, and the top of the file grants read;
#   3. ⛔ NO PERSONAL ACCESS TOKEN: the word `secrets.` does not appear;
#   4. ⛔ NO FORCE PUSH on any `git push` line;
#   5. the crate's rule is consulted before the push, by line order;
#   6. both publishing jobs need the job that runs the two existing checks;
#   7. the archive epoch is read from check-release.sh rather than typed;
#   8. ⭐ THE RULES ACTUALLY REFUSE, driven against the built binary.
#
# ⚠ THE READING IS THIS HALF'S OWN: -match and -split over the file's lines
# where the twin uses awk, sed and grep, so the pair compares two readings
# rather than two wrappers over one.
#
# ⛔ AN ABSENT WORKFLOW IS EXIT 1, NOT 2. The path is fixed and named, and the
# file not being there is precisely the defect this check exists to catch.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-publish.ps1
#   pwsh -NoProfile -File scripts/common/check-publish.ps1 -Json
#
# Exit codes: 0 the workflow declares what it must and the rules refuse,
# 1 one of them does not, 2 could not run.
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
    [Console]::Error.WriteLine('check-publish: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-publish: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-publish: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-publish: cargo not found')
    exit 2
}

$workflow = '.github/workflows/publish.yml'
# ⛔ THE TWO JOBS THAT MAY WRITE, named here and asserted there.
$publishing = @('data-branch', 'release')

$problems = New-Object System.Collections.ArrayList
$triggers = 0
$jobCount = 0
$writes = 0
$cases = 0

function Show-Failure {
    [Console]::Error.WriteLine("publish check failed, $($problems.Count) problem(s):")
    [Console]::Error.WriteLine('')
    $problems | ForEach-Object { [Console]::Error.WriteLine($_) }
    [Console]::Error.WriteLine('')
    [Console]::Error.WriteLine('A force push over the data branch discards every commit a consumer')
    [Console]::Error.WriteLine('pinned. TODO/publish.md, PUB-10.')
}

if (-not (Test-Path -LiteralPath $workflow -PathType Leaf)) {
    [void]$problems.Add("  there is no $workflow, so nothing in this repository publishes anything")
    Show-Failure
    exit 1
}

$lines = @(Get-Content -LiteralPath $workflow)

# ⚠ A TOP-LEVEL BLOCK, read by indentation. A key appearing anywhere in a file
# says nothing about which block it is in.
function Get-TopBlock {
    param([string[]]$Text, [string]$Key)
    $out = New-Object System.Collections.ArrayList
    $inside = $false
    foreach ($line in $Text) {
        if ($line -match "^$Key`:\s*$") { $inside = $true; continue }
        if ($inside -and $line -match '^[a-zA-Z]') { $inside = $false }
        if ($inside) { [void]$out.Add($line) }
    }
    return , $out.ToArray()
}

# -- 1: the three triggers ---------------------------------------------------
$onBlock = Get-TopBlock -Text $lines -Key 'on'
if ($onBlock -match '^  workflow_dispatch:') { $triggers++ }
else { [void]$problems.Add('  the workflow does not declare workflow_dispatch, which is one of the three triggers') }
if ($onBlock -match '^    branches:.*main') { $triggers++ }
else { [void]$problems.Add('  the workflow does not trigger on a push to the default branch') }
if ($onBlock -match '^    tags:') { $triggers++ }
else { [void]$problems.Add('  the workflow does not trigger on a pushed tag, so no release is ever cut') }

# -- 2: the write is job-scoped ----------------------------------------------
$topPermissions = Get-TopBlock -Text $lines -Key 'permissions'
if (-not ($topPermissions -match '^  contents: read\s*$')) {
    [void]$problems.Add('  the top-level permissions block does not grant contents: read and nothing else')
}
if ($topPermissions -match 'write') {
    [void]$problems.Add('  the top-level permissions block grants a write, which every job would then hold')
}

$writes = @($lines | Where-Object { $_ -match '^      contents: write\s*$' }).Count
$wantWrites = $publishing.Count
if ($writes -ne $wantWrites) {
    [void]$problems.Add("  $writes job(s) declare contents: write where $wantWrites publishing job(s) are expected")
}

$jobNames = New-Object System.Collections.ArrayList
$inJobs = $false
foreach ($line in $lines) {
    if ($line -match '^jobs:\s*$') { $inJobs = $true; continue }
    if ($inJobs -and $line -match '^[a-zA-Z]') { $inJobs = $false }
    if ($inJobs -and $line -match '^  ([A-Za-z0-9_-]+):\s*$') { [void]$jobNames.Add($Matches[1]) }
}
$jobCount = $jobNames.Count
foreach ($want in $publishing) {
    if ($jobNames -notcontains $want) {
        [void]$problems.Add("  there is no $want job in $workflow")
    }
}

# -- 3: no personal access token ---------------------------------------------
if ($lines -match 'secrets\.') {
    [void]$problems.Add("  the workflow names a secret. The write is the run's own token and never a personal access token")
}

# -- 4: no force push --------------------------------------------------------
#
# ⛔ A COMMENT MAY SAY THE WORDS; A COMMAND MAY NOT CARRY THEM, so the whole-line
# comments are dropped first.
#
# ⛔ ANY `+` ON A `git push` LINE IS A FORCE, and the rule is that blunt because
# the first version was not. It looked for `:+`, which is where a `+` is NOT: a
# forcing refspec is `+src:dst`, so the plus sits at the START of the token, and
# the mutation that gave the push a leading `+` passed both halves.
$live = @($lines | Where-Object { $_ -notmatch '^\s*#' })
$forced = @($live | Where-Object { $_ -match 'git push' } |
        Where-Object { $_ -match '--force' -or $_ -match '\+' }).Count
if ($forced -ne 0) {
    [void]$problems.Add("  $forced git push line(s) carry a force flag or a + refspec, and the data branch is append-only")
}

# ⛔ AND THE SAME RULE OVER EVERY OTHER WORKFLOW. A control gated on one path and
# not its siblings is the single most recurring hole there is, and this check
# began by reading publish.yml alone while capture.yml carried a force push that
# nothing asserted anything about.
#
# ⚠ THE COUNT IS PINNED rather than the flag banned: CI-04's bot branch is
# force-pushed with a lease on purpose, and a SECOND one is a thing somebody has
# to look at.
$everyPush = @(Get-ChildItem -LiteralPath '.github/workflows' -Filter '*.yml' -File |
        ForEach-Object { Get-Content -LiteralPath $_.FullName } |
        Where-Object { $_ -notmatch '^\s*#' } |
        Where-Object { $_ -match 'git push' })
$everyForced = @($everyPush | Where-Object { $_ -match '--force' -or $_ -match '\+' })
if ($everyForced.Count -ne 1) {
    [void]$problems.Add("  $($everyForced.Count) git push line(s) across every workflow force, and exactly one may: CI-04's bot branch")
}
$badTarget = @($everyForced | Where-Object {
        $_ -match 'refs/heads/data' -or $_ -match 'refs/heads/main' -or $_ -match 'origin\s+main'
    }).Count
if ($badTarget -ne 0) {
    [void]$problems.Add("  $badTarget force push(es) name the data branch or the default branch, and neither is ever force-pushed")
}

# -- 5: the rule is consulted before the push --------------------------------
$ruleAt = 0
$pushAt = 0
for ($i = 0; $i -lt $live.Count; $i++) {
    if ($ruleAt -eq 0 -and $live[$i] -match '\-\- data-branch') { $ruleAt = $i + 1 }
    if ($pushAt -eq 0 -and $live[$i] -match 'git push origin') { $pushAt = $i + 1 }
}
if ($ruleAt -eq 0) {
    [void]$problems.Add('  no step calls b-ids-corpus data-branch, so nothing asks the crate whether the push appends')
}
elseif ($pushAt -eq 0) {
    [void]$problems.Add('  no step pushes the data branch, so this workflow publishes only a release')
}
elseif ($ruleAt -ge $pushAt) {
    [void]$problems.Add("  the push at line $pushAt comes before the rewrite rule at line $ruleAt")
}

# -- 6: both publishing jobs need the job that checks -------------------------
foreach ($check in @('check-release.sh', 'check-data-branch.sh')) {
    if (-not ($lines -match [regex]::Escape($check))) {
        [void]$problems.Add("  $check is not run by the workflow, so a tree that fails it would still publish")
    }
}
$needs = @($lines | Where-Object { $_ -match '^    needs: \[assemble\]\s*$' }).Count
if ($needs -ne $wantWrites) {
    [void]$problems.Add("  $needs job(s) need the assemble job where $wantWrites publishing job(s) are expected")
}

# -- 7: the archive epoch is derived -----------------------------------------
$epoch = ''
foreach ($line in Get-Content -LiteralPath 'scripts/common/check-release.sh') {
    if ($line -match '^TAR_EPOCH="([^"]*)"') { $epoch = $Matches[1]; break }
}
if (-not $epoch) {
    [void]$problems.Add('  check-release.sh no longer states TAR_EPOCH, which the workflow reads')
}
if (-not ($lines -match 'TAR_EPOCH=')) {
    [void]$problems.Add('  the workflow does not read TAR_EPOCH from check-release.sh, so the epoch is stated twice')
}
if (-not ($live -match '--mtime "\$epoch"')) {
    [void]$problems.Add('  the workflow''s tar does not use the epoch it read')
}

# -- 8: the rules actually refuse --------------------------------------------
$suite = Join-Path $root 'crates/b-ids-corpus/tests/publish.rs'
if (-not (Test-Path -LiteralPath $suite -PathType Leaf)) {
    [Console]::Error.WriteLine("check-publish: no suite at $suite")
    exit 2
}
$suiteText = Get-Content -LiteralPath $suite -Raw
foreach ($want in @(
        'publish_a_push_that_would_rewrite_the_data_branch_is_refused',
        'publish_a_tag_this_rule_did_not_produce_is_refused',
        'publish_the_tree_names_no_path_outside_itself')) {
    if ($suiteText -notmatch [regex]::Escape("fn $want")) {
        [void]$problems.Add("  $want is not in the suite")
    }
}

$out = Join-Path $root '.tmp/check-publish-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Recurse -Force }
New-Item -ItemType Directory -Path $out -Force | Out-Null

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-publish: the corpus crate did not build')
    exit 2
}
$bin = Join-Path $root 'target/debug/b-ids-corpus.exe'
if (-not (Test-Path -LiteralPath $bin -PathType Leaf)) {
    $bin = Join-Path $root 'target/debug/b-ids-corpus'
}
if (-not (Test-Path -LiteralPath $bin -PathType Leaf)) {
    [Console]::Error.WriteLine("check-publish: $bin is not executable")
    exit 2
}
$driveLog = Join-Path $out 'drive.log'

# ⛔ READ FROM THE PROCESS, UNPIPED, EVERY TIME.
function Invoke-Rule {
    param([int]$Want, [string]$Why, [string[]]$Arguments)
    & $bin @Arguments *> $driveLog
    $rc = $LASTEXITCODE
    $script:cases++
    if ($rc -ne $Want) {
        [void]$script:problems.Add("  ${Why}: exit $rc where $Want was expected")
    }
}

Invoke-Rule 0 'the first push creates the branch' @('data-branch', '--head', 'none', '--parent', 'none')
Invoke-Rule 0 'a commit built on the branch head appends' @('data-branch', '--head', 'abc123', '--parent', 'abc123')
Invoke-Rule 1 'a commit built on something the branch moved past is a rewrite' @('data-branch', '--head', 'abc123', '--parent', 'def456')
Invoke-Rule 1 'an orphan commit pushed over an existing branch discards every commit on it' @('data-branch', '--head', 'abc123', '--parent', 'none')
Invoke-Rule 2 'data-branch with no --parent must not answer append' @('data-branch', '--head', 'abc123')

$tree = Join-Path $out 'tree'
& $bin publish --root $root --out $tree *> (Join-Path $out 'publish.log')
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine("check-publish: the assembler exited $LASTEXITCODE")
    Get-Content -LiteralPath (Join-Path $out 'publish.log') | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}

Invoke-Rule 0 'a well-formed tag over an assembled tree is releasable' @('release', '--tree', $tree, '--tag', 'v1.2026.01.01.1', '--notes', (Join-Path $out 'NOTES.md'))
Invoke-Rule 1 'a zero-padded counter is not the tag this rule produces' @('release', '--tree', $tree, '--tag', 'v1.2026.01.01.01')
Invoke-Rule 1 'a malformed date is refused' @('release', '--tree', $tree, '--tag', 'v1.2026.1.1.1')
Invoke-Rule 1 'a tag naming another layout is refused' @('release', '--tree', $tree, '--tag', 'v9.2026.01.01.1')
$released = Join-Path $out 'released.txt'
Set-Content -LiteralPath $released -Value 'v1.2026.01.01.1' -NoNewline
Invoke-Rule 1 'a tag that already carries a release is refused' @('release', '--tree', $tree, '--tag', 'v1.2026.01.01.1', '--existing', $released)

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-publish/1","triggers":' + $triggers +
                  ',"jobs":' + $jobCount + ',"writes":' + $writes +
                  ',"cases":' + $cases + ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output "publish ok: $triggers trigger(s) over $jobCount job(s), $writes job-scoped write(s),"
    Write-Output "  no force push and no named secret, and $cases refusal(s) driven against the binary."
    Write-Output "  `u{26D4} Nothing was tagged, uploaded or pushed."
    exit 0
}

Show-Failure
exit 1
