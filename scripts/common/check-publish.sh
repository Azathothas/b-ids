#!/bin/sh
# check-publish.sh - does the workflow that publishes this project declare what
# it must, and do the rules it defers to actually refuse?
#
# ⛔ NOTHING IN THIS REPOSITORY WAS EVER PUBLISHED UNTIL A TRIGGER EXISTED, and
# the first thing a trigger can get wrong is irreversible: a force push over the
# data branch discards every commit a consumer pinned. docs/history/todo/publish.md, PUB-10.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the workflow exists and declares all THREE triggers the operator ruled:
#      a manual dispatch, a push to the default branch, and a pushed tag;
#   2. ⛔ THE WRITE IS JOB-SCOPED. The top of the file grants `contents: read`
#      and exactly the publishing jobs raise it, so the job that decides whether
#      a push may happen cannot itself push;
#   3. ⛔ NO PERSONAL ACCESS TOKEN. The word `secrets.` does not appear: the run
#      carries its own token and nothing else;
#   4. ⛔ NO FORCE PUSH. No `git push` line carries `--force`,
#      `--force-with-lease` or a `+` refspec;
#   5. the crate's rule is consulted BEFORE the push, by line order in the file,
#      so a step added between them cannot slip past it;
#   6. both publishing jobs need the job that runs check-release and
#      check-data-branch, so a tree that fails either publishes nothing;
#   7. the archive epoch is READ from check-release.sh rather than typed here,
#      because a second copy of it is a value in two places;
#   8. ⭐ THE RULES ACTUALLY REFUSE. Release and data-branch cases are driven
#      against the built binary, and each exit code is read from the process
#      that produced it. A guard whose test has never failed is theatre.
#
# -- ⚠ WHAT IT DOES NOT DO ---------------------------------------------------
#
# It is not a YAML parser and it does not simulate a run. It reads the block
# structure this file actually uses, which is the same trade check-workflows
# makes and for the same reason: the CI step that runs a real YAML library over
# every workflow is what proves they parse.
#
# ⚠ THE FORCE-PUSH RULE READS ONE LINE AT A TIME. A `git push` split across a
# backslash continuation would hide a flag from it, and no such line exists in
# this tree; a check that tried to reassemble shell continuations would be a
# shell parser.
#
# ⛔ AN ABSENT WORKFLOW IS EXIT 1, NOT 2. Everywhere else in this tree a missing
# subject means "could not run", because the scope was discovered. Here the path
# is fixed and named, and the file not being there is precisely the defect this
# check exists to catch.
#
# Usage:
#   sh scripts/common/check-publish.sh
#   sh scripts/common/check-publish.sh --json
#
# Exit codes: 0 the workflow declares what it must and the rules refuse,
# 1 one of them does not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-publish: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-publish: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-publish: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-publish: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. docs/history/todo/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-publish: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"
command -v cargo >/dev/null 2>&1 || { printf 'check-publish: cargo not found\n' >&2; exit 2; }

WF=".github/workflows/publish.yml"
# ⛔ THE TWO JOBS THAT MAY WRITE, named here and asserted there.
PUBLISHING="data-branch release"

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

TRIGGERS=0
JOBS=0
CASES=0

if [ ! -f "$WF" ]; then
  note "there is no $WF, so nothing in this repository publishes anything"
  printf 'publish check failed, %s problem(s):\n\n' "$COUNT" >&2
  printf '%s\n' "$PROBLEMS" >&2
  exit 1
fi

# -- 1: the three triggers ---------------------------------------------------
#
# ⚠ READ INSIDE THE `on:` BLOCK. `workflow_dispatch` appearing anywhere in a
# file says nothing about whether it is a trigger.
ON=$(awk '/^on:[ \t]*$/ { inside = 1; next } inside && /^[a-zA-Z]/ { inside = 0 } inside { print }' "$WF")
# ⚠ IF AND ELSE RATHER THAN `A && B || C`. The two are not the same thing: C
# runs when B fails as well as when A does, and an arithmetic assignment that
# ever evaluated to a failure would turn a present trigger into a missing one.
# ⛔ shellcheck SC2015 is a gate here and it caught this shape on the day it was
# written.
trigger() {
  t_why=$2
  if printf '%s\n' "$ON" | grep -qE "$1"; then
    TRIGGERS=$((TRIGGERS + 1))
  else
    note "$t_why"
  fi
}
trigger '^  workflow_dispatch:' \
  "the workflow does not declare workflow_dispatch, which is one of the three triggers"
trigger '^    branches:.*main' \
  "the workflow does not trigger on a push to the default branch"
trigger '^    tags:' \
  "the workflow does not trigger on a pushed tag, so no release is ever cut"
printf '%s\n' "$ON" | grep -qF "tags: ['v0.0.1', 'v1.*']" ||
  note "the tag trigger does not admit exactly v0.0.1 and the dated v1 series"

# -- 2: the write is job-scoped ----------------------------------------------
#
# ⛔ THE TOP OF THE FILE GRANTS READ. A workflow with a write at the top hands
# it to every job in it, including one that downloads an artefact.
TOP=$(awk '/^permissions:[ \t]*$/ { inside = 1; next } inside && /^[a-zA-Z]/ { inside = 0 } inside { print }' "$WF")
printf '%s\n' "$TOP" | grep -qE '^  contents: read[ \t]*$' ||
  note "the top-level permissions block does not grant contents: read and nothing else"
printf '%s\n' "$TOP" | grep -q 'write' &&
  note "the top-level permissions block grants a write, which every job would then hold"

WRITES=$(grep -cE '^      contents: write[ \t]*$' "$WF")
WANT_WRITES=0
for _job in $PUBLISHING; do WANT_WRITES=$((WANT_WRITES + 1)); done
[ "$WRITES" = "$WANT_WRITES" ] ||
  note "$WRITES job(s) declare contents: write where $WANT_WRITES publishing job(s) are expected"

# The jobs, read from the block structure rather than from a glob.
JOB_NAMES=$(awk '/^jobs:[ \t]*$/ { inside = 1; next } inside && /^[a-zA-Z]/ { inside = 0 }
  inside && /^  [A-Za-z0-9_-]+:[ \t]*$/ { name = $1; sub(/:$/, "", name); print name }' "$WF")
JOBS=$(printf '%s\n' "$JOB_NAMES" | awk 'NF' | wc -l | tr -d ' ')
for want in $PUBLISHING; do
  printf '%s\n' "$JOB_NAMES" | grep -qx "$want" ||
    note "there is no $want job in $WF"
done

# -- 3: no personal access token ---------------------------------------------
grep -q 'secrets\.' "$WF" &&
  note "the workflow names a secret. The write is the run's own token and never a personal access token"

# -- 4: no force push --------------------------------------------------------
#
# ⛔ THE DATA BRANCH IS APPEND-ONLY. A comment may say the words; a command may
# not carry them, so the comment lines are dropped first.
# ⛔ ANY `+` ON A `git push` LINE IS A FORCE, and the rule is that blunt because
# the first version was not. It looked for `:+`, which is where a `+` is NOT: a
# forcing refspec is `+src:dst`, so the plus sits at the START of the token. The
# mutation that gave the push a leading `+` passed this check, and a guard whose
# test has never been seen to fail is theatre. A `git push` line takes a remote
# and refspecs, and a `+` in any of them means force.
LIVE=$(sed 's/^[ \t]*#.*$//' "$WF")
FORCED=$(printf '%s\n' "$LIVE" | grep 'git push' | grep -cE -- '--force|\+')

# ⛔ AND THE SAME RULE OVER EVERY OTHER WORKFLOW, because a control gated on one
# path and not its siblings is the single most recurring hole there is. This
# check began by reading publish.yml alone while capture.yml carried a
# `--force-with-lease` that nothing in the tree asserted anything about. Found by
# sweeping every write path rather than by reading this file.
#
# ⚠ THE COUNT IS PINNED rather than the flag banned. One force push exists on
# purpose: CI-04's bot branch, which is force-pushed with a lease after checking
# the last commit's author. A SECOND one is a thing somebody has to look at.
ALL_PUSHES=$(sed 's/^[ \t]*#.*$//' .github/workflows/*.yml | grep 'git push' || true)
ALL_FORCED=$(printf '%s\n' "$ALL_PUSHES" | grep -cE -- '--force|\+')
[ "$ALL_FORCED" = 1 ] ||
  note "$ALL_FORCED git push line(s) across every workflow force, and exactly one may: CI-04's bot branch"
BAD_TARGET=$(printf '%s\n' "$ALL_PUSHES" | grep -E -- '--force|\+' | grep -cE 'refs/heads/data|refs/heads/main|origin[ \t]+main')
[ "$BAD_TARGET" = 0 ] ||
  note "$BAD_TARGET force push(es) name the data branch or the default branch, and neither is ever force-pushed"
[ "$FORCED" = 0 ] ||
  note "$FORCED git push line(s) carry a force flag or a + refspec, and the data branch is append-only"

# -- 5: the rule is consulted before the push --------------------------------
RULE_AT=$(printf '%s\n' "$LIVE" | grep -n -- '-- data-branch' | head -1 | cut -d: -f1)
PUSH_AT=$(printf '%s\n' "$LIVE" | grep -n 'git push origin' | head -1 | cut -d: -f1)
if [ -z "${RULE_AT:-}" ]; then
  note "no step calls b-ids-corpus data-branch, so nothing asks the crate whether the push appends"
elif [ -z "${PUSH_AT:-}" ]; then
  note "no step pushes the data branch, so this workflow publishes only a release"
elif [ "$RULE_AT" -ge "$PUSH_AT" ]; then
  note "the push at line $PUSH_AT comes before the rewrite rule at line $RULE_AT"
fi

# -- 6: both publishing jobs need the job that checks -------------------------
for check in check-release.sh check-data-branch.sh; do
  grep -q "$check" "$WF" ||
    note "$check is not run by the workflow, so a tree that fails it would still publish"
done
NEEDS=$(grep -cE '^    needs: \[assemble\][ \t]*$' "$WF")
[ "$NEEDS" = "$WANT_WRITES" ] ||
  note "$NEEDS job(s) need the assemble job where $WANT_WRITES publishing job(s) are expected"

# -- 7: the archive epoch is derived -----------------------------------------
EPOCH=$(awk -F'"' '/^TAR_EPOCH=/{ print $2; exit }' scripts/common/check-release.sh)
[ -n "${EPOCH:-}" ] || note "check-release.sh no longer states TAR_EPOCH, which the workflow reads"
grep -q 'TAR_EPOCH=' "$WF" ||
  note "the workflow does not read TAR_EPOCH from check-release.sh, so the epoch is stated twice"
printf '%s\n' "$LIVE" | grep -q -- "--mtime \"\$epoch\"" ||
  note "the workflow's tar does not use the epoch it read"

# -- 8: the rules actually refuse --------------------------------------------
#
# ⛔ A GUARD WHOSE TEST HAS NEVER BEEN SEEN TO FAIL IS THEATRE. Each exit code
# below is read from the process that produced it, with no pipe.
SUITE="$REPO_ROOT/crates/b-ids-corpus/tests/publish.rs"
[ -f "$SUITE" ] || { printf 'check-publish: no suite at %s\n' "$SUITE" >&2; exit 2; }
for want in publish_a_push_that_would_rewrite_the_data_branch_is_refused \
  publish_a_tag_this_rule_did_not_produce_is_refused \
  publish_the_tree_names_no_path_outside_itself; do
  grep -q "fn $want" "$SUITE" || note "$want is not in the suite"
done

OUT="$REPO_ROOT/.tmp/check-publish"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-publish: cannot create %s\n' "$OUT" >&2; exit 2; }
cargo build -q -p b-ids-corpus || {
  printf 'check-publish: the corpus crate did not build\n' >&2
  exit 2
}
TARGET_DIR=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
BIN="$TARGET_DIR/debug/b-ids-corpus"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-publish: %s is not executable\n' "$BIN" >&2; exit 2; }

# ⛔ READ FROM THE PROCESS, UNPIPED, EVERY TIME.
drive() {
  d_want=$1
  d_why=$2
  shift 2
  "$BIN" "$@" > "$OUT/drive.log" 2>&1
  d_rc=$?
  CASES=$((CASES + 1))
  [ "$d_rc" = "$d_want" ] ||
    note "$d_why: exit $d_rc where $d_want was expected"
}

drive 0 'the first push creates the branch' data-branch --head none --parent none
drive 0 'a commit built on the branch head appends' data-branch --head abc123 --parent abc123
drive 1 'a commit built on something the branch moved past is a rewrite' \
  data-branch --head abc123 --parent def456
drive 1 'an orphan commit pushed over an existing branch discards every commit on it' \
  data-branch --head abc123 --parent none
drive 2 'data-branch with no --parent must not answer append' data-branch --head abc123

"$BIN" publish --root "$CORPUS_ROOT" --out "$OUT/tree" > "$OUT/publish.log" 2>&1
rc_p=$?
if [ "$rc_p" != 0 ]; then
  printf 'check-publish: the assembler exited %s\n' "$rc_p" >&2
  cat "$OUT/publish.log" >&2
  exit 1
fi

drive 0 'a well-formed tag over an assembled tree is releasable' \
  release --tree "$OUT/tree" --tag v1.2026.01.01.1 --notes "$OUT/NOTES.md"
drive 0 'the explicit bootstrap tag is releasable once' \
  release --tree "$OUT/tree" --tag v0.0.1
drive 1 'a zero-padded counter is not the tag this rule produces' \
  release --tree "$OUT/tree" --tag v1.2026.01.01.01
drive 1 'a malformed date is refused' release --tree "$OUT/tree" --tag v1.2026.1.1.1
drive 1 'a tag naming another layout is refused' release --tree "$OUT/tree" --tag v9.2026.01.01.1
printf 'v1.2026.01.01.1\n' > "$OUT/released.txt"
drive 1 'a tag that already carries a release is refused' \
  release --tree "$OUT/tree" --tag v1.2026.01.01.1 --existing "$OUT/released.txt"
printf 'v0.0.1\n' > "$OUT/released.txt"
drive 1 'the bootstrap release is immutable once published' \
  release --tree "$OUT/tree" --tag v0.0.1 --existing "$OUT/released.txt"
drive 1 'another conventional semantic version is not a corpus release tag' \
  release --tree "$OUT/tree" --tag v0.0.2

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-publish/1","triggers":%s,"jobs":%s,"writes":%s,"cases":%s,"problems":%s}\n' \
    "$TRIGGERS" "$JOBS" "$WRITES" "$CASES" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'publish ok: %s trigger(s) over %s job(s), %s job-scoped write(s),\n' \
    "$TRIGGERS" "$JOBS" "$WRITES"
  printf '  no force push and no named secret, and %s refusal(s) driven against the binary.\n' "$CASES"
  printf '  ⛔ Nothing was tagged, uploaded or pushed.\n'
  exit 0
fi

printf 'publish check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A force push over the data branch discards every commit a consumer\n' >&2
printf 'pinned. docs/history/todo/publish.md, PUB-10.\n' >&2
exit 1
