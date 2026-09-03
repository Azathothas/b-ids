#!/bin/sh
# check-data-branch.sh - is what the data branch would carry exactly what the
# corpus derives to, and would a push that rewrote it be refused?
#
# ⛔ A CONSUMER PINNING A COMMIT ON THE DATA BRANCH KEEPS WORKING FOREVER, and
# that property is free right up until somebody rewrites the branch.
# TODO/publish.md, PUB-02.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the tree is regenerated from the canonical corpus and nothing else, and
#      two builds of it are byte-identical;
#   2. EVERY file has a checksum, in the manifest a program reads AND in the
#      SHA256SUMS a consumer with neither a JSON parser nor this project's code
#      can check;
#   3. ⛔ THE SOURCE, ANY VENDORED DEPENDENCY AND THE REFERENCE CORPUS ARE NOT
#      ON IT. A consumer of the data never has to reason about somebody else's
#      licence, because none of it is in what they downloaded;
#   4. a push that would rewrite history is refused, which is a rule in the
#      crate with its own case rather than a sentence in a workflow.
#
# ⭐ AND IT COMPARES AGAINST WHAT IS PUBLISHED. The regenerated tree and the
# branch's own tree are compared as two git tree objects, which is what "byte
# for byte" means for a branch. The answer is in the JSON as `matched`, so the
# twin comparison can see whether both halves did it. ⚠ WITH NO BRANCH AT ALL
# that leg is a SKIP naming the branch that would make it run: reporting a pass
# over a branch nobody has made is the "step that exits 0 having done nothing"
# row of docs/conventions/forbidden-patterns.md.
#
# ⛔ IT PUSHES NOTHING and creates no branch.
#
# Usage:
#   sh scripts/common/check-data-branch.sh
#   sh scripts/common/check-data-branch.sh --json
#
# Exit codes: 0 the tree is what it should be, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-data-branch: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-data-branch: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-data-branch: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-data-branch: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⛔ THIS CHECK RESOLVES THE CORPUS ROOT AND THEN REFUSES ONE ANSWER. Its
# question is whether the published branch equals what the CANONICAL corpus
# derives to, so a run that resolved to the branch would compare the branch
# against itself and pass without asking anything. ⚠ Once corpus/ leaves the
# default branch that is the honest state of this check: exit 2, "could not
# run", which CI-07 rules is not a failure. TODO/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-data-branch: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"
CORPUS_REF=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh" --ref)
if [ -n "$CORPUS_REF" ]; then
  printf 'check-data-branch: the canonical corpus is not in this tree, so the\n' >&2
  printf 'branch has nothing to be compared against. It resolved to %s.\n' "$CORPUS_REF" >&2
  exit 2
fi
command -v cargo >/dev/null 2>&1 || { printf 'check-data-branch: cargo not found\n' >&2; exit 2; }

# ⛔ THE BRANCH IS NAMED ONCE, here and in the twin, so a rename moves both.
BRANCH=data

SUITE="$REPO_ROOT/crates/b-ids-corpus/tests/publish.rs"
[ -f "$SUITE" ] || { printf 'check-data-branch: no suite at %s\n' "$SUITE" >&2; exit 2; }

WANT='publish_two_builds_over_one_corpus_are_byte_identical
publish_every_artefact_has_a_checksum_and_the_checksum_is_of_the_file
publish_the_tree_carries_no_source_and_no_vendored_dependency
publish_the_tree_carries_the_corpus_the_formats_and_the_routes
publish_a_push_that_would_rewrite_the_data_branch_is_refused'

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

OUT="$REPO_ROOT/.tmp/check-data-branch"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-data-branch: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo build -q -p b-ids-corpus || {
  printf 'check-data-branch: the corpus crate did not build\n' >&2
  exit 2
}
BIN="$REPO_ROOT/target/debug/b-ids-corpus"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-data-branch: %s is not executable\n' "$BIN" >&2; exit 2; }

# -- 1: regenerate, twice ----------------------------------------------------
#
# ⛔ READ FROM THE PROCESS, UNPIPED.
"$BIN" publish --root "$CORPUS_ROOT" --out "$OUT/a" > "$OUT/a.log" 2>&1
rc_a=$?
"$BIN" publish --root "$CORPUS_ROOT" --out "$OUT/b" > "$OUT/b.log" 2>&1
rc_b=$?
if [ "$rc_a" != 0 ] || [ "$rc_b" != 0 ]; then
  printf 'check-data-branch: the build exited %s then %s\n' "$rc_a" "$rc_b" >&2
  cat "$OUT/a.log" >&2
  exit 1
fi
STATUS=$(awk '/^corpus=publish /{ line = $0 } END { print line }' "$OUT/a.log")
FILES=$(printf '%s' "$STATUS" | awk -F'files:' '{ split($2, a, / /); print a[1] }')
[ -n "${FILES:-}" ] || { printf 'check-data-branch: the build printed no status line\n' >&2; exit 1; }
if command -v diff >/dev/null 2>&1; then
  diff -r "$OUT/a" "$OUT/b" > "$OUT/diff.log" 2>&1 ||
    note "two builds of the branch content differ. See .tmp/check-data-branch/diff.log"
fi

# -- 2: every file has a checksum, in both places ---------------------------
#
# ⛔ A CHECKSUM FILE NOBODY CHECKED agrees with itself. Every file on disk is
# looked up in both, so a file written and not recorded is a finding rather than
# an absence nobody counted.
command -v jq >/dev/null 2>&1 || { printf 'check-data-branch: jq not found\n' >&2; exit 2; }
jq -r '.artefacts[].path' "$OUT/a/MANIFEST.json" | tr -d '\r' | LC_ALL=C sort > "$OUT/recorded.txt"
( cd "$OUT/a" && find . -type f | sed 's|^\./||' | LC_ALL=C sort ) > "$OUT/present.txt"
RECORDED=$(wc -l < "$OUT/recorded.txt" | tr -d ' ')
SUMS=$(grep -c . "$OUT/a/SHA256SUMS" 2>/dev/null || echo 0)
[ "$RECORDED" = "$SUMS" ] || note "the manifest records $RECORDED file(s) and SHA256SUMS carries $SUMS"
UNRECORDED=0
while IFS= read -r file; do
  case "$file" in
    MANIFEST.json | SHA256SUMS) continue ;;
  esac
  grep -qxF "$file" "$OUT/recorded.txt" || {
    UNRECORDED=$((UNRECORDED + 1))
    note "$file is on the branch and has no checksum in the manifest"
  }
  grep -qF "  $file" "$OUT/a/SHA256SUMS" || \
    note "$file is on the branch and has no line in SHA256SUMS"
done < "$OUT/present.txt"
PRESENT=$(wc -l < "$OUT/present.txt" | tr -d ' ')

# -- 3: nothing of the source is on it --------------------------------------
for forbidden in crates vendor references scripts target docs TODO; do
  [ -e "$OUT/a/$forbidden" ] && note "$forbidden is on the branch and must not be"
done

# -- 4: the suite, and the branch's own state -------------------------------
cargo test -q -p b-ids-corpus --test publish > "$OUT/tests.log" 2>&1
rc_t=$?
CASES=$(awk '/^running [0-9]+ tests/ { print $2; exit }' "$OUT/tests.log")
[ "$rc_t" = 0 ] || note "the publish suite failed. Its output is in .tmp/check-data-branch/tests.log"
[ "${CASES:-0}" -ge "$CASES_WANTED" ] 2>/dev/null ||
  note "the suite ran ${CASES:-0} case(s) where at least $CASES_WANTED were expected"

# ⛔ THE LEG THAT COULD NOT RUN, AND NOW DOES. Until 2026-09-03 the branch did
# not exist and this reported a skip with the sentence "push it once and this leg
# starts running". ⚠ It was pushed and the sentence stayed, which is a skip that
# had stopped being honest: the branch was there and nothing compared against it.
PUBLISHED=absent
REF=""
if git rev-parse -q --verify "refs/heads/$BRANCH" >/dev/null 2>&1; then
  PUBLISHED=local
  REF="refs/heads/$BRANCH"
elif git rev-parse -q --verify "refs/remotes/origin/$BRANCH" >/dev/null 2>&1; then
  PUBLISHED=remote
  REF="refs/remotes/origin/$BRANCH"
fi

# ⭐ THE COMPARISON IS BETWEEN TWO GIT TREE OBJECTS, which is what "byte for
# byte" means for a branch: one tree object is one set of bytes, over every path
# and every mode. The regenerated tree is written into a TEMPORARY index, so the
# repository's own index is never touched.
MATCHES=""
if [ -n "$REF" ]; then
  IDX="$OUT/compare.index"
  rm -f "$IDX"
  if ( cd "$OUT/a" && GIT_INDEX_FILE="$IDX" git --git-dir="$REPO_ROOT/.git" --work-tree=. \
    add --all --force -- . ) >/dev/null 2>&1; then
    LOCAL_TREE=$(GIT_INDEX_FILE="$IDX" git write-tree 2>/dev/null || true)
    PUBLISHED_TREE=$(git rev-parse -q --verify "$REF^{tree}" 2>/dev/null || true)
    if [ -z "${LOCAL_TREE:-}" ] || [ -z "${PUBLISHED_TREE:-}" ]; then
      note "the $BRANCH branch is $PUBLISHED and neither tree could be read, so nothing was compared"
    elif [ "$LOCAL_TREE" = "$PUBLISHED_TREE" ]; then
      MATCHES="$LOCAL_TREE"
    else
      note "the regenerated tree is $LOCAL_TREE and $REF carries $PUBLISHED_TREE, so what is published is not what this corpus derives to"
    fi
  else
    note "the regenerated tree could not be written into a temporary index, so nothing was compared"
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-data-branch/2","files":%s,"present":%s,"recorded":%s,"cases":%s,"published":"%s","matched":%s,"problems":%s}\n' \
    "${FILES:-0}" "$PRESENT" "$RECORDED" "${CASES:-0}" "$PUBLISHED" \
    "$([ -n "$MATCHES" ] && echo true || echo false)" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'data branch ok: %s file(s) regenerated, identical over two builds,\n' "$PRESENT"
  printf '  %s of them with a checksum in the manifest and in SHA256SUMS, and no\n' "$RECORDED"
  printf '  source, vendored dependency or reference corpus among them.\n'
  if [ -n "$MATCHES" ]; then
    printf '  ⭐ The %s branch is %s and its tree is %s, which is what this\n' \
      "$BRANCH" "$PUBLISHED" "$MATCHES"
    printf '  corpus derives to. One tree object is one set of bytes.\n'
  else
    printf '  ⚠ A SKIP IS NOT A PASS: the %s branch is %s, so the regenerated tree was\n' \
      "$BRANCH" "$PUBLISHED"
    printf '  compared against nothing. Push it once and this leg starts running.\n'
  fi
  printf '  ⛔ Nothing was pushed and no branch was created.\n'
  exit 0
fi

printf 'data branch check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A consumer pinning a commit on this branch keeps working forever, and\n' >&2
printf 'that property is free. TODO/publish.md, PUB-02.\n' >&2
exit 1
