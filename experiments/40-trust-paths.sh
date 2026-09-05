#!/bin/sh
# 40-trust-paths.sh - which trust route lets a browser complete a handshake with
# this project's harness, on this platform?
#
# ⭐ THE QUESTION, and it decides how a capture is taken on each platform. The
# harness mints its own authority, so the subject has to be made to trust it
# somehow, and every route is a CONDITION of whatever is captured through it.
# docs/history/todo/driver.md, DRIVER-04.
#
# -- ⛔ THE FOUR ROUTES, AND WHY ONE OF THEM IS NOT RUN HERE ------------------
#
#   pin                   --ignore-certificate-errors-spki-list=<key>. The
#                         subject verifies against ONE key, for ONE launch. No
#                         trust store is changed and every other key is still
#                         refused. This is the standing route.
#   none                  no flag at all. ⭐ THE NEGATIVE CONTROL: if this
#                         completes a handshake, the pin is not what is doing
#                         the work and every condition recorded is wrong.
#   verification-disabled --ignore-certificate-errors --test-type. Changes what
#                         the browser ACCEPTS after the handshake rather than
#                         what it SENDS. ⛔ A CAPTURE TOOL AND NEVER SOMETHING
#                         TO SHIP IN A CLIENT.
#   trust-store           the harness authority installed as a root the browser
#                         reads. ⛔ NOT RUN HERE. Installing a root is a change
#                         to this machine's security configuration, and the
#                         operator ruled 2026-09-01 that it belongs on a runner
#                         that is thrown away. HARNESS-14 is that job. This
#                         script REPORTS the route as not attempted and names
#                         the command rather than running it.
#
# ⚠ AND THE INHERITED CLAIM THIS TESTS. docs/inherited-claims.md section 8:
# "Chrome on Linux does not read the user's NSS database for server
# authentication", so certutil -t "C,," succeeds and the browser still refuses.
# That is a claim about Linux, inherited and unmeasured here. This script says
# which routes work on THE PLATFORM IT RAN ON and nothing about any other.
#
# -- ⚠ WHAT A ROUTE "WORKING" MEANS ------------------------------------------
#
# A completed TLS handshake with a connection that reached HTTP/2. A browser
# that connects and closes has not completed anything a capture can use, and
# the harness reports the two differently.
#
# Usage:
#   sh experiments/40-trust-paths.sh
#   sh experiments/40-trust-paths.sh --headless
#
# Exit codes: 0 the report ran and the standing route works,
#             1 it ran and the standing route did not complete a handshake,
#             2 it could not run.

set -u

HEADLESS=""
BROWSER=""
while [ $# -gt 0 ]; do
  case "$1" in
    --headless) HEADLESS="--headless" ;;
    --browser) shift; BROWSER="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf '40-trust-paths: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/.." && pwd) || exit 2
cd "$ROOT" || exit 2

command -v cargo >/dev/null 2>&1 || { printf '40-trust-paths: cargo not found\n' >&2; exit 2; }
command -v timeout >/dev/null 2>&1 || { printf '40-trust-paths: timeout not found\n' >&2; exit 2; }

BROWSER_FLAG=""
if [ -n "$BROWSER" ]; then
  BROWSER_FLAG="--browser $BROWSER"
fi

OUT="$ROOT/.tmp/40-trust-paths"
mkdir -p "$OUT" || exit 2

printf 'building\n'
cargo build -q -p b-ids-harness -p b-ids-driver || {
  printf '40-trust-paths: the workspace did not build\n' >&2
  exit 2
}

BIN="$ROOT/target/debug"
HARNESS="$BIN/b-ids-harness"
DRIVER="$BIN/b-ids-driver"
[ -x "$HARNESS" ] || HARNESS="$HARNESS.exe"
[ -x "$DRIVER" ] || DRIVER="$DRIVER.exe"
for exe in "$HARNESS" "$DRIVER"; do
  [ -x "$exe" ] || { printf '40-trust-paths: %s is not executable\n' "$exe" >&2; exit 2; }
done

printf '\nresolving a browser\n'
# shellcheck disable=SC2086 # BROWSER_FLAG is a flag and its value, or empty
"$DRIVER" resolve $BROWSER_FLAG --json > "$OUT/resolved.jsonl" 2>"$OUT/resolve.err" || {
  printf '40-trust-paths: nothing resolved. %s\n' "$(cat "$OUT/resolve.err")" >&2
  exit 2
}
head -1 "$OUT/resolved.jsonl"

# One route. ⚠ Every run mints its own authority and uses its own throwaway
# profile, so no run inherits anything from the one before it.
#
# Prints one fixed line: `route=<name> handshakes=<n> h2=<n> connections=<n>`.
# ⛔ A fixed line rather than prose, because a report a machine cannot read is a
# report that gets summarised by hand.
run_route() {
  rr_route=$1
  rr_captures="$OUT/$rr_route.jsonl"
  rr_err="$OUT/$rr_route.err"
  : > "$rr_captures"
  : > "$rr_err"

  "$HARNESS" --json --ca-out "$OUT/$rr_route.ca.pem" --no-resumption \
    --handshakes 4 --run-timeout-ms 30000 --timeout-ms 5000 \
    > "$rr_captures" 2>"$rr_err" &
  rr_pid=$!

  # ⚠ A BOUNDED TIMER, and it is the one place this script needs one. The base
  # URL is printed by another process at startup and there is no local handle to
  # block on until it exists. docs/conventions/shell.md section 10.
  rr_base=""
  rr_pin=""
  rr_i=0
  while [ $rr_i -lt 40 ]; do
    rr_base=$(awk 'NR==1 { print $1; exit }' "$rr_captures")
    rr_pin=$(awk '/^pin: /{ sub(/^pin: /, ""); print; exit }' "$rr_err")
    [ -n "$rr_base" ] && [ -n "$rr_pin" ] && break
    timeout 0.25 tail -f /dev/null
    rr_i=$((rr_i + 1))
  done
  if [ -z "$rr_base" ] || [ -z "$rr_pin" ]; then
    printf '40-trust-paths: %s printed no base URL or pin in 10s\n' "$rr_route" >&2
    kill "$rr_pid" 2>/dev/null
    wait "$rr_pid" 2>/dev/null
    return 2
  fi

  # ⛔ ONE FLAG DIFFERENT PER ROUTE, and nothing else. A run that also changed
  # the surface or the profile would be changing two things at once and could
  # not attribute either.
  case "$rr_route" in
    pin) rr_trust="--pin $rr_pin" ;;
    none) rr_trust="" ;;
    verification-disabled) rr_trust="--disable-verification" ;;
    *) printf '40-trust-paths: unknown route %s\n' "$rr_route" >&2; return 2 ;;
  esac

  # shellcheck disable=SC2086 # each is a flag and its value, or empty
  "$DRIVER" drive --url "$rr_base" --timeout-ms 25000 \
    --log "$OUT/$rr_route.browser.log" $rr_trust $HEADLESS $BROWSER_FLAG \
    > "$OUT/$rr_route.driven.txt" 2>&1

  wait "$rr_pid"
  rr_connections=$(awk 'NR>1 && /^\{/' "$rr_captures" | wc -l | tr -d ' ')
  # ⚠ NO FALLBACK ON THE EXIT CODE. `grep -c` PRINTS 0 and EXITS 1 when it
  # matches nothing, so a `|| printf 0` appends a second zero and the report
  # reads `handshakes=0\n0`. Measured here on the negative-control route.
  rr_handshakes=$(grep -c '"termination":{' "$rr_captures" 2>/dev/null)
  rr_h2=$(grep -c '"http2":{' "$rr_captures" 2>/dev/null)
  printf 'route=%s handshakes=%s h2=%s connections=%s\n' \
    "$rr_route" "$rr_handshakes" "$rr_h2" "$rr_connections"
  return 0
}

printf '\n-- routes this host can exercise without changing the machine --\n'
# ⛔ RUN ONCE AND CAPTURED. A second launch to re-read the line would be a
# second measurement reported as the first one.
PIN_LINE=$(run_route pin) || exit 2
printf '%s\n' "$PIN_LINE"
run_route none || exit 2
run_route verification-disabled || exit 2

printf '\n-- the route this host does NOT exercise --\n'
printf 'route=trust-store handshakes=- h2=- connections=- not-attempted\n'
printf '  ⛔ Installing a root is a change to this machine, and the operator ruled\n'
printf '     2026-09-01 that it belongs on a runner that is thrown away. HARNESS-14.\n'
case "$(uname -s 2>/dev/null || printf unknown)" in
  MINGW*|MSYS*|CYGWIN*|Windows*)
    printf '  would be: certutil -addstore -user Root <ca.pem>\n'
    printf '  ⚠ and whether a browser reads THAT store is the question DRIVER-04 asks\n' ;;
  Linux*)
    printf '  would be: certutil -d sql:%s/.pki/nssdb -A -t %s -n b-ids -i <ca.pem>\n' "$HOME" '"C,,"'
    printf '  ⚠ docs/inherited-claims.md section 8: Chrome on Linux does NOT read that\n'
    printf '     database for server authentication, so this route is expected to fail\n' ;;
  Darwin*)
    printf '  would be: security add-trusted-cert -d -k <keychain> <ca.pem>\n' ;;
  *)
    printf '  no command is recorded for this platform\n' ;;
esac

printf '\nconditions\n'
printf '  host      %s\n' "$(uname -s -r 2>/dev/null || printf 'unknown')"
# ⚠ The FIELD, not the fourth quoted token. Splitting on a bare quote gave
# `chrome` here, which is the family rather than the build.
printf '  browser   %s %s\n' \
  "$(awk -F'"name":"' 'NR==1 { split($2, a, /"/); print a[1] }' "$OUT/resolved.jsonl")" \
  "$(awk -F'"version":"' 'NR==1 { split($2, a, /"/); print a[1] }' "$OUT/resolved.jsonl")"
printf '  rustc     %s\n' "$(rustc --version 2>/dev/null || printf 'unknown')"
printf '  taken     %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  headless  %s\n' "${HEADLESS:-no}"
printf '  handshakes 4 per route, resumption refused, one throwaway profile each\n'
printf '\nleft in %s\n' "$OUT"

# ⛔ The exit code is about the STANDING route. A run in which the pin stopped
# working is a run in which every capture this project takes is broken, and that
# is not a report, it is a failure.
PIN_H2=$(printf '%s' "$PIN_LINE" | awk '{ for (i = 1; i <= NF; i++) if ($i ~ /^h2=/) { sub(/^h2=/, "", $i); print $i } }')
if [ "${PIN_H2:-0}" -ge 1 ]; then
  printf '\nthe standing route completed %s connection(s) to HTTP/2\n' "$PIN_H2"
  exit 0
fi
printf '\n40-trust-paths: the standing route reached no HTTP/2 connection\n' >&2
exit 1
