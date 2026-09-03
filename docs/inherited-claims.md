# inherited-claims.md

Every value this project carries that was measured somewhere else, with its
source, its date, its status, and the entry that re-measures it.

⭐ **This file exists because the project's first rule is that a fingerprint is
measured, never derived and never inherited.** Every row here is `vendor`
provenance by the definition in [`glossary.md`](glossary.md), which makes it a
draft, a starting point and a test case, and never a corpus entry.

⚠ **Two sections carry a measurement taken here, and section 12 says what
happens to a row that gets one.** Section 5 is the first quantity this project
read off a browser's own wire, on 2026-09-01, beside the reading it inherited,
and ⭐ **it has since left this file**: the block is published, in the profile
that section names. Section 7 re-asked the version endpoints from this machine.

⛔ **Nothing here may be copied into a published profile.** A value nobody here
can re-measure is a value nobody should trust, and copying one across would put
an unverifiable number behind a `wire` label.

⚠ **When a document in this tree states a fingerprint, a version, a codepoint or
a constant, this file is where its provenance lives.** If the two disagree, this
one is right and the other is the defect.

---

## How to read the status column

| status | meaning |
| --- | --- |
| `inherited` | one source, not re-measured here, no known contradiction |
| ⛔ `contested` | two or more sources disagree, or one source contradicts itself. The disagreement is recorded and neither reading is preferred. |
| ⛔ `refuted` | a reading of a reference's own source or specification shows the claim is wrong. The original wording is kept and the correction is written underneath. |
| `confirmed-by-reading` | checked against a reference's source at a named commit. ⚠ Still not a wire measurement **here**. |
| ⭐ `measured-here` | read off a socket by this project's own harness, with the browser, the build, the date and the conditions recorded. The only status that can become a corpus entry. |

⚠ **`confirmed-by-reading` is the weakest kind of confirmation that gets a
name.** It says a claim about somebody's code matches that code. It says nothing
about what a browser puts on a socket.

---

## The sources

| tag | what it is |
| --- | --- |
| ⭐ `ORIGIN` | [`Azathothas/bit-cli`](../references/Azathothas__bit-cli/) at commit `cce8131231abe8b232054f3f27b3feeac19dd411`, fetched 2026-08-31. ⭐ **Every value the founding brief carried was measured there, off a socket, with a committed instrument.** Each row below cites the file. ⚠ Its licence is MIT: this project cites it and copies no source out of it. |
| `SWEEP` | [`reference-sweeps/findings.md`](reference-sweeps/findings.md), this project's own reading of eighteen repositories. Source-level, never a wire. |
| `TRACKER` | An issue or comment on a third-party repository, fetched into [`../references/`](../references/). ⛔ Evidence of what somebody believed. |
| `BRIEF` | ⚠ **The founding brief**, defined below. A row tagged `BRIEF` alone is one the origin tree does not carry, so nothing can check it. |

### ⚠ The founding brief

**The design document this repository was created from.** It was written in
`Azathothas/bit-cli` between 2026-08-29 and 2026-08-30, describing a project
that did not exist yet, and it was retired on 2026-08-31 once its content was in
this tree. It is not tracked here and nothing in this tree depends on it.

⛔ **It was self-reviewed four times, peer-reviewed zero times, and nothing had
ever been built from it.** It said so itself.

⚠ **It is not the same thing as `ORIGIN`, and conflating them is what went
wrong the first time.** The brief is prose about a tree; the tree is the
evidence. Where the two disagree the tree wins, and section 11 lists the places
they did.

---

## 1. Chrome fingerprints

⭐ **Two captures, both from `ORIGIN`, both with a capture instant and a named
instrument.** The instrument is
[`loopback-tlsprobe`](../references/Azathothas__bit-cli/tree/crates/bit-cli-core/examples/loopback-tlsprobe/);
[`reference-sweeps/usable.md`](reference-sweeps/usable.md) section 14 is its
shape.

| | Chrome 151, Windows | Chrome 152, Linux container |
| --- | --- | --- |
| exact build | `151.0.7922.72` | `152.0.7977.64` |
| channel and source | stable, the vendor's own installation | stable, Chrome for Testing, ⚠ **unbranded** |
| captured | `2026-08-30T02:29:47.449Z` | `2026-08-30T02:08:35.149Z` |
| artefact | `fingerprints/bit-cli-browser.json` | `bench/browser-fingerprint-cft-152.json` |
| JA4, cold | `t13i1515h2_8daaf6152771_806a8c22fdea` | `t13i1517h2_8daaf6152771_4980c97edce0` |
| Akamai | `1:65536;2:0;4:6291456;6:262144\|15663105\|1:1:0:255\|m,a,s,p` | **identical** |

All `inherited`.

⛔ **The two JA3 hashes are recorded in the origin tree and are deliberately not
carried here.** JA3 preserves wire order and a browser shuffles, so it changes
per connection: an inherited JA3 cannot be compared against a re-measured one
and would be a number with no use. ⚠ It is also a bare 32-character hexadecimal
string, which the secret sweep refuses by shape, and `TOOL-03` is the entry that
owns that collision. Both artefacts in the table above carry theirs.

The JA4_r forms, which are readable where a JA4 is not:

```text
151  t13i1515h2_002f,0035,009c,009d,1301,1302,1303,c013,c014,c02b,c02c,c02f,
     c030,cca8,cca9_0005,000a,000b,000d,0012,0017,001b,0023,002b,002d,0033,
     44cd,fe0d,ff01_0904,0905,0906,0403,0804,0401,0503,0805,0501,0806,0601

152  t13i1517h2_002f,0035,009c,009d,1301,1302,1303,c013,c014,c02b,c02c,c02f,
     c030,cca8,cca9_0005,000a,000b,000d,0012,0017,001b,0023,002b,002d,0033,
     12e0,44cd,ca34,fe0d,ff01_0904,0905,0906,0403,0804,0401,0503,0805,0501,
     0806,0601
```

⭐ **The cipher list and the signature algorithms are byte-identical across the
two versions, and the extension list differs by exactly two codepoints**,
`12e0` and `ca34`. That is section 3, and it is readable here only because the
raw form was kept.

| claim | value | source | status |
| --- | --- | --- | --- |
| Chrome 152, the same build, resumed JA4 | `t13i1518h2_8daaf6152771_3d1b1b7bef36` | `BRIEF` | ⚠ inherited, and **not in the origin tree**. The resumption behaviour is recorded there and this exact digest is not. |
| Both JA4s begin `t13i` rather than `t13d` | because the capture dialled an IP literal, so no SNI was sent | `ORIGIN` | inherited |
| Chrome 151 on Windows and on a hosted Ubuntu runner produced the same JA4 and the same Akamai string | one version, two platforms | `ORIGIN` | inherited. ⚠ One data point. It does not establish that the TLS half is platform-independent in general. |
| Chrome 151's User-Agent | `Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36` | `ORIGIN`, `page.rs:113` | inherited |
| Chrome 152's User-Agent, in a Linux container, ⚠ after headless normalisation | `Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/152.0.0.0 Safari/537.36` | `ORIGIN` | inherited. ⭐ The capture records `headless_rewritten: ["user-agent"]`, so the rewrite is reported rather than silent. |

⚠ **A JA4 is only comparable against a capture taken the same way.** Whether SNI
was sent, whether the session resumed, and which connection of a navigation was
kept all change it. Any re-measurement records those three before the value.

⚠ **Three builds of Chrome 151 appear in the origin tree** and they are not
interchangeable: `151.0.7922.72` is the build the profile was captured from,
`151.0.7922.76` was on the capture host on 2026-08-29, and `151.0.7922.173` was
on a hosted Ubuntu runner. ⛔ The brief wrote all of them as `151.0.7922.7x`,
which hid which one produced which number.

---

## 2. GREASE, as Chrome sent it

All `ORIGIN`, all `inherited`. Recorded across **two versions and two
platforms**, which is what makes the per-connection claim a pattern rather than
a reading.

| claim | evidence |
| --- | --- |
| GREASE appears at **both ends** of the extension list, first and last | both captures |
| the two values differ within one capture | 152 drew `0x3a3a` first and `0x5a5a` last; 151 drew `0x6a6a` first and `0x4a4a` last |
| the bodies differ and are not both empty | ⭐ the first is **empty**; the last is **one zero byte** |
| GREASE also leads the cipher list and the key-exchange-group list | both, asserted in the origin tree's own tests at `page.rs:1604-1605` |
| the values are redrawn per connection | ⭐ two consecutive captures of one client drew `0x6a6a`/`0x7a7a`, then `0x7a7a`/`0x0a0a` |
| the codepoints are drawn **distinct** | the same value at both ends is a constant a server can key on |
| the extension order is shuffled per connection, and has been since Chrome 110 | the two recorded orders below share no prefix |
| two things stay pinned | `pre_shared_key` last, and GREASE at the two ends |
| ⭐ `alpn` is **inside** the shuffle | position 9 of the Chrome 152 hello, so it is not one of the extensions Chrome pins |

The recorded extension orders, one draw each:

```text
151  6a6a GREASE  0005  0033  000a  44cd  0023  002d  ff01  001b
     000d  000b  0017  fe0d  002b  0012  0010  4a4a GREASE

152  3a3a GREASE  0023  ff01  002d  000b  000a  12e0  0017  0010
     ca34  fe0d  0033  001b  000d  0012  44cd  002b  0005  5a5a GREASE
```

⚠ **Each sequence is one draw.** A re-measurement that reproduces one exactly
should be treated as suspicious rather than as confirmation: two consecutive
captures of one binary must produce two different orders, or the capture is
wrong.

⭐ **Only the raw `ClientHello` bytes can check any of this.** Every digest,
JA4_ro included, strips GREASE before it is computed.
[`reference-sweeps/findings.md`](reference-sweeps/findings.md) finding 4.

---

## 3. The two extensions nobody could reproduce

| codepoint | length | body | what it is | source | status |
| --- | --- | --- | --- | --- | --- |
| `0x12e0` | 2 | `0000` | still unidentified. Two zero bytes, trivially reproducible by anything that can write an arbitrary extension. | `ORIGIN` | inherited, and ⭐ **narrowed by measurement 2026-09-02**: absent from all three Chrome `151` profiles this project has captured, on both platforms, while the origin's `152` capture carries it. `CORPUS-05`. |
| `0xca34` | 206 | a length-prefixed list of 24 identifiers, in the browser's own order | trust anchors, `draft-ietf-tls-trust-anchor-ids` | `ORIGIN` | ⚠ the **body** is measured; the **name** is inferred. No specification was read against the bytes. ⭐ Also absent from every Chrome `151` profile here. |

⛔ **These two are why the origin repository's version bump did not ship.**
Neither `impit`'s closed extension enum nor `rustls`'s one-typed-field-per-
extension struct can name a codepoint learned at runtime, so a profile claiming
Chrome 152 would have been a `ClientHello` that exists nowhere. ⭐ **The ruling
was to stay one version behind, deliberately.** That is this project's third
absolute, reached by somebody paying for it.

⛔ **`0xca34` is a design problem rather than a value.** Its body is a snapshot of
the browser's own root store, so it changes when that store changes, and a
client carrying one build's list is advertising which build it copied. A client
with no root store of its own has three options and none is obviously right:
omit it, carry a captured list, or send it empty, which is a shape no browser
sends.

⭐ **Publishing per-build trust-anchor lists with their capture dates, and a
documented recommendation, is a service nobody currently provides.** `CORPUS-04`
is the entry and [`trust-anchors.md`](trust-anchors.md) is what it produced:
the three options with the cost of each, and no preference asserted.

⭐ **`0xca34` is MEASURED HERE now**, in Chrome `152.0.7977.75` on `linux64` and
`152.0.7977.76` on `win64`, both 2026-09-02. The length matches the row above at
206 bytes on both. ⛔ The body decodes to **32** identifiers where this row's
source records 24, and both can be right: the list is a snapshot of a root store
and it changes per build. ⛔ **The two platforms carry the same 32 identifiers in
a completely different order**, all 32 positions differing, which
[`trust-anchors.md`](trust-anchors.md) states as a second decision for anybody
copying a list. ⚠ The name is still inferred and none of this settles it.

---

## 4. HTTP/2 settings and the connection window

| claim | value | source | status |
| --- | --- | --- | --- |
| Chrome's `SETTINGS_HEADER_TABLE_SIZE` | `65536` | `ORIGIN` (`page.rs:122`), `TRACKER` (two independent reports), `SWEEP` (`curl-impersonate` wrapper scripts) | inherited, ⭐ three sources agreeing, one of them a wire capture |
| Chrome does **not** send `SETTINGS_MAX_FRAME_SIZE` | absent | `ORIGIN` (`page.rs:130`), `TRACKER` | inherited, two sources agreeing |
| Chrome's `SETTINGS_INITIAL_WINDOW_SIZE` | `6291456` | `ORIGIN` (`page.rs:269`), `TRACKER`, `SWEEP` | inherited, three sources agreeing |
| Chrome's `SETTINGS_MAX_HEADER_LIST_SIZE` | `262144` | the same three | inherited |
| Chrome's `SETTINGS_ENABLE_PUSH` | `0` | the same three | inherited |
| Chrome sends settings 1, 2, 4 and 6 and no others | the SETTINGS list, in that order | `ORIGIN`, `page.rs:117-121` | inherited |
| Chrome's connection WINDOW_UPDATE increment | `15663105`, which is 15 MiB minus the protocol's 65,535 default | `ORIGIN`, `TRACKER`, `SWEEP` | inherited, three sources agreeing |
| Chrome's pseudo-header order | `:method, :authority, :scheme, :path`, rendered `m,a,s,p` | `ORIGIN`, both captures' Akamai strings | inherited. ⚠ The origin's own constant at `page.rs:307` carries six names, adding `:protocol` and `:status`; a capture of a navigation shows four. |
| Chrome's ALPN list | `h2`, then `http/1.1` | `ORIGIN`, `page.rs:266` | inherited |
| Chrome 116 sent `SETTINGS_MAX_CONCURRENT_STREAMS` as `3:1000`; a Chrome 150 report has no setting 3 | | `SWEEP` | inherited. ⭐ A settings key that appears in one version and not a later one. |
| Firefox's connection window | 12 MiB, so an increment of `12517377` | `TRACKER` | inherited |
| iOS 18's connection window | 10 MiB, so an increment of `10420225` | `TRACKER` | inherited |

⛔ **The window-versus-increment confusion is real and shipped.** In one
reference database the same field holds the window in one entry and the
increment in seven others, and the seven emit a value 65,535 short.
[`reference-sweeps/usable.md`](reference-sweeps/usable.md) section 2 has the
audit and the two other quantities with the same shape.

⚠ **The origin tree stores the window and subtracts in the emitter**, at
`page.rs:277`, and says so in the type's own comment. That is the other
defensible choice, and it is only defensible because it is named. ⛔ This project
records the **increment**, because the increment is what a capture reads.
`SCHEMA-09` is the entry that names every field for the wire.

---

## 5. ⭐ The PRIORITY block: measured off frame bytes

⭐ **This was recorded here as contested three ways with no measurement on any
side. It is not.** One of the readings is a frame-byte read and the other two
are rendered Akamai strings.

| source | reading | what it read |
| --- | --- | --- |
| ⭐ `ORIGIN`, Chrome 151 and Chrome 152 | stream 1, **exclusive**, dependency **0**, weight **255**, rendered `1:1:0:255` | the HEADERS frame's flags byte for `0x20` and the five bytes behind it, at `h2fp.rs:219-223` |
| `SWEEP`, `curl-impersonate` signatures for Chrome 118, 119 and 120 | the Akamai priority field is `0` | a rendered string |
| `TRACKER`, a Chrome 150 capture through a third-party service | the Akamai priority field is `0` | a rendered string |
| `ORIGIN`, its own client before it patched `h2` | `0` | ⭐ what a stack that cannot write the block produces |

⭐ **The fourth row explains the two zeros rather than contradicting them.** A
`0` in that field is what an unpatched `h2` emits, and two of the three sources
reporting one are reading a tool rather than a browser.

⚠ **The units trap is settled with it.** HTTP/2 encodes weight as `weight - 1`,
so a tool taking `256` puts `255` on the wire. `ORIGIN`'s own type comment says
so at `page.rs:293-299`. `curl-impersonate`'s `256` and this `255` are one
quantity in two units.

### ⭐ Measured here on 2026-09-01, and the two agree

`HARNESS-05` took the capture. The harness terminated the handshake, read the
HEADERS frame's flags byte for `0x20`, skipped no pad byte because none was
set, and decoded the five bytes behind it.

| browser | build | terminated connections | the five raw bytes | parsed |
| --- | --- | --- | --- | --- |
| Chrome | `151.0.7922.76` | 6 of 7 accepted | `80000000ff` on every one | exclusive, dependency 0, weight 255 on the wire |
| Edge | `152.0.4191.53` | 7 of 8 accepted | `80000000ff` on every one | exclusive, dependency 0, weight 255 on the wire |

⭐ **That is the reading in the first row of the table above, taken here.** The
inherited value and the measured value agree exactly, and the two rows
reporting `0` stay explained rather than contradicted: both read a rendered
string, and one of them read a client that could not write the block.

⚠ **The conditions, because a measurement carries them or it is not a
measurement.** One Windows 11 host, 2026-09-01. Both browsers launched into a
throwaway profile directory and pointed at a loopback address. ⛔ The run's own
authority was NOT installed into the machine trust store: each browser was
given `--ignore-certificate-errors-spki-list` carrying the base64 SHA-256 of
that one key, so exactly one key was trusted for one launch.
⭐ **`HARNESS-10` has since measured the surface**, though not the pin: the
raw and terminating surfaces agree on every TLS field that has a stable value,
so the reading above is not an artefact of the handshake having completed.
What a trust store would do instead of a pin is still unmeasured.

⚠ **The first connection of each run terminated nothing.** It is the preconnect
a browser opens and abandons, and it carries no HTTP/2 at all.

⭐ **It is `measured-here` and it is now PUBLISHED**, which is the first value
in this document to leave it. `CORPUS-01` wrote the profile on 2026-09-01 and
the block is in it, as `http2.stream_priority` beside the frame bytes it was
read from in `raw.http2_frames_hex`:

```bash
node -e "const p=require('./corpus/v1/chrome/stable/win64/151.0.7922.76.json'); console.log(JSON.stringify(p.http2.stream_priority))"
```

⚠ **The published profile is the Chrome row above and not the Edge one.** One
profile is one build on one platform, and the Edge reading stays here, measured
and unpublished, until a capture of it is written too.

---

## 6. Header sets

From `ORIGIN`, `inherited`.

Chrome 151, Windows, top-level navigation, in order, after the pseudo-headers:

```text
sec-ch-ua, sec-ch-ua-mobile, sec-ch-ua-platform, upgrade-insecure-requests,
user-agent, accept, sec-fetch-site, sec-fetch-mode, sec-fetch-user,
sec-fetch-dest, accept-encoding, accept-language, priority
```

Chrome 152, Linux, same request kind:

```text
sec-ch-ua, sec-ch-ua-mobile, sec-ch-ua-platform, accept-language,
upgrade-insecure-requests, user-agent, accept, sec-fetch-site, sec-fetch-mode,
sec-fetch-user, sec-fetch-dest, accept-encoding, priority
```

⭐ **`accept-language` moved from twelfth to fourth.** Thirteen fields in both
captures, and that is the only difference in the order.

⛔ **The founding brief listed a fourteenth field, `cache-control`, in the 152
capture, and spent a paragraph on why it might be there.** The capture has no
`cache-control`, and `max-age=0` appears nowhere in the origin tree. Section 11
carries the refutation. ⚠ There is no version-versus-request-kind question to
isolate, and `SCHEMA-04`'s `variants` field is owed no explanation of it.

The Chrome 152 values, from the container capture:

| header | value |
| --- | --- |
| `sec-ch-ua` | `"Not?A_Brand";v="24", "Chromium";v="152"` |
| `sec-ch-ua-mobile` | `?0` |
| `sec-ch-ua-platform` | `"Linux"` |
| `accept-language` | `en-US,en;q=0.9` |
| `accept-encoding` | `gzip, deflate, br, zstd` |
| `priority` | `u=0, i` |

⭐ **Chrome for Testing is unbranded, and it is a hard limit.** That `sec-ch-ua`
has no vendor entry at all.

| you need | use | you also get |
| --- | --- | --- |
| any channel, including beta, dev and canary | Chrome for Testing | an unbranded `sec-ch-ua` |
| the real brand list | the vendor's own package, which on Linux is their apt repository | stable and no other channel |

Both give the host platform's own `sec-ch-ua-platform` and User-Agent OS token,
so a Windows profile captured on Linux needs those substituted, which is exactly
what a per-field provenance map is for.

⚠ **`accept-encoding` is a coherence constraint, not just a value.** The origin
tree advertises those four because those four are what its own decompression is
built with, and a client advertising an encoding it cannot decode hands
compressed bytes to a parser that cannot read them. `VALID-01` check 6.

---

## 7. Version discovery, and the endpoint that misleads

**The defect, from `ORIGIN`,
[`scripts/check-browser-version.ps1:25-37`](../references/Azathothas__bit-cli/tree/scripts/check-browser-version.ps1),
`inherited`.** A version-history endpoint queried for the highest version on a
channel answers with the highest version **known**, which during a staged
rollout is a build almost nobody has. Measured 2026-08-29:

| source | answer |
| --- | --- |
| the channel's versions endpoint, one page, highest first | `153.0.8010.12` |
| that build's rollout fraction | ⚠ **0.005** |
| the build at fraction 1 | `152.0.7977.65` |
| Chrome for Testing, stable | `152.0.7977.64` |

⭐ **So read the releases endpoint and the rollout fraction, take the highest
version at fraction 1, cross-check against the automation build index, and print
the highest published version and its fraction beside the answer** so a reader
can check the choice rather than take it. Chasing a build one user in two
hundred has produces a correct fingerprint of a browser that does not exist.
[`reference-sweeps/usable.md`](reference-sweeps/usable.md) section 12 is the
shape.

⚠ **Note that even the two "settled" answers disagree by one patch component**
(`.65` against `.64`). Two first-party sources, two answers, and the difference
is the finding.

### ⭐ Measured here on 2026-09-01, and every number came back the same

`DRIVER-02` asked both endpoints from this machine, three days after the reading
above was taken elsewhere. The highest known build, its fraction, the build at
full rollout and the automation index's answer are identical, and the two
sources still disagree by one patch component.

```bash
cargo run -p b-ids-driver -- versions --channel stable
```

⭐ **And the control explains the mechanism rather than only confirming it.**
Asked about beta, the two sources agree, and the build they name is
`153.0.8010.12`: the same build stable lists at `0.005`. What a stable channel
"knows about" during a staged rollout is the next channel's build arriving.

⚠ **The status of this section is `measured-here` for the four numbers above and
`inherited` for everything else in it.** The channel state below is stale by
construction and was not re-taken.

**Channel state on 2026-08-30**, from `ORIGIN`, `inherited` and ⚠ **stale by
construction**: stable `152.0.7977.64`, beta `153.0.8010.12`, dev
`154.0.8025.0`, canary `154.0.8032.0`.

⚠ **A third-party user-agent list fetched during the sweep also names Chrome
152 as current on Windows**, which is consistent, and is not independent
evidence: a popularity list is not a version-history source.

**Endpoints worth knowing**, from `ORIGIN`. ⚠ Not called from this repository.

| question | endpoint |
| --- | --- |
| Chrome versions with rollout | `https://versionhistory.googleapis.com/v1/chrome/platforms/{platform}/channels/{channel}/versions/all/releases` |
| Chrome for Testing, per channel, with download URLs | `https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json` |
| Firefox | `https://product-details.mozilla.org/1.0/firefox_versions.json` |
| Edge | `https://edgeupdates.microsoft.com/api/products?view=enterprise` |

---

## 8. Traps in taking a capture

Each cost somebody a day. All from `ORIGIN` unless noted, all `inherited`.

| trap | what happens |
| --- | --- |
| ⛔ ~~**Chrome on Linux does not read the user's NSS database for server authentication.**~~ | ⛔ **REFUTED 2026-09-02**, on `ubuntu-latest` with Chrome `151.0.7922.173`: a root added with exactly that command into `~/.pki/nssdb` completed 2 handshakes and 2 HTTP/2 connections. ⚠ It is not reliable either: a second round accepted no connection at all. The original wording and the measurement are in [`HISTORY/README.md`](HISTORY/README.md); `HARNESS-14` is the run. ⭐ `--ignore-certificate-errors --test-type` also completes a handshake, measured by `DRIVER-04`, and ⚠ never ship such a flag in a client. |
| ⛔ **A GREASE codepoint you name is a codepoint you must parse.** | A parser mapping a GREASE value to a typed field with an empty body rejects the GREASE extension that carries a byte, which is what a browser sends at the end of the list. ⭐ Measured: three of the sixteen reserved values were affected, so about one handshake in five. One CI run failed, the next passed over the same defect, and 64 handshakes completed 64 after the fix. **Any field on a GREASE codepoint takes an arbitrary body.** |
| ⭐ **A harness parses permissively and emits exactly.** | Those are different requirements on one table, and a codebase using one type for both gets one of them wrong. |
| ⛔ **A browser opens sockets it abandons, and it resumes.** | One navigation of Chrome 152 at a probe produced **13 connections**. The **first** carried no HTTP/2 at all, a preconnect the browser abandoned. Later connections offered `pre_shared_key` instead of `session_ticket`, because the session resumed, and the two produce different JA4s. ⭐ Keep the **first connection that completed HTTP/2**. Record a resumed one separately; never average them. |
| ⛔ **One handshake is not a sample.** | Anything drawn per connection means a single handshake tests a single draw, and a three-in-sixteen defect reaches a one-handshake check four times in five. ⭐ Measured over eleven captures of one binary: eight offered `session_ticket` and three offered `pre_shared_key`. The check makes **eight** handshakes now and asserts every one reached HTTP/2. |
| ⛔ **Headless changes the User-Agent.** | A headless capture reports `HeadlessChrome/<version>` where a browser a person runs reports `Chrome/<version>`, and on some builds the substitution reaches `sec-ch-ua` too. ⭐ Normalise it, and **report that you did**: the origin's capture carries a `headless_rewritten` list naming every field it touched. Silently rewriting a captured value is the failure the whole project is about. |
| ⚠ **Completing a handshake can change what the client offers.** | So a digest read through a terminated handshake is not the digest the client ships. The probe's `--raw` mode exists for exactly this, and the golden records which mode produced which field. |
| ⛔ **In a NAT-mode virtual machine the guest cannot reach the host's loopback, and the failure is silent.** | The listener simply never receives a connection. Bind to the host's adapter address instead, read at run time. Measured at `172.23.96.1` on one host, agreeing with what the guest's own routing table said. [`containers.md`](containers.md) is the procedure. |
| ⚠ **Everything a run creates is removed in the same run, and the removal is read back.** | A cancelled run leaves a registered machine and a rootfs image of several hundred megabytes, because cleanup in a `finally` does not survive a hard interrupt. Measured: one killed run left a registered distro and a 74.3 MiB orphaned tarball. |
| ⚠ **Pin tooling by commit and digest, read from the endpoint you will fetch from.** | A repository storing a script as CRLF in a checkout and LF in the index gives two different digests, and the check fails closed. ⛔ And a stale copy beside the launcher can win over the pin silently: one run passed both a commit and a digest, ran the sibling, and verified nothing. |

---

## 9. Emitter limits

| stack | cannot emit | source | status |
| --- | --- | --- | --- |
| rustls | an extension whose codepoint was learned at runtime | `SWEEP` | ⭐ confirmed-by-reading. Its own doc comment says unknown extensions are dropped during parsing, and the extensions struct is crate-private. |
| rustls | an arbitrary captured extension order | `SWEEP` | confirmed-by-reading, ⭐ **with a number the brief did not have: at most 65,536 orders are reachable**, because the order is a hash of a `u16` seed. |
| rustls | GREASE at two chosen codepoints | `ORIGIN`, `SWEEP` | ⭐ confirmed. The fork carries one slot at a fixed `0xbaba`; a browser sends two, drawn per connection. The origin added two more fields to reach it. |
| `h2` | the HEADERS PRIORITY block | `ORIGIN`, `SWEEP` | ⭐ confirmed-by-reading, **and patched.** Both send-path constructors hardcode no dependency; the encode closure that would carry it is passed empty. The diff is in the corpus. `EMIT-03`. |
| `hyper` and `h2` | omitting `SETTINGS_MAX_FRAME_SIZE` | `BRIEF` | ⛔ **refuted.** `ORIGIN` omits it, through a vendored `hyper` and `h2`, and names the flag `BROWSER_H2_OMIT_MAX_FRAME_SIZE` at `page.rs:130`. `SWEEP` independently reports the omission is representable below `reqwest` and not through it. |
| `reqwest` | `SETTINGS_HEADER_TABLE_SIZE`, and preserving request extensions through its builder | `BRIEF`, `TRACKER` | inherited, not verified here. ⚠ Neither `reqwest` nor `hyper` was fetched. |
| a boolean-per-extension database | any unenumerated codepoint at all | `SWEEP`, `ORIGIN` | confirmed-by-reading, and it is what stopped a real version bump |

⭐ **`utls` has the escape hatch the brief says must be designed for**, as an
ordered list of codepoint-and-body pairs, and its default **refuses** an unknown
codepoint rather than dropping it. `confirmed-by-reading`.

---

## 10. ⛔ Claims that a reading has refuted

The original wording is kept. The correction is underneath. ⛔ Never edit a
premise; write what was measured beneath it.

### "JA3 preserves wire order and does not strip GREASE"

**Refuted in half, on 2026-08-30, by `SWEEP`.** The reference implementation
defines a table of all sixteen GREASE values and filters them out of both the
cipher list and the extension list before hashing, and its README states the
intent.

⭐ **The conclusion survives and the reason changes.** JA3 is still unstable per
connection, because it preserves wire order and browsers shuffle. Record JA3,
never assert on it, assert JA4. A session that inherited the original reason
would have gone looking for GREASE values inside a JA3 string.

⚠ **And the origin tree had it right in code all along**: its JA3 takes a
`filter_grease` parameter and is computed both ways deliberately. The brief's
prose disagreed with the tree it was written in.

### "JA4_ro is the only digest that can see what JA4 deliberately hides"

**Refuted on 2026-08-30, by `SWEEP`.** The JA4 specification says the
order-preserving raw form includes the original values in the original order,
**less GREASE values**.

⭐ **So JA4_ro shows order and nothing about GREASE.** The only artefact in which
a GREASE question is answerable is the raw `ClientHello`. That strengthens the
record-the-raw-bytes rule rather than weakening it, and it arrives from the
specification rather than from a fear of parser rot.

### "Nobody publishes the corpus"

**Refuted as stated, on 2026-08-30, by `SWEEP`.** One impersonating client ships
43 per-exact-build signature files covering five browser families across
Windows, Linux, macOS and iOS, with wire-ordered ciphers, an ordered extension
list carrying lengths and bodies, and HTTP/2 frames in order.

⭐ **The narrower claim is true and checkable**: no published corpus carries a
capture date, per-field provenance, the raw hello, a channel dimension, or a
stable route with a checksum. That is the gap, and the README claims that one.

### "curl-impersonate: binary artefacts, no machine-readable corpus, per-version forks"

**Refuted in two of three parts, on 2026-08-30, by `SWEEP`.** The maintained
fork ships a machine-readable target index and the signature corpus above.
⚠ The "per-version forks" half was not examined and is neither confirmed nor
refuted.

### ⛔ "Chrome 152 sends `cache-control: max-age=0` and Chrome 151 does not"

**Refuted on 2026-08-31, by `ORIGIN`.** The brief listed fourteen header fields
for the Chrome 152 capture with `cache-control` first, and then argued that the
difference was probably a request-kind rather than a version one and that
nothing isolated it. The committed capture carries **thirteen fields and no
`cache-control`**, and `max-age=0` appears nowhere in the origin tree. The
capture's own entry states the header change as `accept-language` moving from
twelfth to fourth, which is arithmetic that holds only without a `cache-control`
ahead of it.

⭐ **So a paragraph of hedged reasoning is deleted rather than resolved.** The
`variants` field is still worth having, because a navigation, a subresource
fetch and a reload genuinely are three header sets, and it is no longer owed an
explanation of a difference that was never measured. ⚠ **The lesson is the
cheaper one**: the brief was prose about a tree, and nobody opened the tree for
a day.

---

## 11. What has never been examined at all

⛔ Stated so that nobody reads an absence as a negative result.

- **Firefox, Safari, Edge, Brave, Opera, Vivaldi and every mobile browser.** No
  capture, no reading, by this project or by the origin. Every claim that this
  project's design reaches them is reasoning.
- **QUIC and HTTP/3.** The model was drawn with them in mind and has not been
  tested against either.
- **macOS and Windows as capture hosts.** One inherited data point exists for
  Windows and nothing else.
- **Whether the TLS half is platform-dependent.** One version on two platforms
  agreed. That is one data point, and the matrix exists to answer it.
- **Any of the version-discovery endpoints in section 7**, from this
  repository.
- **The JSON shapes the brief proposed.** Never round-tripped by code.
- **`0x12e0`.** Two zero bytes, position 7 of the Chrome 152 hello. Naming it is
  `CORPUS-05`.
- ⚠ **Whether the origin's numbers reproduce.** They are one capture each, from
  one instrument, on one host, by one author. The instrument is committed and
  they are re-derivable; nothing yet makes them a second opinion.

---

## 12. Retiring a row

⭐ **A row leaves this file when the project measures the same thing itself.**
The measurement goes into the corpus with `wire` provenance, and the row here is
rewritten to say what was believed, what was measured, and whether they agreed.

⛔ **A row is never deleted.** A disagreement between an inherited claim and a
measurement is the most valuable output this project has, and deleting the
inherited half destroys the evidence that there was one.
