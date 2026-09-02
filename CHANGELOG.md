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

### 2026-09-02T02:45:00Z - the linux64 profile lands, and the matrix column reaches the driver

**Record:** [`TODO/corpus.md`](TODO/corpus.md) `CORPUS-02`,
[`TODO/driver.md`](TODO/driver.md) `DRIVER-01`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **The corpus holds three profiles**, two of them captured on machines
  nobody owns. `chrome/stable/linux64/151.0.7922.173` from `ubuntu-latest` is
  the first, and the run that produced it reported `1 cold, 0 resumed,
  6 further cold, 1 abandoned` where the two runs before the resumption fix
  reported `0 cold`.
- ⛔ **The capture matrix's `browser` column reached nothing.**
  `b-ids-driver drive` took the first family that resolved, and the capture
  script wrote the literal `Chrome` into every identity file, so an `edge` lane
  would have published Chrome under Chrome's route inside an artefact called
  `edge`.
- ⭐ **`b-ids-driver --browser NAME`**, for `resolve` and for `drive`. A name
  with no branch is refused and names what it knows; a family this machine
  lacks exits 2, because no browser here and the capture failed are different
  facts.
- ⭐ **`Family::vendor_name`, reported as `Resolved.name`**, so the capture
  script reads the spelling a profile records instead of carrying its own
  table. A test asserts it lower-cases to the family, which is what the corpus
  route is built from.
- **`edge/stable/linux64` is enabled and required** in the plan, with the
  measurement that unblocked it written into its `why`: the driver resolved
  Edge `151.0.4129.101` at `/usr/bin/microsoft-edge` on the `ubuntu-latest`
  image.

### 2026-09-02T02:20:00Z - the capture matrix runs, and a lane that captured nothing says why

**Record:** [`TODO/corpus.md`](TODO/corpus.md) `CORPUS-02`,
[`TODO/harness.md`](TODO/harness.md) `HARNESS-07`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`capture.yml` has run on hosted runners**, twice, every job green. The
  `win64` lane's profile is published: Chrome `151.0.7922.174` from
  `windows-latest`, the first capture in this corpus taken on a machine nobody
  owns.
- ⛔ **`b-ids-corpus add` printed `1 cold` as a literal**, so its report claimed
  a cold connection on a run that had none, on the line above the refusal saying
  it had none. The line is `Selection::report` now, where a test reaches it.
- ⭐ **`b-ids-harness --no-resumption`**, which issues no session tickets so the
  subject cannot resume and every hello is a cold one. It is REFUSED without
  `--ca-out`, because resumption is a property of the terminator and a switch
  that reached nothing would be a flag no code reads.
- ⭐ **`captured.resumption` in the schema and in the published contract**,
  absent rather than defaulted on a profile written before the field existed,
  and read back from what the harness reported rather than typed beside the
  capture.
- ⭐ **`experiments/30-resumption-control.sh`**, the control that says the switch
  is safe: 19 TLS fields compared across three rounds, 0 differing, 2 not
  comparable because they carry a per-connection draw.
- **`compare-modes --labels A,B`**, because a driver that called a terminating
  run `raw` was a display that lies.

⛔ **Two findings about the runners, both measured rather than assumed.**
`ubuntu-latest` and `windows-latest` do not serve the same Chrome build
(`151.0.7922.173` against `151.0.7922.174`), so one build on two platforms needs
pinned acquisition. And the `linux64` lane produced no cold connection at all,
twice, because Chrome abandoned the connections that were not resumed.

### 2026-09-01T15:10:00Z - acquisition, the capture matrix, and a coverage report

**Record:** [`TODO/driver.md`](TODO/driver.md) `DRIVER-05`,
[`TODO/ci.md`](TODO/ci.md) `CI-03`, [`TODO/corpus.md`](TODO/corpus.md)
`CORPUS-02`, and [`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-driver::acquire`.** A build has more than one route, they are tried
  in order, and what answered is recorded on the profile with the digest of what
  arrived. ⛔ The artefact is never redistributed: the URL and the digest are.
- ⭐ **`.github/workflows/capture.yml`**, the fan-out, with every lane allowed to
  fail alone, a collect job that runs regardless, and a fuzz lane that overrides
  the pinned toolchain explicitly.
- ⭐ **`.github/capture-matrix.json`**, the plan, in one place. The workflow
  builds its matrix from it and `check-coverage` reads the same file to say what
  landed.
- **`check-workflows` and `check-coverage`**, both halves, both in the gate and
  both with a row in the twin comparison.
- **`captured.acquisition`** in the schema and in the published contract,
  omitted when absent so a profile written before it still serialises as it was.

⛔ **Three defects in this session's own new code**, each found by running it
rather than by reading it: an uninitialised awk variable used as a subscript is
the empty string and not zero; jq on Windows writes CRLF, which made one half's
human report disagree with its twin while the JSON matched; and the driver
cannot link the harness, so the digest is injected rather than computed.

⚠ **`CORPUS-02` is open with its blocker named.** The apparatus is built and no
lane has run: closing it needs one run of the matrix on a hosted runner and the
`linux64` profile committed.

### 2026-09-01T14:10:00Z - the assertions a push makes, and three checks that were not checking

**Record:** [`TODO/ci.md`](TODO/ci.md) `CI-01`,
[`TODO/tooling.md`](TODO/tooling.md) `TOOL-04`, `TOOL-15`, `TOOL-16`, `TOOL-17`,
and [`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-corpus validate`, and `check-validate` in both halves.** The
  coherence checks now run over what is PUBLISHED rather than over whatever a
  caller listed, and the cross-profile `shared_handshakes` runs with them. A
  second leg asserts the generator answers the same way twice, which
  `b-ids-corpus verify` structurally cannot see.
- ⭐ **`.github/workflows/validate.yml`**, on every push, on two hosts, with
  `CARGO_NET_OFFLINE` set for every assertion step and the whole history
  fetched.
- ⭐ **`check-line-endings`**, extracted from inside both gate halves, reading
  the working-tree column as well as the index one.
- **`check-twins --timings`**, and a scope taken from what it measured.

⛔ **Three checks were reporting green over questions they had not asked.**

| the check | what it was not checking |
| --- | --- |
| `check-corpus` | its history leg ran under `actions/checkout`'s default one-commit clone, so `git log --diff-filter=MDR` saw a single commit and answered "nothing was edited" on every CI run since it was written |
| the gate's line-endings filter | it read git's INDEX column alone, so a file that is CRLF on disk in a tree declaring `eol=lf` passed. It found `scripts/common/check-routes.ps1` LF on disk against its own `eol=crlf` on its first run. |
| `mine-repo` | it exited before the clone when its API route was down, so a host that could clone and not reach the API got nothing at all |

⭐ **`check-twins` costs 636 seconds now rather than 1056**, and the row that
caused it went from 431 to 54. ⛔ Measured with `--timings` before and after on
one host rather than estimated, and no pair was dropped: one was added.

⚠ **A drift reported alongside a moved tree is UNDECIDED now**, not a failure.
It reproduced itself while the entry was being worked: a stopped run's children
outlived the stop, and `check-docs` reported 774 links against 781 for two
implementations that agree exactly on a still tree.

### 2026-09-01T10:17:42Z - latest means stable, and a route check that can refuse

**Record:** [`TODO/tooling.md`](TODO/tooling.md) `TOOL-06`,
[`TODO/corpus.md`](TODO/corpus.md) `CORPUS-03`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- **`check-routes`, in both halves and in the gate.** No published file that
  carries exactly one value may end with a newline, because a consumer of one
  should never have to strip anything. The requirement came from `od -c` over
  somebody else's published dataset, not from taste.
- ⭐ **`latest` means stable and it cannot mean anything else.** The pointer
  file splits `latest`, built from stable profiles alone, from `per_channel`,
  which carries every channel under its own name. The rule is enforced by
  construction and the written file is compared against the derivation, so a
  hand-edited pointer is refused twice.

⛔ **The fixture written to prove `check-routes` could refuse found that it
could not.** `git ls-files` answers a path outside the repository with an empty
list, so both halves reported "ok, 0 files" over the file meant to make them go
red. A route tree that yields no single-value file is exit 2 now, because a
check that reports clean over nothing quietly stops applying.

⚠ **`CORPUS-03`'s own Approach asked for its three rules in two documents**, and
`check-one-home` refused the tree for it. The README owns the wording, the entry
points at it, and the amendment is recorded rather than made silently.

### 2026-09-01T09:48:20Z - a million runs at the parsers, and no panic

**Record:** [`TODO/harness.md`](TODO/harness.md), `HARNESS-09`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **One million coverage-guided runs, no crash and no timeout**, over every
  parser the harness exposes to the network. libFuzzer discovered the HTTP/2
  connection preface on its own, which is what says it reached the frame reader
  rather than bouncing off a length check.
- ⭐ **A second half that runs everywhere.**
  `cargo test -p b-ids-harness hostile` drives the same function over 6767
  mutations of the committed captures in under half a second, with no nightly
  toolchain, so the property is held by the ordinary gate rather than by a tool
  somebody has to remember.
- **`fuzz/`**, a cargo-fuzz crate excluded from the workspace for the reason the
  vendored tree already paid for.

⛔ **Windows cannot run the coverage-guided half, and three routes were
measured** rather than one being assumed. [`fuzz/README.md`](fuzz/README.md)
carries what stopped each: a missing AddressSanitizer runtime, a linker with no
section-boundary symbols, and libFuzzer's Windows shim not compiling under
mingw. The run above is from a Linux container, and the engine was left exactly
as it was found.

⚠ **A trap worth knowing before a CI job hits it.** The pinned
[`rust-toolchain.toml`](rust-toolchain.toml) applies to the fuzz crate too, so a
nightly image is not enough: rustup reads the file, fetches the pinned stable,
and `-Z sanitizer` is then refused. The toolchain is overridden explicitly.

### 2026-09-01T09:27:16Z - the build that is serving, not the one that is published

**Record:** [`TODO/driver.md`](TODO/driver.md), `DRIVER-02`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **`b-ids-driver versions`**, which reads the rollout fraction rather than
  the top of the list, cross-checks the automation-build index, and prints what
  the naive answer would have been beside its own.
- ⭐ **The inherited defect reproduced here, to the digit.** Highest known
  `153.0.8010.12` at fraction `0.005`, the build at full rollout
  `152.0.7977.65`, the automation index at `152.0.7977.64`, and the two
  first-party sources still one patch component apart.
  [`docs/inherited-claims.md`](docs/inherited-claims.md) section 7 carries the
  confirmation beside the reading it inherited.
- **Version ordering has one home**, `b_ids_schema::version_order`. The corpus
  and the driver were about to hold two copies of a comparison where
  `152.0.7977.9` sorts after `152.0.7977.64` if it is done as text.

⚠ **It reveals a gap in this project's own corpus.** The one profile is Chrome
`151.0.7922.76` and stable is serving `152.0.7977.65`, so the corpus is a major
behind. Nothing said so before this command existed.

⛔ **No HTTP client was added.** The command shells out to a fetcher the host
already has, one process per request, because an HTTP client crate brings its
own TLS stack into a workspace that vendors one.

### 2026-09-01T09:13:54Z - measuring did not change what was measured

**Record:** [`TODO/harness.md`](TODO/harness.md), `HARNESS-10`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet.

What landed:

- ⭐ **The answer.** Chrome `151.0.7922.76` offers the same hello whether or not
  the harness completes the handshake: seventeen of nineteen compared TLS fields
  agree exactly, none differ, and the two that cannot be compared carry a value
  the browser draws per connection. One browser, one build, one host, one day.
- **`b_ids_harness::modes`**, which measures each field's stability inside a run
  before comparing across runs, so a per-connection draw is reported as not
  comparable rather than as a difference.
- ⭐ **`experiments/20-compare-capture-modes.sh`**, which drives one resolved
  browser at both surfaces over several rounds.

⚠ **A second finding, which is a mode effect and is not a field.** Only a
surface that completes a handshake can produce a resumption: the raw run resumed
none of eighteen connections and the terminating run resumed eleven. That is
why the comparison excludes resumed connections and prints the counts beside the
field list.

⛔ **The trust-store question is NOT what this answered.** The record's work
order framed this entry as a pin against a real trust anchor; the entry itself
asks for the raw surface against the terminating one, and that is what closed.
Installing a root into a machine's trust store is the operator's action, and it
stays an open question with that recommendation attached.

### 2026-09-01T08:27:32Z - the corpus holds a profile

**Record:** [`TODO/corpus.md`](TODO/corpus.md), `CORPUS-01`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet; the profile is
committed on the default branch and no release and no data branch exist.

What landed:

- ⭐ **The first profile.** Chrome `151.0.7922.76` on Windows, captured by this
  project's own harness, at `corpus/v1/chrome/stable/win64/151.0.7922.76.json`
  with the `ClientHello` bytes it was read from beside it under `raw/v1/`.
- **A store that refuses rather than repairs.** `b-ids-corpus` turns the cold
  connection of a navigation into a profile, writes it once, and refuses a
  route that already holds one. The index and the latest-per-key pointer are
  derived from the tree rather than appended to.
- **`check-corpus`, in both halves and in the gate.** Its git leg asks whether
  a published file was ever modified, deleted or renamed after its first
  commit, which is the one question the working tree cannot answer.
- ⭐ **`experiments/`**, and the script that took the capture, so the run is
  repeatable rather than a transcript.

⚠ **The capture record moved to `harness-capture/4`**: a capture now carries the
instant it was accepted, because a profile's capture instant is never optional
and a reader that stamped one later would record when it read the file.

⚠ **The profile model gained `captured.trust` and `captured.switches`.** Every
capture this project has taken went through a per-launch key pin, and a corpus
that cannot say which profile was taken under which configuration cannot answer
whether the configuration changed the answer.

⛔ **The secret scan went red on the first profile and the hex rule was not
widened.** Two more narrow exclusions, both halves, and both mutation-proved: a
value under a field named `sha256`, and a line under `corpus/` or `raw/` that is
nothing but a quoted hex run. `TOOL-03` is the entry that predicted this shape
would arrive.

### 2026-09-01T05:40:00Z - the first bytes a browser put on a wire

**Record:** [`TODO/harness.md`](TODO/harness.md), `HARNESS-13`, `HARNESS-02`
and `HARNESS-05`, and [`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet, and the corpus
is still empty.

What landed:

- ⭐ **The handshake is terminated.** `--ca-out` mints a per-run authority,
  writes it, and selects a surface that completes a verified handshake and
  reads whatever the peer sends over it. Chrome `151.0.7922.76` and Edge
  `152.0.4191.53` both completed one and both reached HTTP/2.
- ⭐ **The priority block is measured**, on two browsers, on every terminated
  connection: `80000000ff` on the wire, which is exclusive, dependency 0,
  weight 255. It agrees exactly with the reading this project inherited, and
  [`docs/inherited-claims.md`](docs/inherited-claims.md) section 5 now carries
  both with the conditions of the measurement.
- **`HARNESS-02` closed.** All nine switches are implemented; `--ca-out` was
  the ninth and it had been absent rather than inert for a day.

⚠ **The capture record moved to `harness-capture/3`**: a terminated connection
records what the handshake negotiated, and a field added without a version bump
is a positional format that mis-reads silently.

⛔ **The authority was not installed into the machine trust store.** Each
browser was launched with a per-launch flag naming that one key, which is a
condition of every capture taken this way and is recorded as one. `HARNESS-10`
is the entry that measures whether it changed the answer.

⚠ **A test that asserted a refusal became a hang** when the refusal stopped
refusing, and killing it left a test binary locked. Every test that drives the
command now passes a deadline.
### 2026-09-01T04:33:47Z - the TLS terminator is vendored, and the record that keeps it honest

**Record:** [`TODO/vendor.md`](TODO/vendor.md), `VENDOR-01`, and
[`TODO/PROGRESS.md`](TODO/PROGRESS.md).
**Deployed:** no. Nothing is published from this repository yet, and no capture
has been taken.

What landed:

- ⭐ **rustls, vendored at a named commit and compiled by this tree.** 210 files
  under [`vendor/rustls/`](vendor/rustls/), 2.8 MiB, with fifteen of upstream's
  seventeen workspace members excluded and every exclusion carrying its reason
  in [`vendor/upstream.json`](vendor/upstream.json).
- **The four artefacts the vendoring practice asks for**: the manifest, the
  change record in [`patches/README.md`](patches/README.md), a derived series
  regenerated from the tree, and a scan with an offline leg in the gate and a
  network leg outside it.
- **Three tools**: `scripts/common/check-vendor.sh` with its PowerShell twin, and
  the two node helpers that fetch a pristine copy and regenerate the series.

⚠ **Five checks failed the moment the tree landed**, which is the cost
[`docs/methodology/vendoring.md`](docs/methodology/vendoring.md) names in
advance. Each now exempts the vendored trees and the series derived from them,
never the manifest beside them, and the secret scan carries the reading of all
38 hits that was done before its exemption was taken.

⭐ **The exemption boundary paid for itself in the same session**: the secret
scan refused the patch record, because a pasted cargo failure carried the
operator's home directory into a public repository.
### 2026-09-01T02:55:37Z - the half of the fingerprint above TLS

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and `HARNESS-03`,
`HARNESS-04`, `HARNESS-06`, `HARNESS-07`, `HARNESS-08`, `SCHEMA-06`.
**Deployed:** no. Nothing is published from this repository yet, and no capture
has been taken.

What landed:

- ⭐ **An HTTP/2 reader**, reached WITHOUT terminating TLS: a client with prior
  knowledge opens a cleartext connection and every frame that carries the
  fingerprint arrives before the first response. SETTINGS in arrival order, the
  WINDOW_UPDATE increment, and the priority block read as bytes rather than as
  a rendered string.
- ⭐ **An HPACK decoder**, checked against a fetched corpus of 47,142 cases
  across 446 files rather than against itself, so header order is readable.
- **The emitter's first content**, in `b-ids-emit`: a model that refuses what it
  cannot put on the wire byte for byte, beside a parser's model that keeps it.
  `HARNESS-06` has the difference in one table.
- **The connection selection rule and the sampling rule**, each with a
  committed fixture, and `--until-h2` and `--run-timeout-ms` behind them.
- ⭐ **A raw block a profile can be rebuilt from**, asserted rather than
  intended, and refused when its bytes spell out a credential the parsed fields
  dropped.

⚠ **The capture record moved to `harness-capture/2`**: the HTTP/2 half is a
new sibling of the TLS one, and the cleartext surface stopped being named for
HTTP/1.1 alone, because the peer picks the protocol.

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
