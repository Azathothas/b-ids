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
| the task | ⛔ **The FINAL session**, ruled by the operator 2026-09-04 at its start. Close every open entry, leave the tree clean and green and the documents lean. Started 2026-09-04T11:57:16Z, attended. |
| the resume point | `PUB-13`, the source branch, all six steps. The operator ruled all six in scope including the removal from the default branch. |
| in flight | Nothing half-edited. The session has read the router, the record and the six entries in the work order, and has run the probe and the baseline gate. |
| the state of the tree | Clean on `main` at `e7e521e`, pushed. The gate re-measured on this Windows host at the start: **39 passed, 1 skipped** (`check-twins`, which `--fast` skips). |
| the paste | below |

---

## The four rulings this session opened with

⛔ **All four were asked before any work and answered by the operator.**

1. **Scope: all ten open entries.** The six in the work order and the four
   build-outs. The session carries twice the usual budget.
2. **`PUB-13` runs all six steps**, including removing `corpus/` and `raw/` from
   the default branch.
3. ⛔ **No tool is credited in the commit.** The harness asked for a
   `Co-Authored-By` trailer naming a model; absolute 5 in
   [`../AGENTS.md`](../AGENTS.md) forbids it and the operator ruled the
   repository's rule wins. ⚠ It is worth stating that this contradiction exists,
   because the harness default will ask again.
4. **All four open questions are this session's**, and ⭐ the operator
   authorised `gh` for the fourth, which is a setting on this repository's own
   remote.

---

## The one thing to know before anything else

⭐ **The capture matrix works now, and it never had.** Every lane captured,
printed the command that would write the result, and uploaded the checkout
unchanged; nothing ever ran `b-ids-corpus add`. The lane does it now, and the
corpus went from six profiles to twelve on 2026-09-04.

⚠ **A local gate pass proves less than it looks.** `check-twins` has found a
real drift in a file that had already passed `shellcheck`, `PSScriptAnalyzer`
and its own tests, and an exit code has been read through a pipe twice in this
tree. ⛔ Read every exit code from the process that produced it.

⚠ **The `chromium` cell is disabled with a MEASURED reason.** The resolver finds
`/usr/bin/chromium` on `ubuntu-24.04`; the launch aborts in the snap sandbox.
⛔ Do not answer it with `--no-sandbox`.

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
