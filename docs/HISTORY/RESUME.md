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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, closing entries in place with their acceptance commands run. Session began 2026-09-02T01:14:00Z, unattended. |
| the resume point | `CORPUS-02`. Two runner captures exist; the `linux64` one had no cold connection and a second run is out. |
| in flight | The `win64` runner profile is added and uncommitted. Two driver defects are being fixed under `CORPUS-02`. |
| the state of the tree | ⚠ Dirty: `corpus/v1/chrome/stable/win64/151.0.7922.174.json` and its hello are new, `index.json` and `latest.json` rewritten. The gate was green at the session's start, 24 passed with `check-twins` skipped by `-Fast`. |
| the paste | below |

---

## ⭐ What this session has established so far

⭐ **`capture.yml` has run on hosted runners, and it works.** Run 33579619515,
dispatched with the authenticated `gh` on the default branch, completed with all
five jobs green: the plan job, both browser lanes, the fuzz lane and the collect
job. That is the first time the capture matrix has ever run.

⛔ **The two runners do not carry the same Chrome build.** `ubuntu-latest`
served `151.0.7922.173` and `windows-latest` served `151.0.7922.174`, so one
build on two platforms is not obtainable from the preinstalled browser and needs
pinned acquisition instead.

⛔ **`b-ids-corpus add` prints `1 cold` as a literal**, so its report claims a
cold connection on a run that had none. It is beside a refusal that says the
opposite in the same output.

⛔ **The `linux64` capture had no cold connection at all.** Both of its first
two connections were abandoned after the handshake and every later one resumed,
so nothing was publishable from it. The `win64` capture had one and is added.

⛔ **The matrix's `browser` column reaches nothing.** `b-ids-driver drive`
takes the first resolved family and has no switch for choosing one, so an `edge`
lane would drive Chrome. ⚠ The driver DID resolve Edge on the runner at
`/usr/bin/microsoft-edge`, which is the blocker that cell records.

---

## The conditions this session leaves

⚠ **Every capture still goes through a per-launch key pin** rather than a real
trust anchor, recorded per profile in `captured.trust`. `HARNESS-14` is the job
that measures whether it mattered, and `DRIVER-04` lands first.

⚠ **Nothing is published.** The canonical corpus is on the default branch
deliberately. The data branch, the generated formats and the fetchable routes
are `PUB-02`, `SCHEMA-08` and `PUB-03`.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt.** The router names what to read, in what order,
and what a session owes at each end. Everything else is reached from it.
