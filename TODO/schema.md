# schema

The data model, its provenance rules, and the formats generated from it.
Everything else in this project is a producer or a consumer of what these
entries define, so they are first.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## SCHEMA-01. The profile: one browser, one build, one platform, one channel, one instant

**Source** the founding brief; the four provenance kinds are [`../docs/glossary.md`](../docs/glossary.md)
**Category** schema, **Priority** P0, **Effort** M, **Status** done

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

### Closing

**Closed 2026-08-31.** The published schema is the artefact and the Rust types
are checked against it, in that order.

```text
$ cargo test -p b-ids-schema -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

⚠ Five result lines because the acceptance runs five test binaries, one per
entry in this file. 45 tests. The two `0 passed` lines the raw output also
carries are the library and its doc-tests, which have none yet.

### What landed

- [`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json),
  written first, and the Rust types checked against it.
- The three measured halves in their own modules, with `digests` and `raw` as
  siblings. ⭐ A test asserts that shape rather than trusting the sentence:
  `digests_and_raw_are_siblings_of_the_measured_halves` walks each half and
  fails if any of them carries a derived key.
- The identifier derived from the four keys, with a test that moves **each of
  the four** and asserts the identifier moves with it. ⚠ A test that moved one
  would pass over an identifier that ignored the other three.

### ⭐ The schema checker refuses a keyword it does not implement

⛔ **This is the part worth reading.** The published schema is validated by a
subset checker written here rather than by a library. A subset checker is a
guard that silently ignores what it does not implement, so a keyword added to
the schema and implemented by nobody would be a constraint the schema states
and nothing enforces.

So `KNOWN_KEYWORDS` is declared and `check_schema_is_supported` walks the whole
schema and fails on anything outside it. ⚠ The alternative was a full JSON
Schema dependency; the trade is recorded rather than assumed, and the checker
says in its own header that it is a subset and must not be reached for as an
implementation.

### The platform token is not the platform

⚠ **`platform` is `{os, arch, distribution}` and the identifier carries
`linux64`.** They are different spellings on purpose: the identifier uses the
token a download index uses, so a published path and a downloaded build spell
the platform the same way. An architecture the mapping does not know is joined
with a dash rather than refused, because refusing there would design a ceiling
into every published path.

### Mutation-proved

```text
=== SCHEMA-01: captured.at is never optional ===
test a_profile_with_no_capture_instant_is_rejected_naming_the_field ... FAILED
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s
```

The guard was replaced with `if false` and the test refused. ⭐ The schema
checker was proved the same way, by a test that plants an undeclared property
and asserts the checker reports it.

---

## SCHEMA-02. The TLS half, in wire order, with unknown codepoints kept

**Source** the founding brief; the extension model is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 1
**Category** schema, **Priority** P0, **Effort** M, **Status** done

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

### Closing

**Closed 2026-08-31.** An ordered list of codepoint-and-body pairs, taken from
`utls`'s `ClientHelloSpec` rather than from either alternative.

```text
$ cargo test -p b-ids-schema tls_extensions -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### What the model refuses to lose

- **A codepoint with no name**, kept with its bytes and round-tripped to an
  identical string. The fixture carries `0xca34`, which is one of the two that
  stopped a version bump in the origin repository.
- **A GREASE extension carrying one zero byte.** ⛔ No GREASE codepoint is a
  typed field, because a typed field for one makes its body unparseable.
- **A declared length that disagrees with the recorded body**, reported rather
  than repaired. ⚠ Padding or truncating to make the two agree is the forbidden
  pattern; the disagreement is itself the measurement.
- **Key-share entry lengths** beside the group identifiers, because two builds
  sending one group with different key sizes are two different handshakes.

### ⭐ Two tests that would otherwise have been vacuous

⛔ **A round-trip test over a fixture with nothing unusual in it proves
nothing.** So the test asserts first that the fixture contains `0xca34` at all,
and the wire-order test asserts that the fixture's order differs from its sorted
form. Without the second assertion, a model that sorted its extensions would
pass a test named for keeping their order.

⚠ **And the GREASE predicate is counted rather than spot-checked.** RFC 8701
defines sixteen values; the test enumerates the whole `u16` space, asserts
exactly sixteen match, and checks the near misses `0x0b0b` and `0x0a0b`, which a
predicate written as "high equals low" would wrongly accept.

### Mutation-proved

```text
=== SCHEMA-02: the fixture must carry a codepoint with no name ===
test tls_extensions_unknown_codepoint_round_trips_to_identical_bytes ... FAILED
test tls_extensions_validate_against_the_published_schema ... FAILED
test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

⭐ **Two tests refused, and the second is the interesting one.** Removing the
unnamed codepoint from the fixture also broke the schema test, which asserts the
extension array has at least three entries. A fixture quietly shrinking is a way
for a suite to keep passing over less than it was written for.

### ⚠ What is designed rather than measured

**Every field here is a shape, and this project has captured nothing.** The
fixture's values are shaped like a real capture and are not one; the support
module says so in as many words, and no field of it may be copied into the
corpus. [`../docs/inherited-claims.md`](../docs/inherited-claims.md) is where a
value that came from somewhere else lives.

---

## SCHEMA-03. The HTTP/2 half, as an ordered frame sequence

**Source** the founding brief; the units are [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 2
**Category** schema, **Priority** P0, **Effort** S, **Status** done

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

### Closing

**Closed 2026-08-31.** An ordered frame list, so order and absence are both
expressible.

```text
$ cargo test -p b-ids-schema http2_frames -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

The acceptance is `http2_frames_omitting_a_setting_differs_from_sending_it_at_the_default`,
which builds both profiles, asserts they compare unequal and asserts their
serialisations differ.

### The three units, named for the wire

⛔ **The connection quantity is `window_size_increment`**, and a test asserts
that no field is named for the window: a name carrying the other unit is how one
shipped database ended up holding both meanings in one field, with seven of its
entries 65,535 short.

⛔ **The stream weight is `weight_wire`**, the value as encoded, which HTTP/2
defines as the weight minus one. `weight_spec()` derives the other unit on
request, and a test asserts it is **not** stored: storing both is how a field
ends up holding whichever unit its last writer believed in.

### ⭐ The rendered string is derived, and the test states what it loses

`akamai_text()` is computed from the model rather than stored. ⚠ The test
asserts the rendering AND the loss: an absent priority block and a block of
zeroes both render as `0`, while the model still compares them unequal. That is
why the field is the parsed five bytes, and it is why two of the three sources
reporting a zero for this field were reading a tool that could not write the
block rather than a browser.

### Mutation-proved

```text
=== SCHEMA-03: an absent setting is never a default value ===
test http2_frames_omitting_a_setting_differs_from_sending_it_at_the_default ... FAILED
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`sends_setting` was replaced with one that answers for the frame rather than the
entry, which is the shape a settings map filled in with defaults would have. The
test refused.

### ⚠ Still open, and named here so it is not lost

**`SCHEMA-09` is partly satisfied and stays open.** The two quantities this half
carries are named for the wire; the third, and a sweep of every other field for
the same trap, is that entry's. **`HARNESS-05`** is what turns the priority
block from `vendor` provenance into a measurement.

---

## SCHEMA-04. The HTTP half, its variants, and the one privacy rule

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P0, **Effort** S, **Status** done

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
- `cookie` and `authorization` keep their name and their position and lose
  their value even when that switch is on, marked `withheld`. `SCHEMA-14`;
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

### Closing

**Closed 2026-08-31.** The privacy rule is the default shape rather than a flag
a caller has to remember.

```text
$ cargo test -p b-ids-schema header_privacy -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### ⭐ One construction path, and the fixture that makes the test able to fail

⛔ **`HeaderSet::record` is the only way to build a header set from wire
input**, and the filter lives inside it. A second path that skipped the filter
would be the "control gated on one of several paths" defect in the one place
this project cannot afford it.

⛔ **The input fixture carries values and a credential**, and a test asserts
that separately, before anything else. A privacy test over an empty input passes
forever and proves nothing, so the assertion that the input has something to
drop is its own test.

⚠ **`ValuePolicy::NamesOnly` is the `Default`**, and a test asserts that rather
than the documentation of it. A switch that has to be turned off for safety is a
switch that ships on.

⚠ **The credential filter is case-insensitive**, because HTTP/2 lower-cases
header names and a read from an HTTP/1.1 connection does not. A rule that
catches one spelling catches nothing on the other wire.

### ⚠ A design note, recorded rather than acted on

**Dropping `cookie` and `authorization` entirely also drops the fact that the
header was present**, and presence is a fingerprint signal in its own right. The
acceptance in this entry says the capture contains neither, so that is what
landed. ⛔ Recording presence without the value would be a different shape and a
different ruling, and changing an approved acceptance while implementing it is
not this session's to make. It is written here so a later entry can take it up.

### Mutation-proved

```text
=== SCHEMA-04: the default records no value ===
test header_privacy_the_default_records_no_value_at_all ... FAILED
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The policy match was replaced with one that always keeps the value. One test
refused, and the six around it did not, which is what tells you the failure is
the policy rather than the harness.

### ⚠ The variant model is still designed, not derived

This project has **no** measured example of two request kinds differing, and the
claim that sent this entry to be written was refuted against the capture it came
from. The first capture of two kinds at one version is what turns `variants`
into evidence.

---

## SCHEMA-05. Provenance is per field, with four kinds and no more

**Source** the founding brief; the four provenance kinds are [`../docs/glossary.md`](../docs/glossary.md)
**Category** schema, **Priority** P0, **Effort** S, **Status** done

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

### Closing

**Closed 2026-08-31**, with one part of the acceptance landing in a different
place from where it was written, and that is said plainly below.

```text
$ cargo test -p b-ids-schema provenance -- --nocapture
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### The vocabulary is closed in the type

⛔ **Four kinds, and a fifth is refused by the parser rather than carried as a
string.** `ProvenanceEntry::parse` returns a defect naming the field and listing
the four, and a fifth kind written into a profile's JSON fails to deserialise at
all. Two tests, because those are two different doors into the same rule.

⚠ **A reason is required by `substituted` and `unreproducible`, and tested on
both.** A test over one of the two would pass over a rule applied to half its
subject.

### ⚠ Where the acceptance's second half actually lives

The acceptance says a `vendor` field "validates as a draft and fails the
published-profile check". **The draft half is here and the refusal is not.**

⛔ **A draft is not malformed**, and conflating the two would have been the
wrong model: `Profile::check` answers whether these bytes describe a profile at
all, and whether a profile may be PUBLISHED is a different question with a
different consumer. So this crate exposes `is_draft()` and
`Provenance::vendor_fields()`, which returns the field list a publisher has to
print, and `VALID-01` check 8 is what refuses. ⚠ That entry is open, so the
refusal does not exist yet; it is named here rather than left to be discovered.

### Mutation-proved

```text
=== SCHEMA-05: substituted and unreproducible need a reason ===
test provenance_an_unreasoned_unreproducible_field_is_rejected_too ... FAILED
test provenance_an_unreasoned_substituted_field_is_rejected_naming_the_field ... FAILED
```

`requires_reason` was replaced with one that answers `false`, and both tests
refused.

### The map is ordered, and that is not cosmetic

⚠ **A `BTreeMap`, so two serialisations of one profile are byte-identical.** An
unordered map turns a no-op re-emit into a diff, and a diff nobody can explain
is a diff nobody reviews. The round-trip test serialises twice and compares the
two strings, because a single pass would not have shown it.

---

## SCHEMA-06. Record everything the wire carried, from the first commit

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P1, **Effort** M, **Status** done

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

### Closing

**Closed 2026-09-01, and BEFORE the first capture rather than after it.** That
was the whole point of the ordering: retrofitting completeness is paid for in
captures nobody can take again.

```text
$ cargo test -p b-ids-schema raw_backstop -- --nocapture
running 9 tests
test raw_backstop_rebuilds_a_cleartext_request_through_the_one_construction_path ... ok
test raw_backstop_the_schema_is_additive ... ok
test raw_backstop_keeps_a_frame_type_the_model_has_no_name_for ... ok
test raw_backstop_rebuilds_the_http2_half_from_the_raw_block_alone ... ok
test raw_backstop_refuses_a_raw_block_that_disagrees_with_itself ... ok
test raw_backstop_refuses_a_record_layer_that_disagrees_with_the_hello ... ok
test raw_backstop_reports_every_half_the_raw_block_does_not_reproduce ... ok
test raw_backstop_rebuilds_the_tls_half_from_the_raw_block_alone ... ok
test raw_backstop_reports_a_half_that_disagrees_with_its_own_bytes ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### What the raw block carries now

| field | why it cannot be dropped |
| --- | --- |
| `client_hello_hex` | ⭐ the only artefact in which a GREASE question is answerable. Every digest, JA4_ro included, strips GREASE before it is computed. |
| `http2_frames_hex` | every frame in arrival order, head and payload, INCLUDING a frame type this project has no name for |
| `connection_hex` | the whole first message before anything was made of it, which is the widest backstop there is |
| `request_line_hex` | the BYTES of the request line. A request line is not guaranteed to be UTF-8, and a capture that stored it as text could not reproduce one that was not. |
| `record_layer` | the record's own version, its declared length, ⛔ how many bytes actually arrived, and whether the hello was fragmented |
| `settings_frame_hex` | kept for profiles written before the frame list existed, ⛔ with a check that the two agree |

⚠ **Three quantities in one hello are called "the version"**: the record
layer's, the handshake's, and the negotiated one. The record layer's now has a
home of its own, so a profile can say which it means.

### ⭐ The rebuild is a second ENTRY into the parser, not a second parser

Rebuilding with an independent implementation would test the two
implementations against each other and say nothing about whether the stored
bytes are sufficient. ⭐ **The question the raw block exists to answer is
whether the bytes are enough to produce the model again**, and that is what is
asserted: the capture path reads a socket, the rebuild path reads the stored
hex, and the two produce equal halves.

⚠ **The rebuild goes through `HeaderSet::record`**, which is the one
construction path and the one place the credential rule lives. A rebuild that
assembled the fields itself would be a fourth door into that rule, and a test
plants a `cookie` in the stored bytes to prove it is not.

### ⛔ A mutation reported nothing, and that was the finding

**The first mutation removed the comparison from the rebuild check entirely, and
all eight tests still passed.** Every one of them exercised the ABSENT branch,
where the raw block carries no bytes for a half; not one made a rebuilt half
DIFFER from a recorded one, so the comparison had never been seen to fire.

⭐ `raw_backstop_reports_a_half_that_disagrees_with_its_own_bytes` plants the
difference. That is what a parser change or a hand edit looks like from
outside, and it is the whole reason to keep the bytes.

⚠ **This is the second time this session that a mutation which reported nothing
produced a better finding than one that failed.** `HARNESS-04`'s was the same
shape.

### The schema is additive, and a test says so

A profile written before the frame list existed still reads: the new fields
default rather than being required, and the published JSON Schema keeps the two
original names in its `required` list. ⛔ Fields are added, never removed and
never repurposed. Removing one is a new major, and a new major is a promise to
keep serving the old one.

### ⚠ What is named in the approach and is NOT here, with the entry that owns it

⛔ Stated so that nobody reads an absence as a decision against it.

| row | why not, and where it goes |
| --- | --- |
| TCP and IP: source port, MSS, window and scale, TTL, option order | ⛔ **This harness does not read them at all.** It needs a raw socket or an equivalent, which is a capability question rather than a code one, and adding the fields with nothing able to fill them would be the "value the engine reads that nobody can set" defect. `HARNESS-11` establishes the capability first. |
| the TLS response, and whether the client continued or aborted | there is no response: the handshake is not terminated. `--ca-out`. |
| inter-frame gaps, and how long the browser waited | ⚠ a timing surface, and the harness records no clock per frame yet. It is not blocked by anything except that nothing needs it before a capture exists. |
| operating system, architecture, container digest, harness version | ⭐ already modelled, in `platform` and `captured`, rather than in `raw`. A duplicate in the raw block would be one fact in two places. |

### ⚠ A gap in the published schema, found while extending it and not fixed here

**The schema expresses no numeric bounds anywhere.** `u8`, `u16` and `u32` are
each a bare `{"type": "integer"}`, so the published schema accepts 999 for a
field the Rust type holds to a byte. Adding a `minimum` to one new field would
have been one bounded field among dozens of unbounded ones, and the checker's
own guard refuses a keyword it does not enforce, so the new field follows the
convention that is there.

⭐ **The guard behaved exactly as designed**: it refused the schema the moment
it used a keyword nothing enforced. That is the check working rather than a
defect in it.

### Mutation-proved

```text
=== SCHEMA-06: the rebuilt half is COMPARED, mutated to accept any rebuild ===
failures:
    raw_backstop_reports_a_half_that_disagrees_with_its_own_bytes
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

=== SCHEMA-06: the raw block agrees with itself, mutated to skip the check ===
failures:
    raw_backstop_refuses_a_raw_block_that_disagrees_with_itself
    raw_backstop_refuses_a_record_layer_that_disagrees_with_the_hello
test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## SCHEMA-07. What must never be in the model

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P1, **Effort** S, **Status** done

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

### Closing

**Closed 2026-08-31.** Both classes are refused by `Profile::refused_fields`,
which `Profile::check` calls, so no consumer has to remember to run it.

```text
$ cargo test -p b-ids-schema refused_fields -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### ⭐ The distinction the entry rests on, made precise

The entry says a session ticket in the TLS half is refused. **What is refused is
the ticket's CONTENTS, not the codepoint's presence**, and the difference is
load-bearing in both directions:

| what | why |
| --- | --- |
| `session_ticket` present with an EMPTY body | ⭐ identity. A browser sends it that way on a cold connection, and the extension being in the list at all is part of the fingerprint. Kept. |
| `session_ticket` carrying bytes | ⛔ connection state. The browser learned it from a previous connection, and a profile carrying it changes for reasons nothing in the corpus can explain. Refused. |

⚠ **Both codepoints, not one.** A resumed handshake offers `pre_shared_key`
(`0x0029`) where a fresh one offers `session_ticket` (`0x0023`), so a rule
holding one would pass over exactly the connection this project must not average
in. Two tests, one per codepoint.

⛔ **And the raw hello is never edited.** The entry says so directly: the bytes
stay in `raw.client_hello_hex` and what is refused is promoting them into a
parsed field. A test asserts that a raw capture containing ticket bytes is not
itself a defect, because a capture is a moment that cannot be retaken.

### The digest rule, and the pass beside the refusal

⛔ A profile whose `id` equals any of its five digests is refused, naming which
digest. ⭐ **And a test asserts that STORING a digest beside the identity is
fine**, because without it the refusal test proves only that something was
refused rather than that the boundary is in the right place. That is what
`digests` is for; keying on one is what is forbidden.

### ⚠ What this does not reach

**"Settings a server echoed" is named in the approach and is not refused here**,
because nothing in the model can carry one: the HTTP/2 half records what the
CLIENT sent. It becomes reachable when `HARNESS-03` terminates a handshake and a
server's own SETTINGS frame is on the connection, and the rule is written down
here so that entry does not have to rediscover it.

---

## SCHEMA-08. Every generated format, from one generator, round-tripped

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** schema, **Priority** P1, **Effort** L, **Status** done

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
| CSV and TSV | spreadsheets and shell pipelines, one row per profile, key fields flattened |
| Markdown | browsing on the web with no tooling |

CSV and Markdown are lossy. Each says so in its own header, and the round-trip
test asserts the documented subset rather than equality.

### ⭐ Re-scoped 2026-09-01, by the operator, and the other six moved

⛔ **This entry carried nine formats and six of them needed a dependency.** Its
acceptance is a round trip, and a round trip needs a READER as well as a writer.
The five above can be written and read back with what this tree already has;
YAML, TOML, SQLite, CBOR, MessagePack and Protobuf each need an encoder **and**
a decoder, so each is either a new dependency or a new parser this project owns
and has to keep correct.

⭐ **They moved to `SCHEMA-12`, with the trade stated there rather than left
implicit here.** ⛔ What the ruling refused is twelve hand-written
implementations in the crate that already owns four parsers.

⚠ **The premise and the title above are unchanged**, per
[`../docs/methodology/authoring.md`](../docs/methodology/authoring.md): a
re-scope is written underneath rather than edited over the top of what was
believed.

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

### ⭐ Closed 2026-09-02. Five formats, one generator, and a reader for each round trip

⛔ **A format with a writer and no reader can only be checked against the thing
that wrote it**, which is the shape this entry was re-scoped to avoid. Every
lossless format here reads back into profiles and re-renders to byte-identical
canonical JSON; every lossy one reads back into the documented subset and
refuses to become a profile at all.

```text
$ sh scripts/common/check-formats.sh
formats ok: 5 file(s) from 6 profile(s), byte-identical over two runs,
  every lossless format round-trips to canonical JSON and every lossy one
  carries the documented subset.
exit=0

$ sh scripts/common/check-formats.sh --json
{"schema":"check-formats/1","files":5,"profiles":6,"problems":0}

$ pwsh -NoProfile -File scripts/common/check-formats.ps1 -Json
{"schema":"check-formats/1","files":5,"profiles":6,"problems":0}
```

```text
$ cargo test -q -p b-ids-corpus --test formats
running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

#### What each format is, and what the lossy ones leave out

| format | lossless | round trip |
| --- | --- | --- |
| `corpus.json` | ⭐ canonical. Every other format is generated from it. | itself |
| `corpus.ndjson` | yes | reads back into profiles and re-renders to byte-identical canonical JSON |
| `corpus.csv` | ⚠ no | the eight columns of `FLAT_COLUMNS`, read back and compared row by row |
| `corpus.tsv` | ⚠ no | the same |
| `corpus.md` | ⚠ no | ⛔ its own header says it is generated and where the rest of a profile is, because a reader arriving at a rendered table on the web has nothing else to read |

⛔ **`FLAT_COLUMNS` IS the documented subset**, rather than a sentence in a
comment. `read_flat` checks the header row against it, so a column added to the
writer without a reader beside it is a refusal rather than a quiet widening.

```text
id,browser,version,channel,branded,os,arch,captured_at
chrome-151.0.7922.173-linux64-stable,Chrome,151.0.7922.173,stable,true,linux,x86_64,2026-09-02T02:23:31Z
chrome-152.0.7977.75-linux64-stable,Chrome,152.0.7977.75,stable,true,linux,x86_64,2026-09-02T14:23:16Z
edge-151.0.4129.101-linux64-stable,Edge,151.0.4129.101,stable,true,linux,x86_64,2026-09-02T13:52:53Z
```

#### ⛔ Determinism is asserted rather than assumed

⚠ **A generator that read a clock, a hash seed or a directory order would
produce a diff on every run**, and a published artefact that diffs on every run
is one nobody can tell a real change from. The acceptance generates twice into
two directories and compares the bytes; the suite renders twice and compares the
strings. ⭐ The profile order is the store's route order, which is sorted, so it
does not depend on which host walked the tree.

#### ⚠ What this deliberately does not do

| | |
| --- | --- |
| ⛔ **it writes nothing into the tree** | the acceptance generates into `.tmp` and nothing in this repository publishes generated formats yet. `PUB-02` and `PUB-03` are the surfaces that will, and this exists before them so the generator is proved before anything depends on it |
| ⚠ **the six that need a decoder are `SCHEMA-12`'s** | YAML, TOML, SQLite, CBOR, MessagePack and Protobuf each need an encoder AND a decoder, which is a dependency or a parser this project owns. The re-scope above is the ruling and it stands |
| ⚠ **a lossy format cannot be read back into a profile, and says so** | `read_back` on a CSV returns a refusal naming the column count rather than a profile built from eight fields. A profile assembled from a spreadsheet row would be a fabricated one wearing a measurement's label |

⭐ **`check-formats` is in the gate and in `check-twins`**, both halves, so the
two acceptance wrappers cannot drift. ⚠ The row compares two wrappers over one
generator rather than two implementations of the round trip, and that is
deliberate: a round trip written twice would be two readers of five formats,
disagreeing the first time either moved.

---

## SCHEMA-09. Name every field for the wire, because three quantities have two units

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 2
**Category** schema, **Priority** P1, **Effort** S, **Status** done

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

### Closing

**Closed 2026-08-31.** Both quantities carry the wire number, the human number
is a separately named second field, and a check asserts the arithmetic between
them.

```text
$ cargo test -p b-ids-schema units -- --nocapture
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### ⛔ The check computes; it does not quote

The approach says "have a check assert the arithmetic between them rather than a
comment asserting it", and the reason is in the premise: in the reference
database the comment was CORRECT and seven entries beside it were still wrong.
So `Http2Half::check_units` subtracts, and a test walks four offsets around the
boundary rather than checking one wrong value.

### ⭐ The weight message names both units

A plain `u8` already makes 256 unrepresentable, ⚠ **and that is not enough**:
the error it produces reads "expected u8", which sends a reader looking for a
bounds bug rather than telling them they wrote the specification's unit into the
wire's field. So the field is read as a `u16` and refused above 255, and the
message says so:

```text
http2.stream_priority.weight_wire is 256, and the wire encoding is weight minus one,
so it holds 0 to 255. 256 is the specification's unit and 255 is the wire's
```

⚠ A test asserts 255 is ACCEPTED beside the one asserting 256 is refused, or the
pair proves only that something was refused.

### ⚠ A test in SCHEMA-03 forbade what this entry requires

`http2_frames_window_update_is_named_for_the_wire` asserted that no field is
named `connection_window` at all. That was right when it was written and became
wrong here, and the two entries are not actually in conflict once the rule is
stated exactly:

⛔ **No field is named for the window INSTEAD OF the increment.** A second field
named for the window BESIDE the increment, with a check asserting the difference,
is what this entry asks for. The assertion now forbids a bare `window_size`,
which is the name that could hold either, and asserts the arithmetic check
passes.

⭐ **The finding is that the earlier test was a proxy for the rule rather than
the rule.** It caught the right defect by forbidding a string, and a string ban
cannot tell "instead of" from "beside".

### The third quantity is a different shape, and it is already held

"The settings a stack does not override" is the same class and is not a naming
problem: an absent setting produces the stack's own default on the wire. The
model distinguishes absent from defaulted, which is `SCHEMA-03`, and a test here
asserts the two entries agree about it rather than leaving that to a reading.

---

## SCHEMA-10. Record the shuffle as a property, and consider recording its seed

**Source** the founding brief; the ceiling is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 5
**Category** schema, **Priority** P2, **Effort** M, **Status** done

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
cargo test --workspace shuffle -- --nocapture
```

Passing means: a profile marked `shuffled` with exactly one observed order fails
validation, and a fixture of eight captures of one binary that shows a single
order fails with a message saying so.

### Decision, ruled 2026-09-02

**The seed is left out of `browser-profile/1`.** The recommendation above is
taken: it is a property of a reproduction attempt rather than of a browser, and
a consumer finding it in a profile would read it as something the browser sent.
⭐ It belongs in the emitter support matrix, where a stack whose order is
seed-derived can say so; `EMIT-01` is that entry.

⚠ **The ceiling stays worth stating wherever the seed does go**: at most 65,536
orders are reachable from a sixteen-bit seed, out of the factorial of the
extension count, so "reproducible in 65,536 tries" is the property, not
"reproducible".

### ⚠ The acceptance runs the workspace, and the reason is a dependency edge

⛔ **Half of this entry is a refusal in the model and half is a finding in the
validator**, and `b-ids-schema` cannot depend on `b-ids-validator` because the
validator depends on it. The tests live in both crates and the acceptance runs
both:

```bash
cargo test --workspace shuffle -- --nocapture
```

### Closing

**Closed 2026-09-02T05:55:00Z.**

```text
$ cargo test --workspace shuffle -- --nocapture
     Running tests\shuffle.rs (target\debug\deps\shuffle-661855613b6c1cc1.exe)
test shuffle_a_profile_written_before_the_field_existed_still_reads ... ok
test shuffle_observed_with_two_orders_is_accepted ... ok
test shuffle_observed_with_one_order_is_refused ... ok
test shuffle_the_published_schema_carries_the_count ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\shuffle.rs (target\debug\deps\shuffle-ceefb44b54bbb71e.exe)
test shuffle_the_same_profile_is_clean_when_nobody_stated_the_family_shuffles ... ok
test shuffle_eight_draws_of_one_order_is_a_finding_for_a_family_that_shuffles ... ok
test shuffle_one_draw_says_nothing_whatever_the_state_claims ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

#### What the model records now

`Shuffle::Observed` carries `distinct_orders` beside `draws`.

⛔ **A COUNT, never the orders themselves.** A profile is one connection, and
carrying the other connections' orders inside it would fold a set of captures
into one, which [`../docs/inherited-claims.md`](../docs/inherited-claims.md)
section 8 says never to do.

⛔ **Fewer than two is a contradiction and `Profile::check` refuses it.** A state
that says the order differed while reporting one order is a claim its own field
denies, and a consumer reading the state alone would take one draw for the shape.

⚠ **Defaulted on the way in**, so a profile written before the field existed
still reads, and 0 then means "not recorded". ⛔ Such a profile claiming
`observed` is still refused: an absent count cannot support the claim either.

#### ⛔ The second half is a finding, not a defect, and the caller supplies the fact

Eight captures of one binary that produced a single order is a reason to doubt
the capture. ⚠ But whether a FAMILY shuffles is a fact about a browser rather
than about one connection, so `Options.expects_shuffle` carries it and a check
that assumed it would report every non-shuffling browser as broken.

| what the caller says | what the check does with `Fixed { draws: 8 }` |
| --- | --- |
| nothing | ⭐ passes. Not stated is not false. |
| `Some(false)` | passes |
| `Some(true)` | ⛔ a finding naming the draw count and saying a shuffling browser that never moved is a reason to doubt the capture |

#### ⚠ What is not asserted, and it is the interesting part

⛔ **No profile in this corpus records a shuffle observation yet.** The harness
writes `Shuffle::Unknown` at `crates/b-ids-harness/src/hello.rs`, because it
parses one hello at a time and the comparison across a run's connections is not
wired into the capture path. ⭐ The model can now say what a sample showed; the
capture path has not yet been asked to fill it in, and that is one connection's
worth of work in `b_ids_corpus::profile_from` rather than a design question.

---

## SCHEMA-11. The multipart boundary, which is a per-browser surface nobody listed

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 8
**Category** schema, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-02T04:00:00Z.** `http.multipart_boundary` records the shape a
browser generates, never a drawn value: a literal prefix, the length of the
random part and its alphabet, with a matcher that checks all three.

```text
$ cargo test -p b-ids-schema multipart -- --nocapture
     Running tests\multipart.rs (target\debug\deps\multipart-edc0b06126b3ecf9.exe)
running 8 tests
test multipart_the_length_is_checked_as_well_as_the_prefix ... ok
test multipart_a_character_outside_the_alphabet_does_not_match ... ok
test multipart_a_boundary_from_another_browser_does_not_match ... ok
test multipart_a_pattern_with_no_random_part_is_a_constant_and_is_refused ... ok
test multipart_sixteen_boundaries_of_one_browser_all_match_its_pattern ... ok
test multipart_every_alphabet_names_itself_and_is_checkable ... ok
test multipart_no_profile_in_this_tree_claims_a_boundary ... ok
test multipart_the_published_schema_carries_the_pattern ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

#### ⛔ The field is absent everywhere in this corpus, and a test holds that

**This project has not captured a form submission from any browser.** The two
patterns in the test are inherited by reading somebody else's client at a named
commit, and [`../docs/inherited-claims.md`](../docs/inherited-claims.md) is
where a value this project did not measure lives. ⛔ Nothing from there is
published as data, so `http.multipart_boundary` is `None` in every profile and
in both fixtures, and `multipart_no_profile_in_this_tree_claims_a_boundary`
is what stops that changing quietly.

⭐ **What IS measured here is the matcher**, against boundaries generated to the
recorded shape.

#### The pattern, and the three things it carries

| | |
| --- | --- |
| `prefix` | the literal text every boundary from this browser starts with |
| `random_len` | how many characters follow it. ⛔ The schema's minimum is **1**: a zero-length random part records a constant, which is exactly what this field exists to avoid |
| `alphabet` | `lower-hex`, `upper-hex` or `alphanumeric`. ⚠ An enum rather than a literal character set, because the value is compared across profiles and a free string fails silently on an ordering |

#### ⛔ Both halves of the match, on the same value

A pattern that only asserted the prefix would match a boundary from any browser
whose prefix happens to agree; one that only counted would match a prefix that
does not. ⚠ **The hyphen is the case that matters**: it appears in the prefix, so
a matcher reading the whole string rather than the tail would accept it inside
the random part.

#### ⚠ The fixtures are deterministic on purpose

A test that drew randomly would pass or fail on a seed, and a matcher checked
against one lucky draw is a matcher nobody has checked. The sixteen boundaries
walk the alphabet instead, and the test asserts they are sixteen DISTINCT
strings, which is what makes the set a sample rather than one value repeated.

#### ⚠ What is still unmeasured, and it is the whole field

⛔ **Whether Chrome `151.0.7922.76` generates the shape recorded here has not
been checked by this project.** The harness captures a navigation; a multipart
body needs a form submission, which needs a page that posts one. That is a
capture-path change and it is not in this entry.

---

## SCHEMA-12. The six formats that need a decoder as well as an encoder

**Source** ruled by the operator 2026-09-01, splitting `SCHEMA-08`
**Category** schema, **Priority** P2, **Effort** L, **Status** open

### Problem

`SCHEMA-08` publishes five formats this tree can round-trip with what it already
has. The six below are the ones a consumer is as likely to ask for, and not one
of them can be delivered without adding a dependency or writing a parser this
project then owns forever.

⛔ **The cost is the entry.** A format written and never read back is a format
nobody has checked, and `SCHEMA-08`'s acceptance is a round trip precisely
because a generator with no reader drifts silently.

### Premise

⭐ **Counted against the tree rather than believed.** Of the nine formats
`SCHEMA-08` named, five are writable and readable with the standard library plus
`serde_json`, which this workspace already depends on. The other six have no
reader here at all.

⚠ **What is NOT measured** is what each dependency costs in build time, in
supply chain, or in the minimum supported Rust version, and filling that in is
part of this entry rather than a preface to it.

### Approach

⭐ **State the trade per format, with its cost, and let the operator rule on one
table rather than on six questions.** ⛔ Do not pick a set and start writing.

| format | what a writer needs | what a reader needs | the cheaper alternative |
| --- | --- | --- | --- |
| YAML | a serialiser | a parser | publish YAML as a lossy view with no round trip, and say so in the support matrix |
| TOML | a serialiser | a parser | the same |
| SQLite | a library, or generated SQL text | a library | ⭐ generate a `.sql` DUMP rather than a binary database. Text round-trips through the `sqlite3` any host already has, and this project ships no library. |
| CBOR | a codec | the same codec | drop it. MessagePack serves the same consumer. |
| MessagePack | a codec | the same codec | drop it |
| Protobuf | a definition and a code generator | the generated decoder | ⭐ publish the definition with no binaries. A typed consumer generates its own decoder and the corpus owes nothing. |

⚠ **Two of those alternatives cost nothing and deliver most of the value**, and
this entry exists to make that comparison visible rather than to assume it.

⛔ **Whatever is chosen extends `SCHEMA-08`'s one generator.** A second
generator is a second answer to what a profile is.

Must not: add a dependency without recording what it costs, and must not write a
parser this project cannot keep correct.

### Prove

```bash
sh scripts/common/check-formats.sh --require-rows yaml,toml,sqlite,protobuf
```

Passing means: every format the ruling accepted is generated from the canonical
corpus and read back, each lossless one to byte-identical canonical JSON; every
format the ruling declined is absent from the generator AND named in the support
matrix as declined, with its reason; and the check exits non-zero when a required
row produced no output at all.

---

## SCHEMA-13. The published schema accepts 999 for a byte

**Source** found while reading the published schema, 2026-09-01; ruled the same day
**Category** schema, **Priority** P1, **Effort** S, **Status** done

### Problem

`u8`, `u16` and `u32` are each a bare `{"type": "integer"}` in
[`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json),
so a consumer validating a profile against the PUBLISHED schema accepts a value
the Rust type cannot hold. A profile claiming 999 in a byte-wide field satisfies
the contract this project publishes and fails the one it implements.

### Premise

⭐ **Read from the schema file rather than believed.** Every integer field in it
is unbounded today.

### Approach

⭐ **The schema and its checker gain `minimum` and `maximum` in ONE change**, and
that is not a convenience: the checker already refuses a keyword nothing
enforces, so the two cannot land apart.

Bound all three widths at once, derived from the Rust types rather than typed by
hand, so the bound and the type cannot drift.

Must not: bound one field and leave its siblings, which produces a schema that
looks checked and is not.

### Prove

```bash
cargo test -p b-ids-schema bounds -- --nocapture
```

Passing means: every integer field in the published schema carries a `minimum`
and a `maximum` that match its Rust width; a profile with 999 in a byte-wide
field is refused by the published schema as well as by the type; and a field
added without a bound fails the test.

### Closing

**Closed 2026-09-02T02:55:00Z.** Every integer field in the published schema
carries a `minimum` and a `maximum`, the checker enforces both, and the bounds
are derived in the test from the Rust widths rather than typed beside them.

```text
$ cargo test -p b-ids-schema bounds -- --nocapture
     Running tests\bounds.rs (target\debug\deps\bounds-dc604ce60633f46c.exe)
running 4 tests
test bounds_every_integer_field_carries_the_bound_its_rust_type_gives ... ok
test bounds_every_integer_field_is_in_this_table ... ok
test bounds_a_negative_value_is_refused_where_the_rust_type_is_unsigned ... ok
test bounds_the_published_schema_refuses_999_for_a_byte ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
exit=0
```

#### What is bounded, and where the bound came from

| schema node | Rust type | minimum | maximum |
| --- | --- | --- | --- |
| `$defs/u8` | `u8` | 0 | 255 |
| `$defs/u16` | `u16` | 0 | 65535 |
| `$defs/u32` | `u32` | 0 | 4294967295 |
| `tls.padding_len` | `Option<u16>` | 0 | 65535 |
| `http2.connection_window` | `Option<u32>` | 0 | 4294967295 |
| `captured.acquisition.bytes` | `usize` | 0 | 9007199254740991 |
| `raw.record_layer.bytes_arrived` | `usize` | 0 | 9007199254740991 |

⚠ **The two `usize` fields are bounded by JSON rather than by Rust**, and the
schema says so in its own description. `usize` is wider than that on every host
this runs on, but a consumer reading a larger value out of a JSON number reads a
different number than was written, so the contract's real ceiling is the largest
integer a JSON number carries exactly. ⛔ Bounding them at `usize::MAX` would
publish a contract this format cannot honour.

#### ⭐ The guard against the next field

`bounds_every_integer_field_is_in_this_table` walks the schema and fails on an
integer node that has no row in the test's table. ⛔ **So a field added without a
bound cannot pass by being forgotten**, which is the failure mode of every
hand-maintained bound table.

#### Both halves landed together, and that was not optional

The checker in `crates/b-ids-schema/tests/support/mod.rs` refuses a keyword it
does not implement, so adding `minimum` and `maximum` to the schema without
teaching the checker would have failed `check_schema_is_supported`. ⭐ The two
cannot drift apart.

#### ⛔ The guard was seen to fail

`"maximum": 255` removed from `$defs/u8` and nothing else changed:

```text
999 in a byte-wide field is refused by the published schema: []

failures:
    bounds_every_integer_field_carries_the_bound_its_rust_type_gives
    bounds_the_published_schema_refuses_999_for_a_byte

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
exit=101
```

---

## SCHEMA-14. A credential's presence is a fingerprint, and it is currently a hole

**Source** ruled by the operator 2026-09-01, from open question 4 of the previous session
**Category** schema, **Priority** P1, **Effort** M, **Status** done

### Problem

`cookie` and `authorization` are dropped entirely today, name and all. ⛔ That
leaves a hole in a recorded header ORDER and nothing marks it as a hole, so a
consumer reading the order believes it has the whole sequence and does not.

⭐ **Whether the header was sent, and where in the order, is a fingerprint signal
in its own right**, and it carries no secret.

### Premise

⭐ **Read from the code**: the credential filter removes the entry, so the
recorded order closes over the gap and nothing downstream can tell.

### Approach

Record the header as **present, in its wire position, with the value absent by
construction**. A schema field says so explicitly rather than leaving a reader to
infer it from a missing value.

⛔ **The value never appears, on any surface, including the raw block.**
`Raw::check` already refuses that and stays exactly as it is. This entry adds a
way to say "a header was here"; it adds no way to say what was in it.

⚠ The change reaches three places and they land together: the model, the capture
path that builds the header set, and the validator check that reads header order.

Must not: introduce any mode, flag or option under which the value is retained.
⛔ A model whose natural form can carry a credential is the shape that one day
publishes one.

### Prove

```bash
cargo test -p b-ids-schema credentials -- --nocapture
```

Passing means: a capture carrying `cookie` produces a profile whose header list
holds an entry at that position marked as a withheld credential with no value
field at all; the serialised profile contains none of the credential's bytes,
asserted by searching the serialised text; and a profile hand-built with a value
on such an entry is refused.

### Closing

**Closed 2026-09-02T03:15:00Z.** A credential header is recorded as present, in
its wire position, with the value absent by construction and a field that says
so. ⛔ Nothing here added a way to record the value, and three refusals were
added that did not exist.

```text
$ cargo test -p b-ids-schema credentials -- --nocapture
     Running tests\credentials.rs (target\debug\deps\credentials-918349ee6bd1880c.exe)
running 7 tests
test credentials_are_recorded_as_present_in_their_wire_position ... ok
test credentials_a_profile_carrying_a_credential_value_is_refused ... ok
test credentials_the_marker_on_an_ordinary_header_is_refused ... ok
test credentials_a_credential_without_the_marker_is_refused ... ok
test credentials_carry_no_value_field_at_all ... ok
test credentials_an_ordinary_header_carries_no_marker ... ok
test credentials_the_published_schema_carries_the_marker ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
exit=0
```

#### ⭐ Four doors, and the change reached every one of them

⚠ **The entry said three places and there were four**, and the fourth was found
by grepping for the filter rather than by recalling the list.

| door | what it does now |
| --- | --- |
| `HeaderSet::record`, the model | keeps the entry with `value: None` and `withheld: true`. ⛔ There is no branch in it that can put a credential's value into the field. |
| `b_ids_harness::h2::record_fields` | keeps the field with `value: None`, whatever the policy. The decoder still stores it, because the dynamic table has to see every field the encoder inserted or every later index resolves wrongly. |
| the HTTP/1.1 reader in `listener.rs` | pushes the name into `header_names` and pushes nothing into `header_values`. ⚠ The two lists were never parallel: `header_values` is empty under the default policy while `header_names` is full. |
| `Profile::check`, the read path | ⭐ **three refusals rather than one.** A capture-time filter cannot hold a rule about a FILE. |

#### ⛔ What is refused, and each is its own failure

| refused | why it is not the same as the others |
| --- | --- |
| a credential entry carrying a value | the rule this entry must not weaken. It is what the old filter enforced by deletion. |
| a credential entry with no `withheld` marker | it would read as an ordinary header whose value simply was not recorded, and those are different facts about what the wire carried |
| `withheld` on a name that is not a credential | it would claim a value was suppressed where none was, which reads as a hole in the record that is not there |

#### What a consumer sees

`withheld` is absent where it is false, so a profile written before this change
reads unchanged and a profile whose headers carry no credential gains no keys.
⛔ The published schema declares it as a boolean with `additionalProperties:
false` still in force, so a consumer validating against the file this project
publishes accepts the marker and nothing else new.

#### ⚠ Four tests changed their assertions, and none changed its title

`header_privacy_values_on_still_drops_cookie_and_authorization`,
`header_privacy_the_credential_filter_is_case_insensitive`,
`header_privacy_header_order_is_kept`,
`hpack_drops_a_credential_the_dynamic_table_had_to_see`,
`listener_reads_a_cleartext_request_when_that_is_the_surface`,
`switches_header_values_still_drops_a_credential` and
`raw_backstop_rebuilds_a_cleartext_request_through_the_one_construction_path`
all asserted that the NAME was gone. ⭐ Each now asserts that the name is there
and the VALUE is not, which is the rule this entry replaced the old one with.
⛔ None was deleted: a test removed here would be a door that stopped being
watched.

