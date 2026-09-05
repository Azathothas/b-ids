#!/bin/sh
# check-sources.sh - does every external question get asked more than one way,
# and is a disagreement reported rather than resolved?
#
# ⛔ EVERY EXTERNAL DEPENDENCY WILL ONE DAY ANSWER DIFFERENTLY, and a corpus
# that stopped updating in year two was not worth building. docs/history/todo/ci.md, CI-06.
#
# -- ⭐ TWO SOURCES THAT DISAGREE ARE THE MOST VALUABLE SIGNAL HERE ----------
#
# One instance is already measured: two first-party version sources disagreed
# and the disagreement WAS the defect. docs/inherited-claims.md section 7.
# ⛔ So a disagreement is published rather than picked between, and this check
# asserts that the report does exactly that.
#
# -- ⛔ THE THREE THINGS IT ASSERTS ------------------------------------------
#
#   1. PER-SOURCE ISOLATION. Every source appears in the report with its own
#      answer or its own error. A run that dropped a source that failed would
#      report a smaller sample than it took.
#   2. A SILENT SOURCE IS NOT A FAILURE. A report where one source answered and
#      one did not still names a serving build, so a vendor endpoint being down
#      degrades the run rather than ending it.
#   3. A DISAGREEMENT IS FLAGGED, NEVER RESOLVED SILENTLY. Two sources with two
#      answers carry both, and `disagreement` says so.
#
# ⚠ IT NEVER FETCHES ANYTHING. `b-ids-driver versions` asks each source
# separately and this reads what it reported; a second fetcher would be a second
# answer to "what is current". --report takes the same JSON from a file, which
# is how the fixture legs run with no network.
#
# ⛔ WHAT IT DOES NOT DO IS DECIDE WHICH SOURCE IS RIGHT. That is a reading, and
# a check that picked would be the "silently prefer one source" this entry
# forbids in as many words.
#
# Usage:
#   sh scripts/common/check-sources.sh
#   sh scripts/common/check-sources.sh --json
#   sh scripts/common/check-sources.sh --report FILE
#
# Exit codes: 0 the contract holds, 1 it does not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
REPORT=""
CHANNEL="stable"
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    --report) shift; REPORT="${1:-}" ;;
    --channel) shift; CHANNEL="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-sources: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-sources: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-sources: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-sources: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }
command -v node >/dev/null 2>&1 || { printf 'check-sources: node not found\n' >&2; exit 2; }

if [ -z "$REPORT" ]; then
  command -v cargo >/dev/null 2>&1 || { printf 'check-sources: cargo not found\n' >&2; exit 2; }
  mkdir -p "$REPO_ROOT/.tmp/check-sources" || exit 2
  REPORT="$REPO_ROOT/.tmp/check-sources/report.json"
  if ! cargo run -q -p b-ids-driver -- versions --channel "$CHANNEL" --json > "$REPORT" 2>/dev/null
  then
    printf 'check-sources: no source answered at all, so nothing could be checked\n' >&2
    exit 2
  fi
fi
[ -f "$REPORT" ] || { printf 'check-sources: no file at %s\n' "$REPORT" >&2; exit 2; }

# ⛔ THE FIXTURE LEG RUNS ON EVERY INVOCATION, so this check has been seen to
# refuse. Two fixtures, one per clause: a report with a source that answered
# nothing and no error at all, and a report carrying two answers with the
# disagreement flag off.
FIXTURES="$REPO_ROOT/.tmp/check-sources"
mkdir -p "$FIXTURES" || exit 2
node -e '
const fs = require("fs");
const [dir] = process.argv.slice(1);
fs.writeFileSync(dir + "/silent-without-a-reason.json", JSON.stringify({
  answers: [{ source: "releases", version: "9.0.0.1", error: null },
            { source: "chrome-for-testing", version: null, error: null }],
  chosen: { version: "9.0.0.1", fraction: 1, highest_known: "9.0.0.1", highest_fraction: 1 },
  disagreement: false,
}) + "\n");
fs.writeFileSync(dir + "/disagreement-unflagged.json", JSON.stringify({
  answers: [{ source: "releases", version: "9.0.0.1", error: null },
            { source: "chrome-for-testing", version: "9.0.0.2", error: null }],
  chosen: { version: "9.0.0.1", fraction: 1, highest_known: "9.0.0.2", highest_fraction: 1 },
  disagreement: false,
}) + "\n");
' "$FIXTURES"

# ⛔ ONE READER, in node, because the contract is over JSON.
node -e '
const fs = require("fs");
const [reportPath, jsonOut, fixtures] = process.argv.slice(1);

const inspect = (path) => {
  const report = JSON.parse(fs.readFileSync(path, "utf8"));
  const answers = report.answers || [];
  const problems = [];

  if (answers.length < 2) {
    problems.push("only " + answers.length +
      " source(s) in the report, and one source is a single point of failure");
  }
  // ⛔ PER-SOURCE ISOLATION. A source carries an answer or a reason, never
  // neither: a source that vanished from the report is a sample nobody counted.
  for (const answer of answers) {
    if (!answer.source) { problems.push("an answer with no source name"); continue; }
    if (!answer.version && !answer.error) {
      problems.push(answer.source + " reported neither a version nor a reason");
    }
  }
  // ⛔ A DISAGREEMENT IS FLAGGED. Two different versions with the flag off is
  // one source silently preferred, which this entry forbids by name.
  const versions = [...new Set(answers.filter((a) => a.version).map((a) => a.version))];
  if (versions.length > 1 && !report.disagreement) {
    problems.push("sources answered " + versions.join(" and ") +
      " and disagreement is false, which is one source silently preferred");
  }
  if (versions.length <= 1 && report.disagreement) {
    problems.push("disagreement is true and the sources answered " +
      (versions.length ? versions[0] : "nothing"));
  }
  const answered = answers.filter((a) => a.version).length;
  return { answers: answers.length, answered, silent: answers.length - answered,
           disagreement: Boolean(report.disagreement), problems };
};

// ⭐ THE FIXTURE LEG FIRST. A check that cannot refuse must not report a pass.
// ⛔ The fixture directory is PASSED IN, never derived from the report path:
// --report points wherever a caller says, and deriving it there read the
// fixtures out of a directory that has none.
const dir = fixtures + "/";
for (const [name, expect] of [["silent-without-a-reason", "neither a version nor a reason"],
                              ["disagreement-unflagged", "silently preferred"]]) {
  const seen = inspect(dir + name + ".json");
  if (!seen.problems.some((p) => p.includes(expect))) {
    process.stderr.write("check-sources: the " + name +
      " fixture was read as clean, so this check cannot refuse\n");
    process.exit(2);
  }
}

const real = inspect(reportPath);
if (jsonOut === "1") {
  process.stdout.write(JSON.stringify({
    schema: "check-sources/1",
    sources: real.answers,
    answered: real.answered,
    silent: real.silent,
    disagreement: real.disagreement,
    problems: real.problems.length,
  }) + "\n");
} else if (real.problems.length === 0) {
  process.stdout.write("sources ok: " + real.answers + " source(s), " + real.answered +
    " answered, " + real.silent + " did not, disagreement=" + real.disagreement + "\n");
} else {
  process.stderr.write("source contract failed, " + real.problems.length + " problem(s):\n\n");
  for (const problem of real.problems) { process.stderr.write("  " + problem + "\n"); }
  process.stderr.write("\n  Two sources that disagree are the most valuable signal this\n");
  process.stderr.write("  project produces. Record both, publish both, never pick.\n");
  process.stderr.write("  docs/history/todo/ci.md, CI-06.\n");
}
if (real.problems.length > 0) { process.exit(1); }
' "$REPORT" "$JSON" "$FIXTURES"
exit $?
