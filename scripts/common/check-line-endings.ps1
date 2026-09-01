# check-line-endings.ps1 - do the index AND the working tree carry the line
# endings this repository declares?
#
# ⭐ THE TWIN OF check-line-endings.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a file that is CRLF on disk in a tree that
# declares LF. The rule used to live inline in check-gate and it read git's
# INDEX column ALONE, so a tracked file that is CRLF in the working tree and LF
# in the index passed it. Eight files became CRLF that way in one session and
# the gate stayed green throughout. TODO/tooling.md, TOOL-17.
#
# ⛔ TWO COLUMNS, AND THEY ARE DIFFERENT FACTS. The index column says what a
# commit will contain; the working-tree column says what is on disk right now,
# which is what an editor, a compiler and Windows PowerShell 5.1 actually read.
#
# ⭐ THE RULE IS WHAT THE ATTRIBUTES DECLARE, never a fixed value. Measured
# 2026-09-01: 84 files here are CRLF on disk on purpose, and every one of them
# is a `.ps1` declaring `eol=crlf`, because 5.1 mis-parses a here-string whose
# terminator arrives with a bare LF.
# docs/conventions/shell.md section 8. A rule matching `*.ps1` would be a second
# answer to a question git already answers, and the reference corpus carries its
# own attributes files.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-line-endings.ps1
#   pwsh -NoProfile -File scripts/common/check-line-endings.ps1 -Json
#
# Exit codes: 0 clean, 1 a file disagrees with what it declares, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-line-endings: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-line-endings: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location -LiteralPath $root
try {
    $eol = @(& git ls-files --eol | Where-Object { "$_".Trim() -ne '' })
    if ($eol.Count -eq 0) {
        if ($Json) {
            Write-Output '{"schema":"check-line-endings/1","files":0,"index":0,"worktree":0,"problems":0}'
        }
        else {
            [Console]::Error.WriteLine('check-line-endings: git tracks no file here, so nothing was checked.')
        }
        exit 2
    }

    # ⛔ Keep this filter identical to the sh twin's awk program.
    $bad = @($eol | ForEach-Object {
            $cols = $_ -split '\s+'
            if ($cols.Count -lt 4) { return }
            $idx = $cols[0]
            $wt = $cols[1]
            if ($cols[2] -eq 'attr/-text') { return }
            if ($idx -eq 'i/-text' -or $wt -eq 'w/-text') { return }
            if ($idx -notin 'i/lf', 'i/none', 'i/') { "index    $_"; return }
            if ($wt -eq 'w/none') { return }
            if ($cols[3] -eq 'eol=crlf' -and $wt -ne 'w/crlf') { "worktree $_"; return }
            if ($cols[3] -eq 'eol=lf' -and $wt -ne 'w/lf') { "worktree $_"; return }
        })

    $indexBad = @($bad | Where-Object { "$_".StartsWith('index') }).Count
    $worktreeBad = @($bad | Where-Object { "$_".StartsWith('worktree') }).Count
    $problems = $indexBad + $worktreeBad

    if ($Json) {
        Write-Output ('{"schema":"check-line-endings/1","files":' + $eol.Count +
                      ',"index":' + $indexBad + ',"worktree":' + $worktreeBad +
                      ',"problems":' + $problems + '}')
        if ($problems -gt 0) { exit 1 }
        exit 0
    }

    if ($problems -gt 0) {
        Write-Output ("line-ending check failed, $problems file(s) over " + $eol.Count + ' tracked:')
        Write-Output ''
        $bad | ForEach-Object { Write-Output "  $_" }
        Write-Output ''
        Write-Output 'An "index" finding is what a commit would contain and is fixed by'
        Write-Output 'renormalising. A "worktree" finding is what is on disk and reaches no'
        Write-Output 'commit, which is exactly why nothing else notices it.'
        exit 1
    }

    Write-Output ('line endings ok: ' + $eol.Count + ' tracked file(s), index and working tree both agree')
    Write-Output 'with what .gitattributes declares.'
    exit 0
}
finally {
    Pop-Location
}
