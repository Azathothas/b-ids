# check-corpus.ps1 - is the published corpus still append-only, and does every
# profile in it still agree with itself?
#
# ⭐ THE TWIN OF check-corpus.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a corpus that was edited in place. A
# consumer who pinned a value then has no way to tell whether it changed, and a
# reader has no way to tell what it used to say, which is the whole difference
# between a corpus and a table somebody maintains. The premise is not this
# project's own: two published copies of one dataset, both carrying the same
# version number and both naming the same upstream, were measured holding a
# different number of entries. docs/reference-sweeps/usable.md section 9.
#
# -- ⭐ TWO LEGS, AND ONLY ONE OF THEM IS A QUESTION FOR THIS TREE -----------
#
# The working tree cannot answer whether a file was edited after it was
# published, because an edited file and a file that was always that way look
# identical. That question belongs to git and it is asked here, over the whole
# history, with no tool but git.
#
# Everything else is the same question `b-ids-corpus verify` answers, and it is
# delegated rather than re-implemented. ⛔ A second implementation of the layout
# rule in PowerShell would be a second answer to where a profile lives.
#
# ⚠ THIS TWIN EXISTS BECAUSE THE sh ONE CANNOT BE ASSUMED TO RUN HERE. A native
# PowerShell session may have no awk and no sed at all, and its `sort` is an
# alias for Sort-Object, which succeeds and answers differently.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-corpus.ps1
#   pwsh -NoProfile -File scripts/common/check-corpus.ps1 -Json
#
# Exit codes: 0 clean, 1 the corpus was edited or disagrees with itself,
#             2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. `pwsh -File` reports a parameter-binding failure as 1, which is
    # this project's code for "it ran and the thing failed"; the POSIX twin
    # exits 2 for the same input. Measured across every pair 2026-09-02:
    # 22 of 22 disagreed. TODO/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-corpus: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-corpus: git not found')
    exit 2
}
# ⛔ stdout alone. `git rev-parse` in a repository with no commits writes a
# fatal to stderr, and a version of this that merged the streams would put that
# fatal into a path. docs/conventions/shell.md section 3.
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-corpus: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

# ⛔ Every path below is relative to the repository root, so the scope of the
# check does not depend on who called it.
Push-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-corpus: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
# ⛔ AND THE REF THAT CARRIES IT. This check's one question is about a HISTORY
# rather than about files on disk: empty means the working tree, whose history
# is this repository's own, and a ref means the corpus lives on that branch.
$corpusRef = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') -Ref | Select-Object -First 1)
if ($null -eq $corpusRef) { $corpusRef = '' }
$corpusRef = "$corpusRef".Trim()
try {
    $corpusDir = 'corpus'
    $rawDir = 'raw'

    if (-not (Test-Path -LiteralPath (Join-Path $corpusRoot $corpusDir) -PathType Container)) {
        if ($Json) {
            Write-Output '{"schema":"check-corpus/2","corpus":false,"shallow":false,"profiles":0,"edits":0,"problems":0}'
        }
        else {
            [Console]::Error.WriteLine("check-corpus: there is no $corpusDir/ directory, so nothing was verified.")
            [Console]::Error.WriteLine('The corpus is empty. TODO/corpus.md, CORPUS-01.')
        }
        exit 2
    }

    # -- ⛔ A SHALLOW CLONE CANNOT ANSWER THE ONE QUESTION THIS CHECK OWNS ---
    #
    # `actions/checkout` fetches ONE COMMIT by default, so `git log` over the
    # corpus paths sees a single commit and `--diff-filter=MDR` finds nothing.
    # The append-only leg then reports clean having examined no history at all.
    #
    # ⚠ It is not hypothetical: this check ran inside the gate on both CI jobs
    # from the day it was written, under the default checkout depth, and its git
    # leg verified nothing on either. TODO/ci.md, CI-01.
    #
    # ⛔ EXIT 2, NOT 0. The corpus may be fine and this run cannot say so. ⛔ Keep
    # this identical to the sh twin.
    $shallow = (& git rev-parse --is-shallow-repository 2>$null)
    if ("$shallow".Trim() -eq 'true') {
        if ($Json) {
            Write-Output '{"schema":"check-corpus/2","corpus":true,"shallow":true,"profiles":0,"edits":0,"problems":0}'
        }
        else {
            [Console]::Error.WriteLine('check-corpus: this is a SHALLOW clone, so the history leg cannot run and')
            [Console]::Error.WriteLine('nothing was verified about whether a published file was ever edited.')
            [Console]::Error.WriteLine('Fetch the whole history: git fetch --unshallow, or fetch-depth: 0 on the')
            [Console]::Error.WriteLine('checkout step of the workflow that produced this tree.')
        }
        exit 2
    }

    # -- leg one: was anything ever edited or deleted after it was published --
    #
    # ⛔ OVER THE WHOLE HISTORY, not the working tree. An edited file and a file
    # that was always that way are identical on disk, so this is the one
    # question only the history can answer, and it is why this check exists.
    #
    # ⚠ M, D and R together. A modification is the obvious one; a deletion
    # breaks "never delete a superseded profile"; and a rename is a published
    # route changing under a consumer who pinned it.
    # ⛔ THE DERIVED FILES ARE EXCLUDED, AND THIS WAS A DEFECT RATHER THAN A
    # DESIGN. index.json and latest.json are regenerated from the tree every
    # time a profile is added, so they change by construction; a rule refusing
    # their modification would refuse the second profile this corpus ever gets.
    # It fired on exactly that.
    #
    # ⚠ NOTHING GOES UNCHECKED. Their CONTENT is asserted by the second leg,
    # which re-derives both from the profiles and compares. ⛔ Keep this
    # identical to the sh twin.
    $scope = @($corpusDir, $rawDir, ":(exclude)$corpusDir/*/index.json", ":(exclude)$corpusDir/*/latest.json")
    # ⚠ THE REF IS PREPENDED ONLY WHEN THERE IS ONE, which is `git log` over
    # HEAD when the corpus is the working tree and over the branch when it is
    # not. A default of HEAD written out would be a second spelling of it.
    $logArgs = @('log', '--diff-filter=MDR', '--name-status', '--format=commit %h')
    if ($corpusRef -ne '') { $logArgs += $corpusRef }
    $logArgs += '--'
    $logArgs += $scope
    $editLines = @(& git @logArgs 2>$null)
    $edits = @($editLines | Where-Object { $_ -and -not $_.StartsWith('commit ') })
    $editCount = $edits.Count

    # -- leg two: does every profile still agree with itself -----------------
    #
    # ⛔ THE NUMBERS COME FROM THE FIXED STATUS LINE, never from the prose above
    # it. `b-ids-corpus verify` prints `corpus=profiles:N problems:N` as its
    # last line and its usage says that is the contract. check-powershell.ps1
    # carries the same discipline for the same reason.
    $profiles = 0
    $problems = 0
    $verifyRan = $false
    $verifyOut = @()
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        $verifyOut = @(& cargo run -q -p b-ids-corpus -- verify --root $corpusRoot 2>&1)
        $verifyRc = $LASTEXITCODE
        $statusLine = @($verifyOut | Where-Object { "$_" -like 'corpus=*' } | Select-Object -Last 1)
        if (($verifyRc -eq 0 -or $verifyRc -eq 1) -and $statusLine.Count -gt 0) {
            $verifyRan = $true
            if ("$($statusLine[0])" -match 'profiles:(\d+)') { $profiles = [int]$Matches[1] }
            if ("$($statusLine[0])" -match 'problems:(\d+)') { $problems = [int]$Matches[1] }
        }
    }

    # -- report --------------------------------------------------------------
    if ($Json) {
        Write-Output ('{"schema":"check-corpus/2","corpus":true,"shallow":false,"profiles":' + $profiles +
                      ',"edits":' + $editCount + ',"problems":' + $problems + '}')
        if ($editCount -gt 0) { exit 1 }
        if ($problems -gt 0) { exit 1 }
        if (-not $verifyRan) { exit 2 }
        exit 0
    }

    if ($editCount -gt 0) {
        Write-Output ("corpus check failed: $editCount published file(s) modified, deleted or renamed after")
        Write-Output 'their first commit. A published profile is immutable; a correction is a NEW'
        Write-Output 'profile naming the one it replaces in `supersedes`.'
        Write-Output ''
        $editLines | ForEach-Object { Write-Output $_ }
        exit 1
    }

    if ($verifyRan -and $problems -gt 0) {
        Write-Output "corpus check failed: $problems problem(s) over $profiles profile(s)."
        Write-Output ''
        $verifyOut | ForEach-Object { Write-Output $_ }
        exit 1
    }

    if (-not $verifyRan) {
        Write-Output "check-corpus: the history is clean over $corpusDir and $rawDir, and nothing was"
        [Console]::Error.WriteLine('edited after it was published. ⚠ The per-profile leg did NOT run: cargo is')
        [Console]::Error.WriteLine('absent or the workspace did not build, so no profile was validated.')
        $verifyOut | ForEach-Object { [Console]::Error.WriteLine("$_") }
        exit 2
    }

    Write-Output "corpus ok: $profiles profile(s), nothing edited after publication, index and"
    Write-Output 'pointers agree with the tree.'
    exit 0
}
finally {
    Pop-Location
}
