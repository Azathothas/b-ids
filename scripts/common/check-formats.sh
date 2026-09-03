#!/bin/sh
# check-formats.sh - does every published format come out of the one generator,
# round-trip, and produce the same bytes twice?
#
# ⛔ JSON IS ONE CONSUMER, NOT THE CONSUMER. A corpus reachable only by writing a
# JSON walker is a corpus most people copy values out of by hand, and a value
# copied by hand stops matching the day the build moves. TODO/schema.md,
# SCHEMA-08 and SCHEMA-12.
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
# ⚠ IT GENERATES INTO A THROWAWAY DIRECTORY under .tmp and never into the tree.
# Nothing in this repository publishes generated formats yet: PUB-02 and PUB-03
# are the surfaces that will, and this check exists before them so the generator
# is proved before anything depends on it.
#
# Usage:
#   sh scripts/common/check-formats.sh
#   sh scripts/common/check-formats.sh --json
#   sh scripts/common/check-formats.sh --require-rows yaml,toml,sqlite,protobuf
#
# Exit codes: 0 every format round-trips, 1 one did not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
REQUIRE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --require-rows)
      shift
      [ $# -gt 0 ] || { printf 'check-formats: --require-rows needs a list\n' >&2; exit 2; }
      REQUIRE="$1"
      ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-formats: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-formats: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-formats: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-formats: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { printf 'check-formats: cargo not found\n' >&2; exit 2; }

# ⛔ 2, not 1. A tree with no corpus has verified nothing about the generator,
# which is a different fact from a generator that produced a bad file.
[ -d "$REPO_ROOT/corpus" ] || {
  printf 'check-formats: there is no corpus under %s, so there is nothing to generate\n' \
    "$REPO_ROOT" >&2
  exit 2
}

OUT="$REPO_ROOT/.tmp/check-formats"
rm -rf "$OUT"
mkdir -p "$OUT/a" "$OUT/b" || { printf 'check-formats: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo build -q -p b-ids-corpus || {
  printf 'check-formats: the corpus crate did not build\n' >&2
  exit 2
}
BIN="$REPO_ROOT/target/debug/b-ids-corpus"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-formats: %s is not executable\n' "$BIN" >&2; exit 2; }

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

# -- 1 and 2: generate twice, and compare the bytes --------------------------
#
# ⛔ READ FROM THE PROCESS, UNPIPED. A guard on the left of a pipe reports the
# pipeline's status, so one that failed reads as green.
"$BIN" formats --root "$REPO_ROOT" --out "$OUT/a" > "$OUT/a.log" 2>&1
rc_a=$?
"$BIN" formats --root "$REPO_ROOT" --out "$OUT/b" > "$OUT/b.log" 2>&1
rc_b=$?
if [ "$rc_a" != 0 ] || [ "$rc_b" != 0 ]; then
  printf 'check-formats: the generator exited %s then %s\n' "$rc_a" "$rc_b" >&2
  cat "$OUT/a.log" >&2
  exit 1
fi

# ⛔ THE FIXED LAST LINE, never the prose above it. Parsing the report would make
# every wording change a silent behaviour change.
STATUS=$(awk '/^corpus=formats /{ line = $0 } END { print line }' "$OUT/a.log")
FILES=$(printf '%s' "$STATUS" | awk -F'files:' '{ split($2, a, / /); print a[1] }')
PROFILES=$(printf '%s' "$STATUS" | awk -F'profiles:' '{ split($2, a, / /); print a[1] }')
[ -n "${FILES:-}" ] || { printf 'check-formats: the generator printed no status line\n' >&2; exit 1; }

for f in "$OUT/a"/*; do
  name=$(basename "$f")
  if ! cmp -s "$f" "$OUT/b/$name"; then
    note "$name: two runs of the generator differ, so it is not deterministic"
  fi
done

# -- 3 and 4: the round trips, which are the suite's ------------------------
#
# ⛔ THE READERS ARE THE CRATE'S AND SO ARE THE ASSERTIONS. A round trip written
# here in awk would be a second reader of nine formats, disagreeing with the one
# the crate publishes the first time either moved.
cargo test -q -p b-ids-corpus --test formats > "$OUT/tests.log" 2>&1
rc_t=$?
if [ "$rc_t" != 0 ]; then
  note "the round-trip suite failed. Its output is in .tmp/check-formats/tests.log"
fi

# -- 5: the support matrix is the catalogue, and this opens what it names ----
#
# ⭐ THE LIST IS NOT WRITTEN HERE. formats.md is generated from the generator's
# own vocabulary, so a format added, renamed or declined moves it in the same
# change and this check follows without being edited.
MATRIX="$OUT/a/formats.md"
[ -s "$MATRIX" ] || note "formats.md was not generated, or is empty"
awk '
  /^## Published/ { section = "published"; next }
  /^## Declined/  { section = "declined";  next }
  /^\| `/ {
    gsub(/`/, "")
    n = split($0, cell, /\|/)
    name = cell[2]; gsub(/^ +| +$/, "", name)
    if (section == "published") {
      file = cell[3]; gsub(/^ +| +$/, "", file)
      carries = cell[4]; gsub(/^ +| +$/, "", carries)
      print "published\t" name "\t" file "\t" carries
    } else if (section == "declined") {
      why = cell[3]; gsub(/^ +| +$/, "", why)
      print "declined\t" name "\t" length(why)
    }
  }
' "$MATRIX" > "$OUT/catalogue.tsv"

PUBLISHED=0
DECLINED=0
while IFS='	' read -r kind name file rest; do
  case "$kind" in
    published)
      PUBLISHED=$((PUBLISHED + 1))
      [ -s "$OUT/a/$file" ] || note "$name: the matrix names $file and it was not generated, or is empty"
      [ -n "$rest" ] || note "$name: the matrix does not say what it carries"
      ;;
    declined)
      DECLINED=$((DECLINED + 1))
      # ⛔ BOTH HALVES OF A DECLINED FORMAT. Absent from the output, and named
      # with a reason. Either alone is a consumer guessing.
      for spill in "$OUT/a/corpus.$name" "$OUT/a/$name"; do
        [ -e "$spill" ] && note "$name is recorded as declined and $spill was generated"
      done
      [ "${file:-0}" -gt 40 ] 2>/dev/null || note "$name is declined with no reason worth reading"
      ;;
  esac
done < "$OUT/catalogue.tsv"
[ "$PUBLISHED" -gt 0 ] || note "the support matrix publishes nothing"
[ "$DECLINED" -gt 0 ] || note "the support matrix declines nothing, so its reasons are unchecked"

# -- the caller's own assertion ---------------------------------------------
#
# ⛔ A REQUIRED ROW THAT PRODUCED NOTHING IS A FAILURE, which is what makes this
# a command an entry can close on rather than a report.
REQUIRED=0
if [ -n "$REQUIRE" ]; then
  for want in $(printf '%s' "$REQUIRE" | tr ',' ' '); do
    REQUIRED=$((REQUIRED + 1))
    row=$(awk -F'\t' -v n="$want" '$1 == "published" && $2 == n { print $3 }' "$OUT/catalogue.tsv")
    if [ -z "$row" ]; then
      note "$want: required, and the support matrix does not publish it"
    elif [ ! -s "$OUT/a/$row" ]; then
      note "$want: required, and $row was not generated, or is empty"
    fi
  done
fi

# -- 6: a reader that is not this project's ---------------------------------
#
# ⭐ THE DUMP IS TEXT SO THAT SOMETHING ELSE CAN READ IT, and a format only this
# project can read back is a format only this project has checked.
# ⛔ A SKIP IS REPORTED AS A SKIP. sqlite3 absent means nothing about the dump
# was verified by anybody but this tree.
SQLITE=skipped
# ⚠ The column the dump promises, named once here so the message below and the
# query above cannot drift apart.
CANONICAL=canonical_json
if command -v sqlite3 >/dev/null 2>&1; then
  rm -f "$OUT/corpus.db"
  sqlite3 "$OUT/corpus.db" < "$OUT/a/corpus.sql" > "$OUT/sqlite.log" 2>&1
  rc_s=$?
  if [ "$rc_s" != 0 ]; then
    SQLITE=failed
    note "the dump did not load into sqlite3, exit $rc_s. Its output is in .tmp/check-formats/sqlite.log"
  else
    rows=$(sqlite3 "$OUT/corpus.db" 'select count(*) from profile;' 2>>"$OUT/sqlite.log")
    rc_q=$?
    if [ "$rc_q" != 0 ]; then
      SQLITE=failed
      note "the loaded database did not answer a query, exit $rc_q"
    elif [ "$rows" != "$PROFILES" ]; then
      SQLITE=failed
      note "the dump loaded $rows row(s) for $PROFILES profile(s)"
    else
      # ⭐ THE ESCAPING, ASSERTED BY SOMETHING THAT IS NOT THIS PROJECT. A
      # row count says the inserts parsed; it says nothing about whether the
      # quote doubling survived. sqlite3 parsing every stored profile as JSON
      # does.
      #
      # ⛔ THE CAPABILITY IS PROBED SEPARATELY FROM THE QUESTION, and that is
      # not tidiness. A single query answers "this sqlite3 has no JSON1" and
      # "the column the dump promises is not there" with the same failure, and
      # the first is a fact about the host while the second is a broken dump.
      # Measured 2026-09-02: a planted dump whose CREATE TABLE renamed
      # canonical_json PASSED this check while it was one query.
      sqlite3 "$OUT/corpus.db" "select json_valid('{}');" >/dev/null 2>>"$OUT/sqlite.log"
      rc_j=$?
      valid=$(sqlite3 "$OUT/corpus.db" \
        'select count(*) from profile where json_valid(canonical_json);' 2>>"$OUT/sqlite.log")
      rc_v=$?
      if [ "$rc_j" != 0 ]; then
        # ⚠ NOT A FAILURE. A sqlite3 built without JSON1 cannot ask the
        # question, which is a fact about the host rather than about the dump.
        SQLITE=ok-no-json1
      elif [ "$rc_v" != 0 ]; then
        SQLITE=failed
        note "sqlite3 has json_valid and could not read $CANONICAL from the loaded dump, exit $rc_v"
      elif [ "$valid" != "$PROFILES" ]; then
        SQLITE=failed
        note "sqlite3 read $valid of $PROFILES stored profile(s) as valid JSON"
      else
        SQLITE=ok
      fi
    fi
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-formats/2","files":%s,"profiles":%s,"published":%s,"declined":%s,"required":%s,"sqlite":"%s","problems":%s}\n' \
    "${FILES:-0}" "${PROFILES:-0}" "$PUBLISHED" "$DECLINED" "$REQUIRED" "$SQLITE" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'formats ok: %s file(s) from %s profile(s), byte-identical over two runs,\n' \
    "$FILES" "$PROFILES"
  printf '  %s format(s) published and %s declined with a reason, every lossless one\n' \
    "$PUBLISHED" "$DECLINED"
  printf '  round-tripping to canonical JSON and every partial one carrying its subset.\n'
  printf '  sqlite3 load: %s\n' "$SQLITE"
  [ "$SQLITE" != skipped ] || printf '  ⚠ A SKIP IS NOT A PASS: sqlite3 is not on this host.\n'
  exit 0
fi

printf 'formats check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'One generator, canonical JSON in, every format out. Never hand-edit a\n' >&2
printf 'generated file. TODO/schema.md, SCHEMA-08 and SCHEMA-12.\n' >&2
exit 1
