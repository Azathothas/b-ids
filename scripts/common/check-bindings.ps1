# check-bindings.ps1 - does every other ecosystem's package answer identically to
# the Rust crate?
#
# ⭐ THE TWIN OF check-bindings.sh. Same schema, same exit codes, same legs.
# check-twins.sh is what stops the two drifting.
#
# ⛔ A REIMPLEMENTATION IN EACH LANGUAGE IS THE FAILURE TO AVOID. Four
# implementations of one selection rule is four places for it to be wrong, and
# the one that is wrong is the one nobody uses often enough to notice.
# TODO/library.md, LIB-03.
#
# ⭐ THE COMPARISON IS OVER THE ANSWERS RATHER THAN OVER THE INTERFACES, which
# is the entry's own wording, and it includes the case where a profile is
# ABSENT: two implementations agree easily on what exists.
#
# ⚠ THE RUNTIME IS THE SKIP. Without node there is nothing to run the other half
# with, and a skip is reported as a skip.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-bindings.ps1
#   pwsh -NoProfile -File scripts/common/check-bindings.ps1 -Json
#
# Exit codes: 0 every binding agrees, 1 one does not, 2 could not run.
#
# ⛔ Read $LASTEXITCODE from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1, which is what the POSIX twin does for the same input.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-bindings: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-bindings: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-bindings: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-bindings: cargo not found')
    exit 2
}

$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-bindings: no corpus is reachable, so nothing was compared')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
$env:B_IDS_CORPUS_ROOT = $corpusRoot

$asker = Join-Path $root 'scripts/fixtures/bindings-answers.mjs'
if (-not (Test-Path -LiteralPath $asker)) {
    [Console]::Error.WriteLine("check-bindings: no asker at $asker")
    exit 2
}
if (-not (Test-Path -LiteralPath (Join-Path $root 'crates/b-ids/examples/answers.rs'))) {
    [Console]::Error.WriteLine('check-bindings: the Rust crate has no answers example to compare against')
    exit 2
}

$problems = @()
$out = Join-Path (Join-Path $root '.tmp') 'check-bindings-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Recurse -Force }
New-Item -ItemType Directory -Path $out -Force | Out-Null

$tree = Join-Path $out 'tree'
& cargo run -q -p b-ids-corpus -- publish --root $corpusRoot --out $tree *> (Join-Path $out 'publish.log')
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-bindings: the assembler did not build the packages')
    exit 2
}

# ⛔ THE REFERENCE ANSWER, from the crate every binding is a binding OVER.
$rustText = & cargo run -q -p b-ids --example answers 2>(Join-Path $out 'rust.err')
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-bindings: the Rust crate did not answer')
    exit 2
}
$rustText = $rustText -join "`n"
Set-Content -LiteralPath (Join-Path $out 'rust.json') -Value $rustText -Encoding utf8NoBOM
$rust = $rustText | ConvertFrom-Json

$node = Get-Command node -CommandType Application -ErrorAction SilentlyContinue
$nodeState = if ($node) { 'present' } else { 'absent' }
$compared = 0

$jsDir = Join-Path (Join-Path $tree 'packages') 'js'
if ($node -and (Test-Path -LiteralPath $jsDir)) {
    Copy-Item -LiteralPath $asker -Destination $jsDir -Force
    Push-Location -LiteralPath $jsDir
    $jsText = & $node.Source 'bindings-answers.mjs' 2>(Join-Path $out 'js.err')
    $rc = $LASTEXITCODE
    Pop-Location
    if ($rc -ne 0) {
        $problems += 'the js package did not answer'
    }
    else {
        $compared++
        $jsText = $jsText -join "`n"
        Set-Content -LiteralPath (Join-Path $out 'js.json') -Value $jsText -Encoding utf8NoBOM
        $js = $jsText | ConvertFrom-Json
        # ⭐ COMPARED OVER THE ANSWERS, key by key, so a difference names the
        # rule that drifted rather than the file that differs.
        #
        # ⛔ NORMALISED WITH jq ON BOTH SIDES, and the first spelling of this
        # leg is why. It compared each value with ConvertTo-Json, which does not
        # sort the keys of a nested object: the two halves reported `release`
        # as differing over documents that were identical, because PowerShell
        # emitted its members in a different order from the JSON's own. ⚠ The sh
        # half never had that defect, because `jq -S` sorts. Two halves of one
        # check disagreeing about a tree neither of them changed is exactly the
        # drift check-twins exists to surface.
        $norm = {
            param([string]$Path)
            $text = & jq -S . $Path
            if ($LASTEXITCODE -ne 0) { return $null }
            ($text -join "`n") -replace "`r", ''
        }
        $rustNorm = & $norm (Join-Path $out 'rust.json')
        $jsNorm = & $norm (Join-Path $out 'js.json')
        if ($null -eq $rustNorm -or $null -eq $jsNorm) {
            $problems += 'one of the two answer documents is not JSON'
        }
        elseif ($rustNorm -ne $jsNorm) {
            $keys = @($rust.PSObject.Properties.Name) + @($js.PSObject.Properties.Name) |
                Sort-Object -Unique
            foreach ($key in $keys) {
                $a = ($rust.$key | ConvertTo-Json -Depth 12 -Compress)
                $b = ($js.$key | ConvertTo-Json -Depth 12 -Compress)
                if ($a -ne $b) {
                    $problems += "they disagree about $key"
                }
            }
            if ($problems.Count -eq 0) {
                $problems += 'the two answer documents differ and no single key does'
            }
        }
    }
}
elseif ($node) {
    $problems += 'the assembler wrote no packages/js, so there was nothing to compare'
}

# ⛔ AND THE ABSENT CASES ARE ACTUALLY IN THE ANSWER SET.
foreach ($want in @('at_missing', 'latest_chrome_macos', 'latest_safari_linux64')) {
    if (-not $rust.PSObject.Properties.Name.Contains($want)) {
        $problems += "the answer set does not ask $want, and LIB-03 names the absent case"
    }
    elseif ($null -ne $rust.$want) {
        $problems += "$want answered something, so it is not the absent case it is named for"
    }
}

Remove-Item Env:B_IDS_CORPUS_ROOT -ErrorAction SilentlyContinue

if ($Json) {
    Write-Output ('{"schema":"check-bindings/1","compared":' + $compared +
                  ',"node":"' + $nodeState +
                  '","problems":' + $problems.Count + '}')
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -eq 0) {
    Write-Output "bindings ok: $compared binding(s) compared against the Rust crate, answer for"
    Write-Output '  answer over one corpus, and the three absent cases came back empty on'
    Write-Output '  both sides. ⛔ The comparison is over the ANSWERS rather than over the'
    Write-Output '  interfaces.'
    if ($nodeState -eq 'absent') {
        Write-Output '  ⚠ SKIP: no node on this host, so the js package was not run. A skip is'
        Write-Output '  not a pass.'
    }
    exit 0
}

[Console]::Error.WriteLine("bindings check failed, $($problems.Count) problem(s):")
[Console]::Error.WriteLine('')
foreach ($p in $problems) { [Console]::Error.WriteLine('  ' + $p) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('Four implementations of one selection rule is four places for it to be')
[Console]::Error.WriteLine('wrong. TODO/library.md, LIB-03.')
exit 1
