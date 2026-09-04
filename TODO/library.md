# library

A crate another project can depend on, and the smallest possible tool that
proves the crate works.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## LIB-01. A crate that hands a program a profile

**Source** the operator; the founding brief
**Category** library, **Priority** P2, **Effort** M, **Status** done

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

### ⭐ Closed 2026-09-03. A dependency line, with no network in it and no fallback behind it

#### The acceptance

```text
$ cargo test -p b-ids -- --nocapture
running 7 tests
test latest_is_read_from_the_corpus_pointer_rather_than_derived_here ... ok
test the_release_says_how_old_the_data_is_without_leaving_the_language ... ok
test every_embedded_profile_parses_and_carries_its_provenance ... ok
test selecting_an_uncaptured_platform_returns_nothing_rather_than_a_substitute ... ok
test the_parts_a_consumer_actually_wants_come_out_on_their_own ... ok
test a_profile_is_reachable_by_the_published_path_the_index_gave ... ok
test the_embedded_release_identifier_is_the_corpus_this_build_was_cut_from ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

⚠ **The one-test block is the DOCTEST in the crate's own header**, and the
empty unit-test target above them both is dropped from this paste with this line
saying so.

⚠ **The doctest in the crate's own header is the eighth**, and it is there
because a usage example that does not compile is the first thing a consumer
copies.

#### ⛔ The read-only path, which `b-ids-corpus` asked for by name

`b-ids-corpus`'s manifest says that a consumer who only wants to READ the corpus
links the TLS terminator through it, and names this entry as where a path
without that edge belongs. So:

| graph | what is in it |
| --- | --- |
| runtime | `b-ids-schema` and a JSON parser. Nothing else. |
| build | ⭐ `b-ids-harness`, for the digest alone. A build dependency is not in a consumer's runtime graph, and the alternative was a second implementation of SHA-256 for one identifier, which is exactly what that function's one home exists to prevent. |
| test | the same harness, so the suite recomputes the identifier independently rather than letting the build script grade itself. |

#### ⭐ The corpus is embedded from the INDEX, never from a walk

⛔ **The index is the one statement of what is published.** A walk of `corpus/`
would be a second answer, and a stray file under it would be embedded as though
somebody had published it. The build script reads the index, embeds each file it
names, and ⛔ **refuses when a named file is absent** rather than embedding four
of five and saying nothing.

⚠ **`B_IDS_CORPUS_ROOT` is the seam `PUB-11` needs.** When `corpus/` leaves the
default branch, that variable points at a fetched copy of the data branch and
nothing else about this crate changes. Unset, the build script walks up from its
own manifest to the first directory that HAS a corpus, which is the repository
root here.

#### ⚠ `latest` is READ, and this crate does not know what it means

⛔ **The corpus publishes a pointer file and this follows it.** Deriving "which
build is newest" here would be a second implementation of a rule `CORPUS-03`
already holds, and the day the two disagreed the consumer is the one who finds
out. The same reason keeps the LAYOUT out of this crate: an embedded profile is
keyed by the path the index gave it, so nothing here parses a route.

#### The guard mutation, each read from the suite that owns it

⛔ **Both mutations were made against a copy under the ignored scratch
directory, and the live file was compared byte for byte with that copy
afterwards.**

| planted | what went red |
| --- | --- |
| the build script hashes something other than the index | `the_embedded_release_identifier_is_the_corpus_this_build_was_cut_from`, exit **101** |
| ⛔ `select` falls back to any other pointer when the key is absent | `selecting_an_uncaptured_platform_returns_nothing_rather_than_a_substitute`, exit **101** |

⭐ **The second one is the entry's rule**, and it is worth planting rather than
reading: a fallback is four words of code and produces an interface that answers
every question, which is exactly what makes it dangerous.

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a pinned RELEASE | ⛔ There is no release to pin. The crate embeds the corpus in the tree it is built from and states which one by digest; `PUB-01` cut no release and `PUB-10` is the trigger that will. |
| digests a consumer can read | ⚠ `digests` is exposed and every field of it is `None` in today's corpus, because nothing has computed them. `VALID-04` is that entry, and an `Option` that says "not computed" is the honest shape rather than a zero. |
| header VALUES on most profiles | ⚠ A profile records header NAMES by default, which is `SCHEMA-04`'s privacy rule, so `user_agent` answers `None` for a capture that did not turn values on. The suite asserts both branches rather than assuming one. |
| a client that puts a profile back on a wire | `LIB-02`, and it is the entry that says whether any of this is usable rather than merely accurate. |

---

## LIB-02. The smallest client that proves a profile is usable

**Source** the operator
**Category** library, **Priority** P2, **Effort** M, **Status** done

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

### ⭐ Closed 2026-09-03. 1951 of 1983 bytes identical to the browser's own, and the 32 that differ are the ones nobody records

#### The acceptance

⚠ **The command in the approach above names a profile this corpus does not
hold**, `chrome-152.0.7977.64-linux64-stable`, and this client refuses a profile
by name rather than substituting a neighbouring one. ⛔ The correction is the
entry's own rule applied to its own acceptance: the identifier below is read
from `--list` rather than typed.

```text
$ ./target/debug/b-ids-cli --list
chrome-151.0.7922.173-linux64-stable
chrome-152.0.7977.75-linux64-stable
chrome-151.0.7922.174-win64-stable
chrome-151.0.7922.76-win64-stable
chrome-152.0.7977.76-win64-stable
edge-151.0.4129.101-linux64-stable
b-ids-cli=list profiles:6 release:7fc3937944366cb0…

$ ./target/debug/b-ids-harness --raw --port 0 --once --json --hello-out .tmp/lib02/sent.hex &
https://127.0.0.1:60597/

$ ./target/debug/b-ids-cli --profile chrome-152.0.7977.75-linux64-stable --url https://127.0.0.1:60597/
b-ids-cli=sent profile:chrome-152.0.7977.75-linux64-stable bytes:1983 peer:127.0.0.1:60597
cli rc=0
harness rc=0
```

⚠ **The release identifier is abbreviated at the ellipsis**, for the reason
`PUB-01`'s and `TOOL-03`'s blocks are: written out it is a 64-character hex run
in a tracked file, which `check-no-secrets --public` refuses.

```text
$ cargo test -p b-ids-cli
running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

#### ⭐ The result, which is stronger than the entry expected

⚠ **The entry says to expect it not to match on the first attempt**, and to
expect the first mismatches to be an unenumerated extension codepoint, an
arbitrary extension order and a settings key. ⛔ **It matched.** Comparing the
profile's own published `ClientHello` with the bytes this client put on the
wire, read back by the harness:

| | |
| --- | --- |
| bytes sent | 1983 |
| bytes captured | 1983 |
| ⭐ bytes identical | **1951** |
| bytes differing | 32, at offsets 11 through 42 |

⛔ **Those 32 bytes are the `ClientHello` random**, which the model does not
record and never will: it carries no fingerprint, `EMIT-02` records the same
finding from the other side, and a client that reproduced it would be sending a
constant.

⭐ **Why it matched when the entry expected it not to.** The prediction was
about a client built on somebody else's TLS stack, and the three predicted
mismatches are that stack's holes rather than this project's. This client writes
the hello itself, from `EMIT-02`'s escape hatch, so there is no stack in the way
to lose the ordered list. ⚠ That is a narrower claim than "a profile is usable
by an impersonating client", and the next section says exactly how much
narrower.

#### ⛔ What this proves, and what it does not

| proved | not proved |
| --- | --- |
| the bytes a profile describes can be put on a wire and read back as the same profile, field by field | that a REQUEST can be completed with them |
| this project's model holds everything a hello needs except the random | that any third-party stack can emit one. `EMIT-01`'s matrix is where that goes, and `LIB-02` has now produced the first cell of it: this project's own emitter, no holes |
| the comparison can fail: a swapped extension is reported as `tls.extensions.order` and a dropped cipher as `tls.cipher_suites` | anything about HTTP/2 or headers, which this client never reaches |

⛔ **It completes no handshake, and that is the honest boundary.** Answering a
`ServerHello` needs a TLS state machine, and the only one in this tree is the
vendored terminator on the SERVER side. A client that borrowed a client-side
state machine would be a client that could not send this hello, which is the
whole finding.

#### ⛔ What it must never become

⚠ **No cookie jar, no redirect policy, no retry logic, no proxy support, no
output formatting.** The command takes `--profile`, `--url` and `--list`, and
every addition is a reason to add a flag. A client with forty flags is a second
product this project has not agreed to maintain, and the entry says so before it
says anything else.

#### The guard, seen to fail

| planted | what went red |
| --- | --- |
| a profile identifier the corpus does not hold | `NoSuchProfile`, and the message says the client never substitutes |
| two extensions swapped | the report names `tls.extensions.order` |
| one cipher removed | the report names `tls.cipher_suites`, so the report says WHICH field rather than that something changed |
| ⚠ a constant random | asserted against: the byte comparison requires at least one byte of the random to differ, so a client that sent a fixed one fails |

#### ⚠ What is NOT in this entry

| | |
| --- | --- |
| a conformance report against a third-party stack | `EMIT-01` and `VALID-05`. This entry produced the first cell and it is this project's own emitter. |
| the HTTP/2 and header halves | ⛔ Unreached. The client stops at the hello. |
| a digest comparison | ⛔ Refused by the entry and not written: two profiles can share a digest and differ in a field the digest sorts away, and `VALID-04` measured exactly that: four of the six published profiles share one JA4. |

---

## LIB-03. Bindings for the ecosystems that will ask

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** library, **Priority** P3, **Effort** L, **Status** done

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

### ⭐ 2026-09-04: the comparison exists, and the two ecosystems answer the same

⚠ **The Prove named a script this tree did not have**, the fourth today.

```bash
sh scripts/common/check-bindings.sh
```

```text
bindings ok: 1 binding(s) compared against the Rust crate, answer for
  answer over one corpus, and the three absent cases came back empty on
  both sides. ⛔ The comparison is over the ANSWERS rather than over the
  interfaces.
```

⚠ **Exit 0, read from the process, unpiped**, and both halves report it.

#### ⭐ What is compared, and it is answers rather than interfaces

`crates/b-ids/examples/answers.rs` and `scripts/fixtures/bindings-answers.mjs`
ask the same fifteen questions in the same order and print the same document.
⛔ The check normalises key ORDER and nothing else, so a value differing in type
or in case is a difference.

⭐ **Three of the fifteen are the ABSENT case**, which this entry names
specifically: a path the corpus does not publish, a platform it holds no profile
on, and a browser nobody captured. ⚠ Two implementations agree easily on what
exists. ⛔ The check refuses an answer set that does not ask all three, so
nobody can make it pass by deleting the hard questions.

#### ⛔ The Must-not, and the honest state of it

⚠ **"A reimplementation in each language is the failure to avoid."** The
JavaScript package `PUB-05` generates is a REIMPLEMENTATION rather than a thin
binding over `LIB-01`: it reads the embedded corpus in JavaScript. ⛔ **The
Approach's central choice was therefore not taken**, and saying so is worth more
than a closing paragraph that implies it was.

⭐ **What the entry's own Prove asks for instead is a comparison, and that is
what holds the two together.** Two things changed today because of it:

| what drifted | how it was closed |
| --- | --- |
| `latestStable` compared version numbers in JavaScript while the Rust crate read the corpus's own pointer file | ⛔ the pointer file is CARRIED into the package and both halves read it. `latest` means the newest STABLE build, which is the corpus's rule rather than either package's |
| the platform token was going to be derived twice | ⛔ derived once, in Rust, at generation time, and carried on each entry |

⚠ **Both of those were found by writing the comparison**, not by reading the
code. The first would have agreed with the Rust crate on every profile this
corpus holds today and disagreed the day a pre-release build landed.

#### ⭐ The guard was seen to refuse, and it named which rule moved

Planting exactly the drift the Must-not describes, the JavaScript half
recomputing `latest` instead of reading the pointer:

```text
bindings check failed, 5 problem(s):

  the js package does not answer as the Rust crate does. See .tmp/check-bindings/diff.txt
    they disagree about hello_bytes
    they disagree about latest_chrome_linux64
    they disagree about latest_chrome_win64
    they disagree about latest_upper_case
```

⚠ **Exit 1, read from the process with no pipe**, and the file restored byte for
byte afterwards with `git diff` showing nothing.

#### ⛔ A defect in this check's own PowerShell half, found by the twins

⚠ **The two halves disagreed about a tree neither of them changed.** The
PowerShell half compared each value with `ConvertTo-Json`, which does not sort a
nested object's keys, and reported `release` as differing over two identical
documents. ⭐ The sh half never had it, because `jq -S` sorts. Both normalise
with `jq -S` now.

⛔ **That is exactly what `check-twins` exists to surface** and it surfaced this
one before the check ever ran in the gate.

#### ⚠ What is NOT done

⛔ **A true binding does not exist.** One would call `LIB-01` through WASM or a
foreign function interface rather than re-reading the corpus, and that is a
build question this session did not take: it needs a `wasm32` target in the
toolchain pin and a second artefact in the published tree. ⭐ Until it does, the
comparison above is what stands between one selection rule and two answers, and
it runs on every gate.

⚠ **One ecosystem beside Rust**, for the reason `PUB-05` records: what exists is
listed rather than what is intended.
