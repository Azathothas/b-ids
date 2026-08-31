# schema

The data model, its provenance rules, and the formats generated from it.
Everything else in this project is a producer or a consumer of what these
entries define, so they are first.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## SCHEMA-01. The profile: one browser, one build, one platform, one channel, one instant

**Source** the founding brief; the four provenance kinds are [`../docs/glossary.md`](../docs/glossary.md)
**Category** schema, **Priority** P0, **Effort** M, **Status** open

### Problem

Nothing in this repository defines what a captured fingerprint is. Every other
component reads or writes one, so none of them can start.

### Premise

Believed, from a design brief that was never implemented: the unit is one exact
build on one platform in one channel at one instant, and a version alone is not
an identity. A profile keyed on "Chrome 152" cannot express that two builds of
that major sent different bytes.

The proposed shape, which has never been round-tripped by code:

```jsonc
{
  "schema": "browser-profile/1",
  "id": "chrome-152.0.7977.64-linux64-stable",
  "browser": { "name": "Chrome", "version": "152.0.7977.64", "major": 152,
               "channel": "stable", "branded": false },
  "platform": { "os": "linux", "arch": "x86_64", "distribution": "debian-bookworm" },
  "captured": { "at": "2026-08-30T03:53:11.204Z", "method": "container",
                "harness": "loopback-tlsprobe 0.2.0", "operator": "ci" },
  "tls":    {},
  "http2":  {},
  "http":   {},
  "digests":{ "ja3": "", "ja4": "", "ja4_r": "", "ja4_ro": "", "akamai": "" },
  "raw":    { "client_hello_hex": "", "settings_frame_hex": "" }
}
```

### Approach

Write the JSON Schema first, then the Rust types that serialise to it, so the
schema is the artefact and the types are checked against it rather than the
other way round.

The three halves the model carries, each defined in its own entry:

- the TLS half, `SCHEMA-02`;
- the HTTP/2 half, `SCHEMA-03`;
- the HTTP half, `SCHEMA-04`.

`digests` and `raw` are siblings of the measured halves, never inside them. A
sibling `third_party` block is the shape `curl-impersonate` already uses for
derived values, and [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md)
section 3 is the reading.

The identifier is derived from the four keys and is not a hash. It exists so a
path can be constructed without an index.

Must not: key anything on a digest, accept a version without a build number, or
make `captured.at` optional.

### Prove

```bash
cargo test -p b-ids-schema -- --nocapture
```

Passing means: a hand-written profile validates against the published schema, a
profile missing `captured.at` is rejected with a message naming that field, and
a profile whose `id` disagrees with its four keys is rejected.

---

## SCHEMA-02. The TLS half, in wire order, with unknown codepoints kept

**Source** the founding brief; the extension model is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 1
**Category** schema, **Priority** P0, **Effort** M, **Status** open

### Problem

A model that enumerates extensions cannot record an extension nobody has
enumerated. Two such codepoints are already known to exist in a shipped browser,
and the model has to hold them on the day it is written, because a capture
cannot be retaken.

### Premise

Measured by reading, at named commits, in
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md):
one reference database uses a boolean per extension over a closed enum, and one
TLS library documents that unknown extensions are dropped during parsing. A
third library holds an ordered list of codepoint-and-body pairs and refuses an
unknown codepoint by default rather than dropping it.

### Approach

Record, all in wire order:

- `cipher_suites`, `u16[]`, GREASE included;
- `key_exchange_groups`, `u16[]`, GREASE included;
- `signature_algorithms`, `u16[]`;
- `extensions`, an ordered array of objects carrying `type`, `length` and
  `body_hex`. Not booleans, and not a set;
- `alpn`, `string[]`;
- `ech`, its mode and its key-exchange identifier;
- `record_version`, `legacy_version`, `session_id_len`, `session_id_hex`,
  `compression_methods`;
- key-share **entry lengths** as well as group identifiers;
- `signature_algorithms_cert` when present, and the padding extension's length;
- `shuffled`, whether the extension order varies per connection;
- `grease`, the positions, whether the two values were distinct, and the bodies.

Take the ordered-list shape from the library that has it, and take its refusal
too: an unknown codepoint is kept with its bytes, and a parser that cannot keep
it stops rather than dropping it.

Must not: name a GREASE codepoint as a typed field, which makes a GREASE
extension carrying a byte unparseable.

### Prove

```bash
cargo test -p b-ids-schema tls_extensions -- --nocapture
```

Passing means: a fixture containing an extension the parser has no name for
round-trips to identical bytes, and a fixture whose GREASE extension carries one
zero byte parses rather than erroring.

---

## SCHEMA-03. The HTTP/2 half, as an ordered frame sequence

**Source** the founding brief; the units are [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 2
**Category** schema, **Priority** P0, **Effort** S, **Status** open

### Problem

A settings map loses order, and order is part of the fingerprint. A settings map
also cannot say which settings were **absent**, and absence is load-bearing: one
browser sends no `SETTINGS_MAX_FRAME_SIZE` where a general-purpose stack sends
the protocol default, which is a visible difference.

### Premise

Read rather than measured. Three independent sources agree on a Chrome settings
list; one of them reports that a settings key present in one version is absent
in a later one, so the set is not version-invariant.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 4 carries
the values and their status.

### Approach

Record an ordered frame list rather than a settings map: SETTINGS with its
entries in order, the connection WINDOW_UPDATE, and the HEADERS block, as one
sequence with the frame kinds named. That makes order and absence both
expressible, and it is the shape one reference corpus already uses.

Separately:

- `window_update`, named for the **increment** rather than the window, per
  `SCHEMA-09`;
- `stream_priority`, the exclusive bit, the dependency and the weight, or an
  explicit null, from the PRIORITY block on the first HEADERS frame;
- `priority_frames`, standalone PRIORITY frames sent before HEADERS. A different
  seam from the block above, and both exist;
- `pseudo_header_order`.

Must not: represent an absent setting as a default value, or record the priority
block only as a rendered Akamai string. That string cannot distinguish "no block
sent" from "block not read", which is why the field is recorded as the parsed
five bytes and the string is derived from it. [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5 is what
that field is worth today.

### Prove

```bash
cargo test -p b-ids-schema http2_frames -- --nocapture
```

Passing means: a profile that omits one settings key and a profile that carries
it at the protocol default serialise differently and compare unequal.

---

## SCHEMA-04. The HTTP half, its variants, and the one privacy rule

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P0, **Effort** S, **Status** open

### Problem

Header sets differ by request kind. A top-level navigation, a subresource fetch
and a reload are three different sets from one browser, and a corpus that
records one without saying which cannot be compared against anything.

Separately, header **values** can carry credentials, and a corpus that records
them by default is a corpus that will one day publish one.

### Premise

⛔ **The premise this entry was filed on was refuted.** It read that one
inherited capture carried `cache-control: max-age=0` and another did not, with
the cause unisolated. Neither capture carries it, and the founding brief was
wrong about its own source.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 10 has the
refutation and section 6 has both header sets in full.

⭐ **The entry survives the refutation because nothing in it rested on that
capture.** A navigation, a subresource fetch and a reload are three request
kinds by construction, and a corpus that records one set without saying which
kind produced it is uncomparable whether or not any two captures happen to
differ. ⚠ What changes is that this project has **no** measured example of the
difference, so `variants` is designed rather than derived, and the first capture
of two kinds at one version is what turns it into evidence.

### Approach

`headers` is an ordered array of name-and-value pairs. `variants` names which
request kind produced each set: at minimum `navigate`, and the kind is recorded
rather than assumed.

The privacy rule, which is a schema rule because the default shape enforces it:

- the default shape carries header **names only**;
- values are recorded only behind an explicit switch;
- `cookie` and `authorization` are dropped even when that switch is on;
- a capture is taken only from a browser the harness launched itself, into a
  throwaway profile, having visited nothing.

Must not: make the value-carrying shape the default, or leave the switch
untested.

### Prove

```bash
cargo test -p b-ids-schema header_privacy -- --nocapture
```

Passing means: a capture taken with no switch contains no header value at all,
and a capture taken with the switch on contains no `cookie` and no
`authorization`. Both are asserted, and the first is asserted over a fixture
that does contain values, so the test can fail.

---

## SCHEMA-05. Provenance is per field, with four kinds and no more

**Source** the founding brief; the four provenance kinds are [`../docs/glossary.md`](../docs/glossary.md)
**Category** schema, **Priority** P0, **Effort** S, **Status** open

### Problem

A consumer has to be able to ask "was this measured or assumed" per **field**,
not per profile. Without it, a profile that is nine tenths measured and one
tenth copied is indistinguishable from one that is entirely measured, and the
copied tenth is the part that is wrong.

### Premise

Believed. The design brief proposes a sibling map rather than wrapping every
scalar, on the reasoning that wrapping makes every consumer pay for a field
nobody reads.

```jsonc
"provenance": {
  "tls.cipher_suites":               "wire",
  "http.headers.sec-ch-ua":          "wire",
  "http.headers.sec-ch-ua-platform": "substituted:platform-token",
  "tls.extensions.0xca34":           "unreproducible:root-store-snapshot"
}
```

### Approach

Four kinds and no more, each optionally suffixed with a reason after a colon:

| kind | means |
| --- | --- |
| `wire` | read off a socket by this project's harness |
| `substituted` | taken from a capture of the same build on another platform |
| `vendor` | copied from somebody's table, unverified |
| `unreproducible` | measured, and deliberately not shipped |

Two rules the validator enforces, in `VALID-01`: a published profile carries no
`vendor` field, and a `substituted` or `unreproducible` field without a reason is
malformed.

Must not: add a fifth kind. Four is the whole vocabulary, and a fifth is how a
provenance model stops meaning anything.

### Prove

```bash
cargo test -p b-ids-schema provenance -- --nocapture
```

Passing means: a profile with an unreasoned `substituted` field is rejected with
a message naming that field, and a profile with a `vendor` field validates as a
draft and fails the published-profile check.

---

## SCHEMA-06. Record everything the wire carried, from the first commit

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P1, **Effort** M, **Status** open

### Problem

A capture is a moment that cannot be retaken. The build will be gone, the
download will stop being served, and the machine will be reimaged. A field
dropped because nobody could imagine a consumer is a field nobody can recover.

### Premise

Believed, and the cost of being wrong is asymmetric: a kept field costs bytes,
and bytes are the cheapest thing in this project.

### Approach

Add to the model, each of which is one line to record:

| layer | keep |
| --- | --- |
| TCP and IP | source port, MSS, window size and scale, TTL, TCP option order. Free once the project owns the listener, and it is a whole second fingerprint. |
| TLS record | record-layer version, the fragmentation pattern, whether the hello spanned records |
| `ClientHello` | everything in `SCHEMA-02`, verbatim |
| TLS response | what the server selected, and whether the client continued or aborted. A client's reaction to a retry request is itself a fingerprint. |
| HTTP/2 | frame order and sizes, SETTINGS order, WINDOW_UPDATE timing, HPACK dynamic table size updates, whether each header was Huffman coded, header block fragmentation across continuation frames |
| HTTP | every header verbatim including the ones that look like noise, the request kind, and the exact bytes of the request line |
| timing | inter-frame gaps, and how long the browser waited before its first byte. Coarse, and some detectors use it. |
| environment | operating system, architecture, distribution, container image digest, harness version, exact browser build |

Two rules that follow and are part of this entry:

- **the raw bytes are always kept**, as hex, for the hello and the frames. This
  is the backstop against this project's own parser being wrong, which it will
  be, and it is the only artefact in which a GREASE question is answerable;
- **the schema is additive**. Fields are added, never removed and never
  repurposed. Removing one is a new major, and a new major is a promise to keep
  serving the old one.

Must not: treat an unknown thing as an absent thing. A codepoint nobody can name
still gets its length and its body recorded verbatim.

### Prove

```bash
cargo test -p b-ids-schema raw_backstop -- --nocapture
```

Passing means: a profile is rebuilt from its `raw` block alone by a second code
path, and the result compares equal to the parsed profile field by field.

---

## SCHEMA-07. What must never be in the model

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P1, **Effort** S, **Status** open

### Problem

Two classes of value look like identity and are not. Storing either makes a
profile that changes for reasons nothing in the corpus can explain.

### Premise

Structural rather than measured.

### Approach

Refuse, in the schema and in the validator:

- **any digest as a primary key.** JA3, JA4 and the Akamai string are derived.
  Store them, never key on them, and never let a consumer round-trip a profile
  through one.
- **anything a browser learns from the network.** Session tickets, pre-shared
  keys, settings a server echoed, encrypted-hello configurations fetched from
  DNS. That is connection state, not identity, and it is why a resumed handshake
  is recorded separately rather than averaged in.

Must not: silently drop these at capture time. They are recorded in the raw
bytes; what is refused is promoting them into the identity.

### Prove

```bash
cargo test -p b-ids-schema refused_fields -- --nocapture
```

Passing means: a profile whose identifier is derived from a digest is rejected,
and a profile carrying a session ticket in its TLS half is rejected, both with
messages naming the field.

---

## SCHEMA-08. Every generated format, from one generator, round-tripped

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P1, **Effort** L, **Status** open

### Problem

JSON is one consumer, not the consumer. A corpus reachable only by writing a
JSON walker is a corpus most people will copy values out of by hand.

### Premise

Believed. No format has been generated.

### Approach

One generator, canonical JSON in, every format out. A test asserts each format
round-trips back to the canonical form wherever the format is expressive enough,
and asserts the documented subset where it is not.

| format | for |
| --- | --- |
| JSON | canonical. Every other format is generated from it. |
| NDJSON | streaming, line-oriented tools |
| YAML | reading and hand-editing one profile |
| TOML | consumers that already parse it |
| CSV and TSV | spreadsheets and shell pipelines, one row per profile, key fields flattened |
| Markdown | browsing on the web with no tooling |
| SQLite | one queryable file. A query beats a walker, and it is nearly free to produce. |
| CBOR and MessagePack | compact binary, for embedded and mobile consumers |
| Protobuf | a published definition plus binaries, for a typed decoder |

CSV and Markdown are lossy. Each says so in its own header, and the round-trip
test asserts the documented subset rather than equality.

Must not: hand-edit a generated format. If one is ever edited directly, the
generator has lost and the round-trip test is what says so.

### Prove

```bash
sh scripts/common/check-formats.sh
```

Passing means: every format is regenerated from the canonical corpus, each
lossless format round-trips to byte-identical canonical JSON, and two runs of
the generator produce byte-identical output.

---

## SCHEMA-09. Name every field for the wire, because three quantities have two units

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 2
**Category** schema, **Priority** P1, **Effort** S, **Status** open

### Problem

Three quantities in this domain each have a human-facing number and a wire
number, and every reference project examined has confused at least one of them.
The confusion produces a fingerprint that is wrong by a fixed offset, which
looks like a subtle browser difference rather than an arithmetic error.

### Premise

Measured by reading, at a named commit. In one reference database the same
field carries the window in one entry and the increment in seven others; its own
comment defines the field as the window. A third party audited the same field
independently and found the same split.

| quantity | human | wire |
| --- | --- | --- |
| connection window | 15 MiB | that minus the protocol's 65,535 default |
| stream priority weight | 256 | 255, because the protocol encodes weight minus one |
| the settings a stack "does not override" | absent from the profile | the stack's own default, on the wire |

### Approach

Name every field for the wire value: `window_update_increment`, not
`connection_window`. Where a human-facing number is worth carrying, carry it as
a second, separately named field, and have a check assert the arithmetic between
them rather than a comment asserting it.

The third row is not a naming problem but the same class: an `Option::None`
that means "do not override" produces a value on the wire. The schema
distinguishes absent from defaulted, per `SCHEMA-03`.

Must not: rely on a comment to carry a unit. The comment in the reference
database was correct and seven entries beside it were still wrong.

### Prove

```bash
cargo test -p b-ids-schema units -- --nocapture
```

Passing means: a profile whose named increment and named window do not differ by
exactly 65,535 is rejected, and a profile whose weight field is 256 is rejected
with a message naming the encoding.

---

## SCHEMA-10. Record the shuffle as a property, and consider recording its seed

**Source** the founding brief; the ceiling is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 5
**Category** schema, **Priority** P2, **Effort** M, **Status** open

### Problem

A browser that shuffles its extension order produces a different sequence per
connection. A profile that records one sequence and says nothing about the
shuffle is recording one draw as though it were the shape.

### Premise

Measured by reading: one TLS library derives its extension order as a pure
function of a sixteen-bit seed, after removing the extensions that must be last
and any group that must stay contiguous. That has a consequence nobody has
stated: at most 65,536 orders are reachable, out of the factorial of the
extension count.

### Approach

The profile records `shuffled: bool`, how many draws the capture observed, and
the orders it saw. It does not claim a canonical order for a browser that
shuffles.

Consider, and decide in this entry: recording a **seed** alongside, for emitters
whose order is seed-derived. It makes an exact reproduction searchable in 65,536
tries, and it is meaningless for an emitter that shuffles differently.

Two things stay pinned regardless: the pre-shared-key extension is last by
specification, and GREASE occupies the two ends.

Must not: assert equality between two captures of a shuffling browser.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 2 states
what two consecutive captures of one binary have to show, and why reproducing a
recorded order exactly is a reason to doubt the capture.

### Decision

Record the seed as an optional, emitter-scoped field, or leave it out.

Recommendation: leave it out of `browser-profile/1` and carry it in the emitter
support matrix instead. It is a property of a reproduction attempt rather than
of a browser, and putting it in the profile invites a consumer to read it as
something the browser sent.

### Prove

```bash
cargo test -p b-ids-schema shuffle -- --nocapture
```

Passing means: a profile marked `shuffled` with exactly one observed order fails
validation, and a fixture of eight captures of one binary that shows a single
order fails with a message saying so.

---

## SCHEMA-11. The multipart boundary, which is a per-browser surface nobody listed

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 8
**Category** schema, **Priority** P2, **Effort** S, **Status** open

### Problem

The boundary string a browser generates for a multipart form body is
browser-specific, appears in every form submission, and is in no version of this
project's model.

### Premise

Measured by reading, at a named commit: one client generates
`----WebKitFormBoundary` plus sixteen alphanumerics for one browser,
`----geckoformboundary` plus thirty-two hexadecimal characters for another, and
a random identifier for a mobile stack. That client's tracker carries a closed
issue reporting that its earlier format did not match a browser.

### Approach

Add a `http.multipart_boundary` field recording the **pattern**, not one drawn
value: the literal prefix, the length of the random part, and its alphabet.

Must not: record one generated boundary as though it were the value. It is drawn
per request, like GREASE.

### Prove

```bash
cargo test -p b-ids-schema multipart -- --nocapture
```

Passing means: a fixture of sixteen boundaries from one browser all match the
recorded pattern, and a boundary from another browser does not.
