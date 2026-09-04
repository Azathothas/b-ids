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
| the resume point | `CORPUS-02`, which the Gecko lane and the enabled `firefox/stable/linux64` cell now make dispatchable, then `PUB-13`. |
| in flight | Nothing half-edited. `DRIVER-11` is closed in place with both acceptance commands run. |
| the state of the tree | `check-gate.ps1 -Fast`: 39 passed, 1 skipped (`check-twins`). 445 tests. |
| the paste | below |

---

## The one thing to know before anything else

⭐ **The corpus has a non-Chromium profile and it was measured here.** Firefox
154.0.1 completed a TLS 1.3 handshake against this project's own terminator on
this Windows host, and `corpus/v1/firefox/stable/win64/154.0.1.json` is the
capture. `captured.trust` reads `trust-store`.

⛔ **The operator overruled two things during this session**, and both are in
`DRIVER-11`'s closure:

- ⛔ **Do not vendor a niche third-party tree.** The nssdb writer is written in
  Rust in this tree, under `crates/b-ids-driver/src/nssdb/`. The vendoring was
  backed out before it was committed.
- ⭐ **`mozilla/nss` is the reference**, mined to
  [`../../references/mozilla__nss/`](../../references/mozilla__nss/) at commit
  `7db8de42431841b214b49fd2cb7122a07aa631b8` and trimmed by deletion.

⚠ **A local gate pass proves less than it looks.** The previous session pushed
green from this same Windows host and both runners refused it. ⛔ Read the CI
result before calling anything done.

---

## The conditions this session leaves

⚠ **The data branch is BEHIND by 44 artefacts**, which is the designed pending
state rather than a failure. The publisher adds them on the next push to `main`.

⛔ **The publish manifest is `corpus-publish/2`.** It records `derived` per
artefact, and `check-data-branch` reads it: without that, adding any profile
changed nineteen aggregates and read as a rewritten branch, so no capture could
ever have been published again.

⚠ **Firefox updated itself under this session**, 148.0.2 to 154.0.1 within an
hour, and the launcher process that exits while the browser starts is what a
capture taken during that update looks like.

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
