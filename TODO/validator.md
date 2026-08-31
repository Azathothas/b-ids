# validator

Given a profile, answer one question: could a real browser have sent this?
Pure logic over the model. No network, no browser.

⭐ This is the highest-value component in the project and the one nobody has
built, and it can be finished before a single capture exists.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## VALID-01. The coherence checks, as a library and a command and a schema

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P0, **Effort** M, **Status** open

### Problem

Nothing refuses a combination that exists nowhere. A client announcing one
version over another version's handshake, with an invented brand list, is
accepted by every tool examined, and it is the most distinguishing thing such a
client can do.

### Premise

Believed. The checks below are proposed, not implemented, and three of them have
real shipped violations waiting for them, which is what makes them provable
rather than assertable.

### Approach

Ship it three ways, because the three have different consumers and only the
first two can express the checks that need logic: a **library**, a **command**,
and a **JSON Schema**.

The checks, roughly in order of how often each should catch something:

1. **Version coherence.** The major in the brand list, the major in the
   User-Agent, and the recorded major all agree.
2. **Platform coherence.** The platform hint, the User-Agent's operating-system
   token and the recorded platform agree, and the mobile hint agrees with the
   platform.
3. **Brand coherence.** A branded profile has a vendor entry in its brand list;
   an unbranded one does not, and says so in `browser.branded`.
4. **Handshake and version coherence.** The `ClientHello` came from a build
   whose major matches the claimed one. ⭐ This is the check that catches the
   worst failure mode in the field: new User-Agent, old hello.
5. **GREASE coherence.** If the browser shuffles, the profile says so. If GREASE
   is at both ends, the two values are distinct and the bodies match the
   recorded shape.
6. **Encoding honesty.** Every content-encoding token a profile advertises is
   one the **consuming client** can actually decode. A client advertising an
   encoding it cannot decode hands compressed bytes to a parser. The corpus
   enables this check by declaring what each profile needs.
7. **Absence checks.** A setting a browser does not send is as load-bearing as
   one it does. A profile that omits a setting must be emitted by a stack that
   can omit it, and most cannot.
8. **Provenance completeness.** No `vendor` field in a published profile, and no
   `substituted` or `unreproducible` field without a reason.

Each check is a separate function with its own test, and each test fails without
its fix. A check whose test has never been seen to fail is theatre.

Must not: warn where it should refuse. A validator that only warns is a
validator whose output nobody reads.

### Prove

```bash
cargo test -p b-ids-validator -- --nocapture
```

Passing means: eight checks, eight tests, each planting the exact contradiction
its check exists to catch and asserting a non-zero exit with a message naming
the field.

---

## VALID-02. Run it over the prior art, and publish what it finds

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P1, **Effort** S, **Status** open

### Problem

The validator has no evidence that it works on anything but its own fixtures,
and the project has no publishable result at all until a capture exists.

### Premise

⭐ **Measured by reading, and the violations are already located.**
[`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md)
section 7 names three, at file and line, in
[`../references/`](../references/):

- five entries of one reference database return another version's TLS and
  HTTP/2 wholesale beside their own User-Agent and brand list, which is check 4;
- one library serves a cipher table commented with a version from years earlier
  to any version's User-Agent, which is also check 4;
- that library's family classifier can return three families while its data
  carries four, so one family is unreachable, which is `VALID-03`.

### Approach

Write an importer per reference database that produces profiles from their
tables, run the validator over the result, and publish the report: which entries
fail which check, with the file and line in the reference tree at its captured
commit.

⭐ **That is a publishable result on day one and it costs no capture.**

⛔ The imported profiles are `vendor` provenance and are drafts. They are test
input for the validator, and none of them enters the corpus.

⚠ Write it as a technical report, not as a complaint. Name the defect, name the
line, and stop. A characterisation of a project or its maintainers is forbidden
by [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) and
would be read by the person it is about.

Must not: publish a claim without opening the file at the captured commit. A
tracker's description of a defect is evidence of what somebody believed.

### Prove

```bash
cargo run -p b-ids-validator -- import references --report
```

Passing means: the report names at least the three violations above, each with
its file, its line and the check it failed, and re-running against the same
commits produces byte-identical output.

---

## VALID-03. A family the resolver cannot produce is data nobody can reach

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 7
**Category** validator, **Priority** P2, **Effort** S, **Status** open

### Problem

A corpus can carry data for a browser family that no code path can select. It
sits there looking authoritative, and a reader who finds it believes it is used.

### Premise

Measured by reading, at a named commit: one library's classifier returns three
families, and its header-order table carries a fourth key that nothing can
reach. Grep finds that key in exactly one file, as a key, and nowhere else.

### Approach

A check that walks every dimension the corpus carries, browser, channel and
platform, and asserts that the resolver can produce each one. A dimension with
data and no route is a failure, not a warning.

⭐ **Nobody writes this check**, which is why the defect survives in a project
that is otherwise carefully written.

Must not: fix it by deleting the data. The data may be right and the resolver
wrong, and the check's job is to say they disagree rather than to pick.

### Prove

```bash
cargo test -p b-ids-validator reachable_dimensions -- --nocapture
```

Passing means: a fixture corpus carrying a family the resolver has no branch for
fails with a message naming the family and both files.

---

## VALID-04. Reference digest implementations, with published test vectors

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P2, **Effort** M, **Status** open

### Problem

Digest implementations are currently checked against one another's behaviour
rather than against a shared set of inputs with known answers. Two
implementations that agree may be agreeing on the same defect.

### Premise

Believed. ⚠ And one specification detail matters here and was got wrong once
already: the order-preserving raw form of the modern digest also strips GREASE,
so a vector set that expects GREASE in it is wrong.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 10 carries
the correction.

### Approach

Publish raw `ClientHello` bytes plus expected values for every digest this
project computes, as data rather than as code, so an implementation in any
language can be checked against them.

⚠ **A licence question comes first and is not settled.** The modern digest's own
implementation carries an attribution licence and its extended family carries a
restrictive one.
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
finding 5 has the split. Implement from the published specification, never by
copying source, and do not emit any member of the extended family until the
question has an answer written down.

Must not: ship a vector whose expected value came from running one
implementation. A vector's expected value is derived from the specification and
then checked against implementations, not the reverse.

### Prove

```bash
cargo test -p b-ids-validator digest_vectors -- --nocapture
```

Passing means: every published vector's expected value is reproduced by this
project's implementation, the count of vectors is asserted, and a deliberately
corrupted vector fails.

---

## VALID-05. A conformance suite for impersonating clients

**Source** the founding brief; the shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 10
**Category** validator, **Priority** P2, **Effort** L, **Status** open

### Problem

A client author who wants to know how close their client is to a real browser
has to build a capture server and a comparison by hand. Every client examined in
the sweep would use one if it existed.

### Premise

⭐ Believed, and one reference project already has half of it: per-target
expected-signature files asserted by its own test suite. What it lacks is that
the expectation is its own recorded output rather than an independent corpus.

### Approach

Point a client at the harness, capture it, compare against the profile it claims
to be, and report a **field-level diff**: not "the digest differs" but which
extension moved, which setting is absent, and which header changed position.

⭐ The same artefact serves both directions, which is the argument for building
it: it is a conformance report for a client author and a detection reference for
a server author, and neither has one today.

Must not: report only a digest comparison. That is the tool everybody already
has and it says two things differ without saying what.

### Prove

```bash
cargo run -p b-ids-conformance -- --claim chrome-152.0.7977.64-linux64-stable
```

Passing means: run against a client that deliberately differs in one field, the
report names that field and nothing else, and exits non-zero.

---

## VALID-06. Diffs between adjacent versions

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P2, **Effort** S, **Status** open

### Problem

"What changed between these two versions" is the single most useful artefact for
anybody maintaining a client, and it is free once two profiles exist.

### Premise

Believed, and there is already one worked example to test against: a header
moved position between two versions in the inherited captures, and it is the
kind of change only a capture finds.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 6.

### Approach

A field-level diff between any two profiles, rendered so a reader can act on it:
"this header moved from position twelve to position five, and this extension
appeared with a body of this length", not "the digest changed".

Generate it for every adjacent pair in the corpus, publish it as a format per
`SCHEMA-08`, and reuse the same code in the automated pull request body per
`CI-04`, so the two cannot disagree.

Must not: render a diff of two profiles captured under different conditions
without saying so. Two captures that differ in version **and** platform **and**
request kind cannot isolate anything.

### Prove

```bash
cargo run -p b-ids-validator -- diff PROFILE_A PROFILE_B
```

Passing means: the diff of two profiles differing in exactly one header position
names that header and its two positions, and reports no other change.
