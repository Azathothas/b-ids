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
session started 2026-09-01T03:47:48Z
baseline        the gate passes, all 20 checks, both halves of every pair.
                192 tests in 22 files across 4 crates.
entries         total 84  open 49  blocked 0  done 35
```

⚠ **Nothing is `partial` any more.** `HARNESS-02` was, for one day, and its
ninth switch closed this session.

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## What this session did

**2026-09-01. The browser half opened.** Seven entries closed, and the thing
every one of them was waiting on was a TLS server.

### ⭐ Two real browsers completed a handshake, and the harness read their HTTP/2

⛔ **This is the first time anything in this repository has read a byte a
browser put on a wire.** Chrome `151.0.7922.76` produced 7 connections with 6
terminated; Edge `152.0.4191.53` produced 8 with 7. Every terminated one
negotiated `h2` over TLS 1.3, and every one carried a SETTINGS frame, a
connection WINDOW_UPDATE and a HEADERS frame with the priority bit set.

⚠ **The corpus is still empty.** A capture is not a profile, and nothing yet
writes one. `CORPUS-01` is the entry that decides what a profile looks like on
disk, and it is now the top of the work order for that reason.

### ⭐ The priority block is measured, and it agrees with what was inherited

Thirteen HEADERS frames across three driven runs, on two browsers, and every
one of them carries `80000000ff`: exclusive, dependency 0, weight 255 on the
wire. [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 5
now carries the measurement beside the reading it inherited, with the browser,
the build, the date and the conditions, and a new `measured-here` status.

⛔ **It is still not published**, because there is nowhere to publish it.

### What exists now that did not

| | |
| --- | --- |
| ⭐ a vendored TLS terminator | rustls at `v/0.23.43`, 210 files, compiled by this tree, with a manifest, a change record, a derived patch series and a two-legged scan |
| ⭐ `--ca-out` | mints an authority per run, writes the certificate and never its key, prints the public key pin on stderr, and terminates |
| a driver | resolves Chrome and Edge with the source that answered, launches into a profile nobody keeps, and records every switch it passed |
| ⭐ the first publishable result | `b-ids-validator import references --report`: ten exhibits in two public repositories, each with a file, a line and the check it fails |
| a headless normalisation | measured rather than inherited, and it records the substitution rather than hiding it |

### ⛔ The claim audit found a fabricated number in this session's own writing

**The secret-scan reading over the vendored tree was written as 41 hits with 33
long hex runs, from a read of the output rather than a count of it.** Counted:
38 hits, 4 and 4 and 30, of which ten are commit ids in links to other public
repositories.

⚠ **It had reached three places**, including the header of a security check,
which is the file where a number nobody counted is least acceptable. All three
are corrected and the correction says it happened.

### ⚠ Three traps were paid for, and each one is written where it bit

| the trap | what it cost |
| --- | --- |
| ⛔ cargo resolves a path dependency against the OUTERMOST workspace that does not exclude it | the build failed on a key this tree has never had. `vendor` is in `exclude` now with the measurement beside it. |
| ⛔ `chrome.exe --version` on Windows LAUNCHES the browser into the operator's own profile | the resolver hung and a browser opened. That source is skipped by platform now, because a timeout would have fixed the hang and not the side effect. |
| ⚠ a test that asserted a refusal became a HANG when the refusal stopped refusing | ten minutes, and a locked test binary afterwards. Every test that drives the command passes a deadline now. |

### ⚠ A mutation reported nothing, and that was the finding

Removing the sort from the reference importer changed no test result: the walk
underneath is already stable on this host, so equality between two runs cannot
tell a sorted answer from an incidentally stable one. ⭐ **The acceptance asked
for "byte-identical output across runs" and a test written to those words could
never have failed.** A second test asserts sortedness directly, and the same
mutation fails it.

### ⚠ What a vendored tree cost the checks

Five of the checks that read the whole tree failed the moment it landed. Each
now exempts the vendored trees and the patch series derived from them, and
⛔ **none of them exempts the manifest or the patch record beside those**, which
this project wrote. That boundary fired the same day, over this project own patch record, and
[`vendor.md`](vendor.md) has what it caught.

### ⚠ `check-twins` is slower than the measurement in `scripts/README.md`

It was 171s on 2026-08-27. Measured again on 2026-09-01 with the vendored tree
in scope, on the same machine: **1025s**, six times the earlier figure. ⚠ A first
attempt wrapped it in a 590s timeout, and the kill showed up as a DRIFT on
`check-gate` with exit 143, which is SIGTERM rather than a disagreement. ⭐ Run
to completion, every pair agrees and both halves answer
`{"schema":"check-gate/1","total":20,"passed":19,"failed":0,"skipped":1,"strict":0}`
and exit 0. ⛔ A timeout around a comparison turns a slow half into a false
finding, and that is worth knowing before somebody believes one.

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification.

### 1. The door sweep: what other door reaches this code

Swept: every reader of `Capture::termination` and `Termination::plaintext_hex`,
every caller of `read_first_message`, every call site of the credential filter,
every constructor of `Oracle`, every caller of `drive` and `Launch`, and every
path that could leave a throwaway profile behind.

⭐ **Finding, and it is open question 1.** The credential rule now has a third
door. The parsed fields drop `cookie` and `authorization` on all three surfaces,
which the sweep confirmed at four call sites, and the terminated surface keeps
the decrypted first message beside them. ⚠ That is the surface where a real
browser's credentials will actually appear, unlike the cleartext one.

⛔ Nothing can publish it today, because nothing writes a profile, and the sweep
confirmed that by finding no construction of one outside the schema fixtures.

⭐ **What the other passes did not look at:** the callers. Both of the others
read what was written; this one grepped for what was not enumerated.

### 2. The guard mutation: can the new guards actually fail

Swept: nine guards, each planted and each read unpiped. `check-vendor` on three
defects and on both halves, the derived series against an unregenerated tree,
the secret rule narrowed by name, the raw block taken from the wrong bytes, the
bind refusal, the golden schema version, the reference reader going blind, the
reference report losing its order, and the throwaway profile being kept.

⭐ **Finding: one mutation reported nothing.** Removing the sort from the
reference importer changed no test result, because the walk underneath is
already stable on this host. The acceptance asked for "byte-identical output
across runs" and a test written to those words could never have failed. A second
test asserts sortedness directly and the same mutation fails it.

⚠ **Two guards were NOT mutated**, and saying so is the point: the headless
normalisation leaving a windowed capture alone, and the resolver refusing an
executable no source could version. Both would fire on a fixture rather than on
a plant, and neither is reached by a capture path yet.

⭐ **What the other passes did not look at:** whether a green result means
anything. The door sweep reads structure and the claim audit reads prose.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every quoted output in the seven closings, this file,
[`SUMMARY.md`](SUMMARY.md), the changelog, and the two documents the work made
stale.

⛔ **Finding, and it is the most valuable one of the session: a fabricated
number, in this project's own writing.** The secret-scan reading over the
vendored tree was written as 41 hits with 33 long hex runs, from a read of the
output rather than a count of it. Counted with `awk`: 38, as 4 and 4 and 30, of
which ten rather than eight are commit ids in links. It had reached three files
including a security check's own header. All three are corrected.

⭐ **Two other claims were checked and stood.** The priority block was asserted
as `80000000ff` "on every one" after being printed for three connections;
counted across all thirteen HEADERS frames, it holds. The brand list was
asserted identical between headless and headful by eye; compared byte for byte,
it is.

⚠ **One stale document claim, fixed.** `scripts/README.md` said the dependency
graph imposes no floor on the minimum Rust version, "which is this tree's state
today". It stopped being true when the certificate minter arrived: the floor is
now 1.88, which happens to equal the declared value.

⭐ **What the other passes did not look at:** the prose. A guard can be correct
and mutation-proved while the sentence describing it is a number nobody counted.

---
## What is in progress

⛔ **Nothing is half-edited.** Every entry this session touched is closed with
its acceptance command run and its real output pasted.

---

## ⭐ The work order

⚠ **Take these in order and the reason is written down.** Foundations first:
the corpus has a capture path and no corpus, and everything below item 2 is
worth less until that is true.

1. ⭐ **`CORPUS-01`**, content-addressed, append-only, never edited in place.
   ⛔ **It is now the single thing in the way.** The harness takes captures and
   nothing turns one into a profile, so every measurement this session took
   lives in a scratch log rather than in the thing this project exists to
   publish. ⚠ It also owns the credential question in open question 1, because
   a capture becomes publishable exactly there.
2. **`HARNESS-10`**, whether measuring changed what was measured. It is
   takeable now and it was not before: the captures were taken through a
   per-launch key pin rather than a trust store, and that is a condition
   nobody has measured the effect of.
3. **`DRIVER-02`**, the version that is serving rather than the one published.
   The resolver reads what is installed; this reads what is rolling out, and a
   capture of a build almost nobody runs is a correct fingerprint of nothing.
4. **`SCHEMA-08`**, every generated format from one generator, round-tripped.
   ⚠ Before `PUB-*`, because a published route is a contract and a second
   generator is a second answer.
5. **`HARNESS-09`**, fuzz the four parsers. ⚠ `cargo fuzz` needs a nightly
   toolchain and this tree pins an exact stable one; establish the route
   before planning the entry.
6. **`VALID-03`**, a family the resolver cannot produce. The reference
   importer already reports one in somebody else's tree, so the check has a
   worked example to be written against.
7. **`PUB-01`, `PUB-02`, `PUB-03`, `PUB-07`**, once there is one profile.
8. **`CI-01` through `CI-04`**, after which the corpus maintains itself.
9. **`CORPUS-02`**, the matrix, and **`LIB-02`**, which is the only entry that
   proves the corpus is usable rather than merely accurate.

⚠ **Small entries worth taking whenever a larger one is blocked**: `TOOL-04`
(the fetcher stops when one of its two routes is down), `SCHEMA-11` (the
multipart boundary), `CORPUS-05` (name the unidentified extension), `TOOL-06`
(three lines, and it cannot run until `PUB-03` generates a tree).

---

## Open questions for the operator

⛔ **None of these blocks anything.** Each carries a recommendation, so agreeing
costs nothing and a session that does not get an answer proceeds on the
recommendation and records that it did. [`RULES.md`](RULES.md) section 11.

### 1. ⭐ The credential in the raw bytes now has a THIRD door, and it is the real one

**Unresolved, and more urgent than it was.** A terminated capture records the
decrypted first message in `Termination::plaintext_hex`, beside parsed fields
that drop `cookie` and `authorization`. ⚠ **That is the surface where a real
browser's credentials will actually appear**, unlike the cleartext one.

⛔ Nothing can publish it today, because nothing writes a profile. `CORPUS-01`
is exactly where that stops being true.

**Recommendation: unchanged, and now with a deadline.** Keep the refusal,
never redact, and make `CORPUS-01` refuse at the moment a capture becomes a
profile rather than at the moment a profile is published.

### 2. ⚠ NEW. Is a per-launch key pin an acceptable standing capture method?

Every capture this session took was through
`--ignore-certificate-errors-spki-list`, carrying the base64 SHA-256 of the
run's own authority. ⛔ It is not `--ignore-certificate-errors`: verification
still runs and any other key is still refused. But it is not a trusted root
either, and installing one is a change to a machine's security configuration
that belongs to the operator.

**Recommendation: keep the pin as the default capture method, and let
`HARNESS-10` measure the difference against a real trust anchor.** ⚠ What
should NOT happen is a capture taken with verification switched off, which
changes the subject rather than the condition.

### 3. Is the privacy default right, given that it blinds the validator?

`SCHEMA-04` makes the default capture record header NAMES only. Four of the
validator's eight checks read a header VALUE, so over an ordinary capture they
report `NotCheckable`.

**Recommendation: keep the default and take the version coherence capture with
`--header-values` deliberately.** ⚠ What should NOT happen is anybody weakening
the default to make the validator look greener.

### 4. Should `cookie` and `authorization` be dropped entirely, or kept as names?

⚠ Dropping the name also drops the fact that the header was PRESENT, which is a
fingerprint signal in its own right.

**Recommendation: a new entry that records presence without the value.**

### 5. What does a client with no root store put in the trust-anchors extension?

⛔ Unchanged. A Chrome 152 hello carries a 206-byte snapshot of the browser's
own root store.

**Recommendation: publish, and do not choose.** `CORPUS-04`.

### 6. The published schema expresses no numeric bounds at all

`u8`, `u16` and `u32` are each a bare `{"type": "integer"}`, so the published
schema accepts 999 for a field the Rust type holds to a byte.

**Recommendation: an entry that bounds all three at once**, with the checker
gaining `minimum` and `maximum` in the same change. ⭐ The checker's guard
already refuses a keyword nothing enforces, so the two cannot land apart.

### 7. ⚠ NEW. `check-twins` no longer finishes inside ten minutes

**Recommendation: an entry that scopes the slow halves rather than the
comparison.** ⛔ What must not happen is dropping a pair from the comparison to
make it fit, which is how a comparison stops comparing. ⚠ And no wrapper
timeout around it: a killed half reports as a drift.

---

## Settled, and not to be raised again

Kept short on purpose. The list lives in [`RULES.md`](RULES.md); these are the
rulings a later session should not re-open.

- ⭐ **The TLS terminator is vendored here and patched here.** Ruled by the
  operator 2026-09-01, done the same day: rustls at `v/0.23.43` under
  [`../vendor/`](../vendor/), with its manifest, its patch record and its scan.
- **The declared minimum Rust version is a verified upper bound, not the
  floor.** ⚠ The graph now imposes one, 1.88 from the certificate minter, and
  it happens to equal the declared value.
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- ⭐ **The reference corpus keeps whole trees**, and it is exempt from the prose
  checks and the secret scan by directory, never by file.
