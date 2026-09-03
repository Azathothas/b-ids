#!/bin/sh
# check-trust-anchors.sh - does every profile carrying the trust-anchor
# extension have a published list with a capture date, and does the recommendation
# state all three options?
#
# ⛔ ONE EXTENSION CARRIES A SNAPSHOT OF THE BROWSER'S OWN ROOT STORE, so a client
# copying one build's list is advertising which build it copied. It changes on a
# different schedule from everything else a profile carries, which is why it is
# published beside the corpus. TODO/corpus.md, CORPUS-04.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. every profile that CARRIES the extension has a published list;
#   2. every published list names its capture instant and at least one
#      identifier, because a list with no date is a list nobody can place;
#   3. the recommendation states all THREE options, each with its cost, and
#      asserts no preference;
#   4. ⚠ AND IT REFUSES A VACUOUS PASS. A corpus in which no profile carries the
#      extension would satisfy rule 1 by having nothing to check, which is the
#      "acceptance command that cannot fail" row of
#      docs/conventions/forbidden-patterns.md. This exits 2 there and says so.
#
# ⚠ THE NAME OF THE EXTENSION IS INFERRED AND THIS DOES NOT SETTLE IT.
# docs/inherited-claims.md section 3 carries that split. What is checked here is
# the codepoint, the shape and the publication, all of which are measured.
#
# Usage:
#   sh scripts/common/check-trust-anchors.sh
#   sh scripts/common/check-trust-anchors.sh --json
#
# Exit codes: 0 every carrier is published, 1 one is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-trust-anchors: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-trust-anchors: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-trust-anchors: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-trust-anchors: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

# ⭐ THE CORPUS ROOT IS RESOLVED RATHER THAN ASSUMED. It is the working tree for
# as long as that holds a corpus, and a materialised copy of the data branch
# once it does not. corpus-root.sh is the one answer to the question and this
# check does not carry a second one. TODO/publish.md, PUB-11.
CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-trust-anchors: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
# ⛔ AND EXPORTED, because cargo is downstream of this decision. The b-ids
# crate's build script embeds the corpus at build time and reads exactly this
# variable, calling it the seam PUB-11 needs; a check that resolved a root and
# did not export it would build against one corpus and report on another.
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"
command -v cargo >/dev/null 2>&1 || { printf 'check-trust-anchors: cargo not found\n' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { printf 'check-trust-anchors: jq not found\n' >&2; exit 2; }

DOC="$REPO_ROOT/docs/trust-anchors.md"
[ -f "$DOC" ] || {
  printf 'check-trust-anchors: no recommendation at %s\n' "$DOC" >&2
  exit 2
}
[ -d "$CORPUS_ROOT/corpus" ] || {
  printf 'check-trust-anchors: there is no corpus, so no list can be published\n' >&2
  exit 2
}

OUT="$REPO_ROOT/.tmp/check-trust-anchors"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-trust-anchors: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo build -q -p b-ids-corpus || {
  printf 'check-trust-anchors: the corpus crate did not build\n' >&2
  exit 2
}
BIN="$REPO_ROOT/target/debug/b-ids-corpus"
[ -x "$BIN" ] || BIN="$BIN.exe"
[ -x "$BIN" ] || { printf 'check-trust-anchors: %s is not executable\n' "$BIN" >&2; exit 2; }

# ⛔ READ FROM THE PROCESS, UNPIPED.
"$BIN" anchors --root "$CORPUS_ROOT" --out "$OUT" > "$OUT.log" 2>&1
rc=$?
if [ "$rc" != 0 ]; then
  printf 'check-trust-anchors: publishing the lists exited %s\n' "$rc" >&2
  cat "$OUT.log" >&2
  exit 1
fi
STATUS=$(awk '/^corpus=anchors /{ line = $0 } END { print line }' "$OUT.log")
LISTS=$(printf '%s' "$STATUS" | awk -F'lists:' '{ split($2, a, / /); print a[1] }')
PROFILES=$(printf '%s' "$STATUS" | awk -F'profiles:' '{ split($2, a, / /); print a[1] }')

# 1. ⛔ HOW MANY PROFILES CARRY IT, counted from the corpus rather than from the
# publisher's own answer. A publisher that skipped a carrier would otherwise
# agree with itself.
CARRIERS=$(find "$REPO_ROOT/corpus" -name '*.json' -not -name 'index.json' -not -name 'latest.json' \
  -exec jq -r 'if ([.tls.extensions[].codepoint] | index(51764)) then "carrier" else empty end' {} \; \
  2>/dev/null | grep -c carrier)

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

# 4. ⚠ THE VACUOUS PASS, REFUSED FIRST. Everything below is satisfiable by an
# empty set, so a corpus with no carrier has verified nothing.
if [ "$CARRIERS" = "0" ]; then
  printf 'check-trust-anchors: no profile in this corpus carries codepoint 0xca34, so\n' >&2
  printf '  there is nothing to publish and nothing this check can verify. That is a\n' >&2
  printf '  fact about the builds captured, not a pass. TODO/corpus.md, CORPUS-04.\n' >&2
  exit 2
fi

[ "$LISTS" = "$CARRIERS" ] || note "$CARRIERS profile(s) carry the extension and $LISTS list(s) were published"

# 2. every published list names its date and at least one identifier
for f in "$OUT"/*.json; do
  [ -f "$f" ] || continue
  name=$(basename "$f")
  at=$(jq -r '.captured_at // ""' "$f" 2>/dev/null | tr -d '\r')
  n=$(jq -r '.identifiers | length' "$f" 2>/dev/null | tr -d '\r')
  [ -n "$at" ] || note "$name: no capture instant, so nobody can place this list in time"
  [ "${n:-0}" -gt 0 ] 2>/dev/null || note "$name: no identifiers"
done

# 3. ⛔ ALL THREE OPTIONS, each with its cost. A document that named two would be
# a recommendation wearing a trade's clothes.
for phrase in 'Omit the extension' 'Carry a captured list' 'Send it empty'; do
  grep -qF "$phrase" "$DOC" || note "the recommendation does not state the option '$phrase'"
done
grep -qF 'asserts no preference' "$DOC" || note "the recommendation does not say that it asserts no preference"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-trust-anchors/1","carriers":%s,"lists":%s,"profiles":%s,"problems":%s}\n' \
    "${CARRIERS:-0}" "${LISTS:-0}" "${PROFILES:-0}" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'trust anchors ok: %s of %s profile(s) carry codepoint 0xca34, and every one has a\n' \
    "$CARRIERS" "$PROFILES"
  printf '  published list with its capture instant. The recommendation states all three\n'
  printf '  options and asserts no preference.\n'
  exit 0
fi

printf 'trust-anchor check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'The list is a snapshot of a root store and it changes per build.\n' >&2
printf 'docs/trust-anchors.md is the recommendation. TODO/corpus.md, CORPUS-04.\n' >&2
exit 1
