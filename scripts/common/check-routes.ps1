# check-routes.ps1 - does any published route file that carries exactly one
# value end with a newline?
#
# ⭐ THE TWIN OF check-routes.sh. Same schema, same exit codes, same rules.
# check-twins.sh is what stops the two drifting.
#
# The defect this exists to catch is a consumer having to strip something. A
# route a program reads with nothing but `curl` should hand back the value and
# nothing else; a trailing newline means every caller writes a strip, and the
# ones that forget compare a value against a value-plus-newline and get a
# mismatch they cannot see.
#
# ⭐ MEASURED ON THE REFERENCE THE REQUIREMENT CAME FROM. Two single-value files
# published by pkgforge-security/Wordlists each end with a newline, so the model
# this project is copying exhibits the defect the requirement forbids.
# docs/reference-sweeps/usable.md section 9.
#
# ⛔ IT REPORTS, IT DOES NOT STRIP. The generator is what gets fixed.
#
# ⚠ THIS TWIN EXISTS BECAUSE THE sh ONE CANNOT BE ASSUMED TO RUN HERE. A native
# PowerShell session may have no od, no tail and no sort at all.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-routes.ps1
#   pwsh -NoProfile -File scripts/common/check-routes.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-routes.ps1 -Fixtures DIR
#   pwsh -NoProfile -File scripts/common/check-routes.ps1 -AssertLatestIsStable
#
# Exit codes: 0 clean, 1 a route file ends with a newline, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding()]
param(
    [switch]$Json,
    [string]$Fixtures = '',
    [switch]$AssertLatestIsStable,
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2 rather
    # than 1. `pwsh -File` reports a parameter-binding failure as 1, which is
    # this project's code for "it ran and the thing failed"; the POSIX twin
    # exits 2 for the same input. Measured across every pair 2026-09-02:
    # 22 of 22 disagreed. docs/history/todo/ci.md, CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-routes: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-routes: git not found')
    exit 2
}
$root = (& git rev-parse --show-toplevel 2>$null)
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-routes: not a git repository')
    exit 2
}
$root = ($root | Select-Object -First 1).Trim()

Push-Location -LiteralPath $root

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.ps1 is the one answer to the question and this
# check does not carry a second one. docs/history/todo/publish.md, PUB-11.
$corpusRoot = (& pwsh -NoProfile -File (Join-Path $root 'scripts/common/corpus-root.ps1') | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or -not $corpusRoot) {
    [Console]::Error.WriteLine('check-routes: no corpus is reachable, so nothing was checked')
    exit 2
}
$corpusRoot = "$corpusRoot".Trim()
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable; a check that resolved a root and did not export it would build
# against one corpus and report on another.
$env:B_IDS_CORPUS_ROOT = $corpusRoot
try {
    # ⛔ THE PUBLISHED ROUTE TREES, named rather than "everything", and the list
    # is identical to the sh twin's. PUB-03 adds its generated tree to both in
    # the same change.
    $routeDirs = @('raw')
    # ⛔ The extensions this project defines as carrying one value.
    $singleValue = @('hex')

    $generated = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-routes-ps' | Join-Path -ChildPath 'routes'
    $manifest = Join-Path $generated 'routes.json'
    $generate = $true

    if ($Fixtures) {
        if (-not (Test-Path -LiteralPath $Fixtures -PathType Container)) {
            [Console]::Error.WriteLine("check-routes: no directory at $Fixtures")
            exit 2
        }
        $routeDirs = @($Fixtures)
        # ⚠ A fixture run checks the FIXTURE and nothing else.
        $generate = $false
    }

    if ($generate) {
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            [Console]::Error.WriteLine('check-routes: cargo not found, so the route tree could not be generated')
            exit 2
        }
        if (Test-Path -LiteralPath $generated) { Remove-Item -Recurse -Force -LiteralPath $generated }
        $work = Split-Path -Parent $generated
        New-Item -ItemType Directory -Force -Path $work | Out-Null
        & cargo build -q -p b-ids-corpus
        if ($LASTEXITCODE -ne 0) {
            [Console]::Error.WriteLine('check-routes: the corpus crate did not build')
            exit 2
        }
        $targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }
        $bin = Join-Path $targetDir 'debug' | Join-Path -ChildPath 'b-ids-corpus'
        if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
        if (-not (Test-Path -LiteralPath $bin)) {
            [Console]::Error.WriteLine("check-routes: $bin is not there")
            exit 2
        }
        $generateLog = Join-Path $work 'generate.log'
        & $bin routes --root $corpusRoot --out $generated > $generateLog 2>&1
        if ($LASTEXITCODE -ne 0) {
            [Console]::Error.WriteLine("check-routes: the route generator exited $LASTEXITCODE")
            Get-Content -LiteralPath $generateLog | ForEach-Object { [Console]::Error.WriteLine($_) }
            exit 1
        }
        $routeDirs = @('raw', '.tmp/check-routes-ps/routes')
    }

    $present = @($routeDirs | Where-Object { Test-Path -LiteralPath $_ -PathType Container })
    if ($present.Count -eq 0) {
        if ($Json) {
            Write-Output '{"schema":"check-routes/2","files":0,"verified":0,"problems":0,"routes":false}'
        }
        else {
            [Console]::Error.WriteLine('check-routes: no published route tree exists yet, so nothing was checked.')
        }
        exit 2
    }

    # ⚠ Tracked plus untracked-not-ignored, because a route file that has never
    # been staged is exactly the one a generator has just written wrongly.
    #
    # ⛔ A FIXTURE DIRECTORY IS WALKED WITH THE FILESYSTEM, NOT WITH GIT, and
    # this was a defect rather than a design. `git ls-files` refuses a path
    # outside the repository with a fatal on stderr and an empty list on stdout,
    # so both halves reported "ok, 0 files" over the fixture written to prove
    # they could refuse.
    $files = New-Object System.Collections.ArrayList
    if ($Fixtures) {
        foreach ($f in @(Get-ChildItem -LiteralPath $Fixtures -Recurse -File)) {
            [void]$files.Add($f.FullName)
        }
    }
    else {
        foreach ($dir in $routeDirs) {
            if (-not (Test-Path -LiteralPath $dir -PathType Container)) { continue }
            if ($dir -like '.tmp/*') {
                # ⛔ THE GENERATED TREE IS UNDER .tmp, WHICH IS IGNORED, so
                # `git ls-files --others --exclude-standard` answers with
                # NOTHING and the walk reports a clean tree it never opened.
                foreach ($f in @(Get-ChildItem -LiteralPath $dir -Recurse -File)) {
                    [void]$files.Add(($f.FullName.Substring($root.Length + 1) -replace '\\', '/'))
                }
                continue
            }
            foreach ($f in @(& git ls-files -- $dir)) { if ($f) { [void]$files.Add($f) } }
            foreach ($f in @(& git ls-files --others --exclude-standard -- $dir)) { if ($f) { [void]$files.Add($f) } }
        }
    }
    # ⛔ LC_ALL=C-equivalent uniqueness. Sort-Object -Unique compares
    # case-insensitively and would fold two files whose names differ only in
    # case into one, which scripts/README.md carries the measurement for.
    $unique = [System.Collections.Generic.HashSet[string]]::new([string[]]$files, [System.StringComparer]::Ordinal)

    $problems = New-Object System.Collections.ArrayList
    $checked = 0
    foreach ($file in ($unique | Sort-Object -CaseSensitive)) {
        # ⚠ THE WHOLE SUFFIX, not the last dot. A list file and a single-value
        # file both end in `txt`, and a classifier reading only the last dot
        # would call a list single-valued and refuse the newline a list needs.
        $base = [System.IO.Path]::GetFileName($file)
        $extension = [System.IO.Path]::GetExtension($file).TrimStart('.')
        $single = $singleValue -contains $extension
        if ($base -eq 'index.txt') { $single = $false }
        elseif ($base -like '*.list.txt') { $single = $false }
        elseif ($base -like '*.txt') { $single = $true }
        if (-not $single) { continue }
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { continue }
        $checked++
        # ⛔ The LAST BYTE, read from the file rather than from a line count. A
        # file of one line and a file of one line plus a newline both report one
        # line to anything that counts lines.
        $bytes = [System.IO.File]::ReadAllBytes($file)
        if ($bytes.Length -eq 0) { continue }
        $last = $bytes[$bytes.Length - 1]
        if ($last -eq 0x0a -or $last -eq 0x0d) {
            [void]$problems.Add("  ${file}: ends with a line ending, and it carries exactly one value")
        }
    }

    # -- ⭐ every route's value, read back out of the corpus ---------------
    #
    # ⛔ THIS IS THE LEG A GENERATOR CANNOT RUN ON ITSELF. The manifest names
    # the profile and the property behind every route, and this reads the value
    # out of the profile with ConvertFrom-Json. ⚠ THE READING IS THIS HALF'S
    # OWN, where the sh twin uses jq: two readings of the corpus rather than two
    # wrappers over one.
    $verified = 0
    if ($generate) {
        if (-not (Test-Path -LiteralPath $manifest) -or (Get-Item -LiteralPath $manifest).Length -eq 0) {
            [Console]::Error.WriteLine("check-routes: the generator wrote no manifest at $manifest")
            exit 2
        }
        $entries = (Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json).routes
        # ⚠ Each profile is parsed ONCE. Re-parsing per route would read one
        # file 54 times for six answers.
        $parsed = @{}
        foreach ($entry in $entries) {
            $verified++
            if (-not $parsed.ContainsKey($entry.profile)) {
                $parsed[$entry.profile] = Get-Content -LiteralPath $entry.profile -Raw | ConvertFrom-Json
            }
            $profileJson = $parsed[$entry.profile]
            $want = $null
            switch ($entry.property) {
                { $_ -in @('user-agent', 'sec-ch-ua', 'accept-language') } {
                    $set = @($profileJson.http.variants | Where-Object { $_.variant -eq $entry.variant })
                    if ($set.Count -gt 0) {
                        $header = @($set[0].headers | Where-Object { $_.name -eq $entry.property })
                        if ($header.Count -gt 0) { $want = $header[0].value }
                    }
                    break
                }
                'header-order' {
                    $set = @($profileJson.http.variants | Where-Object { $_.variant -eq $entry.variant })
                    if ($set.Count -gt 0) { $want = ($set[0].headers | ForEach-Object { $_.name }) -join "`n" }
                    break
                }
                'alpn' { $want = ($profileJson.tls.alpn) -join "`n"; break }
                'client-hello-hex' { $want = $profileJson.raw.client_hello_hex; break }
                default {
                    # ⛔ A property added to the generator with no reader here is
                    # a refusal rather than a skip.
                    [void]$problems.Add("  $($entry.path): the property $($entry.property) has no reader in this check")
                    continue
                }
            }
            $path = Join-Path $generated $entry.path
            $got = ''
            if (Test-Path -LiteralPath $path) {
                $got = [System.IO.File]::ReadAllText($path)
            }
            # ⚠ TRAILING LINE ENDINGS ARE STRIPPED ON BOTH SIDES, deliberately.
            # A list file ends with one and a single-value file does not; the
            # newline rule is the loop above and this leg is about the VALUE.
            if (("$want").TrimEnd("`r", "`n") -ne $got.TrimEnd("`r", "`n")) {
                [void]$problems.Add("  $($entry.path): the file is not what $($entry.profile) holds for $($entry.property)")
            }
            if ($entry.path -like '*/latest/*') {
                # ⛔ A CONSUMER FOLLOWING `latest` MUST NEVER BE HANDED A
                # PRE-RELEASE BUILD, and the route names the profile it came
                # from, so this asks the profile rather than the path.
                if ($profileJson.browser.channel -ne 'stable') {
                    [void]$problems.Add("  $($entry.path): latest names a $($profileJson.browser.channel) profile")
                }
            }
        }
        if ($verified -eq 0) {
            [Console]::Error.WriteLine('check-routes: the manifest named no route, so nothing was read back')
            exit 2
        }
    }

    # -- the latest pointer, on request -----------------------------------
    #
    # ⛔ DELEGATED, never re-implemented. `b-ids-corpus latest --assert-stable`
    # reads the pointer file on disk and the profiles it names, and its LAST
    # line is a fixed `corpus=latest problems:N`. A second reader of that file
    # in PowerShell would be a second answer to what `latest` may point at.
    if ($AssertLatestIsStable) {
        if ($Fixtures) {
            [Console]::Error.WriteLine('check-routes: -AssertLatestIsStable reads the corpus, so it cannot be')
            [Console]::Error.WriteLine('combined with -Fixtures.')
            exit 2
        }
        $latestRan = $false
        $latestProblems = 0
        $latestOut = @()
        if (Get-Command cargo -ErrorAction SilentlyContinue) {
            $latestOut = @(& cargo run -q -p b-ids-corpus -- latest --assert-stable --root $corpusRoot 2>&1)
            $latestRc = $LASTEXITCODE
            $line = @($latestOut | Where-Object { "$_" -like 'corpus=latest *' } | Select-Object -Last 1)
            if (($latestRc -eq 0 -or $latestRc -eq 1) -and $line.Count -gt 0) {
                $latestRan = $true
                if ("$($line[0])" -match 'problems:(\d+)') { $latestProblems = [int]$Matches[1] }
            }
        }
        if (-not $latestRan) {
            [Console]::Error.WriteLine('check-routes: the latest assertion did NOT run: cargo is absent, the')
            [Console]::Error.WriteLine('workspace did not build, or there is no corpus.')
            $latestOut | ForEach-Object { [Console]::Error.WriteLine("$_") }
            exit 2
        }
        if ($latestProblems -gt 0) {
            Write-Output "route check failed: $latestProblems latest pointer(s) do not name a stable profile."
            Write-Output ''
            $latestOut | ForEach-Object { Write-Output $_ }
            exit 1
        }
    }

    # ⛔ A ROUTE TREE THAT YIELDED NO SINGLE-VALUE FILE HAS VERIFIED NOTHING,
    # and reporting that as clean is how this check would quietly stop applying
    # the day a route type is renamed. Exit 2, for the same reason an absent
    # tree is.
    if ($checked -eq 0) {
        if ($Json) {
            Write-Output '{"schema":"check-routes/2","files":0,"verified":0,"problems":0,"routes":false}'
        }
        else {
            [Console]::Error.WriteLine('check-routes: the route tree holds no single-value file, so nothing')
            [Console]::Error.WriteLine('was checked. The extensions this project treats as single-valued are')
            [Console]::Error.WriteLine('in this script, beside the reason.')
        }
        exit 2
    }

    if ($Json) {
        Write-Output ('{"schema":"check-routes/2","files":' + $checked +
                      ',"verified":' + $verified +
                      ',"problems":' + $problems.Count + ',"routes":true}')
        if ($problems.Count -gt 0) { exit 1 }
        exit 0
    }

    if ($problems.Count -gt 0) {
        Write-Output ("route check failed, " + $problems.Count + " file(s):")
        Write-Output ''
        $problems | ForEach-Object { Write-Output $_ }
        Write-Output ''
        Write-Output 'A consumer of a single-value route should never have to strip anything.'
        Write-Output 'Fix the generator that wrote it, not the file.'
        exit 1
    }

    $suffix = if ($AssertLatestIsStable) { ', and every latest pointer names a stable profile' } else { '' }
    Write-Output "routes ok: $checked single-value file(s), none ends with a line ending,"
    Write-Output "  and $verified generated route(s) each carry the value the corpus holds$suffix"
    exit 0
}
finally {
    Pop-Location
}
