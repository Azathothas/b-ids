# Archived tooling record

This repository's own scripts, gates and toolchain. Everything here was found by
setting the repository up, so each entry names what it is missing rather than
what it would be nice to have.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## TOOL-01. There is no toolchain, and the minimum version is measured rather than chosen

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

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
[`../docs/methodology/vendoring.md`](../../methodology/vendoring.md).

Must not: pin a floating channel, or state a minimum version that no command
derived.

### Prove

```bash
cargo metadata --format-version 1 --no-deps
```

Passing means: the workspace resolves, the pinned toolchain file names an exact
version, and a second command derives the minimum supported version from the
dependency graph and writes it where the check can read it back.

### Closing

**Closed 2026-08-31.** The workspace resolves, the toolchain is pinned to an
exact version, and the minimum supported version is held by a check in both
halves rather than by a number somebody typed into a manifest.

```text
$ cargo metadata --format-version 1 --no-deps > .tmp/meta.json
exit=0
$ wc -c < .tmp/meta.json
7550
$ jq -r '.packages[] | "\(.name) \(.rust_version) \(.edition)"' .tmp/meta.json
b-ids 1.88.0 2024
b-ids-cli 1.88.0 2024
b-ids-conformance 1.88.0 2024
b-ids-driver 1.88.0 2024
b-ids-emit 1.88.0 2024
b-ids-harness 1.88.0 2024
b-ids-schema 1.88.0 2024
b-ids-validator 1.88.0 2024
$ grep channel rust-toolchain.toml
channel = "1.98.0"
```

⚠ **The metadata is 7,550 bytes of machine JSON and is not reproduced here.**
The exit code, the byte count and the eight packages it resolved are what the
command returned; the derived lines above name the command that derived them.

### ⭐ Two version numbers, and they are different facts

⛔ **Conflating them is the mistake this entry exists to prevent**, so they are
written down side by side with how each was arrived at.

| | value | how it was arrived at |
| --- | --- | --- |
| the pinned toolchain, `rust-toolchain.toml` | 1.98.0 | the compiler this tree is built and tested with. Installed on this host and named exactly, never as a channel. |
| the declared minimum, `rust-version` | 1.88.0 | the oldest toolchain present on this host, against which `cargo check --workspace --all-targets` was run and passed |

⚠ **The premise's figure was already stale when this entry was worked.** It
records `cargo 1.94.1` measured 2026-08-30; the probe on the same host reports
`cargo 1.98.0` and `rustc 1.98.0` on 2026-08-31. Nothing was decided from the
stale number, and it is left in the premise rather than edited, per
[`../docs/methodology/authoring.md`](../../methodology/authoring.md).

### ⛔ What the minimum is NOT

**1.88.0 is a verified upper bound on the minimum, not the minimum.** The
workspace compiles there. Nothing here has shown it fails on anything older,
and saying so would be the fabricated number the check exists to refuse.

⚠ **The check's graph leg cannot narrow it either, and today it cannot fire at
all**: the workspace has no dependencies, so the resolved graph imposes no
floor. `--write` refuses in that state rather than inventing one.

**Three routes to the true minimum were considered.**

| route | why it is not taken yet |
| --- | --- |
| install a ladder of old toolchains and bisect | several hundred megabytes per rung, and the answer moves the first time a line of code lands. Worth doing once there is code whose features decide it. |
| `cargo-msrv` | not installed; it is a `cargo install` that compiles a tool to answer a question a bisect answers with the toolchains already here |
| ⭐ verify against the oldest toolchain on this host and declare that | taken. It costs nothing, it is a command anybody can re-run, and it is honest about being a bound rather than the floor. |

### The check, mutation-proved in both directions

⛔ **Leg 2, the graph comparison, has been seen to refuse.** A fixture package
declaring `rust-version = "1.99.0"` was added as an excluded path dependency:

```text
$ sh scripts/common/check-msrv.sh
msrv check failed, 1 problem(s):

  Cargo.toml: rust-version is 1.88.0, and the dependency graph needs 1.99.0 (msrv-fixture). Derive it: sh scripts/common/check-msrv.sh --write

$ sh scripts/common/check-msrv.sh --json
{"schema":"check-msrv/1","declared":"1.88.0","graph_floor":"1.99.0","packages":9,"dependencies":1,"verified":0,"problems":1}

$ sh scripts/common/check-msrv.sh --write
replace Cargo.toml: 1234 -> 1234 bytes
check-msrv: rust-version 1.88.0 -> 1.99.0, derived from msrv-fixture
  Now read it back: sh scripts/common/check-msrv.sh

$ sh scripts/common/check-msrv.sh
msrv ok: declared 1.99.0, graph floor 1.99.0 from msrv-fixture
```

⭐ **The first attempt at that fixture did not fire, and the reason is worth
keeping.** It was added as a plain path dependency, and the check passed with
`dependencies: 0`. Cargo automatically makes a path dependency of a member into
a workspace MEMBER, and members are exactly what the floor excludes. The
fixture only became a dependency once the workspace declared
`exclude = [".tmp/msrv-fixture"]`. ⚠ A guard that had been "proved" by the first
attempt would have been recorded as working on the strength of a run in which
its subject was never present.

⛔ **Leg 1, the absent-field branch, could not be made to fire in this tree, and
that is reported rather than claimed.** Deleting `rust-version` from
`[workspace.package]` makes `cargo metadata` itself fail, because every member
inherits the field with `rust-version.workspace = true`:

```text
$ sh scripts/common/check-msrv.sh
check-msrv: cargo metadata failed. Run it directly to see why:
  cargo metadata --format-version 1
rc=2
```

What would have to be true for it to fire: a workspace whose members declare
their own `rust-version` rather than inheriting one, so cargo resolves and the
field is still absent from the workspace table. The branch stays as a
validation branch, which
[`../docs/conventions/code.md`](../../conventions/code.md) permits by name.

### ⚠ A defect in the check, found by running it

**The `--verify` guard probed the wrong binary.** It ran `cargo --version`
through the candidate toolchain to decide "installed or not". An install
interrupted part-way leaves a working `cargo` beside a `rustc` with no
manifest, so the guard passed and `cargo check` then failed on `rustc -vV`,
which the script reported as **the workspace does not compile**.

⛔ **That is a broken host accusing the tree**, and it is the exact confusion
between exit 1 and exit 2 that the three-code contract exists to prevent. Both
binaries are probed now, in both halves:

```text
$ sh scripts/common/check-msrv.sh --verify
check-msrv: toolchain 1.88.0 is not installed, or is installed incompletely.
  That is "could not run", not "failed". Install it and re-run:
    rustup toolchain install 1.88.0
rc=2

$ sh scripts/common/check-msrv.sh --verify
msrv ok: declared 1.88.0, graph floor none (0 dependency package(s) resolved), compiles on 1.88.0
rc=0
```

### What else landed with it

- **Eight crates**, one per name the entries already use, all libraries.
  ⛔ No binary targets: an entry whose acceptance says `cargo run -p NAME` adds
  the target with the behaviour, because a `main` that does nothing is the dead
  code [`../docs/conventions/forbidden-patterns.md`](../../conventions/forbidden-patterns.md)
  forbids.
- **Lints at the workspace, inherited by every member**: `unsafe_code` denied
  rather than forbidden, so the escape hatch
  [`../docs/conventions/code.md`](../../conventions/code.md) allows can still
  be written with a comment saying why.
- ⛔ **`Cargo.lock` is committed**, and [`../.gitignore`](../../../.gitignore) carries
  the ruling: a measurement taken with an unrecorded dependency set cannot be
  retaken.
- **`*.rs` and `*.lock` given explicit line-ending rules** in
  [`../.gitattributes`](../../../.gitattributes), rather than left to the `text=auto`
  fallback.
- **Continuous integration pins nothing of its own.** Both jobs run
  `rustup show active-toolchain`, which reads `rust-toolchain.toml`. ⛔ A
  version written into the workflow would be a second copy of the pin.

⚠ **This session installed the `1.88.0` and `1.98.0` toolchains on this host**
with `rustup`. They are outside the tree and they stay: they are what the two
version claims above were measured with.

---

## TOOL-02. The gate has no suite in it, and says so in a comment

**Source** found while adopting the gate, 2026-08-30
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

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

### Closing

**Closed 2026-08-31**, in the same change as `TOOL-01`, because the suite has
nothing to run until the workspace exists.

```text
$ sh scripts/common/check-gate.sh --fast --json
{"schema":"check-gate/1","total":19,"passed":18,"failed":0,"skipped":1,"strict":0}
exit=0
```

The total was 15. It is 19: three suite entries from this entry and
`check-msrv` from `TOOL-01`.

### ⚠ The scope grew from one entry to three, and here is the argument

The approach above says "add the suite". What landed is **`cargo fmt`,
`cargo clippy` and `cargo test`, scored separately**, because part (a) of
[`../docs/methodology/gate.md`](../../methodology/gate.md) is "typecheck,
lint, format, the full test suite" rather than the suite alone.

⛔ **Separately rather than behind one verdict**, and the runner already makes
this argument about the PowerShell parse and the analyzer: they can have
different answers, and one verdict over three is how a skipped one reads as a
passed one. The mutation proof below is what settles it, because it produced
two different failing subsets.

### Both halves, and they agree

```text
$ sh scripts/common/check-gate.sh --fast
  ok    cargo fmt
  ok    cargo clippy
  ok    cargo test
gate ok: 18 passed, but 1 SKIPPED on this host: check-twins

$ pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
  ok    cargo fmt
  ok    cargo clippy
  ok    cargo test
gate ok: 18 passed, but 1 SKIPPED on this host: check-twins
```

⚠ Both blocks are the suite lines and the summary line from runs whose full
output is the nineteen-line table; nothing was removed from either that changes
what they report.

### ⭐ Mutation-proved, and each entry has been seen to refuse

⛔ **A suite line nobody has watched fail is a suite line nobody knows runs.**
Two separate defects were planted in `crates/b-ids-schema/src/lib.rs`.

```text
$ printf '\n#[test]\nfn planted_failure() {\n    assert_eq!(1, 2, "planted");\n}\n' >> crates/b-ids-schema/src/lib.rs
$ sh scripts/common/check-gate.sh --fast
  ok    cargo fmt
  ok    cargo clippy
  FAIL  cargo test (exit 101)
GATE FAILED: 1 of 19. Failed: cargo test

$ printf '\npub fn planted(  ) -> i32 { let x = 1 ; return x ; }\n' >> crates/b-ids-schema/src/lib.rs
$ sh scripts/common/check-gate.sh --fast
  FAIL  cargo fmt (exit 1)
  FAIL  cargo clippy (exit 101)
  ok    cargo test
GATE FAILED: 2 of 19. Failed: cargo fmt cargo clippy
```

⭐ **The two runs failed on different subsets**, which is the evidence for
scoring them separately: a single verdict would have reported one failure twice
and hidden that the suite passed over unformatted, lint-failing code.

### ⚠ What this does NOT prove, and it is the honest half

**The suite is eight empty crates and zero tests, so `cargo test` passes
vacuously.** The line is in the gate anyway, because the defect it removes is a
gate that grows its suite line months after the first crate lands, and both
halves of the runner now carry a comment saying exactly this. The first real
test arrives with `SCHEMA-01`.

⚠ **The three entries add real time to every gate run.** `TOOL-07`'s figures
predate them and are labelled with their own conditions; they were not edited
to cover a tree they were not measured on.

---

## TOOL-03. The secret check will refuse the raw captures

**Source** found while adopting the checks, 2026-08-30
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

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
[`../docs/inherited-claims.md`](../../inherited-claims.md) are bare
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

### Closing

**Closed 2026-08-31**, on the day it fired, which is the day this entry
predicted.

⭐ **It was not fixed pre-emptively: it was fixed because the gate went red.**
`HARNESS-01` committed `crates/b-ids-harness/fixtures/client-hello.hex`, 634
hexadecimal characters on one line, and the secret sweep refused it exactly as
this entry said it would. The comment sitting in both halves of the check since
the repository was set up named the shape, named the tempting wrong fix, and
named this entry.

```text
$ sh scripts/common/check-no-secrets.sh --public
== a long hex identifier ==
Cargo.lock:56:checksum = "8f42a60cbdf9a97f5d23...685682"
crates/b-ids-harness/fixtures/client-hello.hex:1:1603010138010001340303...
crates/b-ids-harness/fixtures/client-hello.capture.json:7:  "raw_hex": "160301...
crates/b-ids-harness/fixtures/client-hello.capture.json:12:    "session_id_hex": "1111...

⛔ 1 category/categories matched.
rc=1
```

⚠ **The two hexadecimal values above are abbreviated at the ellipses, and that
is this check working rather than a liberty taken with a paste.** Written out in
full they are a credential-shaped string in a tracked file, which is what this
check refuses; the gate went red on this entry own evidence block before the
ellipses were put in. ⛔ Nothing else in either block is edited, and the four
lines shown are four of the twenty-one the run printed, each hundreds of
characters long.

### ⛔ The hex rule was not widened. Three exclusions, each by name or file type

| exclusion | why it is not a credential |
| --- | --- |
| a hex run assigned to an identifier ending in `_hex` | this project's own naming rule for a field holding wire bytes: `raw_hex`, `body_hex`, `session_id_hex`, `client_hello_hex`, `payload_hex`. A credential is not assigned to something called `body_hex`. |
| a `.hex` file | this project defines the type as one raw capture on one line and nothing else |
| `checksum = "..."` in a lock file | a declared digest of a published artefact, which is the same shape as the action pin the check already excludes |

⭐ **The first is the same mechanism the existing exclusions use**: the value
has to sit beside an identifier that says what it is. That is what makes the
exclusion narrow enough to keep the rule.

### ⛔ Mutation-proved: the exclusion does not swallow its neighbourhood

An exclusion that covered a whole file would be worse than none, so the proof
is a credential planted INSIDE a raw capture, under a different field name:

```text
$ sh scripts/common/check-no-secrets.sh --public
== a long hex identifier ==
crates/b-ids-harness/fixtures/client-hello.capture.json:156:  "api_token": "deadbeefcafe...89abcdef"

⛔ 1 category/categories matched.
rc=1
```

And both halves agree, on the clean tree and on the planted one:

```text
$ sh scripts/common/check-no-secrets.sh --public --json
{"schema":"check-no-secrets/1","findings":0,"public_rules":true,"history_scanned":false}
$ pwsh -NoProfile -File scripts/common/check-no-secrets.ps1 -Public -Json
{"schema":"check-no-secrets/1","findings":0,"public_rules":true,"history_scanned":false}

  with the credential planted:
{"schema":"check-no-secrets/1","findings":1,"public_rules":true,"history_scanned":false}
{"schema":"check-no-secrets/1","findings":1,"public_rules":true,"history_scanned":false}
```

### ⚠ What arrived that this entry did not predict

**`Cargo.lock`.** `TOOL-01` committed it, and eleven registry checksums are
64 hexadecimal characters each. It is the same class as the action pin and it
was not in the entry, because the entry was written before the workspace
existed. ⭐ It is excluded by the identifier `checksum`, narrowly, rather than
by exempting the file.

---

## TOOL-04. The reference fetcher stops when one of its two routes is down

**Source** found while running the reference sweep, 2026-08-30
**Category** tooling, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-01T13:55:00Z.** The API route failing is a gap now, recorded in
the provenance file, and the run continues to the clone. ⛔ Only a run in which
NEITHER route produced anything exits non-zero.

⚠ **Two substitutions in the blocks below, and both are marked.** The
scratch directory the trees were written to reads `SCRATCH`, and the commit
`7fd1a60b01f91b314f59955a4e4d4e80d8edf11d` is abbreviated: this repository
writes a commit id in a code span, and `check-no-secrets` refuses a bare
forty-character hex run inside a fenced block. ⛔ The rule was not widened for
a documentation convenience.

⭐ **The outage was arranged rather than waited for**, by pointing the proxy at
an unresolvable host in the working tree and restoring it afterwards. A defect
that only reproduces on a bad network day is a defect nobody can close.

```text
$ sh scripts/common/mine-repo.sh octocat/Hello-World --route proxy --out SCRATCH
route: proxy
control: ⛔ UNREACHABLE (pkgforge-dev/reverse-proxies answered 000). A 404 below means nothing.
fetching octocat/Hello-World
  metadata: FAILED. Continuing to the clone; the tree is the half that can still be got.
  discussions: skipped (the API route is down)
  tree: 7fd1a60…

mined octocat/Hello-World into SCRATCH/octocat__Hello-World
commit 7fd1a60…, route proxy, 1 gap(s).
⭐ Keep the tree. A conclusion nobody can re-check is an opinion.
⚠ The API route was down, so this fetch has the TREE and no metadata.
Re-run it when the route is back; the gap is named in PROVENANCE.md.
exit=0
```

⭐ **And the gap is written where the procedure looks for it**, rather than only
printed:

```text
## ⛔ What this fetch did NOT get

  - metadata: repos/octocat/Hello-World could not be fetched over the proxy route, so NO API-derived source was fetched: no metadata, no issues, no comments, no releases, no tags, no discussions. Control says: ⛔ UNREACHABLE (pkgforge-dev/reverse-proxies answered 000). A 404 below means nothing.
```

### Both directions, and both halves

```text
$ sh scripts/common/mine-repo.sh octocat/Hello-World --route proxy --out SCRATCH --json
{"schema":"mine-repo/2","target":"octocat/Hello-World","route":"proxy","metadata":false,"commit":"7fd1a60…","gaps":1,"uncommittable":0,"dest":"SCRATCH/octocat__Hello-World"}
exit=0

$ sh scripts/common/mine-repo.sh octocat/this-repository-does-not-exist-b-ids --route proxy --out SCRATCH --json
{"schema":"mine-repo/2","target":"octocat/this-repository-does-not-exist-b-ids","route":"proxy","metadata":false,"commit":"-","gaps":2,"uncommittable":0,"dest":"SCRATCH/octocat__this-repository-does-not-exist-b-ids"}
exit=1

$ pwsh -NoProfile -File scripts/common/mine-repo.ps1 octocat/Hello-World -Route proxy -Out SCRATCH -Json
{"schema":"mine-repo/2","target":"octocat/Hello-World","route":"proxy","metadata":false,"commit":"7fd1a60…","gaps":1,"uncommittable":0,"dest":"SCRATCH/octocat__Hello-World"}
exit=0

$ pwsh -NoProfile -File scripts/common/mine-repo.ps1 octocat/this-repository-does-not-exist-b-ids -Route proxy -Out SCRATCH -Json
{"schema":"mine-repo/2","target":"octocat/this-repository-does-not-exist-b-ids","route":"proxy","metadata":false,"commit":"-","gaps":2,"uncommittable":0,"dest":"SCRATCH/octocat__this-repository-does-not-exist-b-ids"}
exit=1
```

⚠ **The two answered identically on every run**, including the digit the sh half
reports as `000` and the PowerShell half as `0` for an unreachable host, which is
`curl`'s own output in one and a numeric status in the other and does not reach
the machine-readable line.

### ⚠ What the JSON schema move costs, said rather than left to be noticed

The reporting shape gained `metadata`, so it is `mine-repo/2`. ⛔ Adding a field
without moving the version is the "positional or implicit format with no
version" row in
[`../docs/conventions/forbidden-patterns.md`](../../conventions/forbidden-patterns.md),
and this project's own gate reads these lines.

⚠ **The pair's `check-twins` row is unaffected**, and that is not luck: it
compares `--selftest`, which has its own schema and runs before any fetch. ⛔ The
fetching path still has no comparison, because comparing it needs the network,
which is the standing exemption `TOOL-05` recorded and this entry does not
change.

```text
$ sh scripts/common/mine-repo.sh --selftest --json
{"schema":"mine-repo-selftest/1","cases":4,"failed":0}
sh exit=0
$ pwsh -NoProfile -File scripts/common/mine-repo.ps1 -Selftest -Json
{"schema":"mine-repo-selftest/1","cases":4,"failed":0}
ps exit=0
```

⚠ **The fetched trees were written outside the repository and removed.** The
tree's own guard refuses an output directory git would ignore, which is what
sent them there: `.tmp/` is ignored, and a corpus nobody can commit is a corpus
lost on the next machine.

---

## TOOL-05. One twin pair has no comparison at all

**Source** found while adopting the checks, 2026-08-30
**Category** tooling, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-08-31.** The fetcher is compared through its offline self-test,
and it was the last twin in this tree with no comparison at all.

```text
$ sh scripts/common/check-twins.sh
  ok     mine-repo selftest: both say {"schema":"mine-repo-selftest/1","cases":4,"failed":0}, exit 0

✅ every twin pair agrees on this tree.
exit=0
```

⚠ The line above is one row of the fifteen the run prints, and the verdict line
is its last.

### Why the self-test rather than the fetch

⛔ **A row that needs the network must never go in a check that runs in the
gate.** The fetch needs it; the self-test does not. What the self-test drives is
the page joiner and the joiner's own guard against a built-in fixture, which is
the half of that script that has actually been wrong: its own header records
that the guard found a defect on its first run.

```text
$ sh scripts/common/mine-repo.sh --selftest --json
{"schema":"mine-repo-selftest/1","cases":4,"failed":0}
$ pwsh -NoProfile -File scripts/common/mine-repo.ps1 -Selftest -Json
{"schema":"mine-repo-selftest/1","cases":4,"failed":0}
```

### ⚠ What this comparison covers, and what it does not

**It compares the joiner on both hosts. It does not compare the fetch.** The
route handling that `TOOL-04` is about is still uncompared, because exercising
it needs a network and a route that is down. ⛔ That is a real gap and it is
named here rather than left to be assumed covered: this row makes the pair
compared, not the script covered.

---

## TOOL-06. The route check does not exist and it is three lines

**Source** [`../docs/reference-sweeps/usable.md`](../../reference-sweeps/usable.md) section 9
**Category** tooling, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-01.** ⭐ **The entry said it could not run until `PUB-03`
generates a tree, and that stopped being true when `CORPUS-01` landed.** The
corpus publishes a single-value route file already: `raw/v1/.../*.hello.hex`,
one hex line and nothing else.

```text
$ sh scripts/common/check-routes.sh
routes ok: 1 single-value file(s), none ends with a line ending
rc=0
```

#### ⛔ Mutation-proved in all four directions, both halves, exit codes unpiped

| the case | sh | pwsh |
| --- | --- | --- |
| a correct single-value file | 0 | 0 |
| the same file with a trailing newline | ⭐ **1**, naming the file | ⭐ **1**, naming the file |
| a directory holding no single-value file | 2 | 2 |
| the real published tree | 0 | 0 |

```text
$ sh scripts/common/check-routes.sh --fixtures FIX/bad
route check failed, 1 file(s):

  .../routefix/bad/trailing.hex: ends with a line ending, and it carries exactly one value

A consumer of a single-value route should never have to strip anything.
Fix the generator that wrote it, not the file.
```

#### ⛔ The fixture found a defect in the check it was written to prove

⭐ **This is the whole argument for writing the refusing fixture rather than
trusting the passing one.** The first version enumerated with `git ls-files`,
which answers a path outside the repository with a fatal on stderr and an EMPTY
LIST on stdout. Both halves then reported:

```text
routes ok: 0 single-value file(s), none ends with a line ending
rc=0
```

⛔ **Green, over nothing, on the file written to make it go red.** That is the
"step that exits 0 having done nothing it was asked to do" row in
[`../docs/conventions/forbidden-patterns.md`](../../conventions/forbidden-patterns.md),
in a check whose entire job is refusing.

Two changes, and the second matters more than the first:

- `--fixtures` walks the filesystem rather than git, because a fixture
  directory is deliberately not tracked;
- ⭐ **a route tree that yields no single-value file at all is exit 2**, not
  exit 0. A check that reports clean over nothing is a check that quietly stops
  applying the day a route type is renamed, and nothing would say so.

#### ⚠ One rule, one enforcer, and a duplicate was removed to keep it that way

`Store::verify` had grown the same assertion while `CORPUS-01` was being built.
Two checks holding one rule is two places for it to be wrong, so the newline
rule now lives here alone, over every published route file rather than only over
a sidecar that happens to have a profile beside it.

⛔ **What `Store::verify` keeps is the question that needs the profile**:
whether the sidecar holds what `raw.client_hello_hex` says it holds. ⚠ And the
corpus writer's own test still asserts it writes no newline, because the
generator's contract and the check's rule are different things: the check exists
precisely for the day a generator breaks its contract.

#### What is not covered

| | |
| --- | --- |
| a route type that is not `.hex` | The extension list is one entry long because one route type exists. `PUB-03` extends it in the same change that generates the tree, in both halves, and the lists are beside each other for that reason. |
| whether the VALUE in a route file is right | This checks one byte at the end. What the file should contain is the generator's, and `check-corpus` is what compares a sidecar against the profile it came from. |

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
[`../docs/conventions/prose.md`](../../conventions/prose.md) now states the
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

[`../docs/methodology/gate.md`](../../methodology/gate.md) describes a strict
mode that turns a skipped check into a failure, and says it is what a
continuous integration job should pass. Neither half of the gate runner had it.

⛔ **Both failure directions are bad and neither is visible.** A job passing the
flag would have been refused with an unknown-argument error, which reads as a
broken workflow rather than as a missing feature. A job that stopped passing it
would have gone green over any number of skipped checks, which is the whole
thing the flag exists to prevent.

That is the "a setting or flag that no code reads" row in
[`../docs/conventions/forbidden-patterns.md`](../../conventions/forbidden-patterns.md),
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

⛔ **Seven files named scripts/common/fill-license.sh, its PowerShell twin,
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
- the rows and sections in [`../scripts/README.md`](../../../scripts/README.md) and
  [`../AGENTS.md`](../../../AGENTS.md);
- the standing rule in [`RULES.md`](RULES.md) whose only worked example it was.

⚠ **The incident it came from stays.**
[`../docs/history/README.md`](../README.md) records a licence
written by hand whose warranty clause was corrupted and which exited 0. That
happened, and it is why [`../LICENSE`](../../../LICENSE) is not edited by hand now.
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
[`../docs/conventions/forbidden-patterns.md`](../../conventions/forbidden-patterns.md)
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
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

⛔ **A markdown link is checked and a path in a code span is not.**
`check-docs` resolves every `](target)` and reports a broken one. It does
nothing about a path like scripts/common/some-tool.sh written as prose, which is how
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
the path scripts/common/does-not-exist.sh into a document and confirm the check
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

### Closing

**Closed 2026-08-31.** Both halves resolve a cited path, and 33 of them in this
tree are checked that nothing checked before.

```text
$ sh scripts/common/check-docs.sh
docs ok: 48 files, 684 relative links, 33 cited paths, 130 shell blocks. Links, paths and prose clean.
exit=0
```

### Mutation-proved, both halves, on one tree

```text
$ printf '\nA planted citation of \`scripts/common/does-not-exist.sh\` beside the real\n\`scripts/common/check-docs.sh\`.\n' >> docs/glossary.md
$ sh scripts/common/check-docs.sh
documentation check failed, 1 problem(s):

  docs/glossary.md:80 cited path does not exist -> scripts/common/does-not-exist.sh
sh_rc=1
$ pwsh -NoProfile -File scripts/common/check-docs.ps1
documentation check failed, 1 problem(s):

  docs/glossary.md:80 cited path does not exist -> scripts/common/does-not-exist.sh
ps_rc=1
```

⭐ **The real path was planted on the line beside the false one and is not
reported.** A guard tested only with the defect proves it refuses something; the
pair proves it refuses the right thing.

And the two halves agree on the same tree:

```text
$ sh scripts/common/check-docs.sh --json
{"schema":"check-docs/1","problems":0,"files":48,"links":683,"cited_paths":33,"shell_blocks":130}
$ pwsh -NoProfile -File scripts/common/check-docs.ps1 -Json
{"schema":"check-docs/1","problems":0,"files":48,"links":683,"cited_paths":33,"shell_blocks":130}
```

### ⛔ The scope in the approach was too wide, and running it is what said so

The approach says to resolve a code span that "contains a `/`, ends in a known
extension, holds no whitespace and no angle bracket, and does not begin with a
scheme". **Implemented exactly as written, the check reported 30 spans in this
tree and every one of them was legitimate.**

| what the 30 were | why each is not a defect |
| --- | --- |
| 24 paths INSIDE a reference tree, cited as shorthand | the sweep documents write `bench/browser-fingerprint-cft-152.json` for a file under `references/`, and the whole document says which tree it means |
| 5 paths this tree deliberately does not have | a retired tool named in the entry recording its removal, and this entry's own two examples of the defect |
| 1 genuine ambiguity | below |

⚠ **A guard with a thirty-to-nothing false positive rate is a guard somebody
switches off**, which is the same measurement that scoped the banned-vocabulary
rule to fourteen of its eighteen words.

⭐ **So one more rule was added and it is read from git rather than written
down**: a span is a path only when its first segment is one of the top-level
directories this repository owns. That took 30 findings to 6, and the six were
then read one at a time.

### ⭐ The one genuine finding, which is what the check is for

```text
docs/reference-sweeps/usable.md:463 cited path does not exist -> crates/bit-cli-core/src/browser.rs
```

⚠ **That citation was unambiguous when it was written and stopped being so
today**, because `TOOL-01` created a `crates/` directory in THIS tree. A reader
now resolves it against the wrong repository and finds nothing. It is written
out in full now, from the repository root, so it resolves and stays resolvable.

⛔ **This is exactly the defect class the entry was filed for**, arriving through
a door nobody had looked at: not a rename, but a name this tree acquired.

### The convention the other five produced

⛔ **A path in a code span asserts that it resolves.** A path this tree
deliberately does NOT have is written as plain text instead, so a document can
still name a retired tool, an unwritten page or an example of the defect without
either lying or being refused.

That rule is in
[`../docs/conventions/prose.md`](../../conventions/prose.md), beside the
mechanical-half list it belongs to, and
[`../scripts/README.md`](../../../scripts/README.md) carries the measurement behind
the scope. ⚠ Five places were rewritten to follow it, including two in this
entry's own text.

### ⚠ What it still does not check

A bare filename with no directory, a glob, a path carrying a placeholder, and
anything inside a fenced block are all out of scope and stay so. ⛔ And a path
under `references/` written WITHOUT the prefix is now invisible again, which is
the trade the scope rule bought: 24 legitimate spans pass, and a stale one among
them would not be reported. Naming the tree in full is what makes such a
citation checkable, and `docs/methodology/references.md` already asks for that.

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
[`../docs/methodology/reviews.md`](../../methodology/reviews.md), a guard
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
[`../docs/conventions/prose.md`](../../conventions/prose.md) says which half
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
[`bench/browser-fingerprint-cft-152.json`](../../../references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json),
the Chrome 152 capture**: one of the two primary artefacts every inherited value
in [`../docs/inherited-claims.md`](../../inherited-claims.md) is cited against,
and it is cited by name twice in
[`../docs/reference-sweeps/findings.md`](../../reference-sweeps/findings.md).
⭐ **A reader cloning this repository would have found the citation and not the
file**, which is precisely the failure
[`../docs/methodology/references.md`](../../methodology/references.md) section
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

$ printf '[x](../../../../.tmp/SEED.md)
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

---

## TOOL-14. The changelog check read a heading level this repository does not use

**Source** found while running the gate after editing `CHANGELOG.md`, 2026-08-31
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

⛔ **`check-changelog` reported `changelog ok: 0 entries` over a file with two,
and that line asserts all four rules.** It reads an entry as a `### ` heading
under a `## ` section. This repository's [`../CHANGELOG.md`](../../../CHANGELOG.md)
wrote its entries at `## `, which the check reads as a section heading and
skips. So it found nothing, validated nothing, and printed a sentence saying
every entry is dated and names a record and says whether it deployed.

⭐ **It was green in the gate from the first commit**, in both halves, on both
runners. That is the "step that exits 0 having done nothing it was asked to do"
row in
[`../docs/conventions/forbidden-patterns.md`](../../conventions/forbidden-patterns.md),
in a check whose own header argues at length that an absent changelog must be
exit 2 rather than a pass. ⚠ The same reasoning was never applied one level
down, to a file that exists and presents nothing the check can read.

### Premise

⛔ **Measured, not read.** An entry breaking three of the four rules at once was
appended to `CHANGELOG.md` and the check passed:

```text
$ printf '\n## a heading with no date\n\nnothing here names a record and nothing says whether it deployed.\n' >> CHANGELOG.md
$ sh scripts/common/check-changelog.sh
changelog ok: 0 entries, in order, each dated with a record and a deploy line
rc=0
```

That is lens 2 of
[`../docs/methodology/reviews.md`](../../methodology/reviews.md): the guard was
planted with the defect it exists to catch and did not fire.

### Approach

Two changes, and the first alone would leave the class in place.

1. ⭐ **Zero entries is a failure, in both halves.** A check that finds no
   subject reports that rather than reporting a pass over it. This is the
   durable half: it makes the class unrepresentable for any later heading-level
   drift, in either direction.
2. **The document takes the shape the check reads**: a `## Unreleased` section
   with `### ` entries under it.

⚠ **Exit 1 rather than 2**, and the difference is what makes it visible. The
gate maps this check's 2 to a pass, because a project with no `CHANGELOG.md` has
satisfied these rules vacuously. A file that exists and presents nothing has
not.

Must not: fix the document alone. ⛔ The check would then be green again for the
same reason it was green before, and the next session that writes a heading at a
different level gets the same silence.

### Prove

```bash
sh scripts/common/check-changelog.sh
```

Passing means: with entries at the wrong level both halves exit 1 naming what
they looked for; with the document fixed both exit 0 reporting a non-zero entry
count; and each of the four rules, none of which had ever been exercised in this
tree, is seen to fire.

### Closing

**Closed 2026-08-31.** Both halves refuse an empty scope, the document is
reshaped, and the four rules are exercised for the first time.

```text
$ sh scripts/common/check-changelog.sh
changelog check failed, 1 problem(s):

  CHANGELOG.md: no entry heading found. An entry is a '### ' heading under a '## '
  section, and rules 1 to 4 were about to be reported clean over nothing.
rc=1

$ pwsh -NoProfile -File scripts/common/check-changelog.ps1
changelog check failed, 1 problem(s):

  CHANGELOG.md: no entry heading found. An entry is a '### ' heading under a '## '
  section, and rules 1 to 4 were about to be reported clean over nothing.
rc=1
```

With the document reshaped, both halves agree:

```text
$ sh scripts/common/check-changelog.sh --json
{"schema":"check-changelog/1","problems":0,"entries":2}
$ pwsh -NoProfile -File scripts/common/check-changelog.ps1 -Json
{"schema":"check-changelog/1","problems":0,"entries":2}
```

### ⭐ The four rules, exercised for the first time

⛔ **None of them had ever run in this tree.** Both plants below were made
against the reshaped document.

```text
$ printf '\n### an entry with no date\n\nno record line and nothing about deploying.\n' >> CHANGELOG.md
$ sh scripts/common/check-changelog.sh
changelog check failed, 2 problem(s):

  CHANGELOG.md:104 no date in the heading. Nothing can order it.
  CHANGELOG.md: the entry at line 104 names no record. An entry with no record is a claim.
rc=1

$ printf '\n### 2099-01-01 - out of order, and it has a Record: and Deployed: no\n' >> CHANGELOG.md
$ sh scripts/common/check-changelog.sh
changelog check failed, 3 problem(s):

  CHANGELOG.md:104 out of order: 2099-01-01 comes after 2026-08-31T00:30:00Z. Newest first.
  CHANGELOG.md: the entry at line 104 names no record. An entry with no record is a claim.
  CHANGELOG.md: the entry at line 104 does not say whether it deployed. Silence is not an answer.
rc=1
```

### ⚠ A known limit, found in the same plants and left in

**Rule 4 matches the substring `deploy` anywhere in the entry body.** In the
first plant above, "does not say whether it deployed" did not fire, because the
body text read "nothing about deploying" and that contains the word. The rule
fired in the second plant, where the body has no such word.

⛔ **Left as it is, and recorded rather than tightened.** Matching `deployed:`
instead would refuse "not deployed" and "this deployed nothing", which are
legitimate answers, and what counts as a valid answer is
[`../docs/conventions/docs.md`](../../conventions/docs.md)'s question rather
than this check's. ⚠ The looseness costs a false PASS on an entry whose prose
happens to mention deployment; the tightening would cost false FAILURES on
correct entries. The first is the cheaper error for a rule whose other half is
read by a person.

### ⚠ Authoring and implementing happened in one session, deliberately

[`../docs/methodology/authoring.md`](../../methodology/authoring.md) opens
with the rule that they are different sessions, and the reason it gives is that
a premise authored and implemented together is a premise never checked against
the code. ⭐ **That reason is already satisfied here**: the premise is a
measurement taken before a line was changed, and it is pasted above.

⛔ **And it is what this repository already does with a defect its own gate
found.** `TOOL-11`, `TOOL-12` and `TOOL-13` were each found by running the tree,
authored, fixed and closed on 2026-08-31. The alternative was leaving a check
that reports green over nothing sitting in the gate for a session, which is the
more expensive of the two. ⚠ The tension is real and it is written into
[`PROGRESS.md`](PROGRESS.md) as a question for the operator, with this as the
recommendation.

---

## TOOL-15. The twin comparison costs a thousand seconds, and half of it is one row

**Source** ruled by the operator 2026-09-01, from open question 7 of the previous session
**Category** tooling, **Priority** P2, **Effort** M, **Status** done

### Problem

`check-twins.sh` runs both halves of every pair and compares their answers. It
is the slowest thing in this repository by a wide margin, and a gate too slow to
run is a gate that gets run once at the end.

### Premise

⚠ **Measured, and the figures are in `check-gate.sh`'s own header** rather than
recalled: 403s for a full run against 106s for `--fast`, on one Windows 11
machine over 13 pairs, then 171s for `--fast` after the workspace landed and the
pair count reached 15. ⭐ The `check-no-secrets scoped` row alone took 70s in its
sh half on the machine that added it, because it greps every file of every
reference tree.

### Approach

⭐ **Scope the slow halves, never the comparison.** The cost is concentrated in
rows that walk `references/`, which holds nineteen other projects' trees; the
comparison itself is cheap.

Three routes, and the entry measures before it chooses:

| route | what it costs |
| --- | --- |
| run the expensive rows against a narrowed scope, with a fixture proving the scope rule | ⚠ a scope difference with nothing in the tree to exercise it is invisible to the comparison, which `scripts/README.md` already records |
| run the two halves of a pair concurrently | ⛔ refused on its own: it makes the tree-moved problem in `TOOL-16` worse rather than better |
| cache a half's answer against the tree's digest | a second thing to invalidate correctly |

⛔ **Never drop a pair to make it fit.** A twin that is written and not compared
is two behaviours, which is the rule in [`RULES.md`](RULES.md) section 4.

⛔ **And never wrap it in a timeout.** A killed half reports as a drift that is
not one, which is the same false positive `TOOL-16` is about.

Must not: change what is compared in order to change how long it takes.

### Prove

```bash
sh scripts/common/check-twins.sh --json
```

Passing means: every pair the file listed before this entry is still listed
after it; the run reports the same drift count it reported before; and a
timing taken on one host, with its conditions recorded, is materially under the
figure in the premise.

### Closing

**Closed 2026-09-01T14:05:00Z.** ⭐ **The measurement came first and it named
the row.** `--timings` prints the wall seconds each pair cost, and one row was
44 per cent of the file.

⚠ **Measured on one Windows 11 Pro 26200 host, 2026-09-01, twenty pairs, Git
Bash 5.3 and PowerShell 7.6.5, on a machine doing other things.** Seconds per
half, sh then ps:

```text
     122     36  check-docs
      20     13  check-markers
       3      2  check-one-home
       2      1  check-placeholders
      88     21  check-control-bytes
       0      1  check-changelog
      28      0  check-record
       4      1  check-no-secrets
       5      2  check-no-secrets pub
     107     56  check-no-secrets scoped
       1      1  check-msrv
       2      1  check-vendor
       2      1  check-corpus
       2      2  check-line-endings
       2      2  check-validate
       1      1  check-routes
       1      1  mine-repo selftest
     317    114  check-gate
       1      1  git-sync --check
       3      2  check-remote-items
```

⭐ **970 seconds across the pairs, and `check-gate` alone is 431 of them.** The
premise said "about a thousand seconds" and the wall clock for that run was
1056. ⚠ The two figures are different quantities: 970 is the sum of the pairs
and 1056 is the whole run, and the difference is the two probes and the setup.

### ⛔ The reason that row was expensive, and it was not the gate being slow

That row runs BOTH gates in full, and **each gate re-runs the fourteen checks
that already have a row of their own.** So fourteen rules were compared three
times each: once directly, once inside the sh gate, once inside the ps gate. The
two extra times cost more than everything else in the file put together.

⭐ **So a gate running inside `check-twins` now skips them.** What the pair
uniquely proves is untouched: the LIST each half runs, and the checks with no row
of their own, which are the two lints, the analyzer, the three suite entries and
the probe.

```text
$ CHECK_GATE_INNER=1 sh scripts/common/check-gate.sh --json
{"schema":"check-gate/1","total":23,"passed":8,"failed":0,"skipped":15,"strict":0}
inner exit=0
$ CHECK_GATE_INNER=1 pwsh -NoProfile -File scripts\common\check-gate.ps1 -Json
{"schema":"check-gate/1","total":23,"passed":8,"failed":0,"skipped":15,"strict":0}
inner exit=0
```

⛔ **The list going stale is covered, and for free.** The pair compares `skipped`
as well as `passed`, so a list that grows in one half and not the other fails
that row. ⚠ That is why the two lines above are quoted together: they are the
check on the change as well as the effect of it.

### The same measurement afterwards, same host, same day

```text
     120     50  check-docs
      20     24  check-markers
       3      2  check-one-home
       1      2  check-placeholders
      87     30  check-control-bytes
       1      0  check-changelog
      25      0  check-record
       3      1  check-no-secrets
       6      1  check-no-secrets pub
      89     32  check-no-secrets scoped
       1      1  check-msrv
       1      1  check-vendor
       3      1  check-corpus
       2      2  check-line-endings
       2      1  check-validate
       0      1  check-routes
       1      1  mine-repo selftest
      29     25  check-gate
       1      0  git-sync --check
       3      2  check-remote-items
```

| | before | after |
| --- | --- | --- |
| the `check-gate` row | 431s | ⭐ **54s** |
| every pair, summed | 970s | **575s** |
| the whole run, wall clock | 1056s | **636s** |

⭐ **A 40 per cent reduction in wall time, and the row that caused it is down by
seven eighths.** ⚠ Every figure is one run on a machine doing other things, and
the per-row numbers move by tens of per cent between runs: `check-docs` reads 122
then 120 in the sh half and 36 then 50 in the ps half, with nothing changed in
either. ⛔ The row that moved from 431 to 54 is outside that noise; the others
are inside it.

### ⛔ What was NOT done, and why each was refused

| considered | why it was not taken |
| --- | --- |
| scope `check-no-secrets scoped`, the second-largest row at 163s | ⛔ its whole subject is `references/`. Scoping it is deleting it, and it is the one row that turns the reference-corpus exemption OFF. |
| make `check-docs` and `check-control-bytes` cheaper | ⚠ a different entry. Those rows cost what the CHECKS cost, and making a check faster is not scoping a comparison. This entry's subject is the comparison. |
| run the two halves of a pair concurrently | ⛔ it makes `TOOL-16` worse: a tree moving under a run is already hard to tell from a drift, and overlapping the halves widens the window. |
| drop a pair | ⛔ refused by [`RULES.md`](RULES.md) section 4. A twin that is written and not compared is two behaviours. **Every pair the file listed before this entry is still listed after it.** ⚠ The measurement above was taken over twenty, of which `check-line-endings` and `check-validate` were added earlier the same session; `check-workflows` and `check-coverage` landed after it, so the file compares 22 and the two figures are over 20. |
| wrap it in a timeout | ⛔ a killed half reports as a drift that is not one. |

---

## TOOL-16. A tree that moved under the comparison reads as a drift

**Source** ruled by the operator 2026-09-01, from open question 8 of the previous session
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

`check-twins.sh` runs one half of a pair, then the other, then compares. A file
created or removed between the two is reported as a disagreement between two
implementations that agree.

⛔ **The failure mode is worse than a false alarm.** A session that learns to
discount a drift it has not re-checked will one day discount a real one.

### Premise

⭐ **Measured here, on this tree, 2026-09-01.** `repo.has_codegraph` came back
`sh=false ps=true` because `.codegraph/` was created between the two probes.
Both halves use the identical rule and both answer `true` now. ⚠ The only way to
tell that apart from a real drift was to re-run the pair by hand.

### Approach

⭐ **Record the tree's state before and after the run, and say so when they
differ.** A digest of the tracked-plus-untracked file list is enough: it is
cheap, it is what actually changed, and it does not need the run to be atomic.

⛔ **Do not try to make the run atomic.** Copying the tree per pair would cost
more than the comparison, and a lock would not stop an editor outside the
process.

⚠ **A drift reported alongside a moved tree is reported as UNDECIDED**, not as a
pass and not as a failure. Reporting it clean would hide a real drift that
happened to coincide with a write; reporting it failed is the false alarm this
entry exists to remove.

Must not: suppress a drift because the tree moved. The two facts are printed
together and the exit code says the run could not decide.

### Prove

```bash
sh scripts/common/check-twins.sh --json
```

Passing means: with a file deliberately created between two halves of one pair,
the run reports the tree digest changed, names that pair as undecided rather
than drifted, and exits 2; with the tree held still, the same pair is compared
and reported as it was before.

### Closing

**Closed 2026-09-01T13:40:00Z.** `check-twins` reads the tree's state before its
first pair and again after its last, and a drift reported alongside a moved tree
is now UNDECIDED rather than a failure.

⭐ **The failure mode reproduced itself while this entry was being worked, by
accident, and that is the best evidence it could have had.** A run of the gate
was stopped and its child processes outlived the stop; a second run started; and
the two wrote into one log. The second run's `check-twins` reported:

```text
  DRIFT  check-docs: the twins disagree
           sh: exit 0  {"schema":"check-docs/1","problems":0,"files":52,"links":774,"cited_paths":70,"shell_blocks":155}
           ps: exit 0  {"schema":"check-docs/1","problems":0,"files":52,"links":781,"cited_paths":70,"shell_blocks":155}
```

⚠ **774 against 781 links, and neither half is wrong.** Documents were being
edited between the sh half and the ps half. Re-run on a still tree, minutes
later, with nothing changed in either implementation:

```text
$ sh scripts/common/check-docs.sh --json
{"schema":"check-docs/1","problems":0,"files":52,"links":781,"cited_paths":70,"shell_blocks":155}
sh exit=0
$ pwsh -NoProfile -File scripts/common/check-docs.ps1 -Json
{"schema":"check-docs/1","problems":0,"files":52,"links":781,"cited_paths":70,"shell_blocks":155}
ps exit=0
```

⭐ **That is the second independent instance**, after `repo.has_codegraph` last
session, and the two failed on different pairs for the same reason. ⛔ A session
that had not re-run this by hand would have gone looking for a difference
between two files that are identical.

### ⭐ The guard, planted and seen to fire

⛔ **A guard whose test has never been seen to fail is theatre.** So it was
arranged: `check-twins --json` was started, and the closings for this session's
entries were written into `docs/history/todo/*.md` while it ran.

```text
$ sh scripts/common/check-twins.sh --json
  twin pairs, same tree:
  DRIFT  check-docs: the twins disagree
           sh: exit 0  {"schema":"check-docs/1","problems":0,"files":52,"links":787,"cited_paths":70,"shell_blocks":155}
           ps: exit 0  {"schema":"check-docs/1","problems":0,"files":52,"links":790,"cited_paths":72,"shell_blocks":155}
{"schema":"check-twins/2","drift":1,"tree_moved":true}
exit=2
```

⭐ **787 links against 790, and 70 cited paths against 72**, because three more
links and two more code-span paths were written between the sh half and the ps
half. ⛔ Before this entry that was `drift:1` and exit 1, and a session reading
it would have gone looking for a difference between two implementations that are
identical.

⚠ **The JSON and the exit code are what a caller reads and both are proved
here.** The prose banner is the same branch on the same two variables and is not
reached under `--json`; it is not separately demonstrated, and saying so is
cheaper than pretending it was.

⚠ **The schema moved to `check-twins/2`** because the shape gained
`tree_moved`. Nothing in this tree parsed `check-twins/1`: the gate reads its
exit code.

### What it does now, and what it refuses to do

⭐ **Three readings, and each catches something the others do not.** The digest
covers `git ls-files -s`, which catches a staged change; `git status
--porcelain`, which catches an edit and an untracked file; and a listing of the
repository root, which catches a new top-level directory that `.gitignore` hides
from both. ⚠ That third one is not padding: `.codegraph/` is exactly such a
directory, and it is what produced last session's phantom drift.

⛔ **The run is not made atomic, and that is the ruling rather than a shortcut.**
Copying the tree per pair would cost more than the comparison, and a lock would
not stop an editor outside the process. Recording what changed is cheap and it
answers the question that was actually being asked.

⛔ **Undecided is exit 2, which is "could not run".** Reporting a coincident
drift as clean would hide a real one that happened to land during a write;
reporting it as drift is the false alarm this entry removes. ⚠ Under
`check-gate --strict` a skip is a failure, so continuous integration still goes
red on an undecided run rather than passing over it.

⚠ **A moved tree with NO disagreement is reported and passes.** Nothing is in
doubt there; it is printed because a moved tree is the one thing that makes a
future drift unbelievable.

---

## TOOL-17. The gate's line-endings filter cannot see the working tree

**Source** ruled by the operator 2026-09-01, from open question 9 of the previous session
**Category** tooling, **Priority** P1, **Effort** S, **Status** done

### Problem

Both halves of `check-gate` filter `git ls-files --eol` on the INDEX column, so
a tracked file that is CRLF in the working tree and LF in the index passes.

⚠ **Eight files became CRLF in a session that declares `eol=lf`, and the gate
stayed green throughout.** `.gitattributes` normalised them on commit, so
nothing reached the history and nothing said anything was wrong. The defect the
check exists to catch was present in the tree and invisible to the check.

### Premise

⭐ **Measured on this tree.** `git ls-files --eol` reports 4428 files at
`i/lf w/lf`, 82 at `i/lf w/crlf`, 93 at `i/none w/none`, 783 at
`i/-text w/-text` and 2 at `i/mixed w/mixed`. ⚠ Every one of the 82 is a `.ps1`
declaring `eol=crlf`, so on the tree as it stands today the working-tree column
is already correct and this check would pass. The defect is that it would also
have passed on the tree that carried the eight.

### Approach

Read the WORKING-TREE column as well, and compare it against what the attributes
declare rather than against a fixed value.

| declared | the working tree may be |
| --- | --- |
| `eol=lf` | `w/lf`, or `w/none` for a file with no line ending at all |
| `eol=crlf` | `w/crlf`, or `w/none`. ⭐ `references/` and every `.ps1` are legitimately CRLF on disk, and `docs/conventions/shell.md` section 8 says why. |
| `-text` | anything. The bytes are the content. |

⛔ **Honour the attribute rather than the extension.** The reference corpus
carries its own `.gitattributes` files, so a `.ps1` under `references/` resolves
through the nested one; a rule that matched on `*.ps1` here would be a second
answer to a question git already answers.

⚠ **Both halves, and the filter stays identical between them.** The PowerShell
half splits the same output on whitespace and must reach the same verdict.

Must not: report a working-tree difference as a failure of the index, which is a
different fact and a different fix.

### Prove

```bash
sh scripts/common/check-gate.sh --json
```

Passing means: with a tracked `eol=lf` file rewritten with CRLF in the working
tree and not staged, both halves fail the `line-endings` check and name the file;
with the tree as it stands, both pass over the 82 files that are CRLF on purpose;
and a file declared `-text` is not reported either way.

### Closing

**Closed 2026-09-01T13:30:00Z.** The rule reads both columns now, and it is a
check with two halves and a row in the twin comparison rather than eight lines
computed inline in each half of the gate.

⭐ **It found a live defect in this tree on its first run**, which the filter it
replaces could not see:

```text
$ sh scripts/common/check-line-endings.sh
line-ending check failed, 1 file(s) over 5388 tracked:

  worktree i/lf    w/lf    attr/text eol=crlf    	scripts/common/check-routes.ps1

An "index" finding is what a commit would contain and is fixed by
renormalising. A "worktree" finding is what is on disk and reaches no
commit, which is exactly why nothing else notices it.
exit=1
$ pwsh -NoProfile -File scripts/common/check-line-endings.ps1 -Json
{"schema":"check-line-endings/1","files":5388,"index":0,"worktree":1,"problems":1}
exit=1
```

⚠ **`check-routes.ps1` was LF on disk and `eol=crlf` in its attributes.** It was
written last session by a tool that writes LF, the attributes normalised it into
the index, and every check in this tree reported green over it. ⛔ The
declaration is not decoration: Windows PowerShell 5.1 mis-parses a here-string
whose terminator arrives with a bare LF, and
[`../docs/conventions/shell.md`](../../conventions/shell.md) section 8 is why
the exception exists at all.

⭐ **Fixing it produced no git diff**, which is the whole shape of the defect:

```text
$ node -e '... rewrite the file with CRLF ...'
before: 0 CRLF of 203 line endings
after:  203 CRLF of 203
$ git diff --stat -- scripts/common/check-routes.ps1
$ sh scripts/common/check-line-endings.sh
line endings ok: 5388 tracked file(s), index and working tree both agree
with what .gitattributes declares.
exit=0
```

### ⭐ The scope grew during implementation, and here is what changed

The entry as authored said "read the working-tree column too" in both halves of
`check-gate`. Implementing it that way would have left the rule where it was:
**two copies computed in two languages and compared by nothing.**
`check-twins` compares PAIRS OF SCRIPTS, so a rule with no script of its own has
no row, which is exactly how it went eight files wrong without anybody noticing.

⭐ So the rule was extracted into `scripts/common/check-line-endings.{sh,ps1}`
and given a row. That is a larger change than the entry asked for and the gate
was re-passed against it, per
[`../docs/methodology/gate.md`](../../methodology/gate.md).

⚠ **The Prove command below is the entry's own and still exercises it**, through
the gate, under the name `check-line-endings` rather than `line-endings`.

### What the rule is now, and what it deliberately does not judge

| the column | what it decides |
| --- | --- |
| index | what a commit will contain. Unchanged in substance: `i/lf`, `i/none` and an empty entry pass. |
| working tree | what is on disk, compared against **what the attributes declare** rather than against a fixed value |

⛔ **Out of scope, each for a stated reason**: `attr/-text`, because the bytes
are the content; `i/-text` and `w/-text`, because git detected binary content
whatever the attributes say; `i/none w/none`, because a file with no line ending
at all is a shape `PUB-03` publishes deliberately; and any file whose attributes
declare no `eol` at all, because there is nothing to compare a working tree
against.

⚠ **Measured on this tree**: 5388 tracked files, of which 83 are CRLF on disk on
purpose. Every one is a `.ps1` declaring `eol=crlf`, and 66 of those are inside
`references/`, where the declaration comes from a nested `.gitattributes` the
mined tree brought with it. ⛔ A rule matching `*.ps1` would have been a second
answer to a question git already answers, and it would have got those 66 from
the wrong file.

## TOOL-18. The gate is slow because of how it reads files, not because of what it reads

**Source** the operator, 2026-09-02: why do the checks take so long, and are they
reading the vendored and reference trees
**Category** tooling, **Priority** P1, **Effort** M, **Status** done

### Problem

⛔ **The full gate costs about ten minutes on this Windows host and the Rust
work in it is twenty-four seconds.** `TOOL-15` measured the same shape on
2026-09-01, added `--timings`, and closed: the cost was named and not reduced.
⚠ A gate that costs ten minutes gets run once at the end, which is what
happened twice in the session that filed this.

### Premise

⭐ **Measured on 2026-09-02, warm, on one Windows 11 Pro 26200 host with Git
Bash, sh halves only.** ⛔ These are seconds for the sh half alone;
`check-twins` runs both halves of all 27 pairs, so the full gate pays roughly
this twice.

| check | seconds | what it reads | references or vendor |
| --- | --- | --- | --- |
| `check-docs` | 175 | 53 documents, 959 links | ⭐ neither. Both excluded by `grep -vE '^(references\|vendor/[^/]+)/'` |
| `check-control-bytes` | 121 | 384 files | ⛔ **vendor yes**, 146 of the 384. `references/` excluded, `vendor/NAME/` not |
| `check-no-secrets --scope references` | 102 | 4972 files | ⭐ yes, and that is the entire point of the row |
| `check-record` | 34 | 97 entries | neither |
| `check-markers` | 29 | 241 files | ⭐ neither, same exclusion as `check-docs` |
| the other thirteen | 28 together | | |
| `cargo fmt`, `clippy`, `test` | 24 together | the workspace | |

⛔ **The cause is a subprocess per file, on a host where a subprocess costs
54.5 milliseconds.** Measured: 100 bare `grep` spawns took 5450 ms. The hot
loop of `check-control-bytes` spawns roughly six per file, so 384 files cost
about 126 seconds, against 121 measured. ⚠ The arithmetic accounts for the
whole of it, which is what says the cause is the loop rather than the data.

⭐ **And the counter-example is in the same gate.** `check-line-endings` reads
every one of the 5435 tracked files, references and vendor included, in **2.4
seconds**, because it asks `git ls-files --eol` once instead of looping.

### Approach

⭐ **One pass per check, not one pass per file.** Every slow check already has
a scoped file list; feed the whole list to one `grep`, one `awk`, or one
`git` invocation and read filenames out of the output, rather than spawning a
pipeline per name.

⛔ **Nothing about what is checked may change.** `TOOL-15` states it as a rule
and it stands: never change what is compared in order to change how long it
takes. The file lists, the exclusions and the reported counts are identical
before and after, and that is the acceptance below.

⚠ **`check-control-bytes` reading `vendor/NAME/` is a separate question from
the speed one.** Every other prose check and the secret scan exempt it by
directory, because it is somebody else's source at a recorded commit. This
entry does not decide that: it records that 146 of the 384 files are vendored,
so a reader can see what an exemption would and would not buy.

Must not: run the halves of a pair concurrently, or wrap a half in a timeout.
`TOOL-15` refuses both by name, and `TOOL-16` is the false drift they cause.

Must not: cache a half's answer. A second thing to invalidate correctly, over a
check whose whole job is to be believed.

### Consumers

Nothing is published yet, so there are no consumers to break. ⚠ Every one of
these scripts is meant to be copied into other projects, so a per-file loop
copied out of here is a slow gate somewhere else as well.

### Prove

⛔ **The acceptance, and it is a command.**

```bash
sh scripts/common/check-twins.sh --timings
```

Passing means: exit 0; every pair listed before this entry is still listed
after it; each rewritten check reports the identical `--json` payload it
reported before, file counts included; and the per-pair seconds for
`check-docs` and `check-control-bytes` are materially under the 175 and 121 in
the premise, on a host whose conditions are recorded beside the figure.

---

### ⭐ Closed 2026-09-02. One pass per check, and the cause was not the one the premise named

⛔ **The premise blamed a subprocess per file, and that was two thirds of it.**
The third cause was bigger than either and it is not a per-file loop at all: a
command substitution in a `while ... read` **assignment prefix** is re-evaluated
on every iteration, so `IFS="$(printf '\t')" read ...` forks once per LINE READ.
⚠ Measured on this host: a command substitution costs 35 ms, and `check-docs`
reads about 1100 findings.

| the cause | where it was | what it cost |
| --- | --- | --- |
| ⛔ **a fork per line read** | `IFS="$(printf '\t')"` on eleven `while read` loops across seven checks | about 39 seconds in `check-docs` alone, and it is why `check-record` and `check-markers` were slow for no visible reason |
| ⛔ **a `git check-ignore` per link** | `check-docs`, once for every one of 966 link targets that resolves | 52 seconds, now one `git check-ignore --stdin` |
| ⛔ **three processes per fenced block** | `check-docs`: `tr`, `sh -n`, `grep -q` per block, 163 blocks | two of the three moved into the `awk` pass that extracts the block. Only `sh -n` still needs a process, because it is the parse rather than a search |
| ⛔ **six processes per file** | `check-control-bytes`, over 387 files | the C0 class is one `grep` over the whole list now, and the NUL question is asked of the whole list at once and falls back to naming files only when the answer is yes |

⭐ **Measured before and after, on the same tree, with the payloads compared:**

| check | before | after | payload |
| --- | --- | --- | --- |
| `check-control-bytes` | 126 s | **1.1 s** | `{"schema":"check-control-bytes/1","problems":0,"files":387}`, identical |
| `check-docs` | 173.6 s | **28 s** | `{"schema":"check-docs/1","problems":0,"files":53,"links":966,"cited_paths":98,"shell_blocks":163}`, identical |
| `check-markers` | 29 s | **15.6 s** | identical |
| `check-record` | 34 s | **27 s** | identical |

⛔ **Nothing about what is checked changed**, which is the rule `TOOL-15` set
and this entry kept. The file lists, the exclusions and every reported count are
the same before and after, and the `--json` payload is what says so.

#### The acceptance

```text
$ sh scripts/common/check-twins.sh --timings
  ok     check-docs: both say {"schema":"check-docs/1","problems":0,"files":53,"links":966,"cited_paths":98,"shell_blocks":163}, exit 0
  ok     check-control-bytes: both say {"schema":"check-control-bytes/1","problems":0,"files":387}, exit 0
  ok     check-record: both say {"schema":"check-record/1","problems":0,"entries":98,"open":33,"blocked":0,"done":65}, exit 0

  seconds per half, sh then ps:
      28     54  check-docs
      15     19  check-markers
       3      3  check-one-home
       2      1  check-placeholders
       1     27  check-control-bytes
       1      0  check-changelog
      27      1  check-record
       5      1  check-no-secrets
       6      2  check-no-secrets pub
     170     36  check-no-secrets scoped
       1      1  check-msrv
       4      0  check-vendor
       3      1  check-corpus
       2      2  check-line-endings
       2      2  check-validate
       2      1  check-workflows
       1      1  check-coverage
       1      1  check-routes
       2      9  check-exit-codes
       0      1  check-staleness
       1      0  check-sources
       3      1  check-manual-path
       2      3  check-provisioning
       1      1  mine-repo selftest
      43     34  check-gate
       1      1  git-sync --check
       5      3  check-remote-items

✅ every twin pair agrees on this tree.
exit=0
```

⚠ **Conditions**: one Windows 11 Pro 26200 host, Git Bash, warm, 2026-09-02,
with the tree at the commit that closed `DRIVER-08`. Every pair listed before
this entry is still listed.

#### ⛔ What this did NOT fix, named rather than left

| | |
| --- | --- |
| ⛔ **the PowerShell halves still carry the per-file shape** | `check-control-bytes.ps1` is 27 s against its twin's 1 s, and `check-docs.ps1` is 54 s against 28 s. The premise measured the sh halves and this entry rewrote the sh halves; the ps1 halves are the same work again and they are not done. ⚠ A gate run on Windows pays the slow half, and `check-twins` pays both. |
| ⛔ **`check-no-secrets --scope references` is now the largest row by far** | 170 s of the sh side's 333. It is already batched through `xargs`, so it is not the shape this entry fixed: it reads 4972 files because that is the row that exists to scan what every other check exempts. |
| ⚠ **`check-record` is 27 s and its cause is known** | three processes per entry across 98 entries: an `awk` and a `head` to find each entry's file, and a second `awk` to read its status. One pass would fix it the way `check-docs` was fixed. |
| ⚠ **`check-control-bytes` still reads `vendor/NAME/`** | 146 of its 387 files are vendored. This entry records it and does not decide it, exactly as the approach said. |

⭐ **The fast gate went from about 600 seconds to 246**, measured on this host
with `check-provisioning` newly inside it, so the comparison is against a gate
that now runs one more check than the one that cost 600.

---

## TOOL-19. A catalogue nothing checks stops being a catalogue

**Source** found by this session's consolidation pass, 2026-09-03, comparing the documents against the tree
**Category** tooling, **Priority** P1, **Effort** M, **Status** done

### Problem

[`../AGENTS.md`](../../../AGENTS.md) sends a session writing a script to
[`README.md`](../../../scripts/README.md), calling it the contract every script is
held to, and sends a session writing a document to the router's own table of
what each document owns. ⛔ **Neither catalogue is checked against the tree**,
so a script or a document arrives, the catalogue is not touched, and the gate
stays green over a contract that no longer covers the thing it claims to.

### Premise

⭐ **Measured on this tree, 2026-09-03, before the consolidation pass ran.**

```bash
for f in $(ls scripts/common | sed 's/\.\(sh\|ps1\|mjs\)$//' | sort -u); do
  grep -q "$f" scripts/README.md || echo "$f"
done
```

```text
check-cold-start
check-coverage
check-data-branch
check-formats
check-license-consistency
check-line-endings
check-notes-generator
check-pr-body
check-publish
check-release
check-support-matrix
check-trust-anchors
check-workflows
```

⚠ **Thirteen of the checks the gate runs had no section at all**, and two more
carried rows naming entries that had closed four sessions earlier. The document
half was the same shape: the technical reference had no row in the set table of
[`../docs/conventions/docs.md`](../../conventions/docs.md), which is the
document that says a role with no file behind it is a defect.

### Approach

One check pair, `check-catalogues`, holding the two rules a machine can hold.

- ⭐ **Every script is named by [`README.md`](../../../scripts/README.md).** Twins
  collapse to one base name, because a pair is one contract.
- ⭐ **Every document under [`../docs/`](../../) is named by its index**:
  [`../AGENTS.md`](../../../AGENTS.md) for the tree, and
  [`../docs/history/README.md`](../README.md) for the history
  directory, which has its own because a superseded page is not routed to.
- ⚠ **Both directions, on the paths.** A catalogue naming a file the tree does
  not have is the `TOOL-10` defect, and it is the same reading.

⛔ Must not: assert prose. Whether a section is any good is a review, and a
guard that tried would either pass vacuously or refuse legitimate writing.
⛔ Must not: exempt by a list that grows. An exemption is a row somebody has to
delete, and this tree has had one of those go stale already.

### Prove

```bash
sh scripts/common/check-catalogues.sh
```

Passing means exit 0 with every script and every document named by its
catalogue, and the refusal fixtures reporting that a missing script section and
a missing document row are each refused.

### Closing

**Closed 2026-09-03T13:07:00Z.** One check pair, in the gate and in
`check-twins`, holding the two rules a machine can hold.

```text
$ sh scripts/common/check-catalogues.sh
catalogues ok: 44 script(s) named by scripts/README.md, 28 document(s)
named by the index that owns each one.
exit=0

$ sh scripts/common/check-catalogues.sh --fixture
check-catalogues fixture ok: an unlisted script and an unlisted document
are both refused.
exit=0
```

### ⭐ It was run against the tree it was written for, and it refuses it

⛔ **A guard that has only ever been seen to pass is theatre**, so the check was
pointed at this repository as it stood before the consolidation, at commit
`8f031a6`, exported into a scratch directory and read with `--fixtures`. ⚠ That
mode walks the filesystem rather than asking git, because a tree that is not
this repository has no index to ask.

```text
$ git archive 8f031a6 scripts docs | tar -x -C .tmp/head-tree
$ sh scripts/common/check-catalogues.sh --fixtures .tmp/head-tree
catalogue check failed, 13 of 43 script(s) and 27 document(s):

  check-cold-start has no mention in scripts/README.md
  check-coverage has no mention in scripts/README.md
  check-data-branch has no mention in scripts/README.md
  check-formats has no mention in scripts/README.md
  check-license-consistency has no mention in scripts/README.md
  check-line-endings has no mention in scripts/README.md
  check-notes-generator has no mention in scripts/README.md
  check-pr-body has no mention in scripts/README.md
  check-publish has no mention in scripts/README.md
  check-release has no mention in scripts/README.md
  check-support-matrix has no mention in scripts/README.md
  check-trust-anchors has no mention in scripts/README.md
  check-workflows has no mention in scripts/README.md

A script gets a section in scripts/README.md and a document gets a row
in the index that routes to it. docs/conventions/docs.md.
exit=1
```

⭐ **The PowerShell half prints the same thirteen and the same exit code** over
the same directory, which is how the pair was compared before `check-twins` ran
over it.

### ⛔ The check's own first run found a defect in the check

**It asked `git ls-files` alone**, and reported a clean catalogue of 43 scripts
while its own half sat beside it, written that minute, unlisted and uncommitted.
⛔ **A script written this minute is the one most likely to be missing from the
catalogue**, and it was the one the scope could not see.

`check-docs` had already learned this and says so in its own header. Both halves
now read the tracked list plus the untracked-but-not-ignored one, and the count
went from 43 to 44 the moment it could see itself.

### ⚠ What this does NOT hold, and the boundary is the point

| | |
| --- | --- |
| whether a section says anything true | ⛔ a reading. A guard over prose either passes vacuously or refuses legitimate writing, and [`../docs/methodology/reviews.md`](../../methodology/reviews.md) lens 3 is what owns it |
| whether a catalogue names a file that is gone | `check-docs`, which resolves every cited path in every markdown file here. ⛔ Two checks holding one rule is two places for it to be wrong |
| whether an entry a document cites is still open | ⚠ nothing holds it, and this session found three rows naming closed entries as open. A grep for that shape has no honest form: the phrasing is prose |

⛔ **And it does not exempt anything.** An allowlist is a row somebody has to
delete, and [`RULES.md`](RULES.md) section 4 already carries what a stale
exemption cost here.
