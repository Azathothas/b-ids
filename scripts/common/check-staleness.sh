#!/bin/sh
# check-staleness.sh - is the corpus behind the build the vendor is serving, and
# what would replace it?
#
# ⛔ A BROWSER SHIPPING A NEW VERSION IS NOT A DEFECT IN A COMMIT. Asserting
# current versions on push makes every unrelated change fail on the day a
# browser ships, so this runs on a SCHEDULE and never on a push.
# TODO/ci.md, CI-02.
#
# -- ⭐ WHEN IT GOES RED ITS OUTPUT CARRIES THE REPLACEMENT VALUES -----------
#
# A check that only says a fingerprint changed is half a tool. Every stale row
# names the route that is behind, the build it holds, the build the vendor is
# serving, that build's rollout fraction, and every source that answered. The
# session that picks it up applies a diff rather than redoing the work.
#
# -- ⛔ ONE SOURCE BEING UNREACHABLE IS NOT A FAILURE -------------------------
#
# `b-ids-driver versions` fetches each source separately and reports which
# answered, so a vendor endpoint being down is a row that says so rather than a
# run that died. ⚠ This script reads that report and never fetches anything
# itself: a second fetcher would be a second answer to "what is current".
#
# -- ⚠ A STAGED ROLLOUT IS NOT A CHASE --------------------------------------
#
# The chosen build is the one SERVING, which during a rollout is not the highest
# the vendor knows. Both are reported, with their fractions, because a corpus
# that chased the highest would capture a build most people do not have.
# DRIVER-02 owns that reading.
#
# Usage:
#   sh scripts/common/check-staleness.sh
#   sh scripts/common/check-staleness.sh --json
#   sh scripts/common/check-staleness.sh --versions FILE
#   sh scripts/common/check-staleness.sh --corpus DIR --versions FILE
#
# Exit codes: 0 the corpus holds the serving build, 1 it is behind,
#             2 it could not run.
#
# ⛔ Read the exit code from this process, unpiped. ⚠ 1 is the SIGNAL here
# rather than a defect in this tree, which is why this is not a gate check.

set -u

JSON=0
CORPUS=""
VERSIONS=""
CHANNEL="stable"

while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --corpus) shift; CORPUS="${1:-}" ;;
    --versions) shift; VERSIONS="${1:-}" ;;
    --channel) shift; CHANNEL="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-staleness: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-staleness: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-staleness: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-staleness: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

command -v node >/dev/null 2>&1 || { printf 'check-staleness: node not found\n' >&2; exit 2; }

[ -n "$CORPUS" ] || CORPUS="corpus/v1"
POINTER="$CORPUS/latest.json"
[ -f "$POINTER" ] || {
  printf 'check-staleness: no pointer file at %s\n' "$POINTER" >&2
  exit 2
}

# -- what the vendor is serving ----------------------------------------------
#
# ⛔ READ FROM `b-ids-driver versions`, never fetched here. ⚠ Without
# `--versions` this reaches the NETWORK, which is why this check is not in the
# gate: docs/methodology/gate.md part (a) is offline. `--versions` takes the
# same JSON from a file, which is how the fixture leg runs with no network.
REPORT="$REPO_ROOT/.tmp/check-staleness/report.json"
if [ -n "$VERSIONS" ]; then
  [ -f "$VERSIONS" ] || { printf 'check-staleness: no file at %s\n' "$VERSIONS" >&2; exit 2; }
  REPORT="$VERSIONS"
else
  command -v cargo >/dev/null 2>&1 || { printf 'check-staleness: cargo not found\n' >&2; exit 2; }
  mkdir -p "$(dirname "$REPORT")" || exit 2
  if ! cargo run -q -p b-ids-driver -- versions --channel "$CHANNEL" --json > "$REPORT" 2>/dev/null
  then
    printf 'check-staleness: no source answered, so nothing could be compared\n' >&2
    exit 2
  fi
fi

# ⛔ ONE READER, IN NODE, because the comparison is over JSON and the version
# ordering is numeric per component rather than lexical. `151.0.7922.9` is
# BEHIND `151.0.7922.76` and a string comparison says the opposite.
#
# shellcheck disable=SC2016 # the payload is JavaScript, and the shell must not
# expand anything in it. docs/conventions/shell.md section 1.
node -e '
const fs = require("fs");
const [pointerPath, reportPath, jsonOut, channel] = process.argv.slice(1);
const pointer = JSON.parse(fs.readFileSync(pointerPath, "utf8"));
const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));

const chosen = report.chosen || null;
const answered = (report.answers || [])
  .filter((a) => a.version)
  .map((a) => a.source + "=" + a.version);
const silent = (report.answers || [])
  .filter((a) => !a.version)
  .map((a) => a.source + "=" + (a.error || "no answer"));

if (!chosen || !chosen.version) {
  process.stderr.write("check-staleness: the report names no serving build\n");
  process.exit(2);
}

// ⛔ Numeric per component. A build string is dot-separated numbers and a
// lexical comparison puts 9 after 76.
const order = (a, b) => {
  const x = a.split(".").map(Number);
  const y = b.split(".").map(Number);
  for (let i = 0; i < Math.max(x.length, y.length); i += 1) {
    const l = x[i] || 0;
    const r = y[i] || 0;
    if (l !== r) { return l < r ? -1 : 1; }
  }
  return 0;
};

const rows = [];
for (const [key, path] of Object.entries(pointer.per_channel || {})) {
  const parts = key.split("/");
  const held = (path.match(/([^/]+)\.json$/) || [])[1];
  if (!held) { continue; }
  // ⚠ ONLY THE FAMILY THE REPORT IS ABOUT. `b-ids-driver versions` answers for
  // one vendor, and comparing a Firefox route against a Chrome answer would
  // report every non-Chrome route as behind forever.
  if (parts[0] !== "chrome") { continue; }
  if (parts[1] !== channel) { continue; }
  rows.push({
    route: key,
    held,
    serving: chosen.version,
    behind: order(held, chosen.version) < 0,
  });
}

const stale = rows.filter((r) => r.behind);
if (jsonOut === "1") {
  process.stdout.write(JSON.stringify({
    schema: "check-staleness/1",
    routes: rows.length,
    stale: stale.length,
    serving: chosen.version,
    fraction: chosen.fraction,
    highest_known: chosen.highest_known,
    highest_fraction: chosen.highest_fraction,
    answered: answered.length,
    silent: silent.length,
  }) + "\n");
  // ⛔ THE SAME EXIT CODE AS THE HUMAN FORM. A --json run that reported
  // stale:2 and exited 0 would be the "step that exits 0 having done nothing
  // it was asked to do" row of docs/conventions/forbidden-patterns.md, in the
  // mode a scheduled job reads.
  if (stale.length > 0) { process.exit(1); }
} else if (rows.length === 0) {
  process.stderr.write("check-staleness: the pointer names no route for this vendor and channel\n");
  process.exit(2);
} else if (stale.length === 0) {
  process.stdout.write(
    "staleness ok: " + rows.length + " route(s) hold " + chosen.version +
    ", which is what is serving at fraction " + chosen.fraction + "\n");
} else {
  process.stderr.write(
    "staleness: " + stale.length + " of " + rows.length + " route(s) are behind\n\n");
  for (const row of stale) {
    process.stderr.write("  " + row.route + "\n");
    process.stderr.write("    holds    " + row.held + "\n");
    process.stderr.write("    serving  " + row.serving + " at fraction " + chosen.fraction + "\n");
    process.stderr.write("    highest  " + chosen.highest_known +
      " at fraction " + chosen.highest_fraction + "\n");
  }
  process.stderr.write("\n  sources that answered: " +
    (answered.length ? answered.join(", ") : "none") + "\n");
  if (silent.length) {
    process.stderr.write("  sources that did not:  " + silent.join(", ") + "\n");
  }
  process.stderr.write("\n" +
    "  the replacement is a CAPTURE of " + chosen.version + ", not an edit: the corpus\n" +
    "  is append-only and a correction is a new profile. TODO/ci.md, CI-02.\n");
  process.exit(1);
}
' "$POINTER" "$REPORT" "$JSON" "$CHANNEL"
RC=$?

exit "$RC"
