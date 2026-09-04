# check-validate.ps1 - the two assertions a push has to settle about the
# published corpus, with no network and no browser.
#
# ⭐ THE TWIN OF check-validate.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a corpus that is STRUCTURALLY intact and
# INCOHERENT. check-corpus asks whether every profile sits at the route its keys
# derive, publishes the bytes it claims and was never edited after publication;
# every one of those can be true of a profile whose User-Agent says 151 and
# whose brand list says 152.
#
# -- ⭐ TWO LEGS, AND NEITHER IS check-corpus's ---------------------------------
#
#   1. COHERENCE, over what is published. Delegated to `b-ids-corpus validate`.
#      ⛔ A second enumeration of the corpus in PowerShell would be a second
#      answer to which profiles are published.
#
#   2. DETERMINISM of the derived files. The generator is run TWICE over a
#      throwaway copy and the two outputs are compared byte for byte.
#      ⚠ `b-ids-corpus verify` cannot see this class: it compares the committed
#      index against ONE derivation, so a generator that answered differently
#      on alternate runs would fail verify intermittently and read as a flake.
#
# ⚠ THIS TWIN EXISTS BECAUSE THE sh ONE CANNOT BE ASSUMED TO RUN HERE. A native
# PowerShell session may have no cmp, no cp and no mktemp at all.
#
# ⛔ THE BYTE COMPARISON IS BYTES, not text. Compare-Object over lines would
# report two files with different line endings as identical, which is exactly
# the difference a reproducible release turns on.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-validate.ps1
#   pwsh -NoProfile -File scripts/common/check-validate.ps1 -Json
#
# Exit codes: 0 clean, 1 a profile is incoherent or a generator is not
#             deterministic, 2 could not run.
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
    [Console]::Error.WriteLine('check-validate: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Absent {
    param([string]$Why)
    if ($Json) {
        Write-Output '{"schema":"check-validate/1","corpus":false,"profiles":0,"findings":0,"notcheckable":0,"deterministic":true,"problems":0}'
    }
    else {
        [Console]::Error.WriteLine("check-validate: $Why")
    }
    exit 2
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-validate: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-validate: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-validate: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
try {
    $corpusDir = 'corpus'
    $rawDir = 'raw'

    if (-not (Test-Path -LiteralPath (Join-Path $corpusRoot $corpusDir) -PathType Container)) {
        Write-Absent "there is no $corpusDir/ directory, so nothing was validated"
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Absent 'cargo is not on this host, so no profile was validated'
    }

    # -- leg one: is every published profile coherent ------------------------
    #
    # ⛔ THE NUMBERS COME FROM THE FIXED STATUS LINE, never from the prose above
    # it. `b-ids-corpus validate` prints `corpus=validate profiles:N findings:N
    # notcheckable:N` as its last line and its usage says that is the contract.
    $validateOut = @(& cargo run -q -p b-ids-corpus -- validate --root $corpusRoot 2>&1)
    $validateRc = $LASTEXITCODE
    if ($validateRc -ne 0 -and $validateRc -ne 1) {
        Write-Absent 'the coherence leg did not run. cargo is absent, the workspace did not build, or the corpus holds no profile'
    }
    $statusLine = @($validateOut | Where-Object { "$_" -like 'corpus=validate *' } | Select-Object -Last 1)
    if ($statusLine.Count -eq 0) {
        Write-Absent 'b-ids-corpus validate printed no status line, so nothing could be read from it'
    }
    $profiles = 0
    $findings = 0
    $notcheckable = 0
    if ("$($statusLine[0])" -match 'profiles:(\d+)') { $profiles = [int]$Matches[1] }
    if ("$($statusLine[0])" -match 'findings:(\d+)') { $findings = [int]$Matches[1] }
    if ("$($statusLine[0])" -match 'notcheckable:(\d+)') { $notcheckable = [int]$Matches[1] }

    # -- leg two: does the generator answer the same way twice ---------------
    #
    # ⛔ A THROWAWAY COPY, never the tree. The generator writes, so it is pointed
    # at a directory this check made and removes.
    $deterministic = $true
    $detail = @()
    $scratch = Join-Path ([System.IO.Path]::GetTempPath()) ("b-ids-check-validate." + $PID)
    if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force -Confirm:$false }
    $null = New-Item -ItemType Directory -Path (Join-Path $scratch 'root') -Force
    $null = New-Item -ItemType Directory -Path (Join-Path $scratch 'first') -Force
    try {
        Copy-Item -LiteralPath (Join-Path $corpusRoot $corpusDir) -Destination (Join-Path $scratch 'root') -Recurse -Force
        # ⛔ THE RAW BYTES COME FROM THE RESOLVED ROOT, NOT THE WORKING TREE.
        # This read $rawDir relative to the repository, so with the corpus
        # moved out it copied a corpus from the data branch and no raw bytes at
        # all. The generator reads the capture beside each profile, so its
        # first run then failed and this check reported that as a
        # NON-DETERMINISTIC GENERATOR. ⚠ The wrong verdict is the point: the
        # leg was not measuring what its message claimed.
        # TODO/publish.md, PUB-11.
        $rawSource = Join-Path $corpusRoot $rawDir
        if (Test-Path -LiteralPath $rawSource -PathType Container) {
            Copy-Item -LiteralPath $rawSource -Destination (Join-Path $scratch 'root') -Recurse -Force
        }
        $scratchRoot = Join-Path $scratch 'root'
        $derivedDir = Join-Path (Join-Path $scratchRoot $corpusDir) 'v1'

        $null = & cargo run -q -p b-ids-corpus -- index --write --root $scratchRoot 2>&1
        if ($LASTEXITCODE -ne 0) {
            $deterministic = $false
            $detail += "  the generator's first run failed"
        }
        else {
            foreach ($derived in @('index.json', 'latest.json')) {
                $written = Join-Path $derivedDir $derived
                if (Test-Path -LiteralPath $written -PathType Leaf) {
                    Copy-Item -LiteralPath $written -Destination (Join-Path (Join-Path $scratch 'first') $derived) -Force
                }
            }
            $null = & cargo run -q -p b-ids-corpus -- index --write --root $scratchRoot 2>&1
            if ($LASTEXITCODE -ne 0) {
                $deterministic = $false
                $detail += "  the generator's second run failed"
            }
            else {
                foreach ($derived in @('index.json', 'latest.json')) {
                    $first = Join-Path (Join-Path $scratch 'first') $derived
                    $second = Join-Path $derivedDir $derived
                    if (-not (Test-Path -LiteralPath $first -PathType Leaf) -or
                        -not (Test-Path -LiteralPath $second -PathType Leaf)) {
                        $deterministic = $false
                        $detail += "  ${derived}: one of the two runs did not write it"
                        continue
                    }
                    # ⛔ BYTES, not lines. Two files differing only in line endings
                    # are a real difference to a consumer fetching a route.
                    $a = [System.IO.File]::ReadAllBytes($first)
                    $b = [System.IO.File]::ReadAllBytes($second)
                    $same = $a.Length -eq $b.Length
                    if ($same) {
                        for ($i = 0; $i -lt $a.Length; $i++) {
                            if ($a[$i] -ne $b[$i]) { $same = $false; break }
                        }
                    }
                    if (-not $same) {
                        $deterministic = $false
                        $detail += "  ${derived}: two runs of the generator over one corpus wrote different bytes"
                    }
                }
            }
        }
    }
    finally {
        if (Test-Path -LiteralPath $scratch) { Remove-Item -LiteralPath $scratch -Recurse -Force -Confirm:$false }
    }

    # -- report --------------------------------------------------------------
    $problems = $findings
    if (-not $deterministic) { $problems = $problems + 1 }

    if ($Json) {
        $det = if ($deterministic) { 'true' } else { 'false' }
        Write-Output ('{"schema":"check-validate/1","corpus":true,"profiles":' + $profiles +
                      ',"findings":' + $findings + ',"notcheckable":' + $notcheckable +
                      ',"deterministic":' + $det + ',"problems":' + $problems + '}')
        if ($problems -gt 0) { exit 1 }
        exit 0
    }

    if ($findings -gt 0) {
        Write-Output "validate check failed: $findings finding(s) over $profiles published profile(s)."
        Write-Output ''
        $validateOut | ForEach-Object { Write-Output $_ }
    }
    if (-not $deterministic) {
        Write-Output 'validate check failed: the generator is not deterministic.'
        Write-Output ''
        $detail | ForEach-Object { Write-Output $_ }
        Write-Output ''
        Write-Output 'A release nobody can reproduce is a release whose every run looks like a'
        Write-Output 'change. Fix the generator, never this check.'
    }
    if ($problems -gt 0) { exit 1 }

    Write-Output "validate ok: $profiles profile(s) coherent, the generator answers the same way"
    Write-Output "twice, and $notcheckable check(s) reported they had nothing to read."
    exit 0
}
finally {
    Pop-Location
}
