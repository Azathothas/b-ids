# PROGRESS.md

⭐ **The one file every session reads first.** Where the work is, what is next,
and why. [`INDEX.md`](INDEX.md) is the list of entries and the **order lives
here and nowhere else**. [`RULES.md`](RULES.md) is the half of the record that
does not change between sessions, and [`SUMMARY.md`](SUMMARY.md) is the last
session's table, which is a snapshot rather than an authority.

⛔ Rewritten every session. It carries no history: the history is the git log and
the entries themselves. Do not add a "previous sessions" section.

⛔ Edited in the same change as the work, never as a report afterwards.

---

## State

```text
session started 2026-09-01T00:50:02Z
baseline        the gate passes, all 19 checks, both halves of every pair.
                166 tests in 16 files across 3 crates. No capture has been taken.
entries         total 82  open 53  blocked 0  done 28
```

⚠ **One of the 53 open is `partial` rather than untouched**: `HARNESS-02`, with
eight of its nine switches done and `--ca-out` blocked on a TLS server this tree
does not have.

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## What this session did

**2026-09-01. The harness learned to read the half of the fingerprint that sits
above TLS.** Six entries closed, and the biggest unlock in the tree turned out
to be cheaper than it looked.

### ⭐ HTTP/2 was reached WITHOUT terminating TLS, and that is the session's finding

`HARNESS-03` sat behind `--ca-out` because reaching HTTP/2 from a browser needs
a terminated handshake. ⭐ **A client with prior knowledge needs no handshake at
all**, and its frames carry the same fingerprint. The entry has the mechanism
and the measurement: [`harness.md`](harness.md), `HARNESS-03`.

So the cleartext surface now reads whichever protocol the peer actually spoke,
⛔ **decided by the bytes rather than by a flag the operator passed.**

⚠ **What it does NOT reach is a browser.** No browser speaks cleartext HTTP/2,
so the browser half still needs termination. `HARNESS-05` says so in its own
entry with what was tried and what would open it.

### What exists now that did not

- ⭐ **An HTTP/2 frame reader**: the preface, SETTINGS in arrival order, the
  connection WINDOW_UPDATE increment, standalone PRIORITY frames, and the
  HEADERS frame with its flags byte and its priority block read as BYTES.
- ⭐ **An HPACK decoder**, checked against a fetched corpus of **47,142 cases
  across 446 files** rather than against itself. Header order is readable.
- ⭐ **The emitter's first real content**, in `b-ids-emit`: a type that cannot
  represent a declared length disagreeing with its body, which is exactly what
  the parser's type must represent.
- **The connection selection rule** and **the sampling rule**, each with its
  own module and its own committed fixture.
- ⭐ **A raw block a profile can be rebuilt from**, asserted rather than
  intended.
- `--until-h2` moved from refused to implemented. `--run-timeout-ms` is new.

### ⛔ What is still true, and matters more than the list above

**No capture has been taken.** Every value in the tree is a fixture or an
inherited claim, and neither is a measurement. ⛔ Nothing from either may be
published as data.

### ⭐ The door sweep found the credential rule's FOURTH door, and it was open

⛔ **A capture drops `cookie` from its parsed fields and keeps it in the bytes
beside them.** The existing test greps the output for the credential's
plaintext, which is absent; the same credential is present hex-encoded in
`raw_hex`, and `SCHEMA-06` had just routed those bytes into a published
profile's raw block.

Measured by driving the compiled command over loopback with a fixture
credential in the request: the plaintext is absent from the capture and the
same sixteen bytes, hex-encoded, are present in `raw_hex`.

⚠ **The measurement is described rather than pasted**, because the paste is a
long hex run and `check-no-secrets` refuses those in a tracked document. ⭐ The
guard refused this very paragraph on its first draft, which is the check
working rather than a nuisance.

⭐ **Two of this project's own rules collide on the cleartext surface**, and
neither is wrong: `SCHEMA-04` says a capture carries no credential, and
`SCHEMA-07` says the raw bytes are never edited because a capture is a moment
that cannot be retaken.

**What landed is the loud failure, not a resolution.** `Raw::check` refuses a
profile whose cleartext bytes spell out a credential header line, on both
spellings, and `Profile::check` calls it. ⛔ The bytes are never edited: the
profile is refused and the operator decides. The fork itself is an open
question below.

### ⚠ Two mutations reported NOTHING, and both produced better findings than the ones that failed

| the mutation | what reporting nothing revealed |
| --- | --- |
| ⛔ one wrong row in the HPACK Huffman table, and all 47,142 cases still passed | the canonical construction derived every code from the bit-length counts, so the transcribed code column was **decoration**. It is read from the table now, and `check_table_is_canonical` states the assumption the decoder rests on. |
| ⛔ the rebuild's comparison removed, and all eight `raw_backstop` tests still passed | every test exercised the ABSENT branch. The comparison had never been seen to report a difference. A test now plants one. |

⚠ A third mutation did not apply at all because of a shell quoting difference,
so the green result it produced was the unmutated tree. Applied properly it
found a missing test rather than proving one, and `HARNESS-03`'s closing
carries the reason.

### ⚠ A documented flag did not exist, and its own instruction named it

`check-no-secrets`'s reference-corpus exemption says to re-run with
`--scope references` when a tree is added. ⛔ **That flag did not exist.** It
does now, on both halves, with a `check-twins` row and a measured cost of 70s
for the sh half over nineteen trees. The exemption was then re-read over the new
tree and every one of its 52,396 hits was categorised.

### One false doc claim, fixed

`Http2Half::akamai_text`'s comment said an absent priority block and a block of
zeroes both render as `0`. They do not: a block of zeroes renders `1:0:0:0`. The
comment now states what the rendering actually loses.

## What is in progress

⛔ **Nothing is half-edited.** `HARNESS-02` is `partial` by decision: eight of
nine switches are done and `--ca-out` is absent rather than inert, refusing by
name and saying what is missing.

---

## ⭐ The work order

⚠ **Take these in order and the reason is written down.**

1. ⭐ **The vendored TLS server.** ⛔ It is now the single blocker on the whole
   browser half of the project: `--ca-out`, `HARNESS-05`, `HARNESS-10`,
   `DRIVER-01` and every profile with a real `ClientHello` in it wait on it.
   The operator ruled on 2026-09-01 that it is **vendored here and patched
   here**, following `Azathothas/bit-cli`'s practice: a manifest naming what is
   vendored and at which commit, a record of every local change, a derived
   patch series regenerated from the tree, and a scan that reports when
   upstream has moved. ⚠ **No entry exists for it yet**, so the first step is
   authoring one from [`ENTRY.md`](ENTRY.md), per
   [`../docs/methodology/authoring.md`](../docs/methodology/authoring.md).
2. ⭐ **`VALID-02`**, which needs no capture and no TLS. `shared_handshakes` is
   written, and the three violations were re-verified this session by opening
   the files: five `impit` modules return `chrome_100`'s TLS and HTTP/2 beside
   their own headers, one library serves a cipher table commented with a
   version from years earlier, and its classifier returns three families where
   its data carries four. ⛔ It is the project's first publishable result.
3. **`HARNESS-05`**, the moment the handshake terminates. Its probe half is
   done and its entry says exactly what is left.
4. **`DRIVER-01`, `DRIVER-02`, `DRIVER-03`**, then one profile end to end.
   ⚠ Chrome `151.0.7922.76` and Edge `152.0.4191.53` are on the capture host,
   so the resolver has something real to find.
5. **`HARNESS-09`**, which now has four parsers to fuzz rather than two: the
   record layer, the `ClientHello`, the HTTP/2 frame reader and the HPACK
   decoder. ⚠ `cargo fuzz` needs a nightly toolchain and this tree pins an
   exact stable one; establish the route before planning the entry.
6. **`PUB-01`, `PUB-02`, `PUB-03`, `PUB-07`**, before the corpus has more than
   one profile in it.
7. **`CI-01` through `CI-04`**, after which the corpus maintains itself.
8. **`CORPUS-02`**, the matrix.
9. ⚠ **`LIB-02` earlier than its priority suggests.** It is the only entry that
   proves the corpus is usable rather than merely accurate.

⚠ **Small entries worth taking whenever a larger one is blocked**: `TOOL-04`
(the fetcher stops when one of its two routes is down), `SCHEMA-11` (the
multipart boundary), `VALID-03` (a family the resolver cannot produce), and
`TOOL-06` (three lines, and it cannot run until `PUB-03` generates a tree).

---

## Open questions for the operator

⛔ **None of these blocks anything.** Each carries a recommendation, so agreeing
costs nothing and a session that does not get an answer proceeds on the
recommendation and records that it did. [`RULES.md`](RULES.md) section 11.

### 1. ⭐ NEW. What does a capture do with a credential that is in its raw bytes?

**The door sweep found it this session and the loud failure is the only part
that landed.** A cleartext capture drops `cookie` from its parsed fields and
keeps it, hex-encoded, in the bytes beside them. Two of this project's rules
collide there and neither is wrong.

Four options, and none is free:

| option | what it costs |
| --- | --- |
| ⭐ **refuse the profile, keep the capture** | what landed. The capture on disk still carries the credential; only publishing is stopped. |
| redact the bytes | ⛔ destroys the artefact that survives every parser defect, and breaks the rebuild property `SCHEMA-06` just established |
| never store cleartext connection bytes | loses the widest backstop on the one surface that has no TLS to hide behind |
| capture cleartext only against a subject that sends no credential | a procedure rather than a guard, and procedures are what guards exist to replace |

**Recommendation: keep the refusal, and add an entry for a capture-time
refusal beside it**, so the harness declines to write a raw block it knows a
profile cannot carry. ⚠ What should NOT happen is redaction: the moment this
project edits captured bytes, the raw block stops being evidence.

### 2. Is the privacy default right, given that it blinds the validator?

`SCHEMA-04` makes the default capture record header NAMES only. Four of the
validator's eight checks read a header VALUE, so over an ordinary capture they
report `NotCheckable`.

**Recommendation: keep the default and take the version coherence capture with
`--header-values` deliberately.** ⚠ What should NOT happen is anybody weakening
the default to make the validator look greener.

### 3. Should `cookie` and `authorization` be dropped entirely, or kept as names?

⚠ Dropping the name also drops the fact that the header was PRESENT, which is a
fingerprint signal in its own right.

**Recommendation: a new entry that records presence without the value.**

### 4. What does a client with no root store put in the trust-anchors extension?

⛔ Unchanged. A Chrome 152 hello carries a 206-byte snapshot of the browser's
own root store.

**Recommendation: publish, and do not choose.** `CORPUS-04`.

### 5. ⚠ NEW. The published schema expresses no numeric bounds at all

`u8`, `u16` and `u32` are each a bare `{"type": "integer"}`, so the published
schema accepts 999 for a field the Rust type holds to a byte. Found while
extending the schema for `SCHEMA-06`; not fixed there, because one bounded
field among dozens of unbounded ones is worse than none.

**Recommendation: an entry that bounds all three at once**, with the checker
gaining `minimum` and `maximum` in the same change. ⭐ The checker's guard
already refuses a keyword nothing enforces, so the two cannot land apart.

---

## Settled, and not to be raised again

Kept short on purpose. The list lives in [`RULES.md`](RULES.md); these are the
rulings a later session should not re-open.

- ⭐ **The TLS terminator is vendored here and patched here.** Ruled by the
  operator 2026-09-01. Not a registry dependency, and not written from
  scratch.
- **The declared minimum Rust version is a verified upper bound, not the
  floor.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- ⭐ **The reference corpus keeps whole trees**, and the HPACK vector corpus was
  measured rather than trimmed by eye: 26.9 MiB packed across nineteen trees,
  against a threshold of about 100 MiB.
