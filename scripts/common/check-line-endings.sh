#!/bin/sh
# check-line-endings.sh - do the index AND the working tree carry the line
# endings this repository declares?
#
# The defect this exists to catch is a file that is CRLF on disk in a tree that
# declares LF. The rule used to live inline in check-gate and it read git's
# INDEX column ALONE, so a tracked file that is CRLF in the working tree and LF
# in the index passed it. Eight files became CRLF that way in one session and
# the gate stayed green throughout: `.gitattributes` normalises on commit, so
# nothing reached the history and nothing said the working tree was wrong.
# docs/history/todo/tooling.md, TOOL-17.
#
# -- ⛔ TWO COLUMNS, AND THEY ARE DIFFERENT FACTS ------------------------------
#
# `git ls-files --eol` reports both. The INDEX column says what a commit will
# contain; the WORKING-TREE column says what is on disk right now, which is what
# an editor, a compiler and Windows PowerShell 5.1 actually read.
#
# -- ⭐ THE RULE IS WHAT THE ATTRIBUTES DECLARE, NEVER A FIXED VALUE ------------
#
# Measured on this tree 2026-09-01: 84 files are CRLF on disk on purpose, and
# every one of them is a `.ps1` declaring `eol=crlf`, because Windows PowerShell
# 5.1 mis-parses a here-string whose terminator arrives with a bare LF.
# docs/conventions/shell.md section 8.
#
# ⚠ A rule matching `*.ps1` here would be a second answer to a question git
# already answers, and it would be wrong: the reference corpus carries its own
# `.gitattributes` files, so a `.ps1` under `references/` resolves through the
# nested one rather than through this repository's.
#
# What is out of scope, and why each is:
#
#   attr/-text      the bytes ARE the content, so no translation may apply
#   i/-text, w/-text   git detected binary content whatever the attributes say
#   i/none w/none   no line ending at all: an empty file, or a single value with
#                   no trailing newline. That second shape is one this project
#                   PUBLISHES deliberately, per PUB-03, and a filter that
#                   refused it would refuse the shape the requirement asks for.
#   no declared eol nothing to compare a working tree against
#
# ⛔ NO TRACKED FILE AT ALL IS EXIT 2. A check reporting clean over nothing is how
# it quietly stops applying, which is the rule check-routes already carries.
#
# Usage:
#   sh scripts/common/check-line-endings.sh
#   sh scripts/common/check-line-endings.sh --json
#
# Exit codes: 0 clean, 1 a file disagrees with what it declares, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-line-endings: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-line-endings: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-line-endings: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ Every path below is relative to the repository root, so the scope of the
# check does not depend on who called it.
cd "$REPO_ROOT" || { printf 'check-line-endings: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

EOL=$(git ls-files --eol)
FILES=$(printf '%s\n' "$EOL" | awk 'NF' | wc -l | tr -d ' ')

if [ "$FILES" = 0 ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-line-endings/1","files":0,"index":0,"worktree":0,"problems":0}\n'
  else
    printf 'check-line-endings: git tracks no file here, so nothing was checked.\n' >&2
  fi
  exit 2
fi

# ⛔ Keep this awk program identical to the PowerShell twin's filter.
BAD=$(printf '%s\n' "$EOL" | awk '
  $3 == "attr/-text" { next }
  $1 == "i/-text" || $2 == "w/-text" { next }
  $1 != "i/lf" && $1 != "i/none" && $1 != "i/" { print "index    " $0; next }
  $2 == "w/none" { next }
  $4 == "eol=crlf" { if ($2 != "w/crlf") print "worktree " $0; next }
  $4 == "eol=lf" { if ($2 != "w/lf") print "worktree " $0; next }
')

INDEX_BAD=$(printf '%s\n' "$BAD" | awk '$1 == "index"' | wc -l | tr -d ' ')
WORKTREE_BAD=$(printf '%s\n' "$BAD" | awk '$1 == "worktree"' | wc -l | tr -d ' ')
PROBLEMS=$((INDEX_BAD + WORKTREE_BAD))

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-line-endings/1","files":%s,"index":%s,"worktree":%s,"problems":%s}\n' \
    "$FILES" "$INDEX_BAD" "$WORKTREE_BAD" "$PROBLEMS"
  [ "$PROBLEMS" -gt 0 ] && exit 1
  exit 0
fi

if [ "$PROBLEMS" -gt 0 ]; then
  printf 'line-ending check failed, %s file(s) over %s tracked:\n\n' "$PROBLEMS" "$FILES"
  printf '%s\n' "$BAD" | sed 's/^/  /'
  printf '\n'
  printf 'An "index" finding is what a commit would contain and is fixed by\n'
  printf 'renormalising. A "worktree" finding is what is on disk and reaches no\n'
  printf 'commit, which is exactly why nothing else notices it.\n'
  exit 1
fi

printf 'line endings ok: %s tracked file(s), index and working tree both agree\n' "$FILES"
printf 'with what .gitattributes declares.\n'
exit 0
