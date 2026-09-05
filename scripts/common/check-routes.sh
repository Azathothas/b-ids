#!/bin/sh
# check-routes.sh - does any published route file that carries exactly one value
# end with a newline?
#
# The defect this exists to catch is a consumer having to strip something. A
# route a program reads with nothing but `curl` should hand back the value and
# nothing else; a trailing newline means every caller writes a strip, and the
# ones that forget compare a value against a value-plus-newline and get a
# mismatch they cannot see.
#
# ⭐ MEASURED ON THE REFERENCE THE REQUIREMENT CAME FROM, not invented here.
# `od -c` over two single-value files published by pkgforge-security/Wordlists
# shows a trailing newline on each, so the model this project is copying
# exhibits the defect the requirement forbids.
# docs/reference-sweeps/usable.md section 9.
#
# -- ⭐ WHAT COUNTS AS A SINGLE-VALUE FILE, AND WHY IT IS BY EXTENSION --------
#
# A file carries one value because this project SAYS its type does. `.hex` is
# defined by this tree as one raw capture on one line and nothing else, which is
# the same definition scripts/common/check-no-secrets uses to exempt one from
# the credential rule. ⛔ Extending the list is how PUB-03 adds a route type; a
# check that guessed from content would call a one-line JSON file single-valued
# and refuse a newline JSON needs.
#
# ⛔ IT REPORTS, IT DOES NOT STRIP. The generator is what gets fixed. A check
# that repaired its subject would be a check nobody can use to find out whether
# something is wrong.
#
# -- ⭐ IT GENERATES THE ROUTE TREE AND THEN READS THE CORPUS ITSELF ----------
#
# PUB-03's tree is generated into .tmp and checked there, because nothing in
# this repository publishes it yet: PUB-02 is the surface that will. Three
# questions are asked of it, and the second is the one a generator cannot answer
# about itself:
#
#   1. no single-value file ends with a line ending;
#   2. EVERY ROUTE'S VALUE IS THE ONE THE CORPUS HOLDS, read back out of the
#      profile the manifest names with jq. A check that compared the generator
#      with itself would pass over a generator reading the wrong field;
#   3. every `latest` route names a STABLE profile and carries the same value as
#      that profile's own version route. CORPUS-03 is the rule.
#
# ⚠ A SINGLE-VALUE FILE IS `.txt` AND A LIST IS `.list.txt`, so the last dot is
# ambiguous: it answers `txt` for both. The classifier below reads the whole
# suffix, and `index.txt` is a listing rather than a route.
#
# ⚠ WHERE THIS RULE IS NOT ENFORCED: inside a profile's own sidecar comparison.
# `b-ids-corpus verify` asserts a sidecar holds what its profile says
# raw.client_hello_hex is, which is a different question about the same file,
# and it deliberately does not also assert the newline. One rule, one enforcer.
#
# Usage:
#   sh scripts/common/check-routes.sh
#   sh scripts/common/check-routes.sh --json
#   sh scripts/common/check-routes.sh --fixtures DIR
#   sh scripts/common/check-routes.sh --assert-latest-is-stable
#
# Exit codes: 0 clean, 1 a route file ends with a newline, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURES=""
LATEST=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --fixtures) shift; FIXTURES="${1:-}" ;;
    --assert-latest-is-stable) LATEST=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-routes: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-routes: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-routes: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ Every path below is relative to the repository root, so the scope of the
# check does not depend on who called it.
cd "$REPO_ROOT" || { printf 'check-routes: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. docs/history/todo/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-routes: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

# ⛔ THE PUBLISHED ROUTE TREES, named rather than "everything". A check over the
# whole tree would refuse the committed `.hex` fixtures the harness tests read,
# which are inputs rather than routes and are not served to anybody.
# ⚠ PUB-03's generated tree joins this list below, once it has been generated,
# and the twin carries the identical list.
ROUTE_DIRS="raw"

# ⛔ The extensions this project defines as carrying one value.
SINGLE_VALUE="hex"

GENERATED="$REPO_ROOT/.tmp/check-routes/routes"
MANIFEST="$GENERATED/routes.json"
GENERATE=1

if [ -n "$FIXTURES" ]; then
  [ -d "$FIXTURES" ] || { printf 'check-routes: no directory at %s\n' "$FIXTURES" >&2; exit 2; }
  ROUTE_DIRS="$FIXTURES"
  # ⚠ A fixture run checks the FIXTURE and nothing else. Generating beside it
  # would let a fixture written to prove a refusal pass on the tree's own
  # routes.
  GENERATE=0
fi

if [ "$GENERATE" = 1 ]; then
  command -v cargo >/dev/null 2>&1 || {
    printf 'check-routes: cargo not found, so the route tree could not be generated\n' >&2
    exit 2
  }
  rm -rf "$GENERATED"
  mkdir -p "$REPO_ROOT/.tmp/check-routes"
  cargo build -q -p b-ids-corpus || {
    printf 'check-routes: the corpus crate did not build\n' >&2
    exit 2
  }
  TARGET_DIR=${CARGO_TARGET_DIR:-"$REPO_ROOT/target"}
  BIN="$TARGET_DIR/debug/b-ids-corpus"
  [ -x "$BIN" ] || BIN="$BIN.exe"
  [ -x "$BIN" ] || { printf 'check-routes: %s is not executable\n' "$BIN" >&2; exit 2; }
  # ⛔ READ FROM THE PROCESS, UNPIPED.
  "$BIN" routes --root "$CORPUS_ROOT" --out "$GENERATED" \
    > "$REPO_ROOT/.tmp/check-routes/generate.log" 2>&1
  rc_gen=$?
  if [ "$rc_gen" != 0 ]; then
    printf 'check-routes: the route generator exited %s\n' "$rc_gen" >&2
    cat "$REPO_ROOT/.tmp/check-routes/generate.log" >&2
    exit 1
  fi
  ROUTE_DIRS="$ROUTE_DIRS .tmp/check-routes/routes"
fi

PRESENT=0
for dir in $ROUTE_DIRS; do
  [ -d "$dir" ] && PRESENT=1
done
if [ "$PRESENT" = 0 ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-routes/2","files":0,"verified":0,"problems":0,"routes":false}\n'
  else
    printf 'check-routes: no published route tree exists yet, so nothing was checked.\n' >&2
  fi
  # ⚠ 2, not 0. A tree with no routes has neither broken this rule nor
  # satisfied it, which is check-changelog's rule for the same reason.
  exit 2
fi

# ⚠ Tracked plus untracked-not-ignored, because a route file that has never been
# staged is exactly the one a generator has just written wrongly.
#
# ⛔ A FIXTURE DIRECTORY IS WALKED WITH `find`, NOT WITH GIT, and this was a
# defect rather than a design. `git ls-files` refuses a path outside the
# repository with a fatal on stderr and an empty list on stdout, so both halves
# of this check reported "ok, 0 files" over the fixture written to prove they
# could refuse. That is the "step that exits 0 having done nothing it was asked
# to do" row in docs/conventions/forbidden-patterns.md, in the check whose whole
# job is refusing.
list_route_files() {
  if [ -n "$FIXTURES" ]; then
    find "$FIXTURES" -type f 2>/dev/null | LC_ALL=C sort
    return
  fi
  for dir in $ROUTE_DIRS; do
    [ -d "$dir" ] || continue
    case "$dir" in
      # ⛔ THE GENERATED TREE IS UNDER .tmp, WHICH IS IGNORED, so
      # `git ls-files --others --exclude-standard` answers with NOTHING and the
      # walk reports a clean tree it never opened. That is the defect the
      # comment above describes, arriving from the other direction, and it is
      # the "step that exits 0 having done nothing" row of
      # docs/conventions/forbidden-patterns.md.
      .tmp/*) find "$dir" -type f 2>/dev/null ;;
      *) { git ls-files -- "$dir"; git ls-files --others --exclude-standard -- "$dir"; } ;;
    esac
  done | LC_ALL=C sort -u
}

PROBLEMS=""
COUNT=0
FILES=0
for file in $(list_route_files); do
  # ⚠ THE WHOLE SUFFIX, not the last dot. A list file and a single-value file
  # both end in `txt`, and a classifier reading only the last dot would call a
  # list single-valued and refuse the newline a list needs.
  base="${file##*/}"
  extension="${file##*.}"
  single=0
  case " $SINGLE_VALUE " in
    *" $extension "*) single=1 ;;
  esac
  case "$base" in
    index.txt) single=0 ;;
    *.list.txt) single=0 ;;
    *.txt) single=1 ;;
  esac
  [ "$single" = 1 ] || continue
  [ -f "$file" ] || continue
  FILES=$((FILES + 1))
  # ⛔ The LAST BYTE, read from the file rather than from a line count. A file
  # of one line and a file of one line plus a newline both report one line to
  # anything that counts lines, which is how this defect survives review.
  last=$(tail -c 1 "$file" | od -An -tx1 | tr -d ' \n')
  if [ "$last" = "0a" ] || [ "$last" = "0d" ]; then
    PROBLEMS="$PROBLEMS  $file: ends with a line ending, and it carries exactly one value
"
    COUNT=$((COUNT + 1))
  fi
done

# -- ⭐ every route's value, read back out of the corpus ----------------------
#
# ⛔ THIS IS THE LEG A GENERATOR CANNOT RUN ON ITSELF. The manifest names the
# profile and the property behind every route, and jq goes and reads the value
# out of the profile. Comparing the file with the manifest alone would only ask
# whether the generator agrees with itself, which it always does.
#
# ⚠ THE COMPARISON STRIPS TRAILING NEWLINES ON BOTH SIDES, deliberately. `jq -r`
# adds one and a single-value file has none; the newline rule is the loop above
# and this leg is about the VALUE. Two rules, two places.
VERIFIED=0
if [ "$GENERATE" = 1 ]; then
  if ! command -v jq >/dev/null 2>&1; then
    printf 'check-routes: jq not found, so no route was read back against the corpus\n' >&2
    exit 2
  fi
  [ -s "$MANIFEST" ] || {
    printf 'check-routes: the generator wrote no manifest at %s\n' "$MANIFEST" >&2
    exit 2
  }
  # ⚠ One line per route, tab separated, so the loop below forks nothing per
  # iteration. TOOL-18 measured that a command substitution in a `while read`
  # assignment prefix is re-evaluated on every line.
  # ⛔ jq ON WINDOWS WRITES CRLF, measured here and recorded in docs/history/todo/corpus.md,
  # CORPUS-02, when it first bit. The carriage return lands on the LAST field of
  # every line AND on the end of every value, so a comparison against a file
  # this project wrote with LF fails on every route while both sides are
  # correct. Every jq read below is stripped.
  jq -r '.routes[] | [.path, .profile, .property, (.variant // "-")] | @tsv' \
    "$MANIFEST" | tr -d '\r' > "$REPO_ROOT/.tmp/check-routes/manifest.tsv"
  TAB=$(printf '\t')
  while IFS="$TAB" read -r route profile property variant; do
    [ -n "$route" ] || continue
    VERIFIED=$((VERIFIED + 1))
    # ⛔ ONE jq PROGRAM PER PROPERTY, and an unknown property is a refusal
    # rather than a skip. A property added to the generator with no reader here
    # would otherwise be published unchecked.
    case "$property" in
      user-agent|sec-ch-ua|accept-language)
        want=$(jq -r --arg v "$variant" --arg n "$property" \
          '.http.variants[] | select(.variant == $v) | .headers[] | select(.name == $n) | .value' \
          "$profile" 2>/dev/null | tr -d '\r')
        ;;
      header-order)
        want=$(jq -r --arg v "$variant" \
          '.http.variants[] | select(.variant == $v) | [.headers[].name] | join("\n")' \
          "$profile" 2>/dev/null | tr -d '\r')
        ;;
      alpn)
        want=$(jq -r '.tls.alpn | join("\n")' "$profile" 2>/dev/null | tr -d '\r')
        ;;
      client-hello-hex)
        want=$(jq -r '.raw.client_hello_hex' "$profile" 2>/dev/null | tr -d '\r')
        ;;
      *)
        PROBLEMS="$PROBLEMS  $route: the property $property has no reader in this check
"
        COUNT=$((COUNT + 1))
        continue
        ;;
    esac
    got=$(cat "$GENERATED/$route" 2>/dev/null)
    if [ "$want" != "$got" ]; then
      PROBLEMS="$PROBLEMS  $route: the file is not what $profile holds for $property
"
      COUNT=$((COUNT + 1))
    fi
    case "$route" in
      */latest/*)
        # ⛔ A CONSUMER FOLLOWING `latest` MUST NEVER BE HANDED A PRE-RELEASE
        # BUILD, and the route names the profile it came from, so this asks the
        # profile rather than the path. docs/history/todo/corpus.md, CORPUS-03.
        channel=$(jq -r '.browser.channel' "$profile" 2>/dev/null | tr -d '\r')
        if [ "$channel" != "stable" ]; then
          PROBLEMS="$PROBLEMS  $route: latest names a $channel profile
"
          COUNT=$((COUNT + 1))
        fi
        ;;
    esac
  done < "$REPO_ROOT/.tmp/check-routes/manifest.tsv"
  if [ "$VERIFIED" = 0 ]; then
    printf 'check-routes: the manifest named no route, so nothing was read back\n' >&2
    exit 2
  fi
fi

# -- the latest pointer, on request -------------------------------------------
#
# ⛔ DELEGATED, never re-implemented. `b-ids-corpus latest --assert-stable`
# reads the pointer file on disk and the profiles it names, and its LAST line is
# a fixed `corpus=latest problems:N`. A second reader of that file in shell
# would be a second answer to what `latest` may point at.
#
# ⭐ The rule is enforced by CONSTRUCTION as well: the pointer file's `latest`
# map is built from stable profiles alone, so the derivation cannot produce a
# bad entry and `check-corpus` refuses a written file that disagrees with the
# derivation. This flag is what catches a hand-edited one.
LATEST_PROBLEMS=0
LATEST_RAN=0
LATEST_OUT=""
if [ "$LATEST" = 1 ]; then
  if [ -n "$FIXTURES" ]; then
    printf 'check-routes: --assert-latest-is-stable reads the corpus, so it cannot be\n' >&2
    printf 'combined with --fixtures.\n' >&2
    exit 2
  fi
  if command -v cargo >/dev/null 2>&1; then
    LATEST_OUT=$(cargo run -q -p b-ids-corpus -- latest --assert-stable --root "$CORPUS_ROOT" 2>&1)
    LATEST_RC=$?
    LATEST_LINE=$(printf '%s\n' "$LATEST_OUT" | awk '/^corpus=latest /{ line = $0 } END { print line }')
    case "$LATEST_RC" in
      0 | 1)
        if [ -n "$LATEST_LINE" ]; then
          LATEST_RAN=1
          LATEST_PROBLEMS=$(printf '%s' "$LATEST_LINE" | sed 's/.*problems:\([0-9]*\).*/\1/')
        fi
        ;;
      *) LATEST_RAN=0 ;;
    esac
  fi
  if [ "$LATEST_RAN" = 0 ]; then
    printf 'check-routes: the latest assertion did NOT run: cargo is absent, the\n' >&2
    printf 'workspace did not build, or there is no corpus.\n' >&2
    [ -n "$LATEST_OUT" ] && printf '%s\n' "$LATEST_OUT" >&2
    exit 2
  fi
  if [ "$LATEST_PROBLEMS" -gt 0 ]; then
    printf 'route check failed: %s latest pointer(s) do not name a stable profile.\n\n' "$LATEST_PROBLEMS"
    printf '%s\n' "$LATEST_OUT"
    exit 1
  fi
fi

# ⛔ A ROUTE TREE THAT YIELDED NO SINGLE-VALUE FILE HAS VERIFIED NOTHING, and
# reporting that as clean is how this check would quietly stop applying the day
# a route type is renamed. It is exit 2, "could not run", for the same reason an
# absent tree is.
if [ "$FILES" = 0 ]; then
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-routes/2","files":0,"verified":0,"problems":0,"routes":false}\n'
  else
    printf 'check-routes: the route tree holds no single-value file, so nothing\n' >&2
    printf 'was checked. The extensions this project treats as single-valued are\n' >&2
    printf 'in this script, beside the reason.\n' >&2
  fi
  exit 2
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-routes/2","files":%s,"verified":%s,"problems":%s,"routes":true}\n' "$FILES" "$VERIFIED" "$COUNT"
  [ "$COUNT" -gt 0 ] && exit 1
  exit 0
fi

if [ "$COUNT" -gt 0 ]; then
  printf 'route check failed, %s file(s):\n\n' "$COUNT"
  printf '%s\n' "$PROBLEMS"
  printf 'A consumer of a single-value route should never have to strip anything.\n'
  printf 'Fix the generator that wrote it, not the file.\n'
  exit 1
fi

printf 'routes ok: %s single-value file(s), none ends with a line ending,
' "$FILES"
printf '  and %s generated route(s) each carry the value the corpus holds' "$VERIFIED"
if [ "$LATEST" = 1 ]; then
  printf ', and every latest pointer names a stable profile'
fi
printf '\n'
exit 0
