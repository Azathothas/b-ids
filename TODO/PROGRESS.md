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
session ran      2026-09-02T01:14:00Z to 2026-09-02T07:00:00Z, unattended,
                 ended by operator interrupt
baseline         the gate passes: 26 checks on this Windows host with
                 check-twins skipped by -Fast, and check-twins compares 26
                 pairs when run in full. 309 tests in 36 files.
entries          total 91  open 29  blocked 0  done 62
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## The capture matrix has run, and it produced two findings before it produced a profile

**`capture.yml` ran on hosted runners for the first time**, four times in all,
every job green. Dispatched with the authenticated `gh` on the default branch,
which is the route the operator ruled 2026-09-01.

### What the runners actually served

| | |
| --- | --- |
| `ubuntu-latest`, image `ubuntu24/20260823.283` | Chrome `151.0.7922.173`. Edge `151.0.4129.101` also resolved, at `/usr/bin/microsoft-edge` |
| `windows-latest`, image `win25-vs2026/20260824.214` | Chrome `151.0.7922.174` |

⛔ **The two runners do not carry the same Chrome build**, so two profiles of one
build on two platforms is not obtainable from the preinstalled browser at all.
It needs pinned acquisition, which is `DRIVER-05`'s route rather than the lane's.

### ⛔ The `linux64` lane captured nothing, twice, and the reason is resumption

⚠ **Reproduced rather than raced.** Chrome on `ubuntu-latest` abandoned the
connections that were not resumed and resumed every one it kept, so the
navigation had no cold connection and there was nothing to publish. More
connections do not help: the first completed handshake leaves a ticket and
everything after it resumes.

⛔ **And the report said `1 cold` on the line above the refusal saying there was
none.** `b-ids-corpus add` carried the word behind a hardcoded `1` in a format
string. It is `Selection::report` now, where a test reaches it, and the guard was
seen to fail at exit 101 against a mutation that restores the literal.

`b-ids-harness --no-resumption` issues no session tickets, so every hello is a
cold one, and `experiments/30-resumption-control.sh` is the control: three
rounds, 19 TLS fields, 0 differing. The switch changes which connections are
cold, not what a cold hello is.

### What the corpus holds now

⭐ **Three profiles, and two were captured on machines nobody owns**: Chrome
`151.0.7922.174` on `win64` from `windows-latest`, and Chrome `151.0.7922.173`
on `linux64` from `ubuntu-latest`. `corpus=profiles:3 problems:0`, `findings:0`.

---

## ⭐ Fourteen entries closed, nineteen effort points

| | |
| --- | --- |
| `SCHEMA-13` | every integer field in the published schema carries a bound derived from its Rust width |
| `SCHEMA-14` | a credential is recorded as PRESENT, in its wire position, with no value. It was dropped entirely before, so the order closed over an unmarked gap |
| `SCHEMA-11` | `http.multipart_boundary`, as a PATTERN: prefix, random length and alphabet |
| `SCHEMA-10` | `Shuffle::Observed` carries `distinct_orders`, and the seed is ruled out of the profile |
| `VALID-03` | `unreachable_dimensions`, over every browser, channel and platform the corpus carries |
| `VALID-06` | `b-ids-validator diff`, field by field, which says so when two captures differ in more than the version |
| `DRIVER-04` | `experiments/40-trust-paths.sh`. The negative control is the finding: with no trust flag at all, four connections completed zero handshakes |
| `HARNESS-14` | the pin measured against a real trust anchor on a disposable runner: 19 fields, 0 differing. It refutes an inherited claim |
| `CI-02` | `check-staleness`, both halves, plus a scheduled `staleness.yml` |
| `CI-06` | `check-sources`, both halves: per-source isolation, a silent source that does not end the run, a disagreement flagged rather than resolved |
| `CI-07` | `check-exit-codes`. Every PowerShell check answered 1 where its POSIX twin answered 2, 22 pairs of 22 |
| `CI-08` | `check-manual-path`, and a `# manual:` line on all nine jobs |
| `CORPUS-05` | the search is recorded and re-runnable. `0x12e0` is absent from all three Chrome `151` profiles here and present in the origin's `152` capture |
| `DOC-01` | [`../docs/architecture.md`](../docs/architecture.md), the technical reference |

⚠ **Nineteen of the twenty points [`RULES.md`](RULES.md) section 10 asks for**,
counted on that section's own scale: five `M` at two and nine `S` at one. The
twentieth was not reached before the operator ended the session, and the count
is recorded rather than rounded.

---

## ⛔ Findings in this session's own code, each caught by running it

| what | how it showed |
| --- | --- |
| `b-ids-corpus add` printed `1 cold` as a literal | beside a refusal saying there was none |
| every PowerShell check exited 1 where its twin exited 2 | 22 pairs of 22, measured before the check was written |
| `check-staleness --json` exited 0 over a stale corpus | only the human branch carried the exit |
| `check-exit-codes` counted tracked scripts only | a script never staged escaped it; 22 became 23 the moment it read untracked ones |
| the twin comparison read a catch-all parameter as a flag | the doctor pair reported a drift in one place |
| `50-trust-anchor.sh` hung on both platforms | a certificate tool asked for a password from a terminal that is not there |
| it then handed `certutil.exe` an msys path | the Windows lane could not install and exited 2, which was correct |


---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what other door reaches this code

Swept, by grep rather than from memory: every caller of `Selection::report` and
`cold_count`, every call of `Authority::server_config`, every construction of a
`HeaderSet`, every reader of `is_never_recorded`, every caller of `check_grease`,
every construction of `Shuffle` and of `Launch`, and every registration of the
four new checks.

⛔ **Finding: the credential rule has FOUR doors and the entry named three.**
`SCHEMA-14` said the change reaches the model, the capture path and the
validator. The fourth is the HTTP/1.1 reader in `listener.rs`, which had its own
`continue` past a credential and would have gone on closing the recorded order
over an unmarked gap on the cleartext surface. Found by grepping for the filter
rather than by reading the entry.

⛔ **Finding: nothing in the capture path can produce a shuffle observation.**
`b-ids-harness/src/hello.rs` writes `Shuffle::Unknown` unconditionally, so the
`distinct_orders` field `SCHEMA-10` added is a field the model can carry and the
capture path never fills. Recorded in that entry rather than left for a reader to
discover from an always-`Unknown` column.

⭐ **Confirmed, by counting:** three call sites of `server_config` and every
one passes the resumption argument; four readers of `is_never_recorded` and every
one keeps the name; ten callers of `check_grease` and every one passes
`Options`; `check-exit-codes` and `check-manual-path` each appear in both gate
halves, in `COMPARED_DIRECTLY` and in `check-twins`; `check-staleness` and
`check-sources` appear in `check-twins` and deliberately in neither gate, because
without a fixture they reach the network.

⭐ **What the other passes did not look at:** the callers. Both of the others
read what was written; this one grepped for what was not enumerated.

### 2. The guard mutation: can the new guards actually fail

Planted and read unpiped, each in the half where the guard lives:

| planted | what went red |
| --- | --- |
| `cold_count` returned to a literal `1` | `connection_selection_reports_no_cold_connection_when_every_one_resumed`, exit 101 |
| `"maximum": 255` removed from `$defs/u8` | two `bounds` tests, exit 101 |
| `check-msrv.ps1` made to exit 1 for an unknown argument | `check-exit-codes.ps1` named the script, exit 1 |
| `distinct_orders < 2` weakened to `< 0` | `shuffle_observed_with_one_order_is_refused`, exit 101 |

⭐ **And two guards were planted by the tree rather than by hand.**
`check-manual-path` was written before the lines it looks for and named all nine
jobs on its first run; `check-sources` and `check-exit-codes` each carry a
refusal fixture they run on every invocation and exit **2** rather than reporting
anything if it comes back clean.

⚠ **Guards NOT mutated, and saying so is the point:** `check-staleness`'s
version ordering was exercised against a fixture holding `151.0.7922.9` rather
than by mutating the comparison; `check-manual-path`'s "script does not parse"
branch has never been seen to fire; and the `withheld`-on-an-ordinary-header
refusal is asserted by a test but was not planted against the capture path.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every pasted block in the fourteen closings, this file,
the changelog, and the four documents the work made stale.

⛔ **Finding: a count that was wrong when written.** This file said "Eleven
entries closed" over a table of fourteen rows, with a sentence underneath trying
to reconcile the two. Recounted from `INDEX.md`: fourteen entries, nineteen
points, five `M` and nine `S`.

⛔ **Finding: a repetition of a measurement that had not been taken.**
`HARNESS-14`'s closing said `certutil -addstore` failed "twice, after the path
was corrected". Only one Windows run happened after that correction. Corrected to
one, naming the run.

⛔ **Finding: three documents describing a corpus of one profile.**
[`../README.md`](../README.md), [`RULES.md`](RULES.md)'s standing-facts table and
[`../docs/AGENTS.md`](../docs/AGENTS.md) all said so, each written the day the
first landed. All three now say three and name where the count is read from.

⛔ **Finding: a refuted-claim count off by one.** `RULES.md` rule 2 said four
inherited claims had been refuted and that none fell to an experiment. The fifth
did, this session, and it is the first this project has taken down that way.

⭐ **Claims checked that stood, by re-running the command:** `profiles:3
problems:0` and `findings:0 notcheckable:9`; 309 tests in 36 files; 25 scripts
answering 2; 9 workflow jobs each naming a manual equivalent; 26 twin pairs, all
agreeing; 19 reference trees.

⚠ **And one procedural failure of this session's own, recorded rather than
quietly fixed.** [`RULES.md`](RULES.md) warns not to edit the tree while
`check-twins` runs, because it reads the tree before and after. This session
edited `PROGRESS.md` during the final run. The run reported agreement and its
counts match the tree as it now stands, so the verdict is good; ⛔ that is luck
rather than method, and the rule was there first.

---

## ⚠ Seven red `ci` runs on superseded commits, and why they are not rerun

⭐ **The remote is green at `HEAD` and at its parent**, both jobs of both
workflows. ⚠ Seven intermediate runs are red, and the next session should not
chase them:

| commits | failing job | cause |
| --- | --- | --- |
| `bb0f881`, `7346e62`, `a9639c3`, `02ee3fa`, `7144f09`, `dd63a35`, `a5eb7d5` | `gate (ubuntu)` | ⛔ a twin drift this session introduced: the doctor comparison read the new catch-all PowerShell parameter as a command-line flag. Fixed in `27faa3a`. |
| `a9639c3` also | `gate (windows)` | the standing toolchain flake, [`RULES.md`](RULES.md) section 8.5 |

⛔ **Rerunning them would reproduce the failure**, because the tree at those
commits genuinely carried the drift. The fix is a later commit, not a rerun, and
a green rerun of a tree that was broken would be the lie.

---

## What is in progress

⛔ **Nothing is half-edited.** Every entry this session touched is closed in
place with its acceptance command run, or left open with its blocker named.

---

## ⭐ The work order

⚠ **Take these in order.**

1. **`CORPUS-02`**, continued. Two of its four required rows are captured:
   - **the `edge` lane**, enabled and wired and not yet producing. Its last run
     resolved Edge and the browser exited after 1.4 seconds having opened no
     connection; `--log PATH` now records what it says.
   - **`chromium` and `firefox`** need `b_ids_driver::Family` to have branches
     for them at all.
2. **`SCHEMA-08`**, the generator and the five formats whose round trip this
   tree can prove. `VALID-06`'s diff and `CI-04`'s body both want it.
3. **`CI-04`**. A scheduled run that finds a change opens a pull request.
   `CI-02` is closed and its output already carries the replacement values.
   ⭐ The write is ruled: job-scoped, on the collect job alone.
4. **`PUB-03`**, then `PUB-01`, `PUB-02`, `PUB-07`. `PUB-07` cannot close until
   two of its three surfaces exist.
5. **`SCHEMA-12`**, the six formats that need a decoder, once `SCHEMA-08` has a
   generator to extend.
6. **`CI-05`**, the cold-start job, and **`EMIT-03`**, which `HARNESS-05` has
   unblocked and which this session did not start.

⚠ **Small entries worth taking whenever a larger one is blocked**: `DRIVER-06`
(branded and unbranded builds), `CORPUS-04` (per-build trust-anchor lists, which
needs a `152` capture because no `151` profile carries the extension),
`VALID-04` (reference digest implementations).

---

## Open questions for the operator

⚠ **Each carries a recommendation and each was proceeded on**, per
[`RULES.md`](RULES.md) section 10: none made proceeding unsafe, and none of them
edited a published profile.

### 1. Should refusing session tickets be the standing capture configuration?

**It already is**, because the alternative was a lane that cannot capture.
`experiments/10-first-profile.sh` passes `--no-resumption`, and every profile
written from here records `captured.resumption: refused`.

⭐ **The recommendation is to keep it.** The corpus publishes the cold hello and
nothing else, the control measured 0 differing fields of 19, and the switch
defaults off so the resumed-connection sample stays reachable.

⚠ **What it costs**: `HARNESS-07` says a resumed connection is recorded
separately as its own profile, and nothing captured under this switch can ever
produce one.

### 2. Should the driver's `--log` have been its own entry?

⛔ `b-ids-driver drive` gave the browser `Stdio::null()`, so a lane that
captured nothing carried no word from the browser about why, and the `edge` lane
made that concrete. It is implemented under `CORPUS-02`, because the edge row
that entry requires cannot be diagnosed without it.

⭐ **The recommendation is no.** It is one switch, it unblocks a row of an open
entry, and a `DRIVER-07` covering it after the fact would be a record of a
decision rather than a unit of work.

### 3. Windows cannot exercise the trust-store route, and nobody has read why

⛔ `certutil -addstore -user Root` returned non-zero on `windows-latest` under
a bounded call with its stdin closed, after the path was corrected. What is
MEASURED is that the command did not succeed there; why it did not is a reading
nobody has taken.

⭐ **The recommendation is a small entry of its own**, because it is a platform
question rather than a defect in `HARNESS-14`, whose script correctly refused to
report a comparison with one side missing.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

- ⭐ **The write for `CI-04` is JOB-SCOPED.** `contents: write` and
  `pull-requests: write` on the collect job alone, using the run's own
  `GITHUB_TOKEN`. ⛔ Never a personal access token. ⚠ Every capture lane keeps
  `contents: read`. ⚠ It also needs the repository setting that lets Actions
  create pull requests, which is the operator's to enable.
- ⭐ **The first runner capture is fetched with `gh` and added by hand.** Done,
  three times, this session.
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.** ⛔ That is the OPERATOR'S action at a time of their choosing and
  it licenses nothing for a session: no force push, no history rewrite, and the
  corpus stays append-only in every change an agent makes.
- **`SCHEMA-08` is SPLIT.** It keeps the generator plus the five formats whose
  round trip this tree can prove: JSON, NDJSON, CSV, TSV and Markdown.
  `SCHEMA-12` carries YAML, TOML, SQLite, CBOR, MessagePack and Protobuf.
- **Credentials are recorded as PRESENT, never as a value.** `SCHEMA-14`, closed
  2026-09-02.
- **The trust anchor is a job, not a machine change.** `HARNESS-14`, closed
  2026-09-02 on a runner that was thrown away.
- **Header values stay names-only by default.** Corpus captures turn them on
  deliberately.
- **`CORPUS-04` publishes the per-build trust-anchor list and states all three
  options with their costs.** ⛔ It asserts no preference.
- **The schema gains numeric bounds.** `SCHEMA-13`, closed 2026-09-02.
- ⭐ **The shuffle seed stays out of `browser-profile/1`.** Ruled 2026-09-02 in
  `SCHEMA-10`: it is a property of a reproduction attempt rather than of a
  browser, and it belongs in the emitter support matrix.
- **Commit once at the close** unless the session is genuinely at risk of losing
  work. ⚠ This session pushed nine times, because a capture lane runs the tree
  on the default branch and could not be dispatched otherwise.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
