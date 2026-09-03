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
session ran      2026-09-03, unattended, ended by the operator
baseline         gate ok: 34 passed, 1 skipped (check-twins, which --fast skips)
                 on this Windows host at the start. 380 tests.
entries          total 100  open 14  blocked 0  done 86
published        the data branch exists: 200 files on origin/data since
                 2026-09-03, pushed by publish.yml and verified tree-for-tree
gate             three checks joined it today: check-publish, check-cold-start
                 and check-support-matrix, each with a twin and a comparison row.
                 Closing run: gate ok, all 38 checks passed. 401 tests.
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## Eight entries closed, twenty-two effort points

**Two over the twenty [`RULES.md`](RULES.md) section 10 asks for.** Each is
closed in place with its acceptance command actually run and its real output
pasted underneath.

| | |
| --- | --- |
| `PUB-10` `L` | the publishing workflow, three triggers, the write scoped to two jobs, and two conditions from two sources in front of the data branch |
| `EMIT-01` `L` | a support matrix generated from a run, with five holes each carrying a file and a line a check resolves |
| `EMIT-02` `L` | the escape hatch: 1871 of a hello's 1983 bytes emitted and found in the raw capture exactly once |
| `LIB-01` `M` | a crate that hands a program a profile, with no network in it and no fallback behind it |
| `LIB-02` `M` | the smallest client, and 1951 of 1983 bytes identical to what the browser sent |
| `VALID-04` `M` | JA4 from the specification, and sixteen vectors none of whose expected values came from running this code |
| `CI-05` `M` | the cold-start job, every cache refused and every stage named by a report |
| `DRIVER-06` `M` | the branded pair end to end, and both `for-testing` matrix cells enabled |

---

## ⭐ What changed about this project today

**Nothing this project builds had ever been publishable by an event.** `PUB-01`
and `PUB-02` assembled and checked two surfaces and no trigger reached either.
There is one now, on the three events the operator ruled, and ⛔ **the first push
to the default branch after this lands creates the `data` branch.** That is the
first step of the sequence the operator wrote, and it is intended.

**And the corpus is usable rather than only accurate.** A crate hands a
program a profile; a client puts one back on a wire and the harness reads back
the same profile, field by field.

---

## ⛔ Findings in this session's own work, each caught by running it

| what | how it showed |
| --- | --- |
| ⛔ the published route manifest carried the ABSOLUTE PATH of whoever built it | so a build under a relative root and one under an absolute root produced different bytes for one corpus, and a public artefact would have carried a home directory. `check-release` could not see it: it builds twice under ONE root |
| ⛔ a guard added this session passed the mutation it was written for | the force-push rule looked for `:+`, and a forcing refspec is `+src:dst`, so the plus is at the START of the token |
| ⛔ that rule was gated on one workflow and not its sibling | `capture.yml` carries a force push that nothing asserted anything about. The rule now reads every workflow and pins the count at one |
| ⛔ five documents said the corpus holds FIVE profiles and it holds six | and one said a single profile carries the trust-anchor extension where two do. A sixth profile landed and nothing compared the prose against the tree |
| ⛔ two pasted acceptance blocks had a line removed | both commands print more than one target's result, and dropping the empty ones is an edited paste. Rewritten to what the command prints, with the elision stated |
| ⛔ `EMIT-01`'s byte range was read off the first two cells | 1803 where the measured minimum is 1739. Corrected by re-running the generator |
| ⚠ the model cannot hold three of the specification's ALPN examples | `alpn` is a list of strings, so a protocol whose bytes are not UTF-8 is unrepresentable. Recorded rather than repaired: changing it moves every published profile's serialisation |
| ⚠ `EMIT-02`'s acceptance could not be satisfied as written | comparing the emitted bytes with the raw hex needs the `ClientHello` random, which the model does not record and should not |
| ⚠ `LIB-02`'s acceptance named a profile the corpus does not hold | and the client refuses one by name rather than substituting, so the acceptance refuted itself |
| ⚠ check 3 answers not-checkable over a profile that keeps header names only | which is the default. The enforcement half `DRIVER-06` called existing fires only on a capture that turned values on |
| ⛔ this session own placeholder check fired in one half and not the other | PowerShell `-match` is case-INSENSITIVE and the twin `grep -E` is not, so an ordinary sentence containing three lower-case words was a template instruction to one half. Found by `check-twins`, fixed with `-cmatch` |
| ⛔ shellcheck refused three lines of a check written this session | `A && B || C` is not if-then-else, and the gate treats that as a failure rather than a note. Rewritten as a function with an explicit `if` |
| ⛔ a skip whose own condition had been met still reported a skip | `check-data-branch` said the branch was `remote` and printed "push it once and this leg starts running" in the same breath. The branch was there and nothing compared against it |
| ⛔ two PowerShell argument traps in one line, found by the twin disagreeing | `--git-dir=(expr)` passes two arguments rather than one, and a bare `--` is consumed before a native command sees it. The half staged nothing and produced the empty tree, which compares unequal to everything |
| a heredoc ate a backslash in a Rust literal, twice | the tree's own tooling note says a heredoc is not reliably literal here, and the payload has to go through `write-file.mjs` |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what other door reaches this

Swept by grep rather than from memory: every caller of `ja4_lists`, `ja4_r`,
`ja4_ro`, `ja4_prefix`, `ja4_alpn`, `ja4_version`, `extensions_block`,
`client_hello`, `unnamed_codepoints`, `support_matrix`, `parse_tag`,
`would_rewrite`, `plan_release` and `differences`; every workflow that declares
a write; every `git push` in every workflow; every `gh` command that mutates;
and every reference to a secret.

**Confirmed, by counting:** exactly two workflows can write and both scope it
to a job; no workflow names a secret at all; `publish::build` has one production
caller.

⛔ **Finding: two `git push` lines exist and only one was governed.** The new
check read the publishing workflow alone, while the capture workflow
force-pushes a bot branch. That is the one-gated-door class exactly. The rule
now reads every workflow, pins the force-pushing count at one, and refuses any
force naming the data branch or the default branch. ⚠ Planting a force push at
the data branch in the SIBLING file takes both halves red.

**What the other passes did not look at:** the callers and the sibling files.
Both of the others read what was written; this one asked what else reaches it.

### 2. The guard mutation: can the new guards actually fail

Twenty-eight mutations, each read unpiped in the half that owns the guard.
⛔ Every mutation of a tracked file was made against a copy under the ignored
scratch directory, and the live file was compared byte for byte with that copy
afterwards. Each entry's closing carries its own table.

| where | planted | red |
| --- | --- | --- |
| `PUB-10` | ten in the workflow, plus the crate's orphan rule and the assembler's path handling | ⛔ eleven of twelve on the first attempt; the `+` refspec passed and was fixed |
| `CI-05` | seven in the workflow, plus a program made absent | all eight, in both halves |
| `EMIT-01` | five in the generator | all five, in both halves |
| `LIB-01` | the release identifier, and a fallback in the selector | both |
| `LIB-02` | a missing profile, a swapped extension, a dropped cipher, a constant random | all four |
| `VALID-04` | a corrupted expectation and a corrupted input | both |
| `EMIT-02` | a reordered list and a length that disagrees with its body | both |

⚠ **Guards NOT mutated, and saying so is the point:** nothing exercised the
publishing workflow itself, because running it writes to the remote; the
`for-testing` capture lane has never run; and the cold-start job has never run
on a cold machine, because this one is warm by definition.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every pasted block in the eight closings, this file, the
changelog, the README, the router, the technical reference and the two documents
the work made stale.

⛔ **Finding: the profile count was wrong in five places.** A sixth profile
landed and the prose was never compared against the tree, which is the defect
[`RULES.md`](RULES.md) section 3 exists for, in the five most-read documents.
Corrected against `b-ids-corpus verify` and `b-ids-corpus anchors`, each of
whose last line is a fixed count.

⛔ **Finding: two pasted blocks were edited and one number was read off the
wrong rows.** Both corrected by re-running the command and pasting what came
back, with the elision stated where a paste genuinely drops lines.

**Claims checked that stood, by re-running the command:** 198 artefacts at
673814 bytes; 3 triggers over 3 jobs with 2 job-scoped writes and 10 refusals
driven; 11 cold-start stages over 9 programs; 6 matrix cells and 5 resolving
holes; 4 vector cases, 5 escape-hatch cases and 4 client cases; the record at
100 entries, 14 open, 86 done.

---

## ⚠ What is in progress

⛔ **Nothing half-edited.** Every entry this session touched is closed in place.

---

## Open questions for the operator

### ⭐ None about the data branch any more, because it exists

⛔ **The workflow ran and the branch is published.** The push that landed
`PUB-10` triggered it: the assemble job and the data-branch job both succeeded,
the release job skipped because no tag was pushed, and `origin/data` carries
200 files committed by the platform's own actor.

⭐ **Verified, not assumed.** The tree object the branch carries is the tree a
local build of this corpus produces, compared object for object, which is what
"byte for byte" means for a branch. `check-data-branch` does that comparison
now in both halves and reports it as `matched` in its JSON, so the twin row
proves both halves did it.

⛔ **The next step is the operator's sequence, not a question**: `PUB-11` moves
the eleven check pairs that read the corpus from the working tree, and only then
does `corpus/` leave the default branch.

### ⚠ Two things this session deliberately did not do

- ⛔ **No release was cut and no tag pushed.** A tag is the operator's own act,
  which is why it is the only thing that produces a release.
- ⛔ **The moving `v1` and `latest` GIT tags are not wired**, and `PUB-10`'s
  decision says why: a moving tag is a force-update of a ref, and the release
  pointer gives a consumer the same fetch without listing anything.

---

## ⭐ The work order

⚠ **Take these in order.**

1. **`PUB-11`**, which is authored and unstarted, and which the operator's
   sequence puts after the data branch is pushed and verified. Eleven check
   pairs read the corpus from the working tree.
2. **`CORPUS-02`**, whose acceptance names four rows and refuses on two.
   ⛔ Both are blocked on one thing: `b_ids_driver::Family` knows two families.
   `firefox` is the higher value: a genuinely different TLS stack.
3. **`PUB-04`**, which `EMIT-01` has just unblocked: there is a support matrix
   to ask before a snippet is generated.
4. **`VALID-05`**, the conformance suite, which is what would turn a hole in
   that matrix into a cell for somebody else's stack.
5. **`HARNESS-11`**, the p0f layer. ⚠ Establish the capability first: a raw
   socket needs a dependency this workspace does not have and an `unsafe` it
   denies, so the answer may be that it is a local-only extra.
6. **`EMIT-03`**, which needs a second vendored tree before its five bytes.

**Small entries worth taking whenever a larger one is blocked**: `DOC-02` and
`DOC-03`, both of which their own entries say to write only when a specific
thing becomes true. ⛔ Re-checked 2026-09-03 and neither is: no workflow needs a
secret, no release needs a signing key, and no release has shipped.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

### Ruled 2026-09-03 by the operator, after the previous session wrote its record

- **The publishing workflow is triggered three ways**: `workflow_dispatch`, a
  push to `main`, and a pushed tag. ⭐ Done: `PUB-10`. The write is job-scoped,
  using the run's own `GITHUB_TOKEN` and never a personal access token, and the
  data branch is append-only and never force-pushed.
- **Removing `corpus/` and `raw/` from `main` is sequenced, data branch
  first.** ⛔ Nothing was deleted. The order is: push the data branch and verify
  it byte for byte; then move the checks that read the corpus from the working
  tree; then remove it. `PUB-11` is authored for the middle step.
- **The history reset on `main` is not yet.** ⛔ It is the operator's action,
  after the data is published and verified, and no session force-pushes this
  remote.
- **Both `for-testing` matrix cells are enabled.** ⭐ Done: the coverage report
  moves them from `not-attempted` to `absent`, which is the difference between
  planned and tried. ⚠ Nothing has exercised that lane's capture path.

### Ruled 2026-09-03, by the operator's standing instruction for the session

- ⛔ **`for-testing` is a `Channel`.** ⭐ Done: the variant and the schema enum
  value are in, and `DRIVER-06`'s closing states the rest of the ruling and what
  it cost. The alternative lost because it would have changed `corpus/v1/` for
  every consumer to carry a dimension only one browser family has.
- **`SCHEMA-12`'s six formats are four and two.** YAML, TOML, SQLite as a
  text dump and protobuf as a definition are published; CBOR and MessagePack
  are declined with their reasons published beside them.
- ⭐ **Routes are generated only where the corpus HOLDS a value.** A route that
  resolves to a plausible-looking wrong value is worse than one that 404s. ⛔ It
  is why no digest route exists even now that JA4 is computable.
- ⭐ **JA4 is implemented and no member of its extended family is.** JA4 is
  BSD-3 with no patent claim; the rest is patent-pending and
  monetisation-restricted, and that question has no answer written down.
- ⛔ **The release job moves no git tag.**

### Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** Done: `DRIVER-08` closed, and `capture.yml` provisions where the
  cell asks for it.
- **The corpus carries BOTH Chromes, as separate matrix cells.**
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
- ⛔ **A guard on something irreversible is TWO conditions from two sources**,
  and it is never mutated on the machine it protects. Held all session.
- **The write for `CI-04` is JOB-SCOPED**, using the run's own token.
  ⛔ Never a personal access token, and no workflow in this tree names a secret.
- **The first runner capture is fetched with `gh` and added by hand.**
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.** ⛔ That is the OPERATOR'S action at a time of their choosing and
  it licenses nothing for a session: no force push, no history rewrite, and the
  corpus stays append-only in every change an agent makes.
- **Credentials are recorded as PRESENT, never as a value.**
- **The trust anchor is a job, not a machine change.**
- **Header values stay names-only by default.** Corpus captures turn them on
  deliberately.
- **The schema gains numeric bounds.**
- **The shuffle seed stays out of `browser-profile/1`.**
- **Commit once at the close** unless the session is genuinely at risk of losing
  work.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
