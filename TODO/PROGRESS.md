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
session started 2026-09-01T07:58:00Z
baseline        the gate passes: 21 checks and check-twins, both halves of
                every pair. 251 tests in 27 files across 5 crates.
entries         total 84  open 43  blocked 0  done 41
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## What this session did

**2026-09-01. The corpus stopped being empty.** Six entries closed, and the
first of them is the one every other was waiting on.

### ⭐ There is a profile, and it is a measurement

`corpus/v1/chrome/stable/win64/151.0.7922.76.json`, with the `ClientHello` it
was read from at `raw/v1/chrome/stable/win64/151.0.7922.76.hello.hex`. Chrome
`151.0.7922.76`, headful, on this Windows host, captured by this project's own
harness on 2026-09-01T08:26:33Z.

⭐ **Its raw block reproduces both measured halves exactly**, which is asserted
rather than intended: `b-ids-corpus verify` re-parses the stored bytes and
compares. Nothing in the TLS or HTTP/2 halves came from anywhere but the wire.

⭐ **And the first inherited value has left
[`../docs/inherited-claims.md`](../docs/inherited-claims.md).** The priority
block is published, in a profile, beside the frame bytes it was read from.

### ⚠ Canonical on the default branch, published on a branch that does not exist yet

⭐ **These are two different things and both are specified.** `CORPUS-01` puts
the canonical corpus on the default branch as reviewable JSON, which is what
makes an automated capture something a person can read as a diff. `PUB-02` is
the orphan data branch that carries only generated artefacts, and `SCHEMA-08`,
`PUB-03` and `PUB-04` are the formats, the routes and the pasteable snippets.

⛔ **Nothing is published yet**, so what is on the default branch today is the
whole of it. [`../README.md`](../README.md) states the split where a consumer
will look for it.

### What exists now that did not

| | |
| --- | --- |
| ⭐ `b-ids-corpus` | turns the cold connection of a navigation into a profile, writes it once, refuses a route that already holds one, and derives its index from the tree rather than appending to one |
| ⭐ `check-corpus` | asks git, over the whole history, whether a published file was ever modified, deleted or renamed. The working tree cannot answer that. |
| `check-routes` | no published single-value file ends with a newline, and `--assert-latest-is-stable` |
| ⭐ `b-ids-driver versions` | the build that is SERVING, read from the rollout fraction, with what the naive query would have said printed beside it |
| ⭐ `b_ids_harness::modes` | whether the capture surface changed what it measured, with per-connection draws told apart from mode effects |
| `fuzz/` and `hostile` | one million coverage-guided runs, and 6767 mutations that run on every host |
| `experiments/` | two measurements, each taken by a script anybody can re-run |

### ⭐ Three measurements, and every one of them has its conditions

| what | answer |
| --- | --- |
| does terminating the handshake change what the browser offers before it | ⭐ **No.** 17 of 19 TLS fields agree across three rounds, none differ, two carry a per-connection draw and cannot be compared. One browser, one build, one host. |
| is the inherited version-discovery defect real | ⭐ **Yes, to the digit.** Highest known `153.0.8010.12` at fraction `0.005`; the build at full rollout `152.0.7977.65`; the automation index `152.0.7977.64`. The two first-party sources still disagree by one patch component, and beta's two sources agree, which is the control. |
| can any parser be made to panic | ⭐ **Not in a million runs.** No crash, no timeout. libFuzzer discovered the HTTP/2 preface on its own. |

### ⚠ What the version measurement says about this project's own corpus

⛔ **The one profile is a major behind what stable is serving.** Chrome
`151.0.7922.76` is captured; `152.0.7977.65` is at full rollout. Nothing said so
before `DRIVER-02` existed. `DRIVER-05` is acquisition and `CORPUS-02` is the
matrix; between them they close it.

### ⚠ Traps paid for, each written where it bit

| the trap | what it cost |
| --- | --- |
| ⛔ a text writer that translates newlines for the platform | Eight files silently became CRLF in a tree that declares `eol=lf`. ⚠ And `check-gate`'s line-endings filter reads the INDEX column, so a modified-but-unstaged file passes it. `.gitattributes` normalises on commit, so nothing reached the history; the working tree was wrong and the gate could not see it. |
| ⛔ `git ls-files` answers a path outside the repository with an EMPTY LIST | `check-routes` reported "ok, 0 files" over the fixture written to prove it could refuse. A route tree that yields no single-value file is exit 2 now. |
| ⛔ a pinned `rust-toolchain.toml` applies to a fuzz crate under the repository root | A nightly IMAGE is not enough: rustup fetches the pinned stable and `-Z sanitizer` is then refused. `RUSTUP_TOOLCHAIN` has to be set explicitly, and `CI-03` will hit this. |
| ⚠ a single terminating run produced 0 cold connections and 5 resumed | More connections do not buy more cold handshakes; more RUNS do. A cold hello is sampled per launch, not per connection. |
| ⛔ `check-twins` runs the two halves of a pair at DIFFERENT INSTANTS | A tree that changes underneath it reports a drift that is not one. `repo.has_codegraph` came back `sh=false ps=true` because `.codegraph/` was created between the two probes; both halves use the identical rule and both answer `true` now. ⚠ A drift is re-checked by re-running the pair before it is believed, and a run whose tree moved is not evidence. |

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what other door reaches this code

Swept: every caller of `profile_from` and `Store::add`, every construction of a
`Profile` anywhere in the tree, every call site of `HeaderSet::record`, every
writer under `corpus/` and `raw/`, every caller of `version_order` and of
`sha256`, and every reader of `captured.trust`.

⭐ **Confirmed, by grep rather than from memory:** exactly one production
construction of a `Profile`, one write path into the corpus, one `.hex` writer,
one implementation of the version ordering and one of the digest. The credential
filter has one construction path and the store calls `check` before it writes.

⛔ **Finding: `captured.trust` was TYPED, not read.** It is the field
`HARNESS-10`'s future comparison across profiles depends on, and the identity
file it comes from was written by hand, so a profile could have claimed a trust
store while the run used a pin and nothing in the bytes could contradict it.
⭐ Fixed: `experiments/10-first-profile.sh` writes the identity file now, taking
the trust configuration from the switch list the driver actually passed.

⭐ **What the other passes did not look at:** the callers. Both of the others
read what was written; this one grepped for what was not enumerated.

### 2. The guard mutation: can the new guards actually fail

Swept, each planted and each exit code read unpiped: the corpus semantic leg, the
corpus git leg in a throwaway repository, both new secret-scan exclusions in both
halves, `check-routes` in four directions in both halves, the hostile-input
suite, the `latest` pointer, and the frame re-encoder.

⛔ **Finding: two functions added this session had no test at all.** The ISO 8601
formatter that fills `captured.at`, and the frame re-encoder that fills
`raw.http2_frames_hex`. ⭐ Both now have one, and the formatter turned out to be
correct including the 1900 and 2000 leap-century cases, which is the classic
place to be wrong.

⭐ **Finding, from the same mutation, and it is the more useful half:** dropping
the reserved bit from the frame re-encoder made the new direct test fail and the
**corpus suite still pass**, because the fixture's frames do not set that bit.
The corpus rebuild is a real check that is blind to this class on this data,
which is exactly why a direct test was needed rather than an indirect one.

⚠ **Two guards were NOT mutated**, and saying so is the point: the reference
importer's ordering, and the resolver refusing an executable no source could
version. Neither was touched this session.

⭐ **What the other passes did not look at:** whether a green result means
anything.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every pasted block in the six closings, this file,
[`SUMMARY.md`](SUMMARY.md), the changelog, and the documents the work made stale.

⛔ **Three findings, all in this session's own writing.**

- **A number quoted from an assertion's floor rather than measured.** The hostile
  corpus was written up as "over five thousand mutations", which is what the
  assertion refuses below. Counted: **6767**. It had reached two files.
- **A pasted suite count that moved after it was pasted.** `CORPUS-01` closed on
  `22 passed`; `CORPUS-03` later added two tests to the same file. The block is
  left as it was measured with the reason written under it, rather than
  re-pasted as though it had always said 24.
- ⛔ **A fabricated baseline line, in this file.** The state block was written as
  "241 tests in 26 files" before anything was counted. Measured: **251 tests in
  27 files across 5 crates**.

⭐ **Two claims were checked and stood.** "Seventeen of nineteen fields agree,
none differ" was counted out of the comparison output: 17, 2, 0. And the fuzz
corpus figure of 856 files was counted rather than estimated.

⭐ **What the other passes did not look at:** the prose. A guard can be correct
and mutation-proved while the sentence describing it carries a number nobody
counted.

---

## What is in progress

⛔ **Nothing is half-edited.** Every entry this session touched is closed with
its acceptance command run and its real output pasted.

---

## ⭐ The work order

⚠ **Take these in order.** Foundations first: there is one profile and one
platform, so the next thing worth more than anything below it is a second of
each.

1. ⭐ **`CORPUS-02`**, the matrix. ⛔ **It is now the single thing in the way.**
   One profile is not a corpus: `VALID-01`'s handshake check reports
   `NotCheckable` because it needs a second profile of the same build to
   compare against, `HARNESS-10`'s answer rests on one browser, and the one
   profile there is describes a build a major behind stable. Start with the
   cheapest lane that is not this host: Edge is already resolved here and its
   capture path is identical.
2. **`DRIVER-05`**, acquisition. `DRIVER-02` can now say which build should be
   captured and nothing can fetch it, so the corpus can only ever describe what
   somebody happened to install.
3. **`SCHEMA-08`**, every generated format from one generator. ⚠ Read open
   question 1 first: its nine-format list needs six encoder-and-decoder pairs or
   six dependencies, and that is a decision rather than a detail.
4. **`PUB-03`**, then `PUB-01`, `PUB-02`, `PUB-07`. The corpus has routes and
   an index and nothing serves them.
5. **`CI-01` through `CI-04`**, after which the corpus maintains itself.
   ⚠ `CI-03` needs `RUSTUP_TOOLCHAIN` set for the fuzz lane; `fuzz/README.md`
   carries why.
6. **`VALID-03`**, a family the resolver cannot produce. The reference importer
   already reports one in somebody else's tree.
7. **`LIB-01`** and **`LIB-02`**, the only entries that prove the corpus is
   usable rather than merely accurate.

⚠ **Small entries worth taking whenever a larger one is blocked**: `TOOL-04`
(the fetcher stops when one of its two routes is down), `SCHEMA-11` (the
multipart boundary), `CORPUS-05` (name the unidentified extension), `DRIVER-04`
(the root store a browser actually reads).

---

## Open questions for the operator

⛔ **None of these blocks anything.** Each carries a recommendation, so agreeing
costs nothing and a session that does not get an answer proceeds on the
recommendation and records that it did. [`RULES.md`](RULES.md) section 11.

### 1. ⭐ NEW. `SCHEMA-08` lists nine formats and six of them need a dependency

Its acceptance is a round trip, and a round trip needs a READER as well as a
writer. JSON, NDJSON, CSV, TSV and Markdown can be written and read back with
what this tree already has. YAML, TOML, SQLite, CBOR, MessagePack and Protobuf
each need an encoder **and** a decoder, so each is either a new dependency or a
new parser this project owns and has to keep correct.

⚠ **Delivering five of nine would change the entry's acceptance**, which is a
re-scope rather than a deviation, which is why this is a question rather than a
decision already taken.

**Recommendation: split it.** Keep `SCHEMA-08` as the generator plus the five
formats whose round trip this project can prove, and author a second entry for
the dependency-bearing formats with the trade stated. ⛔ What should not happen
is nine hand-written encoders in the crate that already owns four parsers.

### 2. ⚠ Is a per-launch key pin an acceptable standing capture method?

**Narrowed by measurement, not resolved.** `HARNESS-10` measured the capture
SURFACE and it changes nothing: the raw and terminating surfaces agree on every
TLS field that has a stable value. ⛔ What is still unmeasured is the pin against
a real trust anchor, and answering it needs a root installed into the machine's
trust store, which is a change to that machine's security configuration and is
the operator's action rather than an agent's.

**Recommendation: keep the pin as the default, and treat the trust-store
comparison as an operator-run experiment.** ⚠ `DRIVER-04` should land first: on
Windows the store a browser actually reads is not obviously the one `certutil`
writes to.

### 3. Is the privacy default right, given that it blinds the validator?

**Partly answered by doing it.** The first profile was taken with
`--header-values` deliberately, and three of the validator's checks ran and
passed that could not have otherwise. The default is still names-only.

**Recommendation: unchanged.** Keep the default and take a corpus capture with
values deliberately, which is now what `experiments/10-first-profile.sh` does.

### 4. Should `cookie` and `authorization` be dropped entirely, or kept as names?

⚠ Dropping the name also drops the fact that the header was PRESENT, which is a
fingerprint signal in its own right.

**Recommendation: a new entry that records presence without the value.**

### 5. What does a client with no root store put in the trust-anchors extension?

⛔ Unchanged. A Chrome 152 hello carries a snapshot of the browser's own root
store.

**Recommendation: publish, and do not choose.** `CORPUS-04`.

### 6. The published schema expresses no numeric bounds at all

`u8`, `u16` and `u32` are each a bare `{"type": "integer"}`, so the published
schema accepts 999 for a field the Rust type holds to a byte.

**Recommendation: an entry that bounds all three at once**, with the checker
gaining `minimum` and `maximum` in the same change.

### 7. ⚠ `check-twins` no longer finishes inside ten minutes

**Recommendation: an entry that scopes the slow halves rather than the
comparison.** ⛔ What must not happen is dropping a pair to make it fit. ⚠ And no
wrapper timeout around it: a killed half reports as a drift.

### 8. ⚠ NEW. `check-twins` cannot tell a drift from a tree that moved under it

It runs one half, then the other, and compares. A file created between the two
is reported as a disagreement between the implementations. That happened here on
`repo.has_codegraph`, and the only way to tell the two apart was to re-run both
halves by hand and find they agreed.

**Recommendation: have it record the tree's state before and after, and say so
when they differ**, rather than trying to make the run atomic. ⛔ What must not
happen is a session learning to discount a drift it has not re-checked.

### 9. ⚠ NEW. The gate's line-endings filter cannot see an unstaged file

It reads `git ls-files --eol`'s INDEX column, so a tracked file that is CRLF in
the working tree and LF in the index passes. This session produced eight such
files and the gate stayed green; `.gitattributes` normalised them on commit, so
nothing reached the history.

**Recommendation: a small entry that reads the working-tree column too**, with
the `attr/-text` and `eol=crlf` declarations honoured, because the reference
corpus and every `.ps1` are legitimately CRLF on disk.

---

## Settled, and not to be raised again

Kept short on purpose. The list lives in [`RULES.md`](RULES.md); these are the
rulings a later session should not re-open.

- ⭐ **A measured profile goes into the committed corpus with its conditions
  recorded.** Ruled by the operator 2026-09-01 and done the same day.
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound**, and the graph
  now imposes a floor of 1.88 that happens to equal it.
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- ⭐ **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
