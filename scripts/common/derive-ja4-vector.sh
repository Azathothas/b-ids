#!/bin/sh
# derive-ja4-vector.sh - derive one JA4 test vector from a published profile,
# with tools that are NOT this project's code.
#
# ⭐ A HELPER, NOT A CHECK. It prints; it writes nothing. scripts/README.md's
# five-point contract is for checks, and this is held to the header rule and the
# exit-code rule.
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
#
# Exit codes: 0 derived, 1 the profile could not be read, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
SELFTEST=0
PROFILE=""

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --selftest) SELFTEST=1 ;;
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
