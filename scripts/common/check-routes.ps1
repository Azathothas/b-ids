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
    [switch]$AssertLatestIsStable
)

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
try {
    # ⛔ THE PUBLISHED ROUTE TREES, named rather than "everything", and the list
    # is identical to the sh twin's. PUB-03 adds its generated tree to both in
    # the same change.
    $routeDirs = @('raw')
    # ⛔ The extensions this project defines as carrying one value.
    $singleValue = @('hex')

    if ($Fixtures) {
        if (-not (Test-Path -LiteralPath $Fixtures -PathType Container)) {
            [Console]::Error.WriteLine("check-routes: no directory at $Fixtures")
            exit 2
        }
        $routeDirs = @($Fixtures)
    }

    $present = @($routeDirs | Where-Object { Test-Path -LiteralPath $_ -PathType Container })
    if ($present.Count -eq 0) {
        if ($Json) {
            Write-Output '{"schema":"check-routes/1","files":0,"problems":0,"routes":false}'
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
        $extension = [System.IO.Path]::GetExtension($file).TrimStart('.')
        if ($singleValue -notcontains $extension) { continue }
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
            $latestOut = @(& cargo run -q -p b-ids-corpus -- latest --assert-stable --root . 2>&1)
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
            Write-Output '{"schema":"check-routes/1","files":0,"problems":0,"routes":false}'
        }
        else {
            [Console]::Error.WriteLine('check-routes: the route tree holds no single-value file, so nothing')
            [Console]::Error.WriteLine('was checked. The extensions this project treats as single-valued are')
            [Console]::Error.WriteLine('in this script, beside the reason.')
        }
        exit 2
    }

    if ($Json) {
        Write-Output ('{"schema":"check-routes/1","files":' + $checked +
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
    Write-Output "routes ok: $checked single-value file(s), none ends with a line ending$suffix"
    exit 0
}
finally {
    Pop-Location
}
