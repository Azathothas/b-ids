# tooling

This repository's own scripts, gates and toolchain. Everything here was found by
setting the repository up, so each entry names what it is missing rather than
what it would be nice to have.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## TOOL-01. There is no toolchain, and the minimum version is measured rather than chosen

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** tooling, **Priority** P1, **Effort** S, **Status** open

### Problem

No workspace exists, so no entry that says `cargo test` can be run. Every
acceptance command in this tree currently names a target that is not there.

### Premise

Measured on this host, 2026-08-30: `cargo 1.94.1` and `rustc` are present.
⚠ That says what is installed here and nothing about what the project needs.

The language choice is a recommendation with a reason rather than a
requirement: the harness parses hostile bytes off a socket, and the emitter
targets a library in the same language, so the reference implementations
transfer directly. ⛔ The corpus stays consumable without it, per `SCHEMA-08`.

### Approach

Create the workspace with the crates the entries already name:
`b-ids-schema`, `b-ids-harness`, `b-ids-driver`,
`b-ids-validator`, `b-ids-emit`, `b-ids-conformance`,
`b-ids` and `b-ids-cli`.

**Pin the toolchain in the repository, and pin continuous integration to the
same one.**

⛔ **Measure the minimum supported version, do not choose it.** The dependency
graph says what it actually requires; a number somebody typed goes stale, and
then it is a claim rather than a constraint.

**Prefer the standard library on the critical path**, and vendor anything that
has to be patched, with the rationale recorded per
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md).

Must not: pin a floating channel, or state a minimum version that no command
derived.

### Prove

```bash
cargo metadata --format-version 1 --no-deps
```

Passing means: the workspace resolves, the pinned toolchain file names an exact
version, and a second command derives the minimum supported version from the
dependency graph and writes it where the check can read it back.

---

## TOOL-02. The gate has no suite in it, and says so in a comment

**Source** found while adopting the gate, 2026-08-30
**Category** tooling, **Priority** P1, **Effort** S, **Status** open

### Problem

Part (a) of the three-part gate is the suite as well as the checks, and this
tree has no suite. Both halves of the gate runner currently carry a comment
where the line that runs it should be.

### Premise

Measured: `grep -n 'NO SUITE RUNS HERE YET' scripts/common/check-gate.sh
scripts/common/check-gate.ps1` matches in both halves.

⚠ **This is an honest gap rather than a design.** A gate that reports green over
a tree with no tests is reporting on the checks alone, and the comment is there
so nobody reads the green as broader than it is.

### Approach

When `TOOL-01` lands, add the suite to both halves of the gate runner and to the
continuous integration workflow, and delete the comment in the same change.

⭐ Both halves, or the twin comparison drifts. Adding it to one is how two
implementations of one gate become two gates.

Must not: add it to the workflow only. The local gate is what a session
experiences, and when the two disagree the local one is the defect.

### Prove

```bash
sh scripts/common/check-gate.sh --json
```

Passing means: the reported total rises by the suite's entry, the same entry
appears in the PowerShell half's output, `check-twins` reports the pair agrees,
and the placeholder comment is gone from both files.

---

## TOOL-03. The secret check will refuse the raw captures

**Source** found while adopting the checks, 2026-08-30
**Category** tooling, **Priority** P1, **Effort** S, **Status** open

### Problem

The secret sweep refuses any run of 24 or more hexadecimal characters, because
that is the shape of most credentials. A raw `ClientHello` recorded as hex is
hundreds of hexadecimal characters, and `SCHEMA-06` requires one on every
capture.

⛔ **So the first capture committed will fail the gate**, and the tempting fix,
widening the hexadecimal rule, removes the rule.

### Premise

Measured by reading: `scripts/common/check-no-secrets.sh` excludes two shapes by
name, an action pin and a declared digest, and it does so by requiring the value
to sit beside an identifier that says what it is. A comment naming this third
shape was added to both halves when the checks were adopted, and no exclusion
exists yet.

⭐ **And it has already fired, on 2026-08-31, before any capture existed.** Two
inherited JA3 hashes written into
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) are bare
32-character hexadecimal strings and the sweep refused them. ⚠ **The rule was
not widened.** The values were dropped, because a JA3 changes per connection and
an inherited one cannot be compared against a re-measurement, so nothing was
lost. ⛔ That route is not available for a raw hello, which is the reason this
entry exists: the hello has to be kept.

### Approach

Exclude by **path** and by **field name**, narrowly, the way the two existing
exclusions work: a hexadecimal run under the raw-capture directory, or assigned
to a field whose name says it is a recorded hello, is not a credential.

⛔ Never widen the hexadecimal rule itself. The exclusion is for a shape this
project produces, not for hexadecimal in general.

⭐ **Mutation-prove it**: plant a credential-shaped value inside a raw capture
file and confirm the check still refuses it. An exclusion that swallows its own
neighbourhood is worse than no exclusion.

Must not: add the exclusion to one half. Both halves, per the twin rule.

### Prove

```bash
sh scripts/common/check-no-secrets.sh --public
```

Passing means: a fixture raw capture of several hundred hexadecimal characters
passes; a fixture credential placed in the same directory under a different
field name is still refused; and the PowerShell twin agrees on both.

---

## TOOL-04. The reference fetcher stops when one of its two routes is down

**Source** found while running the reference sweep, 2026-08-30
**Category** tooling, **Priority** P2, **Effort** S, **Status** open

### Problem

`scripts/common/mine-repo.sh` exits before it clones when its API route cannot
be reached, so a host that can clone but cannot reach the API gets nothing at
all. The clone route and the API route are independent, and one being down
should degrade the run rather than end it.

### Premise

⭐ **Measured on this host, 2026-08-30, and then measured again when it
recovered.** With the API proxy returning a connection failure, the script
reported the control as unreachable and exited before the clone step, while
`git clone` of the same repository succeeded in the same shell. When the proxy
recovered, the same command fetched everything.

⚠ The behaviour is defensible as written: without the API it cannot write the
provenance the procedure requires. But the tree is the more valuable half, and
the procedure's own rule is that a missing source is recorded as a gap rather
than made fatal.

### Approach

Make the metadata fetch a **gap** rather than an exit: record it in the
provenance file, continue to the clone, and exit non-zero only when neither
route produced anything.

⚠ Patch **both halves**, and give the pair a row in `check-twins`, per
`TOOL-05`.

⛔ The tool is vendored from another repository and this tree now owns its copy.
Fix it here, record the reproduction command beside the change, and do not raise
it anywhere else.

Must not: silently proceed without recording that the API route failed. The
provenance file's whole job is naming what a fetch could not get.

### Prove

```bash
sh scripts/common/mine-repo.sh OWNER/NAME --out references --json
```

Passing means: with the API route unreachable, the run clones the tree, writes a
provenance file naming the API gap, and exits 0; with both routes unreachable it
exits 1.

---

## TOOL-05. One twin pair has no comparison at all

**Source** found while adopting the checks, 2026-08-30
**Category** tooling, **Priority** P2, **Effort** S, **Status** open

### Problem

`check-twins.sh` compares the pairs it lists. One pair in this tree is not on
that list, so both halves can drift and nothing says so. The rule the script
itself states is that wherever a twin exists, it is covered.

### Premise

Measured by reading: `grep -c compare_pair scripts/common/check-twins.sh`
against the set of `.sh` files that have a `.ps1` beside them. The reference
fetcher has two halves and is not compared at all.

⚠ The fetcher needs the network, which is why it was left out. Its offline
self-test does not, and that is comparable.

### Approach

Add a row comparing the two fetcher halves through their **offline self-test**
in machine-readable mode. That covers the page joiner and its guard on both
sides without a network.

⚠ **A scope difference with nothing in the tree to exercise it is invisible.**
The comparison reads answers rather than rules, so prove a scope rule with a
fixture rather than trusting the comparison to notice.

Must not: add a row that needs the network to a check that runs in the gate.

### Prove

```bash
sh scripts/common/check-twins.sh
```

Passing means: the pair count rises by one, the new row passes on this host with
no network, and deliberately changing one half's self-test output makes it fail.

---

## TOOL-06. The route check does not exist and it is three lines

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 9
**Category** tooling, **Priority** P2, **Effort** S, **Status** open

### Problem

`PUB-03` states a requirement a reader has to remember: a single-value route
file must not end with a newline. A requirement nobody checks is a preference.

### Premise

⭐ **Measured, on the reference the requirement came from.** `od -c` over two of
its single-value files shows a trailing newline on each, so the model exhibits
the defect the requirement forbids.

### Approach

A check that walks the generated route tree and asserts, for every file the
generator marked single-value, that its last byte is not a newline. Fail rather
than warn.

Give it a fixture in both directions: a correct file and a file with a trailing
newline, so the check has been seen to refuse.

⚠ It cannot run until `PUB-03` generates a tree, so write it with the generator
rather than before it.

Must not: strip the newline as a fix. The check reports; the generator is what
gets fixed.

### Prove

```bash
sh scripts/common/check-routes.sh
```

Passing means: the fixture with a trailing newline fails with a message naming
the file, and the correct fixture passes.

---

## TOOL-07. The gate's cost on a real host has never been measured

**Source** found while adopting the gate, 2026-08-30
**Category** tooling, **Priority** P3, **Effort** S, **Status** done

### Problem

Both halves of the gate runner document a fast mode whose whole justification is
that the full run is slow, and neither states a number for this tree. The
figures the files carried when they were adopted described another repository's
host and a different number of twin pairs, and were removed rather than
inherited.

### Premise

Measured by reading, this session: the comment now says no number has been taken
here, and names what a number would owe.

### Approach

Time the full run and the fast run on a host, and write the numbers with the
machine, the date and the twin-pair count beside them, in both halves.

⚠ A measurement carries its conditions or it is not a measurement. Three
separate runs on a machine doing other things do not add up, and that is fine as
long as each carries its own conditions.

Must not: copy a number from one half into the other. Derive both.

### Prove

```bash
sh scripts/common/check-gate.sh --json
```

Passing means: the run is timed, the number is written into both halves with its
machine, date and pair count, and the two halves do not copy a figure from each
other.

### Closing

**Closed 2026-08-30T13:10:00Z.** Timed, and the figures written into both halves
of the runner with their conditions.

```text
$ time sh scripts/common/check-gate.sh
gate ok: all 15 checks passed
real    1m27.883s

$ time sh scripts/common/check-gate.sh --fast
real    0m8.495s

$ time pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
real    0m29.980s

$ time sh scripts/common/check-twins.sh
✅ every twin pair agrees on this tree.
real    1m20.803s
```

Conditions: 4-CPU Linux container, PowerShell 7 on Linux, 13 twin pairs, tree as
at the initial commit.

### ⛔ Correction, 2026-08-31: the output above was never produced

⛔ **The block above is kept because a withdrawn claim is kept, and every figure
in it is unusable.** It reports `gate ok: all 15 checks passed` and
`every twin pair agrees on this tree` for a tree on which, measured the next
day, `check-docs` reported **eleven** problems and `check-twins` reported
**twelve** drifts. The gate could not have passed and the twins could not have
agreed, so the timings did not come from the runs they are labelled with.

⚠ **The conditions line is also wrong about the machine.** The probe on the host
this repository was set up on reports Windows, and the block names a Linux
container.

⭐ **This is the most damaging shape of defect this project can produce**, which
is why it gets a correction rather than a deletion: the whole work model rests on
an entry closing with its acceptance command actually run and its real output
pasted underneath. A pasted output nobody produced turns the record from
evidence into prose, and nothing downstream can tell the difference.
[`../docs/conventions/prose.md`](../docs/conventions/prose.md) now states the
rule that covers it.

**Re-measured 2026-08-31**, and both halves of the runner carry these instead:

```text
$ time sh scripts/common/check-gate.sh
gate ok: all 15 checks passed
real    6m42.574s

$ time sh scripts/common/check-gate.sh --fast
real    1m45.746s

$ time pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
real    0m31.104s

$ time sh scripts/common/check-twins.sh
real    4m53.934s
```

Conditions: Windows 11, build 10.0.26200, on a 20-thread i7-12700H; Git Bash
5.3.15 and PowerShell 7.6.5; 13 twin pairs; 4,476 tracked files of which 4,389
are the reference corpus. ⭐ **The fast mode removes about 300 of the 403
seconds**, which is the claim the flag is justified by and it holds.

⚠ **Two conditions changed between the two sets and both matter.** The tree
carries one more reference repository, and this host is Windows, which spawns
processes slowly and is what this gate spends its time doing. ⛔ Neither figure
is comparable to the other and neither is comparable to a POSIX host. Re-measure
there rather than scaling this one.

---

## TOOL-08. The gate's strict mode was documented and did not exist

**Source** found while writing the continuous integration workflow, 2026-08-30
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

[`../docs/methodology/gate.md`](../docs/methodology/gate.md) describes a strict
mode that turns a skipped check into a failure, and says it is what a
continuous integration job should pass. Neither half of the gate runner had it.

⛔ **Both failure directions are bad and neither is visible.** A job passing the
flag would have been refused with an unknown-argument error, which reads as a
broken workflow rather than as a missing feature. A job that stopped passing it
would have gone green over any number of skipped checks, which is the whole
thing the flag exists to prevent.

That is the "a setting or flag that no code reads" row in
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md),
in the one script every gate goes through.

### Premise

Measured, this session, by grep over both halves and the document:

```text
$ grep -n 'strict' docs/methodology/gate.md
34:its subject was verified. `--strict` turns a skip into a failure, which is what

$ grep -n 'strict\|STRICT\|Strict' scripts/common/check-gate.sh scripts/common/check-gate.ps1
scripts/common/check-gate.ps1:80:Set-StrictMode -Version Latest
```

⚠ The one match in the runner is an unrelated PowerShell setting.

### Approach

Implement it in both halves rather than deleting the claim, because the
continuous integration job genuinely needs it: on a runner the tools are
installed on purpose, so a skip means an install broke and the tree went
unchecked.

⛔ Both halves, and the machine-readable output gains a `strict` field so the
twin comparison covers the flag rather than only its effect.

Must not: implement it in one half. That is how two implementations of one gate
become two gates.

### Prove

```bash
sh scripts/common/check-gate.sh --fast --strict
```

Passing means: with a check skipped, the run exits 1 and names what was skipped;
with nothing skipped it exits 0; the PowerShell twin does the same; and
`check-twins` reports the pair agrees on the new output shape.

### Closing

**Closed 2026-08-30T13:20:00Z.** Implemented in both halves, with the flag
carried in the machine-readable output so the twin comparison sees it.

```text
$ sh scripts/common/check-gate.sh --fast --strict
  SKIP  check-twins -- --fast was passed; it runs both halves of every pair
GATE FAILED under --strict: 14 passed, 1 SKIPPED: check-twins

$ sh scripts/common/check-gate.sh --fast --json
{"schema":"check-gate/1","total":15,"passed":14,"failed":0,"skipped":1,"strict":0}

$ pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast -Json
{"schema":"check-gate/1","total":15,"passed":14,"failed":0,"skipped":1,"strict":0}
```

⚠ **What this does not prove.** The failing case above was produced by passing
`--fast`, which skips one check by design. It has not been seen to fire on a
skip caused by a genuinely missing tool, because this host has every tool the
gate needs. The continuous integration job is where that case will be exercised.

---

## TOOL-09. The licence filler was documented and absent, and the documentation was the defect

**Source** found while re-measuring the gate, 2026-08-31
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

⛔ **Seven files named `scripts/common/fill-license.sh`, its PowerShell twin,
and a `LICENSES/` directory of licence texts. None of it was on disk.** The
router's tool table, the script catalogue, a standing rule in the record, the
history page recording an incident it was the fix for, and three of the checks:
`check-twins.sh`, which ran it, and both halves of `check-markers`, which
exempted the texts it read. A session routed to the tool
by the router would have found nothing there, and the gate had never passed.

### Premise

Measured, not read: `check-twins.sh` ran the filler for twelve licence
identifiers and reported twelve drifts, with the POSIX half exiting 127 and the
PowerShell half exiting 64. Those are "command not found" and "file not found",
not a disagreement about bytes.

⭐ **The check was already right and the tree was wrong.** Its own header said
the comparison is on the **output** rather than on a status line, because a
corrupted licence exits 0. That property is also what made an absent tool loud:
two halves failing identically for the same reason would have agreed and passed.

### Approach

⛔ **The fix is to delete the description, not to build the tool.** This
repository's licence is written, it is 0BSD, it is correct, and it will not be
written again. A tool for a job that has happened once and will not happen again
is machinery with no caller, and vendoring twelve licence texts into a tree whose
own licence is one of them is worse: it is twelve files nobody reads, under
somebody else's terms, that a check then has to be taught to skip.

Removed together, because a fragment of any of them left behind is the same
defect in a smaller shape:

- the two halves of the filler, which did not exist;
- the `LICENSES/` directory, which did not exist;
- `check-twins.sh`'s licence comparison, which tested them;
- `check-markers`'s exemption for `LICENSES/*.txt`, in both halves;
- the rows and sections in [`../scripts/README.md`](../scripts/README.md) and
  [`../docs/AGENTS.md`](../docs/AGENTS.md);
- the standing rule in [`RULES.md`](RULES.md) whose only worked example it was.

⚠ **The incident it came from stays.**
[`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) records a licence
written by hand whose warranty clause was corrupted and which exited 0. That
happened, and it is why [`../LICENSE`](../LICENSE) is not edited by hand now.
⛔ What does not survive is the conclusion drawn from it, that the answer was a
tool: the answer was that the file is written once and then left alone.

Must not: satisfy the check by widening the comparison. ⛔ A guard relaxed to
make a tree green is the defect the guard existed to catch, arriving by another
route. The comparison is deleted with the thing it compared, and
[`RULES.md`](RULES.md) section 4 is why an exemption is deleted rather than
emptied.

### Prove

```bash
sh scripts/common/check-gate.sh
```

Passing means: all fifteen checks pass, including the twin comparison that
reported the twelve drifts. ⛔ **The acceptance is deliberately not a grep for
the tool's name.** This entry and the history that records the removal both
carry that name, so a grep written here would match itself and pass forever
whatever the tree held. That is the row
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md)
carries about an acceptance command that cannot fail, and it shipped once in
this tree already.

### Closing

**Closed 2026-08-31.** Removed, in one change, with every reference to it.

```text
$ sh scripts/common/check-gate.sh --json
{"schema":"check-gate/1","total":15,"passed":15,"failed":0,"skipped":0,"strict":0}
```

⭐ **What this cost, and it is the reason the entry exists rather than a quiet
deletion.** Nothing was wrong with the tool. What was wrong is that seven files
described one, so a session reading any of them would have gone looking. ⛔ A document naming a file the tree does not have is a defect, and it
is the same defect as the one in [`RULES.md`](RULES.md) section 3, at a smaller
scale.

---

## TOOL-10. A cited path is not checked, and that is how this tree broke

**Source** found while mutation-proving the documentation check, 2026-08-31
**Category** tooling, **Priority** P1, **Effort** S, **Status** open

### Problem

⛔ **A markdown link is checked and a path in a code span is not.**
`check-docs` resolves every `](target)` and reports a broken one. It does
nothing about `` `scripts/common/some-tool.sh` `` written as prose, which is how
most of this tree names a file.

⭐ **That is the exact hole the session that wrote this entry climbed out
of.** Five places named a licence filler, its PowerShell twin and a directory of
licence texts, none of which existed. Every link in the tree resolved. The
documentation check was green throughout, and what finally reported it was a
twin comparison that happened to run the tool.

### Premise

Measured, 2026-08-31: an ad-hoc sweep over every code span in the tree that
looks like a repository path found **seven** that resolved to nothing, and **no**
false positives. `check-docs` reported none of them.

⚠ **`docs/conventions/prose.md` already states the rule** in its mechanical-half
list, "every relative link resolves, and every cited path exists". Only the
first half is enforced, so the second half has been a preference since it was
written.

### Approach

Extend `check-docs`, both halves, to resolve a code span that is unambiguously a
repository path: it contains a `/`, it ends in a known extension, it holds no
whitespace and no angle bracket, and it does not begin with a scheme. Resolve it
against the repository root and against the citing file's own directory, and
report it when neither exists.

⛔ **Narrow, and refuse to guess.** A bare filename with no directory, a glob, a
path with a placeholder in it, and anything inside a fenced block are all out of
scope. ⚠ The scope rule this check already states applies to itself: a guard that
refuses legitimate writing is worse than an honest one, and
`check-docs`'s own header says so.

⭐ **Mutation-prove it against the defect it was written for**: put
`` `scripts/common/does-not-exist.sh` `` into a document and confirm the check
refuses, then confirm a real path beside it still passes.

Must not: add it to one half. Both halves, per section 4 of
[`RULES.md`](RULES.md), and `check-twins` compares them.

### Prove

```bash
sh scripts/common/check-docs.sh
```

Passing means: a planted citation of a file that does not exist is reported with
its file and line, a citation of a file that does exist is not, both halves
agree on the same tree, and `check-twins` reports no drift on the pair.

---

## TOOL-11. The banned-vocabulary rule was documented and unenforced

**Source** found while mutation-proving the documentation check, 2026-08-31
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

⛔ **A guard that was believed to exist did not.**
`docs/conventions/prose.md` listed eighteen banned words and said a linter
checks them. `docs/agent-tooling.md` said `check-docs` does. `check-docs`'s own
header said so, in both halves, and its success line read "Links and prose
clean". It checked links, fenced blocks, placeholders and orphan pages, and no
vocabulary at all.

### Premise

Measured: a sentence carrying two banned words was appended to a document and
`check-docs` passed. That is lens 2 of
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md), a guard
planted with the defect it exists to catch, and it did not fire.

### Approach

Implemented in both halves, over **fourteen** of the eighteen words rather than
all of them, and the measurement is the reason.

⭐ **Four of the eighteen cannot be held by a check.** `simply`, `just`,
`obviously` and "of course" are banned as dismissals, telling a reader who is
stuck that what they cannot do is easy. They are also ordinary English in a
contrast. Measured over this tree before the check existed: **nineteen matches
on those four, every one of them legitimate, and no defects.** ⛔ A guard with a
nineteen-to-nothing false positive rate is a guard somebody switches off.

So the fourteen unambiguous quality-assertions are the check, the four
dismissals stay a reading, and
[`../docs/conventions/prose.md`](../docs/conventions/prose.md) says which half
owns which.

Must not: widen it to the four to make the rule look fully enforced. ⛔ The
honest scope is the one `check-docs`'s own header already argued for.

### Prove

```bash
sh scripts/common/check-docs.sh
```

Passing means: a planted banned word is reported with its file and line by both
halves identically, a specimen inside a fenced block or a code span is not, and
the tree is clean.

### Closing

**Closed 2026-08-31.** Both halves implemented, mutation-proved, and the one
real violation in the tree cleared.

```text
$ printf 'This addition is world-class and bulletproof.
' >> docs/glossary.md
$ sh scripts/common/check-docs.sh
documentation check failed, 2 problem(s):

  docs/glossary.md:80 banned vocabulary: world-class. docs/conventions/prose.md
  docs/glossary.md:80 banned vocabulary: bulletproof. docs/conventions/prose.md

$ pwsh -NoProfile -File scripts/common/check-docs.ps1
documentation check failed, 2 problem(s):

  docs/glossary.md:80 banned vocabulary: world-class. docs/conventions/prose.md
  docs/glossary.md:80 banned vocabulary: bulletproof. docs/conventions/prose.md
```

⚠ **One violation existed in the tree and it was in a conventions document**,
which is the file set most likely to be believed. It is fixed.

---

## TOOL-12. A mined tree brings its own ignore rules, and 92 files of the corpus were never committed

**Source** found by the continuous integration job on the first push, 2026-08-31
**Category** tooling, **Priority** P0, **Effort** S, **Status** done

### Problem

⛔ **`git` honours a `.gitignore` inside a mined tree**, and a reference clone
brings one. `Azathothas/bit-cli`'s carries `/bench/*.json`, so **92 files** of
that corpus sat on disk and in no commit.

⛔ **One of them is
[`bench/browser-fingerprint-cft-152.json`](../references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json),
the Chrome 152 capture**: one of the two primary artefacts every inherited value
in [`../docs/inherited-claims.md`](../docs/inherited-claims.md) is cited against,
and it is cited by name twice in
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md).
⭐ **A reader cloning this repository would have found the citation and not the
file**, which is precisely the failure
[`../docs/methodology/references.md`](../docs/methodology/references.md) section
4 exists to prevent, arriving through a door nobody had looked at.

### Premise

Measured. The whole gate passed on the machine that mined it and failed on the
first push, in both runners, on a broken link.

⚠ **Every local check resolves a path against the filesystem**, so an untracked
file is indistinguishable from a tracked one until somebody else clones. ⛔ That
is the green-locally red-in-CI shape, and this repository had no guard for it.

`mine-repo` **does** refuse to write into a directory this repository's own
rules would swallow. That guard runs before the fetch, so it cannot see a rule
that arrives inside the clone.

### Approach

Three changes, and the first alone would have left the cause in place.

1. **Take the files.** `git add -f references/`, which tracks them regardless of
   any ignore rule, now and afterwards.
2. ⭐ **`check-docs` reports a link target that is on disk and not committed**,
   in both halves. That turns this class from a CI failure into a local one.
3. **`mine-repo` counts what the clone's own rules would lose** and prints the
   command that takes it, in both halves and in the machine-readable output as
   `uncommittable`. ⚠ It **reports** rather than refuses: the tree is already
   fetched by then, and exiting non-zero over it would be the corpus-losing
   failure the script exists to stop.

Must not: delete the mined `.gitignore` files. They are upstream's content at
the captured commit, and a trim that rewrites a tree invalidates every citation
into it.

### Prove

```bash
sh scripts/common/check-docs.sh
```

Passing means: a link to a path that exists but is ignored is reported by both
halves with the same message, a link to a committed path is not, and
`git ls-files --others --ignored --exclude-standard references` prints nothing.

### Closing

**Closed 2026-08-31.** All three landed, and the guard was mutation-proved
against the defect it exists to catch.

```text
$ git ls-files --others --ignored --exclude-standard references | wc -l
0

$ printf '[x](../../.tmp/SEED.md)
' >> docs/reference-sweeps/findings.md
$ sh scripts/common/check-docs.sh
  docs/reference-sweeps/findings.md:832 link target is on disk and NOT COMMITTED -> ../../.tmp/SEED.md
$ pwsh -NoProfile -File scripts/common/check-docs.ps1
  docs/reference-sweeps/findings.md:832 link target is on disk and NOT COMMITTED -> ../../.tmp/SEED.md
```

⚠ **What this does not cover.** The guard reads **links**. A path written in a
code span is still unchecked, which is `TOOL-10`, and the same file would have
been missed if it had been cited that way. ⭐ Two doors into one defect, and only
one of them is closed.

---

## TOOL-13. The Windows job skipped a lint and the workflow counted it as allowed

**Source** found by the continuous integration job on the first push, 2026-08-31
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

`windows-latest` carries no `shellcheck`, so the gate reported it as skipped.
The workflow allows exactly one skip, the twin comparison, so the job failed
counting two. ⛔ **The tempting fix is to widen the assertion**, which would
turn every future skip on that job green.

### Premise

Measured, from the run: `SKIP shellcheck -- shellcheck is not on PATH`, beside
`GATE FAILED ... Also skipped 2`.

### Approach

Install it, the way the analyzer beside it is already installed, and let the
assertion keep meaning what it says. ⭐ **A shell script linted on one runner
proves nothing about the half that did not run it**, and this repository's whole
twin argument is that a host difference is a real difference.

Must not: relax the skip count. ⛔ An assertion widened to accommodate one
failure accommodates every later one.

### Prove

```bash
gh run list --limit 1
```

Passing means: both jobs are green, and the Windows job reports one skip rather
than two.

### Closing

**Closed 2026-08-31.** Installed in the workflow, and the assertion is
untouched.

```text
gate (windows)
  ✓ shellcheck
  ✓ the gate, this half
  ✓ the analyzer actually ran
gate (ubuntu)
  ✓ the gate
  ✓ yaml parses
```

⚠ **Both jobs pass on the pushed commit**, and the Windows half now reports one
skip: the twin comparison, which the ubuntu job runs.
