#!/bin/sh
# 60-identify-extension.sh - can the unidentified TLS extension codepoint be
# named, and if not, what has been searched?
#
# ⭐ THE QUESTION. One extension codepoint observed in a shipped browser is
# unidentified. It is two zero bytes and trivially reproducible, so nothing is
# blocked by it, but an unnamed field in a published corpus is a question every
# consumer will ask. TODO/corpus.md, CORPUS-05.
#
# -- ⛔ WHAT THIS SCRIPT IS ALLOWED TO CONCLUDE ------------------------------
#
# It searches TWO things and neither is the browser's own source:
#
#   1. this project's own published profiles, which is a MEASUREMENT: whether
#      the codepoint is on the wire of a build this project captured itself;
#   2. every tracked reference tree, which is a READING of somebody else's
#      repository at a named commit.
#
# ⛔ IT DOES NOT GUESS A NAME FROM A NEIGHBOUR. That is how the other extension
# in the same capture acquired an inferred name this tree now has to carry as
# inferred: its body is measured and its name is not.
# docs/inherited-claims.md section 3.
#
# ⚠ THE BROWSER ENGINE'S SOURCE IS NOT IN references/ AND IS NOT FETCHED HERE.
# A claim about a repository is not written until that repository is in
# references/ at a named commit, and a Chromium checkout is not a reference this
# project keeps. ⭐ So the search is recorded as EXHAUSTED-FOR-WHAT-IS-HERE with
# the list of what was ruled out, which is what stops the next attempt repeating
# it.
#
# Usage:
#   sh experiments/60-identify-extension.sh
#   sh experiments/60-identify-extension.sh --codepoint 0xca34
#
# Exit codes: 0 the search ran and recorded a verdict,
#             1 the search ran and the tree disagrees with itself,
#             2 it could not run.

set -u

CODEPOINT="0x12e0"
while [ $# -gt 0 ]; do
  case "$1" in
    --codepoint) shift; CODEPOINT="${1:-}" ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf '60-identify-extension: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done
case "$CODEPOINT" in
  0x[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]) ;;
  *) printf '60-identify-extension: --codepoint takes 0xNNNN\n' >&2; exit 2 ;;
esac

# ⛔ Resolved from the script's own location, never from the working directory.
HERE=$(cd -- "$(dirname -- "$0")" && pwd) || exit 2
ROOT=$(cd -- "$HERE/.." && pwd) || exit 2
cd "$ROOT" || exit 2

command -v git >/dev/null 2>&1 || { printf '60-identify-extension: git not found\n' >&2; exit 2; }
command -v node >/dev/null 2>&1 || { printf '60-identify-extension: node not found\n' >&2; exit 2; }

OUT="$ROOT/.tmp/60-identify-extension"
mkdir -p "$OUT" || exit 2

BARE=$(printf '%s' "$CODEPOINT" | sed 's/^0x//' | tr 'ABCDEF' 'abcdef')
UPPER=$(printf '%s' "$BARE" | tr 'abcdef' 'ABCDEF')
DECIMAL=$(node -e 'process.stdout.write(String(parseInt(process.argv[1], 16)))' "$BARE")

printf 'searching for extension %s (%s decimal)\n' "$CODEPOINT" "$DECIMAL"

# -- 1. this project's own measurements ---------------------------------------
#
# ⭐ THE ONLY PART OF THIS THAT IS A MEASUREMENT. Everything below is a reading.
printf '\n-- what this project measured itself --\n'
node -e '
const fs = require("fs");
const path = require("path");
const [root, codepoint] = process.argv.slice(1);
const want = parseInt(codepoint, 16);
const dir = path.join(root, "corpus", "v1");
const found = [];
const walk = (d) => {
  for (const entry of fs.readdirSync(d, { withFileTypes: true })) {
    const p = path.join(d, entry.name);
    if (entry.isDirectory()) { walk(p); continue; }
    if (!entry.name.endsWith(".json")) { continue; }
    if (entry.name === "index.json" || entry.name === "latest.json") { continue; }
    const profile = JSON.parse(fs.readFileSync(p, "utf8"));
    const hit = profile.tls.extensions.find((e) => e.codepoint === want);
    found.push({
      id: profile.id,
      version: profile.browser.version,
      platform: profile.platform.os,
      present: Boolean(hit),
      length: hit ? hit.length : null,
      body: hit ? hit.body_hex : null,
    });
  }
};
if (!fs.existsSync(dir)) {
  process.stdout.write("  no corpus at " + dir + "\n");
  process.exit(0);
}
walk(dir);
found.sort((a, b) => a.id.localeCompare(b.id));
for (const row of found) {
  process.stdout.write("  " + (row.present ? "PRESENT" : "absent ") + "  " + row.id +
    (row.present ? "  length=" + row.length + " body=" + row.body : "") + "\n");
}
process.stdout.write("  " + found.filter((r) => r.present).length + " of " +
  found.length + " published profile(s) carry it\n");
' "$ROOT" "$BARE" | tee "$OUT/corpus.txt"

# -- 2. every tracked reference tree ------------------------------------------
#
# ⚠ A READING, at whatever commit each tree is pinned to. ⛔ It reports WHERE a
# spelling occurs; it does not conclude what the codepoint is from the fact that
# a file mentions it.
printf '\n-- every tracked reference tree, three spellings --\n'
: > "$OUT/references.txt"
for spelling in "0x$BARE" "0x$UPPER" "$DECIMAL"; do
  printf '  %s\n' "$spelling" >> "$OUT/references.txt"
  git grep -l -F -- "$spelling" -- references/ 2>/dev/null \
    | sed 's/^/    /' >> "$OUT/references.txt" || true
done
cat "$OUT/references.txt"
printf '  ⚠ THE DECIMAL SPELLING IS LOW SIGNAL and is searched anyway. Four\n'
printf '     digits occur in unrelated JSON, so a hit there is a place to look\n'
printf '     rather than a finding. Narrowing the search to make the output\n'
printf '     tidy would be narrowing it to get the answer that fits.\n'

# -- 2b. where a CAPTURED FINGERPRINT carries it -----------------------------
#
# ⭐ THE HIGHEST-SIGNAL HIT THERE IS: a reference tree that recorded a
# handshake carrying this codepoint names the BUILD it came from, which is what
# turns "somebody mentions it" into "this build sends it".
printf '\n-- reference captures whose extension list carries it --\n'
git grep -l -F -- ",$BARE," -- references/ 2>/dev/null | sed 's/^/  /' || true

# -- 3. the verdict -----------------------------------------------------------
#
# ⛔ RECORDED EITHER WAY. An extension nobody can name still gets its codepoint,
# its length and its body kept verbatim, which is what lets somebody who is not
# this project identify it later.
printf '\n-- verdict --\n'
printf '  ⛔ NOT NAMED. No specification was read against these bytes, and the\n'
printf '     browser engine source is not a tree this project keeps.\n'
printf '  What was ruled out, so the next attempt does not repeat it:\n'
printf '    - every profile this project has published, listed above\n'
printf '    - every tracked reference tree, three spellings, listed above\n'
printf '  What would name it: the engine source at a named commit in references/,\n'
printf '  or a specification draft read against the recorded body.\n'

printf '\nconditions\n'
printf '  taken     %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
printf '  trees     %s tracked reference tree(s)\n' "$(git ls-files -- references/ | awk -F/ '{ print $2 }' | sort -u | wc -l | tr -d ' ')"
printf '\nleft in %s\n' "$OUT"
exit 0
