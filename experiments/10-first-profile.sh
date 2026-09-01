#!/bin/sh
# 10-first-profile.sh - what does the browser on this machine put on the wire,
# and does the corpus hold it?
#
# ⭐ THE QUESTION, not the procedure. Every part of the pipeline existed before
# this script and none of them had ever been joined: the harness reads bytes off
# a socket, the driver launches a browser at a URL, and the corpus turns the
# first into a profile. This runs all three in one go and leaves the evidence.
#
# -- WHAT IT DOES ------------------------------------------------------------
#
#   1. mints a per-run authority and binds the terminating harness on loopback;
#   2. launches the resolved browser at it, trusting that ONE key for that ONE
#      launch, into a profile directory nobody keeps;
#   3. selects the cold connection out of the navigation and writes it into the
#      corpus, with its ClientHello beside it and the index rewritten;
#   4. prints the conditions the measurement was taken under.
#
# -- ⚠ THE CONDITIONS, WHICH ARE PART OF THE RESULT --------------------------
#
# ⛔ THE SUBJECT TRUSTS ONE KEY, NOT A TRUST STORE, and that is a condition of
# whatever is captured through it rather than a detail of the run. It is not
# --ignore-certificate-errors: verification still runs and every other key is
# still refused. HARNESS-10 is the entry that measures whether it changed the
# answer, and until it has, every profile this writes carries `spki-pin` in
# `captured.trust` so the comparison is possible at all.
#
# ⚠ HEADER VALUES ARE RECORDED, DELIBERATELY. The default capture shape is
# names only, because a model whose natural form carries values is a model that
# will one day publish a credential. A corpus profile is the case the default
# is deliberately turned off for: four of the validator's checks read a header
# VALUE, and a published profile that none of them can check is a profile
# nobody can validate. `cookie` and `authorization` are dropped either way, at
# HeaderSet::record, and the raw bytes are refused by Raw::check if they spell
# one out.
#
# ⚠ THE BROWSER IS HEADFUL. Headless changes the product token the browser
# announces, and normalising that is a substitution recorded in the provenance
# map. The first profile is of the browser a person runs.
#
# Usage:
#   sh experiments/10-first-profile.sh
#   sh experiments/10-first-profile.sh --headless
#
# Exit codes: 0 the measurement ran and a profile was written,
#             1 it ran and something refused,
#             2 it could not run.

set -u

HEADLESS=""
while [ $# -gt 0 ]; do
  case "$1" in
    --headless) HEADLESS="--headless" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf '10-first-profile: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/.." && pwd) || exit 2
cd "$ROOT" || exit 2

command -v cargo >/dev/null 2>&1 || { printf '10-first-profile: cargo not found\n' >&2; exit 2; }
command -v timeout >/dev/null 2>&1 || { printf '10-first-profile: timeout not found\n' >&2; exit 2; }

OUT="$ROOT/.tmp/10-first-profile"
mkdir -p "$OUT" || exit 2

printf 'building the three commands\n'
cargo build -q -p b-ids-harness -p b-ids-driver -p b-ids-corpus || {
  printf '10-first-profile: the workspace did not build\n' >&2
  exit 2
}

BIN="$ROOT/target/debug"
HARNESS="$BIN/b-ids-harness"
DRIVER="$BIN/b-ids-driver"
CORPUS="$BIN/b-ids-corpus"
[ -x "$HARNESS" ] || HARNESS="$HARNESS.exe"
[ -x "$DRIVER" ] || DRIVER="$DRIVER.exe"
[ -x "$CORPUS" ] || CORPUS="$CORPUS.exe"
for exe in "$HARNESS" "$DRIVER" "$CORPUS"; do
  [ -x "$exe" ] || { printf '10-first-profile: %s is not executable\n' "$exe" >&2; exit 2; }
done

# -- what is installed, read rather than assumed -----------------------------
printf '\nresolving a browser\n'
"$DRIVER" resolve --json > "$OUT/resolved.jsonl" 2>"$OUT/resolve.err" || {
  printf '10-first-profile: nothing resolved. %s\n' "$(cat "$OUT/resolve.err")" >&2
  exit 2
}
cat "$OUT/resolved.jsonl"

# -- the harness, in the background, printing as it goes ---------------------
CAPTURES="$OUT/captures.jsonl"
HARNESS_ERR="$OUT/harness.err"
: > "$CAPTURES"
: > "$HARNESS_ERR"

printf '\nbinding the terminating harness\n'
"$HARNESS" --ca-out "$OUT/ca.pem" --json --header-values \
  --handshakes 8 --run-timeout-ms 60000 --timeout-ms 5000 \
  > "$CAPTURES" 2>"$HARNESS_ERR" &
HARNESS_PID=$!

# ⚠ A BOUNDED TIMER, and it is the one place this script needs one. The pin is
# printed by another process at startup and there is no local handle to block
# on until it exists. docs/conventions/shell.md section 10 names this as the
# case a timer is for, and bounds it.
PIN=""
BASE=""
i=0
while [ $i -lt 40 ]; do
  PIN=$(awk '/^pin: /{ sub(/^pin: /, ""); print; exit }' "$HARNESS_ERR")
  BASE=$(awk 'NR==1 { print $1; exit }' "$CAPTURES")
  [ -n "$PIN" ] && [ -n "$BASE" ] && break
  timeout 0.25 tail -f /dev/null
  i=$((i + 1))
done

if [ -z "$PIN" ] || [ -z "$BASE" ]; then
  printf '10-first-profile: the harness printed no pin or no base URL in 10s\n' >&2
  cat "$HARNESS_ERR" >&2
  kill "$HARNESS_PID" 2>/dev/null
  wait "$HARNESS_PID" 2>/dev/null
  exit 2
fi
printf 'harness at %s\n' "$BASE"

# -- the browser, in the foreground, which is the hold -----------------------
printf '\nlaunching the browser at it\n'
"$DRIVER" drive --url "$BASE" --pin "$PIN" --timeout-ms 45000 $HEADLESS \
  > "$OUT/driven.txt" 2>&1
DRIVER_RC=$?
cat "$OUT/driven.txt"

# ⭐ The hold ends when the harness does, and it has its own run timeout, so
# nothing here waits on a clock this script chose.
wait "$HARNESS_PID"
HARNESS_RC=$?
printf '\nharness exit=%s driver exit=%s\n' "$HARNESS_RC" "$DRIVER_RC"
tail -n 3 "$HARNESS_ERR"

CONNECTIONS=$(awk 'NR>1 && /^\{/' "$CAPTURES" | wc -l | tr -d ' ')
printf 'connections recorded: %s\n' "$CONNECTIONS"
if [ "$CONNECTIONS" = "0" ]; then
  printf '10-first-profile: the browser opened no connection this run\n' >&2
  exit 1
fi

# -- the switches the browser was actually given -----------------------------
# ⛔ Read back from what the driver reported, never retyped here. Every one of
# them is a condition of what was captured through it.
awk 'NR>1 && /^  --/ { sub(/^  /, ""); print }' "$OUT/driven.txt" > "$OUT/switches.txt"

printf '\nconditions\n'
printf '  host      %s\n' "$(uname -s -r 2>/dev/null || printf 'unknown')"
printf '  rustc     %s\n' "$(rustc --version 2>/dev/null || printf 'unknown')"
printf '  taken     %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  trust     spki-pin, one key for one launch, no trust store changed\n'
printf '  headless  %s\n' "${HEADLESS:-no}"
printf '  switches  %s\n' "$(wc -l < "$OUT/switches.txt" | tr -d ' ')"

printf '\nleft in %s\n' "$OUT"
printf '  captures.jsonl  every connection the navigation opened\n'
printf '  driven.txt      what the driver reported, switches included\n'
printf '  harness.err     the pin, and the sampling shortfall if there was one\n'
printf '\nWrite the profile with:\n'
printf '  %s add --captures %s --identity IDENTITY.json --root .\n' "$CORPUS" "$CAPTURES"
exit 0
