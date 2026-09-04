#!/bin/sh
# check-validate.sh - the two assertions a push has to settle about the
# published corpus, with no network and no browser.
#
# The defect this exists to catch is a corpus that is STRUCTURALLY intact and
# INCOHERENT. check-corpus asks whether every profile sits at the route its keys
# derive, publishes the bytes it claims and was never edited after publication;
# every one of those can be true of a profile whose User-Agent says 151 and
# whose brand list says 152. Nothing in this tree ran the coherence checks over
# what is published until this did: `b-ids-validator` takes the paths a caller
# names, so it answered about whatever somebody remembered to list.
#
# -- ⭐ TWO LEGS, AND NEITHER IS check-corpus's ---------------------------------
#
#   1. COHERENCE, over what is published. Delegated to `b-ids-corpus validate`,
#      which is the one place that knows both the layout and the checks. ⛔ A
#      second enumeration of the corpus in shell would be a second answer to
#      which profiles are published.
#
#   2. DETERMINISM of the derived files. The generator is run TWICE over a
#      throwaway copy of the corpus and the two outputs are compared byte for
#      byte. ⚠ `b-ids-corpus verify` cannot see this class: it compares the
#      committed index against one derivation, so a generator that answered
#      differently on alternate runs would fail verify INTERMITTENTLY and be
#      read as a flake. Releases are reproducible or they are not, and this is
#      the assertion that says which.
#
# ⚠ THE COPY IS OUTSIDE THE REPOSITORY, and the generator writes only into it.
# A check that repaired its subject would be a check nobody can use to find out
# whether something is wrong; a check that wrote into the tree it measures would
# be worse.
#
# -- ⛔ NO NETWORK AND NO BROWSER, WHICH IS THE POINT OF CI-01 ------------------
#
# Both legs read files and run one already-built binary. Nothing here resolves a
# browser, opens a socket or asks an upstream what version is current: that is
# CI-02's job on a schedule, and a push that failed because a browser shipped
# would be a push failing for something no commit did. TODO/ci.md, CI-01.
#
# ⚠ WHAT IT DOES NOT ASSERT YET, said rather than implied: the generated
# formats' round trip. There is one generator in this tree and it writes the
# index and the pointer file; SCHEMA-08 is what adds the rest, and it adds them
# to leg 2 in the same change.
#
# Usage:
#   sh scripts/common/check-validate.sh
#   sh scripts/common/check-validate.sh --json
#
# Exit codes: 0 clean, 1 a profile is incoherent or a generator is not
#             deterministic, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-validate: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-validate: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-validate: not a git repository\n' >&2; exit 2; }
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ Every path below is relative to the repository root, so the scope of the
# check does not depend on who called it.
cd "$REPO_ROOT" || { printf 'check-validate: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-validate: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

CORPUS_DIR="corpus"
RAW_DIR="raw"

report_absent() {
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"check-validate/1","corpus":false,"profiles":0,"findings":0,"notcheckable":0,"deterministic":true,"problems":0}\n'
  else
    printf 'check-validate: %s\n' "$1" >&2
  fi
  exit 2
}

[ -d "$CORPUS_ROOT/$CORPUS_DIR" ] || report_absent "there is no $CORPUS_DIR/ directory, so nothing was validated"
command -v cargo >/dev/null 2>&1 || report_absent "cargo is not on this host, so no profile was validated"

# -- leg one: is every published profile coherent ----------------------------
#
# ⛔ THE NUMBERS COME FROM THE FIXED STATUS LINE, never from the prose above it.
# `b-ids-corpus validate` prints `corpus=validate profiles:N findings:N
# notcheckable:N` as its last line and its usage says that is the contract.
# check-corpus carries the identical discipline for the identical reason.
VALIDATE_OUT=$(cargo run -q -p b-ids-corpus -- validate --root "$CORPUS_ROOT" 2>&1)
VALIDATE_RC=$?
STATUS_LINE=$(printf '%s\n' "$VALIDATE_OUT" | awk '/^corpus=validate /{ line = $0 } END { print line }')
case "$VALIDATE_RC" in
  0 | 1) ;;
  # ⚠ 2 is the command's own "there is no corpus" or "it holds no profile". The
  # directory test above ruled out the first, so reaching it means the corpus
  # directory is empty, which has validated nothing.
  *) report_absent "the coherence leg did not run. cargo is absent, the workspace did not build, or the corpus holds no profile" ;;
esac
[ -n "$STATUS_LINE" ] || report_absent "b-ids-corpus validate printed no status line, so nothing could be read from it"

PROFILES=$(printf '%s' "$STATUS_LINE" | sed 's/.*profiles:\([0-9]*\).*/\1/')
FINDINGS=$(printf '%s' "$STATUS_LINE" | sed 's/.*findings:\([0-9]*\).*/\1/')
NOTCHECKABLE=$(printf '%s' "$STATUS_LINE" | sed 's/.*notcheckable:\([0-9]*\).*/\1/')

# -- leg two: does the generator answer the same way twice -------------------
#
# ⛔ A THROWAWAY COPY, never the tree. The generator writes, so it is pointed at
# a directory this check made and removes.
# ⚠ A LITERAL NEWLINE, because a command substitution strips trailing ones
# and an accumulator joined without one ran two findings together on a
# single line. Found by planting a non-deterministic generator and reading
# the message it produced.
NL="
"
DETERMINISTIC=1
DETAIL=""
SCRATCH="${TMPDIR:-/tmp}/b-ids-check-validate.$$"
rm -rf "$SCRATCH"
if mkdir -p "$SCRATCH/root" "$SCRATCH/first" 2>/dev/null; then
  cp -R "$CORPUS_ROOT/$CORPUS_DIR" "$SCRATCH/root/" 2>/dev/null
  # ⛔ THE RAW BYTES COME FROM THE RESOLVED ROOT, NOT THE WORKING TREE. This
  # line read `$RAW_DIR` relative to the repository, so with the corpus moved
  # out it copied a corpus from the data branch and no raw bytes at all. The
  # generator reads the capture beside each profile, so its first run then
  # failed with `raw/v1/.../151.0.7922.173.hello.hex: The system cannot find
  # the path specified`, and this check reported that as a NON-DETERMINISTIC
  # GENERATOR. ⚠ The wrong verdict is the point: the leg was not measuring what
  # its message claimed. TODO/publish.md, PUB-11.
  [ -d "$CORPUS_ROOT/$RAW_DIR" ] && cp -R "$CORPUS_ROOT/$RAW_DIR" "$SCRATCH/root/" 2>/dev/null

  if cargo run -q -p b-ids-corpus -- index --write --root "$SCRATCH/root" >/dev/null 2>&1; then
    # ⚠ Both derived files, not the index alone. The pointer file is derived by
    # the same writer from the same tree and is what a consumer follows.
    for derived in index.json latest.json; do
      if [ -f "$SCRATCH/root/$CORPUS_DIR/v1/$derived" ]; then
        cp "$SCRATCH/root/$CORPUS_DIR/v1/$derived" "$SCRATCH/first/$derived"
      fi
    done
    if cargo run -q -p b-ids-corpus -- index --write --root "$SCRATCH/root" >/dev/null 2>&1; then
      for derived in index.json latest.json; do
        first="$SCRATCH/first/$derived"
        second="$SCRATCH/root/$CORPUS_DIR/v1/$derived"
        if [ ! -f "$first" ] || [ ! -f "$second" ]; then
          DETERMINISTIC=0
          DETAIL="$DETAIL  $derived: one of the two runs did not write it$NL"
        elif ! cmp -s "$first" "$second"; then
          DETERMINISTIC=0
          DETAIL="$DETAIL  $derived: two runs of the generator over one corpus wrote different bytes$NL"
        fi
      done
    else
      DETERMINISTIC=0
      DETAIL="  the generator's second run failed$NL"
    fi
  else
    DETERMINISTIC=0
    DETAIL="  the generator's first run failed$NL"
  fi
  rm -rf "$SCRATCH"
else
  rm -rf "$SCRATCH"
  report_absent "could not create a scratch directory, so the determinism leg did not run"
fi

# -- report ------------------------------------------------------------------
PROBLEMS=$FINDINGS
[ "$DETERMINISTIC" = 0 ] && PROBLEMS=$((PROBLEMS + 1))

if [ "$JSON" = 1 ]; then
  det=true
  [ "$DETERMINISTIC" = 0 ] && det=false
  printf '{"schema":"check-validate/1","corpus":true,"profiles":%s,"findings":%s,"notcheckable":%s,"deterministic":%s,"problems":%s}\n' \
    "$PROFILES" "$FINDINGS" "$NOTCHECKABLE" "$det" "$PROBLEMS"
  [ "$PROBLEMS" -gt 0 ] && exit 1
  exit 0
fi

if [ "$FINDINGS" -gt 0 ]; then
  printf 'validate check failed: %s finding(s) over %s published profile(s).\n\n' "$FINDINGS" "$PROFILES"
  printf '%s\n' "$VALIDATE_OUT"
fi
if [ "$DETERMINISTIC" = 0 ]; then
  printf 'validate check failed: the generator is not deterministic.\n\n'
  printf '%s' "$DETAIL"
  printf 'A release nobody can reproduce is a release whose every run looks like a\n'
  printf 'change. Fix the generator, never this check.\n'
fi
[ "$PROBLEMS" -gt 0 ] && exit 1

printf 'validate ok: %s profile(s) coherent, the generator answers the same way\n' "$PROFILES"
printf 'twice, and %s check(s) reported they had nothing to read.\n' "$NOTCHECKABLE"
exit 0
