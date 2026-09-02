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
session ran      2026-09-02T01:14:00Z, unattended, in progress
baseline         the gate passes: 25 checks on this Windows host with
                 check-twins skipped by -Fast.
entries          total 91  open 37  blocked 0  done 54
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## The capture matrix has run, and it produced two findings before it produced a profile

**`capture.yml` ran on hosted runners for the first time**, twice, every job
green both times: the plan job, both browser lanes, the fuzz lane and the
collect job. Dispatched with the authenticated `gh` on the default branch, which
is the route the operator ruled 2026-09-01.

### What the runners actually served

| | |
| --- | --- |
| `ubuntu-latest`, image `ubuntu24/20260823.283` | Chrome `151.0.7922.173`. ⭐ Edge `151.0.4129.101` also resolved, at `/usr/bin/microsoft-edge` |
| `windows-latest`, image `win25-vs2026/20260824.214` | Chrome `151.0.7922.174` |

⛔ **The two runners do not carry the same Chrome build**, so two profiles of one
build on two platforms is not obtainable from the preinstalled browser at all.
It needs pinned acquisition, which is `DRIVER-05`'s route rather than the lane's.
That is the single highest-value capture available and it is now a known amount
of work rather than an assumption.

### ⛔ The `linux64` lane captured nothing, twice, and the reason is resumption

⚠ **Reproduced rather than raced.** Both runs produced the same shape: Chrome on
`ubuntu-latest` abandoned the connections that were not resumed and resumed
every one it kept, so the navigation had no cold connection and there was
nothing to publish. More connections do not help: the first completed handshake
leaves a ticket and everything after it resumes.

⛔ **And the report said `1 cold` on the line above the refusal saying there was
none.** `b-ids-corpus add` carried the word behind a hardcoded `1` in a format
string. It is `Selection::report` now, where a test reaches it, and the guard was
seen to fail at exit 101 against a mutation that restores the literal.

### ⭐ The fix, with the control that says it is safe

`b-ids-harness --no-resumption` issues no session tickets, so every hello is a
cold one. The harness **reports** the configuration on stderr and
`experiments/10-first-profile.sh` reads that line back into
`captured.resumption` rather than typing it.

⭐ `experiments/30-resumption-control.sh` is the control, three rounds against
Chrome `151.0.7922.76` on this host:

```text
offered: 4 cold, 11 resumed, 3 abandoned
refused: 15 cold, 0 resumed, 3 abandoned
modes=agree differing:0 not_comparable:2 fields:19
```

So the switch changes which connections are cold, not what a cold hello is.

### What the corpus holds now

⭐ **Three profiles, and two were captured on machines nobody owns**: Chrome
`151.0.7922.174` on `win64` from `windows-latest`, and Chrome
`151.0.7922.173` on `linux64` from `ubuntu-latest`, both by the same script a
person runs. `corpus=profiles:3 problems:0`, `findings:0`.

### ⭐ The matrix's `browser` column reaches the driver now

⛔ **It reached nothing before.** `b-ids-driver drive` took the first family
that resolved and the capture script wrote the literal `Chrome` into every
identity file, so an `edge` lane would have published Chrome under Chrome's
route inside an artefact called `edge`. The driver takes `--browser NAME` now,
reports the vendor spelling the corpus routes by, and the `edge/stable/linux64`
cell is enabled and required.

---

## ⭐ Four entries closed beside it

| | |
| --- | --- |
| ⭐ `SCHEMA-13` | every integer field in the published schema carries a bound derived from its Rust width, and a field added without one fails the test |
| ⭐ `SCHEMA-14` | a credential is recorded as PRESENT, in its wire position, with no value. It was dropped entirely before, so the order closed over an unmarked gap. Three refusals added, no way to record a value |
| ⭐ `VALID-03` | `unreachable_dimensions`, over every browser, channel and platform the corpus carries, reading `Family::all` rather than a list of its own |
| ⭐ `DRIVER-04` | `experiments/40-trust-paths.sh`. ⛔ The negative control is the finding: with no trust flag at all, four connections completed zero handshakes |
| ⭐ `SCHEMA-11` | `http.multipart_boundary`, as a PATTERN: prefix, random length and alphabet. ⛔ Absent in every profile, because the two known patterns are inherited by reading and nothing inherited is published as data |
| ⭐ `CI-07` | `check-exit-codes`, both halves. ⛔ Every PowerShell check answered 1 where its POSIX twin answered 2, 22 pairs of 22, and the twin comparison could not see it because it compares runs that succeed |

⚠ **Seven tests changed their assertions and none changed its title.** Each
asserted that a credential NAME was gone; each now asserts the name is there and
the value is not.

---

## What is in progress

⚠ **`CORPUS-02` is open.** The apparatus works and one lane publishes. What
remains is named in the entry: the `edge`, `chromium` and `firefox` rows its
acceptance command requires.

---

## ⭐ The work order

⚠ **Take these in order.**

1. **`CORPUS-02`**, continued. Two of its four required rows are captured:
   - **the `edge` lane**, which is enabled and wired and has not run. One
     dispatch of `capture.yml`, the artefact added with `b-ids-corpus add`.
   - **`chromium` and `firefox`** need `b_ids_driver::Family` to have branches
     for them at all. `VALID-03` is the check that says so from the corpus side.
2. **`DRIVER-04`**, then **`HARNESS-14`**. The root store a browser actually
   reads, then the per-launch pin measured against a real trust anchor on a
   disposable runner. ⚠ `DRIVER-04` lands first: on Windows the store a browser
   reads is not obviously the one `certutil` writes to.
3. **`SCHEMA-13`** and **`SCHEMA-14`**, both small and both about the published
   contract: numeric bounds the schema does not express, and a credential's
   presence recorded without its value.
4. **`CI-02`** and **`CI-04`**. Staleness on a schedule, and a run that finds a
   change opening a pull request. ⭐ `CI-04`'s write is ruled: job-scoped, on the
   collect job alone, with the run's own token. See the settled list.
5. **`SCHEMA-08`**, then `PUB-03`, `PUB-01`, `PUB-02`, `PUB-07`.
6. **`SCHEMA-12`**, the six formats that need a decoder, once `SCHEMA-08` has a
   generator to extend.

⚠ **Small entries worth taking whenever a larger one is blocked**: `SCHEMA-11`
(the multipart boundary), `CORPUS-05` (name the unidentified extension),
`VALID-03` (a family the resolver cannot produce), `DRIVER-06` (branded and
unbranded builds).

---

## Open questions for the operator

⚠ **Each carries a recommendation and each was proceeded on**, per
[`RULES.md`](RULES.md) section 10: none of them made proceeding unsafe, and all
three are reversible without editing a published profile.

### 1. Should refusing session tickets be the standing capture configuration?

**It already is**, because the alternative was a lane that cannot capture.
`experiments/10-first-profile.sh` passes `--no-resumption`, and every profile
written from here records `captured.resumption: refused`.

⭐ **The recommendation is to keep it.** The corpus publishes the cold hello and
nothing else, the control measured 0 differing fields of 19, and the switch
defaults off so the resumed-connection sample stays reachable.

⚠ **What it costs**: `HARNESS-07` says a resumed connection is recorded
separately as its own profile, and nothing captured under this switch can ever
produce one. Nothing does that today.

### 2. The driver discarded the browser's own output, and it is fixed here

⛔ `b-ids-driver drive` gave the browser `Stdio::null()`, so a lane that
captured nothing carried no word from the browser about why. ⚠ **It stopped
being a hypothetical while this session ran**: the `edge` lane launched Edge on
a hosted runner, the browser exited after 1.4 seconds having opened no
connection, and what it said went nowhere.

⭐ **`--log PATH` is implemented, under `CORPUS-02`**, because the edge row
that entry requires cannot be diagnosed without it. ⛔ No new entry was filed:
`ENTRY.md` says an entry is not filed until the operator approves it.

⭐ **The question that remains is whether it should have been its own entry**
rather than part of `CORPUS-02`. The recommendation is no: it is one switch, it
is what unblocks a row of an open entry, and a `DRIVER-07` covering it after the
fact would be a record of a decision rather than a unit of work.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01.**

- ⭐ **The write for `CI-04` is JOB-SCOPED.** `contents: write` and
  `pull-requests: write` on the collect job alone, using the run's own
  `GITHUB_TOKEN`. ⛔ Never a personal access token: a long-lived credential in a
  public repository's automation outlives every run it was issued for. ⚠ Every
  capture lane keeps `contents: read`, so a browser this project downloaded and
  ran can never reach the repository. ⚠ It also needs the repository setting
  that lets Actions create pull requests, which is the operator's to enable and
  cannot be done from the tree.
- ⭐ **The first runner capture is fetched with `gh` and added by hand.** The
  authenticated CLI runs `capture.yml`, downloads the artefact, and
  `b-ids-corpus add` writes it. ⛔ Do not build `CI-04` first: that is machinery
  ahead of the single capture it would review. ⚠ The profile is a real
  measurement either way, taken on the runner by the same script; only the
  transport is manual and the profile's own provenance says so.
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.** ⛔ That is the OPERATOR'S action at a time of their choosing and
  it licenses nothing for a session: no force push, no history rewrite, and the
  corpus stays append-only in every change an agent makes. ⭐ What it does settle
  is that a laptop capture sitting beside runner captures is not a problem to
  engineer around today.
- **`SCHEMA-08` is SPLIT.** It keeps the generator plus the five formats whose
  round trip this tree can prove: JSON, NDJSON, CSV, TSV and Markdown.
  `SCHEMA-12` carries YAML, TOML, SQLite, CBOR, MessagePack and Protobuf with
  the trade stated. ⛔ Twelve hand-written implementations in the crate that
  already owns four parsers is what must not happen.
- **Credentials are recorded as PRESENT, never as a value.** `SCHEMA-14`. ⛔ The
  value never appears on any surface, including the raw block.
- **The trust anchor is a job, not a machine change.** `HARNESS-14`. Every
  profile keeps recording `captured.trust`, because that is what makes the
  comparison possible at all.
- **Header values stay names-only by default.** Corpus captures turn them on
  deliberately. ⛔ A model whose natural form carries them is the shape that one
  day publishes a credential.
- **`CORPUS-04` publishes the per-build trust-anchor list and states all three
  options with their costs.** ⛔ It asserts no preference.
- **The schema gains numeric bounds.** `SCHEMA-13`.
- **Commit once at the close** unless the session is genuinely at risk of losing
  work. No force push and no history rewrite.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
