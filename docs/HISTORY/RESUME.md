# RESUME.md

⚠ **A dead man's switch, not a history.** Overwritten at the start of every
session and refreshed whenever what is in flight changes, so a session that ends
badly still hands something over. ⛔ It is not the record and it carries no work
order: [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md) has both.

⚠ **It is the one file in this directory that is read to do work**, and it lives
here rather than at the repository root because it is a record of a session
rather than a document about the project.

---

| | |
| --- | --- |
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, closing entries in place with their acceptance commands run. Session ran 2026-09-01T03:47:48Z to its operator interrupt, unattended, seven entries closed. |
| the resume point | [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, item 1: **`CORPUS-01`**, which turns a capture into a profile. ⛔ It is the single thing in the way: the harness takes captures from real browsers and nothing writes one down. |
| in flight | ⛔ Nothing. Every entry touched is closed with its acceptance command run, the record moved in the same change, and the tree committed. |
| the state of the tree | Clean and pushed on `main`. The gate passes 20 checks locally, `check-twins` agrees on every pair, and the remote checks are green. ⛔ The corpus is empty: captures exist, no profile does. |
| the paste | below |

---

## ⭐ What changed about this project on 2026-09-01

**It can reach a browser now.** rustls is vendored at `v/0.23.43` under
[`../../vendor/`](../../vendor/), `--ca-out` mints an authority and terminates
the handshake behind it, and Chrome `151.0.7922.76` and Edge `152.0.4191.53`
both completed verified handshakes against it.

⚠ **The one measurement taken is in
[`../inherited-claims.md`](../inherited-claims.md) section 5**, marked
`measured-here` with its conditions, and it is not published because there is
nowhere to publish it.

⛔ **Every capture was taken through a per-launch key pin rather than a trust
store**, which is a condition of the measurement rather than a detail.
`HARNESS-10` is the entry that measures whether it changed the answer.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt.** The router names what to read, in what order,
and what a session owes at each end. Everything else is reached from it.
