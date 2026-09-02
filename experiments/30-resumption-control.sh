#!/bin/sh
# 30-resumption-control.sh - does refusing session tickets change the cold
# hello, or only which connections are cold?
#
# ⭐ THE QUESTION, and a claim in the tree rests on it. The harness can be told
# not to issue session tickets, and experiments/10-first-profile.sh passes that
# switch, so every corpus profile taken since 2026-09-02 is captured under it
# and records `refused` in `captured.resumption`. The switch is worth having
# only if it changes WHICH connections are cold rather than WHAT a cold hello
# is. If it changes the hello, every profile taken through it is a reading of
# the harness rather than of the browser.
#
# -- ⛔ WHY THE SWITCH EXISTS AT ALL ------------------------------------------
#
# MEASURED on hosted runners, twice, capture.yml runs 33579619515 and
# 33580371329: Chrome on ubuntu-latest abandoned both of the connections that
# were not resumed and resumed every one it kept, so the navigation produced NO
# cold connection and nothing could be published from it. More connections do
# not help: the first completed handshake leaves a ticket and everything after
# it resumes. TODO/corpus.md, CORPUS-02.
#
# -- ⛔ THE CONTROL, WHICH IS THE WHOLE DESIGN --------------------------------
#
# ONE resolved browser, TWO runs, both TERMINATING with a per-launch key pin,
# and the ONLY difference is --no-resumption. Both runs are launched with the
# same switches into a fresh throwaway profile. A run that also changed the
# surface would be changing two things at once and could not attribute either.
#
# ⚠ AND A BROWSER DRAWS GREASE PER CONNECTION AND SHUFFLES ITS EXTENSION ORDER
# PER CONNECTION, so two captures of one build differ with no change of
# configuration at all. b_ids_harness::modes measures stability INSIDE each run
# before comparing across runs and reports a field that varies within a run as
# not comparable rather than as a finding.
#
# ⛔ ONLY THE COMPARABLE CONNECTIONS ARE COMPARED. The offered run contains
# resumed connections, and a resumed hello offers a pre-shared key where a cold
# one offers an empty session ticket: comparing the runs whole would report that
# as a finding about the switch, which is exactly what it is not.
# b_ids_harness::comparable is the selector the comparison goes through.
#
# Usage:
#   sh experiments/30-resumption-control.sh
#   sh experiments/30-resumption-control.sh --headless
#   sh experiments/30-resumption-control.sh --rounds 5
#
# Exit codes: 0 the comparison ran and the two configurations agree,
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
    *) printf '30-resumption-control: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done
# ⛔ Validated rather than trusted, the same way 20-compare-capture-modes
# validates it: a --rounds with no value makes the loop below either never run
# or never stop, and both look like the script working until nothing comes out.
case "$ROUNDS" in
  ''|*[!0-9]*) printf '30-resumption-control: --rounds needs a number\n' >&2; exit 2 ;;
esac
[ "$ROUNDS" -ge 1 ] || { printf '30-resumption-control: --rounds is at least 1\n' >&2; exit 2; }

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/.." && pwd) || exit 2
cd "$ROOT" || exit 2

command -v cargo >/dev/null 2>&1 || { printf '30-resumption-control: cargo not found\n' >&2; exit 2; }
command -v timeout >/dev/null 2>&1 || { printf '30-resumption-control: timeout not found\n' >&2; exit 2; }

OUT="$ROOT/.tmp/30-resumption-control"
mkdir -p "$OUT" || exit 2

printf 'building\n'
cargo build -q -p b-ids-harness -p b-ids-driver || {
  printf '30-resumption-control: the workspace did not build\n' >&2
  exit 2
}

BIN="$ROOT/target/debug"
HARNESS="$BIN/b-ids-harness"
DRIVER="$BIN/b-ids-driver"
[ -x "$HARNESS" ] || HARNESS="$HARNESS.exe"
[ -x "$DRIVER" ] || DRIVER="$DRIVER.exe"
for exe in "$HARNESS" "$DRIVER"; do
  [ -x "$exe" ] || { printf '30-resumption-control: %s is not executable\n' "$exe" >&2; exit 2; }
done

printf '\nresolving a browser\n'
"$DRIVER" resolve --json > "$OUT/resolved.jsonl" 2>"$OUT/resolve.err" || {
  printf '30-resumption-control: nothing resolved. %s\n' "$(cat "$OUT/resolve.err")" >&2
  exit 2
}
head -1 "$OUT/resolved.jsonl"

# One run of one configuration. ⚠ Both mint an authority and both pass the pin,
# so the browser is configured identically and the ticket policy is the only
# variable.
run_config() {
  rc_label=$1
  shift
  rc_captures="$OUT/$rc_label.jsonl"
  rc_err="$OUT/$rc_label.err"
  : > "$rc_captures"
  : > "$rc_err"

  printf '\n%s: binding\n' "$rc_label"
  "$HARNESS" --json --ca-out "$OUT/$rc_label.ca.pem" \
    --handshakes 6 --run-timeout-ms 45000 --timeout-ms 5000 "$@" \
    > "$rc_captures" 2>"$rc_err" &
  rc_pid=$!

  # ⚠ A BOUNDED TIMER, and it is the one place this script needs one. The base
  # URL is printed by another process at startup and there is no local handle to
  # block on until it exists. docs/conventions/shell.md section 10.
  rc_base=""
  rc_pin=""
  rc_resumption=""
  rc_i=0
  while [ $rc_i -lt 40 ]; do
    rc_base=$(awk 'NR==1 { print $1; exit }' "$rc_captures")
    rc_pin=$(awk '/^pin: /{ sub(/^pin: /, ""); print; exit }' "$rc_err")
    rc_resumption=$(awk '/^resumption: /{ sub(/^resumption: /, ""); print; exit }' "$rc_err")
    [ -n "$rc_base" ] && [ -n "$rc_pin" ] && break
    timeout 0.25 tail -f /dev/null
    rc_i=$((rc_i + 1))
  done
  if [ -z "$rc_base" ] || [ -z "$rc_pin" ]; then
    printf '30-resumption-control: %s printed no base URL or pin in 10s\n' "$rc_label" >&2
    cat "$rc_err" >&2
    kill "$rc_pid" 2>/dev/null
    wait "$rc_pid" 2>/dev/null
    return 2
  fi
  printf '%s: harness at %s, resumption=%s\n' "$rc_label" "$rc_base" "$rc_resumption"

  "$DRIVER" drive --url "$rc_base" --pin "$rc_pin" --timeout-ms 40000 $HEADLESS \
    > "$OUT/$rc_label.driven.txt" 2>&1

  wait "$rc_pid"
  rc_rc=$?
  printf '%s: harness exit=%s, %s connection(s) recorded\n' \
    "$rc_label" "$rc_rc" "$(awk 'NR>1 && /^\{/' "$rc_captures" | wc -l | tr -d ' ')"
  tail -n 1 "$rc_err"
  return 0
}

# ⛔ The accumulating files are truncated ONCE, before the rounds, and appended
# to after each. A round that truncated them would measure the last round only
# while printing a total.
: > "$OUT/offered.all.jsonl"
: > "$OUT/refused.all.jsonl"

ROUND=1
while [ "$ROUND" -le "$ROUNDS" ]; do
  printf '\n== round %s of %s ==\n' "$ROUND" "$ROUNDS"
  run_config offered || exit 2
  run_config refused --no-resumption || exit 2
  cat "$OUT/offered.jsonl" >> "$OUT/offered.all.jsonl"
  cat "$OUT/refused.jsonl" >> "$OUT/refused.all.jsonl"
  ROUND=$((ROUND + 1))
done

printf '\ncomparing %s round(s)\n' "$ROUNDS"
cargo run -q -p b-ids-harness --example compare-modes -- --labels offered,refused \
  "$OUT/offered.all.jsonl" "$OUT/refused.all.jsonl" > "$OUT/comparison.txt" 2>&1
COMPARE_RC=$?
cat "$OUT/comparison.txt"

printf '\nconditions\n'
printf '  host       %s\n' "$(uname -s -r 2>/dev/null || printf 'unknown')"
printf '  rustc      %s\n' "$(rustc --version 2>/dev/null || printf 'unknown')"
printf '  taken      %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  headless   %s\n' "${HEADLESS:-no}"
printf '  rounds     %s, each one offered run then one refused run\n' "$ROUNDS"
printf '  surface    both terminate, both pin, and the ticket policy is the only variable\n'
printf '\nleft in %s\n' "$OUT"
exit "$COMPARE_RC"
