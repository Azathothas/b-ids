# PROGRESS.md

⭐ **The one file every session reads first.** Where the work is, what is next,
and why. [`INDEX.md`](INDEX.md) is the list of entries and the **order lives
here and nowhere else**. [`RULES.md`](RULES.md) is the half of the record that
does not change between sessions, and [`SUMMARY.md`](SUMMARY.md) is the last
session's table, which is a snapshot rather than an authority.

⛔ Rewritten every session. It carries no history: the history is the git log and
the entries themselves. Do not add a "previous sessions" section.

⛔ Edited in the same change as the work, never as a report afterwards.

---

## State

```text
session ran      2026-09-04, attended, started 2026-09-04T06:04:28Z
baseline         gate ok: 39 passed, 1 skipped (check-twins, which --fast
                 skips), measured on this Windows host at the start. ⚠ The
                 412 tests is carried from the last session's close and was
                 not re-counted before the work began.
entries          total 107  open 0  blocked 0  done 107
corpus           TWELVE profiles, up from six. Five browsers-and-platforms
                 the matrix captured on runners, and one Firefox taken here.
                 ⭐ The first non-Chromium profiles, the first UNBRANDED
                 builds, and the first profiles carrying an acquisition
                 route, URL and digest.
published        the data branch: 405 files on origin/data, up from 237.
                 ⭐ publish.yml pushed them on this session's closing commit
                 and check-data-branch reports the branch's tree IS what this
                 corpus derives to. ⚠ No release: a pushed tag is the only
                 thing that cuts one.
gate             40 checks, 39 plus one skipped on this host. 445 tests,
                 counted from the runner rather than predicted. The closing
                 run is in SUMMARY.md.
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## ⭐ What changed about this project today

**The capture matrix had never added a profile, and every run reported
success.** Six lanes ran green, six captures were actually taken, and the run
printed `corpus=pull-request requests:0 auto:0`. ⛔ The rule that a capture
script does not write into an append-only corpus was applied to the SCRIPT on
2026-09-02 and never to the LANE that runs it, so the deliberate write the rule
asks for had nowhere to happen.

⚠ **It had been that way since the workflow was written**, and
[`corpus.md`](corpus.md), `CORPUS-02`, names which forbidden pattern it is and
what the run that exposed it printed.

⭐ **The corpus doubled the same day the lane gained its missing step.**

---

## Two entries closed, five effort points, and one worked to a measured blocker

| | |
| --- | --- |
| `DRIVER-11` `L` | the Gecko launch path. A switch table per engine, and the trust arranged where Firefox looks for it: an NSS certificate database written into the throwaway profile |
| `DOC-03` `S` | [`../SECURITY.md`](../SECURITY.md), with the threat model and a reporting route that degrades rather than lying |

⚠ **Under the twenty [`RULES.md`](RULES.md) section 10 asks for, and the reason
is stated rather than hidden.** `DRIVER-11` grew from a launch path into four
defects in code that was already green, and `CORPUS-02` grew from a merge into
a workflow that had never worked. ⛔ Both were followed rather than deferred,
which is the trade this session made.

| | |
| --- | --- |
| `CORPUS-02` | worked hard and **open**. Six of nine planned cells are captured and none is absent; the one required row with no capture is `chromium`, and its blocker is MEASURED now rather than predicted |

---

## ⛔ Findings in this session's own work, each caught by running it

| what | how it showed |
| --- | --- |
| the capture matrix had never added anything | six green lanes, six captures, `requests:0 auto:0`. Nothing ran `b-ids-corpus add` |
| a SQLite record dropped the row-id column and the file still passed an integrity check | read back with `sqlite3`: `CKA_CLASS` came back as the label and the label as the certificate, every value one column to the left |
| the launch removed the profile while the browser was still starting | Firefox reported `UnknownCA` on three connections. The parent exited at 158 ms because it was a launcher stub running an update |
| ⚠ the browser updated ITSELF between two captures in one session | 148.0.2 at 06:37Z and 154.0.1 at 06:43Z. `firefox.exe` was rewritten under the session |
| `check-data-branch` called nineteen changed aggregates a rewritten branch | adding one profile changes every index, route, format dump and config. Under that rule no capture could ever be published again |
| `check-coverage` could not see a capture outside the plan | seven profiles in the corpus, six accounted for, nothing saying so |
| `check-trust-anchors` called a measurement a defect | an unbranded build sends the root-store extension with a two-byte body, an EMPTY list, and the check reported "no identifiers" |
| the collect job kept whichever lane's index it copied last | the merged tree failed `b-ids-corpus verify`: an aggregate is a function of the whole set and was being taken from one lane |
| a published profile needs a JA4 vector and nothing derived one | five new profiles, five red tests, and the only fix was a person with a shell |
| ⛔ two digests were written into a test that nobody had computed | caught before the file was saved, by running an independent implementation. They were both wrong |
| ⛔ an exit code was read through a pipe, twice, by this session | `check-coverage --require-rows chromium` read as 0 and is 1. It is absolute 9 and it was broken by the session that was quoting it |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what else reaches the code that changed

Swept: every construction of `Launch`, because a new field with no default
breaks every caller and three tests carried one; every reader of `Driven`,
because `trust` is new and a caller that inferred it from the switch list was
already there; every path that writes into the published tree, because a new
profile moves every aggregate; and every check that reads
`corpus/v1/index.json`, because the lane rewrites it.

⛔ **Finding: the capture experiment inferred the trust configuration from a
Chromium-only switch.** `identity.json` read `spki-pin` when
`--ignore-certificate-errors-spki-list=` appeared in the switch list and
`not-applicable` otherwise, so a Gecko capture recorded `not-applicable` over a
completed handshake. ⭐ The schema refuses exactly that combination, so it would
have failed loudly rather than published, but the fix is that the driver now
NAMES the configuration it used and the script reads it.

⛔ **Finding: three checks would have refused every capture after the first.**
`check-data-branch`, `check-coverage` and `check-trust-anchors` each read the
corpus in a way that was correct for a corpus that never grew. All three are
fixed and two are mutation-proved.

**What the other passes did not look at:** whether the values are right. The
door sweep reads reachability.

### 2. The guard mutation: can the new guards actually fail

⛔ **Every mutation was made against a copy under the ignored scratch directory,
the live file restored from that copy, and the restored file compared byte for
byte before anything else ran.**

| where | planted | red |
| --- | --- | --- |
| `check-data-branch`, both halves | one byte of a published raw capture flipped | exit 1 on both, naming one changed artefact. `restored identical: yes`, and git saw no change |
| `check-coverage` | `--require-rows chromium`, a browser with no capture anywhere | exit 1, naming the row. ⚠ It read as 0 through a pipe first, which is the finding above |
| `nssdb::sqlite` | a row larger than a page | refused by name rather than truncated |
| `nssdb` | a profile that already holds a certificate database | refused, and the fixture was still there afterwards |
| `derive-ja4-vector`, both halves | the empty-list rule | both report twelve zeros rather than the digest of an empty string |

⚠ **Guards NOT mutated, and saying so is the point:** nothing planted a wrong
`CKA_NSS_CERT_SHA1_HASH` and watched Firefox refuse the authority, which is the
one guard in `DRIVER-11` that only the browser can enforce; the publishing
workflow was not mutated, because running it writes to the remote; and no
attestation exists to break.

### 3. The claim audit: which sentence is not backed by the tree

Swept: every number this session pasted, against the command that produced it;
every claim in the entries it closed; and the premise of every cell in the
capture matrix, against what a lane actually did.

⛔ **Finding: a pasted count went stale between being run and being read.**
`DOC-03`'s closure quoted `check-docs` at 1169 links, and closing the entry
added links, so the file it was in made its own number wrong. ⭐ Re-run and
corrected, with the reason written beside it.

⛔ **Finding: `DOC-03`'s ruled contact route is switched off.** Measured from
the forge: `{"enabled":false}`. The document names the route and keeps a
fallback that works either way rather than pointing a reporter at a button that
is not there.

⛔ **Finding: the `chromium` cell's premise was wrong about which half was
blocked.** It said the cell waits on acquisition. The resolver finds
`/usr/bin/chromium` on `ubuntu-24.04` and reads `151.0.7922.0` from it; the
LAUNCH is what fails, in the snap sandbox.

⚠ **What it did NOT find:** any pasted block that could not be reproduced.
Every `text` block written this session came from running the command above it.

---

## ⚠ What is in progress

⛔ **Nothing half-edited.** `CORPUS-02` is worked and open with its blocker
measured. `PUB-13` was read and not started: it removes `corpus/` from the
default branch, and starting it without finishing it would leave the tree in a
state the next session cannot tell from a finished one.

---

## Open questions for the operator

⚠ **Four, each with a recommendation attached, and none blocks the next
session.**

1. **Private vulnerability reporting is off at the forge.**
   [`../SECURITY.md`](../SECURITY.md) names it and keeps a fallback.
   ⭐ **Recommendation: switch it on.** It is one setting, it grants nothing
   away, and this session was not authorised to change repository settings.
2. **The capture lane does not derive the JA4 vector it needs.**
   `scripts/common/derive-ja4-vector` exists now and both halves agree, but a
   run still leaves the gate red until somebody runs it. ⭐ **Recommendation:
   the collect job derives every missing vector over the merged tree**, for the
   same reason it re-derives the index: it is a function of the whole set.
3. **The pull-request generator opens one branch per route and every branch
   carries every route's profiles.** Measured: five branches, one tree,
   `97248d83821e0d13bf4860a6074399938614cd22`. ⭐ **Recommendation: one branch
   per run**, titled for what the run captured. A title naming one route over a
   diff carrying five is a title a reviewer cannot act on.
4. **`firefox/stable/win64` is captured and is not a planned cell.**
   ⭐ **Recommendation: add it to the plan.** The capture exists, it is
   published, and a plan that does not carry it reports it as outside the plan
   forever.

---

## ⭐ The work order

⛔ **The operator has scoped everything below to ONE final session**, ruled
2026-09-04 at the close. ⚠ That is a scope, not a licence to skip: an entry that
cannot be closed properly is recorded open with its blocker measured, the way
`CORPUS-02` is below, rather than closed on machinery.

⚠ **Take these in order.**

1. **`PUB-13`**, the source branch, all six steps. ⛔ Verify tree-for-tree
   before removing anything, and step 6 is the CI change that step 5 makes
   necessary. ⭐ It is first because every capture from now on adds to a
   default branch the ruling says should not carry data.
2. **`CORPUS-02`**, which now needs one thing: an acquisition route serving a
   real Chromium build. ⛔ Not `--no-sandbox` on the snap.
3. **`EMIT-03`**, the vendor-and-patch of `h2`. ⚠ Re-derive the five bytes;
   that tree is MIT.
4. **`PUB-06`** with `HARNESS-11`'s residue: a raw-socket route, then the whole
   TCP half at once rather than spending a schema version on one weak field.
5. **`PUB-09`**, keyless attestation from the runner's own identity. ⛔ No key,
   no secret. ⚠ Its acceptance names a check-signing script this
   tree does not have, so the entry is a check pair as well as a workflow
   change.
6. **`DOC-02`**, whose trigger NARROWED again today: `DRIVER-11` closed without
   needing a machine change, so only `PUB-06`'s raw-socket route is left as a
   candidate. ⛔ Still not written, and still must not be a skeleton.

⭐ **Then the build-outs, largest first**: `EMIT-04`, `PUB-05`, `HARNESS-12` and
`LIB-03`. ⚠ `HARNESS-12` is the one that receives other people's traffic, and
[`../SECURITY.md`](../SECURITY.md)'s threat model already carries the section it
lands under.

⭐ **Four questions are open above, each with a recommendation attached**, and
three of them are one edit each. ⚠ The first needs the operator: a setting on
the remote that a session is not authorised to change.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

### Ruled 2026-09-04 during the session, and both overrule an earlier ruling

⛔ **Do not vendor a niche third-party tree.** `DRIVER-11` was ruled a vendored
NSS `certutil` at the start of the day; the operator overruled that mid-session
and the writer is Rust in this tree, under
[`../crates/b-ids-driver/src/nssdb/`](../crates/b-ids-driver/src/nssdb/). ⚠ The
vendoring was backed out before it reached a commit.

⭐ **`mozilla/nss` is the reference**, mined to
[`../references/mozilla__nss/`](../references/mozilla__nss/) at commit
`7db8de42431841b214b49fd2cb7122a07aa631b8` and trimmed by deletion. Every
constant in the writer is cited against it at file and line.

### Ruled 2026-09-04 at the start of the session

- **The corpus moves to a SOURCE branch.** ⭐ `PUB-13`, and a session may create
  and push it and run all six steps.
- **`EMIT-03` vendors and patches `h2`.** ⚠ `h2` is not the niche case the
  ruling above overruled.
- **`PUB-06` vendors a raw-socket route, and the TCP half lands whole.**
- **`PUB-09` is keyless attestation from the runner's own identity.**
- **A session may dispatch `capture.yml` and merge the green lanes.** ⭐ Done
  three times today, and the third is what found that the lanes added nothing.
- **`DOC-03` points at private vulnerability reporting on the forge.** ⭐ Done,
  and ⚠ the setting itself is off. Question 1 above.
- **The four build-outs are in scope**, largest first.
- **The kick-off prompt is redundancy, and the router stays standalone.**
- **The `HeadlessChrome` User-Agent is fixed by RECAPTURING.** ⚠ Still true of
  the six original profiles; the six new ones were taken the same way and
  carry it too, because headless is a condition of a runner rather than a
  choice. `DRIVER-03` records the substitution.

### Ruled 2026-09-03 by the operator

- **The publishing workflow is triggered three ways**: `workflow_dispatch`, a
  push to `main`, and a pushed tag. ⭐ Done: `PUB-10`.
- **Removing `corpus/` and `raw/` from `main` is sequenced, data branch
  first.** ⛔ Nothing has been deleted. `PUB-13` carries the last step.
- **The history reset on `main` is not yet.** ⛔ The operator's action.
- **Both `for-testing` matrix cells are enabled.** ⭐ And both have now
  captured, which had never happened.
- ⛔ **`for-testing` is a `Channel`.** ⭐ Done: `DRIVER-06`.
- **`SCHEMA-12`'s six formats are four and two.**
- ⭐ **Routes are generated only where the corpus HOLDS a value.**
- ⭐ **JA4 is implemented and no member of its extended family is.**
- ⛔ **The release job moves no git tag.**

### Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** Done: `DRIVER-08`. ⭐ And the profiles it produces carry the route,
  the URL and the digest, which landed in the corpus for the first time today.
- **The corpus carries BOTH Chromes, as separate matrix cells.**
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
- ⛔ **A guard on something irreversible is TWO conditions from two sources.**
- **The write for `CI-04` is JOB-SCOPED**, using the run's own token.
- **The first runner capture is fetched with `gh` and added by hand.**
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.**
- **Credentials are recorded as PRESENT, never as a value.**
- **The trust anchor is a job, not a machine change.** ⭐ `DRIVER-11` honours it:
  the Gecko trust is written into the throwaway profile and removed with it.
- **Header values stay names-only by default.**
- **The schema gains numeric bounds.**
- **The shuffle seed stays out of `browser-profile/1`.**
- **Commit once at the close** unless the session is genuinely at risk of losing
  work. ⚠ Four commits today, each after a unit passed the gate, because the
  session was long and each unit stood on its own.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
