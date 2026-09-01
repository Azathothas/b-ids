#Requires -Version 5.1
<#
.SYNOPSIS
  Does vendor/upstream.json still describe the vendored trees, and has upstream moved past what it
  records?

.DESCRIPTION
  The PowerShell twin of check-vendor.sh. Same subject, same JSON answer, same exit codes.

  The defect this exists to catch is a vendored tree nobody can reconcile. A tree with no recorded
  commit is a fork whose base is lost: the next release cannot be merged onto it, no patch can be
  said to be a diff from anything, and "is this still upstream's code" has no answer.

  WHAT IT ASSERTS, OFFLINE
    - the manifest parses and declares a schema version;
    - every upstream names a repository, a ref, a 40-hex base and an ISO 8601 UTC instant;
    - every upstream's directory exists and is not empty;
    - every EXCLUDED path is absent from that directory, so the exclude list stays true;
    - every crate the manifest names resolves to a Cargo.toml declaring that name;
    - every directory under vendor/ has a manifest entry, so a tree cannot be added silently;
    - every patches/NAME directory names an upstream, every patch in it names a file the tree
      still has, and patches/README.md carries a section naming that patch.

  TWO LEGS, AND ONLY ONE OF THEM IS IN THE GATE
  -Upstream fetches the recorded ref from the remote and reports whether it still resolves to the
  recorded base, and which newer release tags exist. It needs the network, and a gate that needs
  the network fails on a machine that has none.

  A MOVED REF IS REPORTED, NOT FOLLOWED. Reconciling a release is a reading, and
  docs/methodology/vendoring.md says what it owes.

  Exit codes: 0 consistent, 1 inconsistent, 2 could not run.
  Read the exit code from this process, unpiped.
#>
[CmdletBinding()]
param(
    [switch]$Json,
    [switch]$Upstream
)

$ErrorActionPreference = 'Stop'

function Get-RepoRoot {
    $r = & git rev-parse --show-toplevel 2>$null
    if ($LASTEXITCODE -ne 0 -or -not $r) { return $null }
    return ($r | Select-Object -First 1).Trim()
}

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-vendor: git not found')
    exit 2
}
$repoRoot = Get-RepoRoot
if (-not $repoRoot) {
    [Console]::Error.WriteLine('check-vendor: not a git repository')
    exit 2
}
Set-Location -LiteralPath $repoRoot

$manifestPath = 'vendor/upstream.json'
# NO MANIFEST IS EXIT 2, NOT EXIT 0. A tree that vendors nothing has neither broken these rules
# nor satisfied them, and reporting green over an absent file is how a check quietly stops
# applying. Keep this identical to the sh twin.
if (-not (Test-Path -LiteralPath $manifestPath)) {
    [Console]::Error.WriteLine("check-vendor: no manifest at $manifestPath")
    exit 2
}
try {
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
} catch {
    $null = $_
    [Console]::Error.WriteLine("check-vendor: $manifestPath does not parse")
    exit 2
}

$problems = New-Object System.Collections.Generic.List[string]
function Add-Problem([string]$Text) { $problems.Add('  ' + $Text) }

if (-not $manifest.schema_version) {
    Add-Problem "${manifestPath}: no schema_version. A positional format with no version mis-reads silently."
}

$upstreams = @($manifest.upstreams)

# ConvertFrom-Json turns a string that parses as a date into a [datetime], so vendored_at
# arrives as an object and renders in the local short format. Reading it back out of the RAW
# json is what keeps this half validating the same text the sh half validates through jq, and
# the twin comparison is what found it. JSON preserves order, so index i here is upstream i.
$rawManifest = Get-Content -LiteralPath $manifestPath -Raw
$rawStamps = @([regex]::Matches($rawManifest, '"vendored_at"\s*:\s*"([^"]*)"') |
    ForEach-Object { $_.Groups[1].Value })
$upstreamIndex = -1
$nCrates = 0
$nPatches = 0
$moved = 0

foreach ($u in $upstreams) {
    $upstreamIndex += 1
    $name = $u.name
    if ($u.repository -cnotmatch '^https://') {
        $shown = if ($u.repository) { $u.repository } else { 'none' }
        Add-Problem "${name}: repository is not an https URL: $shown"
    }
    if (-not $u.ref) { Add-Problem "${name}: no ref. Without one nothing can fetch the tree again." }
    if ($u.base -cnotmatch '^[0-9a-f]{40}$') {
        $shown = if ($u.base) { $u.base } else { 'none' }
        Add-Problem "${name}: base is not a 40-character commit: $shown"
    }
    $stamp = if ($upstreamIndex -lt $rawStamps.Count) { $rawStamps[$upstreamIndex] } else { '' }
    if ($stamp -cnotmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$') {
        $shown = if ($stamp) { $stamp } else { 'none' }
        Add-Problem "${name}: vendored_at is not ISO 8601 UTC: $shown"
    }

    $dir = $u.directory
    if (-not $dir) { Add-Problem "${name}: no directory"; continue }
    if (-not (Test-Path -LiteralPath $dir -PathType Container)) {
        Add-Problem "${name}: the manifest names $dir and it does not exist"
        continue
    }
    $anyFile = Get-ChildItem -LiteralPath $dir -Recurse -File -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $anyFile) { Add-Problem "${name}: $dir holds no file" }

    # An excluded path that is PRESENT means the tree is not what the manifest says it is, and
    # every reconciliation from here compares against the wrong set.
    foreach ($ex in @($u.exclude)) {
        if (-not $ex) { continue }
        if (Test-Path -LiteralPath (Join-Path $dir $ex)) {
            Add-Problem "${name}: $ex is listed as excluded and is present in $dir"
        }
    }

    # A crate the manifest names has to be a real package with that name, or the record points at
    # something nobody can depend on.
    if ($u.crates) {
        foreach ($p in $u.crates.PSObject.Properties) {
            $nCrates += 1
            $cratePath = Join-Path (Join-Path $dir $p.Value) 'Cargo.toml'
            if (-not (Test-Path -LiteralPath $cratePath -PathType Leaf)) {
                Add-Problem "${name}: crate $($p.Name) is recorded at $($p.Value) and there is no Cargo.toml there"
                continue
            }
            $declares = Select-String -LiteralPath $cratePath -Pattern ('^name\s*=\s*"' + [regex]::Escape($p.Name) + '"') -Quiet
            if (-not $declares) {
                $shown = ($cratePath -replace '\\', '/')
                Add-Problem "${name}: $shown does not declare name = ""$($p.Name)"""
            }
        }
    }
}

# Every tree under vendor/ has a record. A directory added without one is a fork with no base,
# which is the state this check exists to make impossible.
if (Test-Path -LiteralPath 'vendor' -PathType Container) {
    foreach ($d in (Get-ChildItem -LiteralPath 'vendor' -Directory -ErrorAction SilentlyContinue)) {
        $rel = 'vendor/' + $d.Name
        if (-not ($upstreams | Where-Object { $_.directory -ceq $rel })) {
            Add-Problem "$rel exists and no upstream in $manifestPath names it"
        }
    }
}

# The patch series is derived, so a patch naming a file the tree no longer has is a claim about
# the tree that is not true. This is the OFFLINE half of that question.
if (Test-Path -LiteralPath 'patches' -PathType Container) {
    $readmeExists = Test-Path -LiteralPath 'patches/README.md' -PathType Leaf
    $readme = if ($readmeExists) { Get-Content -LiteralPath 'patches/README.md' -Raw } else { '' }
    foreach ($d in (Get-ChildItem -LiteralPath 'patches' -Directory -ErrorAction SilentlyContinue)) {
        $pname = $d.Name
        $up = $upstreams | Where-Object { $_.name -ceq $pname } | Select-Object -First 1
        if (-not $up) {
            Add-Problem "patches/$pname has no upstream named $pname in $manifestPath"
            continue
        }
        foreach ($p in (Get-ChildItem -LiteralPath $d.FullName -Filter '*.patch' -File | Sort-Object Name)) {
            $nPatches += 1
            $shown = 'patches/' + $pname + '/' + $p.Name
            $target = $null
            foreach ($line in (Get-Content -LiteralPath $p.FullName)) {
                if ($line -cmatch '^\+\+\+ b/(.*)$') { $target = $Matches[1]; break }
            }
            if (-not $target) {
                Add-Problem "$shown names no target file"
            } elseif ($target -cne '/dev/null' -and -not (Test-Path -LiteralPath (Join-Path $up.directory $target))) {
                Add-Problem "$shown patches $target and $($up.directory)/$target does not exist"
            }
            if ($readmeExists) {
                if (-not $readme.Contains($p.Name)) {
                    Add-Problem "$shown has no section in patches/README.md saying what it is for"
                }
            } else {
                Add-Problem 'patches/README.md does not exist, so no local change has a reason recorded'
            }
        }
    }
}

# -- the network leg, which is not in the gate -------------------------------
if ($Upstream) {
    foreach ($u in $upstreams) {
        $name = $u.name
        # Dereferenced with ^{}, because an annotated tag resolves to the TAG OBJECT and comparing
        # that to a commit reports a move that did not happen.
        $lines = @(& git ls-remote $u.repository ("refs/tags/" + $u.ref + "^{}") ("refs/tags/" + $u.ref) ("refs/heads/" + $u.ref) 2>$null)
        $now = if ($lines.Count -gt 0) { ($lines[0] -split '\s+')[0] } else { '' }
        if (-not $now) {
            Write-Output ("upstream {0}: ref {1} does not resolve at {2}" -f $name, $u.ref, $u.repository)
            $moved += 1
        } elseif ($now -cne $u.base) {
            Write-Output ("upstream {0}: ref {1} now resolves to {2}, recorded {3}" -f $name, $u.ref, $now, $u.base)
            $moved += 1
        } else {
            Write-Output ("upstream {0}: ref {1} still resolves to the recorded base" -f $name, $u.ref)
        }

        $mineMatches = [regex]::Matches($u.ref, '[0-9]+\.[0-9]+\.[0-9]+')
        if ($mineMatches.Count -gt 0) {
            $mine = [version]$mineMatches[$mineMatches.Count - 1].Value
            # Sort-Object on strings orders 0.23.9 after 0.23.10, so the comparison is on parsed
            # version tuples rather than on text. docs/conventions/shell.md section 5.
            $tags = @(& git ls-remote --tags $u.repository 2>$null |
                ForEach-Object { ($_ -split '\s+')[1] } |
                ForEach-Object { $_ -replace '^refs/tags/', '' -replace '\^\{\}$', '' } |
                Sort-Object -Unique)
            $newer = @()
            foreach ($t in $tags) {
                if ($t -cnotmatch '[0-9]+\.[0-9]+\.[0-9]+$') { continue }
                $m = [regex]::Matches($t, '[0-9]+\.[0-9]+\.[0-9]+')
                $v = [version]$m[$m.Count - 1].Value
                if ($v -gt $mine) { $newer += [pscustomobject]@{ Tag = $t; Version = $v } }
            }
            if ($newer.Count -gt 0) {
                $newest = ($newer | Sort-Object Version | Select-Object -Last 1).Tag
                Write-Output ("upstream {0}: {1} newer release tag(s), newest {2}" -f $name, $newer.Count, $newest)
                $moved += 1
            } else {
                Write-Output ("upstream {0}: no newer release tag" -f $name)
            }
        }
    }
    Write-Output ''
}

# -- report ------------------------------------------------------------------
if ($Json) {
    Write-Output ('{"schema":"check-vendor/1","problems":' + $problems.Count +
                  ',"upstreams":' + $upstreams.Count +
                  ',"crates":' + $nCrates +
                  ',"patches":' + $nPatches +
                  ',"moved":' + $moved + '}')
    if ($problems.Count -gt 0) { exit 1 }
    exit 0
}

if ($problems.Count -gt 0) {
    Write-Output ("vendor check failed, " + $problems.Count + " problem(s):")
    Write-Output ''
    $problems | ForEach-Object { Write-Output $_ }
    Write-Output ''
    Write-Output 'The rules are in docs/methodology/vendoring.md. The manifest is'
    Write-Output 'vendor/upstream.json and the patch record is patches/README.md.'
    exit 1
}

Write-Output ("vendor ok: " + $upstreams.Count + " upstream(s), " + $nCrates + " crate(s), " +
              $nPatches + " patch(es), manifest agrees with the tree")
exit 0
