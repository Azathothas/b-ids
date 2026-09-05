#!/bin/sh
# corpus-root.sh - where is the corpus this run should read?
#
# ⛔ TWELVE CHECKS READ THE CORPUS AND EVERY ONE OF THEM ASSUMED THE WORKING
# TREE. `corpus/` and `raw/` have LEFT the default branch, and on the day they
# did, twelve green checks would have verified nothing or refused to run, with
# the tempting fix for each being to widen it until it passes.
# docs/history/todo/publish.md, PUB-11 and PUB-13.
#
# ⭐ ONE RESOLVER, NOT TWELVE. Every check asks this, and this is the only place
# the question is answered. `Store::at` is the equivalent inside the crate.
#
# -- ⛔ THE ORDER, AND WHY EACH RULE IS WHERE IT IS ---------------------------
#
#   1. $B_IDS_CORPUS_ROOT, if it is set. ⛔ An explicit root is never second
#      guessed: if it holds no corpus this exits 2 rather than falling through
#      to something the caller did not ask for.
#   2. THE WORKING TREE, if it holds corpus/v1/index.json.
#   3. ⭐ THE SOURCE BRANCH, materialised under .tmp/source-branch. This is the
#      CANONICAL corpus since PUB-13: the default branch carries neither
#      directory and the data branch is a derivation of this one.
#   4. the data branch, materialised under .tmp/data-branch.
#
# ⚠ The entry proposed the branch BEFORE the working tree. That order is wrong
# while both exist: a session that adds a profile would have every check read
# the PUBLISHED corpus and report green over the one it is about to publish.
# The working tree is canonical for exactly as long as it holds a corpus.
#
# ⛔ AND 3 BEFORE 4 IS THE WHOLE OF PUB-13. The data branch is DERIVED from the
# source branch, so a resolver that answered `data-branch` first would hand
# check-data-branch the branch it is supposed to be checking, and the comparison
# would be against itself. That defect has already shipped here once, under
# PUB-11, and it reported `data branch ok`.
#
# -- ⚠ HOW A BRANCH IS MATERIALISED, AND WHY NOT tar OR A WORKTREE -----------
#
# A TEMPORARY INDEX and `git checkout-index`. It needs no tar, whose flags
# differ between the GNU and bsd builds this project has already been bitten by,
# and no pipe, which PowerShell is not byte-exact through. ⛔ It never touches
# this repository's own index, and it registers no worktree to clean up.
#
# ⛔ AND A COPY IS REUSED ONLY WHILE THE REF IT CAME FROM HAS NOT MOVED. The
# copy's ref sha is written beside it and compared on every call. Before PUB-13
# a copy was reused whenever it merely held a corpus, which was survivable while
# the fallback was a rare route and is not survivable now that it is the ONLY
# route: a session that pushes a profile to the source branch and re-runs the
# gate would otherwise have every check report on the corpus from before it.
#
# ⭐ IT DOES NOT FETCH. A gate that reaches the network is a gate that is red
# when somebody else's host is down. refs/heads/NAME then
# refs/remotes/origin/NAME, and nothing else.
#
# Usage:
#   sh scripts/common/corpus-root.sh
#   sh scripts/common/corpus-root.sh --json
#   sh scripts/common/corpus-root.sh --ref
#   sh scripts/common/corpus-root.sh --source
#   sh scripts/common/corpus-root.sh --fixture
#
# --fixture asserts the fallbacks actually resolve, and asserts their ORDER: it
# builds a tree with no corpus in it, requires the answer to be the SOURCE
# branch, and then materialises the DATA branch on its own so that the rule the
# order hides is still driven rather than reasoned about.
#
# Exit codes: 0 a root was resolved and its path is on stdout, 2 none was.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURE=0
REF_ONLY=0
SOURCE_ONLY=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --ref) REF_ONLY=1 ;;
    --source) SOURCE_ONLY=1 ;;
    --fixture) FIXTURE=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'corpus-root: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'corpus-root: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'corpus-root: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ THE BRANCHES ARE NAMED ONCE, IN ORDER, and the twin carries the same list.
# A rename moves both halves and nothing else in this file knows a branch name.
BRANCHES='source data'
MARK=corpus/v1/index.json

# ⭐ THE ONE TEST FOR "IS THERE A CORPUS HERE", so four call sites cannot
# answer it four ways.
holds_corpus() {
  [ -f "$1/$MARK" ]
}

# ⛔ MATERIALISE THROUGH A TEMPORARY INDEX. `checkout-index` writes the tree the
# ref carries and nothing else, and the real index is never opened.
materialise() {
  ref="$1"
  dest="$2"
  idx="$dest.index"
  rm -rf "$dest" "$idx"
  mkdir -p "$dest" || return 1
  GIT_INDEX_FILE="$idx" git read-tree "$ref" 2>/dev/null || { rm -f "$idx"; return 1; }
  GIT_INDEX_FILE="$idx" git checkout-index -a --prefix="$dest/" 2>/dev/null || { rm -f "$idx"; return 1; }
  rm -f "$idx"
  return 0
}

branch_ref() {
  if git rev-parse -q --verify "refs/heads/$1" >/dev/null 2>&1; then
    printf 'refs/heads/%s\n' "$1"
  elif git rev-parse -q --verify "refs/remotes/origin/$1" >/dev/null 2>&1; then
    printf 'refs/remotes/origin/%s\n' "$1"
  fi
}

# ⛔ A CACHED COPY IS ONLY AS GOOD AS THE SHA IT CAME FROM. The stamp beside the
# copy is what makes reuse safe; with no stamp the copy is rebuilt.
copy_of() {
  ref="$1"
  dest="$2"
  want=$(git rev-parse -q --verify "$ref^{commit}" 2>/dev/null) || return 1
  have=""
  [ -f "$dest.ref" ] && have=$(cat "$dest.ref" 2>/dev/null)
  if [ "$have" != "$want" ] || ! holds_corpus "$dest"; then
    materialise "$ref" "$dest" || return 1
    printf '%s\n' "$want" > "$dest.ref" || return 1
  fi
  holds_corpus "$dest"
}

resolve() {
  root="$1"
  # 1. explicit
  if [ -n "${B_IDS_CORPUS_ROOT:-}" ]; then
    if holds_corpus "$B_IDS_CORPUS_ROOT"; then
      RESOLVED="$B_IDS_CORPUS_ROOT"
      SOURCE=explicit
      FROM_REF=""
      return 0
    fi
    printf 'corpus-root: B_IDS_CORPUS_ROOT=%s holds no %s\n' "$B_IDS_CORPUS_ROOT" "$MARK" >&2
    return 2
  fi
  # 2. the working tree
  if holds_corpus "$root"; then
    RESOLVED="$root"
    SOURCE=working-tree
    FROM_REF=""
    return 0
  fi
  # 3 and 4. each branch in turn, materialised
  tried=""
  for branch in $BRANCHES; do
    ref=$(branch_ref "$branch")
    if [ -z "$ref" ]; then
      tried="$tried $branch(no ref)"
      continue
    fi
    dest="$root/.tmp/$branch-branch"
    if copy_of "$ref" "$dest"; then
      RESOLVED="$dest"
      SOURCE="$branch-branch"
      FROM_REF="$ref"
      return 0
    fi
    tried="$tried $branch(no $MARK)"
  done
  printf 'corpus-root: no corpus in the working tree, and no branch carried one:%s\n' "$tried" >&2
  return 2
}

if [ "$FIXTURE" = 1 ]; then
  # ⛔ THE FALLBACKS ARE DRIVEN RATHER THAN REASONED ABOUT. Since PUB-13 the
  # SOURCE branch is the everyday route, so this fixture's job has shifted: what
  # it now protects is the ORDER and the route the order HIDES. A data branch
  # that stopped being materialisable would otherwise be invisible until the day
  # the source branch went missing.
  SRC_REF=$(branch_ref source)
  DATA_REF=$(branch_ref data)
  if [ -z "$SRC_REF" ]; then
    printf 'corpus-root: no local ref for source, so the fallback cannot be driven\n' >&2
    exit 2
  fi
  FIX="$REPO_ROOT/.tmp/corpus-root-fixture"
  rm -rf "$FIX"
  mkdir -p "$FIX" || { printf 'corpus-root: cannot create %s\n' "$FIX" >&2; exit 2; }
  if holds_corpus "$FIX"; then
    printf 'corpus-root: the fixture tree already holds a corpus, so it proves nothing\n' >&2
    exit 2
  fi
  B_IDS_CORPUS_ROOT=""
  unset B_IDS_CORPUS_ROOT
  RESOLVED=""
  SOURCE=""
  FROM_REF=""
  resolve "$FIX" || { printf 'corpus-root: the fallback did not resolve\n' >&2; exit 2; }
  if [ "$SOURCE" != "source-branch" ]; then
    printf 'corpus-root: the fixture resolved %s where source-branch was expected\n' "$SOURCE" >&2
    exit 2
  fi
  COUNT=$(find "$RESOLVED/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')
  # ⛔ AND THE HIDDEN ROUTE, driven on its own. The resolver will never reach it
  # while the source branch answers, so nothing else in this tree exercises it.
  DATA_COUNT=0
  if [ -n "$DATA_REF" ]; then
    if copy_of "$DATA_REF" "$FIX/.tmp/data-branch"; then
      DATA_COUNT=$(find "$FIX/.tmp/data-branch/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')
    else
      rm -rf "$FIX"
      printf 'corpus-root: the data branch exists and could not be materialised\n' >&2
      exit 2
    fi
  fi
  rm -rf "$FIX"
  if [ "${COUNT:-0}" -lt 1 ]; then
    printf 'corpus-root: the materialised source branch carried no profile\n' >&2
    exit 2
  fi
  if [ -n "$DATA_REF" ] && [ "${DATA_COUNT:-0}" -lt 1 ]; then
    printf 'corpus-root: the materialised data branch carried no profile\n' >&2
    exit 2
  fi
  printf 'corpus-root fixture ok: a tree with no corpus resolves to the source branch,\n'
  printf 'carrying %s profile(s). ⛔ The data branch is reachable and NOT chosen: it\n' "$COUNT"
  if [ -n "$DATA_REF" ]; then
    printf 'materialises to %s profile(s) when asked for directly.\n' "$DATA_COUNT"
  else
    printf 'has no local ref here, so only the order was proved.\n'
  fi
  exit 0
fi

RESOLVED=""
SOURCE=""
FROM_REF=""
resolve "$REPO_ROOT" || exit 2

# ⭐ THE REF THE ANSWER CAME FROM, empty for the working tree and for an
# explicit root. check-corpus asks for it because its own question is about a
# HISTORY: now that the corpus is only on a branch, the history to read is that
# branch's and not this repository's HEAD. docs/history/todo/publish.md, PUB-11 and PUB-13.
if [ "$REF_ONLY" = 1 ]; then
  printf '%s\n' "$FROM_REF"
  exit 0
fi

# ⛔ WHICH OF THE FOUR ANSWERED, WHICH IS NOT THE SAME QUESTION AS --ref.
# `--ref` is empty for TWO different reasons: the working tree answered, or the
# caller named a root explicitly. A check that reads an empty ref as "the
# working tree is canonical" is therefore wrong whenever B_IDS_CORPUS_ROOT is
# set, and check-data-branch exported that variable on the line ABOVE its own
# guard, which disarmed it. ⚠ It reported `data branch ok` while comparing the
# branch against itself. docs/history/todo/publish.md, PUB-11.
if [ "$SOURCE_ONLY" = 1 ]; then
  printf '%s\n' "$SOURCE"
  exit 0
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"corpus-root/2","source":"%s","ref":"%s","profiles":%s}\n' \
    "$SOURCE" "$FROM_REF" \
    "$(find "$RESOLVED/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')"
  exit 0
fi

# ⛔ THE PATH ALONE ON STDOUT, with no trailing text, because every caller reads
# this through a command substitution. Anything else here is a value the caller
# has to strip, which is the defect check-routes exists for.
printf '%s\n' "$RESOLVED"
exit 0
