#!/bin/sh
# check-release.sh - would a release build produce the same bytes twice, and
# would it refuse to overwrite a tag somebody has already pinned?
#
# ⛔ A CONSUMER THAT PINS A RELEASE AND GETS DIFFERENT BYTES LATER HAS BEEN
# BROKEN SILENTLY. TODO/publish.md, PUB-01.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. two builds over one corpus are byte-identical, artefact by artefact, and
#      produce identical checksums. Nothing in the assembler reads a clock;
#   2. the suite that owns the release rules is present, case by case, and
#      passes. ⚠ THE ASSERTIONS ARE THE CRATE'S;
#   3. ⛔ THE TAG THIS BUILD WOULD TAKE DOES NOT ALREADY EXIST in this
#      repository, read from git rather than assumed;
#   4. a deterministic archive of the tree is byte-identical over two runs,
#      where this host's tar can make one. ⚠ A SKIP IS REPORTED AS A SKIP.
#
# ⛔ IT PUBLISHES NOTHING. No tag is created, no asset is uploaded and no remote
# is written to. --dry-run is required and is the only mode, for the reason
# `latest` requires --assert-stable: a run with no argument would read as though
# it had cut a release.
#
# Usage:
#   sh scripts/common/check-release.sh --dry-run
#   sh scripts/common/check-release.sh --dry-run --json
#
# Exit codes: 0 the build is reproducible and the tag is free, 1 it is not,
# 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
DRY=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --dry-run) DRY=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-release: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

[ "$DRY" = 1 ] || {
  printf 'check-release: --dry-run is required. This check publishes nothing, and a\n' >&2
  printf '  run with no argument would read as though it had cut a release.\n' >&2
  exit 2
}

command -v git >/dev/null 2>&1 || { printf 'check-release: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-release: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-release: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-release: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"
command -v cargo >/dev/null 2>&1 || { printf 'check-release: cargo not found\n' >&2; exit 2; }

SUITE="$REPO_ROOT/crates/b-ids-corpus/tests/publish.rs"
[ -f "$SUITE" ] || { printf 'check-release: no suite at %s\n' "$SUITE" >&2; exit 2; }

# ⛔ THE CASES ARE NAMED HERE AND ASSERTED THERE, so a suite that lost one is
# caught by this check rather than by nobody.
WANT='publish_two_builds_over_one_corpus_are_byte_identical
publish_every_artefact_has_a_checksum_and_the_checksum_is_of_the_file
publish_the_tree_carries_no_source_and_no_vendored_dependency
publish_a_tag_that_already_exists_is_refused
publish_a_date_that_is_not_one_is_refused
publish_a_build_with_no_artefact_is_not_releasable'

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

CASES_WANTED=0
for want in $WANT; do
  CASES_WANTED=$((CASES_WANTED + 1))
  grep -q "fn $want" "$SUITE" || note "$want is not in the suite"
done

OUT="$REPO_ROOT/.tmp/check-release"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-release: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo build -q -p b-ids-corpus || {
  printf 'check-release: the corpus crate did not build\n' >&2
  exit 2
}
BIN="$REPO_ROOT/target/debug/b-ids-corpus"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-release: %s is not executable\n' "$BIN" >&2; exit 2; }

# -- 1: two builds, byte for byte -------------------------------------------
#
# ⛔ READ FROM THE PROCESS, UNPIPED.
"$BIN" publish --root "$CORPUS_ROOT" --out "$OUT/a" > "$OUT/a.log" 2>&1
rc_a=$?
"$BIN" publish --root "$CORPUS_ROOT" --out "$OUT/b" > "$OUT/b.log" 2>&1
rc_b=$?
if [ "$rc_a" != 0 ] || [ "$rc_b" != 0 ]; then
  printf 'check-release: the build exited %s then %s\n' "$rc_a" "$rc_b" >&2
  cat "$OUT/a.log" >&2
  exit 1
fi

STATUS=$(awk '/^corpus=publish /{ line = $0 } END { print line }' "$OUT/a.log")
FILES=$(printf '%s' "$STATUS" | awk -F'files:' '{ split($2, a, / /); print a[1] }')
BYTES=$(printf '%s' "$STATUS" | awk -F'bytes:' '{ split($2, a, / /); print a[1] }')
FROM=$(printf '%s' "$STATUS" | awk -F'from:' '{ split($2, a, / /); print a[1] }')
[ -n "${FILES:-}" ] || { printf 'check-release: the build printed no status line\n' >&2; exit 1; }

if command -v diff >/dev/null 2>&1; then
  diff -r "$OUT/a" "$OUT/b" > "$OUT/diff.log" 2>&1 ||
    note "two builds over one corpus differ. See .tmp/check-release/diff.log"
else
  note "diff is not on this host, so the two builds were not compared"
fi

# -- 2: the suite ------------------------------------------------------------
cargo test -q -p b-ids-corpus --test publish > "$OUT/tests.log" 2>&1
rc_t=$?
CASES=$(awk '/^running [0-9]+ tests/ { print $2; exit }' "$OUT/tests.log")
[ "$rc_t" = 0 ] || note "the publish suite failed. Its output is in .tmp/check-release/tests.log"
[ "${CASES:-0}" -ge "$CASES_WANTED" ] 2>/dev/null ||
  note "the suite ran ${CASES:-0} case(s) where at least $CASES_WANTED were expected"

# -- 3: the tag this build would take ---------------------------------------
#
# ⛔ READ FROM GIT, never assumed. A published release is immutable, so the
# question is whether the tag is free rather than whether the naming rule reads
# well.
TODAY=$(date -u +%Y-%m-%d)
LAYOUT=$(awk -F'"' '/^pub const LAYOUT: &str = /{ print $2; exit }' crates/b-ids-corpus/src/route.rs)
DOTTED=$(printf '%s' "$TODAY" | tr '-' '.')
TAG="$LAYOUT.$DOTTED.1"
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null 2>&1; then
  note "$TAG already exists, so this build cannot be released under it. Bump the counter"
fi
TAGS=$(git tag --list | wc -l | tr -d ' ')

# -- 4: a deterministic archive ---------------------------------------------
#
# ⛔ A SKIP IS REPORTED AS A SKIP. A tar that cannot be told to zero the owner
# and the timestamp produces a different archive every run, and nothing about
# reproducibility was verified on such a host.
ARCHIVE=skipped
# ⚠ TWO TARS, TWO SPELLINGS, ONE DATE FORMAT. Measured on this host 2026-09-03:
# GNU tar 1.35 wants `--owner=0 --group=0` and REFUSES `--uid`; the bsdtar that
# ships with Windows wants `--uid 0 --gid 0` and refuses `--force-local`; and
# `2026-01-01T00:00:00Z` is a bad date string to bsdtar while
# `2026-01-01 00:00:00` is accepted by both. ⛔ The two halves of this check
# resolve DIFFERENT tar binaries on this machine, so a leg that only worked for
# one of them made the pair disagree.
TAR_EPOCH="2026-01-01 00:00:00"
make_tar() {
  case "$1" in
    gnu)
      tar --force-local --format=ustar --numeric-owner --owner=0 --group=0         --mtime "$TAR_EPOCH" -cf "$2" -C "$3" -T "$LIST"
      ;;
    bsd)
      tar --format=ustar --numeric-owner --uid 0 --gid 0         --mtime "$TAR_EPOCH" -cf "$2" -C "$3" -T "$LIST"
      ;;
  esac
}
if command -v tar >/dev/null 2>&1; then
  LIST="$OUT/files.txt"
  ( cd "$OUT/a" && find . -type f | LC_ALL=C sort > "$LIST" ) 2>/dev/null
  MODE=gnu
  make_tar gnu "$OUT/probe.tar" "$OUT/a" > "$OUT/tar.log" 2>&1 || MODE=bsd
  if make_tar "$MODE" "$OUT/one.tar" "$OUT/a" >> "$OUT/tar.log" 2>&1 &&
    make_tar "$MODE" "$OUT/two.tar" "$OUT/b" >> "$OUT/tar.log" 2>&1; then
    if cmp -s "$OUT/one.tar" "$OUT/two.tar"; then
      ARCHIVE=ok
    else
      ARCHIVE=failed
      note "two archives of one build differ, so the archive step is not reproducible"
    fi
  else
    # ⚠ Not a failure. This host's tar takes neither spelling, which is a fact
    # about the host rather than about the build.
    ARCHIVE=skipped
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-release/1","files":%s,"bytes":%s,"cases":%s,"tags":%s,"archive":"%s","problems":%s}\n' \
    "${FILES:-0}" "${BYTES:-0}" "${CASES:-0}" "${TAGS:-0}" "$ARCHIVE" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'release ok: %s artefact(s), %s byte(s), identical over two builds.\n' "$FILES" "$BYTES"
  printf '  built from corpus %s\n' "$FROM"
  printf '  %s would be free, over %s existing tag(s). archive: %s\n' "$TAG" "$TAGS" "$ARCHIVE"
  [ "$ARCHIVE" != skipped ] || printf '  ⚠ A SKIP IS NOT A PASS: this tar cannot make a deterministic archive.\n'
  printf '  ⛔ Nothing was tagged, uploaded or pushed.\n'
  exit 0
fi

printf 'release check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A consumer that pins a release and gets different bytes later has been\n' >&2
printf 'broken silently. TODO/publish.md, PUB-01.\n' >&2
exit 1
