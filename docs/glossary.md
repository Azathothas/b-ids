# glossary.md

Terms this project uses without stopping to define them. One entry per term,
and where a term has a measured caveat the caveat is here rather than in the
page that uses the word.

⚠ **A definition here is not a measurement.** Where an entry states a value or a
behaviour that this project has not observed itself,
[`inherited-claims.md`](inherited-claims.md) carries it with its source.

---

## The digests

| term | meaning |
| --- | --- |
| **JA3** | An MD5 over `ClientHello` fields joined in **wire order**. ⛔ It **strips GREASE**: the reference implementation filters all sixteen reserved values before hashing. It is still unstable per connection for any browser that shuffles its extension order, which is what makes it unfit to assert on. |
| **JA3N** | JA3 with the extension list **sorted** before hashing. Stable under a shuffle, and it exists because plain JA3 is not. |
| **JA4** | The modern replacement. Sorts ciphers and extensions and strips GREASE, so it is stable across a browser's per-connection shuffle. ⭐ This is the digest to assert on. |
| **JA4_r** | JA4's raw form: the same sorted lists, unhashed, so a difference between two captures is readable rather than opaque. |
| **JA4_ro** | The **order-preserving** raw form. ⚠ It also strips GREASE, so it shows what JA4 hides about order and nothing about GREASE. |
| **JA4+** | Everything beyond JA4 itself: JA4S, JA4L, JA4LS, JA4H, JA4X, JA4SSH, JA4T, JA4TS, JA4TScan, JA4Scan, JA4D, JA4D6. ⛔ A different licence from JA4, and patent pending. [`reference-sweeps/findings.md`](reference-sweeps/findings.md) finding 5 is what that means here. |
| **Akamai fingerprint** | Four HTTP/2 facts joined by `\|`: the SETTINGS list, the connection-level WINDOW_UPDATE increment, the stream PRIORITY block, and the pseudo-header order. Rendered as text and optionally hashed. |

⭐ **None of them is a key.** A digest is derived from a profile; a profile is
never derived from a digest, and nothing round-trips through one.

---

## The wire

| term | meaning |
| --- | --- |
| **`ClientHello`** | The first handshake message a TLS client sends. Everything in the TLS half of a profile is read out of it. |
| **GREASE** | RFC 8701. Sixteen reserved values, `0x0a0a` through `0xfafa`, both bytes equal with a low nibble of `a`, sprinkled into lists so servers stay tolerant of values they do not know. Drawn per connection. ⛔ A codepoint on one carries an arbitrary body, so any field holding one must too. |
| **extension order shuffle** | A browser reordering its `ClientHello` extensions per connection, so that no server can come to depend on a fixed order. ⚠ A client whose order never changes is **more** distinguishable, not less: a fixed sequence is itself a signal. |
| **ALPS** | Application-Layer Protocol Settings. It moved codepoint: `0x4469` on older builds, `0x44cd` on newer ones. Which one appears dates the build. |
| **ECH** | Encrypted Client Hello, codepoint `0xfe0d`. It has a GREASE mode, in which a client sends a well-formed but meaningless extension so that real use of it is not distinguishable. ⚠ ECH GREASE and RFC 8701 GREASE are different mechanisms that share a word. |
| **HPACK** | HTTP/2's header compression. Its Huffman coding has to be decoded before header order is readable, which is why a capture harness needs a decoder rather than only a socket. |
| **PSK, resumption** | A resumed TLS session offers `pre_shared_key` where a fresh one offers `session_ticket`. The two produce **different** digests, so a resumed handshake is not a substitute for a cold one. |
| **PRIORITY block** | Five bytes optionally carried inside a HEADERS frame, flagged by `0x20`: an exclusive bit, a 31-bit stream dependency, and a weight. ⚠ Distinct from the standalone PRIORITY **frame**, which RFC 9113 deprecates. A rendered Akamai fingerprint cannot tell an absent block from an unread one, so a profile records the parsed bytes. |
| **stream weight** | The priority weight. ⚠ HTTP/2 encodes it as `weight - 1`, so a tool that takes `256` puts `255` on the wire. |
| **connection window versus increment** | The window is the size a client wants; the increment is what its WINDOW_UPDATE carries, which is the window minus the protocol's own 65,535 default. ⛔ Two numbers, one quantity. Record the increment and name the field for it. |

---

## The corpus

| term | meaning |
| --- | --- |
| **profile** | One browser, at one exact build, on one platform, in one channel, captured at one instant. Not "Chrome 152". |
| **provenance** | Per **field**, not per profile: was this value read off a socket, taken from another platform's capture, copied from somebody's table, or measured and deliberately not shipped. |
| **`wire`** | Provenance: read off a socket by this project's own harness. |
| **`substituted`** | Provenance: taken from a capture of the same build on another platform, with the reason recorded. |
| **`vendor`** | Provenance: copied from somebody else's table and unverified here. ⛔ A profile with any `vendor` field is a draft. |
| **`unreproducible`** | Provenance: measured, and deliberately not shipped, with the reason recorded. |
| **capture harness, oracle** | The listener a browser is pointed at. ⭐ It reads the handshake from outside the client, because a client's own account of what it sent is the account it intended. |
| **variant** | Which kind of request a header set came from. A top-level navigation, a subresource fetch and a reload are three different header sets from one browser. Construction rather than observation here: no capture of two kinds at one version exists. |
| **supersedes** | A published profile is immutable. A correction is a new profile naming the one it replaces and the reason. |
| **channel** | stable, beta, dev, nightly, canary, ESR. ⛔ `latest` means stable and nothing else. |
| **CfT** | Chrome for Testing: a per-channel index of automation builds with download URLs. Addressable by channel, and **unbranded**, so its brand list has no vendor entry. |
| **branded** | Whether a build carries its vendor's own entry in `sec-ch-ua`. An unbranded build cannot produce a branded profile, and the profile says which it is. |
| **rollout fraction** | The share of a channel's users a published version has actually reached. ⚠ The highest version a version-history endpoint knows is not the version people run. |

---

## The project's own words

| term | meaning |
| --- | --- |
| **emitter** | Code that turns a profile back into bytes on a wire, for some stack. |
| **support matrix** | Which stack can emit which profile, ⭐ with the holes left in. A hole is the most useful cell: it tells a client author what they cannot claim. |
| **negative control** | A capture of automation as it actually looks. A corpus that only says what a real browser sends cannot tell anybody the difference. |
| **the record** | [`../TODO/PROGRESS.md`](../TODO/PROGRESS.md). The one file a session reads first, and the only one carrying a work order. |
| **the gate** | The three parts a unit of work passes. [`methodology/gate.md`](methodology/gate.md). |
| **MSRV** | Minimum supported Rust version. Measured from what the dependency graph requires, never chosen. |
| **the founding brief** | The design document this repository was created from, written in `Azathothas/bit-cli` and retired once its content was in this tree. ⛔ Provenance, not a file: it is not tracked and nothing here depends on it. [`inherited-claims.md`](inherited-claims.md) is where its measurements live and where the term is defined. |
| **the origin repository** | `Azathothas/bit-cli`, kept at a named commit in [`../references/`](../references/). Every value this project inherited was measured there, and none of it was measured here. |
