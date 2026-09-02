# architecture.md

⭐ **The technical reference.** What a profile is, what the five components do to
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
| `digests` | ⚠ empty. Nothing here computes JA3 or JA4 yet, and a digest from an unverified implementation is the fabricated value this project refuses. `VALID-04`. | the same file |

⛔ **The published contract is a file, not a doc comment.**
[`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json)
is what a consumer validates against, and a test asserts it agrees with the Rust
types about every integer's width.

---

## 2. Five components, and what each is not

| crate | what it does | ⛔ what it is not |
| --- | --- | --- |
| [`b-ids-schema`](../crates/b-ids-schema/) | the model, its refusals, and the published JSON schema | it captures nothing and reaches no network |
| [`b-ids-harness`](../crates/b-ids-harness/) | a SERVER that accepts connections and reads what arrived: the hello, the frames, the headers | ⛔ not a client. It never asks a browser what it sends; it reads what the browser put on a socket. |
| [`b-ids-driver`](../crates/b-ids-driver/) | resolves a browser on the machine, reports what the vendor is serving, and launches one at a URL into a profile nobody keeps | it parses no bytes |
| [`b-ids-corpus`](../crates/b-ids-corpus/) | turns a capture into a profile, publishes it at its route, and verifies the whole store | ⛔ it never edits a published profile. A correction is a NEW profile naming the one it replaces. |
| [`b-ids-validator`](../crates/b-ids-validator/) | pure logic over the model: could a real browser have sent this? | ⛔ it does not warn. Every check answers passed, failed, or not-checkable. |

⚠ **Four more crates exist and none is a component yet**, and each says so in
its own first line rather than looking finished:
[`b-ids-emit`](../crates/b-ids-emit/) for `EMIT-01`,
[`b-ids-conformance`](../crates/b-ids-conformance/) for `VALID-05`,
[`b-ids`](../crates/b-ids/) for `LIB-01`, and
[`b-ids-cli`](../crates/b-ids-cli/), the front door with nothing to front yet.
⭐ They exist so that the acceptance commands in [`../TODO/`](../TODO/) resolve
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

| path | written by | ⛔ rule |
| --- | --- | --- |
| `corpus/v1/<route>/<version>.json` | `b-ids-corpus add` | append-only. Never edited, never deleted. |
| `raw/v1/<route>/<version>.hello.hex` | the same | the bytes the profile was read from, one line, no trailing newline |
| `corpus/v1/index.json` | `b-ids-corpus index --write` | ⭐ DERIVED. It is rewritten from the tree and is exempt from the append-only rule. |
| `corpus/v1/latest.json` | the same | DERIVED, and `latest` means the newest STABLE build. `CORPUS-03`. |

⚠ **Nothing is published outside this repository yet.** There is no release, no
data branch and no fetchable route; `PUB-01`, `PUB-02` and `PUB-03` are those
three surfaces. [`../TODO/publish.md`](../TODO/publish.md).

---

## 5. ⛔ The limits, stated rather than discovered

- **The corpus holds three profiles.** One from a laptop and two from hosted
  runners, all Chrome 151. [`../TODO/corpus.md`](../TODO/corpus.md), `CORPUS-02`,
  is the matrix.
- **No digest is computed.** `digests` is empty on every profile and will stay
  empty until a reference implementation is verified against published vectors.
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
