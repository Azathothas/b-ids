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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, closing entries in place with their acceptance commands run. Session ran 2026-09-01T12:32:05Z to 2026-09-01T15:20:00Z, unattended, ended by operator interrupt. |
| the resume point | The work order's item 1, **`CORPUS-02`**. Run `capture.yml` on the default branch, take the `linux64` artefact, add it with `b-ids-corpus add`, and close the entry. |
| in flight | ⛔ Nothing. Seven entries closed in place with their acceptance commands run; `CORPUS-02` left open with its blocker named and what landed recorded under it. |
| the state of the tree | Clean and pushed on `main`. The gate passes 25 checks with `check-twins` alongside, both halves of all 22 pairs. |
| the paste | below |

---

## ⭐ What changed about this project on 2026-09-01, second session

**Continuous integration stopped being the last item and became the first.** The
operator ruled that this project is GitHub CI, 100%: a corpus whose captures
depend on one person's machine cannot cover the matrix, cannot be reproduced and
cannot be scheduled, so captures belong on runners.

⭐ **A push now settles what is published.** `validate.yml` runs on two hosts with
the network off for every assertion and the whole history fetched;
`b-ids-corpus validate` runs the coherence checks over what is PUBLISHED rather
than over whatever a caller listed; and the generator is asserted to answer the
same way twice.

⭐ **The capture matrix exists and has never run.** `capture.yml` fans out from
`.github/capture-matrix.json`, every lane failing alone, and `check-coverage`
reads the same plan to say which cells have a profile. ⛔ No lane has run on a
runner: that is `CORPUS-02` and it is the next thing.

⛔ **Four checks were reporting green over questions they had not asked**, and
each is fixed: `check-corpus`'s history leg under a shallow clone, the gate's
line-endings filter reading the index column alone, `check-twins` unable to tell
a drift from a tree that moved, and `mine-repo` exiting before its clone.

---

## The conditions this session leaves

⚠ **The corpus still holds one profile**, taken on a laptop. It is one source,
not two, so `VALID-01`'s handshake check still reports `NotCheckable` and
`CI-04`'s automated-merge condition is still unsatisfiable.

⚠ **Every capture so far went through a per-launch key pin** rather than a real
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
