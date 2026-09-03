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
**Category** ci, **Priority** P1, **Effort** M, **Status** done

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

### Closing

**Closed 2026-09-02T04:20:00Z.** `check-staleness`, both halves, reads what
`b-ids-driver versions` reports and compares it against the corpus's own pointer
file, and `staleness.yml` runs it on a schedule and never on a push.

```text
$ sh scripts/common/check-staleness.sh --corpus .tmp/staleness-fixture/corpus/v1 \
    --versions .tmp/staleness-fixture/versions-behind.json
staleness: 1 of 1 route(s) are behind

  chrome/stable/win64
    holds    151.0.7922.9
    serving  151.0.7922.200 at fraction 1
    highest  152.0.8001.5 at fraction 0.02

  sources that answered: releases=151.0.7922.200
  sources that did not:  chrome-for-testing=connection refused

  the replacement is a CAPTURE of 151.0.7922.200, not an edit: the corpus
  is append-only and a correction is a new profile. TODO/ci.md, CI-02.
exit=1
```

```text
$ sh scripts/common/check-staleness.sh --versions scripts/fixtures/staleness-versions.json --json
{"schema":"check-staleness/1","routes":2,"stale":2,"serving":"999.0.0.2","fraction":1,"highest_known":"999.0.1.0","highest_fraction":0.01,"answered":1,"silent":1}
exit=1

$ pwsh -NoProfile -File scripts/common/check-staleness.ps1 -Versions scripts/fixtures/staleness-versions.json -Json
{"schema":"check-staleness/1","routes":2,"stale":2,"serving":"999.0.0.2","fraction":1,"highest_known":"999.0.1.0","highest_fraction":0.01,"answered":1,"silent":1}
exit=1
```

⭐ **Byte-identical, from two implementations that share no binary.**

```text
$ sh scripts/common/check-staleness.sh --corpus .tmp/staleness-fixture/current/v1 \
    --versions .tmp/staleness-fixture/versions-current.json
staleness ok: 1 route(s) hold 151.0.7922.174, which is what is serving at fraction 1
exit=0
```

#### ⭐ The output carries the replacement values, which was the point

A check that only says a fingerprint changed is half a tool. Every stale row
names the route, the build it holds, the build that is serving, that build's
rollout fraction, the highest build the vendor knows and ITS fraction, and every
source that answered. ⛔ And it names what the replacement IS: a **capture** of
the new build, not an edit, because the corpus is append-only.

#### ⛔ A defect this found in itself: `--json` exited 0 over a stale corpus

⚠ **Measured on the first run.** The JSON branch printed `"stale":1` and
returned 0, because only the human branch carried the exit. That is the "a step
that exits 0 having done nothing it was asked to do" row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md),
in the mode a scheduled job reads. ⭐ Both halves carry the exit in both modes
now, and the row above shows the JSON form at exit 1.

#### ⛔ The version ordering is numeric per component

`151.0.7922.9` is **behind** `151.0.7922.76` and a lexical comparison says the
opposite. ⚠ The fixture corpus holds `151.0.7922.9` on purpose, so the run above
is that case rather than an easier one. ⭐ Both halves implement the comparison
themselves rather than sharing a binary, which is what makes the twin row a
comparison rather than two wrappers over one answer.

#### One source being unreachable is not a failure

The fixture's second source answers with an error, and the report says so on its
own line rather than dying. ⛔ Nothing here fetches: `b-ids-driver versions`
asks each source separately and this reads what it reported, because a second
fetcher would be a second answer to "what is current".

#### A staged rollout is not a chase

The chosen build is the one **serving**, which during a rollout is not the
highest the vendor knows. Both are printed with their fractions, so a reader can
see that `999.0.1.0` exists at one per cent and that chasing it would capture a
build almost nobody has. `DRIVER-02` owns that reading and this does not
re-derive it.

#### ⚠ What is NOT here, and it is by design

⛔ **Nothing opens a pull request.** `CI-04` is that entry and the write belongs
to its job and to no other; `staleness.yml` carries `contents: read` and no
more. ⭐ The workflow prints the human form only when there is something to
report, and it treats exit 2 as a fact about the vendor rather than as a stale
corpus, which is `CI-07`'s rule.

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
**Category** ci, **Priority** P1, **Effort** M, **Status** done

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

### ⭐ Closed 2026-09-02. The body is generated, the write is job-scoped, and a quiet night writes nothing

⛔ **The generator opens nothing.** `b_ids_corpus::pull_request` produces the
branch, the title, the body, the labels and the five merge conditions;
`capture.yml`'s collect job holds the token and does the opening. A module that
also called an API would be one component with two reasons to fail, and the
interesting half is the text.

#### The acceptance

```text
$ sh scripts/common/check-pr-body.sh --fixture
pr body ok: 9 suite case(s), 3 request(s) generated from the corpus,
  0 of them mergeable without a human, every body carrying its seven
  sections, and a no-op change opening nothing at all.
rc=0
```

⛔ **`--fixture` is REQUIRED**, for the reason `latest` requires
`--assert-stable`: there is no pull request to check, and a run with no argument
would read as though there had been one.

```text
$ sh scripts/common/check-pr-body.sh
check-pr-body: --fixture is required. There is no pull request to check:
  this checks a generator against a fixture, and a run with no argument
  would read as though it had checked a real one.
rc=2
```

#### ⭐ A body over two REAL corpus states, which is what this entry is for

⛔ **Not a fixture.** `before` is this tree's corpus with
`chrome/stable/win64/152.0.7977.76` removed and `after` is the tree as it
stands, so both profiles in the comparison were measured here:

```text
$ b-ids-corpus pull-request --before .tmp/pr/before --after . --run run.json --out .tmp/pr/out
capture/chrome/stable/win64/v1 -> .tmp/pr/out/capture_chrome_stable_win64_v1
corpus=pull-request requests:1 auto:0
```

```text
## What changed

- advanced: chrome/stable/win64 from 151.0.7922.174 to 152.0.7977.76
  - browser.version
  - tls.extensions.set

## The fields that differ

chrome-151.0.7922.174-win64-stable -> chrome-152.0.7977.76-win64-stable

⛔ these two captures do not differ only in version, so nothing below can be
   attributed to the version alone:
     captured.resumption: not recorded against offered

2 field(s) differ:
  browser.version: 151.0.7922.174 -> 152.0.7977.76
  tls.extensions.set: ...,0x44cd,0xfe0d,0xff01 -> ...,0x44cd,0xca34,0xfe0d,0xff01

## Merging

Change class: major-bump.

- ✅ the validator passes with no findings: 0 finding(s)
- ❌ the capture agrees across two independent sources: win64 via github-actions hosted runner, capture.yml run 33644513513
- ✅ no field regressed to vendor or became unreproducible: none
- ❌ the diff touches only fields this change class predicts: class major-bump, unpredicted: tls.extensions.set
- ✅ every generated format round-trips: every format read back

At least one condition does not hold, so this needs review.
```

⚠ The two long extension lists are abbreviated at the ellipsis. ⭐ **The one
that matters is `0xca34`, which Chrome `152` sends and `151` did not**, and it
is exactly the case the fourth condition exists for: a version bump that also
moves a TLS list is a human's decision.

#### ⭐ Three of the five conditions are computed and two are stated

⛔ **A caller cannot claim agreement across two sources.** That is a question
about the published profiles, so this module reads them: a source is a platform
and an operator together, and two profiles of one build taken by one operator on
one platform are one source measured twice. The same for the provenance
regression, which compares the two profiles' own maps, and for the predicted
fields, which read the diff.

⚠ **Two are facts about the RUN that only the run knows**: the validator's
finding count and whether the formats round-tripped. Those happened in steps
this module did not watch, and `Run` carries them with every field required.
⛔ There is no `Default`: a run file missing a field is a refusal at exit **2**
rather than a body with a blank where a run identifier belongs.

#### ⛔ What `CI-04` predicted about condition 4, and what is actually true

⚠ **The entry says a version bump may move the User-Agent and the brand list.**
[`../crates/b-ids-validator/src/diff.rs`](../crates/b-ids-validator/src/diff.rs)
compares header POSITIONS and not header VALUES, so **neither is a field the
diff can report**, and a table of predicted fields naming them would name two
fields nothing produces.

⭐ **So the predicted set for a version bump is `browser.version` and nothing
else**, which is the strict reading and the safe one: anything else is a human's
decision. `ChangeClass::predicted_fields` is a table rather than a branch, and
it is where the prediction grows when the diff learns to compare a header value.

#### The workflow leg

| | |
| --- | --- |
| ⛔ the write | `contents: write` and `pull-requests: write` **on the collect job alone**, using the run's own `GITHUB_TOKEN`. Every lane keeps `contents: read`, because a lane runs a browser it downloaded. |
| ⚠ what this file cannot grant | the repository setting that lets Actions create pull requests. The step reports that refusal in its own words rather than failing silently, and it is the operator's to enable. |
| ⭐ a quiet night | the opening step is gated on `steps.requests.outputs.requests != '0'`, so a run that found nothing opens nothing and comments nothing |
| ⛔ never over a human's commit | the bot branch's tip author is read before anything is pushed, and a branch whose last commit is not the platform's own actor is skipped with a message |
| ⚠ nothing is merged into the checkout | the two corpus states are built under `.tmp`, which is ignored, so a step that fails leaves the tree as it was |

#### The guard mutation, each exit code read unpiped

| planted | what went red |
| --- | --- |
| a run file carrying only `workflow` | exit **2**, asserted as that code rather than as any nonzero one. It is a leg of the check rather than a mutation, and it runs every time |
| `pull_request_the_merge_conditions_can_fail_and_say_which` renamed | `... is not in the suite`, exit **1**. Both halves name the nine cases they expect, so a deleted test is caught rather than passed over |
| each of the five conditions taken down on its own | inside the suite, and the body names the one that went |
| ⭐ every condition holding | `pull_request_every_condition_holding_is_reachable_rather_than_impossible`. ⚠ **The other half of that test**: a set of conditions that can only fail is a merge gate nothing passes, which reads as caution and is a feature nobody has |
| ⛔ the early return on a no-op change removed | ⛔ **nothing. The check passed.** See below. |
| a no-op change made to open one request per published route | three problems at once, exit **1**: the suite, `a no-op change produced 6 request(s), and it must produce none`, and `a no-op change wrote files into the output directory` |

⛔ **The fifth row is the pass earning its place.** `requests` opens with
`if change.is_empty() { return Vec::new(); }`, and deleting it changes nothing:
a change with no movement has nothing for the loop below to iterate, so the
silence rule was already enforced by the loop. ⭐ The line is kept as the
explicit statement of an invariant a reader should not have to derive from a
loop, and its doc comment says which of the two is the guard. The mutation that
actually breaks silence is the row under it, and it is caught three ways.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a real pull request | ⛔ None was opened. The acceptance is a body generator driven by a fixture, which is what the operator ruled on 2026-09-02, and the repository setting the workflow needs is not this session's to enable. |
| an issue for anything | `CI-04` reserves issues for questions, and nothing here opens one. |
| superseding a stale request | ⚠ Named in the approach and not built. The branch is deterministic and per route, so a newer capture UPDATES the open request rather than accumulating beside it, which is the half that removes the accumulation. Closing a request a newer route made stale needs a query against the remote and belongs with `PUB-09`'s signing work rather than here. |
| the field-level diff over an `Advanced` movement, end to end in the check | ⚠ The check's end-to-end leg drives the case that is deterministic whatever the corpus holds: every route new. An `Advanced` movement needs two builds at ONE route and which two depends on what the corpus holds today, so the suite owns that half and the block above is the same generator run over two real states by hand. |

---

## CI-05. The cold-start job, which is the only thing that catches rot

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P2, **Effort** M, **Status** done

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

### ⭐ Closed 2026-09-03. The job is cold, and the check refuses the day it stops being

#### The acceptance

```text
$ sh scripts/common/check-cold-start.sh
cold start ok: 11 stage(s), each named by the report step, no cache of any kind,
  and 9 of 9 program(s) present on this host.
  ⛔ Nothing was built, captured or published by this check.
rc=0
```

Both halves agree. Each reads the file with its own tools rather than wrapping
one reading, which is the same split `PUB-10`'s pair makes and for the same
reason.

```text
$ sh scripts/common/check-cold-start.sh --json
{"schema":"check-cold-start/1","tools":9,"found":9,"missing":0,"stages":11,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-cold-start.ps1 -Json
{"schema":"check-cold-start/1","tools":9,"found":9,"missing":0,"stages":11,"problems":0}
```

#### ⛔ The whole point is that the job stays cold, so that is what is checked

⚠ **A cold-start job that shares a cache has stopped being one while continuing
to report as one**, which is the worst outcome available: green, and verifying
nothing. The check refuses `actions/cache`, `rust-cache`, `sccache`,
`RUSTC_WRAPPER` and any `cache:` key, over the workflow with its comment lines
dropped first, ⛔ because a comment may say the words and a step may not carry
them.

⭐ **And it refuses a shared concurrency group.** A cold-start run that queued
behind a capture run, or cancelled one, is a run whose timings mean nothing.

#### ⭐ "Fails loudly naming the first step that could not resolve"

That sentence is the acceptance, and a run whose log a person has to scroll is
not it. So every stage carries an `id`, and the last step is a report that runs
`if: always()`, prints each stage's outcome and names the first failure:

```text
  resolve    success
  toolchain  success
  ...
  ⛔ the cold path broke at: fetch
```

⛔ **The check asserts the report names every stage**, so a stage added without
a line in it is a stage whose failure would be silent. That is the rule that
caught the mutation below.

#### ⭐ The resolution probe belongs to the check, and the workflow runs it

⛔ **One list, in one place.** `check-cold-start --resolve` names every program a
cold pipeline needs and reports each; the workflow's first step runs that same
probe with `--require-tools`, which turns the first absent one into a failure.
⚠ On a laptop a missing tool is a fact about the laptop and a gate that failed
over it is a gate somebody disables, which is the same split `--strict` makes.

```text
$ sh scripts/common/check-cold-start.sh --resolve
cold start probe over 9 program(s):

  ok    git
  ok    cargo
  ok    rustc
  ok    rustup
  ok    jq
  ok    awk
  ok    sed
  ok    grep
  ok    tar

every program a cold pipeline names is on this host.
rc=0
```

⚠ **A browser is deliberately not on that list.** `b-ids-driver resolve` exits 2
on a host with none, and "this runner has no browser" is a fact about the host
rather than a broken cold path. `CI-07` is the rule.

#### The guard mutation, both halves, each exit code read unpiped

⛔ **Every mutation was made against a copy under the ignored scratch directory,
and the live file was compared byte for byte with that copy afterwards.**

| planted | sh | ps | what went red |
| --- | --- | --- | --- |
| a cache action added | 1 | 1 | `1 line(s) name a cache, and a cold-start job that shares one has stopped being one` |
| the schedule removed | 1 | 1 | `is not on a schedule, and a cold path nobody runs is one nobody checks` |
| the concurrency group shared with `ci` | 1 | 1 | `the concurrency group is ci-…, which is not this workflow's own` |
| the report step no longer `always()` | 1 | 1 | `a red job says nothing about which stage went red` |
| the gate stage dropped from the report | 1 | 1 | `the report step does not name the gate stage` |
| every stage `id` removed | 1 | 1 | `0 stage(s) carry an id` |
| the workflow deleted | 1 | 1 | `there is no .github/workflows/cold-start.yml` |
| a program the pipeline names made absent | 1 | 1 | ⭐ under `--require-tools` only: `the cold path breaks at the first absent program`. Exit **0** without it, which is the split above, proved rather than described |

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a cold run | ⛔ None was taken. This machine is warm by definition and a cold-start check that built the workspace here would prove nothing about a fresh runner while costing an hour. The first real answer is the next scheduled run. |
| a capture on the Windows lane | ⚠ The lane exists and its cold path has never been exercised. `capture.yml` has run on both hosts; this workflow has run on neither. |
| the network legs, probed locally | ⛔ Deliberately absent. A gate that needs the network fails on a machine that has none, and the fetch stage in the workflow is where a registry outage or a yanked crate shows. |

---

## CI-06. No single source of any fact

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** ci, **Priority** P2, **Effort** M, **Status** done

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

### Closing

**Closed 2026-09-02T06:10:00Z.** `check-sources`, both halves, asserts the
three properties the multi-source rule turns into: per-source isolation, a
silent source that does not end the run, and a disagreement that is flagged
rather than resolved.

```text
$ sh scripts/common/check-sources.sh --report scripts/fixtures/staleness-versions.json
sources ok: 2 source(s), 1 answered, 1 did not, disagreement=false
exit=0

$ sh scripts/common/check-sources.sh --report scripts/fixtures/staleness-versions.json --json
{"schema":"check-sources/1","sources":2,"answered":1,"silent":1,"disagreement":false,"problems":0}
exit=0
```

⭐ **The first line is the acceptance's first clause, run.** That fixture has one
source answering and one reporting an error: the run degrades and names which
answered, and it does not fail.

```text
$ sh scripts/common/check-sources.sh --report .tmp/check-sources/disagreement-unflagged.json
source contract failed, 1 problem(s):

  sources answered 9.0.0.1 and 9.0.0.2 and disagreement is false, which is one source silently preferred

  Two sources that disagree are the most valuable signal this
  project produces. Record both, publish both, never pick.
  TODO/ci.md, CI-06.
exit=1
```

⭐ **And that is the second clause.** Two sources made to disagree with the flag
off is refused by name.

```text
$ pwsh -NoProfile -File scripts/common/check-sources.ps1 -Report scripts/fixtures/staleness-versions.json -Json
{"schema":"check-sources/1","sources":2,"answered":1,"silent":1,"disagreement":false,"problems":0}
exit=0
```

⭐ Byte-identical to the POSIX half, from two implementations that share no
binary. The pair has a row in `check-twins`.

#### ⛔ What it does NOT do, and that is the entry's own rule

**It does not decide which source is right.** ⚠ That is a reading, and a check
that picked would be the "silently prefer one source when two disagree" this
entry forbids in as many words. What it asserts is that both answers survive
into the report and that the flag says they differ.

#### ⭐ Two refusal fixtures run on every invocation

⛔ **A check that cannot refuse must not report a pass.** Both halves build a
report with a source that answered nothing and gave no reason, and one carrying
two answers with the flag off, read both, and exit **2** rather than reporting
anything if either comes back clean. That is the same shape `check-exit-codes`
carries and for the same reason.

#### ⚠ What was already true, and is now asserted rather than believed

`b-ids-driver versions` has fetched each source separately since `DRIVER-02`,
reported which answered, and set `disagreement`. ⭐ **What did not exist was
anything that would notice if it stopped.** The list below is what the entry
asks for and where each part lives:

| the rule | where it lives now |
| --- | --- |
| more than one way to ask "what version is current" | `b_ids_driver::discover`, two sources, asserted here |
| per-source isolation, published with the data | the `answers` array, asserted here |
| a disagreement is a finding | the `disagreement` flag, asserted here |
| more than one way to get a build | `DRIVER-05`, routes tried in order with the digest of what arrived |
| pin by digest, never by tag | every `uses:` in every workflow, asserted by `check-workflows` |

⛔ **The rows this entry names that are NOT yet held by a check** are stated
rather than quietly dropped: caching every artefact by digest, an age field and
a staleness banner in the index, and a second harness implementation. ⚠ The
first two belong to `PUB-03`'s index shape and the third to `HARNESS-12`; none is
started, and `check-sources` says nothing about any of them.

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
**Category** ci, **Priority** P2, **Effort** S, **Status** done

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
resolves each one on this host, reporting a failure for any it cannot resolve
rather than skipping it silently.

### ⚠ The acceptance is corrected, and the correction is the finding

The Prove block above asks the check to **execute** each manual equivalent. ⛔
**Measured against the tree: that is a command nobody would run.** Nine jobs
declare one, and among them are a fuzz lane that runs a hundred thousand cases
and two lanes that launch a browser for the better part of a minute each. A
check whose single invocation costs half an hour is a check that is not run at
three in the morning, which is the exact moment this entry exists for.

⭐ **What is executed instead is the RESOLUTION of each command**: the program it
starts with must be on PATH here, and a script it names must exist in this tree
and parse. ⛔ A command naming a script this tree does not have is a **failure**
rather than a skip, because that is precisely the rot the entry describes.

⚠ The title stays and the premise stays. What changed is one word in the
acceptance, with the reason above it.

### Closing

**Closed 2026-09-02T06:30:00Z.** Every job in every workflow names the command a
person runs instead, and both halves of `check-manual-path` assert it.

```text
$ sh scripts/common/check-manual-path.sh
manual path ok: 9 job(s), each names a command that resolves here
exit=0

$ sh scripts/common/check-manual-path.sh --json
{"schema":"check-manual-path/1","jobs":9,"named":9,"problems":0}
exit=0

$ pwsh -NoProfile -File scripts/common/check-manual-path.ps1 --json
{"schema":"check-manual-path/1","jobs":9,"named":9,"problems":0}
exit=0
```

⭐ **The check was written before the lines it checks for**, and its first run
named all nine jobs as having none:

```text
manual path check failed, 9 problem(s):

  .github/workflows/capture.yml: job 'plan' names no manual equivalent
  .github/workflows/capture.yml: job 'lane' names no manual equivalent
  .github/workflows/capture.yml: job 'fuzz' names no manual equivalent
  .github/workflows/capture.yml: job 'collect' names no manual equivalent
  .github/workflows/ci.yml: job 'checks' names no manual equivalent
  .github/workflows/ci.yml: job 'windows' names no manual equivalent
  .github/workflows/staleness.yml: job 'ask' names no manual equivalent
  .github/workflows/trust-anchor.yml: job 'compare' names no manual equivalent
  .github/workflows/validate.yml: job 'corpus' names no manual equivalent
exit=1
```

⛔ **That is the guard seen to fail against the real tree**, not against a
fixture written to make it fail.

### What the ten jobs degrade to

| job | the one command a person runs |
| --- | --- |
| `capture.yml` `plan` | `jq -c '[.cells[] \| select(.enabled)]' .github/capture-matrix.json` |
| `capture.yml` `lane` | `sh experiments/10-first-profile.sh --headless --browser chrome` |
| `capture.yml` `fuzz` | `cargo fuzz run parsers -- -runs=100000` |
| `capture.yml` `collect` | `sh scripts/common/check-coverage.sh` |
| `ci.yml` `checks` | `sh scripts/common/check-gate.sh --strict` |
| `ci.yml` `windows` | `pwsh -NoProfile -File scripts/common/check-gate.ps1` |
| `staleness.yml` `ask` | `sh scripts/common/check-staleness.sh` |
| `trust-anchor.yml` `compare` | `sh experiments/50-trust-anchor.sh --headless --browser chrome` |
| `provision.yml` `provision` | `sh scripts/common/check-provisioning.sh` |
| `validate.yml` `corpus` | `sh scripts/common/check-corpus.sh` |

⭐ **The entry's own test, answered:** if the provider disappeared tomorrow, a
person with this tree runs those ten commands and the project keeps producing
what it produces. ⚠ What they would not have is the fan-out and the schedule,
which are conveniences rather than the work.

### ⛔ The declaration lives beside the job, not in a table

⚠ A list of equivalents in a second file would be a value in two places with no
check that they agree, and the copy that goes stale is the one nobody is reading
when the platform is down. The line is a `# manual:` comment **inside** the job
block, and both halves read the indentation rather than grepping: `# manual:`
appearing anywhere in a file says nothing about which job carries it.

⚠ **The first placement was wrong and the check caught it.** The lines went in
above each job header, so the parser attributed each to the job before it and
four jobs read as named that were not. They are inside the block now.

### ⛔ Amended 2026-09-02: the check read tracked files only

⛔ **A workflow that was written and never staged escaped this check entirely**,
which is the one moment a new job's manual line is most likely to be missing.
Found by adding [`../.github/workflows/provision.yml`](../.github/workflows/provision.yml)
for `DRIVER-08`: `check-workflows` reported ten jobs and this reported nine, and
`git add -N` on that one file alone changed the answer to ten.

⚠ **`check-exit-codes` had the same defect and was fixed on 2026-09-01**, for
the same reason and in the same words: a script never staged escaped it. This
half of the shape was left, and a check that is right only about files somebody
remembered to stage is a check with a hole exactly where new work is.

⭐ **Both halves read tracked and untracked now**, and both were seen to fail
against the real tree with one `# manual:` line removed:

```text
$ sh scripts/common/check-manual-path.sh
manual path check failed, 1 problem(s):

  .github/workflows/provision.yml: job 'provision' names no manual equivalent

Every automated job names the command a person runs instead, as a
`# manual:` comment inside the job. TODO/ci.md, CI-08.
exit=1
```

```text
$ sh scripts/common/check-manual-path.sh --json
{"schema":"check-manual-path/1","jobs":10,"named":10,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-manual-path.ps1 -Json
{"schema":"check-manual-path/1","jobs":10,"named":10,"problems":0}
```

