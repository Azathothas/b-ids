# ci

Automation, staleness, the capture matrix, and the durability that decides
whether this project still works in five years.

⭐ The design target: a maintainer who is away for three months comes back to a
corpus that is current and a queue of green pull requests they can merge without
reading each diff closely. If a human has to run a capture by hand, the project
has failed at the thing it exists to do.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## CI-01. Every push: validate, with no network and no browser

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P1, **Effort** M, **Status** open

### Problem

Nothing that must pass on every push may depend on a live browser. Capture jobs
are allowed to be absent; assertions are not.

### Premise

Structural.

### Approach

On every push: validate the corpus against the schema, run the validator over
every profile, assert every emitter still produces its recorded fingerprint, and
assert every generated format still round-trips.

**Golden vectors** are the core of it: every recorded raw `ClientHello` in the
tree parses to exactly its committed profile. ⭐ That is the test that protects
against this project's own parser rotting, and it runs offline, forever, with no
browser.

**Determinism** is asserted, not hoped for: two runs of the generators over the
same corpus produce byte-identical output. Without it, releases are not
reproducible and every run looks like a change.

⚠ Continuous integration is where the gate's PowerShell half and its `shellcheck`
half actually run, because the primary development host may have neither. A skip
there means an install broke, so continuous integration runs the gate in its
strict mode where a skip is a failure.

Must not: let a capture job's absence fail this workflow, and must not let this
workflow reach the network.

### Prove

```bash
sh scripts/common/check-gate.sh --strict
```

Passing means: the workflow runs the gate strictly on two hosts, every golden
vector reparses to its committed profile, and a deliberately altered profile
fails with a message naming the field.

---

## CI-02. Staleness is a schedule, not a push trigger

**Source** the founding brief; the shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 10
**Category** ci, **Priority** P1, **Effort** M, **Status** open

### Problem

A browser shipping a new version is not a defect in a commit. Asserting current
versions on push makes every unrelated change fail on the day a browser ships.

### Premise

Believed. ⚠ Two scheduling details are inherited and worth keeping: run on the
day of the week **after** the day browsers typically ship, so the run reports a
release that has settled; and pick an odd minute, because every scheduled job on
a shared platform queues on the hour.

### Approach

A scheduled workflow that asks what is current, compares against the corpus, and
opens a pull request when they differ, per `CI-04`. Pushes assert only against
the committed corpus.

⭐ **When it goes red, its output carries the replacement values.** A check that
only says a fingerprint changed is half a tool; the session that picks it up
should apply a diff rather than redo the work.

The version question uses `DRIVER-02`, with its rollout fraction and its
cross-check, so a staged rollout does not produce a chase.

Must not: fail the run when one source is unreachable. Trap every fetch
separately and report which answered.

### Prove

```bash
sh scripts/common/check-staleness.sh --json
```

Passing means: run against a fixture corpus that is deliberately one version
behind, the output names the current version, its rollout fraction, every source
that answered, and the replacement values.

---

## CI-03. The capture matrix, fanned out, with every lane allowed to fail alone

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P1, **Effort** L, **Status** open

### Problem

A matrix that cancels every lane when one browser fails to download produces
nothing from a run in which almost everything worked.

### Premise

Believed, and the failure mode is the easiest to get wrong: the default
behaviour of a matrix is to cancel siblings.

### Approach

One job per browser, channel and host. Fan out fully; these are independent and
each is minutes.

- ⛔ **fail-fast off, always.** Every lane must be allowed to fail alone.
- **a tight per-job time limit.** A hung browser holds a runner, and browsers
  hang.
- **a concurrency group per workflow**, so a re-run cancels the previous one
  rather than racing it.
- **each lane uploads its profile and its raw capture as an artefact.** ⛔ No
  lane writes to the repository.
- **one collect job depends on all of them**, merges the artefacts, runs the
  validator over the whole set, and opens **one** pull request. One per run,
  never one per cell: thirty pull requests a night is how an automated project
  gets muted.
- ⭐ **the collect job runs even when lanes failed**, so a run where three lanes
  failed still publishes the twenty-seven that worked and names the three.
  Partial data with an honest gap beats no data.
- **cache browser downloads keyed by browser, channel, version and digest**,
  which is also the durability requirement in `CI-06`.

⭐ **The matrix is what makes automated merging possible**, because `CI-04`
requires agreement across two independent sources and that is only satisfiable
when one build is captured on more than one host.

Must not: assert a fingerprint captured on one platform against a profile
claiming another. That is how platform dependence is discovered, and flattening
it discards the discovery.

### Prove

```bash
sh scripts/common/check-workflows.sh --assert-fail-fast-false
```

Passing means: the workflow parses, every matrix job declares fail-fast off and
a time limit, the collect job declares that it runs regardless, and a fixture
workflow missing any of those fails the check.

---

## CI-04. A scheduled run that finds a change opens a pull request, not an issue

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P1, **Effort** M, **Status** open

### Problem

An issue is a request for somebody else to do work. A pull request with the work
already in it is the deliverable.

### Premise

Believed.

### Approach

**What the branch carries**: the new profile with full provenance; the raw
capture; every generated format regenerated, so nothing is left for a human to
run; the regenerated goldens and the passing tests that assert them; and the
changelog entry, generated by the same code that writes the release body per
`PUB-08`, so the two cannot disagree.

**What the body carries**, as evidence a reviewer can read without checking
anything out: the before and after fingerprints side by side and a **field-level
diff** from `VALID-06`; the capture provenance, meaning the exact build,
channel, platform, image digest, harness version and run identifier; the
validator's output in full; and ⛔ **anything the run could not do, named.** A
pull request that silently omits a field is worse than one that says it could
not capture it.

**Automated merging is allowed**, and the conditions are the interesting part.
Merge without a human when all of:

1. the validator passes with no warnings;
2. the capture agrees across **two independent sources**: two platforms, or a
   container and a hosted runner, or two channels of one build;
3. no field regressed to `vendor` provenance, and no field became
   `unreproducible` that was not already;
4. the diff touches only fields the change class predicts. A version bump may
   move the User-Agent and the brand list; if it also moves the cipher list,
   that is a human's decision;
5. every generated format round-trips.

Anything else opens the request and asks for review, labelled with which
condition failed.

**Issues are for questions only**: a new unidentifiable extension, two sources
that disagree, an emitter that can no longer produce a profile. Those need a
decision, and a decision is what a human is for.

Mechanics that decide whether this is pleasant or hated:

- **deterministic branch names**, one per browser, channel, platform and schema
  major, so a re-run updates the open request instead of opening a second one;
- ⭐ **a no-op run opens nothing and comments nothing.** Silence is the correct
  output for "the browser did not change". A bot that comments on a schedule
  trains people to ignore it;
- ⛔ **never force-push over a human's commit** on a bot branch. Detect a non-bot
  commit and stop, with a comment saying why;
- **supersede rather than accumulate**: when a newer capture makes an open
  request stale, close it with a link to the replacement;
- **every request is reproducible**: the body carries the exact command that
  produced it, runnable locally, which is also the manual fallback in `CI-08`;
- **labels that support triage at a glance**: the change class, the confidence,
  and the subject.

Must not: open a request against any repository but this one.

### Prove

```bash
sh scripts/common/check-pr-body.sh --fixture
```

Passing means: given a fixture corpus change, the generated body contains a
field-level diff, the full provenance, the validator output and a named list of
what the run could not do; and a fixture no-op change generates nothing at all.

---

## CI-05. The cold-start job, which is the only thing that catches rot

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P2, **Effort** M, **Status** open

### Problem

Every warm run passes over a broken cold path. A dead URL, a removed field or a
renamed flag is invisible until the day somebody needs a capture.

### Premise

Believed.

### Approach

A scheduled job that builds from a clean checkout, with an empty cache, on a
fresh runner, and proves the whole pipeline still works end to end.

⭐ **Nothing else catches this**, which is why it is its own entry rather than a
flag on another job.

Must not: let it share a cache with any other workflow, which would defeat it
entirely.

### Prove

```bash
sh scripts/common/check-cold-start.sh
```

Passing means: the job runs with every cache disabled, completes a capture and a
publish end to end, and fails loudly naming the first step that could not
resolve.

---

## CI-06. No single source of any fact

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P2, **Effort** M, **Status** open

### Problem

Every external dependency will one day answer differently, and a corpus that
stopped updating in year two was not worth building.

### Premise

Believed, and one instance is already measured: two first-party version sources
disagreed, and the disagreement was the defect.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 7.

### Approach

For every external question, know more than one way to ask it, try them in
order, record which answered, and treat disagreement as a **finding** rather
than an error.

| question | sources, in order |
| --- | --- |
| what version is current | the version-history API with its rollout fraction; the automation-build index; the vendor's package repository metadata; the installed binary's own report; a third-party end-of-life index; the vendor's release notes |
| where do I get that build | `DRIVER-05` |
| what does it emit | this project's harness; a second harness implementation; a hosted oracle; a stored raw capture |
| where do consumers get the data | releases; the data branch; a mirror; the last release anybody already downloaded |

⭐ **Two sources that disagree are the single most valuable signal this project
produces.** Record both answers, publish both, and let the validator flag it.

Also from the same rule:

- **per-source isolation.** Every fetch is trapped on its own, and the run
  reports which sources answered and which did not, published with the data;
- **cache every artefact fetched, keyed by digest**, and keep the last
  known-good build per channel;
- **pin by digest, never by tag.** A moved tag runs code nobody reviewed;
- **prefer boring dependencies** on the critical path, and where that is not
  possible, vendor and record why;
- ⛔ **never publish something wrong to avoid publishing nothing.** If a capture
  cannot be verified, publish no new profile and let the corpus visibly age.
  Carry an age field and a staleness banner in the index so a consumer can see
  it. An openly old profile beats a silently wrong one;
- ⭐ **the corpus is its own backstop.** If every upstream vanished tomorrow, the
  published data still stands and is still correct about the day it was
  captured.

Must not: silently prefer one source when two disagree.

### Prove

```bash
sh scripts/common/check-sources.sh --json
```

Passing means: with one source made to fail, the run degrades and reports which
answered; with two sources made to disagree, the run publishes both answers and
flags the disagreement rather than picking.

---

## CI-07. Exit 2 means could not run, and it is not a failure

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P2, **Effort** S, **Status** open

### Problem

A capture job on a runner with no browser must not fail the build. A check that
fails because a machine has no browser is a check somebody disables.

### Premise

Structural, and it is already the contract every script in this repository
follows.

### Approach

Every capture job and every check distinguishes three outcomes: it ran and
passed, it ran and the thing failed, it could not run. The third exits 2 and is
reported separately in the summary, never folded into either of the others.

⚠ The same rule reaches the report: a lane that could not run is named in the
collect job's output, so a run with three such lanes says which three.

Must not: collapse 2 into 0 with a shell fallback, which hides a genuine
failure as well.

### Prove

```bash
sh scripts/common/check-exit-codes.sh
```

Passing means: every script in the tree that can fail to run is invoked in a
state where it cannot, and each returns 2; a script that returns 1 for that
state fails the check.

---

## CI-08. A documented manual path, for the day the provider is not there

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P2, **Effort** S, **Status** open

### Problem

A project whose only path to a capture is one provider's automation degrades to
nothing when that provider does.

### Premise

Believed.

### Approach

Every automated step has a written manual equivalent, and the pull request body
in `CI-04` carries the exact command that produced it, runnable locally.

⭐ The test is one sentence: if the provider disappeared, the project degrades to
"somebody runs one command" rather than to nothing.

A second provider is the stronger version and is optional; the documented local
path is the requirement.

Must not: document a command nobody has run. A manual fallback that has never
been executed is a claim.

### Prove

```bash
sh scripts/common/check-manual-path.sh
```

Passing means: every automated step names its manual equivalent, and the check
executes each one on this host, reporting exit 2 for any the host cannot run
rather than skipping it silently.
