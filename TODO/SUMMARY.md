# SUMMARY.md

⛔ **The last session's table, and a snapshot rather than an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and is what a session reads first.
Overwritten every session.

---

## 2026-09-01, second session

| row | measured |
| --- | --- |
| Elapsed | 2026-09-01T07:58:00Z to 2026-09-01T10:43:02Z, **2h 45m**, ended by operator interrupt |
| Commits | **3**, all pushed and all green on both CI jobs: `0df1fd7` mid-session, `af11973` at the close, `2a15aa0` for two corrections the close itself found. ⚠ Three, not one. See the note below. |
| Work | **6 completed, 0 deferred, 0 failed.** `CORPUS-01` (M), `HARNESS-10` (S), `DRIVER-02` (M), `HARNESS-09` (M), `TOOL-06` (S), `CORPUS-03` (S) = **9 effort points** |
| Changes | 70 files changed, 7,684 insertions, 332 deletions, plus 3 new files at the close |
| Size | 43,389 lines to **50,740**, excluding the reference corpus and the vendored tree. +7,351 |
| Checks | started at 19 passed 1 skipped of 20; ends at **21 passed 1 skipped of 22**, and `check-twins` green over all 17 pairs. Two new checks, `check-corpus` and `check-routes`, both halves, both compared |
| Cost | one image pulled and removed (918 MB), one container run, ~300s of processor on the fuzz run. No money. |
| Health | ⭐ the corpus holds its first measured profile. 2 new crates-worth of surface, 0 debts introduced, 0 entries left partial, tree clean. ⛔ Nothing deployed; nothing is published from this repository yet. |

---

## What moved, in one line each

| | |
| --- | --- |
| ⭐ `CORPUS-01` | the corpus is not empty: Chrome `151.0.7922.76`, measured here, with the `ClientHello` it was read from beside it |
| ⭐ `HARNESS-10` | terminating the handshake changes nothing the raw surface can see: 17 of 19 TLS fields agree, none differ |
| ⭐ `DRIVER-02` | the inherited version-discovery defect reproduced here to the digit, and it shows the corpus is a major behind stable |
| `HARNESS-09` | one million coverage-guided runs at the parsers, no crash; 6,767 mutations run on every host in 0.47s |
| `TOOL-06` | no published single-value file ends with a newline, in both halves and in the gate |
| `CORPUS-03` | `latest` means stable, and it cannot mean anything else because of how it is built |

---

## ⚠ The three things a reader should not have to find out later

⛔ **There are three commits, not one.** The first was pushed mid-session as a
crash checkpoint, because four entries of uncommitted work is more than a
session should risk; the third fixed two defects the close itself surfaced.
Squashing them now would rewrite published history and need a force push, which
[`../docs/conventions/git.md`](../docs/conventions/git.md) section 5 and
[`../docs/security/remote-ops.md`](../docs/security/remote-ops.md) both refuse
without the operator's explicit instruction for that specific commit.

⚠ **Three claims in this session's own writing were wrong and were corrected.**
A corpus size quoted from an assertion's floor rather than measured (6,767, not
"over five thousand"); a pasted suite count that moved after it was pasted; and
a baseline line in `PROGRESS.md` itself. All three were found by the claim
audit, and the last one was in the file the claim audit is written into.

⛔ **`check-corpus` refused its own derived index**, on the first commit after
the corpus had a profile in it. `index.json` and `latest.json` are regenerated
whenever a profile is added, so the immutability rule could never have applied to
them; it belongs to a published profile and its raw sidecar. Found by the check
running, not by reading it, and mutation-proved in three directions afterwards.

⛔ **`check-twins` reported a drift that was not one.** `.codegraph/` was created
between its two halves, so the two probes disagreed about a directory that
appeared mid-run. Both use the identical rule and both agree now. A run whose
tree moved underneath it is not evidence, and that is written into the record as
a trap and an open question.
