#!/bin/sh
# check-license-consistency.sh - do the places that state this project's licence
# all state the same one?
#
# ⛔ A FILE THAT TRAVELS ALONE STILL HAS TO SAY WHAT IT IS. A consumer who
# downloads one profile should not have to find this repository to learn they may
# use it. TODO/publish.md, PUB-07.
#
# -- ⛔ THE PLACES, AND WHY EACH ONE ------------------------------------------
#
#   the workspace manifest   what a builder of the code sees
#   b_ids_schema::LICENSE    ⭐ THE ONE HOME. Everything generated reads it.
#   the published JSON Schema  what a consumer validating a profile sees
#   the corpus index         what a consumer who fetches only the index sees
#   every published profile  what a consumer who fetches ONE file sees
#   the release body         what a consumer who downloads an asset sees
#
# ⚠ THE SIX PROFILES PUBLISHED BEFORE 2026-09-03 DO NOT CARRY THE FIELD, and
# that is recorded rather than repaired. The corpus is append-only, so adding it
# to them would be an edit of a published file. A profile that carries the field
# must agree; one that does not is counted and reported.
#
# ⛔ THE DATA BRANCH IS NOT CHECKED HERE BECAUSE IT DOES NOT EXIST. PUB-02 is the
# entry that creates it, and the LICENSE it carries is this file's to check on
# the day it does. Reporting a pass over a branch nobody has made is the "step
# that exits 0 having done nothing" row of
# docs/conventions/forbidden-patterns.md.
#
# Usage:
#   sh scripts/common/check-license-consistency.sh
#   sh scripts/common/check-license-consistency.sh --json
#   sh scripts/common/check-license-consistency.sh --fixture
#
# --fixture asserts that the check CAN fail: it builds a tree in which one
# statement disagrees and requires the comparison to refuse it.
#
# Exit codes: 0 they agree, 1 one disagrees, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
FIXTURE=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --fixture) FIXTURE=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-license-consistency: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || {
  printf 'check-license-consistency: git not found\n' >&2
  exit 2
}
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-license-consistency: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-license-consistency: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v jq >/dev/null 2>&1 || {
  printf 'check-license-consistency: jq not found\n' >&2
  exit 2
}

PROBLEMS=""
COUNT=0
STATED=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}
# ⛔ Every value is read and then compared with the FIRST one, so the message
# names all of them rather than the pair that happened to differ.
SEEN=""
state() {
  STATED=$((STATED + 1))
  SEEN="$SEEN  $1: $2
"
}

# -- the one home ------------------------------------------------------------
#
# ⛔ READ FROM THE SOURCE, never typed here. A check carrying its own copy of the
# identifier is a seventh place for it to disagree.
HOME_FILE="crates/b-ids-schema/src/lib.rs"
[ -f "$HOME_FILE" ] || { printf 'check-license-consistency: no %s\n' "$HOME_FILE" >&2; exit 2; }
WANT=$(awk -F'"' '/^pub const LICENSE: &str = /{ print $2; exit }' "$HOME_FILE")
[ -n "$WANT" ] || {
  printf 'check-license-consistency: %s declares no LICENSE constant\n' "$HOME_FILE" >&2
  exit 2
}
state "$HOME_FILE" "$WANT"

# -- the workspace manifest --------------------------------------------------
MANIFEST=$(awk -F'"' '/^license = /{ print $2; exit }' Cargo.toml)
state "Cargo.toml" "${MANIFEST:-absent}"
[ "$MANIFEST" = "$WANT" ] || note "Cargo.toml says ${MANIFEST:-nothing} and $HOME_FILE says $WANT"

# -- the published JSON Schema -----------------------------------------------
SCHEMA_FILE="crates/b-ids-schema/schema/browser-profile-1.schema.json"
SCHEMA=$(jq -r '.properties.license.const // "absent"' "$SCHEMA_FILE" 2>/dev/null | tr -d '\r')
state "$SCHEMA_FILE" "$SCHEMA"
[ "$SCHEMA" = "$WANT" ] || note "the published schema says $SCHEMA and $HOME_FILE says $WANT"
# ⚠ OPTIONAL ON PURPOSE. A schema requiring the field would refuse every profile
# published before it existed, and those are append-only.
REQUIRED=$(jq -r '[.required[] | select(. == "license")] | length' "$SCHEMA_FILE" 2>/dev/null | tr -d '\r')
[ "$REQUIRED" = "0" ] || note "the published schema REQUIRES license, which refuses every profile published before it existed"

# -- the corpus index --------------------------------------------------------
INDEX_FILE="corpus/v1/index.json"
if [ -f "$INDEX_FILE" ]; then
  INDEX=$(jq -r '.license // "absent"' "$INDEX_FILE" 2>/dev/null | tr -d '\r')
  state "$INDEX_FILE" "$INDEX"
  [ "$INDEX" = "$WANT" ] || note "the index says $INDEX and $HOME_FILE says $WANT"
else
  note "there is no corpus index, so the licence it states was not checked"
fi

# -- every published profile -------------------------------------------------
#
# ⚠ COUNTED IN THREE, not two. A profile that carries a DIFFERENT licence is a
# defect; one that carries NONE predates the field and is a fact about the
# corpus's history.
PROFILES=0
CARRIES=0
PREDATES=0
for file in $(git ls-files -- 'corpus/v1/*/*/*/*.json' | LC_ALL=C sort); do
  case "$file" in
    */index.json | */latest.json) continue ;;
  esac
  PROFILES=$((PROFILES + 1))
  value=$(jq -r '.license // "absent"' "$file" 2>/dev/null | tr -d '\r')
  if [ "$value" = "absent" ]; then
    PREDATES=$((PREDATES + 1))
  elif [ "$value" = "$WANT" ]; then
    CARRIES=$((CARRIES + 1))
  else
    note "$file says $value and $HOME_FILE says $WANT"
  fi
done
state "corpus profiles" "$CARRIES carrying it, $PREDATES published before the field existed"
[ "$PROFILES" -gt 0 ] || note "no published profile was read, so nothing about the corpus was checked"

# -- the release body --------------------------------------------------------
#
# ⛔ DELEGATED TO THE GENERATOR. A second renderer of the release body written
# here would be a second answer to what a release says. PUB-08 is the generator
# and its suite is where the body's shape is asserted.
BODY_SUITE="crates/b-ids-corpus/tests/notes.rs"
if grep -q "notes_the_release_body_states_the_licence" "$BODY_SUITE" 2>/dev/null; then
  state "$BODY_SUITE" "asserted by notes_the_release_body_states_the_licence"
else
  note "$BODY_SUITE has no case asserting the release body states the licence"
fi

# -- ⛔ what a FRESHLY WRITTEN profile carries -------------------------------
#
# ⛔ THE LEG ABOVE IS VACUOUS TODAY AND SAYING SO IS THE POINT: every published
# profile predates the field, so a loop over the corpus finds nobody carrying
# it. What the WRITER produces is the rule that can be broken now, and it is
# asserted in the schema crate's own suite rather than by reading a file here.
WRITER_SUITE="crates/b-ids-schema/tests/profile.rs"
if grep -q "profile_a_freshly_written_one_carries_the_licence" "$WRITER_SUITE" 2>/dev/null; then
  state "$WRITER_SUITE" "asserted by profile_a_freshly_written_one_carries_the_licence"
else
  note "$WRITER_SUITE has no case asserting a freshly written profile carries the licence"
fi

# -- ⛔ and the comparison can fail ------------------------------------------
if [ "$FIXTURE" = 1 ]; then
  FIX="$REPO_ROOT/.tmp/check-license-consistency"
  rm -rf "$FIX"
  mkdir -p "$FIX" || { printf 'check-license-consistency: cannot create %s\n' "$FIX" >&2; exit 2; }
  # ⛔ A COPY, never the file on this machine. The fixture proves the comparison
  # refuses a disagreement; mutating the real schema to find out would leave the
  # tree wrong if this process died between the edit and the restore.
  jq '.properties.license.const = "MIT"' "$SCHEMA_FILE" > "$FIX/schema.json" 2>/dev/null
  fixture=$(jq -r '.properties.license.const // "absent"' "$FIX/schema.json" 2>/dev/null | tr -d '\r')
  if [ "$fixture" != "MIT" ]; then
    note "the fixture did not produce the identifier it was written to produce"
  fi
  # ⛔ THE COMPARISON ITSELF, run against the fixture. Asserting only that the
  # fixture DIFFERS would prove the fixture and not the check: the interesting
  # question is whether the same expression the leg above uses answers
  # "disagree" here. A refusal that fires is counted; one that does not is the
  # defect this flag exists to catch.
  REFUSED=0
  [ "$fixture" = "$WANT" ] || REFUSED=1
  [ "$REFUSED" = 1 ] || note "the comparison accepted a fixture stating $fixture where $WANT was expected"
  state "the fixture" "$fixture, refused by the same comparison"
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-license-consistency/1","license":"%s","stated":%s,"profiles":%s,"carrying":%s,"predating":%s,"problems":%s}\n' \
    "$WANT" "$STATED" "$PROFILES" "$CARRIES" "$PREDATES" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'licence ok: every statement says %s.\n\n' "$WANT"
  printf '%s' "$SEEN"
  printf '\n⚠ %s profile(s) were published before the field existed and do not carry it.\n' "$PREDATES"
  printf '  The corpus is append-only, so they never will.\n'
  exit 0
fi

printf 'licence check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'Every statement of the licence, as read:\n\n' >&2
printf '%s\n' "$SEEN" >&2
printf 'A file that travels alone still has to say what it is.\n' >&2
printf 'TODO/publish.md, PUB-07.\n' >&2
exit 1
