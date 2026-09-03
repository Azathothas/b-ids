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
session ran      2026-09-02T23:21:15Z, unattended, ended by the operator
baseline         gate ok: 30 passed, 1 skipped (check-twins, which --fast skips)
                 on this Windows host at the start. 353 tests.
entries          total 98  open 20  blocked 0  done 78
gate             four checks joined it today: check-pr-body,
                 check-license-consistency, check-release, check-data-branch
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## ⭐ Six entries closed, thirteen effort points

⚠ **Thirteen of the twenty [`RULES.md`](RULES.md) section 10 asks for.** The
operator ended the session; the entries below are each closed in place with the
acceptance command actually run.

| | |
| --- | --- |
| `SCHEMA-12` `L` | four more formats from the one generator and two declined with their reasons published. The generator reads every file back before it writes it |
| `CI-04` `M` | the pull-request body, the branch, the labels and five merge conditions, three of them computed from the published profiles rather than claimed by the run |
| `PUB-03` `M` | fifty-four flat routes, and a check that reads the corpus rather than the generator |
| `PUB-01` `M` | one assembler, a build that is byte-identical twice, and a tag that cannot overwrite one somebody pinned |
| `PUB-02` `M` | the same assembler for the data branch, with every file carrying a checksum in two places |
| `PUB-07` `S` | the licence stated in seven places from one home |

---

## ⭐ The open question is answered, and it cost one enum variant

⛔ **The operator's standing instruction for this session was that an open
question takes whatever this record recommended.** The recommendation was
`for-testing` as a `Channel`, and it is in the model and in the published schema
now.

| | |
| --- | --- |
| ⭐ what it cost | one variant, one schema enum value, and an explicit serde rename. Nothing about the layout moved, because the channel is already part of the route and of the `latest` key |
| ⚠ what it did not cost | no consumer's pin moves, and `latest` is unaffected: the pointer map is built from stable profiles alone |
| ⛔ what it unblocked | the two `for-testing` cells in [`../.github/capture-matrix.json`](../.github/capture-matrix.json), and `DRIVER-06`, which is still open |

---

## ⛔ Findings in this session's own code, each caught by running it

| what | how it showed |
| --- | --- |
| ⛔ two of `SCHEMA-08`'s readers split their input into LINES | and both formats allow a newline inside a quoted value. Found by rendering a profile whose values carry a quote, an apostrophe, a tab, a newline, a backslash and a non-ASCII character. Neither was reachable from today's corpus and both were latent silent-corruption paths |
| ⛔ a guard added this session passed over the mutation that should have taken it red | the SQLite leg read ANY error from one query as "this host has no JSON1", so a dump whose `CREATE TABLE` no longer declared the column the format promises reported `ok-no-json1` and exited 0 |
| ⛔ the early return that makes a no-op change produce nothing enforces nothing | removing it changed no behaviour: the loop below it already had nothing to iterate. Kept as the explicit statement of the invariant, with a comment saying which of the two holds it |
| ⛔ jq on Windows writes CRLF, for the SECOND time in this tree | every one of 54 route comparisons failed while both sides were correct. `CORPUS-02` carries the first occurrence |
| ⛔ a generated tree under `.tmp` is invisible to `git ls-files --others` | so `check-routes` reported a clean tree it had never opened. Same class as the fixture defect its own header already described, from the other direction |
| a list file and a single-value file both end in `txt` | so a classifier reading the last dot would have refused the newline a list needs |
| ⛔ the two halves of one check resolved DIFFERENT tar binaries | GNU tar wants `--owner=0` and refuses `--uid`; the bsdtar Windows ships wants `--uid 0` and refuses `--force-local`; and one date format is a parse error to one of them. The archive leg skipped on every host until both spellings were probed |
| ⚠ `CI-04` predicted a version bump moves the User-Agent and the brand list | the field-level diff compares header POSITIONS and not header VALUES, so neither is a field it can report |
| a PowerShell local named `$json` writes to the `[switch]$Json` PARAMETER | variables are case-insensitive, so the script failed to bind before it ran a line |
| `$profile` is a PowerShell AUTOMATIC variable | assigning to it in a loop is a lint error and, worse, writes to the host's profile path |
| ⛔ an exit code in this session's own measurement was read through a pipe | reporting `rc=0` over a check that had plainly failed. Re-run unpiped, and the row above is the real answer |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what other door reaches this code

Swept, by grep rather than from memory: every caller of `render`, `verify`,
`read_back`, `read_tree`, `read_flat`, `support_matrix`, `routes`, `build`,
`plan_release`, `would_rewrite` and `requests`; every reader of
`b_ids_schema::LICENSE`; and every consumer of `Channel::all`.

⭐ **Confirmed, by counting:** `Channel::all` has two consumers outside the
schema crate and both read it dynamically, so the seventh channel widened what
`VALID-03` calls reachable without anybody editing a list. `LICENSE` is read in
25 places and typed in none. `formats::render` and `publish::build` have exactly
one production caller each.

⛔ **Finding: the licence has two writers and one of them is a JSON file.** The
model, the index and the release body all read the constant; the published JSON
Schema states it as a `const` in a file no Rust code reads. That is a value in
two places, so `check-license-consistency` compares them rather than trusting
them, and its `--fixture` leg asserts the comparison refuses a disagreement.

**What the other passes did not look at:** the callers. Both of the others read
what was written; this one grepped for what was not enumerated.

### 2. The guard mutation: can the new guards actually fail

Planted and read unpiped, each in the half where the guard lives. Every mutation
of a source file was made against a copy under the ignored scratch directory and
the live file was compared byte for byte with that copy afterwards.

| planted | what went red |
| --- | --- |
| the YAML writer drops the last key of every mapping | the generator refused before writing, exit **1** |
| `support_matrix` stops naming the declined formats | `the support matrix declines nothing`, exit **1** |
| `corpus.toml` not written while the matrix still names it | two problems at once, exit **1** |
| the dump's inserts lose their terminator | the generator refused, exit **1** |
| ⛔ the dump's `CREATE TABLE` renames its canonical column | ⛔ **nothing. The check passed.** Fixed, and then exit **1** |
| ⛔ the early return on a no-op change removed | ⛔ **nothing.** It is not the thing enforcing the rule |
| a no-op change made to open one request per route | three problems at once, exit **1** |
| a suite case renamed out of a check's expected list | `... is not in the suite`, exit **1** |
| `Cargo.toml` states a different licence | `Cargo.toml says MIT and ... says 0BSD`, exit **1** |
| the route generator reads the wrong header for a property | 9 problems, exit **1**, in BOTH halves with identical messages |
| a fixture route given a trailing newline | exit **1**, and a `.list.txt` beside it was **not** flagged |

⚠ **Guards NOT mutated, and saying so is the point:** the protobuf definition's
round trip is asserted inside its own suite rather than by planting in the
generator; the data-branch comparison against what is published has never fired
because there is nothing published; and the release tag collision was proved
against `plan_release` rather than by creating a tag on this repository.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every pasted block in the six closings, this file, the
changelog and the documents the work made stale.

⛔ **Finding: a pasted digest is a credential-shaped run in a tracked file.**
`PUB-01`'s block carried the corpus content address in full and
`check-no-secrets --public` refused the tree. Abbreviated at an ellipsis with
the reason stated, which is this tree's own precedent from `TOOL-03` and
`CORPUS-01`, and chosen over widening a security rule for a cosmetic reason.

⛔ **Finding: a mutation table was written before its mutations were run.**
`CI-04`'s closing named a deleted-case mutation and a five-condition mutation as
though both had been performed. One had not. Both were then run and the table
rewritten to what actually happened, including the row where the mutation found
nothing.

**Claims checked that stood, by re-running the command:** `files:10
profiles:6`; 54 routes verified against the corpus; 197 artefacts at 668384
bytes, identical over two builds; 34 scripts answering 2; 35 tracked `.ps1`
files parsing; the record at 98 entries, 20 open, 78 done.

---

## ⚠ What is in progress

⛔ **Nothing half-edited.** `DRIVER-06` is open and carries what landed:
`Channel::ForTesting` is in the model and the published schema, the suite is
green over it, and the entry says in as many words that its acceptance command
selects no test yet.

---

## Open questions for the operator

### ⛔ One, and it is a repository setting rather than a decision

**Actions must be allowed to create pull requests** for `CI-04`'s collect job to
open one. The job carries `contents: write` and `pull-requests: write` and uses
the run's own token; the setting is not something a workflow file can grant, and
the step reports the refusal in its own words rather than failing silently.

**Recommendation: enable it.** ⚠ Nothing is blocked meanwhile: the body
generator, the branch naming, the labels and the merge conditions are all
checked locally by `check-pr-body --fixture`, and a run that finds no change
opens nothing either way.

### ⚠ Two things this session deliberately did not do, and both are one field

- ⛔ **No workflow publishes.** `PUB-01` and `PUB-02` assemble and check; nothing
  cuts a tag, uploads an asset or creates a branch. A workflow that did would be
  an outward-facing action taken by a session rather than by the operator.
- ⛔ **The two `for-testing` matrix cells are still disabled.** The vocabulary no
  longer blocks them, and enabling one makes the next scheduled run attempt a
  lane whose capture path nothing here has exercised.

---

## ⭐ The work order

⚠ **Take these in order.**

1. **`DRIVER-06`**, which is one pair of tests away from its acceptance. The
   validator already refuses a profile whose `browser.branded` disagrees with
   its brand list; what is missing is the driver-side pair named `branded` that
   drives an unbranded acquisition and asserts that refusal end to end.
2. **`CORPUS-02`**, whose acceptance names four rows and refuses on two.
   ⛔ Both are blocked on one thing: `b_ids_driver::Family` knows two families.
   `firefox` is the higher value: a genuinely different TLS stack.
3. **`EMIT-03`**, whose measurement is in: every profile carries the priority
   block, so the entry takes the branch that needs the HTTP/2 library vendored.
4. **`PUB-04`**, which now has somewhere to go: `publish::build` is the
   assembler and a snippet is another artefact in the tree it produces.
5. **`VALID-04`**, ⛔ with a licence question stated in the entry that comes
   first, and `SCHEMA-12`'s note that no digest route exists until it lands.
6. **`HARNESS-11`**, the p0f layer, which is free once the listener is ours.

⚠ **Small entries worth taking whenever a larger one is blocked**: `DOC-02` and
`DOC-03`, both of which their own entries say to write only when a specific
thing becomes true, and `CI-05`, the cold-start job.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

### ⭐ Ruled 2026-09-03, by the operator's standing instruction for the session

- ⛔ **`for-testing` is a `Channel`**, and `branded: false` follows from it
  rather than becoming a fifth path component. ⭐ Done: the variant and the
  schema enum value are in. The alternative lost because it would have changed
  `corpus/v1/` for every consumer to carry a dimension only one browser family
  has.
- ⭐ **`SCHEMA-12`'s six formats are four and two.** YAML, TOML, SQLite as a
  text dump and protobuf as a definition are published; CBOR and MessagePack
  are declined with their reasons published beside them.
- ⭐ **Routes are generated only where the corpus HOLDS a value.** A route that
  resolves to a plausible-looking wrong value is worse than one that 404s.

### Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** ⭐ Done: `DRIVER-08` closed, and `capture.yml` provisions where the
  cell asks for it.
- ⭐ **The corpus carries BOTH Chromes, as separate matrix cells.**
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
- ⛔ **A guard on something irreversible is TWO conditions from two sources**,
  and it is never mutated on the machine it protects. ⭐ Held all session.
- ⭐ **The write for `CI-04` is JOB-SCOPED.** `contents: write` and
  `pull-requests: write` on the collect job alone, using the run's own
  `GITHUB_TOKEN`. ⛔ Never a personal access token.
- ⭐ **The first runner capture is fetched with `gh` and added by hand.**
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
- ⭐ **The shuffle seed stays out of `browser-profile/1`.**
- **Commit once at the close** unless the session is genuinely at risk of losing
  work. ⚠ This session made one checkpoint commit mid-way, squashed before the
  push, because it was unattended and carried several hours of work.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
