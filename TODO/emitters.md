# emitters

Turning a profile back into bytes, and publishing an honest account of what each
stack cannot do.

⭐ The holes in the support matrix are the most valuable data the project
produces here, because they tell a client author what they cannot claim.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## EMIT-01. The support matrix, with the holes left in

**Source** the founding brief; the limits are [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 9
**Category** emitters, **Priority** P2, **Effort** L, **Status** open

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

## EMIT-02. The escape hatch, and where it has to live

**Source** the founding brief; the escape hatch is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 1
**Category** emitters, **Priority** P2, **Effort** L, **Status** open

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
