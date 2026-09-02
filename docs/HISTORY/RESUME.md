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
| the task | Read the artefacts of capture run 33615327503, then work `DRIVER-08` and specifically the leg that has never run, then `TOOL-18`, `HARNESS-15`, `DRIVER-10` and `CORPUS-02`. Session started 2026-09-02T11:22:03Z, unattended. |
| the resume point | `DRIVER-08`'s six remaining items, in the entry's own order. The purge and the install have never executed on a runner. |
| in flight | Nothing yet. The three artefacts are read and downloaded to `.tmp/run-33615327503/`. |
| the state of the tree | Clean on `main`. The gate passes 26 checks with `check-twins` skipped by `-Fast`, measured at 11:20Z on this Windows host. |
| the paste | below |

---

## ⛔ What the scheduled run's three artefacts say

**Run 33615327503 fired on cron at 09:39Z against `0cee89e` and nobody
dispatched it.** All three artefacts are read.

⛔ **No lane wrote a profile, and no lane was ever going to.**
`experiments/10-first-profile.sh` ends by PRINTING the `b-ids-corpus add`
command rather than running it. Its own header says step 3 "selects the cold
connection out of the navigation and writes it into the corpus" and that exit 0
means "a profile was written". Neither is true of the script under it. The
corpus tree inside all three artefacts is byte-identical to the committed one.

⭐ **The `linux64` lane captured, which is the lane that captured nothing
twice.** Eight connections, seven of eight handshakes completed, every one cold
because the harness refuses session tickets. The `win64` lane captured six.

⚠ **Both lanes captured a build the corpus already holds**, `151.0.7922.173` on
`linux64` and `151.0.7922.174` on `win64`, so neither artefact carries a profile
the corpus is missing.

⛔ **The `edge` lane's failure has a cause now, and it is not the one the
record named.** `browser.log` carries it, which is what `DRIVER-07` was built
for:

```text
FATAL:sandbox/linux/suid/client/setuid_sandbox_host.cc:166] The SUID sandbox
helper binary was found, but is not configured correctly. Rather than run
without sandboxing I'm aborting now. You need to make sure that
/opt/microsoft/msedge/msedge-sandbox is owned by root and has mode 4755.
```

The record said Edge "exits after 1.4 seconds having opened no connection",
which is the symptom. The cause is the `ubuntu-latest` image's Edge package,
and Chrome on the same runner in the same run is unaffected.

---

## The conditions this session inherits

⚠ **The corpus holds three profiles, all Chrome 151.** Not a matrix.
`CORPUS-02` is the entry, and two of its four required rows are captured.

⚠ **Every profile written from here records `captured.resumption: refused`.**
The harness issues no session tickets during a corpus capture, because without
that the Linux lane cannot produce a cold handshake at all. `HARNESS-15` is the
entry that removes the condition rather than the switch.

⚠ **Nothing is published outside this repository.** The canonical corpus is on
the default branch deliberately. The data branch, the generated formats and the
fetchable routes are `PUB-02`, `SCHEMA-08` and `PUB-03`.

⛔ **A tool that purges browsers lives in `scripts/common/` and its success
path has never run.** It refuses any machine that is not both marked disposable
by this project and running on a hosted runner. ⛔ Run it with `--plan` and
nothing else on a machine you keep, and do not lift a condition to see what
happens.

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
