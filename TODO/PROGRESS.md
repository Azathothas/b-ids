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
session ran      2026-09-04, attended, started 2026-09-04T00:07:59Z
baseline         gate ok: 38 passed, 1 skipped (check-twins, which --fast skips)
                 on this Windows host at the start. 401 tests.
entries          total 107  open 12  blocked 0  done 95
published        the data branch: 200 files on origin/data. ⚠ BEHIND by 37:
                 PUB-04 added a configs/ tree the publisher has not pushed yet.
                 Nothing published is wrong. ⚠ No release: a pushed tag is the
                 only thing that cuts one.
gate             check-generated-configs joined it, taking the gate from 39
                 checks to 40. 412 tests. The closing run is in SUMMARY.md.
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## ⭐ What changed about this project today

**The Windows CI failure was this repository's own, and the record said it was
not.** [`RULES.md`](RULES.md) section 8.5 had called it a runner fault for three
sessions and prescribed a rerun. ⛔ The cause is that `rustc` and `cargo` are
rustup PROXIES, so this project's own probe, run in a tree pinning a toolchain
the runner does not have, STARTED installing it and then killed the install at
its six-second limit. The conflict the job reported was a fragment of that.

⭐ **A probe measures a machine. It does not change one.** That is the rule
section 8.5 carries now, and `doctor --fixture` is the command that holds it.

---

## Six entries closed, fourteen effort points, and two left open with measured blockers

| | |
| --- | --- |
| `CI-09` `M` | the Windows toolchain failure, traced to this tree's own probe and fixed from two sources |
| `PUB-11` `M` | ten of ten with the corpus moved out, plus two checks that were passing by comparing something to itself |
| `PUB-04` `M` | thirty-seven generated config files, twenty-four of them refusals naming a hole at a file and a line |
| `PUB-14` `M` | the data branch check could not tell a branch that is BEHIND from one that is WRONG, and the gate mistreated its designed exit 2 |
| `VALID-05` `L` | the conformance suite: a field-level diff with a third verdict for what a browser varies per connection |
| `HARNESS-11` `M` | the TCP layer, and the capability answer is one field of six |

⚠ **Under the twenty [`RULES.md`](RULES.md) section 10 asks for**, and the
reason is stated rather than hidden: every remaining open entry needs an
operator ruling, a capability this host does not have, or is a large new build.
Two were worked to that point rather than left untouched.

| | |
| --- | --- |
| `CORPUS-02` | worked and **open**. The resolver knows all four families now; the blocker moved from the resolver to the launcher and to acquisition, and both are measured |
| `EMIT-03` | worked and **open**. Its measurement is in and unanimous; what it needs now is a ruling, below |

---

## ⛔ Findings in this session's own work, each caught by running it

| what | how it showed |
| --- | --- |
| the probe was the installer | a `rustc --version` in a tree pinning an absent toolchain was killed at 6068 ms mid "downloading 5 components", leaving a half-written toolchain the next install refuses |
| `check-data-branch` compared the published branch against a copy of ITSELF and reported green | driven with `corpus/` moved out: `data branch ok: 200 file(s) regenerated`, exit 0. The export on the line ABOVE its guard disarmed the guard |
| `check-corpus` asked THIS repository's history about files that are not in it | the same cause, one line apart. It passed for the wrong reason rather than failing |
| a deleted published path read as BEHIND rather than as a rewrite | planted on a throwaway branch. The branch's own manifest is what tells the two apart, and it was not being read |
| four tests used `firefox` as the example of an impossible family | all four went red on the change that made it possible, which is the most useful thing they could have done |
| the conformance tool forgave a GREASE value MOVED to another position | it stripped GREASE before comparing rather than masking it in place. The test written to catch it did |
| `VALID-05`'s acceptance command named a profile the corpus has never held | the tool refuses it and names what it does hold, which is how it was found |
| the probe reported `rustc 1.75.0` for a compiler that could not run | the version was parsed out of an error message. A non-zero exit is not a version |
| `jq` on Windows writes CRLF, again | a new leg reported all 198 artefacts missing. `CORPUS-02` recorded that same defect against that same tool on 2026-09-02 |
| a Python `open(p,'w')` on Windows rewrote one `.rs` file to CRLF | caught by comparing every touched file against `.gitattributes`. It is why this project mandates `write-file.mjs` |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what else reaches the code that changed

Swept: every `Family::all()` site, because widening an enum from two variants to
four changes behaviour at every ITERATION without a compile error; every caller
of `index_route`, which changed shape; every check that asks the corpus
resolver a second question; and every string that maps a family name to
something, because those are the family lists the compiler cannot see.

⛔ **Finding: `b_ids_validator::vendor_brand` is a second family list**, in
shipped code, and it knows four names that are not the resolver's four. ⭐ It is
protected by a guard one line above it, which declines the check when no
`sec-ch-ua` value was recorded, so the hole is not reachable.

⛔ **Finding, and this one was live: that guard reported THREE different facts
with one message.** A browser that sends no `sec-ch-ua` at all is signal, and
Firefox is one of those; a header recorded under the names-only policy is a gap
in the capture. Both said "no VALUE was recorded". Fixed at three call sites and
mutation-proved.

⛔ **Finding: the gate can now record a skip that CI refuses.** `PUB-14` made
`check-data-branch`'s designed exit 2 a skip, and the ubuntu job runs `--strict`
while the windows job allows exactly one skip. Neither bites until `PUB-13`
removes `corpus/`, and `PUB-13` carries it as a step.

**What the other passes did not look at:** reachability. The claim audit reads
sentences and the guard mutation reads guards.

### 2. The guard mutation: can the new guards actually fail

⛔ **Every mutation was made against a copy under the ignored scratch directory,
the live file restored from that copy, and the restored file compared byte for
byte before anything else ran.** ⚠ The branch mutations were made on a throwaway
worktree and a local branch, both removed; `origin/data` was never written to.

| where | planted | red |
| --- | --- | --- |
| `doctor.sh`, `doctor.ps1` | `RUSTUP_AUTO_INSTALL=0` changed to `=1` | both, exit 1, naming the proxy's own `syncing channel updates` line |
| `check-generated-configs` | the generator writes a `.rs` snippet for every hole stack | exit 1, 24 problems, each naming the stack and the path |
| `check-data-branch` | a published artefact's bytes changed | exit 1: a published artefact is immutable |
| `check-data-branch` | a published path deleted from the branch | exit 1 only after the manifest leg existed. ⭐ Before it, this read as BEHIND, which would have turned a rewritten branch green |
| `check-data-branch` | the canonical corpus moved out of the working tree | the gate records a SKIP where it previously recorded a failure |
| `b_ids_validator::why_no_value` | the two absences collapsed into one message | exit 101, the coherence suite |
| `b_ids_conformance` | GREASE stripped rather than masked | exit 101, on the test written for it |

⚠ **Guards NOT mutated, and saying so is the point:** nothing exercised the
publishing workflow, because running it writes to the remote; the `for-testing`
capture lane has still never run; no capture was taken this session; and
`TcpObservation::every_absence_explained` is asserted true and has never been
seen false.

### 3. The claim audit: which sentence is not backed by the tree

Swept: every number this session wrote, against the command that produces it;
every count carried forward from the previous session; and the premise of every
entry the work order named, against what the corpus and the tree actually hold.

⛔ **Finding: three premises were disproved**, each stated as measured and each
false by the time it was read.

| where | said | is |
| --- | --- | --- |
| [`RULES.md`](RULES.md) section 8.5 | the Windows failure is the runner's | it is this tree's own probe |
| `EMIT-03` | blocked on a measurement taken here | the measurement is in, six of six profiles carry the block |
| `DOC-03` | this repository publishes nothing yet | the data branch has carried a tree a consumer fetches since 2026-09-03 |

⛔ **Finding: a live reference page carried a count that had moved.**
[`../scripts/README.md`](../scripts/README.md) said twelve checks read the
corpus; it is thirteen pairs now. ⭐ It reads the number with a command instead
of writing one.

⚠ **What it did NOT find:** any pasted output that could not be reproduced. Every
`text` block written this session was produced by running the command above it,
and the ones that could not be re-run say so.

---

## ⚠ What is in progress

⛔ **Nothing half-edited.** `CORPUS-02` and `EMIT-03` are worked and open, each
with its blocker measured and named in its own entry.

---

## Open questions for the operator

⛔ **None.** Every question this session raised was put to the operator and
ruled on 2026-09-04, and each ruling is in the entry it governs as well as in
the settled section below. ⚠ A session that finds a new one records it here with
a recommendation attached and keeps working.

---

## ⭐ The work order

⚠ **Take these in order. Everything in it is now unblocked**, and the operator
has scoped the remainder to this session's successor plus at most one more.

⛔ **Three of the six take a third-party tree into `vendor/`.**
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) is
binding on each: the manifest, the change record, the derived series and the
reproduction command, and ⛔ upstreaming is not a topic and nothing is opened on
anybody else's repository.

1. **`DRIVER-11`**, the launcher that speaks only Chromium. Vendor an NSS
   `certutil`, seed the throwaway profile's `cert9.db`, give Gecko its own
   switch list, and record `captured.trust`. ⭐ It is what stands between the
   corpus and its first non-Chromium profile.
2. **`CORPUS-02`**, which closes on captures the session may now dispatch and
   merge. ⛔ It needs `DRIVER-11` for `firefox` and an acquisition route for
   `chromium`, and the same runs are what replace the published
   `HeadlessChrome` User-Agent, because the corpus is append-only.
3. **`PUB-13`**, the source branch, all six steps. ⛔ Verify tree-for-tree
   before removing anything, and step 6 is the CI change that step 5 makes
   necessary.
4. **`EMIT-03`**, which is now a vendor-and-patch. ⭐ Its acceptance command
   becomes runnable for the first time.
5. **`PUB-06`** with `HARNESS-11`'s residue: vendor a raw-socket route, then add
   the whole TCP half at once rather than spending a schema version on one weak
   field.
6. **`PUB-09`**, keyless attestation from the runner's own identity. ⛔ No key,
   no secret, and the record's claim that nothing here needs a credential stays
   true.

⭐ **Then the build-outs, largest first**, which the operator has put in scope:
`EMIT-04`, `PUB-05`, `HARNESS-12` and `LIB-03`. ⚠ `HARNESS-12` is the one that
receives other people's traffic, so `DOC-03`'s threat model lands with it rather
than after it.

**Small entries worth taking whenever a larger one is blocked**: `DOC-02` and
`DOC-03`. ⭐ `DOC-03` is unblocked now and is an hour's work.
⚠ `DOC-02`'s trigger has narrowed to one of its three: keyless attestation means
no workflow needs a secret and no release needs a signing key, so what remains
is a capture lane needing a machine somebody sets up, which the vendored
`certutil` and raw-socket routes are the candidates for.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

### Ruled 2026-09-04 by the operator, and each unblocks an entry

⭐ **Eight rulings, taken together at the end of the session.** Each is written
into the entry it governs as well as here.

- **The corpus moves to a SOURCE branch.** The default branch carries neither
  `corpus/` nor `raw/`, a capture opens its pull request against the source
  branch, and the data branch derives from the source branch. ⭐ `PUB-13`.
  ⚠ The alternatives lost because one leaves the default branch carrying data
  indefinitely and the other removes the comparison entirely.
- **A session may create and push the source branch**, and run all six of
  `PUB-13`'s steps. ⚠ Creating a branch is not rewriting one: the data branch
  stays append-only, and the history reset on `main` is still the operator's own
  action.
- **`EMIT-03` vendors and patches `h2`.** The eleven crates and the async
  runtime are accepted, because the alternative leaves its acceptance command
  permanently unrunnable. ⚠ Re-derive the five bytes; that tree is MIT.
- **`DRIVER-11` vendors an NSS `certutil`** and seeds the throwaway profile's
  certificate database, rather than writing a `cert_override.txt` whose format
  is version-dependent or making a machine change the trust-anchor ruling
  refuses.
- **`PUB-06` vendors a raw-socket route, and the TCP half lands whole.**
  `HARNESS-11` measured one readable field of six; no schema version is spent
  until the other five are readable.
- **`PUB-09` is keyless attestation from the runner's own identity.** No
  signing key and no secret, so the record's claim that nothing here needs a
  credential stays true.
- **A session may dispatch `capture.yml` and merge the green lanes.** It is
  the only route that closes `CORPUS-02` and the only one that replaces the
  published `HeadlessChrome` User-Agent, because the corpus is append-only.
- **`DOC-03` points at private vulnerability reporting on the forge.** One
  setting on the remote, no address published in the tree, and no timeline
  promised.
- **The four build-outs are in scope**, largest first: `EMIT-04`, `PUB-05`,
  `HARNESS-12`, `LIB-03`. ⚠ The operator has scoped the remainder to the next
  session plus at most one more.
- **The kick-off prompt is redundancy, and the router stays standalone.**
  Agents have ignored the rule that a session does not stop early, so the prompt
  is printed at the end of a session as a second copy of that instruction.
  ⛔ It changes nothing about [`../docs/AGENTS.md`](../docs/AGENTS.md), which
  must stay sufficient on its own, and the tree carries no copy of the work
  order.
- **The `HeadlessChrome` User-Agent is fixed by RECAPTURING**, one run of the
  capture lane per enabled cell.

### Ruled 2026-09-03 by the operator

- **The publishing workflow is triggered three ways**: `workflow_dispatch`, a
  push to `main`, and a pushed tag. ⭐ Done: `PUB-10`. The write is job-scoped,
  using the run's own `GITHUB_TOKEN` and never a personal access token, and the
  data branch is append-only and never force-pushed.
- **Removing `corpus/` and `raw/` from `main` is sequenced, data branch
  first.** ⛔ Nothing was deleted. ⭐ `PUB-11` closed the middle step and
  `PUB-13` carries the last one under the ruling above.
- **The history reset on `main` is not yet.** ⛔ It is the operator's action,
  after the data is published and verified, and no session force-pushes this
  remote.
- **Both `for-testing` matrix cells are enabled.** ⚠ Nothing has exercised that
  lane's capture path.
- ⛔ **`for-testing` is a `Channel`.** ⭐ Done: `DRIVER-06`.
- **`SCHEMA-12`'s six formats are four and two.** YAML, TOML, SQLite as a text
  dump and protobuf as a definition are published; CBOR and MessagePack are
  declined with their reasons published beside them.
- ⭐ **Routes are generated only where the corpus HOLDS a value.** ⛔ It is why
  no digest route exists even now that JA4 is computable, and `PUB-04` generates
  no digest artefact for the same reason.
- ⭐ **JA4 is implemented and no member of its extended family is.**
- ⛔ **The release job moves no git tag.**

### Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** Done: `DRIVER-08`.
- **The corpus carries BOTH Chromes, as separate matrix cells.**
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
- ⛔ **A guard on something irreversible is TWO conditions from two sources**,
  and it is never mutated on the machine it protects.
- **The write for `CI-04` is JOB-SCOPED**, using the run's own token.
- **The first runner capture is fetched with `gh` and added by hand.**
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.** ⛔ That is the OPERATOR'S action at a time of their choosing and
  it licenses nothing for a session.
- **Credentials are recorded as PRESENT, never as a value.**
- **The trust anchor is a job, not a machine change.**
- **Header values stay names-only by default.**
- **The schema gains numeric bounds.**
- **The shuffle seed stays out of `browser-profile/1`.**
- **Commit once at the close** unless the session is genuinely at risk of losing
  work.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
