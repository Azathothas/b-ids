#!/bin/sh
# 20-compare-capture-modes.sh - does completing the handshake change what the
# browser offers before it?
#
# ⭐ THE QUESTION, and it decides how much of this corpus is a reading of the
# harness rather than of the browser. This project has two capture surfaces over
# one wire: --raw reads the ClientHello and closes without completing a
# handshake, and --ca-out completes one so the browser's HTTP/2 becomes
# reachable at all. Every profile in the corpus is taken through the second. If
# the second also changes the TLS half, every one of them is measuring the
# instrument.
#
# -- ⛔ THE CONTROL, WHICH IS THE WHOLE DESIGN -------------------------------
#
# ONE resolved browser, TWO runs, and the ONLY difference between them is the
# capture surface. Both runs are launched with the same switches, the same
# throwaway-profile discipline and the SAME per-launch key pin, even though the
# raw run never presents a certificate for that pin to check. A run that dropped
# the pin would be changing two things at once and could not attribute either.
#
# ⚠ AND A BROWSER DRAWS GREASE PER CONNECTION AND SHUFFLES ITS EXTENSION ORDER
# PER CONNECTION. So two captures of one build differ in several fields with no
# mode change involved. b_ids_harness::modes measures stability INSIDE each run
# before comparing across runs, and reports a field that varies within a run as
# not comparable rather than as a finding. A diff that could not tell those
# apart would print a list that reads like evidence and is not.
#
# ⚠ THE TWO RUNS ARE NOT SIMULTANEOUS, and the second is a browser that has
# already talked to this harness once. It is launched into a fresh throwaway
# profile, so it carries no session from the first, but the ORDER is a condition
# of the result rather than a detail.
#
# -- ⭐ SEVERAL ROUNDS, BECAUSE A COLD HANDSHAKE IS SAMPLED PER RUN -----------
#
# ⛔ MEASURED HERE, on the first run of this script: a single terminating run
# produced 0 cold connections and 5 resumed ones. The first connection of a
# navigation completes a handshake, the server issues a ticket, and every
# connection after it resumes. So more CONNECTIONS do not buy more cold
# handshakes; more RUNS do, because each launch gets a fresh throwaway profile
# and its first connection has no ticket to offer.
#
# ⚠ That is the same discipline as "one handshake is not a sample", one level
# up: the thing being sampled here is the cold hello, and its sample size is the
# number of ROUNDS rather than the number of connections. Each round costs two
# browser launches, so the default is small on purpose.
#
# Usage:
#   sh experiments/20-compare-capture-modes.sh
#   sh experiments/20-compare-capture-modes.sh --headless
#   sh experiments/20-compare-capture-modes.sh --rounds 5
#
# Exit codes: 0 the comparison ran and the modes agree,
#             1 it ran and a field differs,
#             2 it could not run.

set -u

HEADLESS=""
ROUNDS=3
while [ $# -gt 0 ]; do
  case "$1" in
    --headless) HEADLESS="--headless" ;;
    --rounds) shift; ROUNDS="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf '20-compare-capture-modes: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done
# ⛔ Validated rather than trusted. `--rounds` with no value or a word would make
# the loop below either never run or never stop, and both look like the script
# working until nothing comes out.
case "$ROUNDS" in
  ''|*[!0-9]*) printf '20-compare-capture-modes: --rounds needs a number\n' >&2; exit 2 ;;
esac
[ "$ROUNDS" -ge 1 ] || { printf '20-compare-capture-modes: --rounds is at least 1\n' >&2; exit 2; }

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/.." && pwd) || exit 2
cd "$ROOT" || exit 2

command -v cargo >/dev/null 2>&1 || { printf '20-compare-capture-modes: cargo not found\n' >&2; exit 2; }
command -v timeout >/dev/null 2>&1 || { printf '20-compare-capture-modes: timeout not found\n' >&2; exit 2; }

OUT="$ROOT/.tmp/20-compare-capture-modes"
mkdir -p "$OUT" || exit 2

printf 'building\n'
cargo build -q -p b-ids-harness -p b-ids-driver || {
  printf '20-compare-capture-modes: the workspace did not build\n' >&2
  exit 2
}

BIN="$ROOT/target/debug"
HARNESS="$BIN/b-ids-harness"
DRIVER="$BIN/b-ids-driver"
[ -x "$HARNESS" ] || HARNESS="$HARNESS.exe"
[ -x "$DRIVER" ] || DRIVER="$DRIVER.exe"
for exe in "$HARNESS" "$DRIVER"; do
  [ -x "$exe" ] || { printf '20-compare-capture-modes: %s is not executable\n' "$exe" >&2; exit 2; }
done

printf '\nresolving a browser\n'
"$DRIVER" resolve --json > "$OUT/resolved.jsonl" 2>"$OUT/resolve.err" || {
  printf '20-compare-capture-modes: nothing resolved. %s\n' "$(cat "$OUT/resolve.err")" >&2
  exit 2
}
head -1 "$OUT/resolved.jsonl"

# One run of one surface. ⚠ The pin is minted per run and passed to the browser
# in BOTH modes, so the browser's own configuration is identical and the surface
# is the only variable.
run_mode() {
  rm_label=$1
  shift
  rm_captures="$OUT/$rm_label.jsonl"
  rm_err="$OUT/$rm_label.err"
  : > "$rm_captures"
  : > "$rm_err"

  printf '\n%s: binding\n' "$rm_label"
  "$HARNESS" --json --handshakes 6 --run-timeout-ms 45000 --timeout-ms 5000 "$@" \
    > "$rm_captures" 2>"$rm_err" &
  rm_pid=$!

  # ⚠ A BOUNDED TIMER, and it is the one place this script needs one. The base
  # URL is printed by another process at startup and there is no local handle to
  # block on until it exists. docs/conventions/shell.md section 10.
  rm_base=""
  rm_pin=""
  rm_i=0
  while [ $rm_i -lt 40 ]; do
    rm_base=$(awk 'NR==1 { print $1; exit }' "$rm_captures")
    rm_pin=$(awk '/^pin: /{ sub(/^pin: /, ""); print; exit }' "$rm_err")
    [ -n "$rm_base" ] && break
    timeout 0.25 tail -f /dev/null
    rm_i=$((rm_i + 1))
  done
  if [ -z "$rm_base" ]; then
    printf '20-compare-capture-modes: %s printed no base URL in 10s\n' "$rm_label" >&2
    cat "$rm_err" >&2
    kill "$rm_pid" 2>/dev/null
    wait "$rm_pid" 2>/dev/null
    return 2
  fi
  printf '%s: harness at %s\n' "$rm_label" "$rm_base"

  # ⚠ The raw surface mints no authority, so it prints no pin. The browser is
  # then launched without one, which is the same configuration minus a flag that
  # names a key nothing will present. It is recorded rather than hidden.
  if [ -n "$rm_pin" ]; then
    "$DRIVER" drive --url "$rm_base" --pin "$rm_pin" --timeout-ms 40000 $HEADLESS \
      > "$OUT/$rm_label.driven.txt" 2>&1
  else
    printf '%s: no pin was minted, so the browser is launched without one\n' "$rm_label"
    "$DRIVER" drive --url "$rm_base" --timeout-ms 40000 $HEADLESS \
      > "$OUT/$rm_label.driven.txt" 2>&1
  fi

  wait "$rm_pid"
  rm_rc=$?
  printf '%s: harness exit=%s, %s connection(s) recorded\n' \
    "$rm_label" "$rm_rc" "$(awk 'NR>1 && /^\{/' "$rm_captures" | wc -l | tr -d ' ')"
  tail -n 1 "$rm_err"
  return 0
}

# ⛔ The accumulating files are truncated ONCE, before the rounds, and appended
# to after each. A round that truncated them would measure the last round only
# while printing a total.
: > "$OUT/raw.all.jsonl"
: > "$OUT/terminated.all.jsonl"

ROUND=1
while [ "$ROUND" -le "$ROUNDS" ]; do
  printf '\n== round %s of %s ==\n' "$ROUND" "$ROUNDS"
  run_mode raw --raw || exit 2
  run_mode terminated --ca-out "$OUT/ca.pem" || exit 2
  cat "$OUT/raw.jsonl" >> "$OUT/raw.all.jsonl"
  cat "$OUT/terminated.jsonl" >> "$OUT/terminated.all.jsonl"
  ROUND=$((ROUND + 1))
done

printf '\ncomparing %s round(s)\n' "$ROUNDS"
cargo run -q -p b-ids-harness --example compare-modes -- \
  "$OUT/raw.all.jsonl" "$OUT/terminated.all.jsonl" > "$OUT/comparison.txt" 2>&1
COMPARE_RC=$?
cat "$OUT/comparison.txt"

printf '\nconditions\n'
printf '  host      %s\n' "$(uname -s -r 2>/dev/null || printf 'unknown')"
printf '  rustc     %s\n' "$(rustc --version 2>/dev/null || printf 'unknown')"
printf '  taken     %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  headless  %s\n' "${HEADLESS:-no}"
printf '  rounds    %s, each one raw run then one terminated run\n' "$ROUNDS"
printf '  order     raw first, then terminated, each into a fresh throwaway profile\n'
printf '\nleft in %s\n' "$OUT"
exit "$COMPARE_RC"
