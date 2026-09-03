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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s order, largest first. Session ran 2026-09-02T23:21:15Z, unattended, and the operator ended it. |
| the resume point | [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, which starts at `DRIVER-06`. |
| in flight | ⛔ Nothing. Six entries closed in place with their acceptance commands run; `DRIVER-06` left open with what landed written into it. |
| the state of the tree | Clean and pushed on `main`. The gate is green in full, `check-twins` included. |
| the paste | below |

---

## ⭐ What changed about this project today

**Everything this project publishes now comes out of one assembler.** A release
archive and a data branch built by two assemblers is two answers to what the
project publishes, and the day they differ nobody finds out from either.

⭐ **Nine published formats, four of them lossless, two declined with their
reasons published.** A consumer asking for CBOR finds out it was weighed and
lost rather than that nobody thought of it.

⭐ **Fifty-four flat routes a program reads with `curl`**, each verified against
the corpus by a check that reads the profiles rather than the generator.

⭐ **The one open question is answered**: `for-testing` is a `Channel`, which
unblocks the two unbranded matrix cells and `DRIVER-06`.

---

## The conditions this session leaves

⚠ **`DRIVER-06` is open and its acceptance selects no test.**
`cargo test -p b-ids-driver branded` matches nothing, so it exits 0 having run
nothing. The enforcement half already exists in the validator; the driver-side
pair is what the entry owes.

⚠ **Nothing publishes.** `PUB-01` and `PUB-02` assemble and check; no workflow
cuts a tag, uploads an asset or creates a branch, and adding that trigger is the
operator's.

⚠ **`CI-04`'s collect job needs a repository setting** this repository's files
cannot grant: Actions must be allowed to create pull requests. The step reports
the refusal rather than failing silently.

⚠ **`captured.operator` is still typed**, and **`Shuffle::Observed` is still
never written**. Both are fields the capture path does not fill.

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
