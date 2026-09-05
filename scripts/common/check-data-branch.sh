#!/bin/sh
# check-data-branch.sh - is what the data branch would carry exactly what the
# corpus derives to, and would a push that rewrote it be refused?
#
# ⛔ A CONSUMER PINNING A COMMIT ON THE DATA BRANCH KEEPS WORKING FOREVER, and
# that property is free right up until somebody rewrites the branch.
# docs/history/todo/publish.md, PUB-02.
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

# ⛔ THIS CHECK RESOLVES THE CORPUS ROOT AND THEN REFUSES SOME ANSWERS. Its
# question is whether the published branch equals what the CANONICAL corpus
# derives to, so a run that resolved to the DATA branch would compare that
# branch against itself and pass without asking anything.
#
# ⭐ SINCE PUB-13 THE CANONICAL CORPUS IS THE SOURCE BRANCH, and this check runs
# against it rather than skipping. Before that ruling, corpus/ leaving the
# default branch left this check with nothing to compare and its honest state
# was exit 2, "could not run". That state cost two CI jobs: ubuntu runs
# --strict, which fails on any skip, and windows asserts that only check-twins
# may skip. ⚠ Neither job needed changing in the end, because the source branch
# gave this check its question back. docs/history/todo/publish.md, PUB-11 and PUB-13.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-data-branch: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
# ⛔ THE GUARD IS ASKED BEFORE THE EXPORT, AND IT ASKS FOR THE SOURCE RATHER
# THAN THE REF. Both halves of that sentence were defects.
#
# ⚠ The export below sets B_IDS_CORPUS_ROOT, and the resolver's first rule is
# that an explicit root is never second guessed. Asking it anything afterwards
# therefore gets `source=explicit` and an EMPTY ref, whatever it would have
# said a line earlier. The guard read that empty ref as "the working tree is
# canonical" and did not fire.
#
# ⭐ Driven 2026-09-04 with corpus/ and raw/ moved out of the working tree:
# this check reported `data branch ok: 200 file(s) regenerated, identical over
# two builds` and exited 0, having compared the published branch against a
# materialised copy of that same branch. ⛔ A check cannot pass by comparing
# something to itself, and this one could. docs/history/todo/publish.md, PUB-11.
# ⛔ TWO ANSWERS ARE CANONICAL AND TWO ARE REFUSED, and the refusals are what
# this guard is for.
#
#   working-tree    a session that has the corpus checked out. Canonical.
#   source-branch   the everyday answer since PUB-13. Canonical.
#   data-branch     ⛔ REFUSED. It is what this check is checking, and comparing
#                   it against itself is how this check reported `data branch
#                   ok` over a corpus nobody had looked at.
#   explicit        ⛔ REFUSED. B_IDS_CORPUS_ROOT may name anything, including a
#                   copy of the data branch, and this guard cannot tell which.
CORPUS_SOURCE=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh" --source)
case "$CORPUS_SOURCE" in
  working-tree | source-branch) ;;
  *)
    printf 'check-data-branch: the canonical corpus did not resolve to this tree or\n' >&2
    printf 'to the source branch, so the data branch has nothing independent to be\n' >&2
    printf 'compared against. It resolved to %s.\n' "$CORPUS_SOURCE" >&2
    exit 2
    ;;
esac
# ⛔ AND EXPORTED ONLY AFTER THE GUARD HAS RUN, because cargo is downstream of
# this decision but the guard must not be.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"
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
TARGET_DIR=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
BIN="$TARGET_DIR/debug/b-ids-corpus"
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
PENDING=0
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
      # ⛔ BEHIND IS NOT THE SAME AS WRONG, AND THIS CHECK REPORTED BOTH AS A
      # FAILURE. Adding an artefact class to the assembler makes the published
      # branch a strict SUBSET of what the corpus now derives to: nothing on the
      # branch is wrong, there is simply less of it than the generator produces.
      # ⚠ The gate then went red on a state the publisher clears on the next
      # push, which is a red nobody can act on and the kind that gets ignored.
      # docs/history/todo/publish.md, PUB-14.
      #
      # ⭐ THE TWO CASES ARE DISTINGUISHABLE, so they are distinguished:
      #
      #   behind    every path the branch carries is still produced, and every
      #             IMMUTABLE one of them is byte-identical. Reported, and not
      #             a failure.
      #   diverged  anything else. A published path that is gone, or a
      #             published byte that a profile owns and that changed, is
      #             what this check exists for.
      #
      # ⛔ IMMUTABLE IS ASKED OF THE PRODUCER, NEVER GUESSED FROM A PATH. The
      # manifest records `derived` per artefact: false for a profile and its raw
      # sidecar, which are append-only, and true for every index, route, format
      # dump and generated config, which are functions of the whole corpus and
      # therefore MOVE when it grows.
      #
      # ⚠ THIS USED TO EXEMPT TWO FILES BY NAME and it was wrong the moment a
      # seventh profile landed. Measured 2026-09-04: adding one Firefox profile
      # changed nineteen published artefacts, every one of them an aggregate,
      # and this check called it a rewritten branch. Under that rule no capture
      # could ever be published again. docs/history/todo/driver.md, DRIVER-11.
      #
      # ⛔ The derived files are still compared by CONTENT rather than trusted:
      # every artefact line the published manifest carries for an IMMUTABLE path
      # has to appear unchanged in the regenerated one, so a digest that moved in
      # the one file that lists every digest is still caught.
      git ls-tree -r --name-only "$REF" | sort > "$OUT/published-paths.txt"
      ( cd "$OUT/a" && find . -type f | sed 's|^\./||' | sort ) > "$OUT/regenerated-paths.txt"
      GONE=$(comm -23 "$OUT/published-paths.txt" "$OUT/regenerated-paths.txt" | wc -l | tr -d ' ')
      ADDED=$(comm -13 "$OUT/published-paths.txt" "$OUT/regenerated-paths.txt" | wc -l | tr -d ' ')

      # ⛔ THE BRANCH AGAINST ITS OWN MANIFEST, and this leg is why the subset
      # test above is safe. A path DELETED from the branch leaves the branch a
      # smaller subset of what the corpus derives to, which is indistinguishable
      # from "not published yet" by comparing the two trees alone. ⚠ Measured by
      # planting exactly that: a published profile removed from a local branch
      # read as BEHIND until this leg existed.
      # ⭐ The branch's own manifest lists every artefact it published, so a path
      # in the manifest and not in the tree is a consumer's 404 and a rewrite.
      # docs/history/todo/publish.md, PUB-14.
      MISSING=0
      if git show "$REF:MANIFEST.json" > "$OUT/published-manifest.json" 2>/dev/null; then
        # ⛔ THE CARRIAGE RETURN IS STRIPPED. jq on Windows writes CRLF, so every
        # path arrived with a CR riding on it and matched nothing: this leg
        # reported all 198 artefacts missing on its first run. It is the same
        # defect CORPUS-02 recorded against this tool and it bit again here.
        jq -r '.artefacts[].path' "$OUT/published-manifest.json" 2>/dev/null |
          tr -d '\r' | sort > "$OUT/manifest-paths.txt" || : > "$OUT/manifest-paths.txt"
        MISSING=$(comm -23 "$OUT/manifest-paths.txt" "$OUT/published-paths.txt" | wc -l | tr -d ' ')
      else
        note "the $BRANCH branch carries no MANIFEST.json, so what it claims to publish cannot be read"
      fi

      # ⛔ THE REGENERATED manifest says which paths a profile owns. The
      # PUBLISHED one is an older schema and cannot be asked.
      jq -r '.artefacts[] | select(.derived == false) | .path' "$OUT/a/MANIFEST.json" 2>/dev/null |
        tr -d '\r' | sort > "$OUT/immutable-paths.txt" || : > "$OUT/immutable-paths.txt"
      if [ ! -s "$OUT/immutable-paths.txt" ]; then
        # ⛔ AN EMPTY IMMUTABLE SET IS A REFUSAL, not a pass. A manifest this
        # cannot read would otherwise make every published byte mutable, which
        # is this check reporting green over the thing it exists to catch.
        note "the regenerated MANIFEST.json names no immutable artefact, so nothing could be compared"
      fi

      CHANGED=0
      DERIVED_CHANGED=0
      while IFS= read -r p; do
        [ -n "$p" ] || continue
        if git show "$REF:$p" 2>/dev/null | cmp -s - "$OUT/a/$p"; then
          continue
        fi
        if grep -qx -- "$p" "$OUT/immutable-paths.txt"; then
          CHANGED=$((CHANGED + 1))
        else
          DERIVED_CHANGED=$((DERIVED_CHANGED + 1))
        fi
      done < "$OUT/published-paths.txt"

      # ⛔ EVERY PUBLISHED ARTEFACT LINE STILL PRESENT, UNCHANGED. This is what
      # makes the derived files safe to treat as expected-to-differ: a digest
      # that moved shows up here even though the file it lives in was allowed
      # to change.
      # ⚠ OVER THE IMMUTABLE PATHS ONLY, for the same reason as above: an
      # aggregate's digest moves whenever the corpus grows, so comparing every
      # line would report a rewrite on every capture.
      SUMS_LOST=0
      if git show "$REF:SHA256SUMS" > "$OUT/published-sums.txt" 2>/dev/null &&
        [ -f "$OUT/a/SHA256SUMS" ]; then
        awk 'NR == FNR { keep[$0] = 1; next }
             { name = $0; sub(/^[0-9a-f]+[ ]+\*?/, "", name); if (name in keep) print }' \
          "$OUT/immutable-paths.txt" "$OUT/published-sums.txt" | sort > "$OUT/ps.txt"
        awk 'NR == FNR { keep[$0] = 1; next }
             { name = $0; sub(/^[0-9a-f]+[ ]+\*?/, "", name); if (name in keep) print }' \
          "$OUT/immutable-paths.txt" "$OUT/a/SHA256SUMS" | sort > "$OUT/rs.txt"
        SUMS_LOST=$(comm -23 "$OUT/ps.txt" "$OUT/rs.txt" | wc -l | tr -d ' ')
      else
        SUMS_LOST=1
      fi

      if [ "$GONE" = 0 ] && [ "$CHANGED" = 0 ] && [ "$SUMS_LOST" = 0 ] &&
        [ "$MISSING" = 0 ] && { [ "$ADDED" -gt 0 ] || [ "$DERIVED_CHANGED" -gt 0 ]; }; then
        PENDING=$((ADDED + DERIVED_CHANGED))
      else
        note "the regenerated tree is $LOCAL_TREE and $REF carries $PUBLISHED_TREE, so what is published is not what this corpus derives to"
        [ "$GONE" = 0 ] ||
          note "$GONE published path(s) are no longer produced at all, which a consumer pinning them would notice"
        [ "$CHANGED" = 0 ] ||
          note "$CHANGED published artefact(s) changed their bytes, and a published artefact is immutable"
        [ "$SUMS_LOST" = 0 ] ||
          note "$SUMS_LOST checksum line(s) the branch publishes are not in the regenerated set"
        [ "$MISSING" = 0 ] ||
          note "$MISSING path(s) the branch's own manifest lists are not on the branch, so a consumer fetching one gets a 404"
      fi
    fi
  else
    note "the regenerated tree could not be written into a temporary index, so nothing was compared"
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-data-branch/3","files":%s,"present":%s,"recorded":%s,"cases":%s,"published":"%s","matched":%s,"pending":%s,"problems":%s}\n' \
    "${FILES:-0}" "$PRESENT" "$RECORDED" "${CASES:-0}" "$PUBLISHED" \
    "$([ -n "$MATCHES" ] && echo true || echo false)" "$PENDING" "$COUNT"
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
  elif [ "$PENDING" -gt 0 ]; then
    printf '  ⚠ The %s branch is %s and BEHIND by %s artefact(s): every path it\n' \
      "$BRANCH" "$PUBLISHED" "$PENDING"
    printf '  carries is still produced and every immutable artefact is byte-identical.\n'
    printf '  The assembler adds or refreshes derived artefacts only, so this is reported\n'
    printf '  rather than failed. The publisher appends the new tree on the next push.\n'
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
printf 'that property is free. docs/history/todo/publish.md, PUB-02.\n' >&2
exit 1
