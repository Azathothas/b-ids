#!/bin/sh
# 50-trust-anchor.sh - does trusting a real root change what the browser puts on
# the wire, against trusting one key for one launch?
#
# ⭐ THE QUESTION, and every profile this project has ever taken depends on the
# answer. Every capture went through a per-launch key pin rather than a root in
# the browser's trust store, and `captured.trust` records which. ⛔ Nothing has
# ever measured whether that choice changes the hello.
# TODO/harness.md, HARNESS-14.
#
# ⚠ HARNESS-10 MEASURED THE ADJACENT QUESTION AND IT IS NOT THIS ONE. It
# compared the raw surface against the terminating surface and found the two
# agree on every TLS field with a stable value. That says the act of terminating
# changes nothing; it says nothing about the pin.
#
# -- ⛔ WHY THIS NEEDS A MACHINE YOU THROW AWAY ------------------------------
#
# Installing a root certificate is a change to a machine's security
# configuration. The operator ruled 2026-09-01 that it belongs on a runner that
# is reclaimed when the job ends, and this script REFUSES to install one unless
# B_IDS_DISPOSABLE=1 says the machine is one. ⚠ On a developer host it reports
# the trust-store leg as not attempted and exits 2, which is "could not run"
# rather than a failure. CI-07.
#
# -- ⛔ THE ROOT IS GENERATED FOR THE RUN AND NEVER COMMITTED -----------------
#
# It is a capture tool. Nothing about it may resemble something to ship in a
# client, and docs/security/secrets.md is the rule. Every run mints its own and
# removes it from the store on the way out; the removal is READ BACK rather than
# assumed, because a cancelled run that left a root behind is the failure this
# whole precaution is about.
#
# -- ⚠ ONLY THE COMPARABLE CONNECTIONS ARE COMPARED --------------------------
#
# A browser draws GREASE per connection and shuffles its extension order per
# connection, so two captures of one build differ with no change of
# configuration at all. b_ids_harness::modes measures stability INSIDE each run
# before comparing across runs and reports a field that varies within a run as
# not comparable rather than as a finding.
#
# ⛔ AND IT REFUSES TO REPORT A COMPARISON AT ALL when only one of the two
# routes completed a handshake. A comparison with nothing on one side compared
# nothing, which is a different fact from a comparison that found no difference.
#
# Usage:
#   sh experiments/50-trust-anchor.sh
#   sh experiments/50-trust-anchor.sh --json
#   sh experiments/50-trust-anchor.sh --headless --rounds 2
#
# Exit codes: 0 the comparison ran and the two routes agree,
#             1 it ran and a field differs,
#             2 it could not run, which includes a machine that is not
#               disposable.

set -u

HEADLESS=""
BROWSER=""
ROUNDS=2
JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --headless) HEADLESS="--headless" ;;
    --json) JSON=1 ;;
    --browser) shift; BROWSER="${1:-}" ;;
    --rounds) shift; ROUNDS="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf '50-trust-anchor: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done
case "$ROUNDS" in
  ''|*[!0-9]*) printf '50-trust-anchor: --rounds needs a number\n' >&2; exit 2 ;;
esac
[ "$ROUNDS" -ge 1 ] || { printf '50-trust-anchor: --rounds is at least 1\n' >&2; exit 2; }

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/.." && pwd) || exit 2
cd "$ROOT" || exit 2

command -v cargo >/dev/null 2>&1 || { printf '50-trust-anchor: cargo not found\n' >&2; exit 2; }
command -v timeout >/dev/null 2>&1 || { printf '50-trust-anchor: timeout not found\n' >&2; exit 2; }

# ⛔ THE REFUSAL, AND IT IS THE FIRST THING THIS SCRIPT DOES AFTER PARSING. A
# developer machine must not gain a root because somebody ran an experiment.
DISPOSABLE="${B_IDS_DISPOSABLE:-}"
if [ "$DISPOSABLE" != "1" ]; then
  printf '50-trust-anchor: this machine is not marked disposable, so no root is installed.\n' >&2
  printf '  Set B_IDS_DISPOSABLE=1 only on a machine that is thrown away afterwards.\n' >&2
  printf '  .github/workflows/trust-anchor.yml is where this runs. TODO/harness.md, HARNESS-14.\n' >&2
  if [ "$JSON" = 1 ]; then
    printf '{"schema":"trust-anchor/1","ran":false,"why":"not a disposable machine"}\n'
  fi
  exit 2
fi

OUT="$ROOT/.tmp/50-trust-anchor"
mkdir -p "$OUT" || exit 2

printf 'building\n'
cargo build -q -p b-ids-harness -p b-ids-driver || {
  printf '50-trust-anchor: the workspace did not build\n' >&2
  exit 2
}

BIN="$ROOT/target/debug"
HARNESS="$BIN/b-ids-harness"
DRIVER="$BIN/b-ids-driver"
[ -x "$HARNESS" ] || HARNESS="$HARNESS.exe"
[ -x "$DRIVER" ] || DRIVER="$DRIVER.exe"
for exe in "$HARNESS" "$DRIVER"; do
  [ -x "$exe" ] || { printf '50-trust-anchor: %s is not executable\n' "$exe" >&2; exit 2; }
done

BROWSER_FLAG=""
if [ -n "$BROWSER" ]; then
  BROWSER_FLAG="--browser $BROWSER"
fi

printf '\nresolving a browser\n'
# shellcheck disable=SC2086 # BROWSER_FLAG is a flag and its value, or empty
"$DRIVER" resolve $BROWSER_FLAG --json > "$OUT/resolved.jsonl" 2>"$OUT/resolve.err" || {
  printf '50-trust-anchor: nothing resolved. %s\n' "$(cat "$OUT/resolve.err")" >&2
  exit 2
}
head -1 "$OUT/resolved.jsonl"

# -- installing and removing the root, per platform ---------------------------
#
# ⛔ THE REMOVAL IS READ BACK, never assumed. A cancelled run that left a root
# behind is the failure this precaution is about, and docs/inherited-claims.md
# section 8 records that a cleanup in a `finally` does not survive a hard
# interrupt.
PLATFORM=$(uname -s 2>/dev/null || printf 'unknown')
STORE_NICK="b-ids-trust-anchor"
# ⚠ Which Windows store took the root, set by install_root and read by
# remove_root. Empty everywhere else, and declared here so `set -u` never sees
# it unset. TODO/harness.md, HARNESS-16.
WINDOWS_STORE=""

# ⛔ EVERY STORE COMMAND IS BOUNDED AND ITS STDIN IS CLOSED, and both halves
# of that are paid for. MEASURED 2026-09-02, run 33590621046: the first run of
# this script hung on both platforms and the job was cancelled at its 25-minute
# limit with the measurement already taken and the comparison never printed. A
# certificate tool that asks for a password reads from a terminal that is not
# there and waits forever; </dev/null turns that into an immediate end of file
# and the timeout turns anything else into a bounded failure.
store() {
  timeout 60 "$@" </dev/null
}

install_root() {
  ir_pem=$1
  case "$PLATFORM" in
    Linux*)
      command -v certutil >/dev/null 2>&1 || return 2
      mkdir -p "$HOME/.pki/nssdb" || return 2
      store certutil -d "sql:$HOME/.pki/nssdb" -N --empty-password >/dev/null 2>&1
      store certutil -d "sql:$HOME/.pki/nssdb" -A -t "C,," -n "$STORE_NICK" -i "$ir_pem" \
        >/dev/null 2>&1 || return 2
      ;;
    Darwin*)
      store security add-trusted-cert -d -r trustRoot \
        -k /Library/Keychains/System.keychain "$ir_pem" >/dev/null 2>&1 || return 2
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows*)
      # ⛔ A NATIVE TOOL GETS A NATIVE PATH. MEASURED 2026-09-02, run
      # 33592736694: this ran under Git Bash on the runner and handed
      # certutil.exe an msys path like /d/a/b-ids/..., which it cannot open, so
      # the install failed and the platform went unmeasured while the Linux lane
      # reported a full comparison. docs/conventions/shell.md is the rule.
      ir_native="$ir_pem"
      if command -v cygpath >/dev/null 2>&1; then
        ir_native=$(cygpath -w "$ir_pem")
      fi
      # ⛔ WHAT THE TOOL SAID IS KEPT, and this is HARNESS-16. The call
      # discarded its own output, so run 33594293802 could report only that
      # `certutil -addstore -user Root` returned non-zero and never why. A
      # bounded call whose diagnosis goes to /dev/null is a measurement nobody
      # can act on.
      #
      # ⚠ AND MORE THAN ONE STORE IS TRIED, in the order a browser on Windows is
      # documented to read them, because DRIVER-04 warned that the store a
      # browser reads is not obviously the one certutil writes to. The FIRST one
      # that takes the root is recorded, and which one it was is part of the
      # answer rather than a detail.
      : > "$OUT/store.log"
      WINDOWS_STORE=""
      for ir_store in "-user Root" "-user CA" "Root"; do
        printf '== certutil -addstore %s ==\n' "$ir_store" >> "$OUT/store.log"
        # ⛔ THE STATUS IS CAPTURED FROM THE COMMAND, not read after an `if`.
        # ⚠ Measured 2026-09-02, trust-anchor.yml run 33647065058: the log said
        # `refused, exit 0`, because `$?` after an `if` whose condition failed is
        # the status of the IF STATEMENT rather than of the command inside it.
        # That is the same class as reading an exit code through a pipe, and the
        # number it produced was meaningless. docs/conventions/shell.md section 2.
        # shellcheck disable=SC2086 # ir_store is a flag and its value, deliberately split
        store certutil -addstore $ir_store "$ir_native" >> "$OUT/store.log" 2>&1
        ir_rc=$?
        if [ "$ir_rc" = 0 ]; then
          WINDOWS_STORE="$ir_store"
          printf '== accepted by %s, exit 0 ==\n' "$ir_store" >> "$OUT/store.log"
          break
        fi
        printf '== refused by %s, exit %s ==\n' "$ir_store" "$ir_rc" >> "$OUT/store.log"
      done
      if [ -z "$WINDOWS_STORE" ]; then
        # ⛔ "THIS PLATFORM CANNOT BE PROVISIONED UNATTENDED" IS AN ANSWER, and
        # the entry says to record it as one. What the tool said is in
        # store.log beside this run.
        printf '50-trust-anchor: no Windows store took the root. certutil said:\n' >&2
        cat "$OUT/store.log" >&2
        return 2
      fi
      printf 'store   %s took the root\n' "$WINDOWS_STORE"
      ;;
    *) return 2 ;;
  esac
  return 0
}

remove_root() {
  case "$PLATFORM" in
    Linux*)
      store certutil -d "sql:$HOME/.pki/nssdb" -D -n "$STORE_NICK" >/dev/null 2>&1
      store certutil -d "sql:$HOME/.pki/nssdb" -L 2>/dev/null | grep -q "$STORE_NICK" && return 1
      ;;
    Darwin*)
      store security delete-certificate -c "$STORE_NICK" \
        /Library/Keychains/System.keychain >/dev/null 2>&1
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows*)
      # ⛔ REMOVED FROM THE STORE IT WENT INTO, which is why install_root records
      # which one took it. A teardown that guessed would leave a root behind on
      # a machine, and "anything a session created on another system, that
      # session removes" is not conditional on the guess being right.
      rr_store="${WINDOWS_STORE:--user Root}"
      # shellcheck disable=SC2086 # rr_store is a flag and its value, deliberately split
      store certutil -delstore $rr_store "$STORE_NICK" >/dev/null 2>&1
      # shellcheck disable=SC2086 # the same
      store certutil -store $rr_store 2>/dev/null | grep -q "$STORE_NICK" && return 1
      ;;
  esac
  return 0
}

# One round of one route. ⚠ Both mint their own authority; the only difference
# is whether the browser is given the pin or the root is in its store.
run_route() {
  rr_route=$1
  rr_captures="$OUT/$rr_route.jsonl"
  rr_err="$OUT/$rr_route.err"
  rr_ca="$OUT/$rr_route.ca.pem"
  : > "$rr_captures"
  : > "$rr_err"

  "$HARNESS" --json --ca-out "$rr_ca" --no-resumption \
    --handshakes 4 --run-timeout-ms 40000 --timeout-ms 5000 \
    > "$rr_captures" 2>"$rr_err" &
  rr_pid=$!

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
    printf '50-trust-anchor: %s printed no base URL or pin in 10s\n' "$rr_route" >&2
    kill "$rr_pid" 2>/dev/null
    wait "$rr_pid" 2>/dev/null
    return 2
  fi

  rr_trust=""
  if [ "$rr_route" = "pin" ]; then
    rr_trust="--pin $rr_pin"
  else
    # ⛔ NO FLAG AT ALL. The browser completes the handshake because the root is
    # in the store it reads, or it does not complete one and that IS the finding.
    if ! install_root "$rr_ca"; then
      printf '50-trust-anchor: could not install the root on this platform\n' >&2
      kill "$rr_pid" 2>/dev/null
      wait "$rr_pid" 2>/dev/null
      return 2
    fi
  fi

  # shellcheck disable=SC2086 # each is a flag and its value, or empty
  "$DRIVER" drive --url "$rr_base" --timeout-ms 35000 \
    --log "$OUT/$rr_route.browser.log" $rr_trust $HEADLESS $BROWSER_FLAG \
    > "$OUT/$rr_route.driven.txt" 2>&1

  wait "$rr_pid"
  if [ "$rr_route" != "pin" ]; then
    remove_root || printf '50-trust-anchor: the root is STILL in the store after removal\n' >&2
  fi
  # ⭐ ONE FIXED LINE PER ROUTE. The first run printed nothing between the
  # round header and the comparison, so a job that hung at the end carried a
  # report saying only which round it had reached.
  rr_conn=$(awk 'NR>1 && /^{/' "$rr_captures" | wc -l | tr -d ' ')
  rr_hs=$(grep -c '"termination":{' "$rr_captures" 2>/dev/null)
  rr_h2=$(grep -c '"http2":{' "$rr_captures" 2>/dev/null)
  printf 'route=%s handshakes=%s h2=%s connections=%s\n' \
    "$rr_route" "$rr_hs" "$rr_h2" "$rr_conn"
  tail -n 1 "$rr_err"
  return 0
}

: > "$OUT/pin.all.jsonl"
: > "$OUT/trust-store.all.jsonl"

ROUND=1
while [ "$ROUND" -le "$ROUNDS" ]; do
  printf '\n== round %s of %s ==\n' "$ROUND" "$ROUNDS"
  run_route pin || exit 2
  run_route trust-store || exit 2
  cat "$OUT/pin.jsonl" >> "$OUT/pin.all.jsonl"
  cat "$OUT/trust-store.jsonl" >> "$OUT/trust-store.all.jsonl"
  ROUND=$((ROUND + 1))
done

# ⛔ THE REMOVAL IS COUNTED, not remembered. docs/security/remote-ops.md:
# verify a teardown by counting, and a count that returns to its baseline is
# evidence.
LEFT=0
case "$PLATFORM" in
  Linux*) store certutil -d "sql:$HOME/.pki/nssdb" -L 2>/dev/null | grep -c "$STORE_NICK" > "$OUT/left.txt" || true ;;
  MINGW*|MSYS*|CYGWIN*|Windows*) store certutil -store -user Root 2>/dev/null | grep -c "$STORE_NICK" > "$OUT/left.txt" || true ;;
  *) printf '0\n' > "$OUT/left.txt" ;;
esac
LEFT=$(tr -d ' \n\r' < "$OUT/left.txt")
printf '\nroots left in the store afterwards: %s\n' "${LEFT:-0}"

printf '\ncomparing %s round(s)\n' "$ROUNDS"
cargo run -q -p b-ids-harness --example compare-modes -- --labels pin,trust-store \
  "$OUT/pin.all.jsonl" "$OUT/trust-store.all.jsonl" > "$OUT/comparison.txt" 2>&1
COMPARE_RC=$?
cat "$OUT/comparison.txt"

printf '\nconditions\n'
printf '  host      %s\n' "$(uname -s -r 2>/dev/null || printf 'unknown')"
printf '  browser   %s %s\n' \
  "$(awk -F'"name":"' 'NR==1 { split($2, a, /"/); print a[1] }' "$OUT/resolved.jsonl")" \
  "$(awk -F'"version":"' 'NR==1 { split($2, a, /"/); print a[1] }' "$OUT/resolved.jsonl")"
printf '  rustc     %s\n' "$(rustc --version 2>/dev/null || printf 'unknown')"
printf '  taken     %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  headless  %s\n' "${HEADLESS:-no}"
printf '  rounds    %s, each one pinned run then one trust-store run\n' "$ROUNDS"
printf '  teardown  %s root(s) left in the store\n' "${LEFT:-0}"

if [ "$JSON" = 1 ]; then
  printf '{"schema":"trust-anchor/1","ran":true,"rounds":%s,"roots_left":%s,"compare_exit":%s}\n' \
    "$ROUNDS" "${LEFT:-0}" "$COMPARE_RC"
fi

printf '\nleft in %s\n' "$OUT"

# ⛔ A ROOT LEFT BEHIND IS A FAILURE WHATEVER THE COMPARISON SAID.
if [ "${LEFT:-0}" != "0" ]; then
  printf '50-trust-anchor: %s root(s) remain in the store\n' "$LEFT" >&2
  exit 1
fi
exit "$COMPARE_RC"
