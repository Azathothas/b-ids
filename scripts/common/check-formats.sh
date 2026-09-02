#!/bin/sh
# check-formats.sh - does every published format come out of the one generator,
# round-trip, and produce the same bytes twice?
#
# ⛔ JSON IS ONE CONSUMER, NOT THE CONSUMER. A corpus reachable only by writing a
# JSON walker is a corpus most people copy values out of by hand, and a value
# copied by hand stops matching the day the build moves. TODO/schema.md,
# SCHEMA-08.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. every format regenerates from the canonical corpus;
#   2. TWO RUNS ARE BYTE-IDENTICAL. A generator that read a clock or a hash seed
#      would produce a diff on every run, and a published artefact that diffs on
#      every run is one nobody can tell a real change from;
#   3. the lossless formats round-trip to byte-identical canonical JSON, which
#      is the half a writer alone cannot prove;
#   4. the lossy ones carry the documented subset and say in their own header
#      what they leave out.
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
#
# Exit codes: 0 every format round-trips, 1 one did not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
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
# here in awk would be a second reader of five formats, disagreeing with the one
# the crate publishes the first time either moved.
cargo test -q -p b-ids-corpus --test formats > "$OUT/tests.log" 2>&1
rc_t=$?
if [ "$rc_t" != 0 ]; then
  note "the round-trip suite failed. Its output is in .tmp/check-formats/tests.log"
fi

# ⚠ AND THE GENERATED FILES ARE THE ONES THE SUITE IS ABOUT, which the suite
# cannot say: it renders its own fixture. This is the one assertion that reads
# what was written to disk.
for want in corpus.json corpus.ndjson corpus.csv corpus.tsv corpus.md; do
  [ -s "$OUT/a/$want" ] || note "$want was not generated, or is empty"
done
if [ -s "$OUT/a/corpus.md" ] && ! grep -q 'Do not edit' "$OUT/a/corpus.md"; then
  note "corpus.md does not say in its own header that it is generated"
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-formats/1","files":%s,"profiles":%s,"problems":%s}\n' \
    "${FILES:-0}" "${PROFILES:-0}" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'formats ok: %s file(s) from %s profile(s), byte-identical over two runs,\n' \
    "$FILES" "$PROFILES"
  printf '  every lossless format round-trips to canonical JSON and every lossy one\n'
  printf '  carries the documented subset.\n'
  exit 0
fi

printf 'formats check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'One generator, canonical JSON in, every format out. Never hand-edit a\n' >&2
printf 'generated file. TODO/schema.md, SCHEMA-08.\n' >&2
exit 1
