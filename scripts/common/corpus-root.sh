#!/bin/sh
# corpus-root.sh - where is the corpus this run should read?
#
# ⛔ TWELVE CHECKS READ THE CORPUS AND EVERY ONE OF THEM ASSUMED THE WORKING
# TREE. `corpus/` and `raw/` are to leave the default branch once the data
# branch carries them, and on that day twelve green checks would verify nothing
# or refuse to run, with the tempting fix for each being to widen it until it
# passes. TODO/publish.md, PUB-11.
#
# ⭐ ONE RESOLVER, NOT TWELVE. Every check asks this, and this is the only place
# the question is answered. `Store::at` is the equivalent inside the crate.
#
# -- ⛔ THE ORDER, AND WHY IT IS NOT THE ONE THE ENTRY PROPOSED --------------
#
#   1. $B_IDS_CORPUS_ROOT, if it is set. ⛔ An explicit root is never second
#      guessed: if it holds no corpus this exits 2 rather than falling through
#      to something the caller did not ask for.
#   2. THE WORKING TREE, if it holds corpus/v1/index.json.
#   3. a materialised copy of the data branch, under .tmp/data-branch.
#
# ⚠ The entry proposed the branch BEFORE the working tree. That order is wrong
# while both exist: a session that adds a profile would have every check read
# the PUBLISHED corpus and report green over the one it is about to publish.
# The working tree is canonical for exactly as long as it holds a corpus, and
# the branch is what answers once it does not.
#
# -- ⚠ HOW THE BRANCH IS MATERIALISED, AND WHY NOT tar OR A WORKTREE ---------
#
# A TEMPORARY INDEX and `git checkout-index`. It needs no tar, whose flags
# differ between the GNU and bsd builds this project has already been bitten by,
# and no pipe, which PowerShell is not byte-exact through. ⛔ It never touches
# this repository's own index, and it registers no worktree to clean up.
#
# ⭐ IT DOES NOT FETCH. A gate that reaches the network is a gate that is red
# when somebody else's host is down. refs/heads/data then refs/remotes/origin/data,
# and nothing else.
#
# Usage:
#   sh scripts/common/corpus-root.sh
#   sh scripts/common/corpus-root.sh --json
#   sh scripts/common/corpus-root.sh --ref
#   sh scripts/common/corpus-root.sh --fixture
#
# --fixture asserts the fallback actually resolves: it builds a tree with no
# corpus in it beside a materialised branch copy, and requires the answer to be
# the copy.
#
# Exit codes: 0 a root was resolved and its path is on stdout, 2 none was.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURE=0
REF_ONLY=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --ref) REF_ONLY=1 ;;
    --fixture) FIXTURE=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'corpus-root: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'corpus-root: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'corpus-root: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

BRANCH=data
MARK=corpus/v1/index.json

# ⭐ THE ONE TEST FOR "IS THERE A CORPUS HERE", so three call sites cannot
# answer it three ways.
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
  if git rev-parse -q --verify "refs/heads/$BRANCH" >/dev/null 2>&1; then
    printf 'refs/heads/%s\n' "$BRANCH"
  elif git rev-parse -q --verify "refs/remotes/origin/$BRANCH" >/dev/null 2>&1; then
    printf 'refs/remotes/origin/%s\n' "$BRANCH"
  fi
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
  # 3. the data branch, materialised
  ref=$(branch_ref)
  if [ -z "$ref" ]; then
    printf 'corpus-root: no corpus in the working tree and no local ref for %s\n' "$BRANCH" >&2
    return 2
  fi
  dest="$root/.tmp/data-branch"
  if ! holds_corpus "$dest"; then
    materialise "$ref" "$dest" || {
      printf 'corpus-root: could not materialise %s into %s\n' "$ref" "$dest" >&2
      return 2
    }
  fi
  if ! holds_corpus "$dest"; then
    printf 'corpus-root: %s carries no %s\n' "$ref" "$MARK" >&2
    return 2
  fi
  RESOLVED="$dest"
  SOURCE=data-branch
  FROM_REF="$ref"
  return 0
}

if [ "$FIXTURE" = 1 ]; then
  # ⛔ THE FALLBACK IS DRIVEN RATHER THAN REASONED ABOUT. A route that is only
  # ever taken on the day the corpus is removed is a route nobody knows works,
  # which is the shape this repository has already been bitten by twice.
  ref=$(branch_ref)
  if [ -z "$ref" ]; then
    printf 'corpus-root: no local ref for %s, so the fallback cannot be driven\n' "$BRANCH" >&2
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
  if [ "$SOURCE" != "data-branch" ]; then
    printf 'corpus-root: the fixture resolved %s where data-branch was expected\n' "$SOURCE" >&2
    exit 2
  fi
  COUNT=$(find "$RESOLVED/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')
  rm -rf "$FIX"
  if [ "${COUNT:-0}" -lt 1 ]; then
    printf 'corpus-root: the materialised branch carried no profile\n' >&2
    exit 2
  fi
  printf 'corpus-root fixture ok: a tree with no corpus resolves to the %s branch,\n' "$BRANCH"
  printf 'carrying %s profile(s).\n' "$COUNT"
  exit 0
fi

RESOLVED=""
SOURCE=""
FROM_REF=""
resolve "$REPO_ROOT" || exit 2

# ⭐ THE REF THE ANSWER CAME FROM, empty for the working tree and for an
# explicit root. check-corpus asks for it because its own question is about a
# HISTORY: once the corpus is only on the data branch, the history to read is
# that branch's and not this repository's. TODO/publish.md, PUB-11.
if [ "$REF_ONLY" = 1 ]; then
  printf '%s\n' "$FROM_REF"
  exit 0
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"corpus-root/1","source":"%s","ref":"%s","profiles":%s}\n' \
    "$SOURCE" "$FROM_REF" \
    "$(find "$RESOLVED/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json 2>/dev/null | wc -l | tr -d ' ')"
  exit 0
fi

# ⛔ THE PATH ALONE ON STDOUT, with no trailing text, because every caller reads
# this through a command substitution. Anything else here is a value the caller
# has to strip, which is the defect check-routes exists for.
printf '%s\n' "$RESOLVED"
exit 0
