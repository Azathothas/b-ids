#!/bin/sh
# check-docs.sh - validate first-party Markdown links, cited paths, vocabulary, and reachability.
#
# Run from the repository root. The POSIX and PowerShell forms must
# return equivalent results. A missing prerequisite is reported, not passed.
#
# Usage: scripts/common/check-docs.sh [--json or -Json and documented options]
# Exit codes: 0 passed, 1 assertion failed, 2 could not run.
set -u

# ⛔ ONE SUBSTITUTION, NOT ONE PER LINE READ. An assignment prefix on a
# `while ... read` is re-evaluated on EVERY iteration, so `IFS="$(printf
# '\t')" read ...` forks once per line. Measured 2026-09-02: a command
# substitution costs 35 ms on this host, and check-docs.sh reads about 1100
# lines that way. docs/history/todo/tooling.md, TOOL-18.
TAB=$(printf '\t')

JSON=0
SCOPE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --path) shift; SCOPE="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-docs: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-docs: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-docs: not a git repository\n' >&2; exit 2; }
command -v awk >/dev/null 2>&1 || { printf 'check-docs: awk not found\n' >&2; exit 2; }
SELF=check-docs
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ EVERY git QUERY BELOW RUNS FROM THE REPOSITORY ROOT. `git ls-files` is
# relative to the process working directory, so without this a run from a
# subdirectory silently scopes itself to that subtree and reports clean over
# everything else. The scope of a guard must not depend on who called it.
cd "$REPO_ROOT" || { printf '%s: cannot enter %s\n' "$SELF" "$REPO_ROOT" >&2; exit 2; }

# ⛔ TRACKED **PLUS UNTRACKED-BUT-NOT-IGNORED**. `git ls-files` alone cannot see
# a file that has never been staged, which is exactly when a new file is most
# likely to carry a defect and exactly what the next `git add -A` will take.
# Ignored files stay out: they are ignored on purpose.

# -- ⛔ THE REFERENCE CORPUS IS EXEMPT, AND ONLY FROM THIS CHECK'S SUBJECT ----
#
# `references/` holds other projects' trees, at named commits, as the evidence
# behind docs/reference-sweeps/findings.md. docs/methodology/references.md is
# why it is tracked rather than deleted.
#
# ⛔ It is somebody else's writing, so this project's rules about how a document
# is written cannot apply to it. Their links point into subtrees this sweep
# trimmed, their pages are orphaned relative to this tree, and their templates
# hold placeholders on purpose. None of that is a defect here, and a check that
# fails on a correct tree gets switched off within a week.
#
# ⭐ EVERY CHECK EXEMPTS IT, AND EACH EXEMPTION WAS PAID FOR SEPARATELY. The
# prose checks, because it is somebody else's writing. check-control-bytes,
# because .gitattributes declares `references/** -text` so the corpus is stored
# byte-exact as evidence, and a finding there could only be fixed by editing the
# bytes a citation points at. check-no-secrets, after every hit it produced over
# the corpus was read once and recorded; its own header carries the counts.
# ⚠ A check whose findings cannot be acted on is a check that gets switched off,
# and an exemption taken without reading first is one nobody can defend.

# -- ⛔ THE VENDORED TREES ARE EXEMPT, AND THE MANIFEST BESIDE THEM IS NOT ---
#
# vendor/NAME/ holds third-party source this tree compiles and patches, at the
# commit vendor/upstream.json records. It is somebody else's writing, so this
# project's rules about how its own files are written do not apply to it, for
# the same reason the reference corpus is exempt above.
#
# ⚠ The exemption is vendor/NAME/ and never vendor/, so vendor/upstream.json,
# which this project wrote, is checked like any other file here. An exemption
# on the whole directory would have covered our own record by accident.
# docs/history/todo/vendor.md, VENDOR-01.

list_files() {
  {
    git ls-files -- "$@" 2>/dev/null
    git ls-files --others --exclude-standard -- "$@" 2>/dev/null
  } | sort -u |
    while IFS= read -r p; do [ -f "$p" ] && printf '%s\n' "$p"; done |
    grep -vE '^(references|vendor/[^/]+)/'
}


# ⛔ NO FILE IS EXEMPT FROM THE LINK CHECK, and the exemption that used to be
# here was removed rather than emptied. It covered a template directory whose
# links are written relative to where the file will live in a PROJECT rather
# than where it sits in the tree. This repository is not a template and has no
# such directory: the one fill-in form it keeps, docs/history/todo/ENTRY.md, is written from
# where it lives, so its links resolve here. ⚠ An exemption for a path that
# does not exist is dead configuration, and the next file to land under that
# path would have inherited it silently.

if [ -n "$SCOPE" ]; then
  FILES=$(list_files "$SCOPE" | grep '\.md$' || true)
else
  FILES=$(list_files '*.md' || true)
fi
[ -z "$FILES" ] && { printf 'check-docs: no markdown files in scope\n' >&2; exit 2; }

TMP="${TMPDIR:-/tmp}/.checkdocs.$$"
mkdir -p "$TMP" || { printf 'check-docs: cannot write to %s\n' "$TMP" >&2; exit 2; }
trap 'rm -rf "$TMP"' EXIT INT TERM

# ⛔ THE BANNED VOCABULARY, AND WHY IT IS FOURTEEN WORDS RATHER THAN EIGHTEEN.
# docs/conventions/prose.md bans words that assert quality instead of
# demonstrating it. Fourteen of them are ALWAYS that, so a match is always a
# defect and a check can hold them.
#
# ⚠ FOUR OF THE EIGHTEEN ARE DELIBERATELY NOT HERE: simply, just, obviously
# and "of course". They are banned as DISMISSALS, telling a reader who is stuck
# that what they cannot do is easy, and they are ordinary English in a contrast:
# "not just the names", "none is obviously right". Measured over this tree on
# 2026-08-31, before this check existed: 19 matches, every one of them
# legitimate, and 0 defects. ⛔ A guard with a nineteen-to-nothing false
# positive rate is a guard somebody switches off, so those four stay a reading
# and prose.md says which half owns them.
BANNED_WORDS="seamless blazing effortless robust powerful cutting-edge state-of-the-art world-class elegant revolutionary game-changing rock-solid bulletproof lightning-fast"

# ⭐ THE TOP-LEVEL DIRECTORIES THIS REPOSITORY OWNS, read from git rather than
# written down, so a new one is covered without anybody remembering to add it.
# It is what scopes the cited-path check to this tree.
ROOTS=$(git ls-files | awk -F/ 'NF > 1 { print $1 }' | sort -u | tr '
' ' ')

PROBLEMS=""
COUNT=0
NFILES=0
NLINKS=0
NSPANS=0
NBLOCKS=0

report() { PROBLEMS="$PROBLEMS  $1
"; COUNT=$((COUNT + 1)); }

# ⭐ EVERY LINK TARGET THAT EXISTS IS COLLECTED HERE AND ASKED ABOUT ONCE, after
# the loop. docs/history/todo/tooling.md, TOOL-18: this was a `git check-ignore` per link,
# 966 of them, and a subprocess costs 54.5 ms on the host that measured it.
: > "$TMP/linktargets"

for f in $FILES; do
  NFILES=$((NFILES + 1))
  # ⚠ Parameter expansion rather than `dirname`, which was a process per file.
  dir=${f%/*}
  [ "$dir" = "$f" ] && dir="."

  # -- one pass: strip fences and code spans, then emit every finding --------
  # ⚠ Stripping code spans is why `[int](2.65)` inside backticks is not
  # reported as a broken link. Markdown does not linkify a code span, and an
  # earlier ad-hoc version of this check reported exactly that as broken.
  awk -v BANNED="$BANNED_WORDS" -v ROOTS="$ROOTS" '
    BEGIN { FS = "\n" }
    /^[ \t]*```/ { fence = !fence; next }
    fence { next }
    {
      line = $0

      # -- ⭐ A CITED PATH IS CHECKED, NOT ONLY A LINK ------------------------
      #
      # A markdown link is resolved below and a path written in a code span was
      # not, which is how most of this tree names a file. Seven code spans named
      # a licence filler, its twin and a directory of texts, none of which
      # existed; every link resolved and this check was green throughout.
      # docs/history/todo/tooling.md TOOL-10.
      #
      # ⛔ NARROW, AND IT REFUSES TO GUESS. A span is a path only when it holds
      # a slash, ends in a known extension, has no whitespace, no angle bracket
      # and no glob character, carries no scheme, and has no ALL-CAPS segment,
      # which is the placeholder convention this tree uses. ⚠ An apostrophe
      # cannot appear anywhere in this awk program: it is one single-quoted
      # shell string, and one apostrophe ends it. A bare filename with no
      # directory is out of scope, and so is anything inside a fenced block,
      # which never reaches here. A guard that refuses legitimate writing is
      # worse than an honest one.
      probe = line
      while (match(probe, /`[^`]*`/)) {
        span = substr(probe, RSTART + 1, RLENGTH - 2)
        probe = substr(probe, RSTART + RLENGTH)
        if (span !~ /\//) continue
        if (span ~ /[ \t<>*?]/) continue
        if (span ~ /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\//) continue
        if (span !~ /\.(md|sh|ps1|psm1|mjs|cjs|js|ts|rs|toml|json|jsonc|yml|yaml|txt|py|go|lock|hex)$/) continue
        placeholder = 0
        n2 = split(span, seg, "/")
        for (j = 1; j <= n2; j++)
          if (seg[j] ~ /^[A-Z0-9_]+$/) placeholder = 1
        if (placeholder) continue

        # ⛔ AND IT MUST START AT ONE OF THIS REPOSITORY OWN TOP-LEVEL
        # DIRECTORIES. Measured on 2026-08-31: without this the check reported
        # 30 spans and every one was legitimate, because the sweep documents
        # cite paths INSIDE the reference trees as shorthand. A guard with a
        # thirty-to-nothing false positive rate is a guard somebody switches
        # off. ⚠ The list is read from git rather than written here, so a new
        # top-level directory is covered without anybody remembering to add it.
        head = seg[1]
        if (n2 < 2) continue
        if (index(" " ROOTS " ", " " head " ") == 0) continue
        print "SPAN\t" NR "\t" span
      }

      while (match(line, /`[^`]*`/))
        line = substr(line, 1, RSTART - 1) substr(line, RSTART + RLENGTH)


      rest = line
      while (match(rest, /\]\([^)\t ]+/)) {
        t = substr(rest, RSTART + 2, RLENGTH - 2)
        print "LINK\t" NR "\t" t
        rest = substr(rest, RSTART + RLENGTH)
      }

      low = tolower(line)
      n = split(BANNED, w, " ")
      for (i = 1; i <= n; i++)
        if (match(low, "(^|[^a-z-])" w[i] "([^a-z-]|$)"))
          print "VOCAB\t" NR "\t" w[i]
    }
  ' "$f" > "$TMP/find" 2>/dev/null || true

  while IFS="$TAB" read -r kind ln detail; do
    case "${kind:-}" in
      LINK)
        case "$detail" in http://*|https://*|mailto:*|'') continue ;; esac
        NLINKS=$((NLINKS + 1))
        target=${detail%%#*}
        [ -z "$target" ] && continue
        if [ -e "$dir/$target" ]; then
          # ⚠ Recorded rather than asked about here. The question is the one
          # below the loop, and asking it once is TOOL-18.
          # ⚠ TO STDOUT, and the whole loop is redirected once at its `done`.
          # An append per link opened and closed the file 966 times, and on the
          # host that measured it that cost 39 of the check's 70 seconds: more
          # than every subprocess in it put together. TOOL-18.
          printf '%s\t%s:%s\t%s\n' "$dir/$target" "$f" "$ln" "$detail"
        else
          report "$f:$ln broken link -> $detail"
        fi ;;
      SPAN)
        # ⚠ Resolved against the REPOSITORY ROOT and against the citing file's
        # own directory, and reported only when neither exists. Most of this
        # tree writes a root-relative path in prose and a directory-relative
        # one in a link, and refusing either would be refusing legitimate
        # writing.
        NSPANS=$((NSPANS + 1))
        if [ ! -e "$detail" ] && [ ! -e "$dir/$detail" ]; then
          report "$f:$ln cited path does not exist -> $detail"
        fi ;;
      VOCAB)
        report "$f:$ln banned vocabulary: $detail. docs/conventions/prose.md" ;;
    esac
  done < "$TMP/find" >> "$TMP/linktargets"

  # ⚠ THE CONTROL-BYTE RULE MOVED, IT WAS NOT DROPPED. It used to live here and
  # scanned markdown only, which left every .ts, .py, .rs, .sh and .yml in the
  # tree unchecked for the one defect that makes a file invisible to review.
  # It now lives in check-control-bytes.sh over EVERY text file. Two checks
  # enforcing one rule is two places for it to be wrong, so this one no longer
  # does it. ⛔ Run both: this one for documents, that one for the whole tree.

  # -- fenced shell blocks: extracted in one pass, then checked -------------
  # ⭐ ONE awk PER FILE INSTEAD OF THREE PROCESSES PER BLOCK. TOOL-18. The
  # carriage returns are stripped as each block is written and the placeholder
  # is matched in the same pass, so only `sh -n` still needs a process of its
  # own: it is the parse itself rather than a search.
  # ⚠ `rm -f` is gone and it used to cost a process per file. The loop below is
  # driven by what THIS awk reported, and awk truncates each block file the
  # first time it writes to it, so a stale block from a longer file is never
  # read.
  awk -v D="$TMP" '
    /^[ \t]*```(bash|sh)[ \t]*$/ { inb = 1; n++; start[n] = NR; ph[n] = 0; next }
    inb && /^[ \t]*```/          { inb = 0; next }
    inb {
      line = $0
      sub(/\r$/, "", line)
      if (line ~ /<[a-z][a-z0-9-]*>/) ph[n] = 1
      print line > (D "/blk." n)
    }
    END { for (i = 1; i <= n; i++) print i "\t" start[i] "\t" ph[i] }
  ' "$f" > "$TMP/blocks" 2>/dev/null || true

  while IFS="$TAB" read -r idx bstart hasph; do
    [ -z "${idx:-}" ] && continue
    blk="$TMP/blk.$idx"
    [ -f "$blk" ] || continue
    NBLOCKS=$((NBLOCKS + 1))
    sh -n "$blk" 2>/dev/null || report "$f:$bstart shell block does not parse"
    if [ "${hasph:-0}" = "1" ]; then
      report "$f:$bstart shell-unsafe placeholder. bash reads it as a redirect; use UPPER_SNAKE"
    fi
  done < "$TMP/blocks"
done

# -- the one question about link targets, asked once -------------------------
#
# ⛔ ON DISK IS NOT THE SAME AS IN THE REPOSITORY, and the difference is
# invisible until somebody else clones. A link to a file this tree does not
# commit resolves on the machine that wrote it and 404s everywhere else, which
# is the "green locally, red in CI" shape. Measured: a mined reference tree
# brought its OWN .gitignore, git honoured it, and 92 files of a corpus this
# repository states it keeps were on disk and in no commit. One of them was a
# primary evidence artefact cited twice.
#
# ⚠ `git check-ignore --stdin` exits 1 when nothing it was given is ignored,
# which is the clean answer rather than an error.
if [ -s "$TMP/linktargets" ]; then
  cut -f1 "$TMP/linktargets" | sort -u > "$TMP/lt.paths"
  git check-ignore --stdin < "$TMP/lt.paths" > "$TMP/lt.ignored" 2>/dev/null || true
  # ⚠ The inner scan runs only when something IS ignored, which is the failing
  # path. A clean tree pays one `git` and one `cut`.
  if [ -s "$TMP/lt.ignored" ]; then
    while IFS= read -r ignored; do
      [ -n "$ignored" ] || continue
      while IFS="$TAB" read -r lt_path lt_where lt_detail; do
        [ "$lt_path" = "$ignored" ] || continue
        report "$lt_where link target is on disk and NOT COMMITTED -> $lt_detail"
      done < "$TMP/linktargets"
    done < "$TMP/lt.ignored"
  fi
fi

# -- a page nothing links to -------------------------------------------------
# ⛔ AN UNLINKED PAGE IS NOT READ, SO IT IS NOT CORRECTED, and that is the state
# every stale document passes through on the way to being wrong.
#
# ⚠ This rule was written in docs/conventions/prose.md with nothing enforcing
# it, and within the hour a new prompt file was added that nothing referenced.
# A door sweep found it by hand. A rule that can be checked should be a check.
#
# Roots are exempt: a README is an entry point, and the files at the repository
# root are what a reader or a raw URL arrives at directly.
LINKED="$TMP/linked"
: > "$LINKED"
for f in $FILES; do
  d=$(dirname "$f")
  awk '
    /^[ \t]*```/ { fence = !fence; next }
    fence { next }
    {
      rest = $0
      while (match(rest, /\]\([^)\t ]+/)) {
        print substr(rest, RSTART + 2, RLENGTH - 2)
        rest = substr(rest, RSTART + RLENGTH)
      }
    }
  ' "$f" 2>/dev/null | while IFS= read -r tgt; do
      case "$tgt" in http://*|https://*|mailto:*|'') continue ;; esac
      t=${tgt%%#*}
      [ -z "$t" ] && continue
      # Normalise to a repo-relative path so a link from a subdirectory and one
      # from the root both name the same file.
      if [ "$d" = "." ]; then printf '%s\n' "$t"; else printf '%s/%s\n' "$d" "$t"; fi
    done >> "$LINKED"
done
# Collapse `a/../b` so ../ links resolve to the same string as a direct one.
sed -e ':a' -e 's![^/][^/]*/\.\./!!;ta' -e 's!^\./!!' "$LINKED" | sort -u > "$LINKED.n"

for f in $FILES; do
  case "$f" in
    */README.md|README.md) continue ;;
    */*) ;;
    *) continue ;;   # a root-level file is an entry point
  esac
  grep -qxF "$f" "$LINKED.n" || report "$f is linked from nowhere. An unlinked page is not read, so it is not corrected."
done

# -- the character rule moved, it was NOT dropped -------------------------
# ⛔ THE FIVE-CHARACTER ALLOWLIST AND THE EM-DASH RULE NOW LIVE IN
# check-markers.sh, over EVERY tracked text file rather than over markdown
# alone. Two checks enforcing one rule is two places for it to be wrong, and
# these two would have been wrong differently: this one strips fenced blocks
# and code spans before it looks and a whole-tree scan that did not would
# refuse the page that names the character it bans.
#
# ⚠ It is the same move the control-byte rule made out of this file, for the
# same reason, and the markdown-only scan is why 164 characters in 28 files
# went unchecked in this tree while this check reported it clean. ⛔ Run both:
# this one for documents, that one for the whole tree.

# -- report ------------------------------------------------------------------
if [ "$JSON" = "1" ]; then
  printf '{"schema":"check-docs/1","problems":%s,"files":%s,"links":%s,"cited_paths":%s,"shell_blocks":%s}\n' \
    "$COUNT" "$NFILES" "$NLINKS" "$NSPANS" "$NBLOCKS"
  [ "$COUNT" -gt 0 ] && exit 1
  exit 0
fi

if [ "$COUNT" -gt 0 ]; then
  printf 'documentation check failed, %s problem(s):\n\n%s\n' "$COUNT" "$PROBLEMS"
  exit 1
fi

printf 'docs ok: %s files, %s relative links, %s cited paths, %s shell blocks. Links, paths and prose clean.\n' \
  "$NFILES" "$NLINKS" "$NSPANS" "$NBLOCKS"
exit 0
