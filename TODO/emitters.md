# emitters

Turning a profile back into bytes, and publishing an honest account of what each
stack cannot do.

⭐ The holes in the support matrix are the most valuable data the project
produces here, because they tell a client author what they cannot claim.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## EMIT-01. The support matrix, with the holes left in

**Source** the founding brief; the limits are [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 9
**Category** emitters, **Priority** P2, **Effort** L, **Status** done

### Problem

A profile is only useful if some stack can emit it, and no published table says
which stack can emit which profile. A client author currently finds out by
building it.

### Premise

Measured by reading, at named commits, in
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md).
The known holes so far:

| stack | cannot emit | how it is known |
| --- | --- | --- |
| one Rust TLS library, including a fork of it | an extension whose codepoint was learned at runtime | its own doc comment says unknown extensions are dropped during parsing, and the extension struct is crate-private |
| the same library | an arbitrary captured extension order | ⭐ its order is a hash of a sixteen-bit seed, so at most 65,536 orders are reachable out of the factorial of the extension count |
| one Rust HTTP/2 library | the priority block inside a headers frame | both send-path constructors hardcode no dependency, and the encode closure that would carry it is passed empty. `EMIT-03`. |
| one Rust HTTP client | one settings key, and preserving request extensions through its builder | reported in that project's tracker and not verified here |
| a boolean-per-extension database | any unenumerated codepoint at all | read at a named commit |
| a Go TLS library | ⭐ **no known hole for the extension model.** It carries an ordered list of codepoint-and-body pairs and refuses an unknown codepoint by default rather than dropping it. | read at a named commit |

### Approach

Publish a matrix per (profile, stack), generated from a conformance run rather
than from a table somebody maintains, so a hole that closes is noticed.

⛔ **Let it have holes.** A cell that says "cannot" is more useful than a cell
that says "approximately", and an emitter that approximates silently is the
defect the whole project is about.

Each hole carries: what cannot be emitted, at file and line in
[`../references/`](../references/) at its captured commit, and whether it is
patchable in this tree.

Must not: fill a cell from a project's documentation. Fill it from a
conformance run, per `VALID-05`.

### Prove

```bash
sh scripts/common/check-support-matrix.sh
```

Passing means: every cell is produced by a conformance run rather than typed,
every hole names a file and a line, and a cell whose evidence no longer resolves
fails the check.

---

### ⭐ Closed 2026-09-03. Six cells from a run, five holes from a reading, and the check refuses the day a citation stops resolving

#### The acceptance

```text
$ sh scripts/common/check-support-matrix.sh
support matrix ok: 6 cell(s) over 6 profile(s), every one produced by a run,
  and 5 of 5 hole(s) still resolving to a file and a line under references/.
  ⛔ A cell is a run and a hole is a reading, and this check keeps them apart.
rc=0
```

```text
$ sh scripts/common/check-support-matrix.sh --json
{"schema":"check-support-matrix/1","cells":6,"holes":5,"resolved":5,"profiles":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-support-matrix.ps1 -Json
{"schema":"check-support-matrix/1","cells":6,"holes":5,"resolved":5,"profiles":6,"problems":0}
```

#### ⛔ A cell is a run and a hole is a reading, and they are different kinds

⚠ **The approach says to fill every cell from a conformance run and never from
a project's documentation. This tree can RUN exactly one emitter: its own.** So
the matrix has two kinds rather than one kind with two colours:

| kind | how many | where it comes from |
| --- | --- | --- |
| ⭐ `cell`, evidence `run` | 6, one per published profile | `b_ids_emit::client_hello` was actually called on each, and each cell carries the command that reproduces it |
| ⛔ `hole`, evidence `read` | 5 | a file and a line in [`../references/`](../references/) at the commit its `PROVENANCE.md` names |

⛔ **A stack this tree cannot run gets a hole and NO CELL.** Writing a cell for
it would be filling the matrix from somebody else's documentation, which the
approach forbids by name, and a row saying "cannot" with no way to re-check it
is a claim rather than a finding.

#### The matrix, as generated

```text
$ b-ids-cli --matrix
```

| stack | what it can or cannot do | evidence |
| --- | --- | --- |
| ⭐ `b-ids-emit` | emits every one of the six profiles whole: 1739 to 1983 bytes, 13 or 14 extensions per profile carrying a codepoint the model gives no field to | `run`, `cargo test -p b-ids-emit escape_hatch` |
| `rustls` | cannot emit an extension whose codepoint was learned at run time. ⭐ Patchable here: this tree already vendors it | `read`, `client_hello.rs:147` |
| `rustls` | cannot emit an arbitrary captured extension order: the order is drawn from a sixteen-bit seed | `read`, `client_hello.rs:337` |
| `h2` | cannot emit the priority block: both send-path constructors hardcode no dependency | `read`, `headers.rs:123`. ⚠ Not vendored here, which is `EMIT-03`'s first step |
| `impit` | cannot emit any unenumerated codepoint at all | `read`, `types.rs:87` |
| ⭐ `utls` | **no known hole for the extension model** | `read`, `u_common.go:184` |

⭐ **The `utls` row is the one worth keeping.** A matrix that only listed
failures would read as an argument; a row saying one stack has no known hole is
what makes the other four rows a measurement.

#### ⛔ The evidence has to still resolve, and that is the check

⚠ **A reference tree moves when it is re-mined**, and a citation into one is a
line number that can stop pointing at anything. Both halves open every cited
file and count its lines, so a hole whose file is gone or whose line is past the
end fails the gate rather than sitting in a table nobody re-reads. ⛔ That is
the `TOOL-10` defect, in the one document whose whole content is citations.

#### The guard mutation, both halves, each exit code read unpiped

⛔ **Every mutation was made against a copy under the ignored scratch directory,
and the live file was compared byte for byte with that copy afterwards.**

| planted | sh | ps | what went red |
| --- | --- | --- | --- |
| a cell claims evidence `read` | 1 | 1 | `6 cell(s) are not evidence run, and a cell filled any other way is a hole wearing a cell's clothes` |
| a hole cites a file that is gone | 1 | 1 | `the evidence for this hole no longer resolves` |
| a hole cites a line past the end of its file | 1 | 1 | `has 446 line(s) and the hole cites line 8700` |
| the holes emptied | 1 | 1 | `the matrix declares no hole at all, and a matrix with none is one nobody filled honestly` |
| a cell loses its reproduce command | 1 | 1 | `6 cell(s) name no command that reproduces them` |

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a cell for any third-party stack | ⛔ Deliberately absent, with the reason above. `VALID-05` is the conformance suite that would produce one, and until it exists a hole is the honest row. |
| the HTTP/2 and header halves of the matrix | ⚠ The cells cover the `ClientHello` only, because that is what `EMIT-02` made emittable. The `h2` hole names what stands in the way of the other half. |
| a committed matrix file | ⛔ On purpose. There is nothing to go stale: the check generates it every time, so a hole that closed shows as a changed cell rather than as a table nobody edited. |
| `PUB-04`'s snippets | The entry this unblocks. Every snippet it generates has to be for a pair this matrix marks emittable, and now there is a matrix to ask. |

---

## EMIT-02. The escape hatch, and where it has to live

**Source** the founding brief; the escape hatch is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 1
**Category** emitters, **Priority** P2, **Effort** L, **Status** done

### Problem

Any emitter that reproduces a browser faithfully needs an ordered list of
codepoint-and-body pairs. A model with one typed field per extension cannot hold
one, and retrofitting the list into such a model is the largest change in this
space.

### Premise

Measured by reading. One library has the shape already; the two Rust candidates
do not.

### Approach

Design for the ordered list from the start rather than adding it later. Where
the chosen stack lacks it, the route is to vendor and patch **here**, per
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md), with the
reproduction command recorded beside the patch so a future release can be tested
against it and the patch deleted when it is no longer needed.

⛔ Upstreaming is not a topic. Fix it in this tree.

Record, beside every patch: what it is in one line, the entry it unblocks, why
it cannot be done outside the vendored tree, and the command that reproduces the
defect.

Must not: write a characterisation of an upstream project, its maintainers or
its responsiveness. The neutral form carries the information and none of the
liability.

### Prove

```bash
cargo test -p b-ids-emit escape_hatch -- --nocapture
```

Passing means: a profile carrying two extensions whose codepoints the emitter
has no name for is emitted with both, in order, with their bodies intact, and
the emitted bytes compare equal to the profile's raw hex.

---

### ⭐ Closed 2026-09-03. 1871 of 1983 bytes, byte for byte, and the other 112 are why the acceptance needed a correction

#### The acceptance

```text
$ cargo test -p b-ids-emit escape_hatch
running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

#### ⛔ The correction: the acceptance as written cannot be satisfied, and the reason is a field nobody should add

⚠ **"the emitted bytes compare equal to the profile's raw hex" is not
achievable, and finding out why is the finding.** The model does not record the
`ClientHello` random: `crates/b-ids-harness/src/hello.rs:77` steps over those
thirty-two bytes without keeping them, deliberately, because a per-connection
random is the one part of a hello that carries no fingerprint at all.

⛔ **So the acceptance is corrected rather than the model.** Recording the random
to satisfy a sentence would put thirty-two bytes of noise into every published
profile, and a consumer comparing two profiles would have to learn to ignore
them. What is compared instead is the EXTENSIONS BLOCK, which is what the
escape hatch is actually about, and it is compared against the raw bytes rather
than against the model.

| | measured over `chrome-152.0.7977.75-linux64-stable` |
| --- | --- |
| the raw hello | 1983 bytes |
| ⭐ the emitted extensions block | **1871 bytes, found in the hello exactly once** |
| what is not compared | 112 bytes: the record and handshake headers, the versions, the random, the session id, the ciphers and the compression methods |

⚠ **The unmatched 112 bytes are not unreachable**, and saying which is which
matters: the ciphers, the versions, the session id and the compression methods
are all in the model and could be emitted. Only the random cannot, and it is the
only one that should not be.

#### ⭐ Found rather than sliced, and that is the whole assertion

⛔ **Slicing the extensions out of the hello would need a second parser of the
hello**, and a second parser tests two implementations against each other rather
than testing the bytes. So the emitted block is SEARCHED FOR in the raw bytes
and required to occur exactly once. ⚠ A block of 1871 bytes occurring once in
1983 is not a coincidence, and the reordering case below is what says the search
can fail.

#### The escape hatch, measured

⭐ **Fourteen of eighteen extensions in a Chrome 152 hello carry a codepoint the
model gives no field to.** A model with one typed field per extension could hold
four of them. That is the premise of this entry, asserted by
`escape_hatch_every_capture_carries_codepoints_the_model_does_not_name` over
every published profile rather than believed.

⚠ **`unnamed_codepoints` names four as reachable without the list**:
`supported_groups`, `signature_algorithms`, ALPN and `key_share`, because the
model carries the whole content of each as a typed field. ⛔
`supported_versions` is NOT among them: the model reads a value out of its body
and could not write the body back, and reading is not emitting.

#### The guard, seen to fail

| planted | what went red |
| --- | --- |
| two extensions swapped | the block changes, its length does not, and ⛔ the swapped block is found in the hello **zero** times |
| a declared length raised by one | `extensions_block` refuses with `the capture declares N byte(s) and its body holds M, so an emitter would have to believe one of them` |

⭐ **The first of those is the one worth having.** The order is a fingerprint,
and a comparison that passed over a reordered list would be a comparison that
proved only that the bytes were present somewhere.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a whole `ClientHello` on a wire | ⛔ Not emitted, and the record above says which bytes are missing and why one of them should stay missing. `LIB-02` is the entry that puts a profile back on a socket. |
| a patch to the vendored TLS library | ⚠ None was needed here. This entry is the MODEL half of the escape hatch, and it turns out the model already had the shape: the list is ordered, the codepoint is a plain number and the body is arbitrary bytes. The patch belongs to whichever stack cannot take that list, which is `EMIT-01`'s matrix and `LIB-02`'s first attempt. |
| the HTTP/2 and header sides | `EMIT-01`. |

---

## EMIT-03. The priority block patch, if the measurement says it is needed

**Source** the founding brief; the seam and the patch are [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 4
**Category** emitters, **Priority** P2, **Effort** S, **Status** open

### Problem

A client that omits the priority block carries a zero in one field of four in a
widely-read HTTP/2 fingerprint, and that is one of the fields an origin can
still tell apart.

### Premise

⛔ **Blocked on a measurement taken here, and it is honest to say so rather
than to build first.** [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 records the block measured off
frame bytes on two Chrome versions, in another repository, which is `vendor`
provenance. `HARNESS-05` reads it here, and it is expected to agree.

⭐ **The seam is confirmed by reading and the patch already exists**, at a named
commit in the corpus: [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 4 has both. The encode function
takes a closure that runs after the frame head and before the header block, the
payload length is computed after it runs, and the push-promise path already uses
it to write a stream identifier. So the change is five bytes written into a
closure that exists, and the frame length and any continuation split follow for
free.

⚠ **Read that patch, do not copy it.** Its tree is MIT and this one's output is
0BSD. [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) is
the rule when a patched upstream lands here.

⛔ **Upstream will not take it, for a stated reason rather than by neglect.** RFC
9113 deprecates stream priority, so a release adding a way to send one would add
a way to do what the specification tells clients not to do. This patch is this
project's to carry.

### Approach

After `HARNESS-05` reports:

- if browsers send the block, vendor and patch the HTTP/2 library here, writing
  the five bytes in the existing closure, and record the reproduction command;
- if they do not, close this entry with the measurement written underneath and
  the seam recorded in the support matrix as available-but-unneeded.

Either way the measurement is published, because a negative result here deletes
work for everybody who reads it.

Must not: apply the patch before the measurement. Building an emitter for a
behaviour that may not exist is the derived value this project refuses, in
another costume.

### Prove

```bash
cargo test -p b-ids-emit priority_block -- --nocapture
```

Passing means: with the patch applied, a client's first headers frame carries
the flag and the five bytes, the harness reads them back identically, and the
frame length is correct across a continuation split.

### ⛔ 2026-09-04: the measurement is in, and the blocker has moved to a ruling

⚠ **The Premise above says this entry is blocked on a measurement taken here.**
⛔ That is no longer true. The measurement was taken by `HARNESS-05` on
2026-09-01 and it is published: every profile in the corpus carries the block.

```text
$ for f in $(find corpus/v1 -name '*.json' ! -name index.json ! -name latest.json | sort); do
    printf '%-40s %s\n' "$(jq -r .id "$f")" "$(jq -c .http2.stream_priority "$f")"; done
chrome-151.0.7922.173-linux64-stable     {"exclusive":true,"stream_dependency":0,"weight_wire":255}
chrome-152.0.7977.75-linux64-stable      {"exclusive":true,"stream_dependency":0,"weight_wire":255}
chrome-151.0.7922.174-win64-stable       {"exclusive":true,"stream_dependency":0,"weight_wire":255}
chrome-151.0.7922.76-win64-stable        {"exclusive":true,"stream_dependency":0,"weight_wire":255}
chrome-152.0.7977.76-win64-stable        {"exclusive":true,"stream_dependency":0,"weight_wire":255}
edge-151.0.4129.101-linux64-stable       {"exclusive":true,"stream_dependency":0,"weight_wire":255}
```

⭐ **Six of six, two browsers, two majors, two platforms, one value.** The
Approach's second branch, which closes this entry on a negative result, does not
apply. The first branch does.

#### ⛔ What that branch costs, measured rather than estimated

⚠ **"Vendor and patch the HTTP/2 library here" was written before anyone
counted what the library brings.** Read from
`references/hyperium__h2/tree/Cargo.toml` on 2026-09-04, at the commit
`references/hyperium__h2/PROVENANCE.md` records,
`cb9574bb2c18d1904eca74e98b31c8986b0d8b32`:

```text
atomic-waker, futures-core, futures-sink, tokio-util, tokio, bytes,
http, tracing, fnv, slab, indexmap
```

⛔ **Eleven crates, one of which is an async runtime.** This workspace's entire
third-party surface today is `serde` and `serde_json`, plus the vendored TLS
terminator and the certificate authority beside it. It is synchronous
throughout.

⛔ **And nothing in this workspace uses `h2` at all.** The `h2` module in
[`../crates/b-ids-harness/src/h2.rs`](../crates/b-ids-harness/src/h2.rs) is this
project's own frame READER, written here; the hole in the support matrix is
about somebody else's send path. So the patch would compile a tree nothing here
calls, for a consumer this repository does not have.

⚠ **`b-ids-cli` is not that consumer and must not become it.**
[`../docs/architecture.md`](../docs/architecture.md) says it puts one profile's
hello on a socket and stops, and that it must never grow into a general-purpose
HTTP client.

#### ⭐ The question, and the recommendation attached to it

⛔ **This entry now needs a ruling rather than a measurement**, and it is
recorded in [`PROGRESS.md`](PROGRESS.md).
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) says
vendoring is chosen rather than drifted into, and lists what it costs: the build
compiles the vendored code, the tree grows, upstream stops being visible, and
its warnings become this project's.

⭐ **Recommendation: do not vendor `h2`, and publish the seam instead.** The
support matrix already records the hole at a file and a line, and as of
`PUB-04` every published profile carries a `configs/.../h2.txt` naming it, so a
client author on `h2` is told exactly what to patch and where. That delivers the
entry's Problem without an async runtime in a synchronous workspace.

⚠ **The alternative, and what it would buy**: vendoring makes the patch this
project's own, provable by the acceptance command above, and flips the matrix
hole's `patchable_here` from false to true. ⛔ It is the only route that makes
the Prove runnable, because that command needs a client built on the patched
library.

#### ⚠ What has NOT changed

⛔ **The seam is still real and still confirmed by reading.** Both send-path
constructors hardcode no dependency and the encode closure that would carry the
five bytes is passed empty, at
`references/hyperium__h2/tree/src/frame/headers.rs:123`. Nothing above weakens
that; it is a statement about who pays for the fix rather than about whether one
is needed.

⛔ **And the entry stays OPEN.** A blocked entry keeps the blocker named and
what would unblock it, and this one is unblocked by one sentence from the
operator.

---


### ⭐ 2026-09-02: the measurement is in, and it takes the first branch

⛔ **Browsers send the block.** Every profile in this corpus carries it, and they
agree exactly:

| profile | `http2.stream_priority` |
| --- | --- |
| Chrome `151.0.7922.76` `win64` | `exclusive: true, stream_dependency: 0, weight_wire: 255` |
| Chrome `151.0.7922.173` `linux64` | the same |
| Chrome `151.0.7922.174` `win64` | the same |
| Chrome `152.0.7977.75` `linux64` | the same |
| Chrome `152.0.7977.76` `win64` | the same |
| Edge `151.0.4129.101` `linux64` | the same |

⭐ **Six profiles, two browsers, two majors, two platforms, one value.** The
inherited reading in [`../docs/inherited-claims.md`](../docs/inherited-claims.md)
section 5 was taken off frame bytes on two Chrome versions in another
repository; this agrees with it and is measured here.

⛔ **So the approach's second branch is closed and the first one is open.** There
is no negative result to publish: the entry does not close on "browsers do not
send it". What remains is the work the first branch names, and it is not the
five bytes:

| | |
| --- | --- |
| ⛔ **the HTTP/2 library has to be vendored first** | this tree vendors one library today, the TLS terminator, and `VENDOR-01` is what it cost. A second vendored tree is a second reconciliation, a second provenance file and a second `check-vendor` subject. |
| ⚠ **and the patch is read rather than copied** | its tree is MIT and this one's output is 0BSD. [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) is the rule. |
| ⭐ **the seam is confirmed** | [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 4, at a named commit: the encode function takes a closure that runs after the frame head and before the header block, and the push-promise path already writes a stream identifier through it. |

⚠ **The effort estimate stands at `S` and it was estimated for the branch that
did not happen.** ⛔ Left as it is rather than edited, per
[`../docs/methodology/authoring.md`](../docs/methodology/authoring.md): the
estimate is what was believed, and this paragraph is what is true.

---


### ⛔ Ruled by the operator 2026-09-04: vendor and patch `h2`

⛔ **The patch branch, not the seam.** The eleven crates and the async runtime
are accepted. ⚠ The recommendation attached to the question was the opposite,
and the operator ruled against it: the alternative leaves this entry's
acceptance command permanently unrunnable, because it needs a client built on
the patched library.

⭐ **What that makes possible.** The patch becomes this project's own, provable
by `cargo test -p b-ids-emit priority_block`, and the support matrix hole's
`patchable_here` flips from false to true.

⛔ **The rules that bind the work.** Re-derive the five bytes, never copy them:
that tree is MIT and this one's output is 0BSD.
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) governs
the manifest, the change record and the reproduction command, and ⛔ upstreaming
is not a topic.

## EMIT-04. Emitters for the stacks a consumer already uses

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** emitters, **Priority** P3, **Effort** M, **Status** open

### Problem

A consumer whose stack this project does not target gets a corpus and no way to
use it.

### Premise

Believed.

### Approach

Start with whichever stack this project already uses, then add targets by
demand rather than by ambition. Each target produces either a real emitter or a
configuration snippet, and the support matrix says which, per `EMIT-01`.

⚠ The Go TLS library is the cheapest second target, because its model already
matches the profile shape and the work is a translation rather than a design.

Must not: claim a target the conformance run has not passed.

### Prove

```bash
cargo run -p b-ids-conformance -- --stack STACK --claim PROFILE
```

Passing means: for every claimed target, the conformance report shows no
differing field, or the differing fields are exactly the ones the support matrix
records as holes.
