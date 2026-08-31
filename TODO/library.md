# library

A crate another project can depend on, and the smallest possible tool that
proves the crate works.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## LIB-01. A crate that hands a program a profile

**Source** the operator; the founding brief
**Category** library, **Priority** P2, **Effort** M, **Status** open

### Problem

A consumer that wants a profile has to fetch a file, parse it, and know its
shape. That is three decisions for what should be a dependency line, and it is
the reason people copy values by hand instead.

### Premise

Believed. Nothing exists.

### Approach

One crate, embedding a pinned corpus release, exposing:

- **selection**: by browser, channel, platform and version, or by the same keys
  with `latest` for the version. The same axes as the published routes in
  `PUB-03`, so a reader who knows one knows the other.
- **the whole profile**, typed, including the raw bytes and the provenance map.
  ⛔ A consumer must be able to ask whether a field was measured, so provenance
  is part of the public shape rather than an internal detail.
- **the parts on their own**: the User-Agent, the client hints, the header list
  in order, the digests. Most consumers want one string.
- **the corpus release it embeds**, in a field a program can read, so a consumer
  can tell how old their data is without leaving their language.

⚠ **Selection returns an option, never a fallback.** A profile for a platform
this project has not captured does not exist, and returning a neighbouring
platform's would put an unmeasured value behind a measured interface. That is
the same rule as the routes in `PUB-03`.

Must not: fetch at runtime by default, expose a builder that lets a caller
assemble a profile from parts of different profiles, or hide the provenance map.

### Prove

```bash
cargo test -p b-ids -- --nocapture
```

Passing means: the crate builds and its tests pass with no network; selecting an
uncaptured platform returns nothing rather than a substitute; and a test asserts
the embedded release identifier matches the corpus the build was cut from.

---

## LIB-02. The smallest client that proves a profile is usable

**Source** the operator
**Category** library, **Priority** P2, **Effort** M, **Status** open

### Problem

A corpus that no client can emit is a corpus that is accurate and useless.
Nothing currently demonstrates that a profile in this project's shape can be put
back on a wire.

### Premise

Believed, and the constraint is the point: this is a proof, not a product.

### Approach

One binary, deliberately minimal. It selects a profile from `LIB-01`, opens a
connection, and makes a request. It supports a method, a URL, headers and a
body, and nothing else.

⛔ **It is not a general-purpose HTTP client and must not grow into one.** No
cookie jar, no redirect policy, no retry logic, no proxy support, no output
formatting. Every one of those is a reason to add a flag, and a client with
forty flags is a second product this project has not agreed to maintain.

The acceptance is the whole reason it exists: point it at the local harness,
capture what it sent, and compare against the profile it claimed, field by
field. ⭐ Anything less than a field-level match is a hole, and it goes in the
support matrix in `EMIT-01` rather than being smoothed over.

⚠ **Expect it not to match on the first attempt**, and expect the first
mismatches to be the known holes: an unenumerated extension codepoint, an
arbitrary extension order, and a settings key the underlying stack will not
omit. That is the honest outcome and it is more valuable than a claim of
success.

Must not: report a pass on a digest comparison alone. Two profiles can share a
digest and differ in a field the digest sorts away.

### Prove

```bash
cargo run -p b-ids-cli -- --profile chrome-152.0.7977.64-linux64-stable --url https://127.0.0.1:PORT/
```

Passing means: run against the harness on that port, the conformance report
names every differing field; the run exits zero only when the differing set is
empty or is exactly the set the support matrix already records as holes for this
stack.

---

## LIB-03. Bindings for the ecosystems that will ask

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** library, **Priority** P3, **Effort** L, **Status** open

### Problem

Most consumers of this kind of data are not writing Rust, and a corpus reachable
only from one language is a corpus most people will re-implement badly.

### Premise

Believed. ⚠ Blocked on `LIB-01` having a stable shape, because a binding written
against a moving interface is a binding that breaks on every release.

### Approach

One package per ecosystem, each a thin binding over `LIB-01` rather than a
reimplementation, published on the same schedule as the corpus release per
`PUB-05`.

⛔ **A reimplementation in each language is the failure to avoid.** Four
implementations of one selection rule is four places for it to be wrong, and the
one that is wrong is the one nobody uses often enough to notice.

Must not: let a binding expose a shape the Rust crate does not have, or diverge
on what happens when a profile is missing.

### Prove

```bash
sh scripts/common/check-bindings.sh
```

Passing means: every binding answers identically to the Rust crate over one
fixture corpus, including the case where a profile is absent, and the comparison
is over the answers rather than over the interfaces.
