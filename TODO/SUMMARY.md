# SUMMARY.md

The last session's table. ⚠ A snapshot rather than an authority:
[`PROGRESS.md`](PROGRESS.md) is the record and [`INDEX.md`](INDEX.md) is the
list. ⛔ Overwritten every session.

**Session of 2026-09-03, unattended, ended by the operator.**

---

## What moved

| | |
| --- | --- |
| entries closed | 3, and one worked and left open |
| effort points | 5, under the 20 [`RULES.md`](RULES.md) section 10 asks for. ⚠ The operator's first instruction was to consolidate the documents against the tree, which is not an entry and scores nothing |
| entries now | 103 total, 14 open, 0 blocked, 89 done |
| tests | 401 at the start, 406 at the end |
| gate | 38 checks at the start, 39 at the end. `check-catalogues` joined the gate, and both it and `corpus-root` joined the twin comparison |
| documents amended | 9, plus 3 script headers, with every superseded passage in [`../docs/HISTORY/stale-documents.md`](../docs/HISTORY/stale-documents.md) |
| published | ⛔ nothing new. The `data` branch is unchanged, no tag was pushed and no release cut |

---

## The entries

| entry | effort | what closed it |
| --- | --- | --- |
| `TOOL-19` | `M` | `sh scripts/common/check-catalogues.sh`: 45 scripts and 28 documents named by their catalogues, and the same check pointed at `8f031a6` refuses 13 |
| `CORPUS-06` | `M` | `cargo test -p b-ids-corpus headless`: 5 cases, and 3 planted mutations each taking a different one red |
| `PUB-12` | `S` | `sh scripts/common/check-license-consistency.sh --json`: 8 statements including the data branch, driven against a branch rewritten to `MIT` |
| ⚠ `PUB-11` | `M` | **not closed.** One resolver, 12 check pairs wired, 7 of 10 passing with `corpus/` moved out of the working tree. The entry names the 3 that do not |

---

## What each review pass found

| pass | finding |
| --- | --- |
| the claim audit | ⛔ 9 documents and 3 script headers described a smaller project than the one on disk, one of them contradicting itself two sections apart |
| the dead-caller sweep | ⛔ `b_ids_driver::headless::normalise` had five passing tests, a documented reason and no caller, so every published profile carries `HeadlessChrome` |
| the guard mutation | ⛔ 8 planted across four entries, all red. The strongest was moving `corpus/` out of the tree entirely, which is what found the three legs that still resolve the workspace root |

---

## What is measured and what is not

| | |
| --- | --- |
| ⭐ measured | the pre-consolidation tree refuses 13 scripts under the new check, in both halves, with the same exit code; a headless capture publishes the windowed token with `substituted` provenance and a windowed one is untouched; a data branch rewritten to `MIT` is refused by both halves; 7 of 10 checks read the corpus off the branch with none in the working tree |
| ⚠ not measured | no capture was taken, so `CORPUS-06` is proved by its suite rather than by a browser; the `for-testing` lane has still never run; the publishing workflow was not exercised, because running it writes to the remote |
| ⛔ not done | `PUB-11` is open; the six published profiles still carry `HeadlessChrome` and always will, because the corpus is append-only |
