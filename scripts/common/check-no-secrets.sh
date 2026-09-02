#!/bin/sh
# check-no-secrets.sh - does any file in this tree carry something that must
# not be published?
#
# ⚠ THE SCOPE IS TRACKED PLUS UNTRACKED-BUT-NOT-IGNORED, not tracked alone.
# `git ls-files` cannot see a file that has never been staged, which is exactly
# when a new file is most likely to carry a credential and exactly what the
# next `git add -A` would take. This header said "tracked" for longer than the
# code did.
#
# The defect this exists to catch is a credential, or a fingerprint of a private
# system, reaching a remote. Once it does, a history rewrite does not undo it:
# the value was readable, and it may be cached, mirrored or already indexed.
# Rotation is the fix; this is what stops it needing one.
#
# ⛔ IT FINDS THE SHAPES IT KNOWS, AND A GREEN RUN IS NOT A CLEARANCE.
# It cannot find a password that looks like a word, a hostname that reads as
# prose, or a page of correct-looking examples that happens to describe a real
# system. It narrows the reading. It does not replace it.
#
# Usage:
#   sh scripts/common/check-no-secrets.sh              tracked + untracked
#   sh scripts/common/check-no-secrets.sh --public     also the fingerprint rules
#   sh scripts/common/check-no-secrets.sh --json
#   sh scripts/common/check-no-secrets.sh --all-history   ⚠ slow; scans every blob
#   sh scripts/common/check-no-secrets.sh --scope references   the exempt corpus
#
# --scope PATH scans ONLY that path, including one the default scope exempts.
# ⛔ It is how the reference corpus exemption below is re-checked when a tree is
# added, and the exemption's own instruction named it for one session before it
# existed. A guard's re-check procedure that cannot be run is not a procedure.
#
# --public adds the rules that only matter for a repository that is or will be
# public: emails, absolute home paths, long hex identifiers. In a private
# project those are legitimate content, which is why they are not the default.
#
# Exit codes: 0 nothing found, 1 something found, 2 could not run.
#
# ⛔ Read the exit code from this process, unpiped. Piping it into anything
# reports the pipeline's status, so a run that failed reads as green.

set -u

PUBLIC=0
JSON=0
HISTORY=0
SCOPE=

while [ $# -gt 0 ]; do
  case "$1" in
    --public)      PUBLIC=1 ;;
    --json)        JSON=1 ;;
    --all-history) HISTORY=1 ;;
    --scope)
      shift
      [ $# -gt 0 ] || { printf 'check-no-secrets: --scope needs a path\n' >&2; exit 2; }
      SCOPE=$1
      ;;
    -h|--help) awk 'NR>1 { if (/^#/) { sub(/^# ?/, ""); print } else exit }' "$0"; exit 0 ;;
    *) printf 'check-no-secrets: unknown argument: %s\n' "$1" >&2; exit 2 ;;
  esac
  shift
done

command -v git >/dev/null 2>&1 || { printf 'check-no-secrets: git not found\n' >&2; exit 2; }
git rev-parse --show-toplevel >/dev/null 2>&1 || { printf 'check-no-secrets: not a git repository\n' >&2; exit 2; }
SELF=check-no-secrets
REPO_ROOT=$(git rev-parse --show-toplevel)

# ⛔ EVERY git QUERY BELOW RUNS FROM THE REPOSITORY ROOT. `git ls-files` is
# relative to the process working directory, so without this a run from a
# subdirectory silently scopes itself to that subtree and reports clean over
# everything else. The scope of a guard must not depend on who called it.
cd "$REPO_ROOT" || { printf '%s: cannot enter %s\n' "$SELF" "$REPO_ROOT" >&2; exit 2; }

# ⛔ TRACKED **PLUS UNTRACKED-BUT-NOT-IGNORED**. `git ls-files` alone cannot see
# a file that has never been staged, which is exactly when a new file is most
# likely to carry a defect and exactly what the next `git add -A` will take.
# Ignored files stay out: they are ignored on purpose.
# shellcheck disable=SC2120
# It takes optional pathspecs. Every caller here wants the whole tree, so none
# passes any; the parameter exists so a scoped caller can be added without
# changing the function.

# -- ⛔ THE REFERENCE CORPUS IS EXEMPT, AND THIS ONE WAS DECIDED BY READING ---
#
# `references/` holds other projects' trees at named commits. Every one of them
# is a PUBLIC repository, so nothing in it is exposed by this tree that its own
# author has not already published, and this check protects against THIS project
# leaking something of its own.
#
# ⭐ THE EXEMPTION WAS TAKEN AFTER READING EVERY HIT, NOT INSTEAD OF READING
# THEM. Measured on 2026-08-30, `--public`, over the corpus as trimmed:
#
#   a private key block   6 hits: 4 doc comments naming the PEM header as text,
#                         2 test keys in one project's own example file
#   an aws access key id  1 hit, inside a base64 pixel blob in a canvas test
#                         record. A false positive on random base64.
#   a password in a url   the rest: proxy documentation of the form
#                         user-colon-password-at-host, test fixtures using the
#                         same shape, and the pattern matching inside fetched
#                         API JSON
#
# ⛔ Not one is a live credential.
#
# ⭐ RE-READ ON 2026-09-01, over the tree HARNESS-04 added, with
# `--public --scope references/http2jp__hpack-test-case`. 52,396 hits in three
# categories and every one was read:
#
#   a long hex identifier 52,330 hits: the `wire` field of every test case,
#                         which is the HPACK-coded bytes the corpus exists to
#                         carry.
#   an absolute home path 62 hits: `:path` header VALUES in the recorded
#                         requests, each a home directory prefix followed by a
#                         year and an image name. They are URL paths on
#                         2012-era websites and not filesystem paths at all. A
#                         false positive on the shape.
#   an email address      4 hits: the upstream author's own published address
#                         in their CI configuration, a token VARIABLE
#                         interpolated into a code-hosting URL, and an ssh
#                         remote form.
#
# ⚠ Those two rows describe their hits rather than quoting them, because a
# guard that carries a specimen of what it refuses refuses itself. Widening the
# rule so it can read its own comment would be widening a credential rule to
# make a comment legal.
#
# ⚠ That CI file also carries a `secure:` blob, which is a value the CI service
# encrypted to its own key, published by its author in a public repository, for
# a service that no longer runs. It is named here because it is the closest
# thing in the tree to a credential shape and an unmentioned one reads like an
# unnoticed one.
#
# ⚠ A future sweep adds trees these readings did not cover, so re-run with
# `--scope references` and READ the hits before trusting this exemption again.
# docs/reference-sweeps/findings.md records it.

# -- ⛔ THE VENDORED TREES ARE EXEMPT, AND THIS ONE WAS DECIDED BY READING ---
#
# vendor/NAME/ holds third-party source this tree compiles, from a PUBLIC
# repository at the commit vendor/upstream.json records, so nothing in it is
# exposed here that its own author has not already published.
#
# ⭐ READ FIRST, EXEMPTED AFTER. Measured 2026-09-01, --public --scope vendor,
# over rustls at v/0.23.43. 38 hits in three categories and every one read:
#
#   a private key block   4 hits: doc comments naming the two PEM headers as
#                         TEXT, in the sign module of each crypto provider.
#   an email address      4 hits: the upstream author's published address in
#                         two licence files, and a mailing list in the README
#                         and the code of conduct.
#   a long hex identifier 30 hits: 10 are commit ids inside links to other
#                         public repositories in doc comments, and the other
#                         20 are literal test payloads, repeated-digit runs
#                         and two hex-encoded handshake fixtures.
#
# ⚠ The counts above were WRONG on their first draft, written from a read
# rather than from a count, and the claim audit caught them. They are the
# output of this script with those flags.
#
# ⛔ Not one is a live credential.
#
# ⚠ The exemption is vendor/NAME/ and never vendor/. vendor/upstream.json is
# this project's own record and stays in scope; its 40-hex base field is
# narrowed below by name rather than by exempting the file.
#
#
# ⭐ THE GENERATED SERIES UNDER patches/NAME/ IS EXEMPT FOR THE SAME REASON,
# and it was read too. Every line of a patch body comes from the tree above, so
# scanning the diff scans the exempt file twice. Measured 2026-09-01 over the
# rustls series: one hit, the base commit in the header vendor-diff.mjs writes,
# which is the same commit the manifest records and is public by construction.
# ⚠ patches/README.md is NOT exempt: it is this project's own writing, and it
# is where the absolute home paths in a pasted cargo failure were caught.
#
# ⚠ A later entry vendors a tree this reading did not cover, so re-run with
# --scope vendor and --scope patches and READ the hits before trusting this
# exemption again.

list_files() {
  if [ -n "$SCOPE" ]; then
    # ⛔ Under --scope the corpus exemption does NOT apply, which is the whole
    # point of the flag: it exists to read the thing the default scope skips.
    {
      git ls-files -- "$SCOPE" 2>/dev/null
      git ls-files --others --exclude-standard -- "$SCOPE" 2>/dev/null
    } | sort -u
    return
  fi
  {
    git ls-files -- "$@" 2>/dev/null
    git ls-files --others --exclude-standard -- "$@" 2>/dev/null
  } | sort -u | grep -vE '^(references|vendor/[^/]+|patches/[^/]+)/'
}


FOUND=0
REPORT=""

hit() {
  FOUND=$((FOUND + 1))
  REPORT="$REPORT
== $1 ==
$2"
}

# --- 1. a credential FILE is tracked -----------------------------------------
# The strongest signal there is: not a value that looks like a secret, but a
# file whose whole purpose is to hold one.
CREDS=$(list_files \
  | grep -E '(^|/)(\.env(\..+)?|\.dev\.vars(\..+)?|.*\.(pem|key|p12|pfx|keystore|jks)|id_rsa|id_ed25519|id_ecdsa|credentials\.json|service-account.*\.json)$' \
  | grep -vE '\.example$|\.sample$|\.template$' || true)
[ -n "$CREDS" ] && hit "a credential file is tracked" "$CREDS"

# --- 2. secret-shaped strings ------------------------------------------------
# Each pattern is a vendor's documented token shape. A generic "high entropy"
# rule is deliberately absent: it fires on hashes, minified code and base64
# fixtures, and a check that cries wolf is a check somebody switches off.
scan() {
  _s_name="$1"; _s_pat="$2"
  _s_out=$(list_files | tr '\n' '\0' | xargs -0 grep -nIE "$_s_pat" 2>/dev/null || true)
  [ -n "$_s_out" ] && hit "$_s_name" "$_s_out"
}

scan "a private key block"      'BEGIN (RSA |OPENSSH |EC |DSA |PGP )?PRIVATE KEY'
scan "an aws access key id"     'AKIA[0-9A-Z]{16}'
scan "a github token"           'gh[pousr]_[A-Za-z0-9]{30,}'
scan "a slack token"            'xox[abprs]-[0-9A-Za-z-]{10,}'
scan "a google api key"         'AIza[0-9A-Za-z_-]{35}'
scan "a stripe key"             'sk_(live|test)_[0-9A-Za-z]{16,}'
scan "a npm token"              'npm_[A-Za-z0-9]{36}'
scan "a bearer literal"         'Bearer [A-Za-z0-9._-]{24,}'
scan "a password in a url"      '://[A-Za-z0-9._%+-]+:[^@/[:space:]]{6,}@'

# --- 3. public-only: fingerprints of a private system ------------------------
if [ "$PUBLIC" = "1" ]; then
  scan "an email address"       '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'

  # ⚠ Narrowed, not switched off. A pinned GitHub Action is a 40-hex commit on
  # a PUBLIC repository, and pinning is the SAFE practice this template asks
  # for: a tag moves and a moved tag runs unreviewed code. A rule that fires on
  # correct hardening is a rule somebody disables, so the `uses:` form is
  # excluded by shape rather than the whole hex rule being dropped.
  #
  # ⚠ A DECLARED PIN is the second such shape: a commit and a SHA-256 written
  # into a script that fetches and verifies code before executing it, so 40 hex
  # and 64 hex, both public by construction, both the SAFE practice.
  # ⚠ THE WRAPPER THAT FIRST PRODUCED THIS SHAPE IS NOT IN THIS TREE, and the
  # exclusion stays because docs/containers.md tells this project to write one.
  #
  # ⛔ A THIRD SHAPE IS COMING HERE AND HAS NO EXCLUSION YET. A raw ClientHello
  # recorded as hex is hundreds of hex characters, and it is the one artefact
  # docs/architecture.md says never to drop. When raw captures land, exclude
  # them by PATH and by the field name that holds them, never by widening the
  # hex rule.
  # ⛔ Excluded by NAME, narrowly. The hex has to be assigned to an identifier
  # that says it is a pin, because a credential is not assigned to something
  # called PinnedSha256. Widening this to all hex would remove the rule.
  _hex_out=$(list_files | tr '\n' '\0' \
    | xargs -0 grep -nIE '\b[0-9a-f]{24,}\b' 2>/dev/null \
    | grep -vE 'uses:[[:space:]]*[A-Za-z0-9._-]+/[A-Za-z0-9._-]+@[0-9a-f]{40}' \
    | grep -vE '[Pp]inned(Ref|Sha256|Commit|Digest)|PINNED_(REF|SHA256)' || true)
  # ⚠ A GIT COMMIT CITED AS PROVENANCE is the third such shape, and it is the
  # one THIS project produces: docs/reference-sweeps/findings.md carries a
  # commit per reference so a line citation can be checked, and
  # docs/methodology/references.md requires exactly that.
  # ⛔ Excluded by SHAPE, narrowly: exactly 40 lower-case hex inside a markdown
  # code span. A credential is not written that way, and widening this to all
  # hex would remove the rule. TOOL-03 is the entry that adds the fourth shape,
  # a recorded ClientHello, when raw captures land.
  # shellcheck disable=SC2016  # The single quotes are deliberate: the backticks
  # below are a markdown code span being MATCHED, not a substitution to run.
  # Double quotes here would hand the shell a command to execute.
  _hex_out=$(printf '%s\n' "$_hex_out" | grep -vE '`[0-9a-f]{40}`' || true)
  # ⚠ THE FIFTH SHAPE: A GIT COMMIT ID IN THE VENDOR MANIFEST. Excluded by
  # NAME, narrowly. vendor/upstream.json records the commit each vendored tree
  # was taken at, which is the field that makes the record checkable at all,
  # and a commit id is public by construction. ⛔ Only a value assigned to
  # `base` is excluded, so any other 40-hex run in that file is still
  # reported. ⛔ Keep this identical to the ps1 twin. TODO/vendor.md.
  _hex_out=$(printf '%s\n' "$_hex_out" | grep -vE '"base":[[:space:]]*"[0-9a-f]{40}"' || true)

  # -- ⭐ THE FOURTH SHAPE, AND IT IS THE ONE THIS PROJECT EXISTS TO PRODUCE ---
  #
  # A raw ClientHello recorded as hex is hundreds of hex characters, and
  # SCHEMA-06 requires one on every capture. The comment above predicted this
  # would fail the gate on the day the first one landed, and it did: TOOL-01
  # created crates/b-ids-harness/fixtures/client-hello.hex and this rule
  # refused it.
  #
  # ⛔ THE HEX RULE ITSELF IS NOT WIDENED. That was the tempting fix and it
  # removes the rule. Three narrow exclusions instead, each by NAME or by FILE
  # TYPE, exactly like the three above:
  #
  #   1. a hex run assigned to an identifier ending in `_hex`. That is this
  #      project's own naming rule for a field that holds wire bytes:
  #      raw_hex, body_hex, session_id_hex, client_hello_hex, payload_hex.
  #      ⚠ A credential is not assigned to something called body_hex, and a
  #      credential assigned to a field with any OTHER name is still refused,
  #      including one sitting in the same file.
  #   2. a `.hex` file, which this project defines as one raw capture on one
  #      line and nothing else.
  #   3. `checksum = "..."` in a lock file, which is a declared digest of a
  #      published artefact and is the same shape as the pin above.
  #
  # ⛔ Mutation-proved: a credential-shaped value planted inside a raw capture
  # file under a different field name is still refused. TOOL-03 carries the run.
  _hex_out=$(printf '%s\n' "$_hex_out" \
    | grep -vE '[A-Za-z0-9_]*_hex"?[[:space:]]*[:=]' \
    | grep -vE '^[^:]*\.hex:' \
    | grep -vE '^[^:]*(Cargo\.lock|\.lock):[0-9]+:[[:space:]]*checksum[[:space:]]*=' || true)

  # -- ⭐ THE SIXTH AND SEVENTH SHAPES, BOTH FROM THE PUBLISHED CORPUS --------
  #
  # CORPUS-01 wrote the first profile, and the rule refused two things in it
  # that the four exclusions above do not cover. ⛔ THE HEX RULE IS STILL NOT
  # WIDENED. Two more narrow exclusions, each by NAME or by PATH-AND-SHAPE:
  #
  #   6. a hex run assigned to an identifier named `sha256`. That is the
  #      content address the corpus index carries beside every published file,
  #      and it is the same shape as the `checksum` exclusion above: a declared
  #      digest of a published artefact, public by construction. ⚠ Only that
  #      exact name, so a credential assigned to anything else is still
  #      refused.
  #   7. AN ELEMENT OF A HEX ARRAY, under corpus/ or raw/ only. Pretty-printed
  #      JSON puts each entry of `http2_frames_hex` on its own line, which
  #      leaves the field name on a line the value is not on, so exclusion 1
  #      cannot see it. ⛔ Narrowed by BOTH the path and the shape: a line under
  #      those two directories that is nothing but a quoted lower-case hex run
  #      and an optional comma. A credential assigned to a field is still
  #      refused there, and so is one on a line carrying anything else.
  #
  # ⚠ AND THOSE BYTES HAVE A SECOND GATE, which is why this exclusion is
  # acceptable at all: b_ids_schema::Raw::check decodes the recorded bytes and
  # REFUSES the profile if they spell out a cookie or authorization header. The
  # one class of credential that could hide inside a frame array is the one
  # thing already checked by the model itself.
  #
  # ⛔ Mutation-proved: a credential-shaped value planted inside a corpus
  # profile under a different field name is still refused. TODO/corpus.md,
  # CORPUS-01, carries the run.
  # ⚠ AND THE DIGEST LINE THE PROVISIONING TOOL PRINTS, which is a label and a
  # hash and nothing else. `provision-browser` prints `sha256  HEX` for the
  # archive it fetched, and an entry that pastes that output is pasting a
  # measurement rather than a credential. ⛔ Narrowed to that exact shape: a
  # label, whitespace, 64 hex, end of line. A hex run with anything else beside
  # it is still refused. TODO/driver.md, DRIVER-08.
  _hex_out=$(printf '%s\n' "$_hex_out" \
    | grep -vE '"sha256"[[:space:]]*:[[:space:]]*"[0-9a-f]{64}"' \
    | grep -vE ':[0-9]+:sha256[[:space:]]+[0-9a-f]{64}$' \
    | grep -vE '"published_sha256":"[0-9a-f]{64}"' \
    | grep -vE ':[0-9]+:verified [0-9a-f]{64} matches the digest' \
    | grep -vE '^(corpus|raw)/[^:]*:[0-9]+:[[:space:]]*"[0-9a-f]+",?$' || true)

  [ -n "$_hex_out" ] && hit "a long hex identifier" "$_hex_out"
  # ⚠ Narrowed rather than switched off. `/home/linuxbrew/` and `/home/runner/`
  # are well-known generic paths, not a fingerprint of anybody's machine, and a
  # check that fires on them is one somebody disables. Whenever this produces a
  # false positive, add the generic path here; do not widen the exclusion to
  # the whole rule.
  _home_out=$(list_files | tr '\n' '\0' \
    | xargs -0 grep -nIE '([A-Za-z]:[\\/]Users[\\/]|/home/|/Users/)[A-Za-z0-9._-]+' 2>/dev/null \
    | grep -vE '/home/(linuxbrew|runner|user|vagrant|ubuntu|node)/' \
    | grep -vE '/Users/(runner|user)/' || true)
  [ -n "$_home_out" ] && hit "an absolute home path" "$_home_out"
fi

# --- 4. the whole history, on request ----------------------------------------
# ⚠ Slow. It reads every blob ever committed, which on a large repository is
# minutes rather than seconds. Worth running once before a repository is first
# published, and not on every commit.
if [ "$HISTORY" = "1" ]; then
  _h_out=$(git rev-list --objects --all 2>/dev/null \
    | git cat-file --batch-check='%(objecttype) %(objectname) %(rest)' 2>/dev/null \
    | awk '$1 == "blob" { print $2, $3 }' \
    | while read -r sha path; do
        if git cat-file blob "$sha" 2>/dev/null \
           | grep -qIE 'BEGIN (RSA |OPENSSH |EC |DSA |PGP )?PRIVATE KEY|AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{30,}'; then
          printf '%s  %s\n' "$sha" "$path"
        fi
      done)
  [ -n "$_h_out" ] && hit "a secret shape in history (rotate first, then decide about the history)" "$_h_out"
fi

# --- report -------------------------------------------------------------------
if [ "$JSON" = "1" ]; then
  printf '{"schema":"check-no-secrets/1","findings":%s,"public_rules":%s,"history_scanned":%s}\n' \
    "$FOUND" \
    "$([ "$PUBLIC" = 1 ] && echo true || echo false)" \
    "$([ "$HISTORY" = 1 ] && echo true || echo false)"
  [ "$FOUND" -gt 0 ] && exit 1
  exit 0
fi

if [ "$FOUND" -gt 0 ]; then
  printf '%s\n\n' "$REPORT"
  printf '⛔ %s category/categories matched.\n\n' "$FOUND"
  printf 'If any of it is a real credential, IN THIS ORDER:\n'
  printf '  1. ROTATE IT. Now, before anything else. It is compromised from the\n'
  printf '     moment it was written, and removing the file does not change that.\n'
  printf '  2. Tell the operator. They own the account.\n'
  printf '  3. Remove it from the tree, and add the ignore rule.\n'
  printf '  4. A history rewrite is the operator%s call and the operator%s action.\n' "'s" "'s"
  printf '     It is tidying after the fix, not the fix.\n\n'
  printf 'If it is a false positive, narrow the pattern in this script rather than\n'
  printf 'switching the check off. See docs/security/secrets.md.\n'
  exit 1
fi

printf 'no secret shapes found in %s files (tracked plus untracked-not-ignored)' "$(list_files | wc -l | tr -d ' ')"
[ "$PUBLIC" = "1" ] && printf ' (public rules included)'
printf '\n'
printf '⚠ This finds the shapes it knows. It is not a clearance: read the diff.\n'
exit 0
