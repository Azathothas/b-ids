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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s order, largest first. Session ran 2026-09-03, unattended, and the operator ended it. |
| the resume point | [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, which starts at `PUB-11`. ⛔ It is gated on the data branch existing and being verified. |
| in flight | ⛔ Nothing. Eight entries closed in place with their acceptance commands run. |
| the state of the tree | Clean and pushed on `main`. The gate is green in full, `check-twins` included. |
| the paste | below |

---

## The one thing to know before anything else

⭐ **The `data` branch exists.** The push that landed `PUB-10` triggered the
workflow, its data-branch job created the branch, and the tree it carries was
compared object for object with a local build of this corpus. Every later push
to the default branch APPENDS to it and never force-pushes.

⛔ **So the next step is `PUB-11`**, which moves the eleven check pairs that
read the corpus from the working tree. Only after that does `corpus/` leave
`main`, and that step is the operator's.

⚠ **No release has been cut**, because no tag has been pushed. The release job
of the same workflow skipped, which is what it should do.

---

## The conditions this session leaves

**The corpus is usable rather than only accurate.** `b-ids` hands a program a
profile with the corpus embedded and no network in it; `b-ids-cli` puts one back
on a wire and the harness reads back the same profile, field by field, with 1951
of 1983 bytes identical to the browser's own.

⚠ **Two acceptance commands were refuted by their own entries** and both are
corrected in place: `EMIT-02`'s asked for a byte comparison the model cannot
make, because it does not record the `ClientHello` random; `LIB-02`'s named a
profile the corpus does not hold, and the client refuses one by name.

⚠ **`captured.operator` is still typed**, and **`Shuffle::Observed` is still
never written**. Both are fields the capture path does not fill.

⚠ **Three of the JA4 specification's ALPN examples cannot be represented**,
because the model holds a protocol as a string and theirs are not UTF-8.
Recorded rather than repaired.

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
