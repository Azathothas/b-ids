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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s order to the end. Session ran 2026-09-04, attended, started 2026-09-04T06:04:28Z. |
| the resume point | `PUB-13`, the source branch, all six steps. It is first because every capture from now on adds to a default branch the ruling says should not carry data. |
| in flight | Nothing half-edited. `DRIVER-11` and `DOC-03` are closed in place with their acceptance commands run; `CORPUS-02` is worked and open with its blocker measured. |
| the state of the tree | Clean and pushed on `main`. The gate is green over 40 checks and 445 tests. |
| the paste | below |

---

## The one thing to know before anything else

⭐ **The capture matrix works now, and it never had.** Every lane captured,
printed the command that would write the result, and uploaded the checkout
unchanged; nothing ever ran `b-ids-corpus add`. The lane does it now, and the
corpus went from six profiles to twelve in one day.

⛔ **The operator overruled a ruling mid-session and it is settled:** do not
vendor a niche third-party tree. `DRIVER-11`'s certificate-database writer is
Rust in this tree, under
[`../../crates/b-ids-driver/src/nssdb/`](../../crates/b-ids-driver/src/nssdb/),
and `mozilla/nss` is mined into
[`../../references/mozilla__nss/`](../../references/mozilla__nss/) as the
reference every constant is cited against.

⚠ **A local gate pass proves less than it looks**, and this session was reminded
twice: `check-twins` found a real drift in a file that had already passed
`shellcheck`, `PSScriptAnalyzer` and its own tests, and an exit code was read
through a pipe twice.

---

## The conditions this session leaves

⚠ **The data branch is BEHIND by 168 artefacts**, which is the designed pending
state rather than a failure. The publisher adds them on the next push to `main`.

⛔ **Four checks that were green would have refused every capture after the
first**, and all four are fixed. The publish manifest is `corpus-publish/2` and
records `derived` per artefact, which is what lets a check tell a branch that is
behind from one that was rewritten.

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
