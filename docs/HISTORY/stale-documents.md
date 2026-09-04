# stale-documents.md

⛔ **Nothing here is read to do work.** This is the wording nine documents and three
script headers carried before 2026-09-03, kept in its original words because
[`../methodology/history.md`](../methodology/history.md) says a superseded
passage is moved rather than summarised.

⚠ **What made it stale was not a wrong reading.** Every passage below was true
when it was written. Four sessions built the schema, the harness, the corpus,
the emitters, the library and the publishing surfaces, moved
[`../../TODO/`](../../TODO/) and the changelog with each of them, and did not
re-read the reference pages those name. ⛔ That is step 4 of
[`../../TODO/RULES.md`](../../TODO/RULES.md) section 10, and skipping it is what
this page records.

---

## What replaced each passage

| where it was | what it said | what is true |
| --- | --- | --- |
| [`../architecture.md`](../architecture.md) section 1 | nothing computes a digest | JA4 and its raw forms are derived from the model; the corpus still stores none |
| [`../architecture.md`](../architecture.md) section 2 | five components | eight crates, plus one that is not a component yet |
| [`../architecture.md`](../architecture.md) section 4 | nothing is published outside this repository | the `data` branch carries a tree built by one assembler |
| [`../../README.md`](../../README.md) | running the checks and reading is all this repository can do | it captures, validates, emits, publishes and hands a program a profile |
| [`../conventions/docs.md`](../conventions/docs.md) | three roles are deliberately empty, one of them the technical reference | two are, and the technical reference is written |
| [`../inherited-claims.md`](../inherited-claims.md) | there is no corpus | there is, and the row that was measured here is published in it |
| [`../../experiments/README.md`](../../experiments/README.md) | what a real trust anchor would do is unmeasured | `50-` measured it on one platform |
| [`../../scripts/README.md`](../../scripts/README.md) | thirteen checks had no section at all, and two rows named closed entries as open | every check has a section, and both rows are corrected |
| [`../agent-tooling.md`](../agent-tooling.md) | fourteen of the tools this repository ships | all of them |
| [`../methodology/work-todo.md`](../methodology/work-todo.md) | a session boundary owes a prompt for the next one | this project refuses one, and section 10 of the rules says why |
| `check-data-branch`, both halves | the comparison against the published branch cannot run | it runs, and reports `matched` in its own JSON |
| `check-formats.sh` | nothing here publishes a generated format | the data branch carries nine of them |
| `check-license-consistency`, both halves | the data branch is not checked because it does not exist | both its manifest identifier and its licence text are compared |
| [`../../TODO/RULES.md`](../../TODO/RULES.md) section 8.5 | the Windows toolchain failure is the runner's, and the answer is to rerun the job | this repository's own probe starts the install the conflict is a fragment of, and kills it |

---

## The passages, in their original words

### `architecture.md`, the `digests` row

```text
| `digests` | ⚠ empty. Nothing here computes JA3 or JA4 yet, and a digest from
an unverified implementation is the fabricated value this project refuses.
`VALID-04`. | the same file |
```

⛔ **`VALID-04` closed on 2026-09-03** with JA4 implemented from the published
specification and checked against sixteen vectors. What survives of the passage
is the corpus half: a published profile still carries `digests: null`, and by
the ruling in [`../../TODO/validator.md`](../../TODO/validator.md) it always
will, because a digest is derived on demand rather than stored.

### `architecture.md`, section 4

```text
⚠ **Nothing is published outside this repository yet.** There is no release, no
data branch and no fetchable route; `PUB-01`, `PUB-02` and `PUB-03` are those
three surfaces. [`../TODO/publish.md`](../TODO/publish.md).
```

⛔ **All three closed**, and the data branch was pushed on 2026-09-03. The
release half of the sentence is the half that stayed true, and it is now stated
where a reader will look for it rather than as part of a claim that is otherwise
wrong.

### `architecture.md`, section 5

```text
- **No digest is computed.** `digests` is empty on every profile and will stay
  empty until a reference implementation is verified against published vectors.
```

⚠ **The condition it names was met.** The reason `digests` is empty changed
from "nothing computes one" to "a derived value does not get a second home in an
append-only corpus", and a limit whose reason has changed is a limit that has to
be restated.

### `README.md`, the quick start

```text
⚠ **These are all this repository can do today**: run its own checks, and read.
```

### `README.md`, the publishing surface

```text
⛔ **The default branch is not the publishing surface.** Nothing is published
from this repository yet. When it is, it goes to an orphan **data branch**
carrying only generated artefacts, with dated snapshots, a `latest` pointer, an
index and a checksums file, append-only and never force-pushed. The generated
formats, the flat fetchable routes and the ready-to-paste client snippets are
generated from the canonical corpus rather than maintained beside it.

⚠ **None of that exists today**, and the entries that build it are named so the
gap is visible rather than implied: `SCHEMA-08` and `SCHEMA-12` are the
generator and its nine formats, `PUB-01` the releases, `PUB-02` the data branch,
`PUB-03` the routes, and `PUB-04` the artefacts that are not data files.
```

⭐ **Four of those five closed.** `PUB-04` is the one that had not, and it is
the only one the live page still names.

### `README.md`, contributing

```text
⚠ **Not yet.** There is nothing to contribute to until the schema and the
harness exist.
```

### `conventions/docs.md`, the empty roles

```text
⚠ **Three roles this set deliberately leaves empty**, because a skeleton with
nothing in it outlives the session that wrote it. There is no technical
reference for a schema nobody has written, no operator runbook for a pipeline
that does not run, and no threat model for a corpus that does not exist.
```

⛔ **The same document already said the opposite two sections lower**, where the
rule that a technical conflict is settled by `architecture.md` names that file.
⚠ A document contradicting itself across two sections is the shape a page takes
when one section is edited and the other is not, and it is why the set table now
carries a row for every document under [`../`](../).

### `inherited-claims.md`, the header

```text
⚠ **One row has been measured here, and section 12 says what happens to it.**
Section 5 carries the first quantity this project read off a browser's own
wire, on 2026-09-01, beside the reading it inherited. ⛔ It is still not in the
corpus, because there is no corpus: `CORPUS-01` is the entry that builds one.
```

⚠ **Section 5 of the same file already said the row was published.** The header
was written the day before the corpus existed and the section under it was
updated when it did.

### `experiments/README.md`, the trust configuration

```text
⭐ `20-` measured the capture SURFACE and found it changes nothing a raw
capture can see; what a real trust anchor would do is still unmeasured, and
answering it needs a root installed into the host's own trust store.
```

⭐ **`HARNESS-14` installed exactly that root**, on a hosted runner that is
thrown away afterwards, and `50-trust-anchor.sh` is the run. The experiment was
listed in the table three lines above the sentence saying it had not happened.

### `scripts/README.md`, the untwinned pair

```text
| [`common/mine-repo.sh`](common/) | ⚠ **It has a twin and the pair is NOT
compared**, which is a defect rather than a decision. It needs the network,
which is why it was left off the list. ⭐ Its offline self-test does not, and
comparing that is `TOOL-05`. |
```

⛔ **`TOOL-05` closed and the row stayed.** The comparison exists, on the
self-test, and the row was deleted rather than emptied by the rule in
[`../../TODO/RULES.md`](../../TODO/RULES.md) section 4. The same page called
`TOOL-04` an open defect in the same tool, four sessions after it closed.

### `methodology/work-todo.md`, the session boundary

```text
- **The next prompt**, in chat only.
```

⛔ **[`../../TODO/RULES.md`](../../TODO/RULES.md) section 10 refuses one**, in
as many words, and so does [`../AGENTS.md`](../AGENTS.md) section 6. The general
methodology page kept the template's wording, so the tree carried two answers to
whether a session ends by printing a prompt.

### `check-data-branch`, the header of both halves

```text
⛔ WHAT IT CANNOT ASSERT YET, AND SAYS SO. The branch does not exist, so
"compares byte-identical to what is published" has nothing to compare against.
```

⚠ **The code was corrected on 2026-09-03 and the header above it was not.** The
comparison runs in both halves and reports `matched` in its JSON. ⛔ A file
whose header describes a behaviour its own body no longer has is the same defect
as a stale document, in the place a reader is most likely to trust it.

### `check-formats.sh`, on where a generated format goes

```text
Nothing in this repository publishes generated formats yet: PUB-02 and PUB-03
are the surfaces that will, and this check exists before them so the generator
is proved before anything depends on it.
```

⭐ **Both surfaces exist and the data branch carries nine of them.** What
survives is the useful half, which the header now states: the published copies
come from this same generator, and that is what makes them checkable here before
anything fetches them.

### `check-license-consistency`, the header of both halves

```text
⛔ THE DATA BRANCH IS NOT CHECKED HERE BECAUSE IT DOES NOT EXIST. PUB-02 is the
entry that creates it, and the LICENSE it carries is this file's to check on
the day it does.
```

⚠ **That day was the day before.** The branch was pushed on 2026-09-03 and the
sentence stayed, which is the same shape as the skip above it: a leg whose own
condition had been met, still declining. `PUB-12` is the entry that added the
two legs.

### `TODO/RULES.md`, section 8.5, on the Windows CI failure

```text
## 8.5 ⚠ The Windows CI job fails at the toolchain step, and it is not yours

The Windows runner intermittently fails installing the pinned toolchain, with a
component conflict rather than anything about this tree:

  error: failed to install component: 'rustfmt-preview-x86_64-pc-windows-msvc',
  detected conflict: 'bin\cargo-fmt.exe'

⭐ How to tell it apart from a real failure: it happens at the toolchain
install, before any check runs, and the Ubuntu job of the same run passes.
Rerun the failed job rather than changing anything.

  gh run rerun RUN_ID --failed

⚠ Measured 2026-09-01: one run failed this way and the rerun of the same commit
passed with no change to the tree.
```

⛔ **Every sentence of that is an accurate observation and the conclusion drawn
from them is wrong.** The failure does happen at the toolchain install, the
Ubuntu job does pass, and a rerun does clear it. None of that makes it the
runner's: the probe step that runs BEFORE the toolchain step asks `rustc
--version`, which in a tree pinning an absent toolchain begins installing one,
and the probe kills it after six seconds. The rerun works because the second
run finds the toolchain already there.

⚠ **The Ubuntu job passing is the detail that misled three sessions.** It was
read as evidence that the tree was innocent and the runner was not. It is
actually the clue: that job passes `--fast`, which skips version probes, so it
never starts an install for anything to interrupt. `CI-09` has the measurement.
