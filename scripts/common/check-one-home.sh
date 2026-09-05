#!/bin/sh
# check-one-home.sh - reject duplicated long prose sentences outside history and imported sources.
#
# Run from the repository root. The POSIX and PowerShell forms must
# return equivalent results. A missing prerequisite is reported, not passed.
#
# Usage: scripts/common/check-one-home.sh [--json or -Json and documented options]
# Exit codes: 0 passed, 1 assertion failed, 2 could not run.
set -u

# ⛔ ONE SUBSTITUTION, NOT ONE PER LINE READ. An assignment prefix on a
# `while ... read` is re-evaluated on EVERY iteration, so `IFS="$(printf
# '\t')" read ...` forks once per line. Measured 2026-09-02: a command
# substitution costs 35 ms on this host, and check-docs.sh reads about 1100
# lines that way. docs/history/todo/tooling.md, TOOL-18.
TAB=$(printf '\t')

JSON=0
# ⚠ A CONSTANT, NOT A FLAG, for the reason check-markers.sh gives about its own
# ceiling: a threshold anybody can raise from a command line gets raised
# instead of met. Twelve words is long enough that two documents do not reach
# it by coincidence and short enough to catch a copied rule.
MINWORDS=12

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-one-home: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-one-home: git not found\n' >&2; exit 2; }
command -v awk >/dev/null 2>&1 || { printf 'check-one-home: awk not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-one-home: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-one-home: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⛔ NO QUOTED PATHSPEC. The extension filter is applied here, by grep, rather
# than handed to git, because a quoted pathspec crossing a shell that does not
# treat a quote as a quote matches nothing and the check then passes over an
# empty set. That is exactly how the first version of this reported a clean
# tree it had never opened.
FILES=$(
  {
    git ls-files 2>/dev/null
    git ls-files --others --exclude-standard 2>/dev/null
  } | sort -u | grep '\.md$' | grep -v '^docs/history/' \
    | grep -vE '^(references|vendor/[^/]+)/' || true
)
if [ -z "$FILES" ]; then
  printf 'check-one-home: no markdown files in scope\n' >&2
  exit 2
fi

TMP="${TMPDIR:-/tmp}/.checkonehome.$$"
mkdir -p "$TMP" || { printf 'check-one-home: cannot write to %s\n' "$TMP" >&2; exit 2; }
trap 'rm -rf "$TMP"' EXIT INT TERM

NFILES=0
: > "$TMP/pairs"

for f in $FILES; do
  [ -f "$f" ] || continue
  NFILES=$((NFILES + 1))
  # One sentence per line, normalised, prefixed with its file.
  LC_ALL=C awk -v F="$f" -v MIN="$MINWORDS" '
    /^[ \t]*```/ { fence = !fence; next }
    fence        { next }
    /^[ \t]*\|/  { next }        # a table row is not a sentence
    /^[ \t]*#/   { next }        # nor is a heading
    {
      line = $0
      while (match(line, /`[^`]*`/))
        line = substr(line, 1, RSTART - 1) " " substr(line, RSTART + RLENGTH)
      gsub(/\[/, " ", line); gsub(/\]\([^)]*\)/, " ", line)
      buf = buf " " line
    }
    END {
      n = split(buf, part, /[.:!?]+[ \t]+/)
      for (i = 1; i <= n; i++) {
        s = tolower(part[i])
        gsub(/[^a-z0-9 ]/, " ", s)
        gsub(/  +/, " ", s)
        sub(/^ /, "", s); sub(/ $/, "", s)
        if (s == "") continue
        if (split(s, w, " ") < MIN) continue
        printf "%s\t%s\n", s, F
      }
    }
  ' "$f" >> "$TMP/pairs" 2>/dev/null || true
done

# ⚠ THE SCOPE IS ASSERTED BEFORE THE VERDICT. See the header: a run over zero
# files is not a clean run.
if [ "$NFILES" -lt 2 ]; then
  printf 'check-one-home: only %s file(s) in scope; nothing to compare\n' "$NFILES" >&2
  exit 2
fi

# Group by sentence, keep the ones seen in more than one DISTINCT file, then
# drop any whose files are all routers.
# ⚠ ONE RECORD PER LINE, files joined by a space. An earlier version joined
# them with a NEWLINE, so a single duplicate occupied three lines and the count
# below, which counts lines, reported one planted duplicate as three. The
# verdict was right and every number beside it was wrong.
sort -u "$TMP/pairs" | awk -F'\t' '
  BEGIN {
    R["AGENTS.md"] = 1
  }
  { key = $1; files[key] = files[key] " " $2; count[key]++ }
  END {
    for (k in count) {
      if (count[k] < 2) continue
      n = split(files[k], fs, " ")
      allrouters = 1
      for (i = 1; i <= n; i++) if (fs[i] != "" && !(fs[i] in R)) allrouters = 0
      if (allrouters) continue
      printf "%s\t%s\n", k, substr(files[k], 2)
    }
  }
' > "$TMP/dups" 2>/dev/null || true

# ⚠ awk, NOT `grep -c`. `grep -c` on a file with no matches prints 0 AND exits
# 1, so `grep -c . f || printf 0` printed BOTH zeros and the result was the
# two-line string "0\n0", which every later numeric test refused as "integer
# expected". The check still exited 0, so it looked like it worked.
COUNT=$(awk 'END { print NR + 0 }' "$TMP/dups" 2>/dev/null)
[ -n "$COUNT" ] || COUNT=0

if [ "$JSON" = "1" ]; then
  printf '{"schema":"check-one-home/1","problems":%s,"files":%s,"min_words":%s}\n' \
    "$COUNT" "$NFILES" "$MINWORDS"
  [ "$COUNT" -gt 0 ] && exit 1
  exit 0
fi

if [ "$COUNT" -gt 0 ]; then
  printf 'one fact, one home: %s sentence(s) appear in more than one document:\n\n' "$COUNT"
  while IFS="$TAB" read -r s rest; do
    [ -n "$s" ] || continue
    printf '  "%s"\n' "$(printf '%s' "$s" | cut -c1-88)"
    for one in $rest; do
      printf '      %s\n' "$one"
    done
    printf '\n'
  done < "$TMP/dups"
  printf 'Keep the fact in the document that owns it and make the other a pointer.\n'
  printf 'docs/conventions/prose.md, "one fact, one home".\n'
  exit 1
fi

printf 'one fact one home: %s documents, no sentence of %s+ words in two of them\n' \
  "$NFILES" "$MINWORDS"
exit 0
