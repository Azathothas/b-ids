#!/bin/sh
# check-catalogues.sh - is every script and every document named by the
# catalogue that claims to list it?
#
# ⛔ A CATALOGUE NOTHING CHECKS STOPS BEING A CATALOGUE. docs/AGENTS.md sends a
# session writing a script to scripts/README.md, calling it the contract every
# script is held to, and sends a session writing a document to its own table of
# what each one owns. Neither was compared against the tree, so both drifted:
# measured 2026-09-03, THIRTEEN of the checks the gate runs had no section in
# scripts/README.md at all, and the gate was green throughout.
# TODO/tooling.md, TOOL-19.
#
# -- ⛔ THE TWO RULES, AND THEY ARE THE ONLY TWO ------------------------------
#
#   1. every script under scripts/ is named by scripts/README.md. Twins collapse
#      to ONE base name, because a pair is one contract and one section;
#   2. every document under docs/ is named by its index: docs/AGENTS.md for the
#      tree, docs/HISTORY/README.md for the history directory, which has its own
#      because a superseded page is not routed to.
#
# ⛔ IT DOES NOT READ THE PROSE. Whether a section is any good is a review, and
# a guard that tried to judge one would either pass vacuously or refuse
# legitimate writing. What it holds is that the row EXISTS.
#
# ⚠ THE OTHER DIRECTION IS ALREADY HELD. check-docs resolves every relative link
# and every cited path in every markdown file in this tree, so a catalogue
# naming a file the tree does not have fails there rather than here. Two checks
# holding one rule is two places for it to be wrong.
#
# -- ⚠ WHY A DOCUMENT IS MATCHED BY ITS PATH AND A SCRIPT BY ITS NAME ---------
#
# A document is cited as `methodology/gate.md`, relative to the index that names
# it, and two documents can share a base name in different directories. A script
# is cited as `common/check-gate.sh` in one place and `check-gate` in another,
# and its two halves differ only by extension, so the base name is the unit the
# contract is written against.
#
# -- ⛔ AN EMPTY SCOPE IS EXIT 2 ----------------------------------------------
#
# A check reporting clean over nothing is how it quietly stops applying, which
# is the rule check-routes carries and the shape that was found there by the
# fixture written to prove it could refuse.
#
# Usage:
#   sh scripts/common/check-catalogues.sh
#   sh scripts/common/check-catalogues.sh --json
#   sh scripts/common/check-catalogues.sh --fixture
#   sh scripts/common/check-catalogues.sh --fixtures DIR
#
# --fixture asserts that the check CAN fail: it builds a tree in which one
# script and one document are missing from their catalogues and requires both to
# be refused. --fixtures runs the same scan over ANOTHER tree, walking the
# filesystem rather than asking git, which is how a session points it at an
# earlier checkout of this repository and sees what it would have said.
#
# Exit codes: 0 every one is named, 1 one is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURE=0
FIXTURES=""

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --fixture) FIXTURE=1 ;;
    --fixtures) shift; FIXTURES="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-catalogues: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-catalogues: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-catalogues: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-catalogues: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⛔ TRACKED PLUS UNTRACKED-BUT-NOT-IGNORED, which check-docs already does and
# this had to learn by running: a script written this minute is not in the index
# yet, so `git ls-files` alone reports a clean catalogue over the one file most
# likely to be missing from it. Found on this check's own first run.
listed() {
  git ls-files -- "$1" 2>/dev/null
  git ls-files --others --exclude-standard -- "$1" 2>/dev/null
}

# ⭐ THE BASE NAME OF EVERY SCRIPT, twins collapsed. Keep this identical to the
# PowerShell twin's projection.
script_names() {
  if [ "$1" = git ]; then
    listed 'scripts/*'
  else
    find scripts -type f 2>/dev/null
  fi | grep -E '\.(sh|ps1|mjs)$' |
    sed -e 's|.*/||' -e 's|\.sh$||' -e 's|\.ps1$||' -e 's|\.mjs$||' |
    sort -u
}

# ⭐ EVERY DOCUMENT WITH THE INDEX THAT OWNS IT, one per line, as
# `<index>|<name>`. ⛔ The two indexes name themselves and are skipped: a file
# listing itself proves nothing about whether anything routes to it.
doc_pairs() {
  if [ "$1" = git ]; then
    listed 'docs/*.md'
  else
    find docs -type f -name '*.md' 2>/dev/null
  fi | sort -u | while IFS= read -r p; do
    case "$p" in
      docs/AGENTS.md | docs/HISTORY/README.md) continue ;;
      docs/HISTORY/*) printf 'docs/HISTORY/README.md|%s\n' "${p#docs/HISTORY/}" ;;
      *) printf 'docs/AGENTS.md|%s\n' "${p#docs/}" ;;
    esac
  done
}

PROBLEMS=""
COUNT=0
report() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

# ⛔ ONE SCAN, USED BY THE REAL RUN AND BY THE FIXTURE, so what the fixture
# proves is what the real run does. A second implementation for the fixture
# would prove the fixture.
scan() {
  MODE="$1"
  NSCRIPTS=0
  NDOCS=0

  SCRIPT_INDEX=scripts/README.md
  if [ ! -f "$SCRIPT_INDEX" ]; then
    printf 'check-catalogues: %s is missing\n' "$SCRIPT_INDEX" >&2
    return 2
  fi

  for name in $(script_names "$MODE"); do
    NSCRIPTS=$((NSCRIPTS + 1))
    grep -qF "$name" "$SCRIPT_INDEX" ||
      report "$name has no mention in $SCRIPT_INDEX"
  done

  for pair in $(doc_pairs "$MODE"); do
    idx=${pair%%|*}
    name=${pair#*|}
    NDOCS=$((NDOCS + 1))
    if [ ! -f "$idx" ]; then
      report "$name is owned by $idx, which is missing"
      continue
    fi
    grep -qF "$name" "$idx" ||
      report "$name has no mention in $idx"
  done

  if [ "$NSCRIPTS" = 0 ] || [ "$NDOCS" = 0 ]; then
    printf 'check-catalogues: scope is empty (%s script(s), %s document(s))\n' \
      "$NSCRIPTS" "$NDOCS" >&2
    return 2
  fi
  return 0
}

if [ "$FIXTURE" = 1 ]; then
  FIX="${TMPDIR:-/tmp}/.checkcatalogues.$$"
  rm -rf "$FIX"
  mkdir -p "$FIX/scripts/common" "$FIX/docs/methodology" ||
    { printf 'check-catalogues: cannot write to %s\n' "$FIX" >&2; exit 2; }
  trap 'rm -rf "$FIX"' EXIT INT TERM

  printf 'catalogue naming alpha and nothing else\n' > "$FIX/scripts/README.md"
  printf '#!/bin/sh\n' > "$FIX/scripts/common/alpha.sh"
  printf '#!/bin/sh\n' > "$FIX/scripts/common/beta.sh"
  printf 'router naming methodology/one.md and nothing else\n' > "$FIX/docs/AGENTS.md"
  printf '# one\n' > "$FIX/docs/methodology/one.md"
  printf '# two\n' > "$FIX/docs/methodology/two.md"

  cd "$FIX" || { printf 'check-catalogues: cannot enter %s\n' "$FIX" >&2; exit 2; }
  scan find
  rc=$?
  cd "$REPO_ROOT" || exit 2

  if [ "$rc" = 2 ]; then
    printf 'check-catalogues: the fixture scan could not run\n' >&2
    exit 2
  fi
  if [ "$COUNT" != 2 ]; then
    printf 'check-catalogues: the fixture expected 2 refusals and got %s.\n' "$COUNT" >&2
    printf '%s' "$PROBLEMS" >&2
    exit 2
  fi
  printf 'check-catalogues fixture ok: an unlisted script and an unlisted document\n'
  printf 'are both refused.\n'
  exit 0
fi

# ⚠ --fixtures WALKS THE FILESYSTEM instead of asking git, because a tree that
# is not this repository has no index to ask. It is how a session proves the
# refusal against a real catalogue rather than against six invented files.
if [ -n "$FIXTURES" ]; then
  case "$FIXTURES" in
    /* | [A-Za-z]:*) TARGET="$FIXTURES" ;;
    *) TARGET="$REPO_ROOT/$FIXTURES" ;;
  esac
  [ -d "$TARGET" ] || { printf 'check-catalogues: no such directory: %s\n' "$FIXTURES" >&2; exit 2; }
  cd "$TARGET" || { printf 'check-catalogues: cannot enter %s\n' "$TARGET" >&2; exit 2; }
  scan find
  rc=$?
else
  scan git
  rc=$?
fi
[ "$rc" = 2 ] && exit 2

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-catalogues/1","scripts":%s,"documents":%s,"problems":%s}\n' \
    "$NSCRIPTS" "$NDOCS" "$COUNT"
  [ "$COUNT" -gt 0 ] && exit 1
  exit 0
fi

if [ "$COUNT" -gt 0 ]; then
  printf 'catalogue check failed, %s of %s script(s) and %s document(s):\n\n' \
    "$COUNT" "$NSCRIPTS" "$NDOCS"
  printf '%s\n' "$PROBLEMS"
  printf 'A script gets a section in scripts/README.md and a document gets a row\n'
  printf 'in the index that routes to it. docs/conventions/docs.md.\n'
  exit 1
fi

printf 'catalogues ok: %s script(s) named by scripts/README.md, %s document(s)\n' \
  "$NSCRIPTS" "$NDOCS"
printf 'named by the index that owns each one.\n'
exit 0
