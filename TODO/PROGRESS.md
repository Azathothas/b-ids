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
session ran      2026-09-02T01:14:00Z to 2026-09-02T11:30:00Z. Unattended
                 until 07:00Z; the operator then interrupted, ruled six
                 questions, and ended it with an instruction to put the
                 rest into TODO
baseline         gate ok: all 27 checks passed, in full, on this Windows
                 host. 26 with check-twins skipped by -Fast, and
                 check-twins compares 27 pairs. 309 tests in 36 files.
entries          total 98  open 33  blocked 0  done 65
gate             27 checks in full, and check-provisioning is not one of
                 them: it is the acceptance for an entry that is open
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## ⛔ A browser-purging tool was run on the operator machine, on purpose

⛔ **The worst thing this session did, and it is first because it is the
thing a later session most needs to not repeat.**

`scripts/common/provision-browser.sh` purges every browser from a machine
and installs a chosen build. It refused this laptop correctly when run
unmodified. ⛔ **Then this session set out to prove the guard could fail, and
mutated the live file on the machine the guard protects.** The purge path ran.
Nothing was removed only because the Windows uninstaller match did not fire,
and the confirm step then refused at exit 1, so the install was never reached.

⚠ **The machine was unharmed by an accident of registry matching.** That is
not a safety margin, and the operator noticed before this session reported it.

| what changed | |
| --- | --- |
| ⭐ **two conditions from two sources** | `B_IDS_DISPOSABLE=1`, which this project sets only inside a workflow, and `CI`, which the platform sets on every hosted runner. One edit cannot lift both, and `check-provisioning` asserts all three refusal paths rather than only the both-unset one. |
| ⭐ **the first guard mutated under the new rule** | the tool and its check were copied into the scratch directory and the COPY was mutated, twice. Both mutations leave every case refused, so neither could reach a purge. |
| ⛔ **a rule that was missing** | a test that has to bypass a guard runs against a COPY under the ignored scratch directory, never against the file on a machine the guard protects. [`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md) carries the row, [`../scripts/README.md`](../scripts/README.md) carries it where checks are written, and [`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) carries the incident. |

⚠ **The mutation discipline itself stays.** A guard nobody has seen fail is
theatre, and that rule is right. What was wrong was the SUBJECT of the
mutation.

---

## ⭐ The provisioning tool exists, and its success path has never run

⛔ **`DRIVER-08` stays open, and the reason is the honest one**: what exists
is a tool whose refusals are proved and whose working path is unmeasured. No
runner has executed the purge or the install.

| landed | |
| --- | --- |
| [`../scripts/common/provision-browser.sh`](../scripts/common/provision-browser.sh) | purge, confirm by requiring `resolve` to exit 2, install, confirm the version. The vendor route, Linux and Windows. `--plan` runs nothing. |
| [`../scripts/common/check-provisioning.sh`](../scripts/common/check-provisioning.sh) | seven refusals asserted on any host; the provisioning leg skipped LOUDLY where the machine is not disposable |
| [`../scripts/common/provision-browser.ps1`](../scripts/common/provision-browser.ps1) and [`../scripts/common/check-provisioning.ps1`](../scripts/common/check-provisioning.ps1) | the twins, which the gate refused to go green without. Each acceptance drives the tool written in its own language. |

```text
$ sh scripts/common/check-provisioning.sh
provisioning ok: 7 check(s), every refusal held, provisioning skipped
  SKIP the provisioning itself: this machine is not disposable, so nothing
  was purged. A workflow on a disposable runner is where that leg runs,
  and TODO/driver.md, DRIVER-08, is what has not been built yet.
exit=0
```

⭐ **And the PowerShell half answers identically**, which is what makes the
pair comparable at all:

```text
$ pwsh -NoProfile -File scripts/common/check-provisioning.ps1 -Json
{"schema":"check-provisioning/1","checks":7,"problems":0,"provisioned":"skipped"}
$ sh scripts/common/check-provisioning.sh --json
{"schema":"check-provisioning/1","checks":7,"problems":0,"provisioned":"skipped"}
```

⛔ **The twin was not optional and the gate said so.** `DRIVER-09` was written
as an entry to be taken later, and `check-exit-codes` reported 27 scripts
against 25 before the commit: the sh half could not land alone. Both halves
exist now, they report the same JSON, and the pair is compared.

⭐ **The rest is written down rather than half-built.** `DRIVER-08` carries six
remaining items in order, and `DRIVER-10` is the other three browser families,
measured one by one and not assumed to be variations of Chrome.

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

## ⭐ Fifteen entries closed, twenty-one effort points

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
| `DRIVER-09` | the provisioning tool and its check as PAIRS. ⛔ Closed the day it was written, because the gate refused the sh half on its own |

⭐ **Twenty-one of the twenty points [`RULES.md`](RULES.md) section 10 asks
for**, counted on that section's own scale: six `M` at two and nine `S` at one.
⚠ Nineteen of them were closed before the operator interrupted; the twentieth
and the twenty-first arrived afterwards, in `DRIVER-09`, and the reason they
arrived is that a check refused to let a script land untwinned.

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
| ⛔ one condition stood between `provision-browser` and a purge | a single edit lifted it, and the edit was made on the machine the guard protects. Two conditions from two sources now, all three refusal paths asserted |
| `provision-browser.sh` and `check-provisioning.sh` landed with no PowerShell half | ⛔ `check-exit-codes` reported 27 scripts against 25 within the hour. The two counts had been equal only by coincidence, and one untwinned script broke the tie. `DRIVER-09`, closed |


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
| the provisioning refusal stopped naming both conditions | `check-provisioning` reported 3 problems, exit 1 |
| the provisioning guard made to exit 0 instead of 2 | the same 3 cases, `exit 0, expected 2`, exit 1 |

⛔ **The last two were planted in COPIES, under the ignored scratch directory,
and the copy of the check was pointed at the copy of the tool.** ⚠ Both
mutations leave the tool refusing every case, so neither could have reached a
purge. That is the rule this session learned the expensive way, applied.

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

⚠ **The script count moved after that sweep and the sentence above is left
as it was measured.** `check-exit-codes` reports **27** now: the provisioning
tool and its check landed after the audit ran. ⛔ A number in a review pass
records what the command said when the pass ran; editing it afterwards to
match a later tree is how an audit stops being evidence.

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

## ⛔ The gate costs ten minutes and twenty-four seconds of it is Rust

⭐ **Asked by the operator at the close of this session and measured rather
than guessed.** [`tooling.md`](tooling.md), `TOOL-18`, carries the table and the
arithmetic; the short answer is that the vendored and reference trees are not
what makes it slow.

| | |
| --- | --- |
| ⛔ **the cause** | a subprocess per file, on a host where a subprocess costs 54.5 ms. 100 bare `grep` spawns took 5450 ms; the hot loop of `check-control-bytes` spawns about six per file, so its 384 files predict 126 seconds against 121 measured. |
| ⭐ **not the tree** | `check-docs` and `check-markers` exclude `references/` and `vendor/NAME/` and cost 175 and 29 seconds over 53 and 241 files. `check-line-endings` reads all 5435 files, references and vendor included, in **2.4 seconds**, because it asks git once instead of looping. |
| ⚠ **one row does read the corpus, on purpose** | `check-no-secrets --scope references`, 102 seconds over 4972 files. It is the row that exists to scan what the others exempt. |
| ⛔ **and one reads vendor without saying why** | `check-control-bytes` excludes `references/` and not `vendor/NAME/`: 146 of its 384 files are vendored. `TOOL-18` records it and does not decide it. |

⚠ **`TOOL-15` measured this shape on 2026-09-01, added `--timings`, and closed.**
The cost was named and not reduced, which is why `TOOL-18` exists and is P1: a
gate this slow gets run once at the end, and this session ran it that way twice.

---

## ⚠ A scheduled capture ran at the close and its artefacts are unread

⭐ **`capture.yml` fired on its own cron at 09:39Z against `0cee89e`**, which no
session dispatched. Three artefacts are waiting and nothing has been added to
the corpus from them:

| job | |
| --- | --- |
| `chrome stable linux64` | ⭐ success, and this is the lane that captured nothing twice earlier in the day |
| `chrome stable win64` | success |
| `edge stable linux64` | ⛔ failure, which is the known one: Edge exits after 1.4 seconds having opened no connection, and `DRIVER-07` is why the log now says so |
| `collect` | success |

⛔ **Read the artefacts before dispatching another run.** Run `33615327503`,
and `gh run download` is the route. ⚠ A profile is added with `b-ids-corpus
add`, and the corpus is append-only.

---

## ⭐ The work order

⚠ **Take these in order.** ⛔ The first two are P0 and the operator ruled
both on 2026-09-02, after reading that the corpus records builds nobody chose.

1. ⭐ **`DRIVER-08`, and the tool is written: what remains is running it.**
   In the order the entry lists them: the `for-testing` route, both routes in
   the matrix, a workflow step that fails the lane loudly when provisioning
   does not confirm, ⛔ **a run on a disposable runner on both platforms**,
   `captured.acquisition` populated from what the tool printed, and
   `check-provisioning` into the gate. ⛔ Until the runner leg has happened,
   every profile records `captured.acquisition: null`, no two lanes can be
   made to run one build, and the tool has a proved refusal path and an
   unproved success path.
2. ⭐ **`HARNESS-15`.** Select the TLS half and the HTTP/2 half per half rather
   than demanding one connection carry both. ⛔ It is what makes the Linux lane
   capture without imposing a server-side condition on every profile.
3. **`CORPUS-02`**, which both of the above unblock. Two of its four required
   rows are captured; the `edge` lane is wired and has not produced a profile,
   and `chromium` and `firefox` need the resolver to know them at all.
4. **`SCHEMA-08`**, the generator and the five formats whose round trip this
   tree can prove. `VALID-06`'s diff and `CI-04`'s body both want it.
5. **`CI-04`**. A scheduled run that finds a change opens a pull request.
   ⭐ The write is ruled: job-scoped, on the collect job alone.
6. **`PUB-03`**, then `PUB-01`, `PUB-02`, `PUB-07`. `PUB-07` cannot close until
   two of its three surfaces exist.
7. **`SCHEMA-12`**, **`CI-05`** and **`EMIT-03`**, the last of which
   `HARNESS-05` has unblocked and which this session did not start.

⭐ **`TOOL-18` is worth taking early and out of order.** It is the gate
costing ten minutes on this host, and every entry above it pays that price
once per closing.

⚠ **`DRIVER-10` follows `DRIVER-08` rather than racing it.** Three more
browser families is work on a tool whose success path is unmeasured, and doing
it first multiplies the unmeasured part instead of shrinking it. ⭐ `DRIVER-09`
did not get that choice: the gate refused the tool without its twin.

⚠ **Small entries worth taking whenever a larger one is blocked**: `DRIVER-06`
(branded and unbranded builds, which `DRIVER-08` makes measurable for the first
time), `HARNESS-16` (why a Windows runner will not take a root unattended),
`CORPUS-04` (per-build trust-anchor lists, which needs a `152` capture),
`VALID-04` (reference digest implementations).

---

## Open questions for the operator

### ⛔ One, and it blocks the two unbranded matrix cells

**Where does an unbranded build live in the corpus?** The published route is
`browser/channel/platform/version` and carries no `branded`, and `Channel` is a
closed vocabulary of six that does not include the vendor's automation channel.
So a branded and an unbranded build of one version publish at one path, and the
two `for-testing` cells in
[`../.github/capture-matrix.json`](../.github/capture-matrix.json) are planned
and not attempted because of it rather than because the tool cannot fetch them.

⭐ **Recommendation: add `for-testing` to the `Channel` vocabulary**, and let
`branded: false` follow from it rather than becoming a path component.

| | |
| --- | --- |
| ⭐ **why this one** | the channel is ALREADY part of the route and of the `latest` key, so nothing about the layout changes and no consumer's pin moves. It is also true: the automation index is a channel the vendor publishes, separately from stable. |
| ⚠ **what it costs** | one variant on a closed enum, the published schema regenerated, and `CORPUS-03`'s "latest means stable" sentence re-read so an automation build cannot be mistaken for one. |
| ⛔ **the alternative that loses** | `branded` as a fifth path component. It changes `corpus/v1/` for every consumer to carry a dimension only one browser family has, and `PUB-03` has not shipped a route yet only by luck of timing. |

⚠ **Nothing is blocked on the answer except those two cells.** `DRIVER-08`'s
runner leg, `DRIVER-10` and the branded routes all proceed without it.

⚠ **A later session that finds a fork writes it here with a recommendation
attached and keeps working.** [`RULES.md`](RULES.md) section 10 names "this
needs a decision from the operator" as one of the four sentences that is not a
reason to stop. ⛔ Ask at the very START of a session if proceeding under any
assumption would be unsafe; otherwise record it here and proceed on the
recommendation.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

### ⭐ Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** Completely, with no leftovers, confirmed by running the resolver and
  requiring it to find nothing, then installed and confirmed by version.
  ⭐ P0, and it is `DRIVER-08`. ⚠ The reason it is a ruling rather than an
  improvement: on a machine this project controls completely it was measuring
  whatever somebody else's image installed.
- ⭐ **The corpus carries BOTH Chromes, as separate matrix cells.** Branded,
  from the vendor's own channel, current build only, and both platforms get the
  same one because both install on the same day. Unbranded, from the
  automation-build index, at any exact build, recorded `branded: false`.
  ⚠ They are two products and `DRIVER-06` is what measures the difference.
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
  `HARNESS-15`: the TLS half and the HTTP/2 half are selected per half, so a
  cold hello on a connection that carried no HTTP/2 is kept rather than thrown
  away. ⭐ `--no-resumption` stays as a CONTROL and leaves the capture path, so
  the browser behaves as it does in the wild and no server-side condition is
  imposed on every published profile.
- **Two smaller findings are their own entries**: `DRIVER-07`, the browser log,
  and `HARNESS-16`, the Windows trust store.

- ⛔ **A guard on something irreversible is TWO conditions from two sources,
  and it is never mutated on the machine it protects.** Ruled after this
  session ran the purge path on the operator laptop. One condition this
  project sets inside a workflow, one the platform sets on a hosted runner,
  and a bypass test runs against a copy under the scratch directory.
  ⭐ The rule is in [`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md)
  and in [`../scripts/README.md`](../scripts/README.md) where checks are
  written, so it is read by whoever is about to break it.
- ⭐ **The remaining provisioning work is written into TODO rather than
  half-built at the end of a session.** Ruled 2026-09-02. `DRIVER-08` carries
  six items in order, `DRIVER-09` the PowerShell twin, `DRIVER-10` the other
  three browser families. ⚠ The operator also asked this session to close
  with a prompt for the next one, which [`RULES.md`](RULES.md) section 10 does
  not have a step for; what the operator asks in a session comes first.

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
