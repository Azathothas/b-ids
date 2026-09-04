# SUMMARY.md

⚠ **The last session's table, and a snapshot rather than an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record. ⛔ Overwritten every session.

---

## 2026-09-04, the final session

⛔ **Ruled the final session by the operator at its start**, with all ten open
entries in scope and twice the usual budget.

| row | before | after | from |
| --- | --- | --- | --- |
| **Elapsed** | started `2026-09-04T11:57:16Z` | `2026-09-04T19:32:53Z` | `date -u`, at both ends. About 7h35m |
| **Commits** | `e7e521e` | 8 pushed, plus this one | `git log --oneline e7e521e..HEAD` |
| **Work** | 10 open, 97 done | **10 completed, 0 deferred, 0 failed. 107 done, 0 open** | `scripts/common/check-record.sh` |
| **Effort** | not counted | **28 points**, against the 20 `RULES.md` section 10 asks for | four `L`, three `M`, two `S`, one `L`: the table in [`PROGRESS.md`](PROGRESS.md) |
| **Changes** | not counted | 173 files, +30,580 / -5,105 | `git diff --shortstat e7e521e..HEAD` |
| **Size** | 97,951 lines | 99,927 lines, **+1,976** | `git ls-files`, excluding `references/` and `vendor/`, over `.rs .sh .ps1 .mjs .md .json .toml .yml` |
| **Checks** | 40, 39 passed and 1 skipped | **44, 43 passed and 1 skipped** | `check-gate.sh --fast`, at both ends. The skip is `check-twins`, which `--fast` skips by design |
| **Tests** | 445, carried from the last close | **463**, counted | `cargo test --workspace --all-features`, summed from the runner |
| **Corpus** | 12 profiles, 3 browsers | **14 profiles, 4 browsers.** 8 of 10 planned cells captured, 0 absent, 0 outside the plan | `b-ids-corpus verify` and `check-coverage.sh` |
| **Cost** | none | one dispatched `capture.yml` run, 11 jobs, all green. No money, no registry, no host | run `33882426404` |
| **Health** | 1 vendored tree, 1 patch | 2 vendored trees, 5 patches, 9 crates, 40 check pairs. Tree clean, pushed. Nothing deployed | `check-vendor.sh`, `git status` |

---

## ⭐ What actually moved

⛔ **The corpus stopped living on the branch that reads it.** Three branches now:
the default branch is the code, `source` is the canonical corpus, `data` is what
the assembler derives from `source`. That makes the derivation checkable again;
it had once reported `data branch ok` while comparing the published branch
against a copy of itself.

⭐ **The `chromium` row is captured, and it answers the question it existed for.**
At build `152.0.7977.75` on `linux64`, with the per-connection GREASE draw
removed, branded Chrome and Chromium have an identical TLS extension set, an
identical cipher list in order, and an identical HTTP/2 half. ⚠ What differs is
one HTTP header, which is a navigation condition, and the CONTENT of the bundled
root store: both send 206 bytes and the bytes are different.

⛔ **And that refutes something this project believed the same morning.** "An
unbranded build publishes an empty trust-anchor list" was a confound: the empty
one was a Chrome for Testing build.

---

## ⚠ What did NOT move, stated rather than omitted

| | |
| --- | --- |
| **the TCP half** | five of six fields are still unread. `PUB-06` measured why: it needs a packet-capture library, which makes the Windows gate fail at link time until one is installed on that runner. A machine decision, and [`../docs/HUMAN.md`](../docs/HUMAN.md) section 3 is the measurement |
| **a release** | none. A pushed tag is the only thing that cuts one, and that is the operator's act. `check-signing`'s live leg reports a skip because of it |
| **anything hosted** | ⛔ nothing. `HARNESS-12`'s oracle mode is built and no endpoint of this project's is reachable |
| **a registry** | ⛔ nothing published to one. Publishing needs a credential and this tree has none |
| **a true binding** | ⚠ `LIB-03` closed on a comparison rather than on a binding, which is the Approach's central choice not taken. The entry says so in its own words |
| **the data branch** | ⭐ it DID move: 496 files, `matched:true`. ⚠ It read behind by 91 while this table was being written, and the closing commit's publish run closed the gap. That gap is the designed state rather than a fault: `check-data-branch` distinguishes behind from wrong |

---

## ⛔ The three review passes each found something

| pass | the finding that mattered most |
| --- | --- |
| the door sweep | `validate.yml` had a SECOND corpus reader the sweep did not reach, and both CI jobs went red on the closing push of `PUB-13` |
| ⛔ the guard mutation | `check-signing` reported **green** over the exact defect it exists to catch: it matched its own explanatory comment rather than the declaration |
| the claim audit | a sentence about a hosted runner that nobody had measured, and four documents saying the corpus holds twelve profiles when it holds fourteen |

⭐ **Eight guards were planted against and seen to refuse**, each against a copy
with the file restored byte for byte afterwards. The full table is in
[`PROGRESS.md`](PROGRESS.md).
