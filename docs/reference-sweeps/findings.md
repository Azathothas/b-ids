# reference-sweeps/findings.md

What eighteen repositories were read for, what was true in them, and what this
project takes from each. The corpus is in
[`../../references/`](../../references/), tracked, one directory per repository,
each with a `PROVENANCE.md` naming its commit.

⭐ **One of the eighteen is not prior art.** `Azathothas/bit-cli` is the
repository this project's founding brief was written in, and every measurement
this project inherited was taken there. It is read as a **source**, and its
section is the one to read first.

[`usable.md`](usable.md) is the other half: the lines and mechanisms a session
doing the work can act on. This file is the verdicts and the reasoning.

---

## ⛔ What this sweep did NOT establish

Read this before the verdicts.

| | |
| --- | --- |
| **Nothing here was measured on a wire by this project.** | Every claim is a reading of source, of tracker text, or of a committed data file. No browser was launched here, no socket was opened here, and no capture was taken here. Where one repository's data disagrees with another's, this file records the disagreement. |
| ⚠ **`bit-cli`'s numbers were measured on a wire, and not by this project.** | They carry a named instrument, a named build and a capture instant, which makes them the strongest evidence in the corpus and still not this project's own. [`../inherited-claims.md`](../inherited-claims.md) holds every one of them as a claim to re-measure. |
| **The trackers are third-party reports.** | An issue body and a maintainer's comment are evidence of what somebody believed. Two of the strongest findings below come from tracker comments and are marked as such. [`../methodology/references.md`](../methodology/references.md) is why that distinction is load-bearing. |
| **Discussions were fetched for one repository only.** | They are GraphQL only, and the credential-free proxy route seventeen of these were fetched over is REST. `Azathothas/bit-cli` was fetched over an authenticated route, and its discussions came back empty. Every other `PROVENANCE.md` records the gap. |
| **The corpus was trimmed after fetching.** | Font blobs, test-run artefacts, image assets and unrelated wordlists were deleted. Each deletion is named in the affected repository's own `PROVENANCE.md`. Nothing was moved, so every remaining path is the path upstream has. |
| **One project was read at a fork, not at the original.** | `curl-impersonate` was read at `lexiforest/curl-impersonate`, the maintained fork. `lwthiker/curl-impersonate` was not fetched, so nothing here describes it. |
| **Depth is uneven and the table says so.** | Four repositories got four reading passes. The rest got one or two, because one or two answered the question they were fetched for. |

⚠ **This is a second revision.** The first covered seventeen repositories,
counted them as sixteen, and omitted the origin repository entirely. ⭐ **Five
claims changed when that tree was finally read**, and they are listed under
"What the second revision corrected". That ratio is the only honest estimate of
how many are still wrong. ⛔ Assume more remain.

---

## Route the reader by budget

| a reader with | reads |
| --- | --- |
| two minutes | the provenance table, then "The five findings that change the plan" |
| ten minutes | those, then "Where the prior art actually stands" |
| the implementation to do | [`usable.md`](usable.md), which is written for that |
| a reason to distrust this | the per-repository sections, each of which cites a file and a line you can open in `references/` |

---

## Provenance

Every tree is at the commit below. ⛔ Cite the commit beside any line reference
taken from it: a path alone is not a citation once upstream moves.

| repository | commit | passes | why it was read |
| --- | --- | --- | --- |
| ⭐ [`Azathothas/bit-cli`](../../references/Azathothas__bit-cli/) | `cce8131231abe8b232054f3f27b3feeac19dd411` | 4 | the origin. Every value this project inherited was measured here |
| [`apify/impit`](../../references/apify__impit/) | `863ddd026aa9285727240f7ef73bc80783d820ec` | 4 | the worked example the founding brief uses throughout |
| [`apify/rustls`](../../references/apify__rustls/) | `61ab1bc8349d35bfb9a9f1a2a983cb404a79159e` | 3 | the fork that is supposed to emit a chosen `ClientHello` |
| [`hyperium/h2`](../../references/hyperium__h2/) | `cb9574bb2c18d1904eca74e98b31c8986b0d8b32` | 3 | whether a client can open a stream with a browser's PRIORITY block |
| [`lexiforest/curl-impersonate`](../../references/lexiforest__curl-impersonate/) | `8d0c2c904c4f3751705d0ede8c28873116309fd2` | 4 | the claim that nobody publishes a machine-readable corpus |
| [`FoxIO-LLC/ja4`](../../references/FoxIO-LLC__ja4/) | `02e78ba3ebac1f5c38bd3eb1a91b4a82e919e5fc` | 3 | the JA4 specification, its test vectors, and its licence |
| [`salesforce/ja3`](../../references/salesforce__ja3/) | `502cc6395811c54743b0561419d61900a6df3ff7` | 2 | whether JA3 strips GREASE |
| [`refraction-networking/utls`](../../references/refraction-networking__utls/) | `23b1dac19c06c51e278468e29ac329eec605a31f` | 4 | the arbitrary-codepoint escape hatch and the raw-hello parser |
| [`damianobarbati/get-browser-fingerprint`](../../references/damianobarbati__get-browser-fingerprint/) | `9d347ee8aa548c5f90d934c7782b4038340a46e6` | 1 | where the scope boundary of this project actually is |
| [`microlinkhq/top-user-agents`](../../references/microlinkhq__top-user-agents/) | `84a7a7110d5cb212271869004e4f4a9671a445b3` | 2 | a working publishing surface for flat, fetchable data |
| [`EIGHTFINITE/top-user-agents`](../../references/EIGHTFINITE__top-user-agents/) | `e4b56da9acbd4f051da817c0013b2a919beff44b` | 2 | what two copies of one dataset do to each other |
| [`Kikobeats/https-tls`](../../references/Kikobeats__https-tls/) | `b096cb866bdebd5e81d8265631a9b86ff01e0e5f` | 3 | deriving TLS parameters from a User-Agent string |
| [`pkgforge-security/Wordlists`](../../references/pkgforge-security__Wordlists/) | `06f537fd60ef45337af52224d743cc359c2d31ee` | 2 | the flat route the operator named as the model |
| [`daijro/camoufox`](../../references/daijro__camoufox/) | `1a67b4a16630d350e00a375542298875046935e0` | 2 | whether it touches the network layer at all |
| [`daijro/browserforge`](../../references/daijro__browserforge/) | `a8b798f37460d1dd02aea33f80c83647913a1bbd` | 2 | generated fingerprints against measured ones |
| [`adryfish/fingerprint-chromium`](../../references/adryfish__fingerprint-chromium/) | `3f61b0dfa665e883da8824b1450601fc529dd006` | 1 | what its repository actually contains |
| [`botswin/BotBrowser`](../../references/botswin__BotBrowser/) | `83471e4a055de9698f5f18eb605be623789e38e1` | 2 | what an open licence over closed data looks like |
| [`CloakHQ/CloakBrowser`](../../references/CloakHQ__CloakBrowser/) | `f04c23da285b3b3d3cf10c8f9d282e7adc1d52ce` | 2 | the same question, in its other shape |

⚠ **`refraction-networking/utls` was not on the reading list and was added.** It
is the only project in the set that already solves the design problem this one
is warned about, so leaving it out would have meant designing around a
constraint somebody has already removed.

⛔ **`Azathothas/bit-cli` is where the reading list came from, and it was
missing.** The first revision cited a founding brief for about sixty inherited
values and never named or fetched the repository those values were measured in,
so not one of them could be checked. It is fetched now, and every row in
[`../inherited-claims.md`](../inherited-claims.md) carries a file and a line
against it.

---

## ⭐ The five findings that change the plan

### 1. The PRIORITY block is settled, by frame bytes, in the origin repository

⛔ **This was recorded as contested three ways with no measurement on any side.
It is not contested. One of the three readings is a frame-byte read and the
other two are rendered strings.**

`bit-cli`'s probe reads the HEADERS frame's flags byte for `0x20` and, when it
is set, decodes the five bytes after the frame head, at
[`h2fp.rs:219-223`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/h2fp.rs):
the exclusive bit is `b[0] >> 7`, the dependency is the remaining 31 bits, and
the weight is `b[4]`. That is exactly the measurement this sweep's first
revision said would settle it.

| source | reading | what it read |
| --- | --- | --- |
| ⭐ `bit-cli`, Chrome 151 on Windows and Chrome 152 in a Linux container | exclusive, dependency 0, weight 255, rendered `1:1:0:255` | **the HEADERS frame's flag bit and the five bytes behind it** |
| `curl-impersonate` signatures for Chrome 118, 119 and 120 | the Akamai priority field is `0` | a rendered Akamai string |
| a third-party Chrome 150 capture reported on `apify/impit` issue 385 | the Akamai priority field is `0` | a rendered Akamai string |

⚠ **And the fourth source explains the two zeros rather than contradicting
them.** `bit-cli`'s own client emitted `0` in that field until it patched `h2`,
and the before-and-after is recorded at
[`patches/UPSTREAM.md:1751-1757`](../../references/Azathothas__bit-cli/tree/patches/UPSTREAM.md).
A `0` is what a stack that cannot write the block produces. Two of the three
sources reporting `0` are reading a tool, not a browser.

⛔ **This does not make the value publishable here.** It is a measurement taken
somewhere else, so it is `vendor` provenance and it stays a claim until this
project reads the same bytes. What changes is the entry that re-measures it:
`HARNESS-05` is now a confirmation with a predicted answer and a positive
control, not an open three-way question.

⭐ **The units trap is settled with it.** `bit-cli` names the field for the wire
and says so in the type's own comment at
[`page.rs:293-299`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/src/page.rs):
the wire weight is one less than the weight the specification talks in, so a
browser asking for 256 puts 255 on the wire. `curl-impersonate`'s
`--http2-stream-weight 256` and `bit-cli`'s `255` are one quantity in two units.

### 2. `curl-impersonate` publishes a machine-readable corpus, and the brief said nobody does

⛔ **This is the finding with the largest effect on how this project describes
itself.**

`lexiforest/curl-impersonate` ships **43 signature files** under
[`tests/signatures/`](../../references/lexiforest__curl-impersonate/tree/tests/signatures/),
one per exact build, covering Chrome, Edge, Firefox, Safari on macOS and iOS,
and Tor. Each carries, per
[`chrome_116.0.5845.180_win10.yaml`](../../references/lexiforest__curl-impersonate/tree/tests/signatures/chrome_116.0.5845.180_win10.yaml):

- cipher suites **in wire order, GREASE included**;
- an ordered `extensions` list with per-extension length and body, including
  `data: !!binary AA==` on the trailing GREASE, which is the one-zero-byte
  GREASE body a reimplementation is most likely to get wrong;
- `key_shares` with entry lengths, `supported_groups` with GREASE,
  `sig_hash_algs`, `handshake_version`, `record_version`, `session_id_length`;
- HTTP/2 `frames` with SETTINGS in order, `window_size_increment`, and the
  HEADERS field list and `pseudo_headers` in order;
- a `third_party:` sibling block carrying `ja3_text`, `ja3_hash`, `ja3n_text`,
  `ja3n_hash`, `akamai_text`, `akamai_hash` and `user_agent`.

So the position "nobody publishes the corpus" is not true as stated. What is
true, and checkable, is what those files do **not** carry. Measured by grep over
all 43:

| absent | consequence |
| --- | --- |
| any capture date | a reader cannot tell a 2022 capture from a 2026 one |
| any provenance marker | measured, assumed and copied are indistinguishable |
| the raw `ClientHello` hex | a parser defect is unrecoverable, and the extension bodies are the parser's output rather than the wire's |
| a channel dimension | stable only; no beta, dev, nightly or canary |
| a release, an index or a checksum | it is a test fixture for one client, reachable only by cloning that client |

⚠ **And one recorded field cannot be right for every capture.** The same file
sets `options.tls_permute_extensions: true`, which says the browser shuffles its
extension order per connection, while `extensions:` records one order and does
not say it is one draw.

⭐ **So the gap this project fills is narrower and sharper than "nobody has the
data": it is provenance, dating, raw bytes, channels and a publishing
contract.** [`../../README.md`](../../README.md) claims that one. Claiming the
wider gap is a claim a reader can disprove in one clone.

⚠ **A second documentation defect, found while checking the first.**
[`tests/signatures/README.md`](../../references/lexiforest__curl-impersonate/tree/tests/signatures/README.md)
states that "Profiles with HTTP/3 support also include normalized QUIC transport
parameters and QUIC TLS fields under `http3`". Grep finds `http3` in zero of the
43 signature files. The documented field does not exist in the data.

### 3. JA3 strips GREASE, and the founding brief says it does not

⛔ **Refuted at the reference implementation.**
[`salesforce/ja3`](../../references/salesforce__ja3/tree/) defines
`GREASE_TABLE` at `python/ja3.py:22-26` with all sixteen RFC 8701 values and
filters them out of both the cipher list (`:98`) and the extension list
(`:119`). `README.md:97` states the intent in as many words: "JA3 ignores these
values completely to ensure that programs utilizing GREASE can still be
identified with a single JA3 hash."

⭐ **The conclusion built on the wrong reason survives, and this is why it
matters that both halves were checked.** JA3 is still unstable per connection,
because it preserves **wire order** and Chrome shuffles its extension order. The
instruction "record JA3, never assert on it, assert JA4" is right. The reason
given for it was half wrong, and a session that inherited the reason would have
looked for GREASE in a JA3 string and not found it.

Independent corroboration in the corpus:
[`chrome_118.0.5993.117_linux.yaml`](../../references/lexiforest__curl-impersonate/tree/tests/signatures/chrome_118.0.5993.117_linux.yaml)
carries a `ja3_text` whose cipher and extension lists contain no GREASE value,
beside a `ciphersuites:` block that lists `GREASE` first. The two are consistent
only if JA3 strips.

⭐ **And the origin repository already had it right in code.**
[`tlsfp.rs:226`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/tlsfp.rs)
takes `filter_grease` as a parameter, so its JA3 is computed both ways
deliberately. The brief's prose and the tree it was written in disagreed, and
the tree was right.

### 4. JA4_ro also strips GREASE, so no digest can see a GREASE question

[`FoxIO-LLC/ja4/technical_details/JA4.md:208`](../../references/FoxIO-LLC__ja4/tree/technical_details/JA4.md):
"The 'o' option includes the original values in the original order, **less
GREASE values**."

⛔ So JA4_ro shows what JA4 hides about **order** and nothing about GREASE. The
GREASE positions, the two drawn values, and the fact that one body is empty and
the other is a single zero byte are visible in exactly one artefact: **the raw
`ClientHello` bytes**.

⭐ That is a second, independent argument for the rule that the raw hello is
recorded on every capture, and it arrives from the digest specification rather
than from a fear of parser rot.

### 5. FoxIO's licensing splits JA4 from JA4+, and only one half is free

⛔ **This affects what a 0BSD project may publish.** From
[`LICENSE-JA4`](../../references/FoxIO-LLC__ja4/tree/LICENSE-JA4) and the
licensing answers beside it, in the file whose name carries a space and
therefore cannot be linked from here, `License FAQ.md` in the same directory:

| | licence | what it permits |
| --- | --- | --- |
| **JA4** (TLS client fingerprinting) | BSD 3-Clause, and the FAQ states FoxIO holds no patent claims and is not pursuing any | free, with the attribution BSD-3 requires on **source** |
| **JA4S, JA4L, JA4LS, JA4H, JA4X, JA4SSH, JA4T, JA4TS, JA4TScan, JA4Scan, JA4D, JA4D6** and future additions | FoxIO License 1.1, **patent pending** | "not permissive for monetization"; an OEM licence is required to monetise, including indirectly |

⚠ **The distinction that keeps this project clean** is between the algorithm, an
implementation, and a value:

- a **JA4 string computed from a capture** is a fact about bytes on a wire, and
  publishing it under 0BSD is not distributing FoxIO's software;
- an **implementation written from the published specification** carries no
  obligation to FoxIO;
- **source copied from the JA4 repository** carries BSD-3's attribution, which
  0BSD data must not silently absorb.

⛔ **JA4H and the other JA4+ members are a different question and the answer is
not yet known.** Shipping a JA4H value would need that answered first. Nothing
in this project should emit a JA4+ variant until it is.

---

## ⭐ `Azathothas/bit-cli`, the origin

**Verdict: source. Every inherited value in this tree is a citation into it.**

⚠ **It is not a fingerprinting project.** Its own description is a
command-line BitTorrent client, and its
[`README.md:1-12`](../../references/Azathothas__bit-cli/tree/README.md) opens on
attaching web seeds to a torrent. The browser work exists because the client
fetches from HTTP origins that fingerprint their callers, and the entry that
produced all of it is `T-244`, "a web page is not a source". ⭐ **That is why the
measurements are trustworthy and the framing is not transferable**: they were
taken to make one client indistinguishable from a browser, not to build a
corpus, so nothing in that tree is dated, provenance-mapped, or published.

⛔ **Licence: MIT**, per its `LICENSE` and its repository metadata. 0BSD covers
what this project writes. Source copied out of that tree carries MIT's
attribution, and this project copies none: it cites.

### The tracker is empty, and the design record is in the tree

⛔ **[`../methodology/references.md`](../methodology/references.md) says the
tracker is the step that gets skipped. Here it is the step that has nothing in
it, and that is a finding rather than an excuse.** The fetch got six items, all
of them dependency-bump pull requests, zero review comments, zero releases, zero
tags, and an empty discussions list.

⭐ **The decisions are in `TODO/cli-surface.md`, 6861 lines of them**, one entry
per unit of work with its measurement, its acceptance command and the command's
real output. Three entries carry everything this project inherited:

| entry | what it settles |
| --- | --- |
| `T-262` | the PRIORITY block, read off frame bytes, and the `h2` patch that emits one |
| `T-263` | GREASE at both ends with distinct codepoints and different bodies, and the per-connection shuffle |
| `T-264` | the container capture of Chrome 152, and the two extensions that block a version bump |

⚠ **So a sweep of a repository whose work record is tracked reads the record,
not the tracker.** Which of the two carries the decisions is a property of the
project and is checked rather than assumed.

### What was measured there, and with what

⭐ **The instrument is committed and runnable**, which is what makes every number
below re-derivable rather than quotable.
[`crates/bit-cli-core/examples/loopback-tlsprobe/`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/)
is a listener a browser is pointed at: `main.rs` is the server, the throwaway
certificate authority and the JSON contract; `tlsfp.rs` parses the
`ClientHello` and computes JA3, JA4, JA4_r and JA4_ro; `h2fp.rs` reads SETTINGS,
WINDOW_UPDATE and the PRIORITY block and renders the Akamai string;
`huffman.rs` decodes HPACK Huffman so the header order is readable.

⭐ **It takes `--expect-ja4`, `--expect-ja3`, `--expect-akamai` and
`--expect-file`, and exits 1 on a mismatch**
([`main.rs:123-127`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/main.rs)).
That is the property [`../methodology/references.md`](../methodology/references.md)
calls the difference between research that decays and research that holds: the
same binary is the measuring device and the regression check.

The two committed captures are the evidence for everything this project
inherited about Chrome:

| artefact | what it holds |
| --- | --- |
| [`fingerprints/bit-cli-browser.json`](../../references/Azathothas__bit-cli/tree/fingerprints/bit-cli-browser.json) | Chrome 151 on Windows, captured `2026-08-30T02:29:47.449Z`, with the note saying which probe mode produced which field |
| [`bench/browser-fingerprint-cft-152.json`](../../references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json) | Chrome for Testing 152.0.7977.64 in a throwaway `debian:bookworm-slim` distro, generated `2026-08-30T02:08:35.149Z`, with the header values and the container's own record |

⚠ **Neither is a profile in this project's sense.** Both are single-purpose
goldens for one client's assertions: no per-field provenance, no channel, no
schema anybody else could consume, and the raw hello is a path on the machine
that took it rather than a committed artefact. ⭐ **That absence is this
project's contribution, stated against the closest thing to it that exists.**

### What the origin tree corrects in the brief

⛔ **The brief's Chrome 152 header list carries a `cache-control` entry that its
own capture does not.** The brief lists fourteen header fields with
`cache-control` first and then spends a paragraph on why it might be there. The
committed capture,
[`bench/browser-fingerprint-cft-152.json`](../../references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json)
`observed.headers`, is **thirteen header fields with no `cache-control` at
all**, and `grep` over the whole tree finds `max-age=0` nowhere. `T-264` states the header
change as `accept-language` moving from twelfth to fourth, which is the
arithmetic that holds only without a `cache-control` ahead of it.

⭐ **So the question the brief could not answer does not exist.** There is no
version-versus-request-kind ambiguity to isolate, because there is no
`cache-control` in either capture. The `variants` field is still worth having,
for the reason `SCHEMA-04` gives; it is no longer owed an explanation of this.

⚠ **Three smaller corrections, all from the same reading:**

- the brief writes the Chrome 151 build as `151.0.7922.7x`. The profile the
  golden was captured from is
  [`page.rs:110`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/src/page.rs),
  `151.0.7922.72`, and `T-264` separately records the capture host at
  `151.0.7922.76` and a hosted Ubuntu runner at `151.0.7922.173`. Three builds
  of one major, and the brief's `7x` hid which one produced which number.
- the brief cross-references its own JA3 stability figure to a section number
  that does not exist in it. The figure's real home is a doc comment,
  [`main.rs:32-34`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/main.rs),
  which attributes it to a survey of one impersonating client. It is
  second-hand in the origin tree too.
- the brief says the patch series is "55 patches across eight upstreams". There
  are 55 patch files and eight directories under
  [`patches/`](../../references/Azathothas__bit-cli/tree/patches/), but one of
  the eight holds a scan artefact and no patch. Seven upstreams are patched.

### What the origin tree adds that the brief did not carry

⭐ Each of these is a value or a mechanism the first revision of this tree did
not have. [`../inherited-claims.md`](../inherited-claims.md) carries the values;
[`usable.md`](usable.md) carries the mechanisms.

- **The JA3 hashes and the JA4_r strings for both captures**, which the brief
  reduced to a JA4 each. A JA4_r is readable, so a later disagreement can be
  localised to a cipher or an extension rather than to a hash.
- **A second GREASE draw, on the other version.** `T-263` records Chrome 151
  sending `0x6a6a` first and `0x4a4a` last, beside 152's `0x3a3a` and `0x5a5a`.
  Two versions, two platforms, four drawn values, all distinct within a
  capture. ⭐ That is what turns "drawn per connection" from a claim into a
  pattern.
- **`alpn` is inside the shuffle**, at position 9 of the 152 hello, so it is
  not one of the extensions Chrome pins. `T-263` states it.
- **The GREASE-body defect, with a measured rate.** A GREASE codepoint given a
  typed field with an empty body rejects a GREASE extension carrying a byte.
  Three of the sixteen reserved values were affected, so about one handshake in
  five: one CI run failed, the next passed over the same defect, and the fix was
  confirmed by 64 handshakes with 64 completions. `T-263`'s correction section
  carries the table.
- **Why eight handshakes.** The check that missed that defect made one, so it
  sampled one draw in sixteen. It makes eight now and every one must reach
  HTTP/2, at
  [`check-fingerprint.ps1:69`](../../references/Azathothas__bit-cli/tree/scripts/check-fingerprint.ps1).
  The brief's "default it to something like 8" is that number with its reason.
- **Resumption, counted.** Over eleven captures of one binary, eight offered
  `session_ticket` and three offered `pre_shared_key`, and the two produce
  different JA4s. `T-263`. Separately, `T-264` records a single Chrome 152
  navigation producing **13 connections**, the first carrying no HTTP/2 at all.
- **The full Chrome 152 header values**, not just the names: the unbranded
  `sec-ch-ua`, the Linux platform token, the User-Agent, and the
  `accept-encoding` list. The capture also records that `user-agent` was
  rewritten by the headless normalisation, in a `headless_rewritten` field, so
  the rewrite is reported rather than silent.
- ⭐ **A one-file home for a whole profile.**
  [`page.rs`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/src/page.rs)
  holds the ciphers, groups, signature algorithms, extension order, ALPN, the
  four HTTP/2 settings, the priority block, the pseudo-header order and the
  headers, and constructs the vendored client's type from them. A version bump
  edits one file that repository owns. [`usable.md`](usable.md) section 13.

### ⛔ What blocked the origin, and why it is this project's whole subject

`T-264` set out to move that profile from Chrome 151 to Chrome 152 and stopped.
Chrome 152 sends two extensions Chrome 151 did not, and the stack underneath
could emit neither:

| codepoint | length | body | why it blocks |
| --- | --- | --- | --- |
| `0x12e0` | 2 | `0000` | reproducible by anything that can write an arbitrary extension. Neither `impit`'s closed `ExtensionType` enum nor `rustls`'s one-typed-field-per-extension struct can name it. |
| `0xca34` | 206 | a length-prefixed list of 24 identifiers | ⛔ a snapshot of the browser's own root store. Not a constant to copy, and a client carrying one build's list advertises which build it copied. |

⭐ **The ruling that came out of it is the rule this project is built on.**
Rather than ship a profile claiming 152 without the two extensions, the origin
kept the profile at 151 deliberately, on the reasoning that a `ClientHello` that
exists nowhere is a stronger tell than being one version behind. That is the
same sentence as "an almost-right fingerprint is more distinguishing than an
honestly old one", reached by paying for it.

⭐ **And the unanswerable half is a service nobody provides.** A client with no
root store of its own has three options for `0xca34` and none is obviously
right: omit it, carry a captured list that ages, or send it empty, which is a
shape no browser sends. `CORPUS-04` is this project's entry for publishing
per-build trust-anchor lists with their capture dates and a documented
recommendation.

---

## Where the prior art actually stands

### `apify/impit`

**Verdict: anti-pattern exhibit, kept on purpose, and partly confirms.**

The database model is a boolean per extension.
[`impit/src/fingerprint/mod.rs:106-135`](../../references/apify__impit/tree/impit/src/fingerprint/mod.rs)
declares `TlsExtensions { server_name: bool, status_request: bool, ... }` with
an `extension_order: Vec<ExtensionType>` over the closed enum at
[`types.rs:87-118`](../../references/apify__impit/tree/impit/src/fingerprint/types.rs).
⛔ There is no variant for a codepoint learned at runtime, so an extension
nobody has enumerated cannot be represented at all.

`Http2Fingerprint`
([`mod.rs:100-105`](../../references/apify__impit/tree/impit/src/fingerprint/mod.rs))
holds four fields. **`header_table_size` does not appear anywhere in the crate**
(grep, whole tree, zero hits), so `SETTINGS_HEADER_TABLE_SIZE` is not
expressible. Only three settings reach the client, at
[`impit/src/impit.rs:250-261`](../../references/apify__impit/tree/impit/src/impit.rs),
and `None` there means "do not override" rather than "do not send", so an
absent setting is emitted at the underlying stack's default.

⭐ **The connection-window field carries two different quantities in one file.**
Audited by reading
[`database/chrome.rs`](../../references/apify__impit/tree/impit/src/fingerprint/database/chrome.rs):

| entry | line | value |
| --- | --- | --- |
| `chrome_151` | 138 | `15_728_640`, with a comment stating the field is the window and the emitter subtracts 65,535 |
| `chrome_142`, `chrome_136`, `chrome_133`, `chrome_131`, `chrome_125`, `chrome_124`, `chrome_100` | 279, 420, 555, 690, 828, 960, 1261 | `15_663_105`, which is the wire increment |

⚠ **Independently confirmed in the project's own tracker**, by a third party who
audited the same field and found the same split plus three Firefox profiles at
`12_517_377`. That comment reports the emitted result as `15597570`, which is
the increment subtracted twice.

⭐ **Five of the thirteen Chrome modules carry only their own headers and reuse
another module's TLS and HTTP/2 wholesale.**
[`chrome.rs:985-996`](../../references/apify__impit/tree/impit/src/fingerprint/database/chrome.rs)
is the pattern: `chrome_101` returns `chrome_100::tls_fingerprint()` and
`chrome_100::http2_fingerprint()` beside a Chrome 101 User-Agent. **That is a
new User-Agent over an old hello, shipped, in the reference database**: the
exact combination a coherence checker exists to refuse.

⚠ **The origin repository vendors this crate and stopped reading its data.**
`T-264` moved every value into
[`page.rs`](../../references/Azathothas__bit-cli/tree/crates/bit-cli-core/src/page.rs)
and left the vendored database carrying nothing that ships, on the reasoning
that a starting point does not get to be the home of the answer. ⭐ That is an
independent arrival at this project's first rule, by somebody who had shipped
against the alternative.

**What the tracker adds, and it is the densest source in this sweep.** Read at
`api/issues.json` and `api/comments.json`:

- **385, open**: all profiles emitted one HTTP/2 fingerprint regardless of
  browser, because the settings came from the underlying stack's defaults. A
  measured table gives real Chrome as
  `1:65536;2:0;4:6291456;6:262144|15663105|...`, and states that Chrome does
  **not** send `MAX_FRAME_SIZE`. ⭐ Both agree with the origin repository's own
  constants, independently.
- **385's comment**: locates the remaining hole precisely. `header_table_size`
  is reachable in `h2` and in `hyper`; the layer without it is `reqwest`. ⚠ Not
  verified here: this sweep did not fetch reqwest or hyper.
- **474, closed as wontfix**: an automated review reported GREASE at the wrong
  index in `chrome_131`; the maintainer closed it with "GREASE order doesn't
  affect JA4/JA3, as both strip GREASE before hashing" and a local measurement.
  ⭐ Finding 3 above says the JA3 half of that reason is right and the wire-order
  half is what makes the position matter. **The two accounts cannot be separated
  from a JA4, which is the argument for publishing an artefact that can
  separate them.**
- **315, closed**: a user's Cloudflare bot score for the Chrome profile matched
  a plain `fetch`; switching to the Firefox profile moved it. Third-party,
  measured, and it is what a corpus consumer actually cares about.
- **432, closed**: the multipart boundary is browser-specific.
  [`mod.rs:41-58`](../../references/apify__impit/tree/impit/src/fingerprint/mod.rs)
  implements `----WebKitFormBoundary` plus 16 alphanumerics for Chrome and
  `----geckoformboundary` plus 32 hex for Firefox. ⭐ **A fingerprint surface the
  founding brief does not mention at all.**
- **472, open**: pseudo-header order is carried in a process-global environment
  variable, so two clients in one process clobber each other. An emitter
  constraint worth knowing before choosing a stack.

### `apify/rustls`

**Verdict: confirms, and gives a number the brief did not have.**

⛔ The doc comment on
[`rustls/src/msgs/client_hello.rs:141-148`](../../references/apify__rustls/tree/rustls/src/msgs/client_hello.rs)
states the constraint outright: "Unknown extensions are dropped during
parsing." The struct is one `Option<T>` field per `ExtensionType`, and it is
`pub(crate)`, so it is not reachable from outside the crate at all.

⭐ **The extension shuffle is a pure function of a `u16` seed.**
`order_insensitive_extensions_in_random_order`
([`client_hello.rs:337-355`](../../references/apify__rustls/tree/rustls/src/msgs/client_hello.rs))
sorts by `low_quality_integer_hash((order_seed << 16) | ext_type)`, after
removing `PreSharedKey`, the two ECH extensions and anything in
`contiguous_extensions`. PSK and ECH are forced last.

That has two consequences the founding brief does not state:

- ⭐ a profile could record the **seed** rather than the order, and an emitter
  could reproduce an exact captured order by searching 65,536 candidates;
- ⛔ **at most 65,536 orders are reachable**, out of the factorial of the
  extension count. An arbitrary captured order is generally **not** reproducible
  by this emitter. That is a quantified hole for the support matrix.

⚠ **The brief's claim that this fork emits one GREASE value at a fixed
codepoint is right about the shape and out of date about the fork.** Grep for
GREASE across `rustls/src/` at the commit above finds only Encrypted Client
Hello GREASE (`client/ech.rs`, `client/hs.rs:747-784`), and no RFC 8701 value in
any cipher, group or extension list. ⭐ **The origin repository's own vendored
copy is where the single slot exists**: `T-263` names
`reserved_grease: Option<()>` at the fixed codepoint `0xbaba`, and adds
`ReservedGreaseFirst` and `ReservedGreaseLast` with a `grease_codepoints` field
because a browser sends two at two chosen codepoints. So the constraint is real
and it is a property of a fork this sweep fetched at a different point than the
brief read.

⛔ **And the shape carried a defect worth copying the fix for.** All three GREASE
fields were typed `Option<()>`, which reads an empty body and nothing else, so a
received hello whose GREASE landed on one of those three codepoints was
rejected. Three values in sixteen. ⭐ The repair is `Option<Payload>` on all
three: **any field on a GREASE codepoint takes an arbitrary body.**

### `hyperium/h2`

**Verdict: confirms, exactly, including the seam, and the patch already exists.**

- Both HEADERS constructors set `stream_dep: None`
  ([`src/frame/headers.rs:120`](../../references/hyperium__h2/tree/src/frame/headers.rs)
  and `:141`) and nothing on the send path sets it. The `PRIORITY` flag constant
  exists at `:113` and the field **is** parsed on receive at `:180`.
- ⭐ `EncodingHeaderBlock::encode` (`:650-660`) takes `f: FnOnce(&mut
  EncodeBuf<'_>)`, calls it at `:667` between the frame head and the HPACK
  block, and computes the payload length at `:692` **after** it ran. So bytes
  written in that closure are counted in the frame length and in any
  CONTINUATION split with no hand arithmetic.
- `PushPromise::encode` (`:498-514`) already uses it to write the promised
  stream id. `Headers::encode` (`:277-291`) passes `|_| {}`.

⭐ **So the change is: populate `stream_dep` and write its five bytes in the
closure that is already there.** The seam is real and the arithmetic is
somebody else's.

⭐ **It has been done, and the diff is in the corpus.**
[`patches/h2/0004-src-frame-headers.rs.patch`](../../references/Azathothas__bit-cli/tree/patches/h2/0004-src-frame-headers.rs.patch)
adds `StreamDependency::encode` and `Headers::set_stream_priority`, which sets
the payload and the flag in one call because a head with the flag and no block
is a frame a peer cannot parse. Its rationale, and what it unblocks, is under
`## h2: a client cannot open a stream with the PRIORITY block a browser sends`
in [`patches/UPSTREAM.md`](../../references/Azathothas__bit-cli/tree/patches/UPSTREAM.md).
Two tests there assert the wire bytes directly: `80 00 00 00 ff` after a nine
byte head with the PRIORITY flag set, and no flag and no block when no priority
is given.

⚠ **Upstream is unlikely to take it, for a stated reason rather than by
neglect.** RFC 9113 section 5.3.1 deprecates stream priority, so a release
adding a way to send one would add a way to do what the specification tells
clients not to do. `EMIT-03` inherits that: the patch is this project's to
carry, not to upstream.

### `refraction-networking/utls`

**Verdict: adopt, and it removes a problem the brief treats as open.**

⭐ **The escape hatch the brief says must be designed for already exists here.**
`GenericExtension { Id uint16; Data []byte }`
([`u_tls_extensions.go:875-878`](../../references/refraction-networking__utls/tree/u_tls_extensions.go))
inside `ClientHelloSpec { CipherSuites []uint16; CompressionMethods []uint8;
Extensions []TLSExtension; ... }`
([`u_common.go:184-196`](../../references/refraction-networking__utls/tree/u_common.go)).
Ordered, codepoint-keyed, arbitrary body.

⭐ **The default refuses rather than dropping.** `ClientHelloSpec.FromRaw`
returns `unsupported extension %d` for an unknown codepoint unless
`allowBluntMimicry` is set, in which case it keeps the codepoint **and its
bytes** (`u_common.go:258-265`). Refusing beats a silent drop, and the flag is
documented as a mimicry-versus-safety trade at
[`u_fingerprinter.go:8-22`](../../references/refraction-networking__utls/tree/u_fingerprinter.go).

⚠ **And there is a dead branch worth learning from.** In the JSON path,
`u_common.go:335-341` returns an error when the type assertion fails, then tests
`if extension == nil || !ok` and logs "Unsupported extension %d added as a
&GenericExtension without Data". That branch cannot be reached: both of its
conditions imply the early return above it. ⭐ **The unknown-is-not-absent rule
needs a test, not a branch.** Here the branch exists and cannot fire.

`Fingerprinter.RawClientHello`
([`u_fingerprinter.go:44-58`](../../references/refraction-networking__utls/tree/u_fingerprinter.go))
takes the full TLS record including both headers and returns a spec. That is the
normaliser shape this project needs, with a stated input contract.

### `Kikobeats/https-tls`

**Verdict: anti-pattern exhibit, and the cheapest one to read.**

It maps a User-Agent string to Node TLS options. Three defects, all at file and
line, all in 388 lines:

1. ⛔ **One table per family, dated by comment.**
   [`src/index.js:57`](../../references/Kikobeats__https-tls/tree/src/index.js)
   labels the Chrome cipher list "Chrome v92" and `:80` labels Firefox "Firefox
   v91". A Chrome 152 User-Agent gets Chrome 92's ciphers. That is a new User-
   Agent over an old hello, by construction rather than by neglect.
2. ⛔ **The classifier cannot produce a family the data has.**
   [`src/browser.js`](../../references/Kikobeats__https-tls/tree/src/browser.js)
   returns `firefox`, `chrome` or `safari`, and **anything unrecognised is
   safari**. `src/headers-order.json:119` carries an `edge` key that nothing can
   reach.
3. ⛔ **The header ordering silently does nothing on the input a modern client
   produces.** `sortHeaders`
   ([`src/headers.js:20-45`](../../references/Kikobeats__https-tls/tree/src/headers.js))
   matches with `key in headers` against a capitalised list (`Host`,
   `User-Agent`), while `getHeader` lowercases. A caller passing lowercase names,
   which is the HTTP/2 shape, gets every header in insertion order and no error.
   It is the step that succeeds having done nothing it was asked to do.

⭐ **Checks these three suggest**, and each is cheap: a profile's classifier
must be able to produce every family the data carries; a profile's cipher list
must belong to the build its version claims; and a transformation that can
no-op must assert its own effect.

### `microlinkhq/top-user-agents` and `EIGHTFINITE/top-user-agents`

**Verdict: adopt the publishing shape; the pair is its own lesson.**

microlink's is a top-100 User-Agent list from about 300 million monthly
requests, auto-updated weekly, published as three flat JSON files
(`src/index.json`, `src/desktop.json`, `src/mobile.json`) fetched over a CDN. ⭐
**That is a working example of the flat, index-free consumption route this
project needs**, maintained by a cron rather than by a person.

⚠ **Two copies of one dataset, both `top-user-agents` version `2.1.132`, both
naming microlink as their repository, do not contain the same data.** Measured:
microlink has 100 entries, EIGHTFINITE has 99, and the missing one is an
Electron application's User-Agent
(`... ScalboostBrowser/4.5.8 Chrome/144.0.7559.177 Electron/40.6.0 ...`).

⭐ **A version number that does not pin its content lets two different contents
share one version.** That is the argument for content addressing and for a
checksum beside every published file, in one sentence, from two files anybody
can diff.

⚠ **And note what a popularity list contains.** An Electron application's
User-Agent is among the hundred most-seen strings. "Most used on the internet"
and "what a browser sends" are different questions, and only the second one is
this project's.

### `pkgforge-security/Wordlists`

**Verdict: adopt the naming; ⛔ do not adopt the trailing newline.**

[`Misc/User-Agents/`](../../references/pkgforge-security__Wordlists/tree/Misc/User-Agents/)
uses `ua_<browser>_<platform>_<latest|all>.txt`: flat, guessable, no index
needed, one value per `_latest` file.

⛔ **Measured with `od -c`: `ua_chrome_windows_latest.txt` ends with a
newline**, and so does `ua_safari_macos_latest.txt`. A `curl` consumer of a
single-value route therefore has to strip it, which is exactly the burden this
project's route design must not impose.

⭐ **So the requirement is testable rather than aspirational**: a check asserts
that every single-value route file's last byte is not `\n`. The model the
operator pointed at is also the reproduction of the defect.

⚠ The scheme has no channel and no exact-version dimension, and `all` is thin
outside Chrome: `ua_firefox_windows_all.txt` is 81 bytes, one entry. This
project's routes need more axes than this one has.

### `damianobarbati/get-browser-fingerprint`

**Verdict: filed elsewhere, and it settles a scope question.**

It hashes canvas, WebGL, WebGPU, audio, fonts, `Intl`, screen, device memory and
permission state into an eight-character identifier, entirely from JavaScript
inside a page.

⭐ **It is the complement of this project, not a competitor.** A detector sees
both surfaces; this project measures the network one and that one measures the
script one. The scope boundary should say so by name, because "browser
fingerprint" means that project to most readers and this one to almost nobody.

⚠ Its README states its assumptions honestly, including that it targets stock
installations with no privacy extensions. That is the kind of limits section
worth copying.

### `daijro/browserforge`

**Verdict: anti-pattern exhibit, and the purest one.**

A Python reimplementation of Apify's `fingerprint-suite`
([`README.md:39`](../../references/daijro__browserforge/tree/README.md)) that
**generates** header sets and fingerprints by sampling a Bayesian network
([`browserforge/bayesian_network.py`](../../references/daijro__browserforge/tree/browserforge/bayesian_network.py)).

⛔ A sample from a conditional distribution is **plausible**, not **observed**.
It is the derived value this project's first rule refuses, and a combination it
produces may exist nowhere. ⚠ The network data is not in the repository; only
`browserforge/injectors/data/utils.js.xz` ships, so the model itself is fetched.

⚠ It is headers and script-surface only. Nothing in it touches TLS or HTTP/2
settings, so it is not a competitor on the network layer either.

### `daijro/camoufox`

**Verdict: confirms the consumer story, and its scope is narrower than its name.**

39 patches under
[`patches/`](../../references/daijro__camoufox/tree/patches/). Grep for `nss`,
`ssl` or `tls` across all of them matches one file, and that file is the
Playwright patch. ⛔ **Nothing in this project patches the TLS stack.**
[`network-patches.patch`](../../references/daijro__camoufox/tree/patches/network-patches.patch)
touches `netwerk/protocol/http/nsHttpHandler.cpp` and `moz.build` only, for
Accept-Language, User-Agent and request urgency.

⭐ **So it spoofs the script surface and two HTTP headers, and ships stock
Firefox's `ClientHello`.** That makes it a natural consumer of measured network
profiles rather than a source of them, and it is the honest way to describe the
relationship: this project publishes data such a project could use, and makes no
claim about what it is used for.

### `adryfish/fingerprint-chromium`, `botswin/BotBrowser`, `CloakHQ/CloakBrowser`

**Verdict: anti-pattern exhibits. Three shapes of one pattern, all checkable.**

| project | the shape | the evidence |
| --- | --- | --- |
| `adryfish/fingerprint-chromium` | **source withheld on a timer.** The default branch contains three files: `README.md`, `README-ZH.md`, `LICENSE`. The README states the policy: binaries now, patch files "when the next version is published (typically one month later)". | `find tree -type f` returns three paths |
| `botswin/BotBrowser` | **open licence, encrypted data.** MIT `LICENSE`; profiles ship as `.enc` files which are JSON envelopes with a base64 `key` and a base64 `profile` ciphertext. Only `userAgent`, `userAgentData`, `screen`, `window` and the GPU strings are in the clear. The README directs readers to a subscription for current profiles. | `jq -r '.profile \| type' profiles/stable/chrome148_win11_x64.enc` returns `string` |
| `CloakHQ/CloakBrowser` | **MIT wrapper, proprietary binary.** `LICENSE` is MIT and covers the Python and JavaScript wrapper. `BINARY-LICENSE.md` is "All rights reserved" and requires "an active paid subscription" for the latest major version. The advertised "73 source-level C++ patches" are not in the repository. | `find tree -name '*.patch' -o -name '*.cc' -o -name '*.cpp' -o -name '*.diff'` returns nothing |

⭐ **The transferable rule is not about anybody's business model. It is that a
repository should be checkable against its own README**, and that a project
whose product is data has to ship the data. Both are things a check can hold
here, and both belong in this project's own rules rather than in a complaint
about somebody else's.

⚠ **Read these as engineering exhibits and nothing else.** Each has a defect
worth recording and none of them owes this project anything.

---

## What was refused

| refused | reason |
| --- | --- |
| copying any fingerprint value out of `bit-cli`, `impit`, `curl-impersonate` or `https-tls` into this project's corpus as data | every one would be a value this project did not measure, which the first rule makes a draft. They are recorded in [`../inherited-claims.md`](../inherited-claims.md) as claims to re-measure, never as corpus entries. |
| copying source out of `Azathothas/bit-cli` | MIT attribution would travel with it into a tree whose output is 0BSD. The mechanisms are cited at file and line and reimplemented. |
| copying JA4 implementation source from `FoxIO-LLC/ja4` | BSD-3 attribution, for the same reason. Implement from the published specification instead. |
| emitting any JA4+ variant | FoxIO License 1.1, patent pending, monetisation-restricted. Finding 5. |
| adopting `impit`'s boolean-per-extension model, or `https-tls`'s derive-from-User-Agent model | both are the failure this project exists to remove |
| treating `browserforge`'s generated distributions as a source of profiles | generated is not measured |

---

## What the second revision corrected

⭐ **Listed because a count of what a previous revision got wrong is the only
honest estimate of what this one still does.** All five came from fetching the
origin repository, which the first revision cited throughout and never opened.

1. ⛔ **The PRIORITY block was recorded as contested three ways with no
   measurement.** It was measured, off frame bytes, on two versions. Finding 1.
2. ⛔ **The Chrome 152 header list carried a `cache-control` field the capture
   does not have**, and a paragraph explaining a difference that does not
   exist.
3. ⚠ **The corpus was counted as sixteen projects and was seventeen.** Every
   document that repeated the number repeated the error.
4. ⚠ **`apify/rustls`'s single fixed GREASE slot was recorded as unconfirmed**
   after a grep found nothing. It is in the origin repository's vendored copy,
   at codepoint `0xbaba`, named by `T-263`.
5. ⚠ **The stream-weight units question was left open** as "may be the same
   measurement in two units". It is: the origin's own type comment says the wire
   weight is one less than the specification's.

---

## ⚠ The claims here most likely to be wrong

Listed before anybody acts on them, and in the order a reader should doubt them.

1. **That `bit-cli`'s Chrome 151 and 152 numbers describe those builds
   generally.** They are one capture each, from one instrument, on one host and
   in one container, by one author. The instrument is committed and the numbers
   are re-derivable; nothing makes them a second opinion.
2. **That `reqwest` is the only layer missing `header_table_size`.** Taken from
   a tracker comment. Neither `reqwest` nor `hyper` was fetched.
3. **That `apify/rustls` upstream adds no RFC 8701 GREASE.** Established by grep
   over `rustls/src/`, which finds what it searches for. A capability
   implemented under a name this sweep did not think of would not appear.
4. **That `0xca34` is the trust-anchors draft.** The codepoint matches and the
   body's shape is consistent with it. ⛔ No specification was read against the
   bytes: the body is measured and the name is inferred.
5. **Every line number.** They are correct at the commits in the provenance
   table and nowhere else.

⛔ **Assume more remain.**
