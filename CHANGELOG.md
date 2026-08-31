# CHANGELOG

What shipped, when, and where the evidence is. Newest first.

⛔ **Nothing in this project has been released.** The entries below are
repository changes, not published artefacts, and every one says so.
[`TODO/PROGRESS.md`](TODO/PROGRESS.md) is where the work stands.

---

## Unreleased

⛔ **Nothing here has been released.** The section exists because an entry is a
`### ` heading under a `## ` section, and a file with no section has no
entries a check can read. TOOL-14.

### 2026-08-31T22:40:00Z - the schema, the validator and the capture oracle

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and `SCHEMA-01` through
`SCHEMA-05`, `SCHEMA-07`, `SCHEMA-09`, `VALID-01`, `HARNESS-01`, `TOOL-03`,
`TOOL-05`, `TOOL-10`, `TOOL-14`.
**Deployed:** no. Nothing is published from this repository yet, and no capture
has been taken.

What landed:

- ⭐ **The profile schema, written first and published**, with the Rust types
  checked against it by a validator that refuses a schema keyword it does not
  implement.
- ⭐ **Eight coherence checks and a command**, with three outcomes rather than
  two: a check that cannot run reports that rather than passing.
- ⭐ **A capture oracle** that reads a `ClientHello` off a real loopback socket,
  parses permissively, keeps the bytes whatever happens, and compares a run
  against a committed golden.
- **The gate went from 15 checks to 19 and from 13 twin pairs to 15.**

⛔ **`check-changelog` had been asserting four rules over zero entries since the
first commit**, because it reads an entry as a `### ` heading and this file
wrote them at `## `. It is fixed, this file is reshaped, and zero entries is now
a failure rather than a pass.

⛔ **The credential rule had a third door.** Capture-time filtering was gated in
two places and tested in both; deserialisation was neither, so a profile read
from a file could carry a cookie header. Found by the door sweep and closed.

⚠ **Two guards were too wide as their entries specified them**, and running them
is what said so: one reported 30 findings that were all legitimate. Neither hex
rule was widened, which was the tempting fix in both cases.

---

### 2026-08-31T14:10:15Z - the workspace exists, and the gate runs a suite

**Record:** [`TODO/tooling.md`](TODO/tooling.md), `TOOL-01` and `TOOL-02`.
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- **A Rust workspace of eight crates**, one per name the entries already use,
  with the toolchain pinned to an exact version rather than a channel. All
  libraries: an entry whose acceptance says `cargo run` adds its binary target
  with the behaviour.
- ⭐ **`scripts/common/check-msrv`, in both halves.** The declared minimum
  supported version is now held by a check that derives the floor from the
  resolved dependency graph and can compile the workspace with the declared
  toolchain, rather than by a number in a manifest.
- **The suite in both halves of the gate**, as three separately scored entries
  rather than one, and the placeholder comment that stood in for it removed. The
  gate went from 15 checks to 19.

⚠ **The suite is eight empty crates and zero tests, so it passes vacuously
today**, and both halves of the runner say so in a comment. The three entries
were each mutation-proved by planting the defect they exist to catch.

⛔ **A defect in the new check was found by running it**, and it is the shape
this project cares most about: the `--verify` guard probed `cargo` where the
compile needs `rustc`, so a toolchain installed incompletely was reported as
"the workspace does not compile" rather than "could not run". A broken host was
accusing the tree. Both halves probe both binaries now.

⭐ **And the first attempt to mutation-prove the graph comparison did not
fire**, because Cargo promotes a path dependency of a member into a workspace
member, which is exactly the set the check excludes. The fixture had to be
declared `exclude`d before it was a dependency at all. A guard recorded as
working on the strength of that first run would have been theatre.

---

### 2026-08-31T00:30:00Z - the repository is initialised

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and
[`TODO/INDEX.md`](TODO/INDEX.md) for the 77 entries this created.
**Deployed:** no. Nothing is published from this repository yet, and
[`TODO/RULES.md`](TODO/RULES.md) records that as a standing fact rather than an
omission.

What landed:

- **The methodology, the conventions and the security rules.** Every document
  that named a file this project does not have was rewritten rather than
  inherited, and three roles were deliberately left unwritten rather than
  shipped as empty skeletons.
- **The gate**, in both its POSIX shell and PowerShell halves, plus the probe,
  the record checker and its writer, the file writer, the commit helper and the
  reference fetcher. ⚠ The gate contains no test suite, because there is no
  code, and both halves of the runner say so in a comment rather than reporting
  green over an absence.
- ⭐ **A sweep of eighteen repositories**, at named commits. The trees are in
  [`references/`](references/) and the write-up is in
  [`docs/reference-sweeps/`](docs/reference-sweeps/). ⭐ One of the eighteen,
  `Azathothas/bit-cli`, is the origin every inherited value was measured in,
  rather than prior art.
- **[`docs/inherited-claims.md`](docs/inherited-claims.md)**, which records every
  value this project carries that it did not measure, each cited at a file in
  the tree it was measured in.
- **[`docs/glossary.md`](docs/glossary.md)**, with the caveat attached to each
  term rather than to the page that uses it.
- **77 work entries** across eleven categories, four of which close.
- **The 0BSD licence**, a bare README, and the repository's own router at
  [`docs/AGENTS.md`](docs/AGENTS.md).

⛔ **Four inherited claims were refuted during the reading**, before any of them
had been acted on. [`docs/HISTORY/README.md`](docs/HISTORY/README.md) lists each
with the reading that took it away. One changes what this project claims about
itself, and [`README.md`](README.md) makes the narrower claim; one was refuted by
the capture the claim was quoting.

⚠ **Two defects in this repository's own tooling were found by its own checks**
and both are recorded rather than quietly repaired. Five places described a
licence filler that was not on disk, which `TOOL-09` closes by deleting the
description; and the reference fetcher stops before cloning when one of its two
routes is down, which is `TOOL-04` and is still open.

**The design brief this repository started from was never committed.** `DOC-04`
carries the table of where each part of it went, and
[`docs/inherited-claims.md`](docs/inherited-claims.md) is what carries its
measurements now.
