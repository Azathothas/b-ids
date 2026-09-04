# SUMMARY.md

⚠ **A snapshot of one session, never an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and
[`INDEX.md`](INDEX.md) is the list. ⛔ Overwritten every session.

Session of 2026-09-04, attended, started `2026-09-04T06:04:28Z`.

| row | before | after | measured by |
| --- | --- | --- | --- |
| Elapsed | `06:04:28Z` | `09:05Z`, about 3h 00m | the recorded start instant and `date -u` |
| Commits | `e368f54` | `e7d2918`, 6 commits | `git log --oneline e368f54..HEAD` |
| Work | 12 open | **2 completed**, 0 deferred, 0 failed. 5 effort points | `DRIVER-11` `L`, `DOC-03` `S`, each closed in place with both acceptance commands run |
| Work, not counted | | `CORPUS-02` advanced and still open; `DOC-02`'s premise corrected | an entry closes on its acceptance command, and `CORPUS-02`'s still exits 1 |
| Changes | | 43 files, +5163 -149, excluding the mined reference tree | `git diff --shortstat e368f54 HEAD -- . ':(exclude)references/'` |
| Changes, everything | | 796 files, +745419 -149 | the same command without the exclusion. ⚠ The difference is `mozilla/nss` |
| Size | 162,152 lines | 167,187 lines, +5,035 | tracked files excluding `references/`, concatenated and counted |
| Size, everything | | 4,030,911 lines | the same, with the reference corpus |
| Checks | gate ok: 39 passed, 1 skipped | **gate ok over all 40**, `check-twins` included | the FULL `check-gate.sh` was run at the close on a still tree. ⚠ Its first run failed on one check, `check-markers`, over two U+2212 minus signs this summary itself had introduced; fixed, and the re-run is green. ⭐ `check-twins` passed, which is what says the new pair's two halves agree |
| Tests | 412, carried from the last session | **445**, counted | `cargo test --workspace`, summing the runner's own `test result: ok` lines |
| Corpus | 6 profiles | **12 profiles** | `b-ids-corpus verify`: `corpus=profiles:12 problems:0` |
| Coverage | 3 of 9 cells captured, 3 absent | **6 of 9 captured, 0 absent** | `check-coverage.sh` |
| Cost | | no money. One reference clone, trimmed to 26 MiB on disk; four workflow runs on hosted runners | the clone is `references/mozilla__nss`; the runs are `33849365530`, `33849934489`, `33851238648`, `33854002345` |
| Health | | debts cleared: 4 checks that would have refused every future capture. Introduced: none known. Tree clean, pushed, and all three remote workflows green on the closing commit. ⭐ The data branch caught up on that push: 237 files to **405**, and its tree IS what this corpus derives to. ⚠ No release | `check-data-branch` after the publish run, and `gh run list` on `0b25a62` |

---

## What the session actually delivered

⭐ **The corpus has profiles from a stack that is not a Chromium**, and it has
them because `DRIVER-11` built a launch path for Gecko. Firefox takes no
command-line equivalent of the Chromium certificate switches, so the trust is
arranged where Firefox looks for it: an NSS certificate database written into
the throwaway profile the launcher already creates, carrying the run's own
authority and a trust record for it.

⭐ **The capture matrix adds profiles now.** It never had. Six lanes ran green
this morning, six captures were actually taken, and the run added nothing and
reported success.

⛔ **Four checks that were green would have refused every capture after the
first**, and all four were found by profiles actually landing rather than by
reading:

| | |
| --- | --- |
| `check-data-branch` | called nineteen changed aggregates a rewritten branch |
| `check-coverage` | could not see a capture outside the plan |
| `check-trust-anchors` | called an unbranded build's EMPTY root-store list a defect |
| the collect job | kept whichever lane's index it copied last, rather than deriving it |

⚠ **And one hole that is now an instrument rather than a manual step.** A
published profile needs a JA4 vector and nothing derived one, so five new
profiles left the gate red until a person ran a command out of a document.
`scripts/common/derive-ja4-vector` is that command, in both halves, compared.

---

## ⛔ What did not move, and what was measured instead

⚠ **`CORPUS-02` did not close**, and its acceptance command still exits 1 on one
row: `chromium`. ⭐ That blocker is measured now rather than predicted. The
resolver finds `/usr/bin/chromium` on `ubuntu-24.04` and reads `151.0.7922.0`
from it, so the cell was never blocked on resolution; the launch aborts on
signal 6 inside `content::ZygoteHostImpl::Init`, which is the snap sandbox.

⛔ **`PUB-13` was read and not started.** It removes `corpus/` from the default
branch, and starting it without finishing it would leave the tree in a state the
next session cannot tell from a finished one.

⚠ **Five effort points against the twenty
[`RULES.md`](RULES.md) section 10 asks for.** Both entries taken grew: a launch
path became four defects in shipped code, and a merge became a workflow that had
never worked. Following them was the trade, and it is stated rather than hidden.
