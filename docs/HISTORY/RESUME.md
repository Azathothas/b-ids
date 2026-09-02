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
| the task | Work [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md)'s work order, closing entries in place with their acceptance commands run. Session ran 2026-09-02T01:14:00Z to 2026-09-02T11:30:00Z, unattended until 07:00Z and then directed by the operator. |
| the resume point | The work order's item 1, **`DRIVER-08`**, and specifically its step 4: ⛔ **the purge and the install have never run on a runner.** The tool and its seven refusals exist; the success path is unmeasured. |
| in flight | ⛔ Nothing. Fourteen entries closed in place with their acceptance commands run; `DRIVER-08` left open with six named items; `DRIVER-09` and `DRIVER-10` written and untouched. |
| the state of the tree | Clean and pushed on `main`. The gate passes 26 checks with `check-twins` alongside, and all 27 pairs agree. `check-provisioning` is the 27th script and sits outside the gate on purpose. |
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

⛔ **And a browser-purging tool was run on the operator machine with its guard
disabled, on purpose, by this session.** Nothing was removed, and that was an
accident of registry matching rather than a safety margin.
[`README.md`](README.md) carries the incident; the guard is two independent
conditions now, and a test that has to bypass a guard runs against a copy.

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

⚠ **A tool that purges browsers lives in `scripts/common/` now.** It refuses
any machine that is not both marked disposable by this project and running on
a hosted runner. ⛔ Run it with `--plan` and nothing else on a machine you
keep, and do not disable a condition to see what happens.

---

```text
Read ./docs/AGENTS.md in full & follow.
```

⭐ **That is the whole prompt, and it stays the minimum.** The router names
what to read, in what order, and what a session owes at each end. Everything
else is reached from it.

⚠ **An operator who wants a session steered adds to it rather than replacing
it**, which is what happened on 2026-09-02: the same line, followed by the
unattended terms, the point to start from, and the standing warnings about the
append-only corpus and history. ⛔ Those additions are instructions for one
session and they do not belong in this file, which is overwritten by the next.
