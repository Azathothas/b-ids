#!/bin/sh
# check-corpus.sh - is the published corpus still append-only, and does every
# profile in it still agree with itself?
#
# The defect this exists to catch is a corpus that was edited in place. A
# consumer who pinned a value then has no way to tell whether it changed, and a
# reader has no way to tell what it used to say, which is the whole difference
# between a corpus and a table somebody maintains. The premise is not this
# project's own: two published copies of one dataset, both carrying the same
# version number and both naming the same upstream, were measured holding a
# different number of entries. docs/reference-sweeps/usable.md section 9.
#
# -- ⭐ TWO LEGS, AND ONLY ONE OF THEM IS A QUESTION FOR THIS TREE -----------
#
# The working tree cannot answer whether a file was edited after it was
# published, because an edited file and a file that was always that way look
# identical. That question belongs to git and it is asked here, over the whole
# history, with no tool but git.
#
# Everything else - every profile validates, sits at the route its own keys
# derive, publishes the bytes it says it publishes, names a real profile in
# `supersedes`, and is listed in an index the tree derives to - is the same
# question `b-ids-corpus verify` answers, and it is delegated rather than
# re-implemented. ⛔ A second implementation of the layout rule in shell would
# be a second answer to where a profile lives.
#
# -- ⚠ AN ABSENT CORPUS IS EXIT 2 -------------------------------------------
#
# A tree with no corpus has neither broken these rules nor satisfied them.
# check-changelog carries the same rule for the same reason: reporting green
# over an absent thing is how a check quietly stops applying.
#
# Usage:
#   sh scripts/common/check-corpus.sh
#   sh scripts/common/check-corpus.sh --json
#
# Exit codes: 0 clean, 1 the corpus was edited or disagrees with itself,
#             2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-corpus: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-corpus: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-corpus: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ Every path below is relative to the repository root, so the scope of the
# check does not depend on who called it. check-record.sh carries the same rule
# and the same reason.
cd "$REPO_ROOT" || { printf 'check-corpus: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

CORPUS_DIR="corpus"
RAW_DIR="raw"

if [ ! -d "$CORPUS_DIR" ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-corpus/1","corpus":false,"profiles":0,"edits":0,"problems":0}\n'
  else
    printf 'check-corpus: there is no %s/ directory, so nothing was verified.\n' "$CORPUS_DIR" >&2
    printf 'The corpus is empty. TODO/corpus.md, CORPUS-01.\n' >&2
  fi
  exit 2
fi

# -- leg one: was anything ever edited or deleted after it was published -----
#
# ⛔ OVER THE WHOLE HISTORY, not the working tree. An edited file and a file
# that was always that way are identical on disk, so this is the one question
# only the history can answer, and it is why this check exists at all.
#
# ⚠ M, D and R together. A modification is the obvious one; a deletion breaks
# "never delete a superseded profile"; and a rename is a published route
# changing under a consumer who pinned it, which is the same defect wearing a
# different name.
EDITS=$(git log --diff-filter=MDR --name-status --format='commit %h' -- "$CORPUS_DIR" "$RAW_DIR" 2>/dev/null)
EDIT_COUNT=$(printf '%s' "$EDITS" | awk 'NF && $1 != "commit"' | wc -l | tr -d ' ')

# -- leg two: does every profile still agree with itself ---------------------
#
# ⚠ Delegated to the one implementation that owns the layout and the index.
# ⛔ The exit code is read from the process that produced it: a command
# substitution is not a pipe.
#
# ⛔ AND THE NUMBERS COME FROM THE FIXED STATUS LINE, never from the prose above
# it. `b-ids-corpus verify` prints `corpus=profiles:N problems:N` as its last
# line and its usage says that is the contract. check-powershell.ps1 carries the
# same discipline for the same reason: parsing prose makes every wording change
# a silent behaviour change.
PROFILES=0
PROBLEMS=0
VERIFY_RAN=0
VERIFY_OUT=""
if command -v cargo >/dev/null 2>&1; then
  VERIFY_OUT=$(cargo run -q -p b-ids-corpus -- verify --root . 2>&1)
  VERIFY_RC=$?
  STATUS_LINE=$(printf '%s\n' "$VERIFY_OUT" | awk '/^corpus=/{ line = $0 } END { print line }')
  case "$VERIFY_RC" in
    0|1)
      if [ -n "$STATUS_LINE" ]; then
        VERIFY_RAN=1
        PROFILES=$(printf '%s' "$STATUS_LINE" | sed 's/.*profiles:\([0-9]*\).*/\1/')
        PROBLEMS=$(printf '%s' "$STATUS_LINE" | sed 's/.*problems:\([0-9]*\).*/\1/')
      fi
      ;;
    # ⚠ 2 is the command's own "there is no corpus", which the directory test
    # above has already ruled out, so reaching it here means the two disagree
    # about where the corpus is. Anything else is a build that did not happen.
    *) VERIFY_RAN=0 ;;
  esac
fi

# -- report ------------------------------------------------------------------
if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-corpus/1","corpus":true,"profiles":%s,"edits":%s,"problems":%s}\n' \
    "$PROFILES" "$EDIT_COUNT" "$PROBLEMS"
  [ "$EDIT_COUNT" -gt 0 ] && exit 1
  [ "$PROBLEMS" -gt 0 ] && exit 1
  [ "$VERIFY_RAN" = 1 ] || exit 2
  exit 0
fi

if [ "$EDIT_COUNT" -gt 0 ]; then
  printf 'corpus check failed: %s published file(s) modified, deleted or renamed after\n' "$EDIT_COUNT"
  printf 'their first commit. A published profile is immutable; a correction is a NEW\n'
  printf 'profile naming the one it replaces in its supersedes field.\n\n'
  printf '%s\n' "$EDITS"
  exit 1
fi

if [ "$VERIFY_RAN" = 1 ] && [ "$PROBLEMS" -gt 0 ]; then
  printf 'corpus check failed: %s problem(s) over %s profile(s).\n\n' "$PROBLEMS" "$PROFILES"
  printf '%s\n' "$VERIFY_OUT"
  exit 1
fi

if [ "$VERIFY_RAN" = 0 ]; then
  printf 'check-corpus: the history is clean over %s and %s, and nothing was\n' "$CORPUS_DIR" "$RAW_DIR"
  printf 'edited after it was published. ⚠ The per-profile leg did NOT run: cargo is\n'
  printf 'absent or the workspace did not build, so no profile was validated.\n' >&2
  [ -n "$VERIFY_OUT" ] && printf '%s\n' "$VERIFY_OUT" >&2
  exit 2
fi

printf 'corpus ok: %s profile(s), nothing edited after publication, index and\n' "$PROFILES"
printf 'pointers agree with the tree.\n'
exit 0
