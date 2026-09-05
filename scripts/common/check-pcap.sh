#!/bin/sh
# check-pcap.sh - is every published packet capture the profile's own bytes, and
# does it say it was synthesised?
#
# ⛔ A SYNTHESISED CAPTURE THAT IS INDISTINGUISHABLE FROM A REAL ONE is the one
# thing docs/history/todo/publish.md, PUB-06, forbids. A reader who opens this file expecting
# a capture has to be told, in the file, in a field a standard tool displays.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the assembler writes one capture per profile that carries a raw hello,
#      and the count is asserted rather than assumed;
#   2. ⭐ EVERY FILE CARRIES THE PROFILE'S OWN ClientHello, BYTE FOR BYTE. The
#      comparison is against raw/v1/, which is the corpus's own sidecar, and it
#      is done by hex-dumping the file rather than by asking the generator;
#   3. ⛔ EVERY FILE SAYS IT IS SYNTHESISED, in its section comment;
#   4. the file is a pcapng section: the block-type magic and the byte-order
#      magic are both where the format puts them;
#   5. the suite that owns the block arithmetic is present, case by case.
#
# ⚠ THE DISSECTION LEG IS A SKIP WHERE THERE IS NO TOOL. `tshark` is what would
# read the file the way a network engineer will, and it is not on every machine.
# ⛔ A skip is reported as a skip: this check does NOT claim a standard tool
# opened the file when none was there to try.
#
# ⭐ WHY THE PAYLOAD LEG IS THE IMPORTANT ONE. Everything else in the file is
# generated, so the only thing a consumer can be misled about is whether the
# bytes are the measured ones. That is the leg that runs everywhere.
#
# Usage:
#   sh scripts/common/check-pcap.sh
#   sh scripts/common/check-pcap.sh --json
#
# Exit codes: 0 every capture is what it should be, 1 one is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-pcap: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-pcap: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-pcap: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-pcap: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v cargo >/dev/null 2>&1 || { printf 'check-pcap: cargo not found\n' >&2; exit 2; }
command -v xxd >/dev/null 2>&1 || { printf 'check-pcap: xxd not found, and the payload leg needs it\n' >&2; exit 2; }

CORPUS_ROOT=$(sh "$REPO_ROOT/scripts/common/corpus-root.sh") || {
  printf 'check-pcap: no corpus is reachable, so nothing was checked\n' >&2
  exit 2
}
export B_IDS_CORPUS_ROOT="$CORPUS_ROOT"

SUITE="$REPO_ROOT/crates/b-ids-corpus/tests/pcap.rs"
[ -f "$SUITE" ] || { printf 'check-pcap: no suite at %s\n' "$SUITE" >&2; exit 2; }

# ⛔ THE CASES ARE NAMED HERE AND ASSERTED THERE, so a suite that lost one is
# caught by this check rather than by nobody.
WANT='pcap_the_client_hello_is_the_profiles_own_bytes
pcap_every_block_declares_its_length_at_both_ends
pcap_the_file_says_it_was_synthesised
pcap_a_profile_with_no_raw_hello_produces_nothing
pcap_the_header_checksums_are_computed_rather_than_zero'

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

CASES_WANTED=0
for want in $WANT; do
  CASES_WANTED=$((CASES_WANTED + 1))
  grep -q "fn $want" "$SUITE" || note "$want is not in the suite"
done

# ⛔ READ FROM THE PROCESS, UNPIPED.
OUT="$REPO_ROOT/.tmp/check-pcap"
rm -rf "$OUT"
mkdir -p "$OUT" || { printf 'check-pcap: cannot create %s\n' "$OUT" >&2; exit 2; }

cargo test -q -p b-ids-corpus --test pcap > "$OUT/tests.log" 2>&1
rc_t=$?
CASES=$(awk '/^running [0-9]+ tests/ { print $2; exit }' "$OUT/tests.log")
[ "$rc_t" = 0 ] || note "the suite failed. Its output is in .tmp/check-pcap/tests.log"
[ "${CASES:-0}" -ge "$CASES_WANTED" ] 2>/dev/null ||
  note "the suite ran ${CASES:-0} case(s) where at least $CASES_WANTED were expected"

cargo run -q -p b-ids-corpus -- publish --root "$CORPUS_ROOT" --out "$OUT/tree" \
  > "$OUT/publish.log" 2>&1 || {
  printf 'check-pcap: the assembler did not build the tree\n' >&2
  cat "$OUT/publish.log" >&2
  exit 2
}

# ⚠ ONE CAPTURE PER PROFILE THAT CARRIES A RAW HELLO, and the denominator is the
# sidecars rather than the profiles: a profile with no sidecar produces none, by
# design, and counting against profiles would report that as a defect.
SIDECARS=$(find "$CORPUS_ROOT/raw/v1" -name '*.hello.hex' 2>/dev/null | wc -l | tr -d ' ')
FILES=$(find "$OUT/tree/pcap" -name '*.pcapng' 2>/dev/null | wc -l | tr -d ' ')
[ "$SIDECARS" = "$FILES" ] ||
  note "$SIDECARS raw hello(s) produced $FILES capture(s), and it is one each"
[ "${FILES:-0}" -ge 1 ] 2>/dev/null ||
  note "no capture was written at all, so nothing below checked anything"

MARKER='SYNTHESISED BY b-ids'
CHECKED=0
for f in $(find "$OUT/tree/pcap" -name '*.pcapng' 2>/dev/null | LC_ALL=C sort); do
  rel=${f#"$OUT/tree/pcap/v1/"}
  route=$(dirname "$rel")
  version=$(basename "$rel" .pcapng)
  sidecar="$CORPUS_ROOT/raw/v1/$route/$version.hello.hex"
  if [ ! -f "$sidecar" ]; then
    note "$rel has no sidecar at raw/v1/$route/$version.hello.hex to compare against"
    continue
  fi

  # ⭐ THE INDEPENDENT COMPARISON. The file is hex-dumped and the corpus's own
  # recorded hex has to appear in it as a contiguous run. Nothing here asks the
  # generator what it wrote.
  # ⚠ Both sides are stripped of whitespace, because the sidecar is one line
  # with no trailing newline and xxd wraps.
  want=$(tr -d ' \t\r\n' < "$sidecar")
  xxd -p "$f" | tr -d ' \t\r\n' > "$OUT/dump.hex"
  if ! grep -qF "$want" "$OUT/dump.hex"; then
    note "$rel does not carry the ClientHello raw/v1/$route/$version.hello.hex records"
  fi

  # ⛔ AND IT SAYS WHAT IT IS.
  grep -qF "$MARKER" "$f" 2>/dev/null || note "$rel does not say it was synthesised"

  # ⚠ THE FORMAT'S OWN TWO MAGIC NUMBERS, at the two places the format puts
  # them: the section-header block type, then the byte-order magic. A file that
  # lost either is not a pcapng section whatever else is in it.
  head=$(xxd -p -l 12 "$f" | tr -d ' \t\r\n')
  case "$head" in
    0a0d0d0a*4d3c2b1a) ;;
    *) note "$rel does not open with a pcapng section header: $head" ;;
  esac
  CHECKED=$((CHECKED + 1))
done

# ⚠ THE DISSECTION LEG, WHICH IS A SKIP RATHER THAN A PASS WHERE THERE IS NO
# TOOL. ⛔ This check does not claim a standard tool opened the file when none
# was there to try.
DISSECTED=skipped
if command -v tshark >/dev/null 2>&1; then
  one=$(find "$OUT/tree/pcap" -name '*.pcapng' 2>/dev/null | LC_ALL=C sort | head -1)
  if [ -n "$one" ]; then
    if tshark -r "$one" -T fields -e frame.number > "$OUT/tshark.log" 2>&1; then
      DISSECTED=ok
    else
      DISSECTED=failed
      note "tshark could not read $one. Its output is in .tmp/check-pcap/tshark.log"
    fi
  fi
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-pcap/1","captures":%s,"sidecars":%s,"checked":%s,"cases":%s,"dissected":"%s","problems":%s}\n' \
    "$FILES" "$SIDECARS" "$CHECKED" "${CASES:-0}" "$DISSECTED" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'pcap ok: %s capture(s) over %s raw hello(s), every one carrying the\n' "$FILES" "$SIDECARS"
  printf '  ClientHello its profile recorded, byte for byte, and every one saying\n'
  printf '  it was synthesised. %s suite case(s).\n' "${CASES:-0}"
  case "$DISSECTED" in
    ok) printf '  ⭐ A standard tool read it: tshark opened the first one.\n' ;;
    *) printf '  ⚠ SKIP the dissection leg: no tshark on this host, so nothing here\n'
       printf '  says a standard tool can open the file. Install tshark and it runs.\n' ;;
  esac
  exit 0
fi

printf 'pcap check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A synthesised capture that is indistinguishable from a real one is the\n' >&2
printf 'one thing this entry forbids. docs/history/todo/publish.md, PUB-06.\n' >&2
exit 1
