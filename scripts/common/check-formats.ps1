# check-formats.ps1 - does every published format come out of the one generator,
# round-trip, and produce the same bytes twice?
#
# ⭐ THE TWIN OF check-formats.sh. TODO/schema.md, SCHEMA-08 and SCHEMA-12, and
# TODO/driver.md, DRIVER-09, is why a script in this directory does not land
# without one.
#
# ⛔ JSON IS ONE CONSUMER, NOT THE CONSUMER. A corpus reachable only by writing a
# JSON walker is a corpus most people copy values out of by hand, and a value
# copied by hand stops matching the day the build moves.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. every format regenerates from the canonical corpus;
#   2. TWO RUNS ARE BYTE-IDENTICAL. A generator that read a clock or a hash seed
#      would produce a diff on every run, and a published artefact that diffs on
#      every run is one nobody can tell a real change from;
#   3. the lossless formats round-trip to byte-identical canonical JSON, which
#      is the half a writer alone cannot prove;
#   4. the partial ones carry the documented subset and say in their own header
#      what they leave out;
#   5. ⭐ every format the SUPPORT MATRIX names has a file, and every format it
#      records as DECLINED has none. The matrix is generated from the generator,
#      so this reads the catalogue rather than a second copy of the list;
#   6. ⭐ the SQLite dump loads into a real database, where sqlite3 is here. That
#      is the one reader in this check that is not this project's own.
#
# ⛔ NEVER HAND-EDIT A GENERATED FORMAT. If one is ever edited directly the
# generator has lost, and this is what says so.
#
# Usage:
#   pwsh -NoProfile -File scripts/common/check-formats.ps1
#   pwsh -NoProfile -File scripts/common/check-formats.ps1 -Json
#   pwsh -NoProfile -File scripts/common/check-formats.ps1 -RequireRows yaml,toml
#
# Exit codes: 0 every format round-trips, 1 one did not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

[CmdletBinding(PositionalBinding = $false)]
param(
    [switch]$Json,
    [string]$RequireRows = '',
    # ⛔ EVERY UNBOUND ARGUMENT LANDS HERE, so an unknown one exits 2. CI-07.
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$UnboundArguments = @()
)

if ($UnboundArguments.Count -gt 0) {
    [Console]::Error.WriteLine('check-formats: unknown argument: ' + $UnboundArguments[0])
    exit 2
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-formats: git not found')
    exit 2
}
$root = & git rev-parse --show-toplevel 2>$null
if ($LASTEXITCODE -ne 0 -or -not $root) {
    [Console]::Error.WriteLine('check-formats: not a git repository')
    exit 2
}
Set-Location -LiteralPath $root
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine('check-formats: cargo not found')
    exit 2
}

# ⛔ 2, not 1. A tree with no corpus has verified nothing about the generator.
if (-not (Test-Path -LiteralPath (Join-Path $root 'corpus'))) {
    [Console]::Error.WriteLine('check-formats: there is no corpus under ' + $root + ', so there is nothing to generate')
    exit 2
}

$out = Join-Path $root '.tmp' | Join-Path -ChildPath 'check-formats-ps'
if (Test-Path -LiteralPath $out) { Remove-Item -Recurse -Force -LiteralPath $out }
New-Item -ItemType Directory -Force -Path (Join-Path $out 'a') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $out 'b') | Out-Null

& cargo build -q -p b-ids-corpus
if ($LASTEXITCODE -ne 0) {
    [Console]::Error.WriteLine('check-formats: the corpus crate did not build')
    exit 2
}
$bin = Join-Path $root 'target' | Join-Path -ChildPath 'debug' | Join-Path -ChildPath 'b-ids-corpus'
if (-not (Test-Path -LiteralPath $bin)) { $bin = $bin + '.exe' }
if (-not (Test-Path -LiteralPath $bin)) {
    [Console]::Error.WriteLine('check-formats: ' + $bin + ' is not there')
    exit 2
}

$problems = @()

# -- 1 and 2: generate twice, and compare the bytes --------------------------
$logA = Join-Path $out 'a.log'
$logB = Join-Path $out 'b.log'
& $bin formats --root $root --out (Join-Path $out 'a') > $logA 2>&1
$rcA = $LASTEXITCODE
& $bin formats --root $root --out (Join-Path $out 'b') > $logB 2>&1
$rcB = $LASTEXITCODE
if ($rcA -ne 0 -or $rcB -ne 0) {
    [Console]::Error.WriteLine('check-formats: the generator exited ' + $rcA + ' then ' + $rcB)
    Get-Content -LiteralPath $logA | ForEach-Object { [Console]::Error.WriteLine($_) }
    exit 1
}

# ⛔ THE FIXED LAST LINE, never the prose above it.
$status = (Get-Content -LiteralPath $logA | Where-Object { $_ -like 'corpus=formats *' } | Select-Object -Last 1)
if (-not $status) {
    [Console]::Error.WriteLine('check-formats: the generator printed no status line')
    exit 1
}
$files = [regex]::Match($status, 'files:(\d+)').Groups[1].Value
$profiles = [regex]::Match($status, 'profiles:(\d+)').Groups[1].Value

foreach ($file in (Get-ChildItem -LiteralPath (Join-Path $out 'a') -File)) {
    $other = Join-Path $out 'b' | Join-Path -ChildPath $file.Name
    $left = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    $right = (Get-FileHash -LiteralPath $other -Algorithm SHA256).Hash
    if ($left -ne $right) {
        $problems += ('  ' + $file.Name + ': two runs of the generator differ, so it is not deterministic')
    }
}

# -- 3 and 4: the round trips, which are the suite's ------------------------
#
# ⛔ THE READERS ARE THE CRATE'S AND SO ARE THE ASSERTIONS. A round trip written
# here would be a second reader of nine formats, disagreeing with the one the
# crate publishes the first time either moved.
& cargo test -q -p b-ids-corpus --test formats > (Join-Path $out 'tests.log') 2>&1
if ($LASTEXITCODE -ne 0) {
    $problems += '  the round-trip suite failed. Its output is in .tmp/check-formats-ps/tests.log'
}

# -- 5: the support matrix is the catalogue, and this opens what it names ----
#
# ⭐ THE LIST IS NOT WRITTEN HERE. formats.md is generated from the generator's
# own vocabulary, so a format added, renamed or declined moves it in the same
# change and this check follows without being edited.
$matrix = Join-Path $out 'a' | Join-Path -ChildPath 'formats.md'
if (-not (Test-Path -LiteralPath $matrix) -or (Get-Item -LiteralPath $matrix).Length -eq 0) {
    $problems += '  formats.md was not generated, or is empty'
}
$publishedRows = @{}
$declinedRows = @{}
$section = ''
foreach ($line in (Get-Content -LiteralPath $matrix)) {
    if ($line -like '## Published*') { $section = 'published'; continue }
    if ($line -like '## Declined*') { $section = 'declined'; continue }
    if ($line -notlike '| ``*') { continue }
    $cells = ($line -replace '`', '') -split '\|'
    $name = $cells[1].Trim()
    if ($section -eq 'published') {
        $publishedRows[$name] = @{ file = $cells[2].Trim(); carries = $cells[3].Trim() }
    }
    elseif ($section -eq 'declined') {
        $declinedRows[$name] = $cells[2].Trim()
    }
}

foreach ($name in $publishedRows.Keys) {
    $file = $publishedRows[$name].file
    $path = Join-Path $out 'a' | Join-Path -ChildPath $file
    if (-not (Test-Path -LiteralPath $path) -or (Get-Item -LiteralPath $path).Length -eq 0) {
        $problems += ('  ' + $name + ': the matrix names ' + $file + ' and it was not generated, or is empty')
    }
    if (-not $publishedRows[$name].carries) {
        $problems += ('  ' + $name + ': the matrix does not say what it carries')
    }
}
foreach ($name in $declinedRows.Keys) {
    # ⛔ BOTH HALVES OF A DECLINED FORMAT. Absent from the output, and named
    # with a reason. Either alone is a consumer guessing.
    foreach ($spill in @(('corpus.' + $name), $name)) {
        $path = Join-Path $out 'a' | Join-Path -ChildPath $spill
        if (Test-Path -LiteralPath $path) {
            $problems += ('  ' + $name + ' is recorded as declined and ' + $path + ' was generated')
        }
    }
    if ($declinedRows[$name].Length -le 40) {
        $problems += ('  ' + $name + ' is declined with no reason worth reading')
    }
}
$published = $publishedRows.Count
$declined = $declinedRows.Count
if ($published -eq 0) { $problems += '  the support matrix publishes nothing' }
if ($declined -eq 0) { $problems += '  the support matrix declines nothing, so its reasons are unchecked' }

# -- the caller's own assertion ---------------------------------------------
#
# ⛔ A REQUIRED ROW THAT PRODUCED NOTHING IS A FAILURE, which is what makes this
# a command an entry can close on rather than a report.
$required = 0
if ($RequireRows) {
    foreach ($want in ($RequireRows -split ',' | Where-Object { $_ })) {
        $required += 1
        if (-not $publishedRows.ContainsKey($want)) {
            $problems += ('  ' + $want + ': required, and the support matrix does not publish it')
            continue
        }
        $file = $publishedRows[$want].file
        $path = Join-Path $out 'a' | Join-Path -ChildPath $file
        if (-not (Test-Path -LiteralPath $path) -or (Get-Item -LiteralPath $path).Length -eq 0) {
            $problems += ('  ' + $want + ': required, and ' + $file + ' was not generated, or is empty')
        }
    }
}

# -- 6: a reader that is not this project's ---------------------------------
#
# ⭐ THE DUMP IS TEXT SO THAT SOMETHING ELSE CAN READ IT, and a format only this
# project can read back is a format only this project has checked.
# ⛔ A SKIP IS REPORTED AS A SKIP. sqlite3 absent means nothing about the dump
# was verified by anybody but this tree.
#
# ⚠ `.read` RATHER THAN A PIPE. PowerShell's native-command pipe is not
# byte-exact: it appends a trailing CRLF, which scripts/common/write-file.mjs
# measured and which would arrive inside the SQL.
$sqlite = 'skipped'
# ⚠ The column the dump promises, named once here so the message below and the
# query above cannot drift apart.
$canonical = 'canonical_json'
$sqlite3 = Get-Command sqlite3 -ErrorAction SilentlyContinue
if ($sqlite3) {
    $db = Join-Path $out 'corpus.db'
    if (Test-Path -LiteralPath $db) { Remove-Item -Force -LiteralPath $db }
    $sqlPath = (Join-Path $out 'a' | Join-Path -ChildPath 'corpus.sql') -replace '\\', '/'
    $sqliteLog = Join-Path $out 'sqlite.log'
    & $sqlite3.Source $db (".read '" + $sqlPath + "'") > $sqliteLog 2>&1
    $rcS = $LASTEXITCODE
    if ($rcS -ne 0) {
        $sqlite = 'failed'
        $problems += ('  the dump did not load into sqlite3, exit ' + $rcS + '. Its output is in .tmp/check-formats-ps/sqlite.log')
    }
    else {
        $rows = & $sqlite3.Source $db 'select count(*) from profile;'
        $rcQ = $LASTEXITCODE
        if ($rcQ -ne 0) {
            $sqlite = 'failed'
            $problems += ('  the loaded database did not answer a query, exit ' + $rcQ)
        }
        elseif ("$rows" -ne "$profiles") {
            $sqlite = 'failed'
            $problems += ('  the dump loaded ' + $rows + ' row(s) for ' + $profiles + ' profile(s)')
        }
        else {
            # ⭐ THE ESCAPING, ASSERTED BY SOMETHING THAT IS NOT THIS PROJECT. A
            # row count says the inserts parsed; it says nothing about whether
            # the quote doubling survived. sqlite3 parsing every stored profile
            # as JSON does.
            #
            # ⛔ THE CAPABILITY IS PROBED SEPARATELY FROM THE QUESTION, and that
            # is not tidiness. A single query answers "this sqlite3 has no
            # JSON1" and "the column the dump promises is not there" with the
            # same failure, and the first is a fact about the host while the
            # second is a broken dump. Measured 2026-09-02: a planted dump whose
            # CREATE TABLE renamed canonical_json PASSED this check while it was
            # one query.
            & $sqlite3.Source $db "select json_valid('{}');" > $null 2>> $sqliteLog
            $rcJ = $LASTEXITCODE
            $valid = & $sqlite3.Source $db 'select count(*) from profile where json_valid(canonical_json);' 2>> $sqliteLog
            $rcV = $LASTEXITCODE
            if ($rcJ -ne 0) {
                # ⚠ NOT A FAILURE. A sqlite3 built without JSON1 cannot ask the
                # question, which is a fact about the host rather than about the
                # dump.
                $sqlite = 'ok-no-json1'
            }
            elseif ($rcV -ne 0) {
                $sqlite = 'failed'
                $problems += ('  sqlite3 has json_valid and could not read ' + $canonical + ' from the loaded dump, exit ' + $rcV)
            }
            elseif ("$valid" -ne "$profiles") {
                $sqlite = 'failed'
                $problems += ('  sqlite3 read ' + $valid + ' of ' + $profiles + ' stored profile(s) as valid JSON')
            }
            else { $sqlite = 'ok' }
        }
    }
}

$count = $problems.Count

if ($Json) {
    Write-Output ('{"schema":"check-formats/2","files":' + $files + ',"profiles":' + $profiles + ',"published":' + $published + ',"declined":' + $declined + ',"required":' + $required + ',"sqlite":"' + $sqlite + '","problems":' + $count + '}')
    if ($count -gt 0) { exit 1 }
    exit 0
}

if ($count -eq 0) {
    Write-Output ('formats ok: ' + $files + ' file(s) from ' + $profiles + ' profile(s), byte-identical over two runs,')
    Write-Output ('  ' + $published + ' format(s) published and ' + $declined + ' declined with a reason, every lossless one')
    Write-Output '  round-tripping to canonical JSON and every partial one carrying its subset.'
    Write-Output ('  sqlite3 load: ' + $sqlite)
    if ($sqlite -eq 'skipped') { Write-Output '  ⚠ A SKIP IS NOT A PASS: sqlite3 is not on this host.' }
    exit 0
}

[Console]::Error.WriteLine('formats check failed, ' + $count + ' problem(s):')
[Console]::Error.WriteLine('')
foreach ($problem in $problems) { [Console]::Error.WriteLine($problem) }
[Console]::Error.WriteLine('')
[Console]::Error.WriteLine('One generator, canonical JSON in, every format out. Never hand-edit a')
[Console]::Error.WriteLine('generated file. TODO/schema.md, SCHEMA-08 and SCHEMA-12.')
exit 1
