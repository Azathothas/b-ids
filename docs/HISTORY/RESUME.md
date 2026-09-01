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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order from item 1, closing entries in place with their acceptance commands run. Session ran 2026-09-01T07:58:00Z to 2026-09-01T10:43:02Z, unattended, ended by operator interrupt with six entries closed. |
| the resume point | The work order's item 1, **`CORPUS-02`**, the capture matrix. One profile on one platform is what everything below it is now waiting on. |
| in flight | ⛔ Nothing. Every entry touched is closed with its acceptance command run, the record moved in the same change, and the tree committed. |
| the state of the tree | Clean and pushed on `main`. The gate passes 21 checks with `check-twins` alongside, both halves of every pair. |
| the paste | below |

---

## What changed about this project on 2026-09-01

⭐ **The corpus is not empty.** One measured profile, Chrome `151.0.7922.76` on
Windows, at `corpus/v1/chrome/stable/win64/151.0.7922.76.json`, with the
`ClientHello` it was read from beside it. `b-ids-corpus` wrote it and
`check-corpus` asks git whether anything published was ever edited.

⭐ **Three measurements, each with its conditions.** Terminating the handshake
changes nothing the raw surface can see; the inherited version-discovery defect
is real to the digit; and no parser panics in a million runs.

⚠ **Two conditions still stand.** Every capture goes through a per-launch key
pin rather than a trust store, recorded per profile in `captured.trust` and
still unmeasured against a real trust anchor. And the corpus describes a build a
major behind what stable is serving, which `DRIVER-02` is what revealed.

⚠ **Nothing is published.** The canonical corpus is on the default branch
deliberately, so a capture is reviewable as a diff. The data branch, the
generated formats and the fetchable routes are `PUB-02`, `SCHEMA-08` and
`PUB-03`, and all three are open.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt.** The router names what to read, in what order,
and what a session owes at each end. Everything else is reached from it.
