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
**Category** ci, **Priority** P1, **Effort** M, **Status** done

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

### Closing

**Closed 2026-09-01T13:10:00Z.** Every push now settles what is published, on
two hosts, with the network off for every assertion and no browser anywhere in
the workflow. ⛔ **Two of the assertions were not being made at all**, and one of
them had been reporting green over an empty question since the day it was
written.

```text
$ sh scripts/common/check-gate.sh --strict
check-gate: (the repository root)

  ok    check-docs
  ok    check-markers
  ok    check-one-home
  ok    check-placeholders
  ok    check-control-bytes
  ok    check-record
  ok    check-no-secrets
  ok    check-vendor
  ok    check-msrv
  ok    check-corpus
  ok    check-validate
  ok    check-workflows
  ok    check-coverage
  ok    check-routes
  ok    check-changelog
  ok    check-line-endings
  ok    sh -n
  ok    shellcheck
  ok    powershell parse
  ok    PSScriptAnalyzer
  ok    cargo fmt
  ok    cargo clippy
  ok    cargo test
  ok    doctor probe
  ok    check-twins

gate ok: all 25 checks passed
exit=0

⚠ **One substitution, and it is marked**: the first line prints the absolute
path of the repository root, and `check-no-secrets --public` refuses an absolute
home path in a public repository. ⛔ The rule was not widened for a pasted
transcript. It was found by the remote gate rather than the local one, because
this block was filled in AFTER the local run and only a subset of the checks was
re-run over it.

⚠ **This is the run of 2026-09-01T15:34:31Z to 15:50:19Z**, on the tree as it
is committed, with nothing else running. ⛔ An earlier attempt at this paste was
assembled by hand from a log two processes had written into, and was replaced
with a marker rather than shipped: the claim audit at the end of the session
caught it.
```

### ⛔ The defect this entry found: the history leg verified nothing in CI

`check-corpus`'s first leg asks git whether a published file was ever modified,
deleted or renamed, and it is the one question the working tree cannot answer.
⚠ `actions/checkout` fetches **one commit** by default, so `git log
--diff-filter=MDR` over the corpus paths saw a single commit and found nothing.

Reproduced on a `--depth 1` clone of this tree, with the check exactly as it
stood this morning:

```text
$ git rev-parse --is-shallow-repository
true
$ git log --oneline | wc -l
1
$ git log --diff-filter=MDR --name-status --format='commit %h' -- corpus raw \
    ':(exclude)corpus/*/index.json' ':(exclude)corpus/*/latest.json' | wc -l
0
$ sh scripts/common/check-corpus.sh
corpus ok: 1 profile(s), nothing edited after publication, index and
pointers agree with the tree.
exit=0
```

⛔ **The append-only rule is the corpus's central promise and continuous
integration had never once checked it.** Which row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md)
this is, and what it means for the check, is written where the check is:
[`../scripts/README.md`](../scripts/README.md).

⭐ **Both halves refuse a shallow clone now**, and both workflows carry
`fetch-depth: 0`, so losing that line fails a job rather than emptying it. The
same clone, with the guard:

```text
$ sh scripts/common/check-corpus.sh
check-corpus: this is a SHALLOW clone, so the history leg cannot run and
nothing was verified about whether a published file was ever edited.
Fetch the whole history: git fetch --unshallow, or fetch-depth: 0 on the
checkout step of the workflow that produced this tree.
exit=2
$ sh scripts/common/check-corpus.sh --json
{"schema":"check-corpus/2","corpus":true,"shallow":true,"profiles":0,"edits":0,"problems":0}
exit=2
$ pwsh -NoProfile -File scripts/common/check-corpus.ps1 -Json
{"schema":"check-corpus/2","corpus":true,"shallow":true,"profiles":0,"edits":0,"problems":0}
exit=2
```

⚠ **The JSON schema went to `check-corpus/2`** because the shape gained a field.
`CORPUS-01`'s closing block above pastes the `/1` form; it is left as it was
measured rather than re-pasted, which is the rule the previous session already
applied to a suite count that moved.

### ⭐ The coherence checks had never run over what is published

`b-ids-validator` takes the paths a caller names, so it answered about whatever
somebody remembered to list, and nothing in the gate listed anything.
⭐ **`b-ids-corpus validate` is the corpus-scale form**, and it is answerable in
that crate because that crate owns the layout:

```text
$ cargo run -q -p b-ids-corpus -- validate --root .
corpus/v1/chrome/stable/win64/151.0.7922.76.json: SKIP  handshake -- deciding whether this hello came from a 151 build needs a per-build corpus to compare against, and none exists yet. b-ids-validator::shared_handshakes is the form that runs across a set of profiles today
corpus/v1/chrome/stable/win64/151.0.7922.76.json: SKIP  encoding -- the caller did not say what the consuming client can decode
corpus/v1/chrome/stable/win64/151.0.7922.76.json: SKIP  absence -- the caller named no target stack
corpus=validate profiles:1 findings:0 notcheckable:3
exit=0
```

⚠ **Three of eight checks report they had nothing to read, and that is counted
rather than folded into the pass.** Two of the three need a caller's intent; the
third needs a second profile of the same build, which is `CORPUS-02`.

⭐ **It also runs the CROSS-profile form of check 4**, `shared_handshakes`, which
no per-profile invocation can reach at all.
[`../scripts/README.md`](../scripts/README.md) says what that check compares.
⚠ It is structurally silent on a corpus of one, and saying so is the point:
`CORPUS-02` is what ends that.

### The acceptance's third leg: a deliberately altered profile

⛔ **Run against a scratch copy and against a working tree that was restored in
the same command**, never against the published file. `browser.major` moved from
151 to 152 and nothing else:

```text
$ sh scripts/common/check-validate.sh
validate check failed: 3 finding(s) over 1 published profile(s).

corpus/v1/chrome/stable/win64/151.0.7922.76.json: FAIL  version: http.headers.user-agent: carries major 151, and browser.major is 152
corpus/v1/chrome/stable/win64/151.0.7922.76.json: FAIL  version: http.headers.sec-ch-ua: no brand claims major 152; it claims Not=A?Brand=99, Google Chrome=151, Chromium=151
corpus/v1/chrome/stable/win64/151.0.7922.76.json: FAIL  version: browser.version: 151.0.7922.76 does not begin with the claimed major 152
exit=1
```

⭐ **Three messages, each naming its field**, which is what the acceptance asked
for. ⚠ One altered field produced three findings because three places in the
profile encode the major, and that is the coherence the check exists to hold.

### ⭐ The determinism leg, and why `verify` cannot see it

`b-ids-corpus verify` compares the committed index against ONE derivation, so a
generator that answered differently on alternate runs would fail it
intermittently and read as a flake. The new leg runs the generator twice over a
throwaway copy and compares the bytes. Planted by making the index writer append
its process id:

```text
$ sh scripts/common/check-validate.sh
validate check failed: the generator is not deterministic.

  index.json: two runs of the generator over one corpus wrote different bytes
  latest.json: two runs of the generator over one corpus wrote different bytes

A release nobody can reproduce is a release whose every run looks like a
change. Fix the generator, never this check.
exit=1
$ pwsh -NoProfile -File scripts/common/check-validate.ps1 -Json
{"schema":"check-validate/1","corpus":true,"profiles":1,"findings":0,"notcheckable":3,"deterministic":false,"problems":1}
```

⚠ **The mutation found a defect in the check's own message**, which is the
second thing a guard mutation is for: the two findings arrived on one line
because the accumulator joined them without a separator. A command substitution
strips trailing newlines, so the separator is a literal one now.

### ⚠ What this entry does NOT assert, said rather than implied

- **the generated formats' round trip.** There is one generator in this tree and
  it writes the index and the pointer file. `SCHEMA-08` is what adds the rest,
  and it adds them to the determinism leg in the same change.
- **the emitters' recorded fingerprints.** `EMIT-01` has not been built, so
  there is nothing to assert and asserting an absence would be theatre.
- ⚠ **the workflows running on a real runner.** ⛔ This is the part of gate (b)
  this host structurally cannot do: a workflow's real behaviour is only
  observable on the provider. Both files parse with `yq` here and the checkouts
  were read back from the parsed document, which is what a local host can prove.
  The remote run is confirmed at the session's close, per
  [`RULES.md`](RULES.md) section 10 step 11.

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
**Category** ci, **Priority** P1, **Effort** L, **Status** done

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


### Closing

**Closed 2026-09-01T15:05:00Z.** The matrix fans out from a plan that lives in
the tree, every lane is allowed to fail alone, and the collect job runs
regardless.

```text
$ sh scripts/common/check-workflows.sh --assert-fail-fast-false
workflows ok: 3 file(s), 7 job(s), every matrix declares fail-fast: false
exit=0
$ sh scripts/common/check-workflows.sh --json --assert-fail-fast-false
{"schema":"check-workflows/1","workflows":3,"jobs":7,"problems":0}
exit=0
$ pwsh -NoProfile -File scripts/common/check-workflows.ps1 -Json -AssertFailFastFalse
{"schema":"check-workflows/1","workflows":3,"jobs":7,"problems":0}
exit=0
```

### ⭐ The fixture, and every rule seen to fire

⛔ **A guard whose test has never been seen to fail is theatre.** A fixture
workflow breaks each rule exactly once, so a run over it reporting fewer than
five problems is a run whose checker has stopped holding one of them.

```text
$ sh scripts/common/check-workflows.sh --assert-fail-fast-false --fixtures FIXTURE
workflow check failed, 5 problem(s) over 1 workflow(s) and 2 job(s):

  bad.yml: uses actions/checkout@v4, which is not a 40-character commit. A moved tag runs code nobody reviewed.
  bad.yml: job lane: no timeout-minutes. A hung step holds a runner for the platform default.
  bad.yml: job lane: declares a matrix and does not declare fail-fast: false. One lane failing cancels its siblings.
  bad.yml: job collect: needs the fan-out job lane and does not run regardless. A collect job that only runs when every lane passed publishes nothing on the nights it matters.
  bad.yml: declares no top-level permissions. The default is whatever the repository grants.
exit=1
$ sh scripts/common/check-workflows.sh --json --assert-fail-fast-false --fixtures FIXTURE
{"schema":"check-workflows/1","workflows":1,"jobs":2,"problems":5}
exit=1
$ pwsh -NoProfile -File scripts/common/check-workflows.ps1 -Json -AssertFailFastFalse -Fixtures FIXTURE
{"schema":"check-workflows/1","workflows":1,"jobs":2,"problems":5}
exit=1
```

⚠ **`FIXTURE` is a directory outside the repository**, and the paths in the
messages are elided to the filename. The fixture is walked with `find` rather
than with `git ls-files`, which answers a path outside the repository with an
empty list: `check-routes` reported "ok, 0 files" over exactly such a fixture in
both halves, and this check was written knowing it.

### ⛔ The parser bug the fixture found, in the checker itself

An uninitialised awk variable used as a SUBSCRIPT is the empty string, not zero.
`names[njobs]` on the first job of every file wrote `names[""]` and left
`names[0]` unset, so the end-of-file loop read an empty name and reported a job
that does not exist:

```text
  .github/workflows/capture.yml: job : no timeout-minutes.
  .github/workflows/ci.yml: job : no timeout-minutes.
  .github/workflows/validate.yml: job : no timeout-minutes.
```

⚠ **The PowerShell half never had it**, because it appends to a list rather than
indexing an array. ⭐ A difference the twin comparison would have reported as a
drift, found first by reading the output.

### ⭐ The `always()` rule is about collecting, not about needing

The rule as first written fired on any job with `needs:`, which would have
refused the `plan` job's own dependent. That is wrong: a lane that runs after a
failed plan step is a lane with no plan. ⭐ It fires only where a job depends on
one that **fans out**, which is exactly the collect job whose whole value is
publishing what the lanes managed.

### What the matrix does, and what it deliberately does not

| | |
| --- | --- |
| ⭐ the plan lives in the tree | [`../.github/capture-matrix.json`](../.github/capture-matrix.json). The `plan` job reads it and the lanes fan out from `fromJSON`, and `check-coverage` reads the same file to say what landed. ⛔ A matrix written into the workflow and a report written from somewhere else is a value in two places with no check that they agree. |
| ⭐ a lane with no browser is exit 2 | "this runner has no browser" and "the capture failed" are different facts. The lane records the resolver's code and skips the capture on 2 rather than failing. `CI-07`. |
| ⭐ the capture path is the one a person runs | the lane runs `experiments/10-first-profile.sh`, which is also `CI-08`'s manual equivalent. Two pipelines is two things to keep correct and one of them stops being run. |
| ⛔ no lane writes to the repository | every job keeps `contents: read`. A lane runs a browser it downloaded, and that is the last thing that should hold a write token. `CI-04` is where a write belongs, on the collect job alone. |
| ⛔ the fuzz lane overrides the toolchain | `RUSTUP_TOOLCHAIN: nightly`, explicitly. `rust-toolchain.toml` pins an exact stable and applies to `fuzz/` too, so a nightly image is not enough. [`../fuzz/README.md`](../fuzz/README.md) carries the measurement that cost a run. |

⚠ **The action pins were RESOLVED rather than recalled**, and their declared
runtimes were read at the pinned commit. The v4 artefact actions still declare
`node20`, which the platform is deprecating, so this workflow pins
`upload-artifact` v7.0.1 and `download-artifact` v8.0.1, both `node24`.
⛔ `ci.yml` and `validate.yml` are unchanged on that point and still use only
`checkout`, which is already `node24`.

### ⚠ What is NOT proved here

⛔ **No lane has run on a runner.** This host can parse the workflow, hold every
structural rule and see the checker refuse a fixture; it cannot observe a hosted
runner. `CORPUS-02` is the entry that runs the matrix and is open with that
named as its blocker.

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
**Category** ci, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-02T03:40:00Z.** Both halves of `check-exit-codes` invoke every
script in the tree with an argument no script accepts and assert the answer is
**2**, and 22 PowerShell scripts were fixed to give it.

```text
$ sh scripts/common/check-exit-codes.sh
exit codes ok: 22 script(s), each answers 2 for an argument it cannot act on
exit=0

$ pwsh -NoProfile -File scripts/common/check-exit-codes.ps1
exit codes ok: 22 script(s), each answers 2 for an argument it cannot act on
exit=0
```

### ⛔ The finding: every PowerShell half answered 1, 22 pairs of 22

⚠ **Measured before the check was written, and it is why the check exists.**
Every POSIX half returned 2 for an argument it cannot act on. Every PowerShell
half returned **1**, because `pwsh -File` reports a parameter-binding failure as
1 and PowerShell rejects an unknown parameter in `param()` before a line of the
script body runs.

⛔ **1 is this project's code for "it ran and the thing failed".** So both halves
of every pair disagreed about whether a state is a failure, which is the exact
defect this entry is about. ⚠ **And `check-twins` could not see it**, because it
compares the JSON of runs that SUCCEED: a pair that differs only in how it
refuses is invisible to it. That pair has a row now.

⭐ **The fix is a remaining-arguments parameter** in every `param()` block, which
catches what would otherwise fail to bind, plus four lines that report and exit
2.

### ⚠ The name of that parameter cost one clean run

⛔ **`$Rest` collided.** PowerShell variables are case-insensitive and
`check-markers.ps1` already used a local `$rest` thirteen times; the parameter
shadowed it and took the script from a clean run to `Cannot convert value "{" to
type "System.Int32"`. ⭐ Renamed to `$UnboundArguments`, which nothing in the
tree uses, checked by grep across every script rather than by looking at one.

### Why an unknown argument is the input

⛔ **It is the one state EVERY script can be put into from outside**, with no
missing tool, no missing browser and no network. ⚠ A check that needed a real
unrunnable condition per script would have as many special cases as scripts, and
the ones it could not construct would go unchecked.

⛔ **0 is not accepted either.** A script that ignored an argument it does not
understand and ran anyway is worse than one that refused: it did something other
than what it was asked to do and reported success.

### ⛔ The guard was seen to fail, twice

**Planted**, in `check-msrv.ps1`, one character changed:

```text
exit code check failed, 1 script(s) did not answer 2:

  scripts/common/check-msrv.ps1: exit 1, and could-not-run is 2

Exit 2 is could-not-run. 1 is it ran and the thing failed, and 0 is it ran
and passed. TODO/ci.md, CI-07.
exit=1
```

Restored, and the same command answered 0.

⭐ **And the check carries its own fixture leg**, run on every invocation.
[`../scripts/README.md`](../scripts/README.md) describes what it plants and why
the check refuses to report at all when that leg comes back wrong.

### ⚠ What each half covers, and it is not the same set

⛔ **The `.sh` half checks the `.sh` scripts and the `.ps1` half checks the
`.ps1` scripts.** A POSIX half that shelled out to `pwsh` would report a green
half of a pair as the whole pair on a host without it, and
[`../scripts/README.md`](../scripts/README.md) is the contract that says
`check-twins` runs both halves on one machine.

### The lane half of this entry was already true

`capture.yml`'s resolve step already reads `rc` and accepts `0` or `2`, so a
runner with no browser does not fail the build, and the collect job already
names the lanes that produced nothing. ⭐ What was missing was the rule holding
in the scripts, and that is what closed here.

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
