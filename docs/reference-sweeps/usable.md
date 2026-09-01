# reference-sweeps/usable.md

The half of the sweep written for the session that does the work: the
mechanisms, at file and line, with what each one changes here.

[`findings.md`](findings.md) is the verdicts and the reasoning, and it opens
with what the sweep did not establish. ⛔ Read that first. Nothing on this page
was measured on a wire **by this project**, and the rows taken from
`Azathothas/bit-cli` were measured on a wire by somebody else.

Every path below is under [`../../references/`](../../references/) at the commit
its `PROVENANCE.md` names.

---

## 1. The extension model, settled

⭐ **Take `utls`'s shape, not `impit`'s or `rustls`'s.**

```text
references/refraction-networking__utls/tree/u_common.go:184
references/refraction-networking__utls/tree/u_tls_extensions.go:875
```

```go
type ClientHelloSpec struct {
	CipherSuites       []uint16
	CompressionMethods []uint8
	Extensions         []TLSExtension
	TLSVersMin uint16
	TLSVersMax uint16
	GetSessionID func(ticket []byte) [32]byte
}

type GenericExtension struct {
	Id   uint16
	Data []byte
}
```

An ordered list, keyed by codepoint, with an arbitrary body. It is what the two
alternatives cannot do:

| model | why it fails |
| --- | --- |
| `impit`'s `TlsExtensions { server_name: bool, ... }` plus a closed `ExtensionType` enum | a codepoint learned at runtime has no variant. `impit/src/fingerprint/mod.rs:106`, `types.rs:87` |
| `rustls`'s `ClientExtensions { server_name: Option<T>, ... }` | its own doc comment says "Unknown extensions are dropped during parsing", and the struct is `pub(crate)`. `rustls/src/msgs/client_hello.rs:141` |

⭐ **Copy the refusal too.** `ClientHelloSpec.FromRaw`
(`u_common.go:258-265`) errors with `unsupported extension %d` unless a mimicry
flag is set, and only then keeps the unknown codepoint with its bytes. A parser
that silently drops is worse than one that stops.

⚠ **And do not copy the branch that cannot fire.** `u_common.go:335-341` logs
"added as a &GenericExtension without Data" in a branch guarded by conditions
the early return above it has already handled. ⛔ The unknown-is-not-absent rule
is proved by a test over a fixture, never by the existence of a branch.

⛔ **This is not academic: it is what stopped the origin repository.** `T-264`
tried to move a profile from Chrome 151 to Chrome 152, found `0x12e0` and
`0xca34` in the newer hello, and could name neither in either model. The bump
did not ship. `SCHEMA-02` and `EMIT-02` exist so it can here.

---

## 2. The three quantities that are recorded in the wrong unit, everywhere

⛔ **Three separate off-by-one traps, all the same shape: a human-facing number
and a wire number sharing one field.** Name every field for the wire.

| quantity | human value | wire value | seen in |
| --- | --- | --- | --- |
| connection window | `15728640` (15 MiB) | `15663105` (window minus the 65,535 protocol default) | `impit/src/fingerprint/database/chrome.rs:138` versus `:279` and six others |
| stream priority weight | `256` | `255` (HTTP/2 encodes `weight - 1`) | `curl-impersonate/bin/curl_chrome116` passes `--http2-stream-weight 256` |
| the same field, two meanings, in one file | seven `impit` entries hold the increment in a field its own comment defines as the window | | audited in [`findings.md`](findings.md) |

⭐ **`curl-impersonate` already names it correctly** and this project should use
the same word: its signatures carry `window_size_increment` inside a
`frame_type: WINDOW_UPDATE` block, so the unit is unambiguous from the name.

⭐ **And the origin repository shows both halves of the weight trap in one
type.** `Azathothas__bit-cli/tree/crates/bit-cli-core/src/page.rs:293-299`
declares `BROWSER_H2_STREAM_PRIORITY: (u32, u8, bool)` as
`(dependency, wire weight, exclusive)` and states in the doc comment that the
wire weight is one less than the weight the specification talks in. `255` and
`256` are one quantity. ⛔ A profile that does not say which it holds is a
profile nobody can emit from.

⚠ It keeps the connection window as a **window**,
`BROWSER_H2_CONNECTION_WINDOW = 15_728_640` at `page.rs:277`, with the
subtraction in the emitter, and says so in the comment. That is the other
defensible choice and it is only defensible because it is named and documented.
⭐ **This project records the increment**, because the increment is what the
capture reads and a profile records what the wire carried.

---

## 3. What a recorded signature should carry, and what one already does

`curl-impersonate`'s signature files are the closest existing thing to this
project's profile. Read one whole before designing the schema:

```text
references/lexiforest__curl-impersonate/tree/tests/signatures/chrome_116.0.5845.180_win10.yaml
```

⭐ **Adopt directly:**

- extensions as an **ordered list of objects**, each with its `type`, its
  `length` and its body where it has one, including
  `data: !!binary AA==` for the trailing GREASE;
- `key_shares` recording **entry lengths** as well as group ids;
- `handshake_version`, `record_version`, `session_id_length`,
  `comp_methods` as first-class fields;
- HTTP/2 as an ordered **frame list** rather than a settings map, so SETTINGS
  order, the WINDOW_UPDATE and the HEADERS block are one sequence;
- ⭐ a **`third_party:` block, sibling to `signature:`**, holding `ja3_text`,
  `ja3_hash`, `ja3n_text`, `ja3n_hash`, `akamai_text`, `akamai_hash`. Derived
  values live outside the measured ones and are visibly derived.

⛔ **Add what it does not have**, which is where this project's contribution is:
a capture instant, a per-field provenance map, the raw `ClientHello` hex, the
channel, and a published route with a checksum.

⚠ **And fix what it gets wrong**: it sets `tls_permute_extensions: true` beside
a single recorded extension order, without saying that the order is one draw. A
profile records the shuffle as a property and says how many draws it saw.

⚠ **The origin repository's goldens are the other end of the same spectrum and
are worth reading for contrast.**
`Azathothas__bit-cli/tree/fingerprints/bit-cli-browser.json` and
`bench/browser-fingerprint-cft-152.json` carry a capture instant and a note
saying which probe mode produced which field, which the signatures do not, and
carry no per-field provenance, no channel and no committed raw hello, which they
also do not. ⭐ Between the two, every field this project's schema needs is
already justified by somebody's absence of it.

---

## 4. The PRIORITY block: the seam in `h2`, and the patch that already exists

**The seam.** `references/hyperium__h2/tree/src/frame/headers.rs`:

| line | what is there |
| --- | --- |
| `:113` | `const PRIORITY: u8 = 0x20;`, so the flag exists |
| `:120`, `:141` | both send-path constructors hardcode `stream_dep: None` |
| `:180` | `StreamDependency::load(&src[..5])`, so it is parsed on receive |
| `:650-660` | `EncodingHeaderBlock::encode<F>(..., f: F) where F: FnOnce(&mut EncodeBuf<'_>)` |
| `:667` | `f(dst)` runs after the frame head and before the HPACK block |
| `:692` | the payload length is computed after `f` ran, so bytes it wrote are counted |
| `:498-514` | `PushPromise::encode` already uses the closure, to write `promised_id` |
| `:277-291` | `Headers::encode` passes `\|_\| {}` |

⭐ So the change is five bytes written in a closure that already exists, and the
frame length and any CONTINUATION split follow for free.

⭐ **The diff is in the corpus, so `EMIT-03` starts from a patch rather than
from a seam.**

```text
references/Azathothas__bit-cli/tree/patches/h2/0004-src-frame-headers.rs.patch
```

It adds `StreamDependency::encode`, which is the half `load` never had, and
`Headers::set_stream_priority`, which sets the payload **and** the flag in one
call: ⛔ a head carrying the flag with no block is a frame a peer cannot parse,
and a block with no flag is five bytes of header block. The rationale, the
files it touches and the upstream verdict are under `## h2: a client cannot open
a stream with the PRIORITY block a browser sends` in `references/Azathothas__bit-cli/tree/patches/UPSTREAM.md`.

⚠ **Read it, do not copy it.** That tree is MIT and this one's output is 0BSD.
The mechanism is four sentences; the diff is somebody else's file.

**The measurement to take first** is no longer a question with three answers.
`h2fp.rs:219-223` in the same tree reads the HEADERS flags byte for `0x20` and
decodes the five bytes behind it, and both Chrome 151 and Chrome 152 read
`1:1:0:255`. ⭐ **So `HARNESS-05` is a confirmation with a predicted answer and a
positive control**, and the control is the patched client above, which is known
to emit a block. [`findings.md`](findings.md) finding 1 has the table.

---

## 5. Reproducing an extension order, and the ceiling on doing so

`references/apify__rustls/tree/rustls/src/msgs/client_hello.rs:337-355`:

```rust
order.sort_by_cached_key(|new_ext| {
    let seed = ((self.order_seed as u32) << 16) | (u16::from(*new_ext) as u32);
    low_quality_integer_hash(seed)
});
```

⭐ **The order is a pure function of a `u16`.** So an emitter can reproduce a
captured order by searching 65,536 seeds, and a profile could record the seed
instead of the sequence.

⛔ **And that is the ceiling: 65,536 reachable orders**, against the factorial of
the extension count. An arbitrary captured order generally is not reachable.
That is a row for the support matrix, with a number in it.

⚠ PSK and the two ECH extensions are removed before the sort and forced last,
and `contiguous_extensions` names groups that must stay adjacent. Any emitter
this project writes needs both concepts.

⛔ **The trap in it, paid for in the origin repository.** `T-263` found the
shuffle was already built and doing nothing, because naming **every** extension
in `extension_order` left the random set empty. The fix was to name none. ⭐ A
shuffle whose input list is exhaustive is a fixed order with a hash in front of
it, and nothing reports that: two captures simply come out identical.

⚠ **Forced placement is a second-order tell.** That fork forces
`encrypted_client_hello` second to last, and the Chrome 152 capture has it at
position 11, inside the shuffle. `T-263` records it as a remaining difference. ⛔
An emitter's placement rules are part of its support matrix row, not an
implementation detail.

---

## 6. Digests: what each one can and cannot see

| digest | strips GREASE | preserves order | so it can show |
| --- | --- | --- | --- |
| JA3 | ⛔ **yes.** `salesforce__ja3/tree/python/ja3.py:22,98,119` | yes | an order change, and nothing about GREASE |
| JA3N | yes | no, sorted | neither |
| JA4 | yes, `FoxIO-LLC__ja4/tree/technical_details/JA4.md:40` | no, sorted | neither |
| JA4_r | yes | no, sorted | the same lists, unhashed |
| JA4_ro | ⛔ **yes**, `JA4.md:208` | yes | an order change, and nothing about GREASE |
| **the raw `ClientHello` hex** | n/a | yes | ⭐ everything, including which GREASE values were drawn, where they sat, and that one body is empty and the other is one zero byte |

⭐ **So the raw bytes are not a backstop against parser rot alone. They are the
only artefact in which a GREASE question is answerable at all.** Store them on
every capture, from the first commit.

⚠ Assert on JA4. Record JA3 and JA4_ro as diagnostics. Never key on any of them.

⭐ **A worked demonstration that JA4 hides a real difference.** `T-263` in the
origin tree records one client and one browser producing the **same JA4**,
`t13i1515h2_8daaf6152771_806a8c22fdea`, over two different wire orders, one with
GREASE at both ends and one with none. ⛔ Two artefacts agreeing on a digest is
not two artefacts agreeing.

---

## 7. Version coherence: the check, and three shipped violations to test it against

⭐ **A validator's first check has three real failures waiting for it in the
corpus, which means the check can be proved rather than asserted.**

| exhibit | what it is |
| --- | --- |
| `impit/src/fingerprint/database/chrome.rs:985-996` | `chrome_101` returns `chrome_100::tls_fingerprint()` and `chrome_100::http2_fingerprint()` beside a Chrome 101 User-Agent and a Chrome 101 `sec-ch-ua`. Four more modules do the same. |
| `Kikobeats__https-tls/tree/src/index.js:59,78,100` | one cipher table per family, commented "Chrome v92", "Firefox v91" and "Safari v14", served to any version's User-Agent |
| `Kikobeats__https-tls/tree/src/browser.js` | the classifier returns three families and `headers-order.json:119` carries a fourth, `edge`, that nothing can reach |

⭐ **Three checks fall straight out of those:**

1. the major version in `sec-ch-ua`, in the User-Agent and in `browser.major`
   agree;
2. the TLS half and the header half come from the same build, not from a
   neighbour's;
3. every family the data carries is a family the resolver can actually produce.

⚠ Check 3 is the one nobody writes. It catches dead data, which is data a reader
will eventually believe.

### ⭐ Read by a tool on 2026-09-01, and it found two the eye had not

`VALID-02` implemented a reader for both trees. It agrees with the three rows
above and adds two, which is the argument for reading a table with a tool
rather than by eye:

| what the reader added | where |
| --- | --- |
| a sixth entry returning another version's handshake, in a second family | `impit/src/fingerprint/database/firefox.rs:444`, where `firefox_144` returns `firefox_135`'s |
| a third table commented for one version and served to all of them | `Kikobeats__https-tls/tree/src/index.js:100`, "Safari v14" |

⚠ **Two line numbers in the row above were corrected in the same pass.** The
cipher tables were cited at 57 and 80 and the comments naming their versions
are at 59 and 78. ⭐ A citation nobody re-opened is exactly what this project's
third rule is about, and here the tool re-opened it.

```bash
cargo run -p b-ids-validator -- import references --report
```

⭐ **Check 1 has a working implementation to read**, in the origin tree's own
test suite. `page.rs:1618-1632`, `the_header_list_and_the_user_agent_agree_
about_the_major`, asserts three things against one constant: the User-Agent
carries `Chrome/<major>.0.0.0`, the exact build starts with that major, and
`sec-ch-ua` names `"Google Chrome";v="<major>"`. Fourteen lines, no network, and
a header block and a version constant cannot drift apart in that repository at
all.

---

## 8. Surfaces the plan does not currently record

Each is measured by somebody else and cheap to add to the model.

| surface | evidence | why it matters |
| --- | --- | --- |
| ⭐ **the multipart boundary** | `impit/src/fingerprint/mod.rs:41-58`: Chrome emits `----WebKitFormBoundary` plus 16 alphanumerics, Firefox `----geckoformboundary` plus 32 hex, OkHttp a UUID | it is per-browser, it appears in every form POST, and it is not in the profile model |
| `SETTINGS_MAX_CONCURRENT_STREAMS` | `curl-impersonate/bin/curl_chrome116` sends `3:1000`; the Chrome 150 table in `impit` issue 385 has no setting 3 | a settings key present in one Chrome version and absent in a later one, which refutes any assumption that SETTINGS are version-invariant |
| `SETTINGS_HEADER_TABLE_SIZE` | the same two sources agree on `1:65536`, and `bit-cli`'s `page.rs:122` is a third, from a wire capture | the setting `impit` cannot express at all |
| ALPS codepoint | `chrome_118`'s `ja3_text` carries `17513`, which is `0x4469`; the Chrome 152 hello carries `0x44cd` | the codepoint moved with a version, and which one appears dates the build |
| pseudo-header order | recorded by every source, as `m,a,s,p` for Chrome | already in the plan; noted because `impit` issue 472 shows it carried in a process-global, which is an emitter hazard |
| ⭐ **whether `alpn` is inside the shuffle** | `T-263`: position 9 of the Chrome 152 hello, so it is not pinned | an emitter that pins it produces an order no browser produces |

---

## 9. Publishing: the route shape, and the defect to avoid in it

⭐ **The shape works and two projects prove it.**

| project | route |
| --- | --- |
| `microlinkhq/top-user-agents` | three flat JSON files under `src/`, fetched over a CDN, refreshed weekly by CI |
| `pkgforge-security/Wordlists` | `Misc/User-Agents/ua_<browser>_<platform>_<latest\|all>.txt` |

⛔ **And the defect to avoid is in the second one.** Measured with `od -c`:
`ua_chrome_windows_latest.txt` and `ua_safari_macos_latest.txt` both end with a
newline, so a `curl` consumer of a single-value route has to strip it.

⭐ **Write the check first.** For every route file that carries exactly one
value, assert the last byte is not `\n`. It is three lines and it turns a
requirement into a gate.

⚠ Neither scheme has a channel or an exact-version axis. This project's routes
need both, and a `latest` that means stable and only stable.

⚠ **And carry a checksum beside every published file.** Two copies of
`top-user-agents` version `2.1.132` contain 100 and 99 entries. A version number
that does not pin its content is not a pin.

---

## 10. Automation shapes worth copying

| shape | where | what it gives |
| --- | --- | --- |
| a weekly CI job that regenerates a dataset and commits it | `microlinkhq/top-user-agents` | the corpus stays current without anybody running anything |
| a per-target expected-signature file, asserted by the test suite | `curl-impersonate/tests/signatures/` and `tests/targets.yaml` | ⭐ the conformance suite shape: a client points at the corpus and gets a per-field diff |
| a machine-readable target index separate from the data | `curl-impersonate/browsers.json` | identity is queryable without parsing the fingerprints |
| ⭐ a staleness job on a cron, with the replacement values in its output | `bit-cli/.github/workflows/staleness.yml`, `cron: "17 6 * * 1"` | a red check that carries the fix. Section 12. |

⭐ **The conformance suite is the highest-value one**, because it is the same
artefact from both directions: this project's corpus plus a comparison harness
is exactly what tells a client author which fields they get wrong.

---

## 11. Scope, in one table, because the names collide

| project | measures | this project's relationship |
| --- | --- | --- |
| `damianobarbati/get-browser-fingerprint` | canvas, WebGL, audio, fonts, `Intl`, screen, from JavaScript in a page | ⭐ the complement. A detector sees both surfaces; this one measures the network half. |
| `daijro/camoufox` | patches the script surface and two HTTP headers. Grep over its 39 patches finds no TLS change. | a consumer of measured network profiles, not a source of them |
| `daijro/browserforge` | generates header sets by sampling a Bayesian network | ⛔ generated, not measured. The thing the first rule refuses. |
| `apify/impit`, `curl-impersonate`, `Azathothas/bit-cli` | emit a chosen fingerprint | consumers, and the three whose defects define the validator |
| `FoxIO-LLC/ja4`, `salesforce/ja3` | specify digests | dependencies of the schema, and a licence question |

⛔ **Say this in the README.** "Browser fingerprint" means the first row to most
readers and this project to almost nobody, and a scope stated late is a scope
argued about.

---

## 12. Version discovery: read the fraction, not the top of the list

`references/Azathothas__bit-cli/tree/scripts/check-browser-version.ps1`

⛔ **The highest version a channel knows is not the version anybody runs.** The
one-page `.../versions?pageSize=1` form answers with the highest **known**
build, which during a staged rollout reaches a fraction of a percent of users.
The header comment at `:25-37` carries the measurement that produced the rule.

The shape to copy, at `:162-190`:

- read `.../versions/all/releases`, which carries a `fraction` per release;
- take the **highest version at fraction 1**; only where there is none does the
  highest fraction win;
- cross-check against the automation build index and report both;
- ⭐ print the highest published version and its fraction **beside** the answer,
  so a reader can check the choice rather than take it.

⛔ **Every fetch is trapped on its own**, at `:167`, `:198`, `:211` and `:219`.
One dead vendor endpoint degrades that field and leaves the others intact. A
check that reports nothing during somebody else's outage is a check people
disable.

⚠ **Two first-party sources disagreed by one patch component** on the day it was
measured, and that disagreement is the finding rather than the error.
[`../inherited-claims.md`](../inherited-claims.md) section 7 carries the numbers
and the endpoints.

---

## 13. One file holds a whole profile, and that is the shape to take

`references/Azathothas__bit-cli/tree/crates/bit-cli-core/src/page.rs`

⭐ **Every value a client puts on the wire lives in one file that repository
owns**: `BROWSER_MAJOR`, `BROWSER_BUILD`, `BROWSER_USER_AGENT`,
`BROWSER_HEADERS`, `BROWSER_CIPHER_SUITES`, `BROWSER_KEY_EXCHANGE_GROUPS`,
`BROWSER_SIGNATURE_ALGORITHMS`, `BROWSER_EXTENSION_ORDER`, `BROWSER_ALPN`, the
four HTTP/2 settings, `BROWSER_H2_STREAM_PRIORITY` and
`BROWSER_PSEUDO_HEADER_ORDER`. A function beside them constructs the vendored
client's own type from those constants.

⛔ **It got there by being moved out of a vendored database**, and the reason is
this project's first rule reached independently: a starting point does not get
to be the home of the answer. Before the move, a version bump meant reconciling
somebody else's table; after it, a bump edits one file and a staleness check
names that file.

⚠ **The move was proved behaviour-neutral rather than assumed.** The goldens did
not change: same JA4, same header order. ⭐ **That is the acceptance shape for
`SCHEMA-08` and for every refactor this project does to a generator**: a change
that is supposed to move nothing is proved by an assertion that was already
there.

⭐ **Two properties to copy into `SCHEMA-01`:**

- `BROWSER_BUILD` is the **exact build** beside the major, because two builds
  of one major have differed and a major alone does not say which capture
  produced a value;
- a superseded value is kept beside the live one rather than deleted.
  `BROWSER_EXTENSION_ORDER` is empty, because nothing is pinned; the list it
  used to hold survives as `BROWSER_EXTENSION_ORDER_WAS` and a test asserts it
  is still non-empty, so the fact that a pinned order once existed cannot be
  lost by an edit.

---

## 14. The capture harness, as a working shape

`references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/`

⭐ **Four files, and the split is the one `HARNESS-01` should take:**

| file | owns |
| --- | --- |
| `main.rs` | the listener, the per-run certificate authority, argument parsing, the JSON contract |
| `tlsfp.rs` | `ClientHello` parsing, and JA3, JA4, JA4_r, JA4_ro |
| `h2fp.rs` | SETTINGS, WINDOW_UPDATE, the PRIORITY block, the pseudo-header order, the Akamai string |
| `huffman.rs` | HPACK Huffman decode, 87 lines, which is what makes header order readable |

**The switches, at `main.rs:106-128`**, each of which exists because something
went wrong without it:

| switch | why |
| --- | --- |
| `--raw` | do not terminate TLS. ⛔ Completing a handshake can change what a client offers, so a digest read through a terminated handshake is not the digest it ships. |
| `--plain` | cleartext HTTP/1.1, header order only. The capture that works when a client cannot be told to trust anything. |
| `--ca-out <PATH>` | mint an authority per run and write it, so a client completes a **verified** handshake and the HTTP/2 half becomes reachable without disabling verification |
| `--bind <ADDR>` | reach a browser that is not on this machine. ⛔ Refuses a hostname and refuses the unspecified address, by name: the leaf certificate needs a literal, and a fixture that records headers does not belong on every interface. |
| `--hello-out <PATH>` | the raw `ClientHello` as one hex line |
| `--header-values` | record values, not just names. ⛔ The one switch that can log a credential. |
| `--until-h2` | stop at the first connection that reached HTTP/2. Section 15. |
| `--once` | stop at the first connection at all. ⚠ Wrong for a browser, for the same reason. |
| `--json` | one object per connection on stdout, after one line carrying the base URL |

⭐ **And the five that make it a check rather than a probe**, at `:123-127`:
`--expect-ja4`, `--expect-ja3`, `--expect-akamai`, `--expect-file` and
`--write-golden`. The first four exit 1 on a mismatch; the last writes the
golden the fourth reads. ⛔ **`HARNESS-01` ships these in the same change as the
listener.** [`../methodology/references.md`](../methodology/references.md) is why:
an instrument that cannot fail is research that decays, and one that exits
non-zero is a regression check the project keeps.

⚠ **Design it multi-protocol even where one protocol is implemented.** A TLS
listener, an HTTP/1.1 listener, an HTTP/2 listener and later a QUIC listener are
four capture surfaces, and retrofitting the fourth into a TLS-shaped harness is
a rewrite.

---

## 15. Driving a browser: what a navigation actually does

`references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/browser-capture.rs`,
with the resolver at
`references/Azathothas__bit-cli/tree/crates/bit-cli-core/src/browser.rs`. Two jobs kept
separate: **resolve** a browser on this machine, and **drive** it at a URL.
`DRIVER-01` is that split.

⛔ **One navigation is not one connection.** `T-264` drove Chrome 152 at the
probe and got **13 connections**: the first carried no HTTP/2 at all, a
preconnect the browser abandoned, and every one after the second offered
`pre_shared_key` rather than `session_ticket` because the session resumed. ⭐
**Keep the first connection that completed HTTP/2.** That is the cold handshake.
Record a resumed one separately, labelled; never average them.

⛔ **One handshake is not one draw either.** `T-263` counted eleven captures of
one binary: eight `session_ticket`, three `pre_shared_key`. The check that had
made a single handshake makes eight now, at `scripts/check-fingerprint.ps1:69`,
and every one must reach HTTP/2. `HARNESS-08` is that number with its reason.

**The browser flags that work**, measured against Chrome 151 and 152, from the
container capture's own launch line at `scripts/check-browser-fingerprint.ps1:362-365`:

```text
--headless=new --no-sandbox --user-data-dir=<throwaway>
--no-first-run --no-default-browser-check
--disable-search-engine-choice-screen --disable-gpu
--test-type --ignore-certificate-errors
--dump-dom <URL>
```

⚠ `--dump-dom` is a mode and the URL is its positional argument. A URL passed as
a bare argument makes Chrome navigate and sit there, so the whole thing is
wrapped in a timeout: a browser that cannot complete a handshake does not exit.

⛔ **`--ignore-certificate-errors --test-type` go to the browser and never into
a client.** They change what the browser **accepts** after the handshake, not
what it **sends**. `T-264` reached for them because Chrome on Linux does not
read the user's NSS database for server authentication: adding the probe's
authority with `certutil` and seeing `certutil -L` list it still produced
`CertificateUnknown`.

---

## 16. What not to take

| | why |
| --- | --- |
| any value, from any project here, as a corpus entry | it would be a value this project did not measure, which is a draft by rule. [`../inherited-claims.md`](../inherited-claims.md) is where they are recorded instead. |
| source from `Azathothas/bit-cli` | MIT attribution would travel into a 0BSD tree. Cite the mechanism, write the code. |
| JA4 implementation source from `FoxIO-LLC/ja4` | BSD-3 attribution, same reason. Implement from `technical_details/JA4.md`. |
| any JA4+ variant | FoxIO License 1.1, patent pending, monetisation-restricted |
| a whole architecture from any of them | a reference's shape answers its own constraints. Take the mechanism, cited at file and line, with the reason it applies here. |
| ⛔ the origin repository's framing | it built one client that had to look like a browser. This project publishes what browsers send, for anybody, in both directions. Nothing in that tree is dated, provenance-mapped or published, and those absences are this project's whole subject. |
