# RULES.md

The part of the record that does not change between sessions.
[`PROGRESS.md`](PROGRESS.md) is what changed since last time and carries the work
order; [`INDEX.md`](INDEX.md) is the list of entries; this is the standing state
and the rules that are this repository's own.

⛔ **A rule is here only if it cost something here.** Everything general lives in
[`../docs/conventions/`](../docs/conventions/) and
[`../docs/methodology/`](../docs/methodology/), and this file links rather than
repeats, so the two cannot fork.

---

## The standing facts

⚠ **Read from the machine or the API, never typed from memory.** Each row names
where it is checked.

| fact | value | where it is read from |
| --- | --- | --- |
| visibility and licence | public, 0BSD | [`../LICENSE`](../LICENSE) |
| what it publishes | six profiles, on the default branch under `corpus/v1/` with their bytes under `raw/v1/`, and ⭐ **200 files on the `data` branch since 2026-09-03**, pushed by `publish.yml` and verified tree-for-tree against a local build. ⚠ No release has been cut: a tag is the only thing that produces one. | `b-ids-corpus verify` and `scripts/common/check-data-branch.sh`, each of whose last line is a fixed count |
| work model | todo | [`../docs/methodology/work-todo.md`](../docs/methodology/work-todo.md) |
| push policy | commit and push, to this repository's own remote only, on the working branch | [`../docs/conventions/git.md`](../docs/conventions/git.md) section 2 |
| the local gate | `sh scripts/common/check-gate.sh --fast`, or its `.ps1` twin | [`../scripts/README.md`](../scripts/README.md) |
| the identity a commit carries | the machine's git configuration, per invocation | [`../docs/conventions/git.md`](../docs/conventions/git.md) section 1 |
| the router | [`../docs/AGENTS.md`](../docs/AGENTS.md), and there is no second one | |
| the resume file | [`../docs/HISTORY/RESUME.md`](../docs/HISTORY/RESUME.md), written at the start of a session and **committed** | rule 6 below |

⛔ **The gate's measured cost belongs in [`PROGRESS.md`](PROGRESS.md), not
here.** It is re-timed on the machine that ran it and it moves, which is the
definition of something that does not belong in this file. `TOOL-07` is the
entry that takes the measurement.

---

## 1. ⭐ A value that cannot be traced to a socket is a different product

**This is the rule that makes this repository unlike an ordinary one**, and it
is the one most likely to be got wrong, because the wrong thing looks like
progress.

Every other project in this field carries a table of fingerprints. Filling this
project's corpus from those tables would produce something that looks finished
in an afternoon and is worth nothing, because the whole contribution is that a
consumer can ask where a value came from and get an answer.

So: [`../docs/inherited-claims.md`](../docs/inherited-claims.md) is where every
value this project did not measure lives, ⛔ **and nothing in it may be
published as data.** A row leaves that file only when this project measures the
same thing itself.

**What it cost, so far, is nothing, and that is the point of writing it down
now.** ⚠ The temptation is strongest on the first day the harness works, when
sixty inherited values are sitting in a document and one import would fill the
corpus.

## 2. A claim in this tree names its provenance or it is a defect

⛔ **Five inherited claims have been refuted**, and the fifth is the first one
an experiment took down. [`../docs/HISTORY/README.md`](../docs/HISTORY/README.md)
lists them, each in its original wording with what refuted it underneath.

That is the ratio to expect from a document nobody has checked, and it is why
every number in this tree carries a source tag and a status rather than a
citation. ⚠ A citation says where a claim came from; a status says whether
anybody has checked it, and those are different questions.

⭐ **The fifth is worth its own sentence.** "Chrome on Linux does not read the
user's NSS database for server authentication" was carried from 2026-08-31 and
read as settled; a root added there on a hosted runner let the browser complete
two handshakes. `HARNESS-14` is the run. ⚠ Four claims fell to READING and one
to a MEASUREMENT, which is the first time this project has been able to take
one down that way at all.

**What it cost.** One would have sent a session looking for GREASE values inside
a digest that strips them. One would have made the raw capture look like a
backstop rather than the only artefact that can answer a whole class of
question. ⭐ **And the fourth is the one that produced the rule below it**: it was
a claim about a repository, refuted by opening that repository, which nobody had
done for a day.

## 3. ⛔ A document is checked against the tree it describes

⭐ **The most expensive defect this repository has had.** The founding brief was
transcribed into this tree without fetching `Azathothas/bit-cli`, the repository
every measurement in it was taken in. **Every** inherited value arrived carrying
a provenance tag naming an unidentified document, so a later session could check
none of them; five claims were wrong or imprecise; and one measurement was
called unavailable across four entries and both halves of the sweep while it sat
committed in a file one command away.

**What it cost.** One fetch, taken later, and a whole document set rewritten.
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
lists what changed.

⛔ **So: a claim about a repository is not written until that repository is in
[`../references/`](../references/) at a named commit.**
[`../docs/methodology/references.md`](../docs/methodology/references.md) is the
procedure and `scripts/common/mine-repo.sh` is the one tool for it.

## 4. Every check has two halves, and one machine runs both

⛔ **A POSIX shell check cannot be assumed to run on Windows**, and the reverse
is equally true. [`../scripts/README.md`](../scripts/README.md) carries the
measurement and the exceptions.

⚠ **A twin that is written and not compared is two behaviours.** `check-twins`
runs both halves of every pair on one tree. Adding a twin without adding its row
there is how drift starts, and `TOOL-05` records the pair in this tree that is
still uncovered.

⭐ **And a comparison outlives what it compared unless somebody deletes it.**
`TOOL-09` is the worked example: a pair was removed and its comparison was
removed in the same change, because a check pointed at nothing fails for a
reason nobody can read. ⛔ An exemption or a row is deleted, never emptied.

## 5. The reference corpus is tracked, and it is not this project's code

⭐ [`../references/`](../references/) holds nineteen repositories' trees at
named commits, one of which is the origin every inherited value came from. It is the evidence behind
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md),
and a conclusion nobody can re-check is an opinion.

⚠ **Eighteen were swept and the nineteenth was not, and the two are different
things.** A swept reference was read in passes and carries a verdict.
[`../references/http2jp__hpack-test-case/`](../references/http2jp__hpack-test-case/)
is a corpus of test VECTORS that a check in this tree runs against, fetched by
`HARNESS-04` because a decoder written without them is a decoder checked
against its own misreading. It has no verdict because nothing was concluded
from reading it.

Three rules on it:

- ⛔ **It is not ignored and it is not deleted.** Two sweeps elsewhere lost their
  corpora, in opposite ways, and both cost the same thing.
- ⚠ **It was trimmed, and every deletion is recorded** in the affected project's
  own provenance file. Nothing was moved, so every remaining path is the path
  upstream has.
- ⛔ **0BSD covers what this project writes, not what it quotes.** Each tree is
  under its own licence, and the data branch carries none of it.

## 6. The resume file is committed here

[`../docs/methodology/sessions.md`](../docs/methodology/sessions.md) says the
project decides whether to commit it. This project commits it.

**Why.** Sessions here run in containers that are reclaimed when they end, so an
untracked file does not survive the session that wrote it, which is the exact
failure the resume file exists to prevent. ⚠ The cost is a file that changes on
every commit, and that is accepted.

## 7. The record moves in the same change as the work

Specified in
[`../docs/methodology/work-todo.md`](../docs/methodology/work-todo.md), which
also carries the incident behind it and the arithmetic hazard. The mechanics
here:

```bash
node scripts/common/set-record.mjs status HARNESS-05 done
```

```bash
sh scripts/common/check-record.sh
```

⛔ **The writer does not grade itself.** The first command moves the numbers and
prints the reader's command; the second asserts independently and runs as a
gate.

## 8. An entry closes on a command, not on a paragraph

An entry is authored from [`ENTRY.md`](ENTRY.md) and closes in place, with the
acceptance command actually run and its real output pasted underneath.

⛔ **Nothing here closes as "won't fix" or "somebody else's repository".** A
blocked entry stays open with the blocker named and what would unblock it.
`EMIT-03` is the worked example: it is blocked on a measurement, it says so, and
it names the entry that takes the measurement.

⚠ **Where the blocker is code this project vendors, the answer is to patch it**
here. [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md)
also settles that upstreaming is not a topic, and `TOOL-04` is the first entry
this applies to.

## 8.5 ⛔ A probe does not change the machine it is probing

⛔ **A standing fact, and it lives here rather than in
[`PROGRESS.md`](PROGRESS.md) because that file is rewritten every session and
this was lost that way once.**

⚠ **This section used to say the Windows job's toolchain failure was the
runner's and to rerun it. That was wrong**, and it stood for three sessions.
`CI-09` measured the cause: `rustc` and `cargo` are rustup proxies, so this
repository's own probe, run in a tree pinning a toolchain the runner does not
have, STARTED installing it and then killed the install at its six-second
limit. The conflict the job reported was a fragment of that. The superseded
wording is in
[`../docs/HISTORY/stale-documents.md`](../docs/HISTORY/stale-documents.md).

⭐ **The rule that came out of it, and it is general.** A probe measures a
machine. It does not change one. A version flag looks read-only and is not:
a proxy, a wrapper or a shim can install, download or start a daemon in answer
to it, and a probe that kills what it started leaves the machine worse than it
found it.

Two things hold it, from two sources, because one of them can be absent:

- both halves of [`../scripts/doctor/`](../scripts/doctor/) export
  `RUSTUP_AUTO_INSTALL=0`, which rustup reads from 1.28;
- [`../.github/workflows/ci.yml`](../.github/workflows/ci.yml) installs the
  pinned toolchain BEFORE it runs the probe, in both jobs, so an older rustup
  that ignores the variable still has nothing left to interrupt.

⭐ **The claim has a command**, because a claim about what a script does not do
is the kind nobody re-checks:

```bash
sh scripts/doctor/doctor.sh --fixture
```

⛔ **So a red Windows job is now read as a real failure.** Do not rerun it to
see whether it goes away. ⚠ If a toolchain conflict does appear again, the
machine is carrying a half-installed toolchain from before this fix, and
`rustup toolchain uninstall` on the named version clears it.

⚠ **And it does not always name the same component.** This section carried the
`rustfmt-preview` spelling for three sessions; the last run before the fix,
`33760180207`, reported `clippy-preview` instead. ⛔ The conflict names whichever
component was mid-unpack when the kill landed, so matching on one spelling would
read a second instance as a different defect.

---

## 9. What a session owes at its end

⛔ **Section 10 is the list, and it is the only copy.** It is written there
rather than here because the two halves of one rule are "do not stop" and "what
to do when you do", and splitting them across two sections is how one of them
gets followed.

[`../docs/methodology/sessions.md`](../docs/methodology/sessions.md) is the
general specification; section 10 is what is this repository's own.

## 10. ⛔ A session does not stop early, and it does not defer

⛔ **DO NOT STOP OR DEFER ITEMS TO A NEXT SESSION.** Continue working until the
operator interrupts, or until this session has completed at least **twenty
effort points**, whichever comes first.

⭐ **The scale is the one [`INDEX.md`](INDEX.md) already defines**, so nothing
new has to be judged:

| effort | points |
| --- | --- |
| S | 1 |
| M | 2 |
| L | 4 |
| XL | ⛔ not a unit of work. Split it. |

⚠ **Twenty points is five `L` entries or their equivalent**, and the equivalence
is the point: five small entries closed with their acceptance commands run is
the same work as one large one, and a session that closes twenty small entries
has done more than a session that half-finishes one large one.

⛔ **What does not count.** An entry is complete when it is closed in place with
its acceptance command actually run and its real output pasted underneath, and
the record moved in the same change. Reading, planning, restating and reporting
are not points.

### ⛔ The four sentences that are not reasons to stop

Each has been used, and each is refused here.

| the sentence | what it actually means |
| --- | --- |
| "this is a good stopping point" | nothing measured says so. Keep going. |
| "budget is running low" | ⚠ a prediction about a number this session cannot read. [`../docs/methodology/sessions.md`](../docs/methodology/sessions.md) says a wall closes a route and not a question. |
| "this needs a decision from the operator" | ⛔ then do everything that does not depend on it, write the question into [`PROGRESS.md`](PROGRESS.md) **with a recommendation attached**, and work a different entry. A question is not a blocker until proceeding under any assumption would be unsafe. |
| "I will pick this up next session" | there is no next session until this one ends, and ending it is what this rule governs. |

⚠ **A blocker on one entry is a reason to work another one**, recording on the
one you left what was tried, which routes failed, and what would open it. ⛔ It
is never a reason to end the session.

### ⭐ When the operator says the session is ending

⛔ **Follow this list exactly, in order, and none of it is conditional on the
session having gone well.**

1. **Finish or checkpoint what is in flight.** ⛔ Never leave a half-edit across
   the boundary: the next session cannot tell one from finished work.
2. **Close every entry this session touched**, in place, with its acceptance
   command run and its real output pasted, and move the record in the same
   change. `node scripts/common/set-record.mjs` moves the counts.
3. **Rewrite [`PROGRESS.md`](PROGRESS.md)**: the state line, what this session
   did, what is in progress, the work order, and the open questions with their
   recommendations. ⛔ It carries no history.
4. **Re-read every document this session changed** and every document those
   name, and fix what went stale. ⚠ A document is checked against the tree it
   describes, never the reverse.
5. **Run the gate, all three parts, and get it green.**
   [`../docs/methodology/gate.md`](../docs/methodology/gate.md). ⛔ A skipped
   check is reported as a skip and never as a pass.
6. **Do the deep reviews the gate requires**, each naming what it swept and what
   the other passes did not look at.
   [`../docs/methodology/reviews.md`](../docs/methodology/reviews.md). ⭐ A pass
   that reports nothing says what would have had to be true for it to fire.
7. **Overwrite [`SUMMARY.md`](SUMMARY.md)** with this session's table and print
   it in chat. ⛔ Every cell grounded in something you can point at, including
   the cells that say nothing moved.
8. **Rewrite [`../docs/HISTORY/RESUME.md`](../docs/HISTORY/RESUME.md)** to the state the tree is actually
   left in.
9. **Leave the tree clean**: no throwaway branch, no scratch file outside
   `.tmp/`, nothing untracked that should be tracked.
10. **Commit and push** with `git-sync`, which enforces
    [`../docs/conventions/git.md`](../docs/conventions/git.md) rather than
    trusting you to remember it. ⛔ No tool is credited in the message.
11. **Confirm the remote's checks are green**, and fix and amend rather than
    leaving a red build behind.
12. **Remove anything this session created on another system.**

⛔ **There is no thirteenth step printing a prompt for whoever comes next.** The
next session is started with `Read ./docs/AGENTS.md in full`, and everything it
needs is reached from there. A prompt that restates the work order is a second
copy of it going stale.

---

---

## Settled, and not to be raised again

- **The licence is 0BSD**, for the code and for the generated data. Ruled
  2026-08-30. The reasoning is in [`../README.md`](../README.md); an
  attribution-carrying licence makes people copy values by hand, which defeats
  the project.
- **The work model is todo**, not stages. Ruled 2026-08-30. The work is a large
  set of independent items with few hard orderings, which is what that model is
  for.
- **There is one router and it is [`../docs/AGENTS.md`](../docs/AGENTS.md).**
  Ruled 2026-08-30. A second one at the repository root would restate the
  absolutes, and two files stating one rule is two places for it to be wrong.
- **The reference corpus stays in the tree** rather than on a side branch.
  Ruled 2026-08-30, and re-measured whenever a tree is added. On 2026-09-01,
  after the HPACK vector corpus landed, `references/` packs to 26.9 MiB across
  nineteen trees, which is small enough that a side branch would cost more in
  ceremony than it saves in bytes. ⚠ Re-take the measurement when it passes
  about 100 MiB packed.

  ```bash
  tar cf - references/ | gzip -9 | wc -c
  ```
- ⭐ **`Azathothas/bit-cli` is kept like any other reference**, in the tree, at a
  named commit, under its own licence. Ruled 2026-08-31. It is where every
  inherited value was measured, so a tree without it is a tree in which none of
  them can be checked.
- ⛔ **Upstreaming a patch is not a topic.** Ruled by
  [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md), which
  carries the incident.
- ⭐ **A measured profile goes into the committed corpus, with its conditions
  recorded.** Ruled by the operator 2026-09-01, at the start of the session that
  built the store, and done the same day. The corpus is append-only and a
  published profile is never edited or deleted, so the first one written is
  permanent, and the ruling was asked for on exactly that ground. ⚠ Every
  capture today goes through a per-launch key pin, so `captured.trust` records
  which configuration a profile was taken under and `HARNESS-10` is what
  measures whether it mattered.
