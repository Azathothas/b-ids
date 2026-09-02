# trust-anchors.md

One extension in a modern Chrome `ClientHello` carries a snapshot of the
browser's own root store. A client that copies one build's list is advertising
which build it copied.

⚠ **This page states a trade with three answers and asserts no preference.**
[`../TODO/corpus.md`](../TODO/corpus.md), `CORPUS-04`, says so in as many words,
and a page that recommended one option would be answering a question that
depends on what the caller is doing.

---

## What is measured, and what is not

| | |
| --- | --- |
| ⭐ **the codepoint** | `0xca34`. Measured here on 2026-09-02, in Chrome `152.0.7977.75` on `linux64`, captured on a hosted runner. |
| ⭐ **the length** | 206 bytes in that build, which is what [`inherited-claims.md`](inherited-claims.md) section 3 also records from another capture. |
| ⭐ **the body's shape** | a two-byte big-endian length, then that many bytes of one-byte-length-prefixed identifiers. |
| ⛔ **the NAME is inferred and stays inferred** | `draft-ietf-tls-trust-anchor-ids` is the draft the name comes from, and no specification has been read against these bytes in this project. [`inherited-claims.md`](inherited-claims.md) section 3 carries that split and this page does not resolve it. |

⛔ **The identifier count is not a constant, and that is the point of publishing
per build.** The build measured here carries **32** identifiers of 4, 5 and 8
bytes. The inherited claim records **24**, from a different build's capture.
⚠ Both can be right: a root store changes, and this extension is a snapshot of
one.

---

## Where the lists are

⭐ **Beside the corpus rather than inside a profile**, because the list changes
on a different schedule from everything else a profile carries.

```bash
cargo run -q -p b-ids-corpus -- anchors --root . --out dist/anchors
```

One file per build that carries the extension, named
`BROWSER-VERSION-PLATFORM.json`, holding the profile it came from, the capture
instant, the declared extension length and every identifier **in the browser's
own order**. ⛔ The order is part of what was measured; sorting it would publish
a list no browser sent.

⚠ **Most profiles do not carry the extension at all.** Measured 2026-09-02: one
of the five profiles in this corpus carries it, and every Chrome `151` captured
here does not. That is a fact about those builds rather than a gap here.

---

## ⭐ The three options, and what each one costs

⛔ **None is obviously right.** What follows is the cost of each, stated so a
caller can choose.

### 1. Omit the extension

**What it costs.** The `ClientHello` is one extension short of the build it
claims to be, and the extension is one a modern Chrome sends. Anything counting
extensions or hashing the ordered list sees a different value.

**When it is the right answer.** When the consumer is not trying to be
indistinguishable, or when the peer does not read the extension.

### 2. Carry a captured list

**What it costs.** ⚠ It is honest on the day it was captured and a fingerprint
of that day afterwards. A client shipping a list from a build three months old
is advertising a root store nobody currently has, which is a narrower signal
than sending nothing.

⛔ **And there is a second decision inside this one: whether to copy the
ORDER.** Measured 2026-09-02 across the two Chrome `152` profiles here: the SET
of 32 identifiers is identical on `linux64` and `win64`, and **all 32 positions
differ**. The two bodies are not the same bytes.

⚠ **This project cannot yet say whether that order is per platform or per
connection**, and one capture per platform distinguishes the two not at all.
⭐ What would settle it is two connections of ONE navigation on ONE platform,
compared. Until then, a client copying a fixed order is copying something that
may be a constant it invented.

⭐ **This is the option the published lists serve**, and the capture date is
published with each one so a consumer can decide how stale is too stale.

**When it is the right answer.** When the list can be refreshed on the same
cadence as the build being impersonated.

### 3. Send it empty

**What it costs.** ⛔ A shape no browser sends. An empty list is not what any
measured build carries, so it is more distinguishing than either of the other
two, not less.

**When it is the right answer.** ⚠ This project has not measured a case where it
is, and says so rather than leaving the option out.

---

## What would settle the inferred name

⭐ **Read `draft-ietf-tls-trust-anchor-ids` against these bytes**, and record
whether the identifier encoding it specifies is the one measured above. That is
one reading and it removes an inherited claim from this tree.

⛔ **Until then the name stays inferred**, and
[`inherited-claims.md`](inherited-claims.md) section 3 is where its status
lives. A page that used the name as though it were measured would be publishing
an inference as a measurement, which is the one thing this project must not do.
