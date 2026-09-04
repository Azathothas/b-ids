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
| the task | Rule the open questions, fix the Windows CI failure at its cause, and work the order. Session ran 2026-09-04, attended, started 2026-09-04T00:07:59Z. |
| the resume point | `DRIVER-11`, the launcher that speaks only Chromium. It is first in the work order and `CORPUS-02` is blocked on it. |
| in flight | Nothing half-edited. Six entries closed in place with their acceptance commands run; `CORPUS-02` and `EMIT-03` open with measured blockers and rulings attached. |
| the state of the tree | Clean and pushed on `main`. The gate is green over 40 checks. |
| the paste | below |

---

## The one thing to know before anything else

⭐ **There are no open questions.** Eight were put to the operator and ruled on
2026-09-04, and each is written into the entry it governs as well as into the
record's settled section. ⛔ Three of the next six units of work take a
third-party tree into `vendor/`, and
[`../methodology/vendoring.md`](../methodology/vendoring.md) is binding on each.

⚠ **The Windows CI failure was this repository's own.** The record had called it
a runner fault for three sessions. `rustc` and `cargo` are rustup proxies, so
the probe started installing the pinned toolchain and killed it at six seconds;
the component conflict the job reported was a fragment of that.
⭐ `sh scripts/doctor/doctor.sh --fixture` is the command that now holds the
rule, and `TODO/RULES.md` section 8.5 states it.

---

## The conditions this session leaves

⚠ **The data branch is BEHIND by 37 artefacts and nothing published is wrong.**
`PUB-04` added a `configs/` tree the publisher has not pushed yet.
`check-data-branch` reports that as a pending publish rather than a failure,
which is `PUB-14`, and the push at the end of this session is what triggers
`publish.yml` to close the gap.

⛔ **Two checks were passing by comparing something to itself**, and both are
fixed: `check-data-branch` compared the published branch against a materialised
copy of that same branch, and `check-corpus` asked this repository's history
about files that are not in it. ⚠ Both had the identical cause, an export on the
line above the guard.

⚠ **The corpus still publishes a `HeadlessChrome` User-Agent** on all six
profiles and every `user-agent` route. ⭐ The operator has ruled that a session
may dispatch `capture.yml` and merge the green lanes, which is the only route
that replaces it, because the corpus is append-only.

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
