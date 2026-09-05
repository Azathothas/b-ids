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
#   3. writes the identity the capture was taken under, including where the
#      build came from when this project fetched it;
#   4. prints the conditions the measurement was taken under, and PRINTS THE
#      COMMAND that writes the profile.
#
# ⛔ IT DOES NOT WRITE INTO THE CORPUS, and this header said it did until
# 2026-09-02. The corpus is append-only and a profile in it is permanent, so
# the write is a deliberate act by whoever read the conditions above it, not a
# side effect of taking a measurement. `b-ids-corpus add` is the command, and
# the last thing this script prints is that command with its arguments filled
# in. Found by reading the artefacts of capture.yml run 33615327503, where two
# lanes reported success and neither had added anything.
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
# ⭐ THE HARNESS OFFERS SESSION TICKETS, so the subject resumes when it can and
# behaves as it does in the wild. --no-resumption is NOT passed here, and it has
# not been since HARNESS-15: it stays as a CONTROL for
# experiments/30-resumption-control.sh and it stopped being a condition every
# published profile is taken under.
#
# ⚠ WHY IT WAS A CONDITION, measured on hosted runners twice, capture.yml runs
# 33579619515 and 33580371329: with tickets offered, Chrome on ubuntu-latest
# abandoned both of the connections that were not resumed and resumed every one
# it kept, so no single connection carried both a cold hello and HTTP/2. The
# selection required one connection to carry both, so the navigation published
# nothing and the switch was the way round it.
#
# ⭐ THE TWO HALVES ARE SELECTED INDEPENDENTLY NOW. The cold hello on a
# connection that reached no HTTP/2 is the one that is read, the frames come
# from the first connection that did, and the profile records which connection
# each half came from. The resumption configuration is still recorded in
# `captured.resumption`, read back from what the harness reported.
#
# ⚠ HEADER VALUES ARE RECORDED, DELIBERATELY. The default capture shape is
# names only, because a model whose natural form carries values is a model that
# will one day publish a credential. A corpus profile is the case the default
# is deliberately turned off for: four of the validator's checks read a header
# VALUE, and a published profile that none of them can check is a profile
# nobody can validate. `cookie` and `authorization` keep their name and lose
# their value either way, at
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
#   sh experiments/10-first-profile.sh --browser edge
#
# Exit codes: 0 the measurement ran and the evidence is on disk,
#             1 it ran and something refused,
#             2 it could not run.

set -u

HEADLESS=""
# ⛔ EMPTY MEANS "the first family that resolved", never "chrome". A default
# spelled here would be a second place the resolver's order is decided.
BROWSER=""
while [ $# -gt 0 ]; do
  case "$1" in
    --headless) HEADLESS="--headless" ;;
    --browser) shift; BROWSER="${1:-}" ;;
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

# ⛔ PASSED THROUGH, never interpreted here. The driver owns which families
# exist and refuses a name it has no branch for; a second list in this script
# would be a value in two places with no check that they agree.
BROWSER_FLAG=""
if [ -n "$BROWSER" ]; then
  BROWSER_FLAG="--browser $BROWSER"
fi

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
# shellcheck disable=SC2086 # BROWSER_FLAG is a flag and its value, or empty
"$DRIVER" resolve $BROWSER_FLAG --json > "$OUT/resolved.jsonl" 2>"$OUT/resolve.err" || {
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

# -- which trust route this engine has, asked of the driver ------------------
# ⭐ ASKED, NEVER MAPPED HERE. Chromium takes the authority as a key pin on the
# command line and Gecko takes no certificate switch at all, so the trust goes
# into the certificate database of the profile the launch creates. A case
# statement here keyed on a family name would be a second family list, which is
# the defect DRIVER-10 records. docs/history/todo/driver.md, DRIVER-11.
# shellcheck disable=SC2086 # BROWSER_FLAG is a flag and its value, or empty
TRUST_ROUTE=$("$DRIVER" trust-route $BROWSER_FLAG 2>"$OUT/trust-route.err")
case "$TRUST_ROUTE" in
  switch) TRUST_FLAGS="--pin $PIN" ;;
  profile-database) TRUST_FLAGS="--ca-file $OUT/ca.pem" ;;
  *)
    printf '10-first-profile: the driver named no trust route: %s\n' \
      "$(cat "$OUT/trust-route.err")" >&2
    kill "$HARNESS_PID" 2>/dev/null
    wait "$HARNESS_PID" 2>/dev/null
    exit 2
    ;;
esac
printf 'trust route: %s\n' "$TRUST_ROUTE"

# -- the browser, in the foreground, which is the hold -----------------------
printf '\nlaunching the browser at it\n'
# shellcheck disable=SC2086 # each is a flag and its value, or empty
"$DRIVER" drive --url "$BASE" $TRUST_FLAGS --timeout-ms 45000 \
  --log "$OUT/browser.log" $HEADLESS $BROWSER_FLAG \
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
  # ⛔ WHAT THE BROWSER SAID, on the one path where it is the whole
  # diagnosis. An `edge` lane on a hosted runner exited after 1.4 seconds
  # having opened nothing, and its own output had been discarded.
  printf '10-first-profile: what the browser wrote:\n' >&2
  tail -n 40 "$OUT/browser.log" >&2 || printf '  (nothing)\n' >&2
  exit 1
fi

# -- the switches the browser was actually given -----------------------------
# ⛔ Read back from what the driver reported, never retyped here. Every one of
# them is a condition of what was captured through it.
awk 'NR>1 && /^  --/ { sub(/^  /, ""); print }' "$OUT/driven.txt" > "$OUT/switches.txt"

# -- the resumption configuration the HARNESS reported ----------------------
# ⛔ Read back from the harness rather than from what was asked for. The
# switch is a CONDITION of the capture, and a run in which it was refused
# would otherwise be recorded under a condition it did not have.
RESUMPTION=$(awk '/^resumption: /{ sub(/^resumption: /, ""); print; exit }' "$HARNESS_ERR")
if [ -z "$RESUMPTION" ]; then
  printf '10-first-profile: the harness reported no resumption line\n' >&2
  exit 1
fi

printf '\nconditions\n'
printf '  host      %s\n' "$(uname -s -r 2>/dev/null || printf 'unknown')"
printf '  rustc     %s\n' "$(rustc --version 2>/dev/null || printf 'unknown')"
printf '  taken     %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  trust     %s, read from what the driver reported\n' "$(awk 'NR==1 { for (i = 1; i <= NF; i++) if ($i ~ /^trust=/) { sub(/^trust=/, "", $i); print $i } }' "$OUT/driven.txt")"
printf '  resumption %s, read from what the harness reported\n' "$RESUMPTION"
printf '  headless  %s\n' "${HEADLESS:-no}"
printf '  switches  %s\n' "$(wc -l < "$OUT/switches.txt" | tr -d ' ')"

# -- the identity, written from what the run did rather than typed ------------
#
# ⛔ THE DOOR THIS CLOSES. `captured.trust` and `captured.switches` are the
# conditions of the measurement, and `HARNESS-10`'s comparison across profiles
# reads the first of them. A hand-written identity file is a field somebody
# TYPES, so a profile could claim a trust store while the run used a pin and
# nothing in the bytes could contradict it. Found by the door sweep at the end
# of the session that added the field.
#
# ⚠ What is still typed is the channel, whether the build is branded, and the
# operator. Those are labels on the subject rather than readings off this run,
# and `DRIVER-06` is the entry that measures the second of them. ⭐ The NAME
# stopped being typed on 2026-09-02: the driver reports it, because the corpus
# derives a route by lower-casing it and an `edge` lane that wrote `Chrome`
# would publish one browser under another one's route.
# ⭐ WHERE THE BUILD CAME FROM, when this project fetched it. Written by
# scripts/common/provision-browser.sh into its own scratch directory, and read
# here rather than retyped. ⚠ ABSENT IS THE NORMAL CASE AND IT IS NOT AN ERROR:
# a build already installed on the machine was not obtained by this project and
# has no route or digest to record, which is a different fact from an
# acquisition that failed. docs/history/todo/driver.md, DRIVER-08.
ACQUISITION="$ROOT/.tmp/provision-browser/acquisition.json"
[ -f "$ACQUISITION" ] || ACQUISITION=""
printf '  acquisition %s\n' "${ACQUISITION:-none, this build was already on the machine}"

IDENTITY="$OUT/identity.json"
node -e '
const fs = require("fs");
const [resolvedPath, switchesPath, out, headless, resumption, acquisitionPath,
  drivenPath] = process.argv.slice(1);
const resolved = fs.readFileSync(resolvedPath, "utf8").split(/\r?\n/)
  .filter(Boolean).map((l) => JSON.parse(l));
// ⛔ THE FIRST LINE, whatever family it is. The driver was given the same
// --browser flag, so this file holds the family this run actually drove; a
// find() for a hardcoded family here would label an Edge capture Chrome.
const subject = resolved[0];
if (!subject) { throw new Error("nothing resolved"); }
fs.writeFileSync(out, JSON.stringify({
  // ⛔ Read from what the driver reported. The corpus derives a route by
  // lower-casing this, so a name typed here would be a second copy of a value
  // b_ids_driver::Family::vendor_name already owns.
  name: subject.name,
  version: subject.version,
  channel: "stable",
  branded: true,
  os: process.platform === "win32" ? "windows" : "linux",
  arch: "x86_64",
  distribution: null,
  method: "host",
  harness: "b-ids-harness 0.0.0",
  operator: "",
  // ⛔ READ FROM WHAT THE DRIVER REPORTED, never inferred from the switch
  // list. Inferring it held only while every engine took its trust on the
  // command line: a Gecko launch passes no certificate switch at all, so the
  // same rule read not-applicable over a completed handshake, which is a
  // combination the schema refuses. The driver names the configuration it
  // actually used. docs/history/todo/driver.md, DRIVER-11.
  trust: (() => {
    const m = fs.readFileSync(drivenPath, "utf8").match(/(?:^|[ ])trust=([^ \r\n]+)/);
    if (!m) { throw new Error("the driver reported no trust configuration"); }
    return m[1];
  })(),
  switches: fs.readFileSync(switchesPath, "utf8").split(/\r?\n/).filter(Boolean),
  // ⛔ Read from what the HARNESS reported, never from what this script
  // asked for. A run whose switch was refused would otherwise record a
  // condition it did not have, and a cold hello looks the same either way,
  // so nothing in the bytes could contradict it.
  resumption: resumption || null,
  headless: headless === "--headless",
  // ⛔ READ FROM WHAT THE PROVISIONING TOOL WROTE, never composed here. The
  // route, the URL, the digest and the byte count are facts about a fetch this
  // script did not perform, and a value typed here would be a claim.
  // ⚠ null where nothing was fetched, which the schema serialises as absent.
  acquisition: acquisitionPath
    ? JSON.parse(fs.readFileSync(acquisitionPath, "utf8"))
    : null,
}, null, 2) + "\n");
' "$OUT/resolved.jsonl" "$OUT/switches.txt" "$IDENTITY" "${HEADLESS:-}" "$RESUMPTION" \
  "$ACQUISITION" "$OUT/driven.txt" || {
  printf '10-first-profile: could not write the identity file\n' >&2
  exit 1
}
printf '  identity  trust=%s\n' "$(awk -F'"' '/"trust"/ { print $4 }' "$IDENTITY")"

printf '\nleft in %s\n' "$OUT"
printf '  captures.jsonl  every connection the navigation opened\n'
printf '  driven.txt      what the driver reported, switches included\n'
printf '  browser.log     what the BROWSER itself wrote, stdout and stderr\n'
printf '  harness.err     the pin, and the sampling shortfall if there was one\n'
printf '  identity.json   ⚠ the operator fills in the channel, branded and the\n'
printf '                  operator; everything else is read from this run\n'
# ⚠ THE ROOT IS NAMED RATHER THAN ASSUMED TO BE HERE. corpus/ left the default
# branch in PUB-13, so a person pasting this line in a fresh checkout would be
# adding a profile to a directory that is not there. ⛔ The resolver materialises
# the source branch under .tmp, and a write into THAT copy is not a publish: the
# published corpus is a branch now, so adding a profile means committing it there.
printf '\nWrite the profile with, from a checkout that carries the corpus:\n'
# shellcheck disable=SC2016 # the substitution is PRINTED for a person to run,
# not expanded here. docs/conventions/shell.md section 1.
printf '  %s add --captures %s --identity %s --root "$(sh scripts/common/corpus-root.sh)"\n' \
  "$CORPUS" "$CAPTURES" "$IDENTITY"
exit 0
