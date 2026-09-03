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
baseline         gate ok: 37 passed, 1 skipped (check-twins, which --fast skips)
                 on this Windows host at the start. 401 tests.
entries          total 103  open 14  blocked 0  done 89
published        the data branch: 200 files on origin/data, unchanged by this
                 session. ⚠ No release: a pushed tag is the only thing that
                 cuts one.
gate             check-catalogues joined it, taking the gate from 38 checks to
                 39, and two pairs joined the twin comparison. The closing run
                 is in SUMMARY.md and its output is below.
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

### The closing gate, run twice and both readings kept

⛔ **The full run came first, over the code, and it is the one that includes
`check-twins`.**

```text
$ pwsh -NoProfile -File scripts/common/check-gate.ps1
gate ok: all 39 checks passed
```

⚠ **The record was still being written when that started**, so a second run
covers the documents it did not see. `--fast` skips exactly one check, and that
check compares implementations rather than reading prose: nothing under
`scripts/` or `crates/` changed between the two runs.

```text
$ pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
gate ok: 38 passed, but 1 SKIPPED on this host: check-twins
```

⚠ **And the full run is slower than it was**, measurably: every PowerShell check
that reads the corpus now spawns a `pwsh` to resolve the root, and `check-twins`
runs the gate's PowerShell half inside itself. `TOOL-07` is the entry that owns
the gate's cost and the next session that touches it should re-time it.


---

## Three entries closed, five effort points, and one worked and left open

⚠ **Under the twenty [`RULES.md`](RULES.md) section 10 asks for**, and the
reason is stated rather than hidden: the operator's first instruction was to
consolidate the documents against the tree, which is not an entry and scores
nothing, and it took the first third of the session. The operator then ended it.

| | |
| --- | --- |
| `TOOL-19` `M` | a check pair holding both catalogues, which refuses thirteen scripts on the tree as it stood this morning |
| `CORPUS-06` `M` | the headless normalisation wired where `DRIVER-03` said it belonged, four sessions after `CORPUS-01` landed without it |
| `PUB-12` `S` | the licence check reads the branch a consumer fetches, driven against a branch that disagrees |
| ⚠ `PUB-11` `M` | worked and **open**. One resolver, twelve check pairs wired, seven of ten passing with the corpus moved out of the working tree. Its entry names the three that do not and why. |

---

## ⭐ What changed about this project today

**The documents had stopped describing the tree.** Four sessions built the
schema, the harness, the corpus, the emitters, the library and both publishing
surfaces, moved [`INDEX.md`](INDEX.md) and the changelog with each of them, and
never re-read the reference pages those name. ⛔ The README told a reader this
repository could run its own checks and nothing else; the technical reference
said nothing was published and no digest was computed; the document set table
said there was no technical reference, two sections above the rule naming it.

⭐ **A check holds both catalogues now**, so the next session that adds a script
or a document without listing it fails the gate rather than the reading.

---

## ⛔ Findings in this session's own work, each caught by running it

| what | how it showed |
| --- | --- |
| ⛔ the headless normalisation had no caller at all | `DRIVER-03` built it, named the seam, and said so; `CORPUS-01` landed and nothing wired it. Every published profile and every published `user-agent` route carries `HeadlessChrome` |
| ⛔ a check written this session passed over its own missing entry | `check-catalogues` asked `git ls-files` alone, so the one file most likely to be unlisted, the one written that minute, was outside its scope |
| ⛔ a `[switch]` parameter and a lower-case local are ONE variable in PowerShell | a script-scope `$ref = Get-BranchRef` assigned a string to `[switch]$Ref` and threw before anything ran |
| ⛔ a `'*/index.json'` pattern matches nothing on Windows | `$_.FullName` is backslash-separated, so the twin counted the derived files as profiles and said 8 where the POSIX half said 6 |
| ⛔ `check-license-consistency` declined the one surface a consumer fetches | its header said the data branch did not exist, on the day after it was pushed |
| ⛔ two script headers described behaviour their own bodies no longer had | `check-data-branch` still said its comparison could not run, and `check-formats` still said nothing published a generated format |
| ⚠ the entry's own resolution order was wrong | `PUB-11` puts the data branch before the working tree; that reads the PUBLISHED corpus while a session is adding to the working one |
| ⚠ the shell ate a backticked payload inside a `node -e` string | a paragraph reached a document with three code spans emptied. The payload goes through `write-file.mjs` from a file now |
| ⚠ `check-one-home` fired twice on this session's own prose | once on a sentence quoted from another entry, once on a phrase copied from a script's comment. Both are pointers now |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The claim audit: which document sentence is not backed by the tree

Swept: every markdown file under [`../docs/`](../docs/), the README, the
scripts contract, the experiments index, the changelog's front matter, and the
header comment of every script the session touched. The method was to read each
absence claim and check it against the tree rather than against another
document.

⛔ **Finding: nine documents and two script headers were wrong**, every one of
them by having been true once. The full list, with the original wording, is
[`../docs/HISTORY/stale-documents.md`](../docs/HISTORY/stale-documents.md).
⭐ The worst was a document contradicting itself two sections apart.

**What the other passes did not look at:** the prose. Neither of the others
reads a sentence for whether it is true.

### 2. The dead-caller sweep: what is built, tested, documented and unreachable

Swept by grep rather than from memory: every `pub fn` in
`b-ids-driver`, every function the capture path could call, and every entry
closing that says a seam was left for later. The question was not "does this
work" but "does anything reach it".

⛔ **Finding: `b_ids_driver::headless::normalise` had no caller outside its own
module**, with five passing tests and a closing paragraph naming the seam it was
waiting for. That seam had existed for two sessions. `CORPUS-06`.

⚠ **What it did not find, and saying so is the point:** every other function
checked has a production caller, and the sweep cannot see a caller that exists
but is unreachable at run time. That is the driven pass's question.

**What the other passes did not look at:** reachability. The claim audit reads
documents and the guard mutation reads guards.

### 3. The guard mutation: can the new guards actually fail

⛔ **Every mutation of a tracked file was made against a copy under the ignored
scratch directory, the live file restored from that copy, and the restored file
compared against `HEAD` before anything else ran.**

| where | planted | red |
| --- | --- | --- |
| `CORPUS-06` | the `normalise` call deleted; the launch gate replaced with `if true`; the provenance `insert` made unreachable | all three. ⭐ The launch gate is the one no other case covers: a normalisation that fired on every capture would look correct on every profile in the corpus, because every one of them was taken headless |
| `TOOL-19` | the check pointed at this repository at `8f031a6`, where thirteen scripts were unlisted | both halves, same thirteen, same exit code |
| `PUB-12` | a local `data` branch built off `origin/data` with its manifest rewritten to `MIT`, then with its `LICENSE` replaced too | both halves, one problem and then two |
| `PUB-11` | `corpus/` and `raw/` moved out of the working tree entirely | seven of ten checks resolved off the branch and passed; the three that did not are named in the entry |

⚠ **Guards NOT mutated, and saying so is the point:** nothing exercised the
publishing workflow, because running it writes to the remote; the `for-testing`
capture lane has still never run; and no capture was taken this session, so
`CORPUS-06`'s fix is proved by its suite rather than by a browser.

---

## ⚠ What is in progress

⛔ **Nothing half-edited.** `PUB-11` is worked and open, and its entry carries
what landed, what was driven, and the three legs that still reach the corpus
through code resolving the workspace root.

---

## Open questions for the operator

### 1. ⭐ Where does a new capture go once `corpus/` leaves the default branch?

⚠ **The sequence has an unanswered step in it.** Publish the branch, verify it,
move the checks, remove `corpus/`: after that last step the data branch is the
canonical corpus rather than a derivation of it, and two things stop meaning
what they mean today.

- `check-data-branch` compares the branch against what the canonical corpus
  derives to. With no canonical corpus in the tree it would compare the branch
  against itself, so it refuses instead and exits 2.
- The capture workflow adds a profile by writing `corpus/v1/...` in the working
  tree and committing it. With the directory gone there is nowhere to write.

⭐ **Recommendation: the capture lane opens its pull request against a branch
that still carries the corpus, and the default branch keeps `corpus/` until a
capture path that writes to the data branch exists.** ⛔ That makes the removal
step the LAST one rather than the fourth, and it is a smaller change than making
a workflow push data.

### 2. ⚠ Two documents disagree about whether a session prints a prompt

[`RULES.md`](RULES.md) section 10 and [`../docs/AGENTS.md`](../docs/AGENTS.md)
section 6 both refuse one, in as many words.
[`../docs/methodology/work-todo.md`](../docs/methodology/work-todo.md) listed one
as owed at every session boundary. ⭐ **Amended to defer to this project's own
rule**, so the tree now has one answer.

⚠ **The operator asks for a kick-off prompt at the end of every session**, which
is the opposite of what the rule says. ⭐ **Recommendation: keep printing it and
soften the rule to "no prompt is written into the tree"**, which is the defect
the rule was actually written against: a second copy of the work order going
stale in a file. A prompt printed in chat and never committed cannot go stale.

### 3. ⚠ The corpus publishes a `HeadlessChrome` User-Agent and will keep doing so

⛔ **Six profiles and every `user-agent` route carry it**, and the corpus is
append-only, so `CORPUS-06`'s fix reaches only the next capture. ⭐
**Recommendation: run the capture lane once on each enabled cell**, which
produces new profiles at new versions carrying the normalised value, rather than
anything that edits what is published.

---

## ⭐ The work order

⚠ **Take these in order.**

1. **`PUB-11`**, which is worked and open. Its entry names three legs that reach
   the corpus through Rust resolving the workspace root; give them the resolved
   root and re-run the driven pass for ten of ten.
2. **`CORPUS-02`**, whose acceptance names four rows and refuses on two.
   ⛔ Both are blocked on one thing: `b_ids_driver::Family` knows two families.
   `firefox` is the higher value: a genuinely different TLS stack.
3. **`PUB-04`**, which `EMIT-01` unblocked: there is a support matrix to ask
   before a snippet is generated.
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
  it byte for byte; then move the checks that read the corpus; then remove it.
  ⚠ `PUB-11` is the middle step and open question 1 above is about the last one.
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
