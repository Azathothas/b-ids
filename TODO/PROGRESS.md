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
session started 2026-08-31T00:30:00Z
baseline        no code exists. The gate covers documents and scripts only.
entries         total 81  open 74  blocked 0  done 7
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## What this session did

**2026-08-31. The research was reconciled against the repository it came from,
and the two failing gate checks were fixed.**

This session wrote no project code, by instruction.

### ⛔ What it found first

The tree described itself as coherent and its gate as green. Both were false,
and the reason was one omission.

⛔ **The founding brief had been transcribed without fetching
`Azathothas/bit-cli`, the repository every measurement in it was taken in.**
**Every** inherited value carried a provenance tag naming an unidentified
document, so a later session could check none of them. Five claims were wrong
or imprecise. And the tree called one measurement unavailable in the entry that
would take it, in the two entries blocked on it, and in both halves of the
sweep, when it was committed in a file one command away.

⛔ **And the gate had never passed.** `check-docs` reported eleven broken links
and `check-twins` reported twelve drifts, because **seven files** described a
licence filler, its PowerShell twin and a directory of licence texts, and none
of it was on disk.

### What it produced

- ⭐ **`Azathothas/bit-cli` fetched, at a named commit, into
  [`../references/`](../references/)** and read in four passes. It is the
  eighteenth tree in the corpus and the only one that is a source rather than
  prior art.
- **[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
  and [`usable.md`](../docs/reference-sweeps/usable.md) rewritten.** Five
  findings changed, and four new sections carry the mechanisms that tree already
  proves: the harness, the driver, the one-file profile, and the version
  discovery that reads a rollout fraction.
- **[`../docs/inherited-claims.md`](../docs/inherited-claims.md) rewritten.**
  Every row now cites a file in that tree rather than a document nobody has, and
  it gained values the brief had dropped: two JA3 hashes, both JA4_r forms, the
  exact builds, the Chrome 152 header values, and a second GREASE draw on the
  other version.
- **The gate fixed and green**, both halves of every pair. ⛔ The absent tool was
  **deleted from the documents that described it** rather than built: a licence
  is written once, and `TOOL-09` carries the ruling.
- **[`../docs/HISTORY/RESUME.md`](../docs/HISTORY/RESUME.md) written**, and a
  document set that named a `docs/public/` page nobody had written was resolved
  by folding what it owed into
  [`../docs/security/secrets.md`](../docs/security/secrets.md) and the publish
  entries, rather than by adding a page.
- ⛔ **The founding brief removed.** It was never committed, and
  [`../docs/inherited-claims.md`](../docs/inherited-claims.md) is what carries
  its measurements now. `DOC-04` has the table of where every part of it went.

### ⚠ Four claims this project inherited have been refuted

⭐ **This is the most important paragraph on this page**, because it sets the
expectation for everything else the founding brief said.

| the claim | what refuted it |
| --- | --- |
| a digest "does not strip GREASE" | the reference implementation defines a table of all sixteen values and filters them, and its own README states the intent |
| the order-preserving raw form "is the only digest that can see what the sorted one hides" about GREASE | the specification says that form is also less GREASE values, so no digest can see it and only the raw bytes can |
| "nobody publishes the corpus" | one impersonating client ships 43 per-exact-build signature files with wire-ordered ciphers and an ordered extension list |
| ⭐ "Chrome 152 sends `cache-control: max-age=0` and Chrome 151 does not" | the capture the claim was taken from. It carries thirteen header fields and no `cache-control`, and the string appears nowhere in that repository |

All four are in [`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) with
the reading that took each away, and none had been acted on.

⚠ **The fourth is the one that produced a rule.** The first three were found by
reading somebody else's project. The fourth was found by reading the project the
claim came from, which nobody had opened. [`RULES.md`](RULES.md) section 3.

### What reading the origin changed

- ⭐ **The priority block is not contested.** One of the three readings is a
  frame-byte read of the HEADERS flag and the five bytes behind it, on two
  Chrome versions; the other two are rendered Akamai strings, and one of them
  comes from a tool that could not write the block. `HARNESS-05` is now a
  confirmation with a predicted answer and a positive control.
- ⭐ **The stream-weight units question is settled with it.** The wire weight is
  one less than the specification's, so a tool passing `256` and a capture
  reading `255` are one quantity.
- **The emitter patch already exists**, as a diff in the corpus, so `EMIT-03`
  starts from a patch rather than from a seam. ⚠ It is MIT and this tree's
  output is 0BSD: read it, do not copy it.
- ⛔ **The reason this project exists is a ruling somebody else had to make.**
  The origin tried to move its profile from Chrome 151 to Chrome 152, found two
  extensions its stack could not name, and kept the older profile deliberately
  rather than ship a `ClientHello` that exists nowhere. That is absolute 2,
  reached by paying for it.

### What the sweep found that the brief did not have

- ⭐ **A whole design problem is already solved by somebody**: one library holds
  an ordered list of codepoint-and-body pairs and refuses an unknown codepoint
  rather than dropping it, which is exactly the escape hatch the brief said had
  to be designed for.
- ⭐ **A quantified emitter hole nobody had stated**: one library's extension
  order is a hash of a sixteen-bit seed, so at most 65,536 orders are reachable.
- **Three shipped violations of the validator's first check**, located at file
  and line, which makes `VALID-02` a publishable result before any capture.
- **A licence split** between one digest and its extended family, which decides
  what this project may emit at all.
- **A fingerprint surface the brief does not mention**: the multipart boundary.

### ⚠ Five defects in this repository's own tooling, found by running it

- **A licence filler and its licence texts were absent**, with seven files in
  the tree describing them: the router, the script catalogue, a standing rule,
  the history page, and three of the checks. ⭐ `check-twins` is what said so, because it compared
  that pair on its **output** rather than on a status line, so an absent tool
  failed loudly rather than passing vacuously in both halves. `TOOL-09` is closed
  on the removal, not on a restoration.
- ⚠ **The reference fetcher stops before cloning when one of its two routes is
  down**, which is `TOOL-04`. The route was up this session and the entry
  carries a measurement taken in both states.
- ⛔ **The banned-vocabulary rule was documented in four files and enforced in
  none.** Found by planting the defect it exists to catch, which is review lens
  2. It is implemented in both halves now, over fourteen of its eighteen words:
  ⭐ the other four match nineteen times in this tree and every one is
  legitimate, so they stay a reading. `TOOL-11`.

⭐ **And the same pass found the hole this whole session climbed out of.** A
markdown link is checked and a path written in a code span is not, so seven
files naming a tool that did not exist were green throughout. `TOOL-10` is the
check, it is `P1`, and it is the highest-leverage small entry in the tree.

- ⛔ **92 files of the corpus were on disk and in no commit**, because a mined
  tree brings its own ignore rules and git honours them. One of them is the
  Chrome 152 capture that every inherited value is cited against. ⭐ The first
  push is what found it: every local check resolves a path against the
  filesystem, so an untracked file and a tracked one look the same until
  somebody else clones. `TOOL-12`, and it closes with a guard in both halves
  and a report in the fetcher.
- ⚠ **The Windows runner skipped a lint** and the workflow's own skip budget
  caught it. It is installed rather than allowed. `TOOL-13`.

## What is in progress

Nothing. The tree is coherent and the gate passes, all fifteen checks, both
halves of every pair.

---

## ⭐ The work order

⚠ **Take these in order and the reason is written down.**
[`INDEX.md`](INDEX.md)'s closing section carries the argument; this is the
sequence.

1. ⭐ **`TOOL-01`**, the workspace and the measured minimum version. First and
   smallest. ⛔ Every acceptance command in this tree names a target that does
   not exist, so nothing below can be proved until this lands.
2. **`SCHEMA-01`, `SCHEMA-05`, then `SCHEMA-02` and `SCHEMA-03`.** The identity
   and the provenance map before the two halves, because the provenance map is
   the one field that cannot be retrofitted: a profile captured before it exists
   can never acquire one.
3. ⭐ **`VALID-01` and `VALID-02`.** Pure logic over the model, no network, no
   browser, and `VALID-02` produces a publishable result on day one because the
   sweep already located three violations at file and line.
4. ⭐ **`HARNESS-05`**, which is one capture. It confirms the priority block
   here, settles `SCHEMA-03`'s priority field and decides whether `EMIT-03` is
   work at all. ⚠ It needs `HARNESS-01` and `HARNESS-02` in their smallest form
   first, which is why it is fourth rather than first.
5. **`HARNESS-01`, `HARNESS-02`, then `HARNESS-06` through `HARNESS-08`.** The
   oracle and the three traps that decide whether a capture means anything:
   parse permissively, keep the cold connection, and take more than one sample.
6. **`TOOL-03`**, before the first raw capture is committed. ⚠ The secret sweep
   will refuse a raw hello, and the tempting fix removes a security rule.
7. **`SCHEMA-06`** in the same change as the first capture, never after it.
   `HARNESS-01` says what retrofitting completeness costs, and the cost is paid
   in captures nobody can take again.
8. **`DRIVER-01` and `DRIVER-02`**, then one profile end to end: captured,
   validated, published in every format.
9. **`PUB-01`, `PUB-02`, `PUB-03` and `PUB-07`**, before the corpus has more
   than one profile in it. Publishing is a contract, and contracts are cheap to
   establish and expensive to change.
10. **`CI-01` through `CI-04`.** From here the corpus maintains itself and
    everything after is additive.
11. **`CORPUS-02`**, the matrix, which is what turns this from a curiosity about
    one browser into the thing anybody uses, and what makes `CI-04`'s automated
    merging satisfiable at all.
12. ⚠ **`LIB-02` earlier than its priority suggests.** It is the only entry that
    proves the corpus is usable rather than merely accurate, and the honest
    expectation is that it will not match on the first attempt.

---

## Open questions for the operator

⛔ **None of these blocks anything.** Each carries a recommendation, so agreeing
costs nothing and a session that does not get an answer proceeds on the
recommendation and records that it did. [`RULES.md`](RULES.md) section 11.

### 1. ⭐ What does a client with no root store put in the trust-anchors extension?

⛔ **This is the question that stopped the origin repository's version bump**, so
it is a real fork rather than a hypothetical one. A Chrome 152 hello carries a
206-byte snapshot of the browser's own root store. A client that has none has
three options: omit it, carry a captured list that ages into a fingerprint of
its capture date, or send it empty, which is a shape no browser sends.

**Recommendation: publish, and do not choose.** This project's job is
`CORPUS-04`: per-build trust-anchor lists with their capture dates and the three
options written down with their costs. Choosing for a client is an emitter's
decision and this project ships no client.

### 2. Does the first capture come before or after the schema is finished?

The order above puts three schema entries first, on the reasoning that a capture
taken against no schema has to be taken again.

**Recommendation: as written, with one exception.** Take `HARNESS-05`'s
measurement as soon as a listener can accept a connection, even in a form too
crude to publish, because it is one number that unblocks two entries and it does
not need a schema to be true.

### 3. Should `LIB-02`'s tool be allowed to grow?

The entry forbids it: a method, a URL, headers, a body, and nothing else.

**Recommendation: hold the line.** The moment it grows a cookie jar it is a
second product this project has not agreed to maintain, and the scope boundary
in [`../README.md`](../README.md) says the project ships no fetching product.

### 4. Is there a second operator, and does the push policy need to change?

The current policy is commit and push to this repository's own remote only.

**Recommendation: leave it.** Nothing here needs a wider policy until `PUB-01`
cuts a release, and that is when the signing key question in `PUB-09` arrives
too.

---

## Settled, and not to be raised again

Kept short on purpose. The list lives in [`RULES.md`](RULES.md); the entries
that keep getting re-opened would go here, and none has yet.
