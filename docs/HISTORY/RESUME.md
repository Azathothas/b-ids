# RESUME.md

⚠ **A dead man's switch, not a history.** Overwritten at the start of every
session and refreshed whenever what is in flight changes, so a session that ends
badly still hands something over. ⛔ It is not the record and it carries no work
order: [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md) has both.

It is the one file in this directory that is read to do work, and it lives here
rather than at the repository root because it is a record of a session rather
than a document about the project.

---

| | |
| --- | --- |
| the task | ⛔ **The FINAL session**, ruled by the operator 2026-09-04. Close every open entry, leave the tree clean and green and the documents lean. Ran 2026-09-04T11:57:16Z to 19:32:53Z, attended. |
| the resume point | ⛔ **There is none, and that is the state rather than an omission.** 107 of 107 entries are done and no work order exists. A session that opens this file is starting something new, and it AUTHORS from [`../../TODO/ENTRY.md`](../../TODO/ENTRY.md) rather than picking up. |
| in flight | Nothing. No half-edit, no throwaway branch, no scratch file outside `.tmp/`. |
| the state of the tree | Clean on `main`, pushed. The gate is green over **44 checks**, 43 passed and one skipped, with **463 tests**. |
| the paste | below |

---

## The three things to know before anything else

⭐ **1. The corpus is not on the default branch.** `corpus/`, `raw/`, `vectors/`
and `LICENSE` are on `source`; `data` is what the assembler derives from it; the
default branch carries the code and no corpus at all. ⛔ Every reader resolves
the root rather than assuming it:

```bash
sh scripts/common/corpus-root.sh --source
```

⚠ **A bare `cargo test` in a fresh checkout will fail**, because
`crates/b-ids/build.rs` embeds the corpus and finds none. The gate exports
`B_IDS_CORPUS_ROOT` around its three cargo steps; by hand it is:

```bash
B_IDS_CORPUS_ROOT=$(sh scripts/common/corpus-root.sh) cargo test --workspace
```

⭐ **2. Every entry is closed.** There is no work order and
[`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md) says so; what it carries
instead is five candidates this session measured and did not take, each with the
reason it is not an entry yet. ⛔ The largest is the TCP half, and its blocker is
a machine rather than code: [`../HUMAN.md`](../HUMAN.md) section 3.

⛔ **3. Three things are the operator's own act.** A pushed tag, which is the
only thing that cuts a release; the history reset on `main`; and the decision to
host the capture oracle, whose mode is built and whose retention answer is
already written into [`../../SECURITY.md`](../../SECURITY.md).

---

## What this session left that is worth not rediscovering

⚠ **A guard can report green over the exact defect it exists to catch.**
`check-signing` did, because it matched its own explanatory COMMENT rather than
the declaration it meant to read. ⛔ Every check in this tree that greps a file
it also documents has that shape available to it.

⚠ **`jq` on Windows writes CRLF**, and this project has now been bitten four
times. ⛔ The nastiest shape is a `for` over a command substitution of a
multi-line `jq` read: the substitution strips only the LAST line ending, so the
last element is clean and every one before it carries a `\r`. That was correct
for as long as the list had one element.

⚠ **A synthesised artefact must say so in a field a tool displays.** The
published captures are pcapng rather than pcap for exactly that reason, and the
values that were not measured are visibly impossible rather than plausible.

⛔ **A tool that purges browsers lives in `scripts/common/`.** It refuses any
machine that is not both marked disposable by this project and running on a
hosted runner. ⛔ Run it with `--plan` and nothing else on a machine you keep.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt, and it stays the minimum.** The router names
what to read, in what order, and what a session owes at each end. Everything
else is reached from it.

⚠ **An operator who wants a session steered adds to it rather than replacing
it.** ⛔ Those additions are instructions for one session and they do not belong
in this file, which is overwritten by the next.
