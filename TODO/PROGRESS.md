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
session started 2026-08-31T14:10:15Z
baseline        the gate passes, all 19 checks, both halves of every pair.
                92 tests across 4 crates. No capture has been taken.
entries         total 82  open 59  blocked 0  done 22
```

⚠ **One of the 59 open is `partial` rather than untouched**: `HARNESS-02`, with
seven of its nine switches done and the other two blocked on `HARNESS-03`.

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## What this session did

**2026-08-31. The project stopped being documents and started being code.**
Every P0 is closed except one, which is `partial` with its blocker named.

### What exists now that did not

- ⭐ **A Rust workspace of eight crates**, with the toolchain pinned to an exact
  version and the minimum supported version held by a check rather than by a
  number somebody typed.
- ⭐ **The profile schema**, published as
  [`../crates/b-ids-schema/schema/browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json)
  and written FIRST, with the Rust types checked against it. Three measured
  halves, `digests` and `raw` as siblings, per-field provenance with four kinds
  and no more.
- ⭐ **A validator**: eight coherence checks, a command, and three outcomes
  rather than two, because a check that cannot run has not passed.
- ⭐ **A capture oracle**: a listener that reads a `ClientHello` off a real
  loopback socket, parses it permissively, keeps the bytes whatever happens, and
  compares a run against a committed golden.
- **Four more gate checks**, taking it from 15 to 19: `check-msrv`, `cargo fmt`,
  `cargo clippy`, `cargo test`. **Two more twin pairs**, taking it from 13 to 15.

### ⛔ What is still true, and matters more than the list above

**No capture has been taken.** Every value in the tree is a fixture or an
inherited claim, and neither is a measurement. The fixture says so in its own
header; the inherited values live in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) with their sources.
⛔ Nothing from either may be published as data.

### ⚠ Five defects found by running the tree, not by reading it

| what | how it was found |
| --- | --- |
| ⛔ **`check-changelog` asserted four rules over zero entries.** It reads an entry as a `### ` heading; this repository wrote them at `## `, which it reads as a section. Green in the gate from the first commit. | editing `CHANGELOG.md` and noticing the count said 0 |
| ⛔ **`check-msrv --verify` reported a broken toolchain as "the workspace does not compile".** It probed `cargo` where the compile needs `rustc`. A broken host was accusing the tree. | an interrupted install left a toolchain with a working cargo and no rustc |
| ⛔ **A refusal test HUNG instead of failing.** With the `--bind` guard removed, the command bound successfully and blocked on the accept. | mutating the guard the test exists to prove |
| ⛔ **The credential rule had a third door and it was open.** Capture-time filtering was gated twice and tested twice; DESERIALISATION was neither, so a profile read from a file could carry a cookie header. | the door sweep, at the end of the session |
| ⚠ **A sweep citation became ambiguous because THIS tree acquired the name.** A path beginning crates/bit-cli-core meant a path inside a reference tree until `TOOL-01` created a `crates/` directory here. | the cited-path check, on its first run |

⭐ **The last two are the ones worth carrying forward.** Neither is a rename and
neither is a typo: one is a rule enforced at one of three doors, and one is a
citation that rotted because the tree moved underneath it.

### ⚠ Two guards were too wide as specified, and running them said so

- **`TOOL-10`'s cited-path check, implemented exactly as the entry described,
  reported 30 spans and every one was legitimate.** The sweep documents cite
  paths inside the reference trees as shorthand. One more rule, read from git
  rather than written down, took it to 6, and those six were read one at a time.
- **`TOOL-03`'s exclusion had to be narrower than "a raw capture directory".**
  It is by field name and file type, and it is mutation-proved against a
  credential planted inside a raw capture under a different field name.

⛔ **Neither hex rule was widened**, which was the tempting fix in both cases and
would have removed the rule.

### ⚠ One acceptance was contradicted by an earlier one

`SCHEMA-03`'s test asserted that no field is named `connection_window`.
`SCHEMA-09` requires exactly that field, beside the increment, with a check
asserting the arithmetic. They are not in conflict once the rule is stated
precisely: no field is named for the window INSTEAD of the increment. ⭐ The
finding is that the earlier test banned a string as a proxy for a rule, and a
string ban cannot tell "instead of" from "beside".

## What is in progress

**`HARNESS-02` is `partial`.** Seven of its nine switches are implemented,
exercised and mutation-proved. `--ca-out` and `--until-h2` both need the
handshake terminated, which is `HARNESS-03`. ⛔ They are absent rather than
present-and-inert: the command refuses each by name and says which entry
implements it.

---

## ⭐ The work order

⚠ **Take these in order and the reason is written down.** The schema, the
validator and the oracle exist now, so the order below is what turns them into
a corpus.

1. ⭐ **`HARNESS-03` and `HARNESS-04`.** Terminate the handshake behind
   `--ca-out`, read SETTINGS, the WINDOW_UPDATE and the PRIORITY block, and
   decode HPACK Huffman so header order is readable. ⛔ This is the single
   biggest unlock in the tree: it closes `HARNESS-02`, and `HARNESS-05` cannot
   be taken without it.
2. ⭐ **`HARNESS-05`**, which is one capture and settles the priority block
   here. It has a predicted answer and a positive control, and it decides
   whether `EMIT-03` is work at all.
3. **`HARNESS-06`, `HARNESS-07`, `HARNESS-08`.** The three traps that decide
   whether a capture means anything. ⚠ `HARNESS-06` needs an emitter type
   distinct from the parser type, so it reaches into `b-ids-emit`.
4. **`SCHEMA-06`**, in the same change as the first capture and never after it.
   Retrofitting completeness is paid for in captures nobody can take again.
5. **`DRIVER-01` and `DRIVER-02`**, then one profile end to end: captured,
   validated, published.
6. ⭐ **`VALID-02`**, which needs no capture at all. `shared_handshakes` is
   written and the three violations are located at file and line. It is the
   project's first publishable result.
7. **`PUB-01`, `PUB-02`, `PUB-03`, `PUB-07`**, before the corpus has more than
   one profile in it.
8. **`CI-01` through `CI-04`**, after which the corpus maintains itself.
9. **`CORPUS-02`**, the matrix.
10. ⚠ **`LIB-02` earlier than its priority suggests.** It is the only entry that
    proves the corpus is usable rather than merely accurate.

⚠ **Two small entries are worth taking whenever a larger one is blocked**:
`TOOL-06` (the route check, three lines, and it cannot run until `PUB-03`
generates a tree) and `SCHEMA-11` (the multipart boundary).

---

## Open questions for the operator

⛔ **None of these blocks anything.** Each carries a recommendation, so agreeing
costs nothing and a session that does not get an answer proceeds on the
recommendation and records that it did. [`RULES.md`](RULES.md) section 11.

### 1. ⭐ Is the privacy default right, given that it blinds the validator?

**This is the one that surfaced from building both halves.** `SCHEMA-04` makes
the default capture record header NAMES only. Four of the validator's eight
checks read a header VALUE, so over an ordinary capture they report
`NotCheckable` rather than passing or failing.

**Recommendation: keep the default and take the version coherence capture with
`--header-values` deliberately.** The switch exists, it drops credentials even
when on, and the alternative is a default that will one day publish one. ⚠ What
should NOT happen is anybody weakening the default to make the validator look
greener.

### 2. Should `cookie` and `authorization` be dropped entirely, or kept as names?

`SCHEMA-04`'s acceptance says a capture contains neither, so that is what
landed. ⚠ Dropping the name also drops the fact that the header was PRESENT,
which is a fingerprint signal in its own right.

**Recommendation: a new entry that records presence without the value.** It is a
schema change and a ruling rather than an implementation detail, and changing an
approved acceptance while implementing it was not this session's to do.

### 3. What does a client with no root store put in the trust-anchors extension?

⛔ **Unchanged from last session, and it is the question that stopped the origin
repository's version bump.** A Chrome 152 hello carries a 206-byte snapshot of
the browser's own root store.

**Recommendation: publish, and do not choose.** `CORPUS-04` is per-build
trust-anchor lists with their capture dates and the three options written down
with their costs.

### 4. Does the first capture come before or after the schema is finished?

⭐ **Answered by this session: the schema is finished.** `SCHEMA-01` through
`SCHEMA-05`, `SCHEMA-07` and `SCHEMA-09` are closed, and `SCHEMA-06` is the one
that must land in the same change as the first capture rather than before it.

### 5. Is there a second operator, and does the push policy need to change?

**Recommendation: leave it.** Nothing here needs a wider policy until `PUB-01`
cuts a release.

---

## Settled, and not to be raised again

Kept short on purpose. The list lives in [`RULES.md`](RULES.md); these are the
rulings this session made that a later one should not re-open.

- **The declared minimum Rust version is a verified upper bound, not the
  floor.** The workspace compiles on 1.88.0 and nothing has shown it fails
  below. `TOOL-01` names the three routes to the true minimum and why none is
  taken yet.
- **`Cargo.lock` is committed.** A measurement taken with an unrecorded
  dependency set cannot be retaken.
- **A path in a code span asserts that it resolves.** A path this tree
  deliberately does not have is written as plain text instead.
