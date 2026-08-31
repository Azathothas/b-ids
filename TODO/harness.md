# harness

The capture oracle. A listener a browser is pointed at, the parsers that read
what arrives, and the traps that cost somebody a day each.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## HARNESS-01. The oracle is a server, not a client

**Source** the founding brief; the harness shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 14
**Category** harness, **Priority** P0, **Effort** L, **Status** open

### Problem

There is no way to read a fingerprint in this repository. Asking a client what
it sent returns what it intended, which is a different thing and is the commonest
way a whole set of numbers turns out to describe nothing.

### Premise

Believed, and it is the design decision everything else rests on: the only
honest reading of a handshake is off the wire, from outside the client, so the
harness is a listener rather than a probe inside a browser.

### Approach

A TLS listener that accepts a connection, reads the record, parses the
`ClientHello`, and writes one JSON object per connection to standard output
after one line carrying the base URL a caller should point a browser at.

⭐ **Design it multi-protocol from the first commit even while implementing
one.** A TLS listener, an HTTP/1.1 listener, an HTTP/2 listener and later a QUIC
listener are four capture surfaces, and the model in `SCHEMA-01` already has
room for all four. Retrofitting the fourth into a TLS-shaped harness is a
rewrite rather than an addition.

The record-everything rule from `SCHEMA-06` applies from the first commit, not
from a later pass. Retrofitting completeness means re-capturing, and by then the
build is gone.

Must not: complete a handshake in order to read one, by default. Completing it
can change what the client offers, which is `HARNESS-11`.

### Prove

```bash
cargo test -p b-ids-harness listener -- --nocapture
```

Passing means: a committed fixture of raw bytes is fed to the listener over a
loopback socket and produces the profile committed beside it, byte for byte, with
no browser involved.

---

## HARNESS-02. The switches, each of which exists because something went wrong without it

**Source** the founding brief; the harness shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 14
**Category** harness, **Priority** P0, **Effort** M, **Status** open

### Problem

A harness with one mode can capture one thing. Every switch below is a capture
that is otherwise impossible or a value that is otherwise wrong.

### Premise

Inherited from a working implementation in another repository, described but
never read here.

### Approach

| switch | why it exists |
| --- | --- |
| `--raw` | do not terminate TLS. Completing a handshake can change what the client offers, so a digest read through a terminated handshake is not the digest it ships. |
| `--plain` | cleartext HTTP/1.1, header order only. The capture that works when a client cannot be told to trust anything. |
| `--ca-out PATH` | mint a certificate authority per run and write it, so a client can complete a **verified** handshake. Nothing has to disable verification to reach the HTTP/2 half. |
| `--bind ADDR` | reach a browser that is not on this machine. Refuses a hostname, and refuses the unspecified address by name. |
| `--hello-out PATH` | the raw `ClientHello` as hex. The one artefact that survives every hashing scheme and every parser defect. |
| `--header-values` | record values, not only names. Gated because it is the one switch that can log a credential, per `SCHEMA-04`. |
| `--until-h2` | stop at the first connection that reached HTTP/2, per `HARNESS-07`. |
| `--json` | one object per connection on standard output, after one line carrying the base URL |
| `--handshakes N` | how many connections to accept before exiting, per `HARNESS-08` |

Must not: make `--header-values` the default, or let `--bind` accept the
unspecified address without saying so, since that accepts the local network as
well.

### Prove

```bash
cargo test -p b-ids-harness switches -- --nocapture
```

Passing means: every switch is exercised, `--bind 0.0.0.0` is refused with a
message naming the reason, and the default run's output contains no header value.

---

## HARNESS-03. Read HTTP/2 settings, the window update and the priority block

**Source** the founding brief; the block is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5
**Category** harness, **Priority** P1, **Effort** M, **Status** open

### Problem

Half the fingerprint is above TLS. Without the HTTP/2 read there is no settings
order, no window increment, no pseudo-header order and no priority block.

### Premise

Read rather than measured here. Values and their sources are in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 4.

### Approach

Read and record, in arrival order: the connection preface, every SETTINGS frame
with its entries in order, the connection-level WINDOW_UPDATE increment, any
standalone PRIORITY frames, and the first HEADERS frame including its flags
byte.

Derive the Akamai string from those, and store it in the derived block rather
than in the measured one.

Must not: record only the rendered Akamai string. It cannot distinguish an
absent priority block from an unread one, which is why two published readings of
that field are unusable and `HARNESS-05` reads the frame instead.

### Prove

```bash
cargo test -p b-ids-harness http2 -- --nocapture
```

Passing means: a fixture of frame bytes produces a profile whose settings list
is in arrival order, and a fixture that omits one settings key produces a
profile that records it as absent rather than as its default.

---

## HARNESS-04. Decode HPACK Huffman, because header order is behind it

**Source** the founding brief; the harness shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 14
**Category** harness, **Priority** P1, **Effort** M, **Status** open

### Problem

HTTP/2 header names arrive Huffman-coded. Without a decoder the harness cannot
read the header order, which is a first-class part of the fingerprint.

### Premise

Structural.

### Approach

A decoder for the static Huffman table, plus the dynamic table size updates,
enough to read names and, when the value switch is on, values.

Record **whether each header was Huffman coded**, per `SCHEMA-06`. That is a
choice the encoder made and it is part of the fingerprint.

⚠ **Test vectors exist and are not in this tree.** The reference corpus of HPACK
test cases is a separate upstream project, and it was deliberately deleted from
[`../references/hyperium__h2/`](../references/hyperium__h2/) during the sweep;
that project's `PROVENANCE.md` records where it came from. Fetching it is part
of this entry.

Must not: write a decoder without vectors and call it done. A Huffman decoder
that is subtly wrong produces plausible header names.

### Prove

```bash
cargo test -p b-ids-harness hpack -- --nocapture
```

Passing means: every case in the fetched vector corpus decodes to its expected
output, and the count of cases run is asserted, so a table that stopped early
cannot report green over a smaller suite.

---

## HARNESS-05. Settle the priority block, and do it first

**Source** [`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md) finding 1
**Category** harness, **Priority** P1, **Effort** S, **Status** open

### Problem

Whether a browser opens its first stream with a PRIORITY block inside the
HEADERS frame decides a whole emitter patch, and this project has not read a
byte of it. Three published readings disagree, and until one is taken here no
profile may carry the field.

### Premise

⭐ **Measured elsewhere, off frame bytes, on two versions.** [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 has
the table. The origin repository's probe reads the HEADERS flags byte for
`0x20` and decodes the five bytes behind it, and reports exclusive, dependency
zero, weight 255 for both Chrome 151 and Chrome 152.

⚠ **The two sources reporting zero read a rendered Akamai string**, not frame
bytes, and one of them is a tool that could not write the block at the time it
recorded its own data. A zero in that field is what an unpatched stack emits.

⛔ **That makes this a confirmation with a predicted answer, not an open
question, and it does not make the value publishable.** It was measured
somewhere else, which is `vendor` provenance, and the first rule is why the
capture still has to be taken here.

### Approach

Read the HEADERS frame's flags byte for the `0x20` bit and, if it is set, the
five bytes after the frame head: one exclusive bit, a 31-bit dependency, and one
weight byte. Report the raw five bytes as well as the parsed values.

⭐ **Run a positive control in the same session.** A probe that finds nothing may
have been looking in the wrong place, and only a control separates the two. The
control is any client known to write a block; the patch that makes one is cited
in [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 4.

Run it against at least two browsers and two versions, so a disagreement with
the predicted answer is localisable to a version rather than to the probe.

⭐ **This is the first capture the harness should take**, because it is one
measurement, it is cheap, and it decides whether `EMIT-03` is work at all.

Must not: report the Akamai string as the answer. That is the artefact whose
ambiguity created the disagreement. ⛔ And must not skip the control because the
answer is predicted: a probe that agrees with an expectation it was told is the
one result that proves nothing.

### Prove

```bash
cargo run -p b-ids-harness -- --plain --json --handshakes 8
```

Passing means: the output records, per connection, whether the priority flag was
set and the five raw bytes when it was; the positive control shows the flag set;
and the result is written into [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 with the browser, the build and
the date, beside the inherited reading and saying whether the two agree.

---

## HARNESS-06. Parse permissively, emit exactly

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** harness, **Priority** P1, **Effort** S, **Status** open

### Problem

A parser that maps a GREASE codepoint to a typed field with an empty body
rejects the GREASE extension that carries a byte, which is what a browser sends
at the end of its list. It passes locally, fails once in continuous integration,
and passes the next run.

### Premise

Measured elsewhere and inherited: three of the sixteen GREASE values failed that
way, so about one handshake in five.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 carries
it.

### Approach

Two separate types, deliberately: the parser's model accepts any codepoint with
any body, and the emitter's model refuses anything it cannot reproduce exactly.
A codebase that uses one type for both gets one of them wrong.

Any field on a GREASE codepoint takes an arbitrary body, including a body that
is not empty.

Must not: share one type between the two. That is the whole entry.

### Prove

```bash
cargo test -p b-ids-harness grease_bodies -- --nocapture
```

Passing means: a fixture per GREASE value, each with a one-byte body, all parse;
and the emitter refuses a profile whose extension body it cannot reproduce,
rather than emitting an approximation.

---

## HARNESS-07. A browser opens sockets it abandons, and it resumes

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** harness, **Priority** P1, **Effort** S, **Status** open

### Problem

One navigation produces many connections, and neither the first nor the last is
the one to keep. A harness that takes either is recording something other than a
cold handshake.

### Premise

Measured elsewhere and inherited: driving one browser at a probe produced
thirteen connections from one navigation. The first carried no HTTP/2 at all, a
preconnect the browser abandoned. Every one after the second offered a
pre-shared key instead of a session ticket, because the session resumed, and the
two produce different digests.

### Approach

Keep the **first connection that completed HTTP/2**. That is the cold handshake
and it is what a fresh client sends. Record a resumed connection separately, as
its own profile with its own provenance, and never average the two or assert
they are equal.

`--until-h2` is the switch, per `HARNESS-02`.

Must not: deduplicate connections by digest. Two connections that differ are the
data.

### Prove

```bash
cargo test -p b-ids-harness connection_selection -- --nocapture
```

Passing means: a fixture of a thirteen-connection navigation selects connection
two, and the resumed connections are emitted as a separate labelled set whose
digest differs from the cold one.

---

## HARNESS-08. One handshake is not a sample

**Source** the founding brief; the trap is [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8
**Category** harness, **Priority** P1, **Effort** S, **Status** open

### Problem

Anything drawn per connection means a single handshake tests a single draw. A
defect that fires on three values in sixteen reaches a one-handshake check four
times in five, and passes.

### Premise

Arithmetic over the measured GREASE behaviour in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 2.

### Approach

Make the handshake count a parameter, default it to eight, and assert that every
one of them completed. A run where six of eight completed is a run that reports
six, not a run that reports success.

Report the per-draw variation: which values were drawn, and how many distinct
orders were seen. That is what `SCHEMA-10` records.

Must not: default to one, and must not report a pass when fewer handshakes
completed than were asked for.

### Prove

```bash
cargo test -p b-ids-harness sampling -- --nocapture
```

Passing means: a run configured for eight handshakes where the fixture supplies
six exits non-zero with a message naming both numbers.

---

## HARNESS-09. Fuzz the parsers. A panic here is unacceptable

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** harness, **Priority** P1, **Effort** M, **Status** open

### Problem

The harness reads bytes chosen by whoever connects to it. A parser that panics
on a truncated or hostile record is a parser that can be stopped by anybody who
can reach the listener.

### Premise

Structural.

### Approach

A fuzz target per parser: the record layer, the `ClientHello`, the HTTP/2 frame
reader, and the Huffman decoder. Each must return an error and never panic, on
any input.

Every parser returns an option or a result rather than panicking, including on a
length field that claims more bytes than arrived. Trusting a declared length
instead of counting what arrived is its own defect class.

Must not: treat a fuzz finding as a corpus entry. A crash input is a test case;
it is not a capture.

### Prove

```bash
cargo fuzz run client_hello -- -runs=1000000
```

Passing means: every target runs its budget with no crash and no timeout, and
each corpus seed is committed so the run is reproducible.

---

## HARNESS-10. Check whether measuring changed what was measured

**Source** the founding brief; [`../docs/methodology/experiments.md`](../docs/methodology/experiments.md)
**Category** harness, **Priority** P2, **Effort** S, **Status** open

### Problem

An instrument that has to relax something in order to see anything may have
changed what it is watching. A value captured that way is not the value the
subject ships.

### Premise

Inherited: in a related sweep, disabling certificate verification so a probe
could terminate a connection also changed the client's advertised algorithms.
The answer there was to capture that one field passively instead.

### Approach

For every field the harness records, establish which capture mode produced it,
and compare the raw mode against the terminating mode on the same browser and
build. Where they differ, the field is captured in the mode that does not
perturb it, and the reason is recorded beside the field rather than in a commit
message.

⭐ Ask this of every instrument this project builds. It is not an exotic case.

Must not: publish a field measured in a mode that changes it, without saying so.

### Prove

```bash
sh experiments/20-compare-capture-modes.sh
```

Passing means: the script captures the same browser in raw and terminating modes
and prints a field-level diff, and every differing field is named in the profile
schema's documentation with the mode that owns it.

---

## HARNESS-11. The p0f layer, which is free once the listener is ours

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** harness, **Priority** P2, **Effort** M, **Status** open

### Problem

A whole second fingerprint sits below TLS and nothing in the plan reads it. It
is thrown away by every project examined, and it costs almost nothing when the
listener already belongs to this project.

### Premise

Believed. Not attempted.

### Approach

Record the source port, the maximum segment size, the window size and scale, the
time to live, and the order of the TCP options. That is operating-system-level
fingerprinting and it belongs in the same profile as everything else.

⚠ It needs a raw socket or an equivalent, which is a capability question rather
than a code question, and it may be unavailable on a hosted runner. Establish
that first and record the answer, because it decides whether this is a lane in
the capture matrix or a local-only extra.

Must not: fabricate a value the platform did not give. An unavailable field is
absent, with a reason.

### Prove

```bash
cargo test -p b-ids-harness tcp_layer -- --nocapture
```

Passing means: a capture on a host where the capability exists records all five
fields, and a capture on a host where it does not records them as absent with a
reason rather than as zero.

---

## HARNESS-12. A public capture oracle

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** harness, **Priority** P2, **Effort** L, **Status** open

### Problem

Every existing hosted fingerprint service returns a subset and no raw hello.
Somebody who wants to know what their own browser sends has to trust a page.

### Premise

Believed, and it is a service question as much as a code one.

### Approach

A hosted endpoint anybody can point a browser at, returning the full model from
`SCHEMA-01`, including the raw hello, rather than a hash and a marketing page.
The same harness binary, run as a service.

⚠ **It changes this project's obligations.** A hosted endpoint receives traffic
from people, which is the one thing the scope boundary says this project does
not do. Settle before building: what is logged, for how long, and whether
anything is retained at all. The default that keeps the boundary intact is to
retain nothing and return the result to the caller only.

Must not: retain a capture from a visitor's browser as a corpus entry. The
corpus is captures the harness took of browsers it launched itself.

### Decision

Whether to run this at all.

Recommendation: build the mode, and do not host it until the retention question
has an answer written down and a human has approved it.

### Prove

```bash
cargo run -p b-ids-harness -- --serve --no-retain
```

Passing means: the endpoint returns a full profile to a browser pointed at it,
nothing is written to disk, and a test asserts the no-retain default by checking
that the process created no file.
