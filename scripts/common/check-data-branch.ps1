# check-data-branch.ps1 - is what the data branch would carry exactly what the
# corpus derives to, and would a push that rewrote it be refused?
#
# ⭐ THE TWIN OF check-data-branch.sh. docs/history/todo/publish.md, PUB-02, and
# docs/history/todo/driver.md, DRIVER-09, is why a script in this directory does not land
# without one.
#
# ⛔ A CONSUMER PINNING A COMMIT ON THE DATA BRANCH KEEPS WORKING FOREVER, and
# that property is free right up until somebody rewrites the branch.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the tree is regenerated from the canonical corpus and two builds of it
#      are byte-identical;
#   2. EVERY file has a checksum, in the manifest AND in SHA256SUMS;
#   3. ⛔ the source, any vendored dependency and the reference corpus are NOT
#      on it;
#   4. a push that would rewrite history is refused, which is a rule in the
#      crate with its own case.
#
# ⭐ AND IT COMPARES AGAINST WHAT IS PUBLISHED, as two git tree objects, with
# the answer in the JSON as `matched`. ⚠ WITH NO BRANCH AT ALL that leg is a
# SKIP naming the branch that would make it run.
#
# ⛔ IT PUSHES NOTHING and creates no branch.
#
# ⚠ THE READING IS THIS HALF'S OWN: ConvertFrom-Json and Get-FileHash where the
# twin uses jq and diff.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-data-branch.ps1
#   pwsh -NoProfile -File scripts/common/check-data-branch.ps1 -Json
#
# Exit codes: 0 the tree is what it should be, 1 it is not, 2 could not run.
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
    [Console]::Error.WriteLine('check-data-branch: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-data-branch: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-data-branch: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. docs/history/todo/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-data-branch: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ THIS CHECK RESOLVES THE ROOT AND THEN REFUSES SOME ANSWERS. Its question is
# whether the published branch equals what the CANONICAL corpus derives to, so
# a run that resolved to the DATA branch would compare it against itself and
# pass without asking anything.
#
# ⭐ SINCE PUB-13 THE CANONICAL CORPUS IS THE SOURCE BRANCH, and this check runs
# against it rather than skipping. docs/history/todo/publish.md, PUB-11 and PUB-13.
#
#   working-tree    a session that has the corpus checked out. Canonical.
#   source-branch   the everyday answer since PUB-13. Canonical.
#   data-branch     ⛔ REFUSED. It is what this check is checking.
#   explicit        ⛔ REFUSED. B_IDS_CORPUS_ROOT may name anything, including a
#                   copy of the data branch, and this guard cannot tell which.
#
# ⛔ THE GUARD IS ASKED BEFORE THE EXPORT, AND IT ASKS FOR THE SOURCE RATHER
# THAN THE REF. Both halves of that sentence were defects. The export sets
# B_IDS_CORPUS_ROOT, and the resolver's first rule is that an explicit root is
# never second guessed, so anything asked afterwards answers `explicit` with an
# EMPTY ref whatever it would have said a line earlier.
# ⭐ Driven 2026-09-04 with corpus/ and raw/ moved out: this check reported
# `data branch ok` and exited 0, having compared the published branch against a
# materialised copy of that same branch.
$corpusSource = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') -Source | Select-Object -First 1)
if ($null -eq $corpusSource) { $corpusSource = '' }
$corpusSource = "$corpusSource".Trim()
if ($corpusSource -ne 'working-tree' -and $corpusSource -ne 'source-branch') {
    [Console]::Error.WriteLine('check-data-branch: the canonical corpus did not resolve to this tree or')
    [Console]::Error.WriteLine('to the source branch, so the data branch has nothing independent to be')
    [Console]::Error.WriteLine("compared against. It resolved to $corpusSource.")
    exit 2
}
# ⛔ AND EXPORTED ONLY AFTER THE GUARD HAS RUN, because cargo is downstream of
# this decision but the guard must not be. The b-ids crate's build script
# embeds the corpus at build time and reads exactly this variable.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-data-branch: cargo not found')
    exit 2
}

# ⛔ THE BRANCH IS NAMED ONCE, here and in the twin.
$branch = 'data'

$suite = Join-Path $root 'crates/b-ids-corpus/tests/publish.rs'
if (-not (Test-Path -LiteralPath $suite)) {
    [Console]::Error.WriteLine("check-data-branch: no suite at $suite")
    exit 2
}

$want = @(
    'publish_two_builds_over_one_corpus_are_byte_identical',
    'publish_every_artefact_has_a_checksum_and_the_checksum_is_of_the_file',
    'publish_the_tree_carries_no_source_and_no_vendored_dependency',
    'publish_the_tree_carries_the_corpus_the_formats_and_the_routes',
    'publish_a_push_that_would_rewrite_the_data_branch_is_refused'
)

$problems = New-Object System.Collections.ArrayList
$suiteText = Get-Content -LiteralPath $suite -Raw
foreach ($name in $want) {
    if ($suiteText -notmatch [regex]::Escape('fn ' + $name)) {
        [void]$problems.Add("  $name is not in the suite")
    }
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-data-branch-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -Recurse -Force -LiteralPath $out }
New-Item -ItemType Directory -Force -Path $out | Out-Null

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-data-branch: the corpus crate did not build')
    exit 2
}
$targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
$bin = Join-Path $targetDir 'debug' | Join-Path -ChildPath 'b-ids-corpus'
if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine("check-data-branch: $bin is not there")
    exit 2
}

# -- 1: regenerate, twice ----------------------------------------------------
$logA = Join-Path $out 'a.log'
$logB = Join-Path $out 'b.log'
& $bin publish --root $corpusRoot --out (Join-Path $out 'a') > $logA 2>&1
$rcA = $LASTEXITCODE
& $bin publish --root $corpusRoot --out (Join-Path $out 'b') > $logB 2>&1
$rcB = $LASTEXITCODE
if ($rcA -ne 0 -or $rcB -ne 0) {
    [Console]::Error.WriteLine("check-data-branch: the build exited $rcA then $rcB")
    Get-Content -LiteralPath $logA | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}
$status = Get-Content -LiteralPath $logA | Where-Object { $_ -like 'corpus=publish *' } | Select-Object -Last 1
if (-not $status) {
    [Console]::Error.WriteLine('check-data-branch: the build printed no status line')
    exit 1
}
$files = [int][regex]::Match($status, 'files:(\d+)').Groups[1].Value

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
    if (-not $right.ContainsKey($key) -or $left[$key] -ne $right[$key]) {
        [void]$problems.Add("  $key differs between two builds of the branch content")
    }
}

# -- 2: every file has a checksum, in both places ---------------------------
$manifest = Get-Content -LiteralPath (Join-Path $out 'a/MANIFEST.json') -Raw | ConvertFrom-Json
$recordedSet = [System.Collections.Generic.HashSet[string]]::new(
    [string[]]@($manifest.artefacts | ForEach-Object { $_.path }), [System.StringComparer]::Ordinal)
$sums = Get-Content -LiteralPath (Join-Path $out 'a/SHA256SUMS') -Raw
$sumLines = @($sums -split "`n" | Where-Object { $_.Trim() })
$recorded = $recordedSet.Count
if ($recorded -ne $sumLines.Count) {
    [void]$problems.Add("  the manifest records $recorded file(s) and SHA256SUMS carries $($sumLines.Count)")
}
$present = 0
foreach ($key in ($left.Keys | Sort-Object -CaseSensitive)) {
    $present++
    if ($key -eq 'MANIFEST.json' -or $key -eq 'SHA256SUMS') { continue }
    if (-not $recordedSet.Contains($key)) {
        [void]$problems.Add("  $key is on the branch and has no checksum in the manifest")
    }
    if (-not $sums.Contains("  $key")) {
        [void]$problems.Add("  $key is on the branch and has no line in SHA256SUMS")
    }
}

# -- 3: nothing of the source is on it --------------------------------------
foreach ($forbidden in @('crates', 'vendor', 'references', 'scripts', 'target', 'docs', 'TODO')) {
    if (Test-Path -LiteralPath (Join-Path $out "a/$forbidden")) {
        [void]$problems.Add("  $forbidden is on the branch and must not be")
    }
}

# -- 4: the suite, and the branch's own state -------------------------------
$testLog = Join-Path $out 'tests.log'
& cargo test -q -p b-ids-corpus --test publish > $testLog 2>&1
$rcT = $LASTEXITCODE
$cases = 0
$running = Get-Content -LiteralPath $testLog | Where-Object { $_ -match '^running (\d+) tests' } | Select-Object -First 1
if ($running -and $running -match '^running (\d+) tests') { $cases = [int]$Matches[1] }
if ($rcT -ne 0) { [void]$problems.Add('  the publish suite failed. Its output is in .tmp/check-data-branch-ps/tests.log') }
if ($cases -lt $want.Count) {
    [void]$problems.Add("  the suite ran $cases case(s) where at least $($want.Count) were expected")
}

# ⛔ THE LEG THAT COULD NOT RUN, AND NOW DOES. Until 2026-09-03 the branch did
# not exist and this reported a skip saying "push it once and this leg starts
# running". ⚠ It was pushed and the sentence stayed, which is a skip that had
# stopped being honest.
$published = 'absent'
$ref = ''
& git rev-parse -q --verify ("refs/heads/" + $branch) > $null 2>&1
if ($LASTEXITCODE -eq 0) { $published = 'local'; $ref = "refs/heads/$branch" }
else {
    & git rev-parse -q --verify ("refs/remotes/origin/" + $branch) > $null 2>&1
    if ($LASTEXITCODE -eq 0) { $published = 'remote'; $ref = "refs/remotes/origin/$branch" }
}

# ⭐ TWO GIT TREE OBJECTS, which is what "byte for byte" means for a branch: one
# tree object is one set of bytes, over every path and every mode. The
# regenerated tree goes into a TEMPORARY index, so the repository's own index is
# never touched.
$matched = $false
$pending = 0
if ($ref) {
    $indexFile = Join-Path $out 'compare.index'
    if (Test-Path -LiteralPath $indexFile) { Remove-Item -LiteralPath $indexFile -Force }
    $here = Get-Location
    try {
        Set-Location -LiteralPath (Join-Path $out 'a')
        $env:GIT_INDEX_FILE = $indexFile
        # ⛔ THE ARGUMENT IS BUILT AS ONE STRING FIRST. `--git-dir=(Join-Path
        # ...)` is not an interpolation: PowerShell passes `--git-dir=` and the
        # path as TWO arguments, so git gets an empty directory, warns about
        # `/config` and stages nothing. The empty tree 4b825dc6 is what that
        # looks like, and it compares unequal to everything, which is how this
        # was found.
        # ⚠ And no bare `--` separator, which PowerShell consumes before a
        # native command ever sees it.
        $gitDir = Join-Path $root '.git'
        & git "--git-dir=$gitDir" '--work-tree=.' add --all --force '.' > $null 2>&1
        $localTree = (& git write-tree 2>$null | Select-Object -First 1)
    }
    finally {
        Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue
        Set-Location -LiteralPath $here
    }
    $publishedTree = (& git rev-parse -q --verify ($ref + '^{tree}') 2>$null | Select-Object -First 1)
    if (-not $localTree -or -not $publishedTree) {
        [void]$problems.Add("  the $branch branch is $published and neither tree could be read, so nothing was compared")
    }
    elseif ($localTree.Trim() -eq $publishedTree.Trim()) {
        $matched = $true
        $matchedTree = $localTree.Trim()
    }
    else {
        # ⛔ BEHIND IS NOT THE SAME AS WRONG. The sh half carries the full
        # reasoning and the two cases it separates; this follows it.
        # docs/history/todo/publish.md, PUB-14.
        $a = Join-Path $out 'a'
        $publishedPaths = @(& git ls-tree -r --name-only $ref | ForEach-Object { $_.Trim() } | Sort-Object)
        $regenPaths = @(Get-ChildItem -LiteralPath $a -Recurse -File |
            ForEach-Object { $_.FullName.Substring($a.Length + 1).Replace('\', '/') } | Sort-Object)
        $gone = @($publishedPaths | Where-Object { $regenPaths -notcontains $_ }).Count
        $added = @($regenPaths | Where-Object { $publishedPaths -notcontains $_ }).Count

        # ⛔ THE BRANCH AGAINST ITS OWN MANIFEST. A deleted path leaves the
        # branch a smaller subset, which the tree comparison alone cannot tell
        # from 'not published yet'.
        $missing = 0
        $publishedManifest = Join-Path $out 'published-manifest.json'
        & git show ($ref + ':MANIFEST.json') > $publishedManifest 2>$null
        if ($LASTEXITCODE -eq 0 -and (Test-Path -LiteralPath $publishedManifest)) {
            $manifestPaths = @((Get-Content $publishedManifest -Raw | ConvertFrom-Json).artefacts |
                ForEach-Object { $_.path })
            $missing = @($manifestPaths | Where-Object { $publishedPaths -notcontains $_ }).Count
        } else {
            [void]$problems.Add("  the $branch branch carries no MANIFEST.json, so what it claims to publish cannot be read")
        }

        # Immutable is asked of the producer, never guessed from a path. The
        # regenerated manifest records derived per artefact: false for a
        # profile and its raw sidecar, true for every index, route, format
        # dump and generated config, which move whenever the corpus grows.
        # Exempting two files by name was wrong the moment a seventh profile
        # landed: adding one changed nineteen aggregates and this read as a
        # rewritten branch.
        $immutable = @($manifest.artefacts |
            Where-Object { -not $_.derived } | ForEach-Object { $_.path })
        if ($immutable.Count -eq 0) {
            [void]$problems.Add(
                "  the regenerated MANIFEST.json names no immutable artefact, so nothing could be compared")
        }

        $changed = 0
        $derivedChanged = 0
        $sumsLost = 0
        foreach ($rel in $publishedPaths) {
            $localFile = Join-Path $a $rel
            if (-not (Test-Path -LiteralPath $localFile)) { continue }
            $blob = Join-Path $out 'blob.tmp'
            & git show ($ref + ':' + $rel) > $blob 2>$null
            $same = $false
            if (Test-Path -LiteralPath $blob) {
                $same = @(Compare-Object -ReferenceObject ([System.IO.File]::ReadAllBytes($blob)) `
                    -DifferenceObject ([System.IO.File]::ReadAllBytes($localFile))).Count -eq 0
            }
            if (-not $same) {
                if ($immutable -contains $rel) { $changed++ }
                else { $derivedChanged++ }
            }
        }

        $publishedSums = Join-Path $out 'published-sums.txt'
        & git show ($ref + ':SHA256SUMS') > $publishedSums 2>$null
        if ((Test-Path -LiteralPath $publishedSums) -and (Test-Path -LiteralPath (Join-Path $a 'SHA256SUMS'))) {
            # Over the immutable paths only: an aggregate digest moves
            # whenever the corpus grows.
            $keep = {
                param($line)
                $name = $line -replace '^[0-9a-f]+\s+\*?', ''
                $immutable -contains $name
            }
            $ps = @(Get-Content $publishedSums | ForEach-Object { $_.TrimEnd() } |
                Where-Object { & $keep $_ })
            $rs = @(Get-Content (Join-Path $a 'SHA256SUMS') | ForEach-Object { $_.TrimEnd() } |
                Where-Object { & $keep $_ })
            $sumsLost = @($ps | Where-Object { $rs -notcontains $_ }).Count
        } else {
            $sumsLost = 1
        }

        if ($gone -eq 0 -and $changed -eq 0 -and $sumsLost -eq 0 -and $missing -eq 0 -and
            ($added -gt 0 -or $derivedChanged -gt 0)) {
            $pending = $added + $derivedChanged
        } else {
            [void]$problems.Add("  the regenerated tree is $($localTree.Trim()) and $ref carries $($publishedTree.Trim()), so what is published is not what this corpus derives to")
            if ($gone -ne 0) { [void]$problems.Add("  $gone published path(s) are no longer produced at all, which a consumer pinning them would notice") }
            if ($changed -ne 0) { [void]$problems.Add("  $changed published artefact(s) changed their bytes, and a published artefact is immutable") }
            if ($sumsLost -ne 0) { [void]$problems.Add("  $sumsLost checksum line(s) the branch publishes are not in the regenerated set") }
            if ($missing -ne 0) { [void]$problems.Add("  $missing path(s) the branch's own manifest lists are not on the branch, so a consumer fetching one gets a 404") }
        }
    }
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-data-branch/3","files":' + $files + ',"present":' + $present +
                  ',"recorded":' + $recorded + ',"cases":' + $cases +
                  ',"published":"' + $published + '","matched":' +
                  $matched.ToString().ToLowerInvariant() + ',"pending":' + $pending +
                  ',"problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output "data branch ok: $present file(s) regenerated, identical over two builds,"
    Write-Output "  $recorded of them with a checksum in the manifest and in SHA256SUMS, and no"
    Write-Output '  source, vendored dependency or reference corpus among them.'
    if ($matched) {
        Write-Output "  `u{2B50} The $branch branch is $published and its tree is $matchedTree, which is what this"
        Write-Output '  corpus derives to. One tree object is one set of bytes.'
    }
    elseif ($pending -gt 0) {
        Write-Output "  `u{26A0} The $branch branch is $published and BEHIND by $pending artefact(s): every path it"
        Write-Output '  carries is still produced and every immutable artefact is byte-identical.'
        Write-Output '  The assembler adds or refreshes derived artefacts only, so this is reported'
        Write-Output '  rather than failed. The publisher appends the new tree on the next push.'
    }
    else {
        Write-Output "  `u{26A0} A SKIP IS NOT A PASS: the $branch branch is $published, so the regenerated tree was"
        Write-Output '  compared against nothing. Push it once and this leg starts running.'
    }
    Write-Output "  `u{26D4} Nothing was pushed and no branch was created."
    exit 0
}

[Console]::Error.WriteLine("data branch check failed, $count problem(s):")
[Console]::Error.WriteLine('')
$problems | ForEach-Object { [Console]::Error.WriteLine($_) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('A consumer pinning a commit on this branch keeps working forever, and')
[Console]::Error.WriteLine('that property is free. docs/history/todo/publish.md, PUB-02.')
exit 1
