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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, closing entries in place with their acceptance commands run. Session ran 2026-09-02T01:14:00Z to 2026-09-02T07:00:00Z, unattended, ended by operator interrupt. |
| the resume point | The work order's item 1, **`CORPUS-02`**, continued: the `edge` lane is enabled and wired and has not produced a profile. |
| in flight | ⛔ Nothing. Fourteen entries closed in place with their acceptance commands run; `CORPUS-02` left open with its blocker named. |
| the state of the tree | Clean and pushed on `main`. The gate passes 26 checks with `check-twins` alongside, and all 26 pairs agree. |
| the paste | below |

---

## ⭐ What changed about this project on 2026-09-02

**The capture matrix stopped being apparatus and started producing data.**
`capture.yml` ran on hosted runners four times, every job green, and the corpus
went from one profile to three. Two of the three were captured on machines
nobody owns.

⛔ **Two findings came before the first runner profile did.** The two runner
images do not serve the same Chrome build, so one build on two platforms needs
pinned acquisition rather than the preinstalled browser. And the Linux lane
captured nothing twice, because Chrome abandoned the connections that were not
resumed: the harness refuses session tickets now, and a control says that
changes which connections are cold rather than what a cold hello is.

⭐ **An inherited claim fell to an experiment, which had not happened here
before.** "Chrome on Linux does not read the user's NSS database for server
authentication" is refuted: a root added there let the browser complete two
handshakes on a runner that was then thrown away.

⭐ **And the pin costs nothing, on one platform.** 19 TLS fields compared
between a per-launch key pin and a real trust anchor, 0 differing. ⚠ One
platform, one build, one day.

---

## The conditions this session leaves

⚠ **The corpus holds three profiles, all Chrome 151.** Not a matrix.
`CORPUS-02` is the entry, and two of its four required rows are captured.

⚠ **Every profile written from here records `captured.resumption: refused`.**
The harness issues no session tickets during a corpus capture, because without
that the Linux lane cannot produce a cold handshake at all.

⚠ **Nothing is published outside this repository.** The canonical corpus is on
the default branch deliberately. The data branch, the generated formats and the
fetchable routes are `PUB-02`, `SCHEMA-08` and `PUB-03`.

⚠ **Windows cannot exercise the trust-store route**, and nobody has read why.
`50-trust-anchor.sh` exits 2 there rather than reporting a one-sided comparison.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt.** The router names what to read, in what order,
and what a session owes at each end. Everything else is reached from it.
