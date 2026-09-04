# CHANGELOG

What shipped, when, and where the evidence is. Newest first.

⛔ **No release has been cut**, and a pushed tag is the only thing that produces
one. Most entries below are repository changes rather than published artefacts,
and ⛔ **every entry says which it was**: the `data` branch is the one surface
anything here has reached.
[`TODO/PROGRESS.md`](TODO/PROGRESS.md) is where the work stands.

---

## Unreleased

⛔ **Nothing here has been released.** The section exists because an entry is a
`### ` heading under a `## ` section, and a file with no section has no
entries a check can read. TOOL-14.

### 2026-09-04T08:30:00Z - the capture matrix added five profiles, which is five more than it has ever added

**Record:** [`TODO/corpus.md`](TODO/corpus.md) `CORPUS-02` (open),
[`TODO/docs.md`](TODO/docs.md) `DOC-03`.
**Deployed:** no. No tag was pushed and no release cut. ⚠ The data branch gains
five profiles, five raw sidecars and the regenerated aggregates on the next push
to the default branch.

What landed:

- ⛔ **The capture lane adds the profile it took.** Nothing ever did: the rule
  that a capture script does not write into an append-only corpus was applied to
  the script and never to the lane, so six green lanes on run `33849934489` took
  six captures and added nothing, and the run reported success.
- ⭐ **Five profiles**, verified locally and merged: the first Gecko capture on a
  runner, the first two UNBRANDED builds, and two stable Chrome builds an hour
  apart on two platforms carrying two different versions.
- ⭐ **The first profiles carrying `captured.acquisition`**: a route, a URL and
  the digest of the archive that was fetched.
- ⛔ **The trust-anchor check called a measurement a defect.** An unbranded build
  publishes an EMPTY root-store list, two bytes on the wire, and the check
  reported "no identifiers" and went red. It tells an empty list from a decode
  that produced nothing now.
- ⛔ **The collect job re-derives the index** rather than keeping whichever
  lane's copy it merged last, which left the tree failing `b-ids-corpus verify`.
- ⭐ **`SECURITY.md` exists**, with the threat model: what the harness accepts
  connections from, what a hosted oracle would receive, and what is in scope.
  `DOC-03`.
- ⚠ **The `chromium` cell is enabled**, so the next run answers whether a snap
  can be driven rather than leaving it predicted.

### 2026-09-04T07:20:00Z - the corpus has a non-Chromium profile, and three checks that were green would have refused every capture after it

**Record:** [`TODO/driver.md`](TODO/driver.md) `DRIVER-11`.
**Deployed:** no. No tag was pushed and no release cut. ⚠ The data branch gains
the Firefox profile, its raw sidecar and 42 regenerated aggregates on the next
push to the default branch; every profile it already carries is unchanged.

What landed:

- ⭐ **Firefox 154.0.1 completed a TLS 1.3 handshake against this project's own
  terminator**, on one Windows host, and the capture is published as
  `corpus/v1/firefox/stable/win64/154.0.1.json`. `captured.trust` reads
  `trust-store`. It is the first profile in the corpus from a stack that is not
  a Chromium.
- ⛔ **The launcher has a path per engine.** The switch list is a table, so Gecko
  is given `--profile`, `--headless` and `--new-instance` rather than arguments
  it reads as file names, and a trust configuration an engine has no route for
  is refused by name.
- ⭐ **Firefox takes no certificate switch at all**, so the trust is arranged
  where it looks for it: the launch writes an NSS certificate database into the
  throwaway profile carrying the run's own authority and a trust record for it.
  `crates/b-ids-driver/src/nssdb/` is a SQLite writer, a DER field reader and a
  SHA-1, each cited against `mozilla/nss` at a named commit in
  [`references/mozilla__nss/`](references/mozilla__nss/).
- ⛔ **A trust record without the certificate's SHA-1 is discarded in silence.**
  That is NSS's rule and it is why this tree carries a SHA-1 at all. A
  certificate object alone is a certificate the browser knows and does not
  trust.
- ⛔ **The publish manifest is `corpus-publish/2`** and records `derived` per
  artefact. Without it, adding one profile changed nineteen aggregates and
  `check-data-branch` called it a rewritten branch, so under the old rule no
  capture could ever be published again.
- ⛔ **`check-coverage` reports a capture no planned cell covers.** The corpus
  held seven profiles and the report accounted for six.
- ⚠ **The `firefox/stable/linux64` cell is enabled**, and nothing has run a
  Gecko lane on a hosted runner yet.

### 2026-09-04T05:20:00Z - the Windows CI failure was this repository's own probe, and two checks were passing by comparing something to itself

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-09`,
[`TODO/publish.md`](TODO/publish.md) `PUB-11`, `PUB-04` and `PUB-14`,
[`TODO/validator.md`](TODO/validator.md) `VALID-05`,
[`TODO/harness.md`](TODO/harness.md) `HARNESS-11`.
**Deployed:** no. No tag was pushed and no release cut. ⚠ The data branch gains
a `configs/` tree on the next push to the default branch; the profiles it
carries are unchanged.

What landed:

- ⛔ **The Windows toolchain failure was traced to this tree.** `rustc` and
  `cargo` are rustup proxies, so the probe, run in a tree pinning a toolchain
  the runner does not have, started installing it and killed the install at its
  six-second limit. The component conflict the job reported was a fragment of
  that. Both probe halves now refuse to start an install, the workflow installs
  before it probes, and `doctor --fixture` holds the claim. `CI-09`.
- ⛔ **Two checks were passing by comparing something to itself.**
  `check-data-branch` compared the published branch against a materialised copy
  of that same branch and reported green; `check-corpus` asked this repository's
  history about files that are not in it. Both had one cause: an export on the
  line above the guard. `PUB-11`.
- ⭐ **`PUB-11` closed at ten of ten** with `corpus/` and `raw/` moved out of the
  working tree, and both directories restored and compared against `HEAD`.
- ⭐ **Generated client configuration is published.** Thirty-seven files per
  build, twenty-four of them refusals naming a hole at a file and a line, gated
  on the support matrix by `check-generated-configs` and its twin. `PUB-04`.
- ⛔ **The data branch check can tell BEHIND from WRONG.** Adding an artefact
  class made it red on a state the publisher clears; it now reports a pending
  publish and still refuses a changed byte, a deleted path and a rewritten
  branch. `PUB-14`.
- ⭐ **A conformance suite compares a client against the profile it claims to
  be**, field by field, with a third verdict for what a browser varies per
  connection. `VALID-05`.
- ⭐ **The TCP layer's capability is measured**: one field of six is readable
  without a raw socket, and `TcpStream::ttl` is this host's own hop limit rather
  than the peer's. `HARNESS-11`.
- ⚠ **The resolver knows four browser families**, not two, and `CORPUS-02`'s
  blocker moved from the resolver to the launcher and to acquisition.

### 2026-09-03T13:07:00Z - the documents were checked against the tree, and nine of them were wrong about what it does

**Record:** [`TODO/tooling.md`](TODO/tooling.md) `TOOL-19`,
[`TODO/corpus.md`](TODO/corpus.md) `CORPUS-06`,
[`TODO/publish.md`](TODO/publish.md) `PUB-12` and `PUB-11`.
**Deployed:** no. No tag was pushed and no release cut. ⚠ The data branch
rebuilds on the next push to the default branch, and the profiles it carries are
unchanged by this.

What landed:

- ⛔ **Nine documents and three script headers described a smaller project than
  the one on disk.** The reference pages still said there was no corpus, nothing
  published and no digest computed, four sessions after each became false. Every
  one is amended in place and the superseded wording is in
  [`docs/HISTORY/stale-documents.md`](docs/HISTORY/stale-documents.md).
- ⭐ **A check now holds both catalogues.** `check-catalogues` asserts every
  script is named by [`scripts/README.md`](scripts/README.md) and every document
  by the index that routes to it. Pointed at this repository as it stood before
  the pass, it refuses thirteen scripts that had no section at all.
- ⛔ **The headless normalisation had no caller.** `DRIVER-03` built it and named
  the seam; `CORPUS-01` landed and nothing wired it, so every published profile
  and every published `user-agent` route carries `HeadlessChrome`. It is wired
  now, gated on the launch rather than on the value, and the six published
  profiles keep what they have because the corpus is append-only.
- ⭐ **The licence check reads the branch a consumer actually fetches**, both the
  manifest identifier and the bytes of the `LICENSE` beside it, from a local ref
  rather than a fetch.
- **One resolver answers where the corpus is**, and twelve check pairs ask it
  instead of assuming the working tree. ⚠ `PUB-11` stays open: three legs reach
  the corpus through Rust that still resolves the workspace root, and the entry
  names each at a file.

The story of four findings, kept here because the documents say what is true
rather than what happened:

- ⚠ **A check's own first run found a defect in the check.** `check-catalogues`
  asked `git ls-files` alone and reported a clean catalogue of 43 scripts while
  its own half sat beside it, written that minute and untracked.
- ⛔ **A `[switch]` parameter and a lower-case local are one variable in
  PowerShell.** A script-scope `$ref = Get-BranchRef` assigned a string to the
  `[switch]$Ref` parameter and threw a type-conversion error before anything
  ran.
- ⛔ **A `'*/index.json'` pattern matches nothing on Windows**, because
  `$_.FullName` is backslash-separated. The twin counted the two derived files
  as profiles and said 8 where the POSIX half said 6, which is what comparing
  two answers is for.
- ⚠ **The shell ate a backticked payload inside a `node -e` string**, and a
  paragraph reached a document with three code spans emptied. The tree's own
  note says a payload goes through `write-file.mjs` from a file, and it does
  now.

### 2026-09-03T09:00:00Z - the trigger, the consumer crate, and a client that puts a profile back on a wire

**Record:** [`TODO/publish.md`](TODO/publish.md) `PUB-10`,
[`TODO/library.md`](TODO/library.md) `LIB-01` and `LIB-02`,
[`TODO/emitters.md`](TODO/emitters.md) `EMIT-01` and `EMIT-02`,
[`TODO/validator.md`](TODO/validator.md) `VALID-04`,
[`TODO/ci.md`](TODO/ci.md) `CI-05`, [`TODO/driver.md`](TODO/driver.md)
`DRIVER-06`.
**Deployed:** ⭐ the data branch, and nothing else. The push that landed this
change ran the workflow it adds, which created `origin/data` with 200 files.
⛔ No tag was created, no asset uploaded and no release cut: a tag is the only
thing that produces one, and pushing one is the operator's act.

What landed:

- ⛔ **Something triggers the two publishing surfaces at last.** `PUB-01` and
  `PUB-02` assembled and checked; nothing reached either. A workflow now does,
  on a manual dispatch, a push to the default branch and a pushed tag, with the
  write scoped to the two publishing jobs.
- ⛔ **Two conditions from two sources stand in front of the data branch**: the
  crate's own rewrite rule, and a push carrying no force flag and no `+`
  refspec so the remote refuses anything that is not a fast-forward.
- ⛔ **A defect in the assembler, found by running it two ways.** The published
  route manifest carried the absolute path of whoever built it, so a build under
  a relative root and one under an absolute root produced different bytes for
  one corpus. Fixed, and a case keeps it fixed.
- ⭐ **A crate hands a program a profile**, with the corpus embedded at build
  time, no network code at all, and no substitute for a platform this project
  has not captured.
- ⭐ **A client puts a profile back on a wire.** 1951 of 1983 bytes are
  identical to the browser's own captured hello; the 32 that differ are the
  random the model does not record and never will.
- ⭐ **JA4, implemented from the published specification** and checked against
  sixteen vectors whose expected values came from that specification or from a
  derivation in `jq` and `sha256sum`. ⛔ No JA4+ member is computed anywhere.
- ⭐ **A support matrix generated from a run**, with five holes each carrying a
  file and a line that a check resolves.
- ⭐ **A cold-start job**, with every cache refused and every stage named by a
  report that runs whatever happened.
- Three checks joined the gate: `check-publish`, `check-cold-start` and
  `check-support-matrix`, each with a twin and a comparison row.
- ⭐ **The data branch exists.** The push that landed this change triggered the
  workflow it adds: the branch was created with 200 files and its tree was
  compared object for object with a local build. ⚠ The release job of the same
  run skipped, because no tag was pushed. `check-data-branch` compares against
  the published branch now instead of reporting a skip, in both halves, and its
  schema moved to `check-data-branch/2` because the object gained a field.

### 2026-09-03T05:40:00Z - one assembler, two publishing surfaces

**Record:** [`TODO/publish.md`](TODO/publish.md) `PUB-01` and `PUB-02`.
**Deployed:** no. No release was cut, no tag created and no branch pushed. Both
acceptances are local and say so in their own output.

What landed:

- ⭐ **`PUB-01` and `PUB-02` publish the same bytes**, because there is one
  assembler and both surfaces take what it produced. Two surfaces built by two
  assemblers is two answers to what this project publishes.
- ⛔ **Nothing in the assembler reads a clock.** A build is stamped with a
  digest of the corpus, so a rebuild is byte-identical and a change is not.
- ⛔ **A tag that already exists is refused**, along with a malformed date and
  an empty build. A published release is immutable and consumers pin releases.
- ⛔ **The source, the vendored trees and the reference corpus are not on the
  data branch**, and both halves of the check assert each of seven directories
  is absent.

The story of two findings, kept here because the documents say what is true
rather than what happened:

- ⚠ **The two halves of a check can resolve different `tar` binaries on one
  machine.** GNU tar wants `--owner=0` and refuses `--uid`; the bsdtar Windows
  ships wants `--uid 0` and refuses `--force-local`; and one of the two date
  formats is a parse error to one of them. The archive leg skipped on every host
  until both spellings were probed.
- ⛔ **A leg that always skips is a leg nobody knows works**, and this change
  had two candidates for that: the archive comparison, which was fixed, and the
  comparison against the published data branch, which genuinely cannot run
  because the branch does not exist and is reported as a skip with the reason.

### 2026-09-03T04:20:00Z - the licence stated in seven places, from one home

**Record:** [`TODO/publish.md`](TODO/publish.md) `PUB-07`.
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **One home for the identifier**, `b_ids_schema::LICENSE`, with every
  generated statement reading it and a check asserting the seven agree. A file
  that travels alone still has to say what it is.
- ⛔ **The field is OPTIONAL in the published JSON Schema**, and the check
  asserts it is not in `required`. A schema requiring it would refuse every
  profile in the corpus, because the six published before today predate the
  field and the corpus is append-only.
- ⭐ **The leg that can fail is what the WRITER emits**, not what the corpus
  holds: a loop over the published set finds nobody carrying the field today,
  and the entry says so rather than reporting a pass.

### 2026-09-03T03:10:00Z - fifty-four routes a program can read with curl

**Record:** [`TODO/publish.md`](TODO/publish.md) `PUB-03`.
**Deployed:** no. The tree is generated into an ignored directory and checked
there; `PUB-02` is the surface that will serve it.

What landed:

- ⭐ **One file per value, at every permutation the corpus holds a value for**,
  with an `index.txt` per directory and `latest` as a real file rather than a
  redirect. A single-value file carries the value and nothing else, so a
  consumer never strips anything.
- ⭐ **The check reads the corpus rather than the generator.** A manifest names
  the profile and the property behind every route, and each half goes and reads
  the value out of the profile: one in jq, one in ConvertFrom-Json.
- ⛔ **Nothing is published for a value the corpus does not hold.** No JA3, JA4
  or Akamai route exists, because nothing here computes one.

The story of three findings, kept here because the documents say what is true
rather than what happened:

- ⛔ **jq on Windows writes CRLF, for the second time in this tree.** Every one
  of the 54 value comparisons failed while both sides were correct.
- ⛔ **The generated tree lives under an ignored directory**, so the git-based
  walk answered with nothing and the check reported a clean tree it had never
  opened. It is the same defect the check's own header already described for
  the fixture path, arriving from the other direction.
- ⛔ **A list file and a single-value file both end in `txt`**, so a classifier
  reading the last dot would have refused the newline a list needs.

### 2026-09-03T01:40:00Z - a scheduled run that finds a change opens a pull request

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-04`.
**Deployed:** no. Nothing is published from this repository yet, and no pull
request has been opened by anything in this change.

What landed:

- ⭐ **The body, the branch, the labels and the five merge conditions come out
  of the crate**, as the third renderer of the model the release body and the
  changelog already use. The workflow holds the token and does the opening, so
  the generator is testable without a network.
- ⭐ **Three of the five merge conditions are computed from the published
  profiles** rather than claimed by the run: agreement across two independent
  sources, a provenance regression, and whether the diff touched a field the
  change class predicts. The other two are facts about the run, and every field
  of the run file is required.
- ⛔ **The write is job-scoped**, on the collect job alone, using the run's own
  token. ⚠ It also needs a repository setting this file cannot grant, and the
  step says so rather than failing silently.

The story of two findings, kept here because the documents say what is true
rather than what happened:

- ⚠ **`CI-04` predicted that a version bump moves the User-Agent and the brand
  list**, and the field-level diff compares header positions rather than header
  values, so neither is a field it can report. The predicted set for a bump is
  the version and nothing else, which is the strict reading and the safe one.
- ⛔ **A guard added in this change turned out not to be the thing enforcing its
  own rule.** Removing the early return that makes a no-op change produce
  nothing changed no behaviour at all, because a change with no movement has
  nothing for the loop below it to iterate. The line is kept as the explicit
  statement of the invariant and its comment now says which of the two holds it.

### 2026-09-02T23:55:00Z - four more formats, two declined, and a support matrix nobody types

**Record:** [`TODO/schema.md`](TODO/schema.md) `SCHEMA-12`.
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **YAML, TOML, a SQLite dump and a protobuf definition**, from the one
  generator `SCHEMA-08` established, each with a reader beside its writer. CBOR
  and MessagePack were weighed and declined, and the reason for each is
  published rather than left to a consumer to guess at.
- ⭐ **The support matrix is generated from the generator**, so a format added,
  renamed or declined moves it in the same change, and both halves of
  `check-formats` read it as their catalogue rather than carrying a second copy
  of the list.
- ⭐ **The dump loads into a real database**, which is the one reader in this
  check that is not this project's own.

The story of two fixes, kept here because the documents say what is true rather
than what happened:

- ⛔ **Two of `SCHEMA-08`'s readers split their input into LINES**, and both
  formats allow a newline inside a quoted value. The published corpus carries no
  such value, so neither defect was reachable from real data; rendering a
  profile whose values carry a quote, an apostrophe, a tab, a newline, a
  backslash and a non-ASCII character took both red at once. Both readers are
  quote-aware across newlines now.
- ⛔ **A guard added in this change passed over the mutation that should have
  taken it red.** The SQLite leg asked one question and read any error from it
  as "this host has no JSON1", so a dump whose `CREATE TABLE` no longer declared
  the column the format promises reported `ok-no-json1` and exited 0. The
  capability is probed separately from the question now.

### 2026-09-02T15:30:00Z - the corpus stopped recording builds nobody chose

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and the eight entries it
names: [`TODO/driver.md`](TODO/driver.md) `DRIVER-08` and `DRIVER-10`,
[`TODO/schema.md`](TODO/schema.md) `SCHEMA-08`,
[`TODO/harness.md`](TODO/harness.md) `HARNESS-15` and `HARNESS-16`,
[`TODO/tooling.md`](TODO/tooling.md) `TOOL-18`,
[`TODO/corpus.md`](TODO/corpus.md) `CORPUS-04`, and
[`TODO/publish.md`](TODO/publish.md) `PUB-08`.
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **The purge and the install ran on hosted runners**, both platforms and
  both routes, which is the leg that had never executed anywhere. Three of the
  six profiles now name the URL and the digest of the artefact the browser was
  installed from; before today every profile carried
  `captured.acquisition: null`.
- ⭐ **A browser that is not Chrome is in the corpus.** Edge `151.0.4129.101`,
  provisioned from the vendor's enterprise index, which publishes a SHA-256 per
  artefact that the tool compares what arrived against. It could not capture at
  all before: its SUID sandbox helper was not configured on the runner image,
  and the browser's own log said so, which is what `DRIVER-07` kept it for.
- ⭐ **A cold hello is no longer thrown away because its own connection carried
  no HTTP/2.** The two halves are selected independently, and `captured.connections`
  records which connection each came from. The Edge lane is the proof: its only
  cold hello arrived on a connection that reached no HTTP/2.
- ⭐ **Five published formats from one generator**, each with a reader, so a
  round trip is a round trip rather than a comparison of two writers.
- ⭐ **The trust-anchor extension is measured here** rather than inherited, and
  published per build with its capture date.
  [`docs/trust-anchors.md`](docs/trust-anchors.md) states the three options with
  the cost of each and asserts no preference.
- ⭐ **The gate costs 213 seconds where it cost about 600**, with four more
  checks in it than before.

⛔ **A ruling's reasoning was refuted by measurement.** The vendor route was
ruled to give both platforms the same build "because both install on the same
day"; measured one hour apart on one day, it served `152.0.7977.75` on Linux and
`152.0.7977.76` on Windows. The ruling stands and one sentence of its reasoning
does not.

⛔ **`certutil -addstore -user Root` does not fail on `windows-latest`. It
returns 124**, which is the timeout verdict: it never answers. The same tool one
store apart, `-user CA`, returns 0 in under a second. So that platform cannot
have a root installed unattended by that route, which is a result rather than an
unread failure, and `HARNESS-14`'s comparison still has no answer there.

⛔ **Nine defects in this session's own work, every one caught by running it**,
including two that only a specific lens found: a guard added the same day that
passed because a different line satisfied it, and one fact carrying two names
across two types. [`TODO/PROGRESS.md`](TODO/PROGRESS.md) has the table.

### 2026-09-02T12:10:00Z - the exact-build route, and two findings that moved what it can promise

**Record:** [`TODO/driver.md`](TODO/driver.md) `DRIVER-08`,
[`TODO/ci.md`](TODO/ci.md) `CI-08`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-driver acquire`**: reads the automation-build index and names the
  archive URL for one exact build on one platform. It fetches nothing and
  touches no machine, which is what lets `provision-browser` fetch with the one
  tool each platform already has. ⛔ It runs BEFORE the resolver, like
  `versions` does, because a provisioning run calls it immediately after
  purging every browser off the machine.
- ⭐ **`Source::ManifestFile`**, a third way to read which build an executable
  is. The automation archive is flat: `chrome.exe` sits beside a
  `VERSION.manifest` and there is no version-shaped directory, so on Windows,
  where `--version` is not asked, nothing could version an automation build at
  all. `b_ids_driver::sources_for` is now the one reader all three sources go
  through and `resolve` is a caller of it.
- ⭐ **`.github/workflows/provision.yml`**, dispatch only, two platforms, each
  running the acceptance in its own language. ⛔ Its own workflow rather than a
  step in `capture.yml`: the capture lanes produce the corpus today and an
  unproved purge does not go in front of them. `trust-anchor.yml` is the
  precedent.
- **The capture matrix names its routes.** Every cell carries `route`,
  `branded` and `build`, and two unbranded cells are planned and not attempted.

⛔ **Two findings, each one moving a claim the entry made before it.** The
automation index publishes a SUBSET of builds: 2497 in all, 67 of Chrome `151`,
and neither of the two builds the hosted runner images served, so those two
cannot be reproduced through the exact-build route at all. And `resolve` could
not version an automation build on Windows, which would have made the
provisioning tool's own confirm step unpassable there.

⛔ **And one defect in a check, of a shape this project had already fixed
once.** `check-manual-path` read tracked files only, so a workflow that was
written and never staged escaped it: it reported 9 jobs over a tree carrying 10,
and `git add -N` alone changed the answer. `check-exit-codes` had the same
defect and was fixed on 2026-09-01; this half of the shape was left. Both halves
read untracked files now, and both were seen to fail at exit 1 against a
workflow with its `# manual:` line removed.

### 2026-09-02T11:25:00Z - a tool that purges browsers, and the guard it needed twice

**Record:** [`TODO/driver.md`](TODO/driver.md) `DRIVER-08`, `DRIVER-09` and
`DRIVER-10`, [`docs/HISTORY/README.md`](docs/HISTORY/README.md), and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`scripts/common/provision-browser.sh`**: purge every browser of a family
  from a machine, confirm the purge by requiring `b-ids-driver resolve` to exit
  2, install the build that was asked for, and confirm the version that
  arrived. ⚠ The branded vendor route only, and it refuses a `--version` the
  channel cannot honour rather than accepting one it would ignore.
- ⭐ **`scripts/common/check-provisioning.sh`**: seven refusals asserted on any
  host, and the provisioning leg skipped loudly rather than silently where the
  machine is not disposable. ⛔ Outside the gate on purpose, because it is the
  acceptance for an entry that is open.
- ⛔ **The guard is two independent conditions from two sources**, after one
  condition was measured to be too few: this session mutated the single
  condition and ran the purge path on the operator's own machine. Nothing was
  removed, and that was an accident of registry matching rather than a margin.
  [`docs/HISTORY/README.md`](docs/HISTORY/README.md) carries the incident.
- ⛔ **Two rules were written down that this project did not have**: a guard on
  something irreversible takes two conditions from two sources, and a test that
  has to bypass a guard runs against a copy rather than against the file on a
  machine the guard protects. Both are in
  [`docs/conventions/forbidden-patterns.md`](docs/conventions/forbidden-patterns.md)
  and in [`scripts/README.md`](scripts/README.md) where checks are written.
- ⛔ **Both scripts are PAIRS, because the gate refused them otherwise.**
  `check-exit-codes` reported 27 scripts against 25 the minute the sh halves
  landed: the two counts had been equal only by coincidence, and one untwinned
  script broke the tie. `provision-browser.ps1` and `check-provisioning.ps1`
  exist, they report the same JSON as their halves, and the pair is compared by
  `check-twins`. `DRIVER-09`, closed the day it was written.
- ⭐ **The first guard mutated under the new rule**: the tool and its check
  were copied into the ignored scratch directory and the copy was mutated
  twice, each mutation leaving every case refused so that neither could reach a
  purge. The check reported 3 problems and exit 1 both times.
- ⚠ **`DRIVER-08` stays open and `DRIVER-10` was written rather than
  started**: the three browser families beyond Chrome that the matrix names.
  ⛔ The purge and the install have never run on a runner, so the tool has a
  proved refusal path and an unproved success path.

### 2026-09-02T06:50:00Z - more than one source, a manual path for every job, and a diff

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-06` and `CI-08`,
[`TODO/validator.md`](TODO/validator.md) `VALID-06`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`check-sources`, both halves**, asserting that every source carries its
  own answer or its own reason, that a silent source does not end the run, and
  that two different answers set the disagreement flag rather than one being
  preferred silently. ⛔ It does not decide which source is right.
- ⭐ **`check-manual-path`, both halves**, and a `# manual:` line on all nine
  jobs. Its first run named every one of the nine as having none, which is the
  guard seen to fail against the real tree rather than a fixture.
- ⭐ **`b-ids-validator diff`**, field by field, naming a header that moved and
  its two positions. ⛔ It says so above the list when the two captures differ
  in more than the version.
- ⭐ **The first published fact about a version change**: between Chrome
  `151.0.7922.76` and `151.0.7922.174` on `win64`, only the version string
  moved.

### 2026-09-02T05:55:00Z - the shuffle is a property, and the seed stays out of the profile

**Record:** [`TODO/schema.md`](TODO/schema.md) `SCHEMA-10` and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`Shuffle::Observed` carries `distinct_orders`**, a COUNT rather than
  the orders themselves: a profile is one connection, and carrying the others
  would fold a set of captures into one.
- ⛔ **Fewer than two distinct orders beside `observed` is refused.** A state
  that says the order differed while reporting one order is a claim its own
  field denies.
- ⭐ **`Options.expects_shuffle`**, so eight draws of one order is a finding
  for a family the caller says shuffles and passes for one nobody described.
  Whether a family shuffles is a fact about a browser, not about a connection.
- **The seed is ruled out of `browser-profile/1`** and belongs in the emitter
  support matrix: it is a property of a reproduction attempt rather than of a
  browser.

### 2026-09-02T05:30:00Z - the pin measured against a real trust anchor, and a claim refuted

**Record:** [`TODO/harness.md`](TODO/harness.md) `HARNESS-14`,
[`docs/HISTORY/README.md`](docs/HISTORY/README.md), and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **The per-launch key pin does not change what the browser sends.** On
  `ubuntu-latest` with Chrome `151.0.7922.173`: 19 TLS fields compared against a
  root installed in the store the browser reads, 0 differing, 2 not comparable
  because they carry a per-connection draw. ⚠ One platform, one build, one day.
- ⛔ **An inherited claim is refuted.** "Chrome on Linux does not read the
  user's NSS database for server authentication" was carried since 2026-08-31;
  a root added there completed 2 handshakes and 2 HTTP/2 connections. ⚠ And it
  is not reliable: a second round accepted no connection at all.
- ⛔ **Windows is unmeasured and the script says so**, exiting 2 rather than
  reporting a comparison with one side missing.
- **0 roots left in the store afterwards**, read back rather than assumed.

### 2026-09-02T04:45:00Z - the technical reference exists, and the conflict rule names it

**Record:** [`TODO/docs.md`](TODO/docs.md) `DOC-01` and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **[`docs/architecture.md`](docs/architecture.md)**: what a profile is,
  what each of the five components does to one and what it is not, the state a
  capture passes through as a diagram, what is published against what is
  derived, and the limits stated rather than left to be discovered.
- **The conflict rule names it**, and the interim wording it replaced is in
  [`docs/HISTORY/README.md`](docs/HISTORY/README.md) with its date rather than
  deleted. Its three exceptions survived the replacement deliberately.
- ⛔ **The router said the corpus holds one profile.** It holds three.

### 2026-09-02T04:35:00Z - the trust anchor apparatus, and a search that narrowed itself

**Record:** [`TODO/harness.md`](TODO/harness.md) `HARNESS-14`,
[`TODO/corpus.md`](TODO/corpus.md) `CORPUS-05`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`experiments/50-trust-anchor.sh` and `trust-anchor.yml`.** The script
  refuses to install a root unless `B_IDS_DISPOSABLE=1` says the machine is
  thrown away afterwards, and the workflow is the only place that is set. The
  removal is read back rather than assumed, and a root left behind fails the
  run whatever the comparison said.
- ⭐ **`CORPUS-05` closes with a measurement it did not expect.** `0x12e0` is
  absent from all three Chrome `151` profiles this project has captured, on
  both platforms, while the origin's `152` capture carries it: a codepoint
  added in one release rather than one hidden somewhere in an engine.
- ⛔ **The extension is still not named**, and what was searched is written
  down so the next attempt does not repeat it. The engine source is not a tree
  this project keeps, and a claim about a repository is not written until it is.

### 2026-09-02T04:20:00Z - staleness is a schedule, and its output carries the replacement

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-02` and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`check-staleness`, both halves**, comparing what
  `b-ids-driver versions` reports against the corpus's own pointer file. Every
  stale row names the route, the build it holds, the build that is serving, the
  rollout fraction and every source that answered.
- ⭐ **`.github/workflows/staleness.yml`**, on a schedule and never on a push,
  read-only. `CI-04` is the entry that opens a pull request from it.
- ⛔ **A defect this found in itself**: `--json` printed `"stale":1` and
  exited 0, because only the human branch carried the exit. Both halves carry
  it in both modes now.
- ⛔ **The version ordering is numeric per component**, so `151.0.7922.9` is
  behind `151.0.7922.76`. The fixture corpus holds that build on purpose.
- ⚠ **`check-exit-codes` counted tracked scripts only**, so a script that had
  never been staged escaped it. It reads untracked-not-ignored too now, and the
  count went from 22 to 23 the moment it did.

### 2026-09-02T04:00:00Z - the multipart boundary, as a pattern rather than a value

**Record:** [`TODO/schema.md`](TODO/schema.md) `SCHEMA-11` and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`http.multipart_boundary`**, carrying a literal prefix, the length of
  the random part and its alphabet, with a matcher that checks all three. The
  boundary is drawn per request like a GREASE codepoint, so one captured value
  would publish a constant no browser has.
- ⛔ **The field is absent in every profile and both fixtures, and a test
  holds that.** The two patterns are inherited by reading somebody else's
  client, and nothing inherited is published as data.
- **The schema bounds `random_len` at 1**, because a zero-length random part
  records a constant, which is what the field exists to avoid.

### 2026-09-02T03:45:00Z - every script answers 2 for a state it cannot act on

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-07` and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⛔ **Every PowerShell check answered 1 where its POSIX twin answered 2**,
  22 pairs of 22. `pwsh -File` reports a parameter-binding failure as 1, and 1
  is this code base's "it ran and the thing failed". Every `param()` block
  carries a remaining-arguments parameter now.
- ⭐ **`check-exit-codes`, both halves**, registered in both gate halves and
  with a row in the twin comparison. The gate runs 25 checks rather than 24.
- ⚠ **`$Rest` collided with a local `$rest`** in `check-markers.ps1` and took
  it from a clean run to a type-conversion error. The parameter is
  `$UnboundArguments`, checked by grep across every script.

### 2026-09-02T03:20:00Z - four entries close: bounds, credentials, reachability and trust routes

**Record:** [`TODO/schema.md`](TODO/schema.md) `SCHEMA-13` and `SCHEMA-14`,
[`TODO/validator.md`](TODO/validator.md) `VALID-03`,
[`TODO/driver.md`](TODO/driver.md) `DRIVER-04`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`SCHEMA-13`: the published schema bounds every integer field.** A
  profile claiming 999 in a byte-wide field satisfied the contract this project
  publishes and failed the one it implements. The bounds are derived in the test
  from the Rust widths, and a field added without one fails.
- ⭐ **`SCHEMA-14`: a credential is recorded as present, in its wire
  position, with no value.** It was dropped entirely before, so a recorded
  header order closed over a gap nothing marked. ⛔ Three refusals were added
  and no way to record a value was.
- ⭐ **`VALID-03`: `unreachable_dimensions`**, which walks every browser,
  channel and platform the corpus carries and reports each one no resolver
  branch can select. It reads `Family::all` rather than a list of its own.
- ⭐ **`DRIVER-04`: `experiments/40-trust-paths.sh`**, which reports which
  trust route completes a handshake on the platform it ran on. ⛔ The negative
  control is the finding: with no flag at all, four connections completed zero
  handshakes.
- **`b-ids-driver --disable-verification` and `--log PATH`.** The first is a
  capture tool and never something to ship in a client, and it is refused
  beside a pin. The second exists because an `edge` lane exited after 1.4
  seconds and its own output had been discarded.

⚠ **Seven existing tests were rewritten rather than removed**, and
[`TODO/schema.md`](TODO/schema.md) `SCHEMA-14` names each and says what it
asserts now.

### 2026-09-02T02:45:00Z - the linux64 profile lands, and the matrix column reaches the driver

**Record:** [`TODO/corpus.md`](TODO/corpus.md) `CORPUS-02`,
[`TODO/driver.md`](TODO/driver.md) `DRIVER-01`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **The corpus holds three profiles**, two of them captured on machines
  nobody owns. `chrome/stable/linux64/151.0.7922.173` from `ubuntu-latest` is
  the first, and the run that produced it reported `1 cold, 0 resumed,
  6 further cold, 1 abandoned` where the two runs before the resumption fix
  reported `0 cold`.
- ⛔ **The capture matrix's `browser` column reached nothing.**
  `b-ids-driver drive` took the first family that resolved, and the capture
  script wrote the literal `Chrome` into every identity file, so an `edge` lane
  would have published Chrome under Chrome's route inside an artefact called
  `edge`.
- ⭐ **`b-ids-driver --browser NAME`**, for `resolve` and for `drive`. A name
  with no branch is refused and names what it knows; a family this machine
  lacks exits 2, because no browser here and the capture failed are different
  facts.
- ⭐ **`Family::vendor_name`, reported as `Resolved.name`**, so the capture
  script reads the spelling a profile records instead of carrying its own
  table. A test asserts it lower-cases to the family, which is what the corpus
  route is built from.
- **`edge/stable/linux64` is enabled and required** in the plan, with the
  measurement that unblocked it written into its `why`: the driver resolved
  Edge `151.0.4129.101` at `/usr/bin/microsoft-edge` on the `ubuntu-latest`
  image.

### 2026-09-02T02:20:00Z - the capture matrix runs, and a lane that captured nothing says why

**Record:** [`TODO/corpus.md`](TODO/corpus.md) `CORPUS-02`,
[`TODO/harness.md`](TODO/harness.md) `HARNESS-07`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`capture.yml` has run on hosted runners**, twice, every job green. The
  `win64` lane's profile is published: Chrome `151.0.7922.174` from
  `windows-latest`, the first capture in this corpus taken on a machine nobody
  owns.
- ⛔ **`b-ids-corpus add` printed `1 cold` as a literal**, so its report claimed
  a cold connection on a run that had none, on the line above the refusal saying
  it had none. The line is `Selection::report` now, where a test reaches it.
- ⭐ **`b-ids-harness --no-resumption`**, which issues no session tickets so the
  subject cannot resume and every hello is a cold one. It is REFUSED without
  `--ca-out`, because resumption is a property of the terminator and a switch
  that reached nothing would be a flag no code reads.
- ⭐ **`captured.resumption` in the schema and in the published contract**,
  absent rather than defaulted on a profile written before the field existed,
  and read back from what the harness reported rather than typed beside the
  capture.
- ⭐ **`experiments/30-resumption-control.sh`**, the control that says the switch
  is safe: 19 TLS fields compared across three rounds, 0 differing, 2 not
  comparable because they carry a per-connection draw.
- **`compare-modes --labels A,B`**, because a driver that called a terminating
  run `raw` was a display that lies.

⛔ **Two findings about the runners, both measured rather than assumed.**
`ubuntu-latest` and `windows-latest` do not serve the same Chrome build
(`151.0.7922.173` against `151.0.7922.174`), so one build on two platforms needs
pinned acquisition. And the `linux64` lane produced no cold connection at all,
twice, because Chrome abandoned the connections that were not resumed.

### 2026-09-01T15:10:00Z - acquisition, the capture matrix, and a coverage report

**Record:** [`TODO/driver.md`](TODO/driver.md) `DRIVER-05`,
[`TODO/ci.md`](TODO/ci.md) `CI-03`, [`TODO/corpus.md`](TODO/corpus.md)
`CORPUS-02`, and [`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-driver::acquire`.** A build has more than one route, they are tried
  in order, and what answered is recorded on the profile with the digest of what
  arrived. ⛔ The artefact is never redistributed: the URL and the digest are.
- ⭐ **`.github/workflows/capture.yml`**, the fan-out, with every lane allowed to
  fail alone, a collect job that runs regardless, and a fuzz lane that overrides
  the pinned toolchain explicitly.
- ⭐ **`.github/capture-matrix.json`**, the plan, in one place. The workflow
  builds its matrix from it and `check-coverage` reads the same file to say what
  landed.
- **`check-workflows` and `check-coverage`**, both halves, both in the gate and
  both with a row in the twin comparison.
- **`captured.acquisition`** in the schema and in the published contract,
  omitted when absent so a profile written before it still serialises as it was.

⛔ **Three defects in this session's own new code**, each found by running it
rather than by reading it: an uninitialised awk variable used as a subscript is
the empty string and not zero; jq on Windows writes CRLF, which made one half's
human report disagree with its twin while the JSON matched; and the driver
cannot link the harness, so the digest is injected rather than computed.

⚠ **`CORPUS-02` is open with its blocker named.** The apparatus is built and no
lane has run: closing it needs one run of the matrix on a hosted runner and the
`linux64` profile committed.

### 2026-09-01T14:10:00Z - the assertions a push makes, and three checks that were not checking

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-01`,
[`TODO/tooling.md`](TODO/tooling.md) `TOOL-04`, `TOOL-15`, `TOOL-16`, `TOOL-17`,
and [`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-corpus validate`, and `check-validate` in both halves.** The
  coherence checks now run over what is PUBLISHED rather than over whatever a
  caller listed, and the cross-profile `shared_handshakes` runs with them. A
  second leg asserts the generator answers the same way twice, which
  `b-ids-corpus verify` structurally cannot see.
- ⭐ **`.github/workflows/validate.yml`**, on every push, on two hosts, with
  `CARGO_NET_OFFLINE` set for every assertion step and the whole history
  fetched.
- ⭐ **`check-line-endings`**, extracted from inside both gate halves, reading
  the working-tree column as well as the index one.
- **`check-twins --timings`**, and a scope taken from what it measured.

⛔ **Three checks were reporting green over questions they had not asked.**

| the check | what it was not checking |
| --- | --- |
| `check-corpus` | its history leg ran under `actions/checkout`'s default one-commit clone, so `git log --diff-filter=MDR` saw a single commit and answered "nothing was edited" on every CI run since it was written |
| the gate's line-endings filter | it read git's INDEX column alone, so a file that is CRLF on disk in a tree declaring `eol=lf` passed. It found `scripts/common/check-routes.ps1` LF on disk against its own `eol=crlf` on its first run. |
| `mine-repo` | it exited before the clone when its API route was down, so a host that could clone and not reach the API got nothing at all |

⭐ **`check-twins` costs 636 seconds now rather than 1056**, and the row that
caused it went from 431 to 54. ⛔ Measured with `--timings` before and after on
one host rather than estimated, and no pair was dropped: one was added.

⚠ **A drift reported alongside a moved tree is UNDECIDED now**, not a failure.
It reproduced itself while the entry was being worked: a stopped run's children
outlived the stop, and `check-docs` reported 774 links against 781 for two
implementations that agree exactly on a still tree.

### 2026-09-01T10:17:42Z - latest means stable, and a route check that can refuse

**Record:** [`TODO/tooling.md`](TODO/tooling.md) `TOOL-06`,
[`TODO/corpus.md`](TODO/corpus.md) `CORPUS-03`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- **`check-routes`, in both halves and in the gate.** No published file that
  carries exactly one value may end with a newline, because a consumer of one
  should never have to strip anything. The requirement came from `od -c` over
  somebody else's published dataset, not from taste.
- ⭐ **`latest` means stable and it cannot mean anything else.** The pointer
  file splits `latest`, built from stable profiles alone, from `per_channel`,
  which carries every channel under its own name. The rule is enforced by
  construction and the written file is compared against the derivation, so a
  hand-edited pointer is refused twice.

⛔ **The fixture written to prove `check-routes` could refuse found that it
could not.** `git ls-files` answers a path outside the repository with an empty
list, so both halves reported "ok, 0 files" over the file meant to make them go
red. A route tree that yields no single-value file is exit 2 now, because a
check that reports clean over nothing quietly stops applying.

⚠ **`CORPUS-03`'s own Approach asked for its three rules in two documents**, and
`check-one-home` refused the tree for it. The README owns the wording, the entry
points at it, and the amendment is recorded rather than made silently.

### 2026-09-01T09:48:20Z - a million runs at the parsers, and no panic

**Record:** [`TODO/harness.md`](TODO/harness.md), `HARNESS-09`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **One million coverage-guided runs, no crash and no timeout**, over every
  parser the harness exposes to the network. libFuzzer discovered the HTTP/2
  connection preface on its own, which is what says it reached the frame reader
  rather than bouncing off a length check.
- ⭐ **A second half that runs everywhere.**
  `cargo test -p b-ids-harness hostile` drives the same function over 6767
  mutations of the committed captures in under half a second, with no nightly
  toolchain, so the property is held by the ordinary gate rather than by a tool
  somebody has to remember.
- **`fuzz/`**, a cargo-fuzz crate excluded from the workspace for the reason the
  vendored tree already paid for.

⛔ **Windows cannot run the coverage-guided half, and three routes were
measured** rather than one being assumed. [`fuzz/README.md`](fuzz/README.md)
carries what stopped each: a missing AddressSanitizer runtime, a linker with no
section-boundary symbols, and libFuzzer's Windows shim not compiling under
mingw. The run above is from a Linux container, and the engine was left exactly
as it was found.

⚠ **A trap worth knowing before a CI job hits it.** The pinned
[`rust-toolchain.toml`](rust-toolchain.toml) applies to the fuzz crate too, so a
nightly image is not enough: rustup reads the file, fetches the pinned stable,
and `-Z sanitizer` is then refused. The toolchain is overridden explicitly.

### 2026-09-01T09:27:16Z - the build that is serving, not the one that is published

**Record:** [`TODO/driver.md`](TODO/driver.md), `DRIVER-02`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-driver versions`**, which reads the rollout fraction rather than
  the top of the list, cross-checks the automation-build index, and prints what
  the naive answer would have been beside its own.
- ⭐ **The inherited defect reproduced here, to the digit.** Highest known
  `153.0.8010.12` at fraction `0.005`, the build at full rollout
  `152.0.7977.65`, the automation index at `152.0.7977.64`, and the two
  first-party sources still one patch component apart.
  [`docs/inherited-claims.md`](docs/inherited-claims.md) section 7 carries the
  confirmation beside the reading it inherited.
- **Version ordering has one home**, `b_ids_schema::version_order`. The corpus
  and the driver were about to hold two copies of a comparison where
  `152.0.7977.9` sorts after `152.0.7977.64` if it is done as text.

⚠ **It reveals a gap in this project's own corpus.** The one profile is Chrome
`151.0.7922.76` and stable is serving `152.0.7977.65`, so the corpus is a major
behind. Nothing said so before this command existed.

⛔ **No HTTP client was added.** The command shells out to a fetcher the host
already has, one process per request, because an HTTP client crate brings its
own TLS stack into a workspace that vendors one.

### 2026-09-01T09:13:54Z - measuring did not change what was measured

**Record:** [`TODO/harness.md`](TODO/harness.md), `HARNESS-10`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **The answer.** Chrome `151.0.7922.76` offers the same hello whether or not
  the harness completes the handshake: seventeen of nineteen compared TLS fields
  agree exactly, none differ, and the two that cannot be compared carry a value
  the browser draws per connection. One browser, one build, one host, one day.
- **`b_ids_harness::modes`**, which measures each field's stability inside a run
  before comparing across runs, so a per-connection draw is reported as not
  comparable rather than as a difference.
- ⭐ **`experiments/20-compare-capture-modes.sh`**, which drives one resolved
  browser at both surfaces over several rounds.

⚠ **A second finding, which is a mode effect and is not a field.** Only a
surface that completes a handshake can produce a resumption: the raw run resumed
none of eighteen connections and the terminating run resumed eleven. That is
why the comparison excludes resumed connections and prints the counts beside the
field list.

⛔ **The trust-store question is NOT what this answered.** The record's work
order framed this entry as a pin against a real trust anchor; the entry itself
asks for the raw surface against the terminating one, and that is what closed.
Installing a root into a machine's trust store is the operator's action, and it
stays an open question with that recommendation attached.

### 2026-09-01T08:27:32Z - the corpus holds a profile

**Record:** [`TODO/corpus.md`](TODO/corpus.md), `CORPUS-01`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet; the profile is
committed on the default branch and no release and no data branch exist.

What landed:

- ⭐ **The first profile.** Chrome `151.0.7922.76` on Windows, captured by this
  project's own harness, at `corpus/v1/chrome/stable/win64/151.0.7922.76.json`
  with the `ClientHello` bytes it was read from beside it under `raw/v1/`.
- **A store that refuses rather than repairs.** `b-ids-corpus` turns the cold
  connection of a navigation into a profile, writes it once, and refuses a
  route that already holds one. The index and the latest-per-key pointer are
  derived from the tree rather than appended to.
- **`check-corpus`, in both halves and in the gate.** Its git leg asks whether
  a published file was ever modified, deleted or renamed after its first
  commit, which is the one question the working tree cannot answer.
- ⭐ **`experiments/`**, and the script that took the capture, so the run is
  repeatable rather than a transcript.

⚠ **The capture record moved to `harness-capture/4`**: a capture now carries the
instant it was accepted, because a profile's capture instant is never optional
and a reader that stamped one later would record when it read the file.

⚠ **The profile model gained `captured.trust` and `captured.switches`.** Every
capture this project has taken went through a per-launch key pin, and a corpus
that cannot say which profile was taken under which configuration cannot answer
whether the configuration changed the answer.

⛔ **The secret scan went red on the first profile and the hex rule was not
widened.** Two more narrow exclusions, both halves, and both mutation-proved: a
value under a field named `sha256`, and a line under `corpus/` or `raw/` that is
nothing but a quoted hex run. `TOOL-03` is the entry that predicted this shape
would arrive.

### 2026-09-01T05:40:00Z - the first bytes a browser put on a wire

**Record:** [`TODO/harness.md`](TODO/harness.md), `HARNESS-13`, `HARNESS-02`
and `HARNESS-05`, and [`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet, and the corpus
is still empty.

What landed:

- ⭐ **The handshake is terminated.** `--ca-out` mints a per-run authority,
  writes it, and selects a surface that completes a verified handshake and
  reads whatever the peer sends over it. Chrome `151.0.7922.76` and Edge
  `152.0.4191.53` both completed one and both reached HTTP/2.
- ⭐ **The priority block is measured**, on two browsers, on every terminated
  connection: `80000000ff` on the wire, which is exclusive, dependency 0,
  weight 255. It agrees exactly with the reading this project inherited, and
  [`docs/inherited-claims.md`](docs/inherited-claims.md) section 5 now carries
  both with the conditions of the measurement.
- **`HARNESS-02` closed.** All nine switches are implemented; `--ca-out` was
  the ninth and it had been absent rather than inert for a day.

⚠ **The capture record moved to `harness-capture/3`**: a terminated connection
records what the handshake negotiated, and a field added without a version bump
is a positional format that mis-reads silently.

⛔ **The authority was not installed into the machine trust store.** Each
browser was launched with a per-launch flag naming that one key, which is a
condition of every capture taken this way and is recorded as one. `HARNESS-10`
is the entry that measures whether it changed the answer.

⚠ **A test that asserted a refusal became a hang** when the refusal stopped
refusing, and killing it left a test binary locked. Every test that drives the
command now passes a deadline.
### 2026-09-01T04:33:47Z - the TLS terminator is vendored, and the record that keeps it honest

**Record:** [`TODO/vendor.md`](TODO/vendor.md), `VENDOR-01`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet, and no capture
has been taken.

What landed:

- ⭐ **rustls, vendored at a named commit and compiled by this tree.** 210 files
  under [`vendor/rustls/`](vendor/rustls/), 2.8 MiB, with fifteen of upstream's
  seventeen workspace members excluded and every exclusion carrying its reason
  in [`vendor/upstream.json`](vendor/upstream.json).
- **The four artefacts the vendoring practice asks for**: the manifest, the
  change record in [`patches/README.md`](patches/README.md), a derived series
  regenerated from the tree, and a scan with an offline leg in the gate and a
  network leg outside it.
- **Three tools**: `scripts/common/check-vendor.sh` with its PowerShell twin, and
  the two node helpers that fetch a pristine copy and regenerate the series.

⚠ **Five checks failed the moment the tree landed**, which is the cost
[`docs/methodology/vendoring.md`](docs/methodology/vendoring.md) names in
advance. Each now exempts the vendored trees and the series derived from them,
never the manifest beside them, and the secret scan carries the reading of all
38 hits that was done before its exemption was taken.

⭐ **The exemption boundary paid for itself in the same session**: the secret
scan refused the patch record, because a pasted cargo failure carried the
operator's home directory into a public repository.
### 2026-09-01T02:55:37Z - the half of the fingerprint above TLS

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and `HARNESS-03`,
`HARNESS-04`, `HARNESS-06`, `HARNESS-07`, `HARNESS-08`, `SCHEMA-06`.
**Deployed:** no. Nothing is published from this repository yet, and no capture
has been taken.

What landed:

- ⭐ **An HTTP/2 reader**, reached WITHOUT terminating TLS: a client with prior
  knowledge opens a cleartext connection and every frame that carries the
  fingerprint arrives before the first response. SETTINGS in arrival order, the
  WINDOW_UPDATE increment, and the priority block read as bytes rather than as
  a rendered string.
- ⭐ **An HPACK decoder**, checked against a fetched corpus of 47,142 cases
  across 446 files rather than against itself, so header order is readable.
- **The emitter's first content**, in `b-ids-emit`: a model that refuses what it
  cannot put on the wire byte for byte, beside a parser's model that keeps it.
  `HARNESS-06` has the difference in one table.
- **The connection selection rule and the sampling rule**, each with a
  committed fixture, and `--until-h2` and `--run-timeout-ms` behind them.
- ⭐ **A raw block a profile can be rebuilt from**, asserted rather than
  intended, and refused when its bytes spell out a credential the parsed fields
  dropped.

⚠ **The capture record moved to `harness-capture/2`**: the HTTP/2 half is a
new sibling of the TLS one, and the cleartext surface stopped being named for
HTTP/1.1 alone, because the peer picks the protocol.

### 2026-08-31T22:40:00Z - the schema, the validator and the capture oracle

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and `SCHEMA-01` through
`SCHEMA-05`, `SCHEMA-07`, `SCHEMA-09`, `VALID-01`, `HARNESS-01`, `TOOL-03`,
`TOOL-05`, `TOOL-10`, `TOOL-14`.
**Deployed:** no. Nothing is published from this repository yet, and no capture
has been taken.

What landed:

- ⭐ **The profile schema, written first and published**, with the Rust types
  checked against it by a validator that refuses a schema keyword it does not
  implement.
- ⭐ **Eight coherence checks and a command**, with three outcomes rather than
  two: a check that cannot run reports that rather than passing.
- ⭐ **A capture oracle** that reads a `ClientHello` off a real loopback socket,
  parses permissively, keeps the bytes whatever happens, and compares a run
  against a committed golden.
- **The gate went from 15 checks to 19 and from 13 twin pairs to 15.**

⛔ **`check-changelog` had been asserting four rules over zero entries since the
first commit**, because it reads an entry as a `### ` heading and this file
wrote them at `## `. It is fixed, this file is reshaped, and zero entries is now
a failure rather than a pass.

⛔ **The credential rule had a third door.** Capture-time filtering was gated in
two places and tested in both; deserialisation was neither, so a profile read
from a file could carry a cookie header. Found by the door sweep and closed.

⚠ **Two guards were too wide as their entries specified them**, and running them
is what said so: one reported 30 findings that were all legitimate. Neither hex
rule was widened, which was the tempting fix in both cases.

---

### 2026-08-31T14:10:15Z - the workspace exists, and the gate runs a suite

**Record:** [`TODO/tooling.md`](TODO/tooling.md), `TOOL-01` and `TOOL-02`.
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- **A Rust workspace of eight crates**, one per name the entries already use,
  with the toolchain pinned to an exact version rather than a channel. All
  libraries: an entry whose acceptance says `cargo run` adds its binary target
  with the behaviour.
- ⭐ **`scripts/common/check-msrv`, in both halves.** The declared minimum
  supported version is now held by a check that derives the floor from the
  resolved dependency graph and can compile the workspace with the declared
  toolchain, rather than by a number in a manifest.
- **The suite in both halves of the gate**, as three separately scored entries
  rather than one, and the placeholder comment that stood in for it removed. The
  gate went from 15 checks to 19.

⚠ **The suite is eight empty crates and zero tests, so it passes vacuously
today**, and both halves of the runner say so in a comment. The three entries
were each mutation-proved by planting the defect they exist to catch.

⛔ **A defect in the new check was found by running it**, and it is the shape
this project cares most about: the `--verify` guard probed `cargo` where the
compile needs `rustc`, so a toolchain installed incompletely was reported as
"the workspace does not compile" rather than "could not run". A broken host was
accusing the tree. Both halves probe both binaries now.

⭐ **And the first attempt to mutation-prove the graph comparison did not
fire**, because Cargo promotes a path dependency of a member into a workspace
member, which is exactly the set the check excludes. The fixture had to be
declared `exclude`d before it was a dependency at all. A guard recorded as
working on the strength of that first run would have been theatre.

---

### 2026-08-31T00:30:00Z - the repository is initialised

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and
[`TODO/INDEX.md`](TODO/INDEX.md) for the 77 entries this created.
**Deployed:** no. Nothing is published from this repository yet, and
[`TODO/RULES.md`](TODO/RULES.md) records that as a standing fact rather than an
omission.

What landed:

- **The methodology, the conventions and the security rules.** Every document
  that named a file this project does not have was rewritten rather than
  inherited, and three roles were deliberately left unwritten rather than
  shipped as empty skeletons.
- **The gate**, in both its POSIX shell and PowerShell halves, plus the probe,
  the record checker and its writer, the file writer, the commit helper and the
  reference fetcher. ⚠ The gate contains no test suite, because there is no
  code, and both halves of the runner say so in a comment rather than reporting
  green over an absence.
- ⭐ **A sweep of eighteen repositories**, at named commits. The trees are in
  [`references/`](references/) and the write-up is in
  [`docs/reference-sweeps/`](docs/reference-sweeps/). ⭐ One of the eighteen,
  `Azathothas/bit-cli`, is the origin every inherited value was measured in,
  rather than prior art.
- **[`docs/inherited-claims.md`](docs/inherited-claims.md)**, which records every
  value this project carries that it did not measure, each cited at a file in
  the tree it was measured in.
- **[`docs/glossary.md`](docs/glossary.md)**, with the caveat attached to each
  term rather than to the page that uses it.
- **77 work entries** across eleven categories, four of which close.
- **The 0BSD licence**, a bare README, and the repository's own router at
  [`docs/AGENTS.md`](docs/AGENTS.md).

⛔ **Four inherited claims were refuted during the reading**, before any of them
had been acted on. [`docs/HISTORY/README.md`](docs/HISTORY/README.md) lists each
with the reading that took it away. One changes what this project claims about
itself, and [`README.md`](README.md) makes the narrower claim; one was refuted by
the capture the claim was quoting.

⚠ **Two defects in this repository's own tooling were found by its own checks**
and both are recorded rather than quietly repaired. Five places described a
licence filler that was not on disk, which `TOOL-09` closes by deleting the
description; and the reference fetcher stops before cloning when one of its two
routes is down, which is `TOOL-04` and is still open.

**The design brief this repository started from was never committed.** `DOC-04`
carries the table of where each part of it went, and
[`docs/inherited-claims.md`](docs/inherited-claims.md) is what carries its
measurements now.
