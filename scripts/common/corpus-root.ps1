# corpus-root.ps1 - where is the corpus this run should read?
#
# ⭐ THE TWIN OF corpus-root.sh. Same schema, same exit codes, same order.
# check-twins.sh is what stops the two drifting.
#
# ⛔ TWELVE CHECKS READ THE CORPUS AND EVERY ONE OF THEM ASSUMED THE WORKING
# TREE, which is leaving the default branch. TODO/publish.md, PUB-11.
#
# ⛔ THE ORDER: $env:B_IDS_CORPUS_ROOT if set, then the working tree if it holds
# corpus/v1/index.json, then a materialised copy of the data branch under
# .tmp/data-branch. ⚠ An explicit root is never second guessed: if it holds no
# corpus this exits 2 rather than falling through to something the caller did
# not ask for.
#
# ⚠ THE ENTRY PROPOSED THE BRANCH BEFORE THE WORKING TREE. That order is wrong
# while both exist: a session adding a profile would have every check read the
# published corpus and report green over the one it is about to publish.
#
# ⚠ MATERIALISED THROUGH A TEMPORARY INDEX, so no tar and no pipe are involved:
# PowerShell is not byte-exact through a native pipe, and the two tar builds
# this project meets disagree about flags. ⛔ It never touches this
# repository's own index and registers no worktree.
#
# ⭐ IT DOES NOT FETCH. refs/heads/data then refs/remotes/origin/data.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/corpus-root.ps1
#   pwsh -NoProfile -File scripts/common/corpus-root.ps1 -Json
#   pwsh -NoProfile -File scripts/common/corpus-root.ps1 -Ref
#   pwsh -NoProfile -File scripts/common/corpus-root.ps1 -Source
#   pwsh -NoProfile -File scripts/common/corpus-root.ps1 -Fixture
#
# Exit codes: 0 a root was resolved and its path is on stdout, 2 none was.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    [switch]$Fixture,
    [switch]$Ref,
    [switch]$Source,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1, which is what the POSIX twin does for the same input.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('corpus-root: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('corpus-root: git not found')
    exit 2
}
$repoRoot = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $repoRoot) {
    [Console]::Error.WriteLine('corpus-root: not a git repository')
    exit 2
}
$repoRoot = ($repoRoot | Select-Object -First 1).Trim()

$branch = 'data'
$mark = 'corpus/v1/index.json'

# ⭐ THE ONE TEST FOR "IS THERE A CORPUS HERE".
function Test-Corpus {
    param([string]$Root)
    if (-not $Root) { return $false }
    Test-Path -LiteralPath (Join-Path $Root $mark) -PathType Leaf
}

function Get-BranchRef {
    & git rev-parse -q --verify "refs/heads/$branch" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return "refs/heads/$branch" }
    & git rev-parse -q --verify "refs/remotes/origin/$branch" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return "refs/remotes/origin/$branch" }
    return ''
}

# ⛔ MATERIALISE THROUGH A TEMPORARY INDEX.
function Copy-BranchTree {
    param([string]$FromRef, [string]$Dest)
    $idx = $Dest + '.index'
    if (Test-Path -LiteralPath $Dest) { Remove-Item -LiteralPath $Dest -Recurse -Force }
    if (Test-Path -LiteralPath $idx) { Remove-Item -LiteralPath $idx -Force }
    New-Item -ItemType Directory -Path $Dest -Force | Out-Null
    $env:GIT_INDEX_FILE = $idx
    try {
        & git read-tree $FromRef 2>$null | Out-Null
        if ($LASTEXITCODE -ne 0) { return $false }
        & git checkout-index -a --prefix="$Dest/" 2>$null | Out-Null
        if ($LASTEXITCODE -ne 0) { return $false }
    }
    finally {
        Remove-Item Env:GIT_INDEX_FILE -ErrorAction SilentlyContinue
        if (Test-Path -LiteralPath $idx) { Remove-Item -LiteralPath $idx -Force }
    }
    return $true
}

function Resolve-CorpusRoot {
    param([string]$Root)
    $explicit = [Environment]::GetEnvironmentVariable('B_IDS_CORPUS_ROOT')
    if ($explicit) {
        if (Test-Corpus -Root $explicit) {
            return @{ Root = $explicit; Source = 'explicit'; Ref = '' }
        }
        [Console]::Error.WriteLine("corpus-root: B_IDS_CORPUS_ROOT=$explicit holds no $mark")
        return $null
    }
    if (Test-Corpus -Root $Root) {
        return @{ Root = $Root; Source = 'working-tree'; Ref = '' }
    }
    $ref = Get-BranchRef
    if (-not $ref) {
        [Console]::Error.WriteLine("corpus-root: no corpus in the working tree and no local ref for $branch")
        return $null
    }
    $dest = Join-Path (Join-Path $Root '.tmp') 'data-branch'
    if (-not (Test-Corpus -Root $dest)) {
        if (-not (Copy-BranchTree -FromRef $ref -Dest $dest)) {
            [Console]::Error.WriteLine("corpus-root: could not materialise $ref into $dest")
            return $null
        }
    }
    if (-not (Test-Corpus -Root $dest)) {
        [Console]::Error.WriteLine("corpus-root: $ref carries no $mark")
        return $null
    }
    return @{ Root = $dest; Source = 'data-branch'; Ref = $ref }
}

function Measure-Profile {
    param([string]$Root)
    $dir = Join-Path (Join-Path $Root 'corpus') 'v1'
    if (-not (Test-Path -LiteralPath $dir)) { return 0 }
    @(Get-ChildItem -LiteralPath $dir -Recurse -File -Filter '*.json' -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne 'index.json' -and $_.Name -ne 'latest.json' }).Count
}

if ($Fixture) {
    # ⛔ THE FALLBACK IS DRIVEN RATHER THAN REASONED ABOUT.
    #
    # ⚠ NAMED $branchRef RATHER THAN $ref, AND THAT IS NOT STYLE. PowerShell
    # variable names are case-insensitive, so a script-scope `$ref` IS the
    # `[switch]$Ref` parameter above, and assigning a string to it throws
    # "cannot convert System.String to SwitchParameter" before anything runs.
    # Found by running this flag. docs/conventions/shell.md.
    $branchRef = Get-BranchRef
    if (-not $branchRef) {
        [Console]::Error.WriteLine("corpus-root: no local ref for $branch, so the fallback cannot be driven")
        exit 2
    }
    $fix = Join-Path (Join-Path $repoRoot '.tmp') 'corpus-root-fixture-ps'
    if (Test-Path -LiteralPath $fix) { Remove-Item -LiteralPath $fix -Recurse -Force }
    New-Item -ItemType Directory -Path $fix -Force | Out-Null
    if (Test-Corpus -Root $fix) {
        [Console]::Error.WriteLine('corpus-root: the fixture tree already holds a corpus, so it proves nothing')
        exit 2
    }
    $saved = [Environment]::GetEnvironmentVariable('B_IDS_CORPUS_ROOT')
    Remove-Item Env:B_IDS_CORPUS_ROOT -ErrorAction SilentlyContinue
    try {
        $r = Resolve-CorpusRoot -Root $fix
    }
    finally {
        if ($saved) { $env:B_IDS_CORPUS_ROOT = $saved }
    }
    if (-not $r) {
        [Console]::Error.WriteLine('corpus-root: the fallback did not resolve')
        exit 2
    }
    if ($r.Source -ne 'data-branch') {
        [Console]::Error.WriteLine("corpus-root: the fixture resolved $($r.Source) where data-branch was expected")
        exit 2
    }
    $count = Measure-Profile -Root $r.Root
    Remove-Item -LiteralPath $fix -Recurse -Force -ErrorAction SilentlyContinue
    if ($count -lt 1) {
        [Console]::Error.WriteLine('corpus-root: the materialised branch carried no profile')
        exit 2
    }
    Write-Output "corpus-root fixture ok: a tree with no corpus resolves to the $branch branch,"
    Write-Output "carrying $count profile(s)."
    exit 0
}

$resolved = Resolve-CorpusRoot -Root $repoRoot
if (-not $resolved) { exit 2 }

# ⭐ THE REF THE ANSWER CAME FROM, empty for the working tree and for an explicit
# root. check-corpus asks for it because its own question is about a HISTORY:
# once the corpus is only on the data branch, the history to read is that
# branch's and not this repository's. TODO/publish.md, PUB-11.
if ($Ref) {
    Write-Output $resolved.Ref
    exit 0
}

# ⛔ WHICH OF THE THREE ANSWERED, WHICH IS NOT THE SAME QUESTION AS -Ref.
# -Ref is empty for TWO different reasons: the working tree answered, or the
# caller named a root explicitly. A check that reads an empty ref as "the
# working tree is canonical" is therefore wrong whenever B_IDS_CORPUS_ROOT is
# set, and check-data-branch exported that variable on the line ABOVE its own
# guard, which disarmed it. ⚠ It reported `data branch ok` while comparing the
# branch against itself. TODO/publish.md, PUB-11.
if ($Source) {
    Write-Output $resolved.Source
    exit 0
}

if ($Json) {
    Write-Output ('{"schema":"corpus-root/1","source":"' + $resolved.Source +
                  '","ref":"' + $resolved.Ref +
                  '","profiles":' + (Measure-Profile -Root $resolved.Root) + '}')
    exit 0
}

# ⛔ THE PATH ALONE ON STDOUT, with no trailing text, because every caller reads
# this through a command substitution.
Write-Output $resolved.Root
exit 0
