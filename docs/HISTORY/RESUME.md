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
| the task | Read the artefacts of capture run 33615327503, then `DRIVER-08`, `TOOL-18`, `HARNESS-15`, `DRIVER-10` and `CORPUS-02`. Session started 2026-09-02T11:22:03Z, unattended. |
| the resume point | `HARNESS-16`, whose measurement is dispatched as `trust-anchor.yml` run 33647065058 and unread. Then the work order in [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md). |
| in flight | ⚠ One thing: `trust-anchor.yml` run 33647065058, dispatched to answer `HARNESS-16`. Everything else is closed in place or left open with its blocker named. |
| the state of the tree | Clean and pushed on `main`. The gate passes 29 checks with `check-twins` skipped by `-Fast`, measured on this Windows host. |
| the paste | below |

---

## ⭐ What changed about this project today

**The corpus stopped recording builds nobody chose.** Six profiles now, two
browsers, two majors, two platforms, and three of them name the URL and the
digest of the artefact the browser was installed from. Before today every
profile carried `captured.acquisition: null`.

⭐ **The purge and the install ran on hosted runners**, both platforms and both
routes, which is the leg that had never executed anywhere.

⭐ **A browser that is not Chrome is in the corpus.** Edge `151.0.4129.101`,
provisioned from the vendor's enterprise index, which publishes a SHA-256 per
artefact that the tool checks what arrived against. It could not capture at all
before: its SUID sandbox helper was not configured on the runner image, and the
browser's own log said so.

⭐ **A cold hello is no longer thrown away because its own connection carried no
HTTP/2.** The two halves are selected independently now, and the Edge lane is
the proof: its only cold hello arrived on a connection that reached no HTTP/2,
so under the old rule that navigation published nothing.

⭐ **The gate costs 213 seconds where it cost about 600.** Three causes, and the
largest was not a per-file loop: a command substitution in a `while read`
assignment prefix is re-evaluated once per line read.

---

## The conditions this session leaves

⚠ **Two matrix cells are blocked on a question the operator has**, and it is the
only open question: an unbranded build and a branded build of one version
publish at one path, because the corpus route carries no `branded` and `Channel`
is a closed vocabulary. [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)
carries it with a recommendation.

⚠ **`captured.operator` is still typed.** The identity writer leaves it empty
and it is filled in by hand for every runner profile, this session's two
included.

⚠ **`Shuffle::Observed` is still never written.** Nothing in the capture path
produces a shuffle observation, so a field the model carries is one the capture
path never fills.

⛔ **A tool that purges browsers lives in `scripts/common/` and its success path
is measured now.** It still refuses any machine that is not both marked
disposable by this project and running on a hosted runner. ⛔ Run it with
`--plan` and nothing else on a machine you keep, and do not lift a condition to
see what happens.

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
