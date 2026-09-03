# architecture.md

⭐ **The technical reference.** What a profile is, what the eight crates do to
one, what state a capture passes through, and where the limits are. ⛔ When two
documents in this tree disagree about a technical fact, this one settles it;
[`conventions/docs.md`](conventions/docs.md) is the rule that says so.

⚠ **It describes what the thing IS, not what the project did.** A reference page
that turns into a diary stops being read.
[`HISTORY/README.md`](HISTORY/README.md) is where a superseded explanation goes.

---

## 1. The product is a profile

A **profile** is one browser, one build, one platform, one channel, one instant.
It is a JSON file with a schema identifier, published at a route derived from its
own keys, with the `ClientHello` it was read from beside it.

⭐ **Every field carries where it came from.** The provenance map is per field,
with four kinds and no more, and it is the one thing that cannot be retrofitted:
a profile captured before it existed can never get one, because the capture is
gone. [`../crates/b-ids-schema/src/provenance.rs`](../crates/b-ids-schema/src/provenance.rs).

| half | what it holds | where |
| --- | --- | --- |
| `tls` | the `ClientHello` in wire order, with unknown codepoints kept | [`../crates/b-ids-schema/src/tls.rs`](../crates/b-ids-schema/src/tls.rs) |
| `http2` | an ordered frame sequence, the settings, the window and the priority block | [`../crates/b-ids-schema/src/http2.rs`](../crates/b-ids-schema/src/http2.rs) |
| `http` | one header set per request kind, in wire order | [`../crates/b-ids-schema/src/http.rs`](../crates/b-ids-schema/src/http.rs) |
| `raw` | the bytes, so this project's own parser can be checked against them | [`../crates/b-ids-schema/src/profile.rs`](../crates/b-ids-schema/src/profile.rs) |
| `captured` | the conditions: when, by what, under which trust and resumption configuration, with which switches | the same file |
| `digests` | ⚠ null on every published profile, and it stays null. A digest is DERIVED from a profile on demand rather than stored in it, so the append-only corpus never carries a value a later reading might correct. ⛔ JA3 is not implemented and no JA4+ member is. Section 4 says where the derivation lives. | the same file |

⛔ **The published contract is a file, not a doc comment.**
[`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json)
is what a consumer validates against, and a test asserts it agrees with the Rust
types about every integer's width.

---

## 2. Eight crates, and what each is not

| crate | what it does | ⛔ what it is not |
| --- | --- | --- |
| [`b-ids-schema`](../crates/b-ids-schema/) | the model, its refusals, the published JSON schema, and the rendered digest lists | it captures nothing and reaches no network |
| [`b-ids-harness`](../crates/b-ids-harness/) | a SERVER that accepts connections and reads what arrived: the hello, the frames, the headers. It also hashes a JA4, because SHA-256 has one home and it is here | ⛔ not a client. It never asks a browser what it sends; it reads what the browser put on a socket. |
| [`b-ids-driver`](../crates/b-ids-driver/) | resolves a browser on the machine, reports what the vendor is serving, and launches one at a URL into a profile nobody keeps | it parses no bytes |
| [`b-ids-corpus`](../crates/b-ids-corpus/) | turns a capture into a profile, publishes it at its route, verifies the whole store, and assembles the tree that leaves this repository | ⛔ it never edits a published profile. A correction is a NEW profile naming the one it replaces. |
| [`b-ids-validator`](../crates/b-ids-validator/) | pure logic over the model: could a real browser have sent this? | ⛔ it does not warn. Every check answers passed, failed, or not-checkable. |
| [`b-ids-emit`](../crates/b-ids-emit/) | turns a profile back into the bytes a `ClientHello` carries, and generates the support matrix from a run | ⛔ it refuses rather than approximating. A hello it cannot write byte for byte is a refusal naming every reason. |
| [`b-ids`](../crates/b-ids/) | hands a program a profile, with the corpus embedded at build time | ⛔ it never fetches and never substitutes. A platform this project has not captured returns nothing. |
| [`b-ids-cli`](../crates/b-ids-cli/) | the smallest client: it puts one profile's hello on a socket and stops | ⛔ not a general-purpose HTTP client, and it must never grow into one. No cookie jar, no redirects, no retries. |

⚠ **One more crate exists and is not a component yet**, and it says so in its
own first line rather than looking finished:
[`b-ids-conformance`](../crates/b-ids-conformance/) for `VALID-05`.
⭐ It exists so that the acceptance commands in [`../TODO/`](../TODO/) resolve
a target.

---

## 3. The state a capture passes through

⭐ **Read this in order. Every arrow is a place a value can be lost, and the
project's rules are mostly about one of them.**

```text
  a browser on a machine
        |  b-ids-driver: resolve, then launch into a throwaway profile
        v
  a socket to b-ids-harness
        |  the listener reads the first TLS record itself
        v
  a Capture per connection            <- one navigation is MANY connections
        |  b_ids_harness::select, PER HALF: the first cold hello, and the
        |  first connection that reached HTTP/2. They need not be the same one.
        v
  two connections, and the profile records both numbers
        |  b_ids_corpus::profile_from, with an identity read from the run
        v
  a Profile, checked by Profile::check
        |  Store::add, refusing a route that is already published
        v
  corpus/v1/<browser>/<channel>/<platform>/<version>.json
  raw/v1/...  the ClientHello beside it
```

### ⛔ The four things that go wrong at those arrows

| where | what goes wrong | what holds it |
| --- | --- | --- |
| browser to socket | the subject will not complete a handshake with a certificate it does not trust | a per-launch key pin, recorded as `captured.trust`. [`../experiments/40-trust-paths.sh`](../experiments/40-trust-paths.sh) reports which routes work per platform |
| many connections to one | a browser opens sockets it abandons and it resumes, and a resumed hello is a different hello | `select` chooses PER HALF: the TLS half comes from the first hello offering no pre-shared key, whether or not that connection reached HTTP/2, and the HTTP/2 half from the first connection that did. `captured.connections` records both numbers and `captured.resumption` records what the harness offered |
| capture to profile | a field somebody typed rather than read | the identity file is written from what the run reported, and the capture script reads the driver's and the harness's own output back |
| profile to store | an edit to something already published | `Store::add` refuses a path that exists. The corpus is append-only. |

---

## 4. What is published, and what is derived

⭐ **One assembler builds everything that leaves this repository, and both
surfaces take what it produced.**
[`../crates/b-ids-corpus/src/publish.rs`](../crates/b-ids-corpus/src/publish.rs)
writes a tree carrying the corpus, the raw bytes, the generated formats, the
flat routes, the per-build anchor lists, the digest vectors, a manifest and a
checksums file. ⛔ It reads no clock and it writes to no remote: a workflow
pushes what it built, and the manifest is stamped with a digest of the corpus,
so a rebuild is byte-identical and a change is not.

| surface | state |
| --- | --- |
| the `data` branch | ⭐ published. `origin/data` carries that tree. Every push to the default branch appends to it, a push that would change nothing is a no-op, and ⛔ it is never force-pushed. |
| a tagged release | ⚠ none. A pushed tag is the only thing that cuts one, and pushing one is the operator's act. |

⭐ **A digest is derived, never stored.**
[`../crates/b-ids-schema/src/tls.rs`](../crates/b-ids-schema/src/tls.rs) renders
JA4's sorted lists and its two raw forms and
[`../crates/b-ids-harness/src/digest.rs`](../crates/b-ids-harness/src/digest.rs)
hashes them, both from the model rather than from bytes, so one parser answers
for every consumer. ⛔ **No digest route is generated**, because a route exists
only where the corpus holds the value and the corpus holds none.
[`../TODO/publish.md`](../TODO/publish.md) and
[`../TODO/validator.md`](../TODO/validator.md) carry both rules.

| path | written by | ⛔ rule |
| --- | --- | --- |
| `corpus/v1/<route>/<version>.json` | `b-ids-corpus add` | append-only. Never edited, never deleted. |
| `raw/v1/<route>/<version>.hello.hex` | the same | the bytes the profile was read from, one line, no trailing newline |
| `corpus/v1/index.json` | `b-ids-corpus index --write` | ⭐ DERIVED. It is rewritten from the tree and is exempt from the append-only rule. |
| `corpus/v1/latest.json` | the same | DERIVED, and `latest` means the newest STABLE build. `CORPUS-03`. |

---

## 5. ⛔ The limits, stated rather than discovered

- **The corpus holds six profiles.** Five Chrome and one Edge, majors 151 and
  152, on two platforms. ⚠ Three of them were taken through whatever build the
  runner image shipped and record `captured.acquisition: null`; three name the
  URL and the digest they were installed from.
  [`../TODO/corpus.md`](../TODO/corpus.md), `CORPUS-02`, is the matrix.
- **No digest is stored in a profile**, by the rule in section 4, and ⛔ JA3 is
  not computed anywhere: it is an MD5, this tree links no MD5, and a digest that
  changes with a browser's per-connection shuffle is not one to assert on.
- **Every capture went through a per-launch key pin**, and on ONE platform that
  is now measured to cost nothing: on `ubuntu-latest` with Chrome `151`, 19 TLS
  fields compared against a root in the store the browser reads, 0 differing.
  ⚠ Windows is unmeasured, because the install could not be made to succeed
  there non-interactively.
  [`../experiments/50-trust-anchor.sh`](../experiments/50-trust-anchor.sh) is the
  run and `HARNESS-14` is the entry.
- **The resolver knows two families.** `chrome` and `edge`. A corpus dimension no
  resolver can produce is reported by `b_ids_validator::unreachable_dimensions`.
- **The HTTP half is one variant.** A capture records the navigation; a
  subresource fetch and a reload are shapes the model has and the capture path
  does not yet produce.
- ⚠ **A credential header is recorded as present with no value.** Whether it was
  sent and where in the order is a fingerprint signal; the value never appears on
  any surface.

---

## 6. Where to look next

| you want | read |
| --- | --- |
| the terms, each with its caveat | [`glossary.md`](glossary.md) |
| a value this project did not measure | [`inherited-claims.md`](inherited-claims.md) |
| what a script does and what it is held to | [`../scripts/README.md`](../scripts/README.md) |
| a measurement this project took | [`../experiments/README.md`](../experiments/README.md) |
| how work is done here | [`AGENTS.md`](AGENTS.md) |
