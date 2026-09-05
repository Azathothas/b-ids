# check-one-home.ps1 - reject duplicated long prose sentences outside history and imported sources.
#
# Run from the repository root. The POSIX and PowerShell forms must
# return equivalent results. A missing prerequisite is reported, not passed.
#
# Usage: scripts/common/check-one-home.ps1 [--json or -Json and documented options]
# Exit codes: 0 passed, 1 assertion failed, 2 could not run.
[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. `pwsh -File` reports a parameter-binding failure as 1, which is
    # this project's code for "it ran and the thing failed"; the POSIX twin
    # exits 2 for the same input. Measured across every pair 2026-09-02:
    # 22 of 22 disagreed. docs/history/todo/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-one-home: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ⚠ A CONSTANT, NOT A FLAG, for the reason check-markers gives about its own
# ceiling: a threshold anybody can raise from a command line gets raised
# instead of met.
$minWords = 12

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-one-home: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-one-home: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location $root
try {
    $tracked = @(& git ls-files 2>$null)
    $untracked = @(& git ls-files --others --exclude-standard 2>$null)
}
finally { Pop-Location }

# ⛔ NO QUOTED PATHSPEC HANDED TO GIT. The filter is applied here. See header.
# ⚠ -cnotmatch: PowerShell's default comparison is case-INSENSITIVE, and this
# trap has already made an exclusion in a sibling check swallow every finding.
# -- ⛔ THE REFERENCE CORPUS IS EXEMPT, AND ONLY FROM THIS CHECK'S SUBJECT ----
#
# `references/` holds other projects' trees, at named commits, as the evidence
# behind docs/reference-sweeps/findings.md. It is somebody else's writing, so
# this project's rules about how a document is written cannot apply to it, and a
# check that fails on a correct tree gets switched off within a week.
#
# ⭐ Every check exempts it, and each exemption was paid for separately: the
# prose checks because it is somebody else's writing, check-control-bytes because
# .gitattributes stores the corpus byte-exact as evidence, and check-no-secrets
# after every hit over the corpus was read once and recorded.
# ⛔ Keep this identical to the sh twin.
$files = @($tracked + $untracked |
    ForEach-Object { $_.Trim() } |
    Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) -and
        $_ -cmatch '\.md$' -and $_ -notmatch '^docs/history/' -and
        $_ -cnotmatch '^(references|vendor/[^/]+)/' } |
    Sort-Object -Unique)

if ($files.Count -lt 2) {
    [Console]::Error.WriteLine("check-one-home: only $($files.Count) file(s) in scope; nothing to compare")
    exit 2
}

$routers = @{ 'AGENTS.md' = $true }

function Get-SentenceList([string]$Text) {
    $sb = New-Object System.Text.StringBuilder
    $fence = $false
    foreach ($line in ($Text -split "`r?`n")) {
        if ($line -match '^[ \t]*```') { $fence = -not $fence; continue }
        if ($fence) { continue }
        if ($line -match '^[ \t]*\|') { continue }   # a table row is not a sentence
        if ($line -match '^[ \t]*#') { continue }    # nor is a heading
        $l = [regex]::Replace($line, '`[^`]*`', ' ')
        $l = [regex]::Replace($l, '\]\([^)]*\)', ' ')
        $l = $l -replace '\[', ' '
        [void]$sb.Append(' ').Append($l)
    }
    $out = New-Object System.Collections.ArrayList
    foreach ($part in [regex]::Split($sb.ToString(), '[.:!?]+[ \t]+')) {
        $s = ([regex]::Replace($part.ToLowerInvariant(), '[^a-z0-9 ]', ' '))
        $s = ([regex]::Replace($s, ' +', ' ')).Trim()
        if (-not $s) { continue }
        if (($s -split ' ').Count -lt $minWords) { continue }
        [void]$out.Add($s)
    }
    return $out
}

$seen = @{}
$nfiles = 0
foreach ($rel in $files) {
    $full = Join-Path $root $rel
    if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { continue }
    $nfiles++
    $text = [System.IO.File]::ReadAllText($full, [System.Text.UTF8Encoding]::new($false))
    foreach ($s in (Get-SentenceList $text)) {
        if (-not $seen.ContainsKey($s)) { $seen[$s] = New-Object System.Collections.Generic.HashSet[string] }
        [void]$seen[$s].Add($rel)
    }
}

if ($nfiles -lt 2) {
    [Console]::Error.WriteLine("check-one-home: only $nfiles file(s) readable; nothing to compare")
    exit 2
}

$dups = New-Object System.Collections.ArrayList
foreach ($k in $seen.Keys) {
    $where = $seen[$k]
    if ($where.Count -lt 2) { continue }
    $allRouters = $true
    foreach ($f in $where) { if (-not $routers.ContainsKey($f)) { $allRouters = $false } }
    if ($allRouters) { continue }
    [void]$dups.Add([pscustomobject]@{ Sentence = $k; Files = @($where | Sort-Object) })
}

$count = $dups.Count

if ($Json) {
    Write-Output ('{"schema":"check-one-home/1","problems":' + $count + ',"files":' + $nfiles + ',"min_words":' + $minWords + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -gt 0) {
    Write-Output ("one fact, one home: {0} sentence(s) appear in more than one document:" -f $count)
    Write-Output ''
    foreach ($d in $dups) {
        $shown = $d.Sentence
        if ($shown.Length -gt 88) { $shown = $shown.Substring(0, 88) }
        Write-Output ('  "' + $shown + '"')
        foreach ($f in $d.Files) { Write-Output ('      ' + $f) }
        Write-Output ''
    }
    Write-Output 'Keep the fact in the document that owns it and make the other a pointer.'
    Write-Output 'docs/conventions/prose.md, "one fact, one home".'
    exit 1
}

Write-Output ("one fact one home: {0} documents, no sentence of {1}+ words in two of them" -f $nfiles, $minWords)
exit 0
