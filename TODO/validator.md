# validator

Given a profile, answer one question: could a real browser have sent this?
Pure logic over the model. No network, no browser.

⭐ This is the highest-value component in the project and the one nobody has
built, and it can be finished before a single capture exists.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## VALID-01. The coherence checks, as a library and a command and a schema

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P0, **Effort** M, **Status** done

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

### Closing

**Closed 2026-08-31.** Eight checks, twenty-six tests, and a command that
returns three different exit codes for three different facts.

```text
$ cargo test -p b-ids-validator -- --nocapture
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

### ⭐ Three outcomes, because a check that cannot run has not passed

⛔ **This is the design decision the entry did not anticipate and it changes
what the component reports.** Most of these checks read a header VALUE, and
`SCHEMA-04`'s default capture policy records header NAMES only. A validator
that reported those as passes would be green over a profile it had not read,
which is the "step that exits 0 having done nothing it was asked to do" defect
in the one component whose whole job is refusing things.

So every check answers `Passed`, `Failed` or `NotCheckable`, the last carrying
the reason, and the command's exit code separates them: 0 clean, 1 a check
refused, 2 nothing could be checked at all. ⚠ It still does not WARN, which is
what the entry forbids: `NotCheckable` is a statement about what was read, not
a softened refusal.

### ⛔ Check 4 cannot be decided within one profile, and it says so

The entry describes check 4 as "the `ClientHello` came from a build whose major
matches the claimed one". **Deciding that inside one profile needs a per-build
corpus of handshakes to compare against, and this project has captured none.**
Answering it anyway would be inventing the comparison.

⭐ **So it ships in the form that CAN run today**, across a set of profiles:
`shared_handshakes` reports two profiles of one browser claiming different
majors and carrying a byte-identical TLS half, because at most one of them was
measured. That is exactly the shipped violation the sweep located, where five
modules of one reference database return a neighbour's fingerprint wholesale
beside their own User-Agent. `VALID-02` is what runs it over that tree.

⚠ The within-profile leg reports `NotCheckable` naming what would make it
checkable, and still refuses outright a hello with no extensions at all.

### The eight, and what each was planted with

| check | the contradiction its test plants |
| --- | --- |
| version | a Chrome 151 User-Agent on a profile claiming 152, and separately a brand list claiming 151 |
| platform | a Windows User-Agent on a Linux capture; a `macOS` hint; a mobile hint on a desktop capture **and** the reverse |
| brand | an unbranded build whose list carries a vendor entry, and a branded one whose list does not |
| handshake | two majors sharing one byte-identical TLS half |
| grease | a position list disagreeing with the hello; one draw reused across both slots; a shuffle state claimed from one draw |
| encoding | a profile advertising `zstd` to a consumer that decodes `gzip` only |
| absence | a stack that cannot omit a setting, and separately a target with four holes at once |
| provenance | a `vendor` field while publishing, and an unreasoned `substituted` field either way |

⛔ **Every test asserts the failure AND that the same check passes over the
unmodified fixture.** One without the other proves half of it: a check that
refused everything would pass a test that only planted a defect.

### Driven, not only tested

```text
$ cargo run -q -p b-ids-validator -- .tmp/profile.json
.tmp/profile.json: ok    version
.tmp/profile.json: ok    platform
.tmp/profile.json: ok    brand
.tmp/profile.json: SKIP  handshake -- deciding whether this hello came from a 152 build needs a per-build corpus to compare against, and none exists yet. b-ids-validator::shared_handshakes is the form that runs across a set of profiles today
.tmp/profile.json: ok    grease
.tmp/profile.json: SKIP  encoding -- the caller did not say what the consuming client can decode
.tmp/profile.json: SKIP  absence -- the caller named no target stack
.tmp/profile.json: ok    provenance
exit=0

$ cargo run -q -p b-ids-validator -- --publishing --decodes gzip .tmp/broken.json
.tmp/broken.json: FAIL  version: http.headers.user-agent: carries major 151, and browser.major is 152
.tmp/broken.json: ok    platform
.tmp/broken.json: FAIL  brand: http.headers.sec-ch-ua: browser.branded is true and the brand list has no Google Chrome entry
.tmp/broken.json: SKIP  handshake -- deciding whether this hello came from a 152 build needs a per-build corpus to compare against, and none exists yet. b-ids-validator::shared_handshakes is the form that runs across a set of profiles today
.tmp/broken.json: ok    grease
.tmp/broken.json: FAIL  encoding: http.headers.accept-encoding: advertises deflate, which the consuming client cannot decode
.tmp/broken.json: FAIL  encoding: http.headers.accept-encoding: advertises br, which the consuming client cannot decode
.tmp/broken.json: FAIL  encoding: http.headers.accept-encoding: advertises zstd, which the consuming client cannot decode
.tmp/broken.json: SKIP  absence -- the caller named no target stack
.tmp/broken.json: ok    provenance
exit=1
```

⭐ **The profile it was driven against is generated rather than hand-written**,
by `cargo run -p b-ids-schema --features fixtures --example dump`, so the file
the command reads cannot drift from the types.

### The three ways it ships

- **A library**, which is what `b-ids-harness` and `b-ids-conformance` will
  call.
- **A command**, above.
- **A JSON Schema**, which is
  [`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json)
  and landed with `SCHEMA-01`. ⚠ It expresses shape rather than coherence: a
  schema cannot say that a User-Agent and a brand list disagree, which is why
  the other two forms exist.

### ⚠ What the header reader deliberately does not do

`headers.rs` answers exactly the three questions the checks ask and returns
`None` where it cannot, ⛔ never a guess. A header parser that guessed would
turn a refusal into a coin toss. Two known limits are written into its own
doc comment rather than defended against: a brand containing a comma or a
semicolon would break the split, and neither appears in any shipped brand list.

### ⚠ Still open, and this entry does not close them

- **`VALID-02`** is the run over the prior art, and `shared_handshakes` is the
  function it needs. The three violations are located at file and line and none
  has been run against yet.
- **`VALID-03`**, a family the resolver cannot produce, is the third check the
  sweep derived and it is not one of these eight.
- **`SCHEMA-04`'s privacy default and this component pull against each other**,
  and the tension is real rather than a defect: the safest capture is the one
  several of these checks cannot read. The answer is per-capture rather than
  per-model, and it is recorded here so nobody resolves it by weakening the
  default.

---

## VALID-02. Run it over the prior art, and publish what it finds

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P1, **Effort** S, **Status** done

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


### Closing

**Closed 2026-09-01T06:05:00Z.** ⭐ **Ten exhibits, in two public repositories,
each with a file, a line and the check it fails.** The reader agrees with the
three the sweep located by eye and adds two more.

```text
$ cargo test -p b-ids-validator import
running 7 tests
test import_refuses_a_corpus_it_cannot_read ... ok
test import_names_every_entry_that_returns_another_version_handshake ... ok
test import_returns_its_exhibits_in_sorted_order ... ok
test import_names_the_family_no_classifier_can_reach ... ok
test import_names_the_cipher_table_served_to_every_version ... ok
test import_report_carries_the_check_name_beside_every_exhibit ... ok
test import_produces_byte_identical_output_across_runs ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
exit=0
```

And the report itself, which is the result:

```text
$ cargo run -p b-ids-validator -- import references --report
b-ids-validator import report/1

Kikobeats/https-tls
  src/headers-order.json:119  unreachable-data
    the header order for "edge" is data no caller can reach: the classifier returns "chrome", "firefox", "safari" and "edge" is not one of them
  src/index.js:59  handshake
    the chrome cipher list is commented "Chrome v92" and is the only one this library has for chrome, so every version of chrome is served it
  src/index.js:78  handshake
    the firefox cipher list is commented "Firefox v91" and is the only one this library has for firefox, so every version of firefox is served it
  src/index.js:100  handshake
    the safari cipher list is commented "Safari v14" and is the only one this library has for safari, so every version of safari is served it
apify/impit
  impit/src/fingerprint/database/chrome.rs:993  handshake
    chrome_101 claims Chrome 101 and returns chrome_100's handshake, which 5 other entries in this file also return
  impit/src/fingerprint/database/chrome.rs:1026  handshake
    chrome_104 claims Chrome 104 and returns chrome_100's handshake, which 5 other entries in this file also return
  impit/src/fingerprint/database/chrome.rs:1059  handshake
    chrome_107 claims Chrome 107 and returns chrome_100's handshake, which 5 other entries in this file also return
  impit/src/fingerprint/database/chrome.rs:1092  handshake
    chrome_110 claims Chrome 110 and returns chrome_100's handshake, which 5 other entries in this file also return
  impit/src/fingerprint/database/chrome.rs:1125  handshake
    chrome_116 claims Chrome 116 and returns chrome_100's handshake, which 5 other entries in this file also return
  impit/src/fingerprint/database/firefox.rs:444  handshake
    firefox_144 claims Firefox 144 and returns firefox_135's handshake, which 1 other entry in this file also returns

10 exhibit(s)
exit=1
```

⚠ **Exit 1, because it found something.** A command that reported violations
and exited 0 would be a command nothing downstream could act on. ⛔ Exit 2 is
reserved for a reader that went blind, which is a different fact from a corpus
with nothing wrong in it.

### ⛔ The approach said to synthesise profiles, and that would have meant inventing wire data

"Write an importer per reference database that produces profiles from their
tables, run the validator over the result." ⚠ **A `TlsHalf` has sixteen fields
of wire data and neither reference states them.** Building one would mean
inventing the bytes this project exists to measure, and the report would then
be a report about the invention rather than about the table.

⭐ **What is shared instead is the vocabulary.** Every exhibit carries the
`Check` it fails, so the report and the validator name the same thing, and
`shared_handshakes` keeps its own job over real profiles. ⚠ The title stays and
the premise stays; only the mechanism changed, and this is the correction.

### ⭐ The reader found two the sweep had not, and corrected two line numbers

| what | where |
| --- | --- |
| a sixth entry returning another version's handshake, in a second family | `references/apify__impit/tree/impit/src/fingerprint/database/firefox.rs:444` |
| a third cipher table commented for one version and served to all of them | `references/Kikobeats__https-tls/tree/src/index.js:100` |
| two cited line numbers that were two lines out | the sweep cited 57 and 80; the comments naming the versions are at 59 and 78 |

⛔ **Every one of these was read by opening the file at the captured commit**,
which the entry required in as many words. The reader is what re-opened them.

### ⚠ What the readers are, and what they are not

They know the SHAPE of four files: a Rust module and constructor, a JavaScript
object of cipher lists, a classifier of string literals, and a JSON object of
families. ⛔ **A reference edited into a shape they do not know makes them find
nothing, and finding nothing is an error rather than a clean report.** Without
that, a shipped violation could leave the report by being reformatted.

⚠ **Only two of the nineteen reference trees are read.** A tree that is not
listed is not examined, which is a different fact from a tree with nothing wrong
in it, and the module says so where the list is.

### Mutation-proved

| what was planted | what happened |
| --- | --- |
| the module prefix the Rust reader matches, changed by one word | four tests FAILED, and the error named the reader as blind rather than reporting a clean corpus |
| ⛔ the sort removed | **every test passed.** The walk underneath is already stable on this host, so equality between two runs cannot tell a sorted answer from an incidentally stable one. A second test now asserts sortedness directly, and the same mutation fails it. |

⭐ **The second row is the one worth reading.** The acceptance asked for
"byte-identical output across runs" and a test written to those words could
never have failed. What the entry wanted was an order that does not depend on a
directory walk, and only an assertion about the order can see that.

---

## VALID-03. A family the resolver cannot produce is data nobody can reach

**Source** [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 7
**Category** validator, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-02T03:30:00Z.** `b_ids_validator::unreachable_dimensions`
walks every browser, channel and platform the corpus carries and reports each
one no resolver branch can select, naming the dimension, the value and every
profile that carries it.

```text
$ cargo test -p b-ids-validator reachable_dimensions -- --nocapture
     Running tests\reachable_dimensions.rs (target\debug\deps\reachable_dimensions-60ba3c943a3fcfa6.exe)
running 5 tests
test reachable_dimensions_a_platform_with_no_route_is_reported_too ... ok
test reachable_dimensions_the_comparison_is_on_the_route_spelling ... ok
test reachable_dimensions_every_profile_carrying_it_is_named ... ok
test reachable_dimensions_a_family_the_resolver_cannot_produce_is_reported ... ok
reachable_dimensions: 3 published profile(s), every dimension reachable
test reachable_dimensions_the_published_corpus_is_wholly_reachable ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

#### ⭐ The resolver's list is injected, and that is the design

The validator is pure logic over the model and does not depend on the driver.
⭐ **The caller names what its resolver can produce**, so the same check answers
for this project's driver, for a fixture, and for a resolver nobody has written
yet. `b-ids-driver` is a **dev** dependency of this crate, used by the
acceptance test alone.

⛔ **And the test reads `Family::all` rather than listing names.** A list of
families written in the test would be a second copy of the driver's with nothing
checking that the two agree, which is this entry's own defect one file along.

#### ⚠ The comparison is on the route spelling, and that is not cosmetic

`b_ids_corpus::route` derives a path by lower-casing `browser.name`, and the
driver reports `Chrome` while a route reads `chrome`. A check comparing the two
verbatim would report every profile in the corpus as unreachable, which is a
guard that fires on everything and therefore reports nothing.
`reachable_dimensions_the_comparison_is_on_the_route_spelling` is that case.

#### ⭐ A positive control, over the corpus this repository publishes

⛔ **A check that only ever fires on a fixture has not been shown to pass over
anything real.** `reachable_dimensions_the_published_corpus_is_wholly_reachable`
reads every profile under `corpus/v1/` and asserts the whole set is reachable:
three profiles today, and it will say so about the next hundred.

#### ⛔ It says they disagree; it does not pick

The entry's own rule, and the code says so where it lives: the data may be right
and the resolver wrong. ⚠ A check that "fixed" this by deleting a profile would
destroy a measurement to satisfy a code path, which is the wrong way round in a
project whose product is measurements.

#### ⚠ What this does NOT check

- **Whether a resolver branch that exists can actually run here.** `Family::edge`
  has a branch and Edge resolved on a hosted runner; whether it completes a
  capture is `CORPUS-02`'s business and it currently does not.
- **A dimension the corpus does not carry.** A family nothing has captured is
  absent rather than unreachable, and `check-coverage` is what reports that.

---

## VALID-04. Reference digest implementations, with published test vectors

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P2, **Effort** M, **Status** done

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

### ⭐ Closed 2026-09-03. Sixteen vectors, and not one expected value came from running this code

#### The acceptance

```text
$ cargo test -p b-ids-validator digest_vectors
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

#### ⛔ The licence question, answered before a line was written

⛔ **Ruled by this session under the operator's standing instruction, taking
what the record recommends.**
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
finding 5 splits FoxIO's licensing, and the answer follows from it:

| | |
| --- | --- |
| ⭐ JA4 itself | BSD-3-Clause, and FoxIO states it holds no patent claim. **Implemented here from the published specification**, at `references/FoxIO-LLC__ja4/tree/technical_details/JA4.md`, commit `02e78ba3ebac`. ⚠ Abbreviated in the published vector file too: a 40-character hex run is refused there by `check-no-secrets --public`, which excludes one inside a markdown code span and not one inside JSON. ⛔ No source was copied. |
| ⛔ every JA4+ member | FoxIO License 1.1, patent pending, monetisation-restricted. **Nothing in this tree computes one**, and this entry does not change that. |
| ⚠ JA3 | not implemented and not planned. It is an MD5, this tree links no MD5, and the record's own rule is to record JA3 and never assert on it. |

⚠ **One reading was taken from the reference implementation rather than from the
specification, and it is recorded as such.** The specification says the version
falls back to "the Protocol Version" and that "Handshake version (located at the
top of the packet) should be ignored", which is ambiguous about which of the two
version fields it means. `references/FoxIO-LLC__ja4/tree/rust/ja4/src/tls.rs:573`
settles it with a comment saying the field is not to be confused with the record
version. ⛔ Read to settle an ambiguity, never copied.

#### ⭐ Where the expected values came from, which is the whole point

⛔ **The entry forbids a vector whose expected value came from running one
implementation**, so the sixteen come from two places and neither is this code:

| kind | count | derived from |
| --- | --- | --- |
| `specification` | 10 | the specification's own worked example, which publishes each hash beside the list it belongs to, plus the ALPN rule's examples |
| `capture` | 6 | this project's six published profiles, derived with `jq` and `sha256sum` |

The capture derivation, which is not this project's code:

```bash
jq -r -f scripts/fixtures/ja4-derive.jq corpus/v1/chrome/stable/linux64/152.0.7977.75.json
```

⭐ **And the two agree where they overlap.** Every Chrome and Edge capture in the
corpus offers the specification's own example cipher list, so its published
`8daaf6152771` is reproduced from this project's captures by `jq` and
`sha256sum`, and then again by the Rust implementation. Three paths, one value.

#### ⛔ The ALPN rule reads as a contradiction until it is stated per BYTE

⚠ **The specification lists eight ALPN examples and one of them refutes the
obvious reading.** `0x30 0xAB 0xCD 0x31` is given as `01`, which no hex-of-the-
whole-value rule produces. The rule that reproduces all eight is:

- if the first AND last bytes are ASCII alphanumeric, use those two characters;
- otherwise use the FIRST character of the first byte's hex and the LAST
  character of the last byte's hex.

⭐ Under that reading `0x30 0xAB 0xCD 0x31` is not a hex case at all: `0x30` is
`0` and `0x31` is `1`, both alphanumeric, so the answer is `01` by the ordinary
rule. ⚠ It reads as a counter-example and is a contrast case.

#### ⛔ A finding: the model cannot hold three of the specification's examples

⚠ **`TlsHalf::alpn` is a `Vec<String>`, so a protocol whose bytes are not UTF-8
cannot be represented at all.** Five of the eight ALPN examples carry `0xAB` or
`0xCD`, and three of those are unreachable through the model. The vector file
carries the five that are representable and this entry records the other three
rather than pretending the branch is covered.

⛔ **Not repaired here.** Changing `alpn` to a byte vector is a schema change
that moves every published profile's serialisation, and the corpus is
append-only. It is worth an entry when a capture ever carries one.

#### What the vectors measured, which was not the point and is worth having

⭐ **Six profiles, two distinct JA4 values, and the split is by MAJOR rather
than by platform or by browser:**

```text
chrome-151.0.7922.173-linux64-stable  t13i1515h2_8daaf6152771_806a8c22fdea
chrome-151.0.7922.174-win64-stable    t13i1515h2_8daaf6152771_806a8c22fdea
chrome-151.0.7922.76-win64-stable     t13i1515h2_8daaf6152771_806a8c22fdea
edge-151.0.4129.101-linux64-stable    t13i1515h2_8daaf6152771_806a8c22fdea
chrome-152.0.7977.75-linux64-stable   t13i1516h2_8daaf6152771_cb7bf5808d99
chrome-152.0.7977.76-win64-stable     t13i1516h2_8daaf6152771_cb7bf5808d99
```

⚠ **Edge 151 is indistinguishable from Chrome 151 at JA4**, and a Windows
capture is indistinguishable from a Linux one at the same major. ⛔ That is one
digest over six captures and it is not a conclusion about platform independence:
`CORPUS-02`'s premise says the same thing and says it needs a matrix. What it
does establish is that JA4 hides the difference `VALID-01`'s handshake check
would need, which is exactly why the raw hello is published beside it.

⚠ **`i` rather than `d` in every one**, because every capture here is against an
address rather than a name, so no `ClientHello` in this corpus carries SNI.

#### ⭐ The vectors are published

`publish::build` copies `vectors/ja4/v1.json` verbatim into the tree both
surfaces take, so the release archive and the data branch carry them. ⛔ Copied,
never regenerated: a build that recomputed a vector would publish this
implementation's own answer as the thing it is checked against.

```text
$ sh scripts/common/check-release.sh --dry-run --json
{"schema":"check-release/1","files":198,"bytes":673814,"cases":11,"tags":0,"archive":"ok","problems":0}
```

#### The guard, seen to fail

⛔ `digest_vectors_a_corrupted_vector_fails` corrupts one character of the
expected value and one codepoint of the input list, and asserts the comparison
refuses both. ⚠ It also asserts the corruption changed something, because a
"corruption" that produced the same string would make the assertion vacuous.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| digests written into profiles | ⛔ The corpus is append-only and the six published profiles carry `digests: null`. They never will. A digest is derived on demand from the profile, which is what `Digests` being all-`Option` already says. |
| digest routes under `PUB-03` | ⛔ Declined. A route resolves to a value the corpus HOLDS, and the corpus holds no digest; a route computed at generation time would be exactly the plausible-looking value `PUB-03` refused. `SCHEMA-12`'s note that no digest route exists is still true. |
| JA3, and every JA4+ member | Above, with the reasons. |
| a QUIC or DTLS vector | ⚠ The prefix is `t` and nothing here can capture either transport. |

---

## VALID-05. A conformance suite for impersonating clients

**Source** the founding brief; the shape is [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 10
**Category** validator, **Priority** P2, **Effort** L, **Status** done

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

### ⭐ Closed 2026-09-04. Twenty-eight fields, three verdicts, and the third is the one that matters

⛔ **A field-level diff, never a digest comparison.**
[`../crates/b-ids-conformance/`](../crates/b-ids-conformance/) compares a
captured client against the profile it claims to be and names the fields.

```text
$ cargo run -q -p b-ids-conformance -- --fixture
conformance fixture ok: one swapped header pair is reported as exactly one
differing field, http.navigate.header_order, with 27 other field(s) conforming, 0 varying per
connection and 0 not checkable.
exit=0
```

```text
$ cargo test -p b-ids-conformance
test conformance_a_changed_setting_value_is_named_and_the_order_is_not ... ok
test conformance_a_field_one_side_does_not_carry_is_not_checkable_rather_than_agreed ... ok
test conformance_a_grease_draw_is_not_a_difference ... ok
test conformance_a_grease_value_moved_to_another_position_is_still_caught ... ok
test conformance_a_reordered_header_is_named ... ok
test conformance_a_swapped_extension_pair_is_reported_as_per_connection_not_wrong ... ok
test conformance_every_field_it_lists_is_a_field_it_compares ... ok
test conformance_a_profile_against_itself_differs_on_nothing ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

#### ⭐ Three verdicts, not two, and the third is what makes it usable

⛔ **A field a real browser varies PER CONNECTION is neither a pass nor a
failure**, and a tool that had only two verdicts would have to lie either way.

| verdict | when |
| --- | --- |
| conforms | both sides carry the field and agree |
| differs | both carry it and disagree, on something a browser holds still |
| ⭐ per-connection | both carry it and disagree, on the extension shuffle or a GREASE draw. Reported with the reason a single capture cannot conclude from it |
| not checkable | one side carries nothing for it. ⛔ Never reported as agreement, which is how a client passes on a field nobody looked at |

⚠ **The third verdict was not designed in; it was measured.** The first version
reported six differing fields between two real profiles, and three of them were
GREASE draws and a shuffle. A report naming four fields on every run is a report
nobody reads.

#### ⛔ A hole in the GREASE forgiveness, found by the test written to find it

⛔ **The first version STRIPPED GREASE before comparing**, which makes
`[GREASE, a, b]` and `[a, GREASE, b]` compare equal: a client that put its
GREASE in the wrong position would have been forgiven as a redraw.
`conformance_a_grease_value_moved_to_another_position_is_still_caught` went red
on exactly that. ⭐ It masks in place now, so the position is kept and only the
drawn value is forgiven.

#### ⭐ What the driven pass measured, which is not a test result

⛔ **Two adjacent Chrome builds across two platforms agree on every comparable
field.**

```text
$ cargo run -q -p b-ids-conformance -- --claim chrome-151.0.7922.174-win64-stable --observed corpus/v1/chrome/stable/linux64/151.0.7922.173.json
conformance: 28 field(s) compared, 26 conform, 0 differ, 2 vary per connection, 0 not checkable
  claimed   chrome-151.0.7922.174-win64-stable
  observed  chrome-151.0.7922.173-linux64-stable

⭐ Every field both sides carry agrees.
exit=0
```

⚠ **That is NOT the measurement `CORPUS-02` wants and saying so is the point.**
Those are two DIFFERENT builds, `.173` on `linux64` and `.174` on `win64`, so it
is two adjacent builds on two platforms rather than one build on two. ⛔ The
corpus still holds no build captured on both platforms, which is what would let
anyone conclude that the TLS half is platform-independent.

⭐ **What it does establish** is that the difference between those two builds and
those two platforms, taken together, is zero over 26 comparable fields. That is
a field-level result where the premise in `CORPUS-02` had only a digest match.

#### ⛔ The acceptance command in the Prove above cannot pass, and it is corrected here

⛔ **It names `chrome-152.0.7977.64-linux64-stable`, and no such profile has
ever been in this corpus.** The nearest are `152.0.7977.75` on `linux64` and
`152.0.7977.76` on `win64`. ⚠ This is the "acceptance command that cannot pass"
defect [`../docs/conventions/prose.md`](../docs/conventions/prose.md) records,
and it sat in the entry from the day it was authored.

⭐ **The tool refuses it correctly rather than guessing**, which is how it was
found:

```text
$ cargo run -q -p b-ids-conformance -- --claim chrome-152.0.7977.64-linux64-stable --observed corpus/v1/chrome/stable/linux64/151.0.7922.173.json
b-ids-conformance: no profile in this corpus has the id chrome-152.0.7977.64-linux64-stable. It holds:
  chrome-151.0.7922.173-linux64-stable
  chrome-152.0.7977.75-linux64-stable
  chrome-151.0.7922.174-win64-stable
  chrome-151.0.7922.76-win64-stable
  chrome-152.0.7977.76-win64-stable
  edge-151.0.4129.101-linux64-stable
exit=2
```

⭐ **The acceptance is the two commands at the top of this closing**, which do
run: the fixture, which builds the differ-in-one-field case over the real corpus
and asserts the report names that field and nothing else, and the suite.

#### ⚠ What this does NOT do

⛔ **It does not capture.** The observed side is a file. Standing up the harness
and pointing a client at it is what
[`../experiments/10-first-profile.sh`](../experiments/10-first-profile.sh)
already does for a browser, and a client author runs that once and this as often
as they like. ⚠ Wiring the two into one command is worth doing and is not this
entry.

⛔ **And no client has been run through it.** Every comparison above is between
two profiles this project captured. The entry's premise is that every client in
the sweep would use one of these if it existed; nothing here tests that claim.

---

## VALID-06. Diffs between adjacent versions

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** validator, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-02T06:50:00Z.** `b-ids-validator diff` reports what changed
between two profiles field by field, and says so when the two differ in more
than the version.

```text
$ cargo run -p b-ids-validator -- diff corpus/v1/chrome/stable/win64/151.0.7922.76.json \
    corpus/v1/chrome/stable/win64/151.0.7922.174.json
chrome-151.0.7922.76-win64-stable -> chrome-151.0.7922.174-win64-stable

1 field(s) differ:
  browser.version: 151.0.7922.76 -> 151.0.7922.174
exit=0
```

⭐ **That is a real result and it is the first this project can publish about a
version change.** Between Chrome `151.0.7922.76` and `151.0.7922.174` on
`win64`, every TLS field this diff compares and every header position is
identical: the only thing that moved is the version string.

```text
$ cargo run -p b-ids-validator -- diff corpus/v1/chrome/stable/win64/151.0.7922.174.json \
    corpus/v1/chrome/stable/linux64/151.0.7922.173.json
chrome-151.0.7922.174-win64-stable -> chrome-151.0.7922.173-linux64-stable

⛔ these two captures do not differ only in version, so nothing below can be
   attributed to the version alone:
     platform: win64 against linux64
     captured.resumption: not recorded against refused

1 field(s) differ:
  browser.version: 151.0.7922.174 -> 151.0.7922.173
exit=0
```

⛔ **The "must not" of this entry, working.** Two captures differing in version
AND platform AND resumption configuration cannot isolate anything, and the
warning is rendered ABOVE the change list because a reader who sees it after has
already attributed the list.

⚠ **What that pair does show, with the caveat attached**: the TLS half and the
header order of Chrome `151` are identical across `win64` and `linux64`. ⛔ Two
builds and two platforms moved at once, so it is a data point for `CORPUS-02`'s
question rather than an answer to it.

```text
$ cargo test -p b-ids-validator --test version_diff
running 5 tests
test version_diff_ignores_a_grease_draw ... ok
test version_diff_names_the_header_and_its_two_positions ... ok
test version_diff_says_when_more_than_the_version_moved ... ok
test version_diff_of_a_profile_with_itself_is_empty ... ok
test version_diff_reports_a_header_that_appeared_or_left ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit=0
```

#### ⭐ It names the change, not the digest

`http.headers.user-agent: position 1 -> position 2` is something a client author
can act on. ⛔ A header that MOVED is the kind of change only a capture finds,
and the acceptance's own case asserts the diff names the header and both
positions and reports nothing else.

| what it compares | how |
| --- | --- |
| `browser.version` | verbatim |
| `tls.cipher_suites` | ⛔ GREASE removed. It is drawn per connection, so reporting it would report a draw as a change on every pair ever taken. |
| `tls.extensions.set` | sorted, so an order shuffle is not reported as an appearance |
| `tls.key_exchange_groups`, `tls.alpn` | GREASE removed |
| every header | ⭐ by POSITION, and a header that appeared or left is reported in both directions |

#### ⚠ Exit 0 whatever it finds

A diff is a report rather than a verdict: two versions differing is what
versions do, and a command that exited 1 for it would make every pipeline treat
a normal release as a failure. ⛔ 2 is still "could not run": a path that is not
a profile, or a call that named other than exactly two.

#### ⚠ What is NOT here, and both are named rather than implied

- ⛔ **Nothing generates a diff for every adjacent pair in the corpus.** That is
  a published format and it belongs to `SCHEMA-08`'s generator, which does not
  exist.
- ⛔ **Nothing puts it in a pull-request body.** `CI-04` is that entry, and the
  point of putting the diff in a library is that the two cannot disagree when it
  does.

