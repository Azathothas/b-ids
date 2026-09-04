#!/bin/sh
# check-signing.sh - can a consumer tell a capture from an assertion without
# trusting a file that travelled with the artefact?
#
# ⛔ A CHECKSUMS FILE PUBLISHED IN THE SAME RELEASE AS THE ARTEFACT PROVES
# TRANSPORT, NOT AUTHORSHIP, because whoever could replace one could replace the
# other. TODO/publish.md, PUB-09.
#
# ⭐ THE ANSWER IS KEYLESS, RULED BY THE OPERATOR 2026-09-04. The runner's own
# OIDC identity signs, so no long-lived key exists, nothing is rotated, and no
# workflow names a secret. That preserves the property TODO/RULES.md states
# about this tree: nothing in it needs a credential.
#
# -- ⛔ WHAT IT ASSERTS -------------------------------------------------------
#
#   1. the release job declares BOTH writes keyless attestation needs, and
#      declares them on the JOB rather than at the top of the file;
#   2. ⛔ NO KEY AND NO SECRET. The workflow names no `secrets.`, and the tree
#      carries no private key of any kind under any of the names one takes;
#   3. the attestation action is pinned to a COMMIT, not a tag;
#   4. ⛔ IT ATTESTS BEFORE IT RELEASES, by line order in the file. A release
#      that existed with no attestation beside it is a window in which a
#      consumer verifies nothing and is told nothing is wrong;
#   5. it attests the ARCHIVE and not only the manifest. Attesting a list and
#      not the thing the list describes is verifying the wrong object;
#   6. ⭐ THE PUBLISHED VERIFICATION COMMAND IS THE ONE THIS CHECK NAMES, so a
#      consumer reading the documents runs what this asserts about.
#
# ⚠ THE LIVE LEG IS A SKIP AND SAYS WHY. Verifying a real attestation needs a
# release to verify, and a pushed tag is the only thing that cuts one, which is
# the operator's own act. ⛔ A skip is reported as a skip: nothing here claims an
# attestation was verified when none exists.
#
# Usage:
#   sh scripts/common/check-signing.sh
#   sh scripts/common/check-signing.sh --json
#
# Exit codes: 0 the surface is what PUB-09 asks for, 1 it is not, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped.

set -u

JSON=0
while [ $# -gt 0 ]; do
  case "$1" in
    --json) JSON=1 ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-signing: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-signing: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || {
  printf 'check-signing: not a git repository\n' >&2
  exit 2
}
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT" || { printf 'check-signing: cannot enter %s\n' "$REPO_ROOT" >&2; exit 2; }

WF=".github/workflows/publish.yml"
[ -f "$WF" ] || { printf 'check-signing: no %s\n' "$WF" >&2; exit 2; }

# ⭐ THE ONE PLACE THE VERIFICATION COMMAND IS WRITTEN, and every document that
# publishes it is compared against this. A consumer running something this check
# does not assert about is a consumer running an unchecked claim.
VERIFY_COMMAND='gh attestation verify'

PROBLEMS=""
COUNT=0
note() {
  PROBLEMS="$PROBLEMS  $1
"
  COUNT=$((COUNT + 1))
}

# -- 1: the two writes, on the job ------------------------------------------
#
# ⚠ READ FROM THE RELEASE JOB'S OWN BLOCK, not from the file. A grep over the
# whole file would pass on a permission granted at the top, which is the thing
# job-scoping exists to prevent.
#
# ⛔ AND WITH THE COMMENTS STRIPPED, WHICH THE MUTATION PASS FOUND. `id-token:
# write` is REMOVED from the release job, both halves of this check reported
# `signing ok`. The job's own comment explains what `id-token: write` is for and
# spells it exactly, so the search matched the PROSE and never looked at the
# declaration. ⚠ It is the same defect this check had already caught in the
# workflow's line ordering, in the check itself, one leg later. ⭐ That is what a
# guard mutation is for: this leg had never been seen to refuse.
JOB=$(awk '/^  release:/ { inside = 1 } inside && /^  [a-z-]+:/ && !/^  release:/ { inside = 0 } inside { print }' "$WF" | grep -v '^ *#')
for want in 'id-token: write' 'attestations: write'; do
  printf '%s\n' "$JOB" | grep -qF "$want" ||
    note "the release job does not declare $want, and keyless attestation needs it"
done
TOP=$(awk '/^permissions:[ \t]*$/ { inside = 1; next } inside && /^[a-zA-Z]/ { inside = 0 } inside { print }' "$WF" | grep -v '^ *#')
for forbidden in 'id-token' 'attestations'; do
  printf '%s\n' "$TOP" | grep -q "$forbidden" &&
    note "$forbidden is granted at the top of the file, and it belongs to one job"
done

# -- 2: no key, no secret ----------------------------------------------------
#
# ⛔ THE WHOLE POINT OF THE RULING. A workflow that named a secret would mean a
# key exists somewhere, and the record says nothing in this tree needs one.
grep -q 'secrets\.' "$WF" && note "$WF names a secret, and keyless attestation needs none"
KEYS=$(git ls-files | grep -Ei '\.(pem|key|p12|pfx|jks|gpg|asc)$|(^|/)id_(rsa|ed25519|ecdsa)$' || true)
[ -z "$KEYS" ] || note "the tree carries key-shaped file(s): $(printf '%s' "$KEYS" | tr '\n' ' ')"

# -- 3: pinned to a commit ---------------------------------------------------
PIN=$(grep -o 'attest-build-provenance@[0-9a-f]*' "$WF" | head -1 | cut -d@ -f2)
case "${#PIN}" in
  40) ;;
  *) note "the attestation action is not pinned to a 40-character commit: ${PIN:-none}" ;;
esac

# -- 4 and 5: it attests, before it releases, and over the archive -----------
#
# ⛔ BY LINE ORDER IN THE FILE, which is the order the runner executes them in.
#
# ⚠ COMMENT LINES ARE EXCLUDED, AND THIS CHECK FOUND THAT ITSELF ON ITS FIRST
# RUN. `publish.yml` explains the release step in a comment thirty lines above
# the step, so a grep for the command matched the PROSE and reported that the
# release happens before the attestation. ⛔ A check whose model of a file is
# "any line mentioning the thing" reads documentation as behaviour, which is the
# same class as a guard that has never been seen to refuse.
uncommented() { grep -n "$1" "$WF" | grep -v '^[0-9]*: *#' | head -1 | cut -d: -f1; }
ATTEST_AT=$(uncommented 'uses: actions/attest-build-provenance@')
RELEASE_AT=$(uncommented 'gh release create')
if [ -z "${ATTEST_AT:-}" ]; then
  note "nothing in $WF attests anything"
elif [ -z "${RELEASE_AT:-}" ]; then
  note "nothing in $WF creates a release, so the ordering could not be checked"
elif [ "$ATTEST_AT" -ge "$RELEASE_AT" ]; then
  note "the attestation is at line $ATTEST_AT and the release is created at line $RELEASE_AT, so a release would exist before anything attested it"
fi
SUBJECTS=$(awk '/uses: actions\/attest-build-provenance@/ { inside = 1 } inside && /^      - name:/ && !/attest/ { inside = 0 } inside { print }' "$WF")
printf '%s\n' "$SUBJECTS" | grep -q 'tar\.gz' ||
  note "the attestation does not name the release archive, so a consumer could verify the list and not the thing it describes"
printf '%s\n' "$SUBJECTS" | grep -q 'SHA256SUMS' ||
  note "the attestation does not name SHA256SUMS"

# -- 6: the published command is this one ------------------------------------
#
# ⭐ A CONSUMER RUNS WHAT THE DOCUMENTS SAY, so what the documents say is what
# this check asserts about. A command published nowhere is an instruction nobody
# can follow; one published in two spellings is two instructions.
PUBLISHED=$(grep -rl "$VERIFY_COMMAND" README.md docs/ TODO/ 2>/dev/null || true)
[ -n "$PUBLISHED" ] ||
  note "no document publishes '$VERIFY_COMMAND', so a consumer is not told how to verify"

# ⚠ THE LIVE LEG. Verifying needs something to verify, and a pushed tag is the
# only thing that cuts a release.
VERIFIED=skipped
RELEASES=0
if command -v gh >/dev/null 2>&1; then
  RELEASES=$(gh release list --limit 1 --json tagName --jq 'length' 2>/dev/null || echo 0)
  case "$RELEASES" in
    ''|*[!0-9]*) RELEASES=0 ;;
  esac
fi

if [ "$JSON" = 1 ]; then
  printf '{"schema":"check-signing/1","pinned":"%s","releases":%s,"verified":"%s","problems":%s}\n' \
    "${PIN:-none}" "$RELEASES" "$VERIFIED" "$COUNT"
  [ "$COUNT" = 0 ] || exit 1
  exit 0
fi

if [ "$COUNT" = 0 ]; then
  printf 'signing ok: the release job signs with the runner'"'"'s own identity, over the\n'
  printf '  archive and the two files a consumer fetches, before the release exists.\n'
  printf '  ⛔ No key, no secret, and the action is pinned to %s.\n' "$PIN"
  printf '  ⚠ SKIP the live leg: %s release(s) exist, and verifying needs one. A\n' "$RELEASES"
  printf '  pushed tag is the only thing that cuts a release, and that is the\n'
  printf '  operator'"'"'s own act. Nothing here says an attestation was verified.\n'
  exit 0
fi

printf 'signing check failed, %s problem(s):\n\n' "$COUNT" >&2
printf '%s\n' "$PROBLEMS" >&2
printf 'A checksums file published beside the artefact proves transport rather\n' >&2
printf 'than authorship. TODO/publish.md, PUB-09.\n' >&2
exit 1
