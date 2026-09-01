# check-docs.ps1 - do the documents still resolve, and are they written the way
# this repository writes documents?
#
# ⭐ THE TWIN OF check-docs.sh. Same schema, same exit codes, same exemptions.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a document that was true when it was
# written. Three shapes of it, and every one is invisible to every other check:
#
#   - a link or a path that stopped resolving when something was renamed;
#   - a fenced shell block that does not parse, which is a block nobody can
#     copy and paste;
#   - an angle-bracket placeholder inside a shell block: a human reads it as
#     "fill this in" and bash reads it as a redirect, so the reader gets a
#     cryptic syntax error instead of an obvious instruction.
#
# ⚠ CONTROL BYTES ARE NOT CHECKED HERE. That rule scanned markdown only while
# every .ts, .py, .rs and .sh in the tree went unchecked, so it moved to
# check-control-bytes.ps1, which reads every text file. Run both.
#
# ⚠ THE CHARACTER HALF OF THE PROSE RULE IS NOT HERE. No em dash and no
# character outside the five belong to check-markers.ps1, which reads every
# tracked text file rather than markdown alone. Run both. What stays here is
# what is specific to a document: links, fenced blocks, placeholders, banned
# vocabulary and orphan pages.
#
# ⛔ WHAT IT DOES NOT CHECK IS WHETHER A CLAIM IS TRUE. That is a reading, and
# it belongs to the review pass. A guard that tried to verify prose would
# either pass vacuously or refuse legitimate writing, and both are worse than
# an honest scope.
#
# ⚠ THE SHELL-BLOCK PARSE NEEDS A POSIX SHELL, AND THIS HOST MAY NOT HAVE ONE.
# When no `sh` is on PATH the blocks are still COUNTED, so the schema matches
# the sh twin, and the parse rule is reported as SKIPPED on stderr rather than
# silently passing. ⛔ A rule that cannot run must say so: reporting green for
# a check that never executed is the failure this whole repository is built to
# avoid.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-docs.ps1
#   pwsh -NoProfile -File scripts/common/check-docs.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-docs.ps1 -Path docs
#
# Exit codes: 0 clean, 1 something is wrong, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    [string]$Path = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-docs: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-docs: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location $root
try {
    $tracked = @(& git ls-files 2>$null)
    $untracked = @(& git ls-files --others --exclude-standard 2>$null)
}
finally { Pop-Location }


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
$all = @($tracked + $untracked | ForEach-Object { $_.Trim() } |
    Where-Object { $_ -and $_ -cnotmatch '^(references|vendor/[^/]+)/' } | Sort-Object -Unique)
$files = @($all | Where-Object { $_ -match '\.md$' })
if ($Path) {
    $prefix = $Path.TrimEnd('/', '\').Replace('\', '/')
    $files = @($files | Where-Object { $_ -like "$prefix/*" -or $_ -eq $prefix })
}
if ($files.Count -eq 0) {
    [Console]::Error.WriteLine('check-docs: no markdown files in scope')
    exit 2
}

# THE BANNED VOCABULARY, AND WHY IT IS FOURTEEN WORDS RATHER THAN EIGHTEEN.
# docs/conventions/prose.md bans words that assert quality instead of
# demonstrating it. Fourteen of them are ALWAYS that, so a match is always a
# defect and a check can hold them.
#
# FOUR OF THE EIGHTEEN ARE DELIBERATELY NOT HERE: simply, just, obviously and
# "of course". They are banned as DISMISSALS, telling a reader who is stuck
# that what they cannot do is easy, and they are ordinary English in a
# contrast: "not just the names", "none is obviously right". Measured over
# this tree on 2026-08-31, before this check existed: 19 matches, every one of
# them legitimate, and 0 defects. A guard with a nineteen-to-nothing false
# positive rate is a guard somebody switches off, so those four stay a reading
# and prose.md says which half owns them.
#
# The list is the same in both halves, in the same order. check-twins compares
# the two implementations on the same tree.
$bannedWords = @(
    'seamless', 'blazing', 'effortless', 'robust',
    'powerful', 'cutting-edge', 'state-of-the-art',
    'world-class', 'elegant', 'revolutionary', 'game-changing',
    'rock-solid', 'bulletproof', 'lightning-fast'
)

$problems = New-Object System.Collections.ArrayList
$count = 0
$nlinks = 0
$nspans = 0
$nblocks = 0
function Add-Problem([string]$Text) {
    [void]$script:problems.Add('  ' + $Text)
    $script:count++
}
$script:problems = $problems
$script:count = 0

# A POSIX shell, if this host has one. See the header: absence is reported,
# never silently treated as a pass.
$shell = $null
foreach ($c in 'sh', 'bash') {
    $g = Get-Command $c -ErrorAction SilentlyContinue
    if ($g -and $g.CommandType -in 'Application', 'ExternalScript') { $shell = $g.Source; break }
}
$skippedParse = 0

# ⛔ NO FILE IS EXEMPT FROM THE LINK CHECK, and the exemption that used to be
# here was removed rather than emptied. It covered a template directory whose
# links are written relative to where the file will live in a PROJECT rather
# than where it sits in the tree. This repository is not a template and has no
# such directory: the one fill-in form it keeps, TODO/ENTRY.md, is written from
# where it lives, so its links resolve here. ⚠ An exemption for a path that
# does not exist is dead configuration, and the next file to land under that
# path would have inherited it silently.

$linked = New-Object System.Collections.Generic.HashSet[string]

# ⭐ THE TOP-LEVEL DIRECTORIES THIS REPOSITORY OWNS, read from git rather than
# written down, so a new one is covered without anybody remembering to add it.
# It is what scopes the cited-path check to this tree.
$roots = New-Object System.Collections.Generic.HashSet[string]
foreach ($tracked in (& git ls-files)) {
    $head = ($tracked -split '/')[0]
    if ($head -and $head -ne $tracked) { [void]$roots.Add($head) }
}

# The extensions a code span has to end in before it is treated as a path.
$pathExtensions = @(
    'md', 'sh', 'ps1', 'psm1', 'mjs', 'cjs', 'js', 'ts', 'rs', 'toml',
    'json', 'jsonc', 'yml', 'yaml', 'txt', 'py', 'go', 'lock', 'hex'
)

function Get-CitedPath([string]$Text) {
    <#
      ⭐ A CITED PATH IS CHECKED, NOT ONLY A LINK.

      A markdown link is resolved elsewhere in this file and a path written in
      a code span was not, which is how most of this tree names a file. Seven
      code spans named a licence filler, its twin and a directory of texts,
      none of which existed; every link resolved and this check was green
      throughout. TODO/tooling.md TOOL-10.

      ⛔ NARROW, AND IT REFUSES TO GUESS. A span is a path only when it holds a
      slash, ends in a known extension, has no whitespace, no angle bracket and
      no glob character, carries no scheme, has no ALL-CAPS segment, and starts
      at one of this repository's own top-level directories. Measured
      2026-08-31: without that last rule the check reported 30 spans and every
      one was legitimate, because the sweep documents cite paths INSIDE the
      reference trees as shorthand.
    #>
    $out = New-Object System.Collections.ArrayList
    $fence = $false
    $n = 0
    foreach ($line in ($Text -split "`r?`n")) {
        $n++
        if ($line -match '^[ \t]*```') { $fence = -not $fence; continue }
        if ($fence) { continue }
        foreach ($m in [regex]::Matches($line, '`([^`]*)`')) {
            $span = $m.Groups[1].Value
            if ($span -notmatch '/') { continue }
            if ($span -match '[\s<>*?]') { continue }
            if ($span -match '^[a-zA-Z][a-zA-Z0-9+.-]*://') { continue }
            $ext = ($span -split '\.')[-1]
            if ($pathExtensions -notcontains $ext) { continue }
            $segments = $span -split '/'
            if ($segments.Count -lt 2) { continue }
            $placeholder = $false
            foreach ($seg in $segments) { if ($seg -cmatch '^[A-Z0-9_]+$') { $placeholder = $true } }
            if ($placeholder) { continue }
            if (-not $roots.Contains($segments[0])) { continue }
            [void]$out.Add([pscustomobject]@{ Line = $n; Target = $span })
        }
    }
    return $out
}

function Get-LinkTarget([string]$Text) {
    # Strip fenced blocks, then code spans, then take every ](...) target.
    # ⚠ Stripping code spans is why a backticked expression is not reported as
    # a broken link. Markdown does not linkify a code span, and an earlier
    # version of this check reported exactly that as broken.
    $out = New-Object System.Collections.ArrayList
    $fence = $false
    $n = 0
    foreach ($line in ($Text -split "`r?`n")) {
        $n++
        if ($line -match '^[ \t]*```') { $fence = -not $fence; continue }
        if ($fence) { continue }
        $clean = [regex]::Replace($line, '`[^`]*`', '')
        foreach ($m in [regex]::Matches($clean, '\]\(([^)\s]+)')) {
            [void]$out.Add([pscustomobject]@{ Line = $n; Target = $m.Groups[1].Value })
        }
    }
    return $out
}

foreach ($rel in $files) {
    $full = Join-Path $root $rel
    if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { continue }
    $text = [System.IO.File]::ReadAllText($full)
    # ⛔ FORWARD SLASHES, ALWAYS. Split-Path returns a WINDOWS separator, so
    # `docs\conventions` has no `/` in it, and the `..` collapse below then
    # treats the whole thing as ONE segment: `docs\conventions/../../x`
    # collapsed to `../x`, which resolves outside the repository and reported
    # thirty-one correct links as broken. git speaks forward slashes and so
    # does every link in a markdown file; the only thing that did not was this
    # one call.
    $dir = (Split-Path -Parent $rel).Replace([char]92, '/')
    if (-not $dir) { $dir = '.' }

    # -- links ---------------------------------------------------------------
    foreach ($t in (Get-LinkTarget $text)) {
        $target = $t.Target
        if ($target -match '^(https?:|mailto:)' -or -not $target) { continue }
        # ⚠ COUNTED BEFORE THE EMPTY TEST, to match the sh twin exactly. A
        # pure-anchor link like the section links in this repository's own
        # documents has no path part, so it is counted as examined and then
        # skipped. Counting it after instead put the two implementations one
        # apart on a clean tree, which check-twins reports as drift and which
        # is a real disagreement about what the number means.
        $bare = ($target -split '#')[0]
        $script:nlinks++
        if (-not $bare) { continue }
        # Normalise to a repo-relative path so a link from a subdirectory and
        # one from the root name the same file.
        #
        # ⛔ THE FRAMEWORK RESOLVES THE '..', NOT A REGEX. The hand-rolled
        # collapse this replaces was `[^/]+/\.\./`, and `[^/]+` MATCHES '..'
        # ITSELF, so a link going up three levels ate its own segments:
        # `a/b/c/../../../docs/x` collapsed to `a/b/docs/x` and every correct
        # link from a directory three deep was reported broken. ⚠ In the tree
        # where it was found it stayed invisible until something WAS three deep,
        # which is the "a scope difference with nothing to exercise it is
        # invisible" lesson in scripts/README.md, arriving in the check rather
        # than in the thing being checked.
        # ⭐ The sh twin never had it: it asks the filesystem, with
        # `[ -e "$dir/$target" ]`. This now does the same thing.
        $joined = if ($dir -eq '.') { $bare } else { $dir + '/' + $bare }
        $abs = ''
        try { $abs = [IO.Path]::GetFullPath((Join-Path $root ($joined -replace '/', [IO.Path]::DirectorySeparatorChar))) }
        catch { $abs = '' }
        $rootFull = [IO.Path]::GetFullPath($root).TrimEnd([char]92, [char]47) + [IO.Path]::DirectorySeparatorChar
        if ($abs -and $abs.StartsWith($rootFull, [StringComparison]::OrdinalIgnoreCase)) {
            [void]$linked.Add(($abs.Substring($rootFull.Length) -replace '\\', '/'))
        }
        if (-not $abs -or -not (Test-Path -LiteralPath $abs)) {
            Add-Problem ($rel + ':' + $t.Line + ' broken link -> ' + $target)
        } else {
            # ON DISK IS NOT THE SAME AS IN THE REPOSITORY, and the difference
            # is invisible until somebody else clones. A link to a file this
            # tree does not commit resolves on the machine that wrote it and
            # 404s everywhere else, which is the green-locally red-in-CI shape.
            # Measured: a mined reference tree brought its OWN .gitignore, git
            # honoured it, and 92 files of a corpus this repository states it
            # keeps were on disk and in no commit. One of them was a primary
            # evidence artefact cited twice.
            $rp = if ($dir -eq '.') { $bare } else { $dir + '/' + $bare }
            git check-ignore -q -- $rp 2>$null
            if ($LASTEXITCODE -eq 0) {
                Add-Problem ($rel + ':' + $t.Line + ' link target is on disk and NOT COMMITTED -> ' + $target)
            }
        }
    }

    # -- cited paths, which a link check cannot see ---------------------------
    # ⚠ Resolved against the REPOSITORY ROOT and against the citing file's own
    # directory, and reported only when neither exists. Most of this tree
    # writes a root-relative path in prose and a directory-relative one in a
    # link, and refusing either would be refusing legitimate writing.
    foreach ($c in (Get-CitedPath $text)) {
        $script:nspans++
        $atRoot = Join-Path $root $c.Target
        $atFile = Join-Path (Split-Path -Parent $full) $c.Target
        if (-not (Test-Path -LiteralPath $atRoot) -and -not (Test-Path -LiteralPath $atFile)) {
            Add-Problem ($rel + ':' + $c.Line + ' cited path does not exist -> ' + $c.Target)
        }
    }

    # -- banned vocabulary ---------------------------------------------------
    # A specimen inside a fenced block or a code span is permitted, and it has
    # to be: a page that bans a word cannot otherwise show which word it means.
    $fence = $false
    $lineNo = 0
    foreach ($line in ($text -split "`r?`n")) {
        $lineNo++
        if ($line -match '^[ 	]*```') { $fence = -not $fence; continue }
        if ($fence) { continue }
        $low = ([regex]::Replace($line, '`[^`]*`', '')).ToLowerInvariant()
        foreach ($w in $bannedWords) {
            if ($low -match ('(^|[^a-z-])' + [regex]::Escape($w) + '([^a-z-]|$)')) {
                Add-Problem ($rel + ':' + $lineNo + ' banned vocabulary: ' + $w + '. docs/conventions/prose.md')
            }
        }
    }
}

# -- fenced shell blocks -----------------------------------------------------
foreach ($rel in $files) {
    $full = Join-Path $root $rel
    if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { continue }
    $lines = [System.IO.File]::ReadAllText($full) -split "`r?`n"
    $inBlock = $false
    $start = 0
    $buf = New-Object System.Collections.ArrayList
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        if (-not $inBlock -and $line -match '^[ \t]*```(bash|sh)[ \t]*$') {
            $inBlock = $true; $start = $i + 1; [void]$buf.Clear(); continue
        }
        if ($inBlock -and $line -match '^[ \t]*```') {
            $inBlock = $false
            $nblocks++
            $body = ($buf -join "`n")

            if ($body -match '<[a-z][a-z0-9-]*>') {
                Add-Problem ($rel + ':' + $start + ' shell-unsafe placeholder. bash reads it as a redirect; use UPPER_SNAKE')
            }

            if ($shell) {
                # ⛔ A TEMP FILE, NOT stdin. docs/conventions/shell.md: from
                # PowerShell a native command's stdin is not byte-exact, and a
                # trailing CRLF gets appended. For a syntax check that is the
                # difference between a real answer and a fabricated one.
                $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ('checkdocs-' + [guid]::NewGuid().ToString('N') + '.sh')
                try {
                    [System.IO.File]::WriteAllText($tmp, ($body -replace "`r", '') + "`n")
                    $prev = $ErrorActionPreference
                    $ErrorActionPreference = 'Continue'
                    try { & $shell -n $tmp 2>$null | Out-Null } finally { $ErrorActionPreference = $prev }
                    if ($LASTEXITCODE -ne 0) {
                        Add-Problem ($rel + ':' + $start + ' shell block does not parse')
                    }
                }
                finally { Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue }
            }
            else { $skippedParse++ }
            continue
        }
        if ($inBlock) { [void]$buf.Add($line) }
    }
}

# -- a page nothing links to -------------------------------------------------
# ⛔ AN UNLINKED PAGE IS NOT READ, SO IT IS NOT CORRECTED, and that is the state
# every stale document passes through on the way to being wrong.
# Roots are exempt: a README is an entry point, and the files at the repository
# root are what a reader or a raw URL arrives at directly.
foreach ($rel in $files) {
    if ($rel -match '(^|/)README\.md$') { continue }
    if ($rel -notmatch '/') { continue }
    if (-not $linked.Contains($rel)) {
        Add-Problem ($rel + ' is linked from nowhere. An unlinked page is not read, so it is not corrected.')
    }
}

# -- the character rule moved, it was NOT dropped -------------------------
# ⛔ THE FIVE-CHARACTER ALLOWLIST AND THE EM-DASH RULE NOW LIVE IN
# check-markers.ps1, over EVERY tracked text file rather than over markdown
# alone. Two checks enforcing one rule is two places for it to be wrong, and
# these two would have been wrong differently: this one strips fenced blocks
# and code spans before it looks and a whole-tree scan that did not would
# refuse the page that names the character it bans.
#
# ⚠ It is the same move the control-byte rule made out of this file, for
# the same reason. ⛔ Run both: this one for documents, that one for the
# whole tree.

$count = $script:count

if ($Json) {
    Write-Output ('{"schema":"check-docs/1","problems":' + $count + ',"files":' + $files.Count + ',"links":' + $nlinks + ',"cited_paths":' + $nspans + ',"shell_blocks":' + $nblocks + '}')
    if ($skippedParse -gt 0) {
        [Console]::Error.WriteLine('⚠ no POSIX shell on PATH: ' + $skippedParse + ' shell block(s) counted but NOT parsed')
    }
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -gt 0) {
    Write-Output ('documentation check failed, ' + $count + ' problem(s):')
    Write-Output ''
    $problems | ForEach-Object { Write-Output $_ }
    Write-Output ''
    if ($skippedParse -gt 0) {
        Write-Output ('⚠ no POSIX shell on PATH: ' + $skippedParse + ' shell block(s) counted but NOT parsed')
    }
    exit 1
}

Write-Output ('docs ok: ' + $files.Count + ' files, ' + $nlinks + ' relative links, ' + $nspans + ' cited paths, ' + $nblocks + ' shell blocks. Links, paths and prose clean.')
if ($skippedParse -gt 0) {
    Write-Output ('⚠ no POSIX shell on PATH: ' + $skippedParse + ' shell block(s) counted but NOT parsed')
}
exit 0
