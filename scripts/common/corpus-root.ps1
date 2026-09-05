# corpus-root.ps1 - where is the corpus this run should read?
#
# ⭐ THE TWIN OF corpus-root.sh. Same schema, same exit codes, same order.
# check-twins.sh is what stops the two drifting.
#
# ⛔ TWELVE CHECKS READ THE CORPUS AND EVERY ONE OF THEM ASSUMED THE WORKING
# TREE, which has now LEFT the default branch. docs/history/todo/publish.md, PUB-11 and
# PUB-13.
#
# ⛔ THE ORDER: $env:B_IDS_CORPUS_ROOT if set, then the working tree if it holds
# corpus/v1/index.json, then a materialised copy of the SOURCE branch under
# .tmp/source-branch, then one of the data branch under .tmp/data-branch.
# ⚠ An explicit root is never second guessed: if it holds no corpus this exits 2
# rather than falling through to something the caller did not ask for.
#
# ⚠ THE ENTRY PROPOSED A BRANCH BEFORE THE WORKING TREE. That order is wrong
# while both exist: a session adding a profile would have every check read the
# published corpus and report green over the one it is about to publish.
#
# ⛔ AND SOURCE BEFORE DATA IS THE WHOLE OF PUB-13. The data branch is DERIVED
# from the source branch, so a resolver answering `data-branch` first would hand
# check-data-branch the branch it is meant to be checking and the comparison
# would be against itself. That defect has shipped here once already.
#
# ⚠ MATERIALISED THROUGH A TEMPORARY INDEX, so no tar and no pipe are involved:
# PowerShell is not byte-exact through a native pipe, and the two tar builds
# this project meets disagree about flags. ⛔ It never touches this
# repository's own index and registers no worktree.
#
# ⛔ A COPY IS REUSED ONLY WHILE THE REF IT CAME FROM HAS NOT MOVED. The sha is
# stamped beside the copy and compared on every call. Reuse-on-presence alone
# was survivable while this was a rare route and is not now that it is the only
# one: a profile pushed to the source branch would otherwise be invisible to
# every check until somebody deleted .tmp by hand.
#
# ⭐ IT DOES NOT FETCH. refs/heads/NAME then refs/remotes/origin/NAME.
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

# ⛔ THE BRANCHES ARE NAMED ONCE, IN ORDER, and the sh half carries the same
# list. A rename moves both halves and nothing else here knows a branch name.
$branches = @('source', 'data')
$mark = 'corpus/v1/index.json'

# ⭐ THE ONE TEST FOR "IS THERE A CORPUS HERE".
function Test-Corpus {
    param([string]$Root)
    if (-not $Root) { return $false }
    Test-Path -LiteralPath (Join-Path $Root $mark) -PathType Leaf
}

function Get-BranchRef {
    param([string]$Name)
    & git rev-parse -q --verify "refs/heads/$Name" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return "refs/heads/$Name" }
    & git rev-parse -q --verify "refs/remotes/origin/$Name" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) { return "refs/remotes/origin/$Name" }
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

# ⛔ A CACHED COPY IS ONLY AS GOOD AS THE SHA IT CAME FROM.
function Get-BranchCopy {
    param([string]$FromRef, [string]$Dest)
    $want = (& git rev-parse -q --verify "$FromRef^{commit}" 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $want) { return $false }
    $want = ($want | Select-Object -First 1).Trim()
    $stamp = $Dest + '.ref'
    $have = ''
    if (Test-Path -LiteralPath $stamp -PathType Leaf) {
        $have = (Get-Content -LiteralPath $stamp -Raw -ErrorAction SilentlyContinue)
        if ($have) { $have = $have.Trim() }
    }
    if ($have -ne $want -or -not (Test-Corpus -Root $Dest)) {
        if (-not (Copy-BranchTree -FromRef $FromRef -Dest $Dest)) { return $false }
        Set-Content -LiteralPath $stamp -Value $want -NoNewline
    }
    return (Test-Corpus -Root $Dest)
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
    $tried = ''
    foreach ($branch in $branches) {
        $branchRef = Get-BranchRef -Name $branch
        if (-not $branchRef) {
            $tried += " $branch(no ref)"
            continue
        }
        $dest = Join-Path (Join-Path $Root '.tmp') "$branch-branch"
        if (Get-BranchCopy -FromRef $branchRef -Dest $dest) {
            return @{ Root = $dest; Source = "$branch-branch"; Ref = $branchRef }
        }
        $tried += " $branch(no $mark)"
    }
    [Console]::Error.WriteLine("corpus-root: no corpus in the working tree, and no branch carried one:$tried")
    return $null
}

function Measure-Profile {
    param([string]$Root)
    $dir = Join-Path (Join-Path $Root 'corpus') 'v1'
    if (-not (Test-Path -LiteralPath $dir)) { return 0 }
    @(Get-ChildItem -LiteralPath $dir -Recurse -File -Filter '*.json' -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne 'index.json' -and $_.Name -ne 'latest.json' }).Count
}

if ($Fixture) {
    # ⛔ THE FALLBACKS ARE DRIVEN RATHER THAN REASONED ABOUT, and since PUB-13
    # what this protects is the ORDER and the route the order HIDES.
    #
    # ⚠ NAMED $sourceRef RATHER THAN $ref, AND THAT IS NOT STYLE. PowerShell
    # variable names are case-insensitive, so a script-scope `$ref` IS the
    # `[switch]$Ref` parameter above, and assigning a string to it throws
    # "cannot convert System.String to SwitchParameter" before anything runs.
    # Found by running this flag. docs/conventions/shell.md.
    $sourceRef = Get-BranchRef -Name 'source'
    $dataRef = Get-BranchRef -Name 'data'
    if (-not $sourceRef) {
        [Console]::Error.WriteLine('corpus-root: no local ref for source, so the fallback cannot be driven')
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
    if ($r.Source -ne 'source-branch') {
        [Console]::Error.WriteLine("corpus-root: the fixture resolved $($r.Source) where source-branch was expected")
        exit 2
    }
    $count = Measure-Profile -Root $r.Root
    # ⛔ AND THE HIDDEN ROUTE, driven on its own. The resolver will never reach
    # it while the source branch answers, so nothing else here exercises it.
    $dataCount = 0
    if ($dataRef) {
        $dataDest = Join-Path (Join-Path $fix '.tmp') 'data-branch'
        if (Get-BranchCopy -FromRef $dataRef -Dest $dataDest) {
            $dataCount = Measure-Profile -Root $dataDest
        }
        else {
            Remove-Item -LiteralPath $fix -Recurse -Force -ErrorAction SilentlyContinue
            [Console]::Error.WriteLine('corpus-root: the data branch exists and could not be materialised')
            exit 2
        }
    }
    Remove-Item -LiteralPath $fix -Recurse -Force -ErrorAction SilentlyContinue
    if ($count -lt 1) {
        [Console]::Error.WriteLine('corpus-root: the materialised source branch carried no profile')
        exit 2
    }
    if ($dataRef -and $dataCount -lt 1) {
        [Console]::Error.WriteLine('corpus-root: the materialised data branch carried no profile')
        exit 2
    }
    Write-Output 'corpus-root fixture ok: a tree with no corpus resolves to the source branch,'
    if ($dataRef) {
        Write-Output "carrying $count profile(s). ⛔ The data branch is reachable and NOT chosen: it"
        Write-Output "materialises to $dataCount profile(s) when asked for directly."
    }
    else {
        Write-Output "carrying $count profile(s). ⛔ The data branch is reachable and NOT chosen: it"
        Write-Output 'has no local ref here, so only the order was proved.'
    }
    exit 0
}

$resolved = Resolve-CorpusRoot -Root $repoRoot
if (-not $resolved) { exit 2 }

# ⭐ THE REF THE ANSWER CAME FROM, empty for the working tree and for an explicit
# root. check-corpus asks for it because its own question is about a HISTORY:
# now that the corpus is only on a branch, the history to read is that branch's
# and not this repository's HEAD. docs/history/todo/publish.md, PUB-11 and PUB-13.
if ($Ref) {
    Write-Output $resolved.Ref
    exit 0
}

# ⛔ WHICH OF THE FOUR ANSWERED, WHICH IS NOT THE SAME QUESTION AS -Ref.
# -Ref is empty for TWO different reasons: the working tree answered, or the
# caller named a root explicitly. A check that reads an empty ref as "the
# working tree is canonical" is therefore wrong whenever B_IDS_CORPUS_ROOT is
# set, and check-data-branch exported that variable on the line ABOVE its own
# guard, which disarmed it. ⚠ It reported `data branch ok` while comparing the
# branch against itself. docs/history/todo/publish.md, PUB-11.
if ($Source) {
    Write-Output $resolved.Source
    exit 0
}

if ($Json) {
    Write-Output ('{"schema":"corpus-root/2","source":"' + $resolved.Source +
                  '","ref":"' + $resolved.Ref +
                  '","profiles":' + (Measure-Profile -Root $resolved.Root) + '}')
    exit 0
}

# ⛔ THE PATH ALONE ON STDOUT, with no trailing text, because every caller reads
# this through a command substitution.
Write-Output $resolved.Root
exit 0
