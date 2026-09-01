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
session ran      2026-09-01T12:32:05Z to 2026-09-01T15:20:00Z, unattended,
                 ended by operator interrupt
baseline         the gate passes: 25 checks and check-twins, both halves of
                 every pair. 256 tests in 28 files across 5 crates.
entries          total 91  open 43  blocked 0  done 48
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## The correction that reordered everything

**The operator ruled 2026-09-01 that this project is GitHub CI, 100%.** The
previous session captured its one profile on a developer's Windows laptop and
ordered continuous integration last. That was the wrong way round: a corpus of
browser fingerprints whose captures depend on one person's machine cannot cover
the matrix, cannot be reproduced by anybody else, and cannot be scheduled.
**Captures belong on runners.**

Three consequences, each of which moved an entry:

- a runner is **disposable**, so installing a root certificate into its trust
  store is free and undoes itself. `HARNESS-14` is that job.
- a runner has **no browser** unless something fetches one, so `DRIVER-05` was
  the blocker rather than a later nicety. ⚠ Except `ubuntu-latest`, which ships
  Chrome preinstalled and is therefore the first lane in the matrix.
- the capture path already works headless and the fuzz run already proved the
  Linux-container route, so none of this was speculative.

⛔ **The one profile in the corpus stays.** It is a real measurement with its
conditions recorded and the corpus is append-only. It is simply not the model
for how the next hundred arrive.

---

## What this session did

**Seven entries closed, seven authored, and four checks that were reporting
green over questions they had not asked.**

### The pipeline a push now runs

| | |
| --- | --- |
| ⭐ `b-ids-corpus validate` | the coherence checks over what is PUBLISHED, plus the cross-profile `shared_handshakes` no per-profile invocation can reach |
| ⭐ `check-validate` | that answer, plus a determinism leg: the generator runs twice over a throwaway copy and the bytes are compared |
| ⭐ `check-line-endings` | extracted from inside both gate halves, reading the working-tree column as well as the index one |
| ⭐ `check-workflows` | four structural rules over every workflow, with a fixture that breaks each one |
| ⭐ `check-coverage` | every planned capture cell reported as captured, absent or not attempted |
| ⭐ `validate.yml` | every push, two hosts, `CARGO_NET_OFFLINE` on every assertion, whole history fetched |
| ⭐ `capture.yml` | the matrix, fanned out from a plan in the tree, every lane failing alone |
| ⭐ `b-ids-driver::acquire` | routes tried in order, the one that answered recorded with the digest of what arrived |

### ⛔ Four checks were green over questions they had not asked

| the check | what it was not checking |
| --- | --- |
| `check-corpus` | its history leg ran under `actions/checkout`'s default one-commit clone, so `git log --diff-filter=MDR` saw a single commit and answered "nothing was edited" on every CI run since it was written. Reproduced on a `--depth 1` clone. |
| the gate's line-endings filter | it read git's INDEX column alone. It found `scripts/common/check-routes.ps1` LF on disk against its own `eol=crlf` on its first run, and fixing that produced no git diff at all. |
| `check-twins` | it could not tell a real drift from a tree that moved under it. It proved itself twice this session, once by accident. |
| `mine-repo` | it exited before the clone when its API route was down, so a host that could clone and not reach the API got nothing |

### ⭐ Three findings in this session's own new code

- ⛔ **An uninitialised awk variable used as a SUBSCRIPT is the empty string, not
  zero.** `check-workflows` reported a job that does not exist, once per file.
  ⚠ Its PowerShell twin never had it, because it appends to a list.
- ⛔ **jq on this Windows host writes CRLF.** `check-coverage`'s human report
  dropped the word `required` from every required row while its JSON, which does
  not carry that field, matched its twin exactly. ⚠ A divergence the comparison
  structurally could not see.
- ⛔ **The driver cannot link the harness.** Its manifest keeps `b-ids-harness`
  as a dev dependency on purpose, so `acquire_with` takes the digest as a second
  injected function rather than hashing the bytes itself.

### ⭐ `check-twins` costs 636 seconds now rather than 1056

⛔ **Measured with `--timings` before and after rather than estimated.** 970
seconds across twenty pairs, of which the `check-gate` row alone was 431: that
row runs both gates in full, and each gate re-runs the fourteen checks that
already have a row of their own. A gate running inside `check-twins` skips them
now and the row costs 54 seconds. ⛔ No pair was dropped; the file compared 18
at the start of this session and compares 22 now.


### ⛔ The remote went red on the first push, and the finding is this session's own

`validate` passed on both hosts. `ci` failed on both, for one reason: the gate
transcript pasted into `CI-01`'s closing carries the absolute path of the
repository root on its first line, and `check-no-secrets --public` refuses an
absolute home path in a public repository.

⛔ **The local gate would have caught it and was not re-run.** The block was
filled in AFTER the run it pastes, and only the prose checks were run over the
edit. ⚠ That is the discipline this repository already states: a unit of work
whose content changed re-passes the gate against what it is NOW, not against
what it was when the transcript was taken.

⭐ **Fixed by eliding the path and marking the substitution**, the way the
`TOOL-04` paste already elides its scratch directory. ⛔ The rule was not
widened for a pasted transcript.

⚠ **This session therefore has TWO commits rather than one.** Amending the
first would need a force push, and "no force push, no history rewrite" is a
standing ruling. Fixing forward is what
[`RULES.md`](RULES.md) section 10 step 11 asks for.

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what other door reaches this code

Swept, by grep rather than from memory: every caller of `acquire_with` and
`plan`, every reader and writer of `captured.acquisition`, every construction of
`Captured` in the tree, every registration of the five new checks in both gate
halves and in the twin comparison, and every place the `COMPARED_DIRECTLY` list
is read.

⛔ **Finding: nothing validated `captured.acquisition`.** The published schema
constrains its route to an enum and its object to four required fields;
`Profile::check` did not, so a profile could claim a route no driver can produce
and a digest that is not one, and every check in this tree would have passed it.
⭐ Fixed: the route is checked against a named list and the digest against the
same 64-lower-case-hex shape the corpus index uses for every published file.

⭐ **Confirmed, by counting:** four constructions of `Captured` and every one
now names the field; five new checks and every one appears in both gate halves,
in the `COMPARED_DIRECTLY` list and in `check-twins`; two implementations of the
acquisition route list, and the one that can be compared against the schema is.

⭐ **What the other passes did not look at:** the callers. Both of the others
read what was written; this one grepped for what was not enumerated.

### 2. The guard mutation: can the new guards actually fail

Planted and read unpiped, each in both halves where a twin exists: the
shallow-clone refusal, against a real `--depth 1` clone; the determinism leg,
against an index writer made to append its process id; the tree-moved detection,
by editing `TODO/` while the comparison ran; `check-workflows`, against a fixture
that breaks each of its five rules exactly once; `check-coverage --require-rows`,
against three browsers with no capture; `mine-repo` with one route down and with
both; and the new `captured.acquisition` route check, disabled with `if false &&`
and seen to take one test red.

⛔ **Finding: the determinism leg's message ran two findings onto one line.** A
command substitution strips trailing newlines and the accumulator joined without
one. Found by the mutation rather than by review, and fixed.

⛔ **Finding: `check-workflows` reported a job that does not exist**, once per
file. `CI-03` carries the awk rule behind it. ⚠ Its PowerShell twin never had
the defect, because it appends to a list rather than indexing an array.

⚠ **Two guards were NOT mutated**, and saying so is the point: the `always()`
rule fires on the fixture but has never been seen to pass over a `needs` on a
job that does not fan out other than in this tree's own workflows, and
`check-twins`'s human UNDECIDED banner is the same branch on the same two
variables as its JSON and was proved only through the JSON.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every pasted block in the seven closings, this file, the
changelog, and the two documents the work made stale.

⛔ **Finding: a fabricated block, caught before it was committed.** `CI-01`'s
closing was written with a gate transcript assembled by hand from a run whose log
two processes had written into. It was replaced with a marker and then with the
real run below.

⛔ **Finding: a pair count that was right when written and wrong when read.**
"one was ADDED" and "three were added" were both counted against the file at the
moment of writing. Counted from the tree: 18 pairs at the session's first commit
and 22 now, with the timing figures taken over 20.

⛔ **Finding: a section number that names nothing.** This file cited
`RULES.md` section 11 for a whole session; that file has ten sections and a
settled list.

⭐ **Claims checked that stood:** 970 seconds against the sum of the twenty rows;
431 and 54 against the `check-gate` row before and after; 1056 and 636 against
the two wall clocks; 84 files CRLF on disk of which 66 are under `references/`;
5388 tracked files; 256 tests in 28 files.

---

## What is in progress

⛔ **Nothing is half-edited.** Every entry this session touched is closed in
place with its acceptance command run, or left open with its blocker named.

⚠ **`CORPUS-02` is open and it is the next thing.** Its apparatus is built - the
plan file, the coverage check in both halves, and the fan-out that reads the
plan - and no lane has run. ⛔ Closing it needs one run of `capture.yml` on a
hosted runner and the `linux64` profile committed, which needs this session's
commit on the default branch.

---

## ⭐ The work order

⚠ **Take these in order.**

1. ⭐ **`CORPUS-02`.** Run `capture.yml` on the default branch with the
   authenticated `gh`, download the `linux64` artefact, add it with
   `b-ids-corpus add`, and close the entry. ⭐ The operator ruled this route
   2026-09-01: `gh` is authenticated, and `CI-04` is not built first. ⭐ **Two profiles of ONE build on TWO platforms is the single
   highest-value capture available**: it decides whether the TLS half is
   platform-independent, and `VALID-01`'s handshake check reports
   `NotCheckable` until it exists. ⚠ The one profile there today came from a
   laptop, so it is one source rather than two.
2. **`DRIVER-04`**, then **`HARNESS-14`**. The root store a browser actually
   reads, then the per-launch pin measured against a real trust anchor on a
   disposable runner. ⚠ `DRIVER-04` lands first: on Windows the store a browser
   reads is not obviously the one `certutil` writes to, and measuring against
   the wrong store gives a confident wrong answer.
3. **`SCHEMA-13`** and **`SCHEMA-14`**, both small and both about the published
   contract: numeric bounds the schema does not express, and a credential's
   presence recorded without its value.
4. **`CI-02`** and **`CI-04`**. Staleness on a schedule, and a run that finds a
   change opening a pull request. ⭐ `CI-04`'s write is ruled: job-scoped, on the
   collect job alone, with the run's own token. See the settled list.
5. **`SCHEMA-08`**, then `PUB-03`, `PUB-01`, `PUB-02`, `PUB-07`.
6. **`SCHEMA-12`**, the six formats that need a decoder, once `SCHEMA-08` has a
   generator to extend.

⚠ **Small entries worth taking whenever a larger one is blocked**: `SCHEMA-11`
(the multipart boundary), `CORPUS-05` (name the unidentified extension),
`VALID-03` (a family the resolver cannot produce), `DRIVER-06` (branded and
unbranded builds).

---

## Open questions for the operator

⭐ **None.** All three were put to the operator interactively at the close of
2026-09-01 and all three were answered; the rulings are in the section below.

⚠ **A later session that finds a fork writes it here with a recommendation
attached and keeps working.** [`RULES.md`](RULES.md) section 10 names "this
needs a decision from the operator" as one of the four sentences that is not a
reason to stop. ⛔ Ask at the very START of a session if proceeding under any
assumption would be unsafe; otherwise record it here and proceed on the
recommendation.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01.**

- ⭐ **The write for `CI-04` is JOB-SCOPED.** `contents: write` and
  `pull-requests: write` on the collect job alone, using the run's own
  `GITHUB_TOKEN`. ⛔ Never a personal access token: a long-lived credential in a
  public repository's automation outlives every run it was issued for. ⚠ Every
  capture lane keeps `contents: read`, so a browser this project downloaded and
  ran can never reach the repository. ⚠ It also needs the repository setting
  that lets Actions create pull requests, which is the operator's to enable and
  cannot be done from the tree.
- ⭐ **The first runner capture is fetched with `gh` and added by hand.** The
  authenticated CLI runs `capture.yml`, downloads the `linux64` artefact, and
  `b-ids-corpus add` writes it. ⛔ Do not build `CI-04` first: that is machinery
  ahead of the single capture it would review. ⚠ The profile is a real
  measurement either way, taken on the runner by the same script; only the
  transport is manual and the profile's own provenance says so.
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.** ⛔ That is the OPERATOR'S action at a time of their choosing and
  it licenses nothing for a session: no force push, no history rewrite, and the
  corpus stays append-only in every change an agent makes. ⭐ What it does settle
  is that a laptop capture sitting beside runner captures is not a problem to
  engineer around today.

- **`SCHEMA-08` is SPLIT.** It keeps the generator plus the five formats whose
  round trip this tree can prove: JSON, NDJSON, CSV, TSV and Markdown.
  `SCHEMA-12` carries YAML, TOML, SQLite, CBOR, MessagePack and Protobuf with
  the trade stated. ⛔ Twelve hand-written implementations in the crate that
  already owns four parsers is what must not happen.
- **Credentials are recorded as PRESENT, never as a value.** `SCHEMA-14`. ⛔ The
  value never appears on any surface, including the raw block.
- **The trust anchor is a job, not a machine change.** `HARNESS-14`. Every
  profile keeps recording `captured.trust`, because that is what makes the
  comparison possible at all.
- **Header values stay names-only by default.** Corpus captures turn them on
  deliberately. ⛔ A model whose natural form carries them is the shape that one
  day publishes a credential.
- **`CORPUS-04` publishes the per-build trust-anchor list and states all three
  options with their costs.** ⛔ It asserts no preference.
- **The schema gains numeric bounds.** `SCHEMA-13`.
- **Commit once at the close** unless the session is genuinely at risk of losing
  work. No force push and no history rewrite.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
