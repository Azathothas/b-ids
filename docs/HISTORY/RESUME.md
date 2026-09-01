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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order from item 1, closing entries in place with their acceptance commands run. Session started 2026-09-01T07:58:00Z, unattended, quota twenty effort points. |
| the resume point | `DRIVER-02`, then `SCHEMA-08`, `HARNESS-09`, and the small entries the work order lists. `CORPUS-01` and `HARNESS-10` are closed. |
| in flight | Nothing is half-edited. Both closed entries carry their acceptance output and the record moved in the same change. |
| the state of the tree | ⛔ Dirty and uncommitted on `main` at `bd02224`. The new crate, the first profile, two experiments, `check-corpus` and every record edit are in the working tree and not committed. The gate was green at 20 passed before the last few edits. |
| the paste | below |

---

## What changed about this project in this session

**The corpus is not empty.** One measured profile, Chrome `151.0.7922.76` on
Windows, at `corpus/v1/chrome/stable/win64/151.0.7922.76.json`, with the
`ClientHello` it was read from under `raw/v1/`. `b-ids-corpus` wrote it and
`scripts/common/check-corpus` is the pair that keeps it honest.

**Terminating the handshake changes nothing the raw surface can see.** Measured
over three rounds of one browser: seventeen of nineteen TLS fields agree, none
differ, and the two that cannot be compared carry a per-connection draw.

⚠ **Two conditions still stand.** Every capture goes through a per-launch key
pin rather than a trust store, which is recorded per profile in
`captured.trust` and is still unmeasured against a real trust anchor. And the
operator ruled at the start of the session that a measured profile goes into the
committed corpus with its conditions recorded;
[`../../TODO/RULES.md`](../../TODO/RULES.md) carries that ruling.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt.** The router names what to read, in what order,
and what a session owes at each end. Everything else is reached from it.
