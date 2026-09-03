# check-release.ps1 - would a release build produce the same bytes twice, and
# would it refuse to overwrite a tag somebody has already pinned?
#
# ⭐ THE TWIN OF check-release.sh. TODO/publish.md, PUB-01, and TODO/driver.md,
# DRIVER-09, is why a script in this directory does not land without one.
#
# ⛔ A CONSUMER THAT PINS A RELEASE AND GETS DIFFERENT BYTES LATER HAS BEEN
# BROKEN SILENTLY.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. two builds over one corpus are byte-identical, artefact by artefact;
#   2. the suite that owns the release rules is present, case by case;
#   3. ⛔ THE TAG THIS BUILD WOULD TAKE DOES NOT ALREADY EXIST, read from git;
#   4. a deterministic archive is byte-identical over two runs, where this
#      host's tar can make one. ⚠ A SKIP IS REPORTED AS A SKIP.
#
# ⛔ IT PUBLISHES NOTHING. -DryRun is required and is the only mode.
#
# ⚠ THE COMPARISON IS THIS HALF'S OWN: Get-FileHash where the twin uses diff.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-release.ps1 -DryRun
#   pwsh -NoProfile -File scripts/common/check-release.ps1 -DryRun -Json
#
# Exit codes: 0 reproducible and the tag is free, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [switch]$DryRun,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-release: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not $DryRun) {
    [Console]::Error.WriteLine('check-release: -DryRun is required. This check publishes nothing, and a')
    [Console]::Error.WriteLine('  run with no argument would read as though it had cut a release.')
    exit 2
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-release: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-release: not a git repository')
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
    [Console]::Error.WriteLine('check-release: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-release: cargo not found')
    exit 2
}

$suite = Join-Path $root 'crates/b-ids-corpus/tests/publish.rs'
if (-not (Test-Path -LiteralPath $suite)) {
    [Console]::Error.WriteLine("check-release: no suite at $suite")
    exit 2
}

$want = @(
    'publish_two_builds_over_one_corpus_are_byte_identical',
    'publish_every_artefact_has_a_checksum_and_the_checksum_is_of_the_file',
    'publish_the_tree_carries_no_source_and_no_vendored_dependency',
    'publish_a_tag_that_already_exists_is_refused',
    'publish_a_date_that_is_not_one_is_refused',
    'publish_a_build_with_no_artefact_is_not_releasable'
)

$problems = New-Object System.Collections.ArrayList
$suiteText = Get-Content -LiteralPath $suite -Raw
foreach ($name in $want) {
    if ($suiteText -notmatch [regex]::Escape('fn ' + $name)) {
        [void]$problems.Add("  $name is not in the suite")
    }
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-release-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -Recurse -Force -LiteralPath $out }
New-Item -ItemType Directory -Force -Path $out | Out-Null

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-release: the corpus crate did not build')
    exit 2
}
$bin = Join-Path $root 'target' | Join-Path -ChildPath 'debug' | Join-Path -ChildPath 'b-ids-corpus'
if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine("check-release: $bin is not there")
    exit 2
}

# -- 1: two builds, byte for byte -------------------------------------------
$logA = Join-Path $out 'a.log'
$logB = Join-Path $out 'b.log'
& $bin publish --root $corpusRoot --out (Join-Path $out 'a') > $logA 2>&1
$rcA = $LASTEXITCODE
& $bin publish --root $corpusRoot --out (Join-Path $out 'b') > $logB 2>&1
$rcB = $LASTEXITCODE
if ($rcA -ne 0 -or $rcB -ne 0) {
    [Console]::Error.WriteLine("check-release: the build exited $rcA then $rcB")
    Get-Content -LiteralPath $logA | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}
$status = Get-Content -LiteralPath $logA | Where-Object { $_ -like 'corpus=publish *' } | Select-Object -Last 1
if (-not $status) {
    [Console]::Error.WriteLine('check-release: the build printed no status line')
    exit 1
}
$files = [int][regex]::Match($status, 'files:(\d+)').Groups[1].Value
$bytes = [int][regex]::Match($status, 'bytes:(\d+)').Groups[1].Value
$from = [regex]::Match($status, 'from:([0-9a-f]+)').Groups[1].Value

# ⚠ THE COMPARISON IS THIS HALF'S OWN. Every file on both sides is hashed and
# the two maps are compared, so a file present on one side alone is a finding
# rather than something a recursive diff might report differently.
function Get-TreeHashMap($dir) {
    $map = @{}
    foreach ($f in (Get-ChildItem -LiteralPath $dir -Recurse -File)) {
        $relative = $f.FullName.Substring($dir.Length + 1) -replace '\\', '/'
        $map[$relative] = (Get-FileHash -LiteralPath $f.FullName -Algorithm SHA256).Hash
    }
    return $map
}
$left = Get-TreeHashMap (Join-Path $out 'a')
$right = Get-TreeHashMap (Join-Path $out 'b')
foreach ($key in $left.Keys) {
    if (-not $right.ContainsKey($key)) {
        [void]$problems.Add("  $key was produced by one build and not the other")
    }
    elseif ($left[$key] -ne $right[$key]) {
        [void]$problems.Add("  $key differs between runs")
    }
}
foreach ($key in $right.Keys) {
    if (-not $left.ContainsKey($key)) {
        [void]$problems.Add("  $key was produced by one build and not the other")
    }
}

# -- 2: the suite ------------------------------------------------------------
$testLog = Join-Path $out 'tests.log'
& cargo test -q -p b-ids-corpus --test publish > $testLog 2>&1
$rcT = $LASTEXITCODE
$cases = 0
$running = Get-Content -LiteralPath $testLog | Where-Object { $_ -match '^running (\d+) tests' } | Select-Object -First 1
if ($running -and $running -match '^running (\d+) tests') { $cases = [int]$Matches[1] }
if ($rcT -ne 0) { [void]$problems.Add('  the publish suite failed. Its output is in .tmp/check-release-ps/tests.log') }
if ($cases -lt $want.Count) {
    [void]$problems.Add("  the suite ran $cases case(s) where at least $($want.Count) were expected")
}

# -- 3: the tag this build would take ---------------------------------------
$today = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd')
$layout = ''
foreach ($line in (Get-Content -LiteralPath 'crates/b-ids-corpus/src/route.rs')) {
    if ($line -match '^pub const LAYOUT: &str = "([^"]+)";') { $layout = $Matches[1]; break }
}
$tag = $layout + '.' + ($today -replace '-', '.') + '.1'
& git rev-parse -q --verify ("refs/tags/" + $tag) > $null 2>&1
if ($LASTEXITCODE -eq 0) {
    [void]$problems.Add("  $tag already exists, so this build cannot be released under it. Bump the counter")
}
$tags = @(& git tag --list).Count

# -- 4: a deterministic archive ---------------------------------------------
#
# ⛔ A SKIP IS REPORTED AS A SKIP.
$archive = 'skipped'
if (Get-Command tar -ErrorAction SilentlyContinue) {
    $list = Join-Path $out 'files.txt'
    $names = @(Get-ChildItem -LiteralPath (Join-Path $out 'a') -Recurse -File |
        ForEach-Object { './' + ($_.FullName.Substring((Join-Path $out 'a').Length + 1) -replace '\\', '/') } |
        Sort-Object -CaseSensitive)
    Set-Content -LiteralPath $list -Value ($names -join "`n") -NoNewline
    $tarLog = Join-Path $out 'tar.log'
    # ⚠ THE OWNER FLAGS DIFFER BETWEEN TARS, and GNU tar reads a Windows path as
    # a remote host spec without --force-local. Both spellings are tried.
    # ⚠ TWO TARS, TWO SPELLINGS, ONE DATE FORMAT. Measured on this host
    # 2026-09-03: GNU tar 1.35 wants `--owner=0 --group=0` and refuses `--uid`;
    # the bsdtar that ships with Windows wants `--uid 0 --gid 0` and refuses
    # `--force-local`; and `2026-01-01T00:00:00Z` is a bad date string to bsdtar
    # while `2026-01-01 00:00:00` is accepted by both. ⛔ The two halves of this
    # check resolve DIFFERENT tar binaries on this machine, so a leg that only
    # worked for one of them made the pair disagree.
    $epoch = '2026-01-01 00:00:00'
    $flags = @('--force-local', '--format=ustar', '--numeric-owner', '--owner=0', '--group=0', '--mtime', $epoch)
    & tar @flags -cf (Join-Path $out 'probe.tar') -C (Join-Path $out 'a') -T $list > $tarLog 2>&1
    if ($LASTEXITCODE -ne 0) {
        $flags = @('--format=ustar', '--numeric-owner', '--uid', '0', '--gid', '0', '--mtime', $epoch)
    }
    & tar @flags -cf (Join-Path $out 'one.tar') -C (Join-Path $out 'a') -T $list >> $tarLog 2>&1
    $rcOne = $LASTEXITCODE
    & tar @flags -cf (Join-Path $out 'two.tar') -C (Join-Path $out 'b') -T $list >> $tarLog 2>&1
    $rcTwo = $LASTEXITCODE
    if ($rcOne -eq 0 -and $rcTwo -eq 0) {
        $h1 = (Get-FileHash -LiteralPath (Join-Path $out 'one.tar') -Algorithm SHA256).Hash
        $h2 = (Get-FileHash -LiteralPath (Join-Path $out 'two.tar') -Algorithm SHA256).Hash
        if ($h1 -eq $h2) { $archive = 'ok' }
        else {
            $archive = 'failed'
            [void]$problems.Add('  two archives of one build differ, so the archive step is not reproducible')
        }
    }
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-release/1","files":' + $files + ',"bytes":' + $bytes +
                  ',"cases":' + $cases + ',"tags":' + $tags + ',"archive":"' + $archive +
                  '","problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output "release ok: $files artefact(s), $bytes byte(s), identical over two builds."
    Write-Output "  built from corpus $from"
    Write-Output "  $tag would be free, over $tags existing tag(s). archive: $archive"
    if ($archive -eq 'skipped') {
        Write-Output "  `u{26A0} A SKIP IS NOT A PASS: this tar cannot make a deterministic archive."
    }
    Write-Output "  `u{26D4} Nothing was tagged, uploaded or pushed."
    exit 0
}

[Console]::Error.WriteLine("release check failed, $count problem(s):")
[Console]::Error.WriteLine('')
$problems | ForEach-Object { [Console]::Error.WriteLine($_) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A consumer that pins a release and gets different bytes later has been')
[Console]::Error.WriteLine('broken silently. TODO/publish.md, PUB-01.')
exit 1
