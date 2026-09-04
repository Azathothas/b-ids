#!/bin/sh
# derive-ja4-vector.sh - derive one JA4 test vector from a published profile,
# with tools that are NOT this project's code.
#
# ⭐ A HELPER, NOT A CHECK. scripts/README.md's five-point contract is for
# checks, and this is held to the header rule and the exit-code rule.
#
# ⚠ IT WRITES IN EXACTLY ONE MODE AND SAYS SO: `--fill ROOT` rewrites that
# root's vectors/ja4/v1.json with the vectors that were MISSING from it, and
# every other mode prints. ⛔ It never edits a vector that is already there: a
# published expectation that moves is a vector nobody can trust, and a profile
# whose vector disagrees is a finding for the suite rather than something to
# overwrite here.
#
# -- ⛔ THE DEFECT THIS EXISTS TO CATCH -------------------------------------
#
# TODO/validator.md, VALID-04, forbids a vector whose expected value came from
# running the implementation it is meant to check, so every capture vector is
# derived with `jq` and `sha256sum`. That rule was written down and the command
# lived in a document, so deriving one was a job somebody did by hand.
#
# ⚠ On 2026-09-04 the capture matrix started adding profiles for the first time
# and five landed in one run. Every one of them failed
# `digest_vectors_every_capture_vector_matches_the_profile_it_names` until five
# vectors were derived by hand, so a working capture pipeline left the gate red
# and the only fix was a person with a shell. TODO/corpus.md, CORPUS-02.
#
# -- ⭐ WHY AUTOMATING IT DOES NOT WEAKEN THE VECTOR -------------------------
#
# ⛔ THE ARITHMETIC IS STILL NOT THIS PROJECT'S. The list building is
# scripts/fixtures/ja4-derive.jq, which jq evaluates, and the digest is
# `sha256sum`. Neither is b_ids_harness::sha256, and the test compares this
# answer against the Rust one. Two implementations, still compared; what changed
# is who types the command.
#
# ⚠ WHAT WOULD WEAKEN IT is deriving the vector from `b-ids-corpus` or from
# `b_ids_harness::digest`. Do not.
#
# -- WHAT IT PRINTS ----------------------------------------------------------
#
# Three lines, `ja4`, `ja4_r` and `ja4_ro`, or one JSON object with --json,
# shaped exactly as a `capture` entry of vectors/ja4/v1.json wants it.
#
# Usage:
#   sh scripts/common/derive-ja4-vector.sh corpus/v1/chrome/stable/win64/151.0.7922.76.json
#   sh scripts/common/derive-ja4-vector.sh --json PROFILE
#   sh scripts/common/derive-ja4-vector.sh --selftest        prove it, offline
#   sh scripts/common/derive-ja4-vector.sh --fill ROOT       every missing one
#
# ⭐ --fill IS WHY THE CAPTURE PIPELINE CAN FINISH ITS OWN JOB. A capture that
# lands a profile leaves the gate red until a vector exists for it, and before
# this mode the only fix was a person with a shell. The collect job derives them
# over the MERGED tree, for the same reason it re-derives the index there: a
# vector is a function of the profile set the run would publish, not of one
# lane's view of it.
#
# Exit codes: 0 derived, 1 the profile could not be read, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
SELFTEST=0
FILL=""
PROFILE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --selftest) SELFTEST=1 ;;
    # ⚠ --fill TAKES ITS ROOT AS THE NEXT ARGUMENT rather than reusing the
    # positional, because the positional is a PROFILE and a root is not one.
    # A flag that silently accepted either would derive one vector when it was
    # asked to fill a tree.
    --fill)
      shift
      [ $# -gt 0 ] || { printf 'derive-ja4-vector: --fill needs a corpus root\n' >&2; exit 2; }
      FILL="$1"
      ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    -*) printf 'derive-ja4-vector: unknown argument: %s\n' "$1" >&2; exit 2 ;;
    *) PROFILE="$1" ;;
  esac
  shift
done

command -v jq >/dev/null 2>&1 || { printf 'derive-ja4-vector: jq not found\n' >&2; exit 2; }
command -v sha256sum >/dev/null 2>&1 || {
  printf 'derive-ja4-vector: sha256sum not found\n' >&2
  exit 2
}
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'derive-ja4-vector: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
DERIVE="$REPO_ROOT/scripts/fixtures/ja4-derive.jq"
[ -f "$DERIVE" ] || { printf 'derive-ja4-vector: no %s\n' "$DERIVE" >&2; exit 2; }

# ⛔ THE ONE HASH RULE, IN ONE PLACE. The specification truncates SHA-256 to the
# first twelve hexadecimal characters, and a list that is empty hashes to
# `000000000000` rather than to the digest of an empty string. That second half
# is the one an implementation gets wrong.
truncated() {
  if [ -z "$1" ]; then
    printf '000000000000'
    return 0
  fi
  printf '%s' "$1" | sha256sum | cut -c1-12
}

derive() {
  _profile="$1"
  [ -f "$_profile" ] || { printf 'derive-ja4-vector: no %s\n' "$_profile" >&2; return 1; }
  _d=$(jq -r -f "$DERIVE" "$_profile" 2>/dev/null) || {
    printf 'derive-ja4-vector: %s is not a profile this can read\n' "$_profile" >&2
    return 1
  }
  _cs=$(printf '%s' "$_d" | jq -r '.ciphers_sorted')
  _es=$(printf '%s' "$_d" | jq -r '.extensions_sorted')
  _co=$(printf '%s' "$_d" | jq -r '.ciphers_original')
  _eo=$(printf '%s' "$_d" | jq -r '.extensions_original')
  _nc=$(printf '%s' "$_d" | jq -r '.ncipher')
  _nx=$(printf '%s' "$_d" | jq -r '.next')
  _sni=$(printf '%s' "$_d" | jq -r '.sni')
  _alpn=$(printf '%s' "$_d" | jq -r '.alpn')
  # ⚠ TWO DIGITS EACH, AND SATURATING AT 99, which is the specification's rule
  # rather than a formatting choice.
  [ "$_nc" -gt 99 ] 2>/dev/null && _nc=99
  [ "$_nx" -gt 99 ] 2>/dev/null && _nx=99
  _prefix=$(printf 't13%s%02d%02d%s' "$_sni" "$_nc" "$_nx" "$_alpn")
  JA4="${_prefix}_$(truncated "$_cs")_$(truncated "$_es")"
  JA4_R="${_prefix}_${_cs}_${_es}"
  JA4_RO="${_prefix}_${_co}_${_eo}"
  return 0
}

# -- ⭐ THE SELF-TEST, WHICH NEEDS NO CORPUS AND NO NETWORK ------------------
#
# ⛔ It drives the ONE rule above that has no other cover: an empty list hashes
# to twelve zeros rather than to the digest of an empty string, and a non-empty
# one is truncated to twelve characters. That is what the twin comparison
# compares, because a fetch of a profile is not something a gate check may need.
if [ "$SELFTEST" = 1 ]; then
  empty=$(truncated "")
  known=$(truncated "002f,0035")
  problems=0
  [ "$empty" = "000000000000" ] || {
    printf 'derive-ja4-vector: an empty list hashed to %s\n' "$empty" >&2
    problems=$((problems + 1))
  }
  [ "${#known}" = 12 ] || {
    printf 'derive-ja4-vector: a digest was %s characters\n' "${#known}" >&2
    problems=$((problems + 1))
  }
  case "$known" in
    [0-9a-f]*) ;;
    *) printf 'derive-ja4-vector: a digest was not lower-case hexadecimal\n' >&2
       problems=$((problems + 1)) ;;
  esac
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"derive-ja4-vector/1","selftest":true,"empty":"%s","width":%s,"problems":%s}\n' \
      "$empty" "${#known}" "$problems"
  else
    printf 'derive-ja4-vector selftest: empty=%s width=%s problems=%s\n' \
      "$empty" "${#known}" "$problems"
  fi
  [ "$problems" = 0 ] || exit 1
  exit 0
fi

# -- ⭐ FILL: EVERY VECTOR THE TREE IS MISSING, AND NOT ONE IT ALREADY HAS ----
#
# ⛔ THE ONE MODE THAT WRITES. It appends the capture vectors for profiles that
# have none and leaves every existing vector byte for byte. A vector already in
# the file is a published expectation, and rewriting one would replace the thing
# the suite checks against with whatever this run computed, which is the exact
# circularity VALID-04 forbids.
if [ -n "$FILL" ]; then
  VECTORS="$FILL/vectors/ja4/v1.json"
  [ -d "$FILL/corpus/v1" ] || {
    printf 'derive-ja4-vector: %s holds no corpus/v1\n' "$FILL" >&2
    exit 2
  }
  [ -f "$VECTORS" ] || {
    # ⛔ REFUSED RATHER THAN CREATED. The file carries the specification vectors
    # and the provenance block as well as the captures, and a fresh one written
    # here would carry neither while looking complete.
    printf 'derive-ja4-vector: no %s to fill\n' "$VECTORS" >&2
    exit 2
  }
  PRESENT_IDS=$(jq -r '[.vectors[] | select(.kind == "capture") | .id] | .[]' "$VECTORS" | tr -d '\r')
  DERIVED=0
  PRESENT=0
  TOTAL=0
  WORK="$FILL/.tmp-ja4-fill"
  rm -rf "$WORK"
  mkdir -p "$WORK" || { printf 'derive-ja4-vector: cannot create %s\n' "$WORK" >&2; exit 2; }
  cp "$VECTORS" "$WORK/current.json" || exit 2
  # ⚠ SORTED, so two runs over one tree produce one file. LC_ALL=C because a
  # locale-aware sort orders differently and this file is compared byte for byte
  # by check-data-branch once it is published.
  for p in $(find "$FILL/corpus/v1" -name '*.json' ! -name index.json ! -name latest.json | LC_ALL=C sort); do
    TOTAL=$((TOTAL + 1))
    id=$(jq -r '.id' "$p" | tr -d '\r')
    if printf '%s\n' "$PRESENT_IDS" | grep -qxF "$id"; then
      PRESENT=$((PRESENT + 1))
      continue
    fi
    derive "$p" || exit 1
    # ⚠ RELATIVE TO THE ROOT BEING FILLED, not to this repository. The tree may
    # be a merged copy under .tmp, and a path relative to the wrong base is a
    # sidecar reference that resolves nowhere.
    rel=${p#"$FILL"/}
    hello=$(printf '%s' "$rel" | sed 's|^corpus/|raw/|; s|\.json$|.hello.hex|')
    jq -n --arg id "$id" --arg hello "$hello" --arg ja4 "$JA4" \
      --arg r "$JA4_R" --arg ro "$JA4_RO" \
      '{kind: "capture", id: $id, hello: $hello, ja4: $ja4, ja4_r: $r, ja4_ro: $ro}' \
      > "$WORK/one.json" || exit 1
    jq --slurpfile add "$WORK/one.json" '.vectors += $add' "$WORK/current.json" \
      > "$WORK/next.json" || exit 1
    mv "$WORK/next.json" "$WORK/current.json"
    DERIVED=$((DERIVED + 1))
    printf 'derived %s\n' "$id"
  done
  if [ "$DERIVED" -gt 0 ]; then
    # ⚠ TWO SPACES OF INDENT AND A TRAILING NEWLINE, which is what the file
    # already carries. jq's default is two spaces; the newline is jq's too.
    #
    # ⛔ AND THE CARRIAGE RETURN IS STRIPPED. jq ON WINDOWS WRITES CRLF, which
    # this project has now been bitten by three times: the capture matrix's plan
    # reader, check-data-branch's manifest leg, and this. Measured here by
    # running both halves over one planted tree: the sh half wrote `{\r\n` and
    # the PowerShell half wrote `{\n`, so the twins produced different bytes for
    # the same derivation. A raw CR cannot appear in this file legitimately,
    # because jq escapes one inside a string as \r.
    tr -d '\r' < "$WORK/current.json" > "$VECTORS" || exit 1
  fi
  rm -rf "$WORK"
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"derive-ja4-vector/2","fill":true,"profiles":%s,"derived":%s,"present":%s}\n' \
      "$TOTAL" "$DERIVED" "$PRESENT"
  else
    printf 'derive-ja4-vector fill: %s profile(s), %s vector(s) derived, %s already present.\n' \
      "$TOTAL" "$DERIVED" "$PRESENT"
    printf '⛔ Nothing already in the file was rewritten.\n'
  fi
  exit 0
fi

[ -n "$PROFILE" ] || { printf 'derive-ja4-vector: name a profile\n' >&2; exit 2; }
derive "$PROFILE" || exit 1

if [ "$JSON" = 1 ]; then
  ID=$(jq -r '.id' "$PROFILE" | tr -d '\r')
  # ⚠ The raw sidecar's path is DERIVED from the profile's own path, the way the
  # publisher derives it, rather than named separately.
  REL=${PROFILE#"$REPO_ROOT"/}
  HELLO=$(printf '%s' "$REL" | sed 's|^corpus/|raw/|; s|\.json$|.hello.hex|')
  jq -n --arg id "$ID" --arg hello "$HELLO" --arg ja4 "$JA4" \
    --arg r "$JA4_R" --arg ro "$JA4_RO" \
    '{kind: "capture", id: $id, hello: $hello, ja4: $ja4, ja4_r: $r, ja4_ro: $ro}'
else
  printf 'ja4    %s\n' "$JA4"
  printf 'ja4_r  %s\n' "$JA4_R"
  printf 'ja4_ro %s\n' "$JA4_RO"
fi
exit 0
