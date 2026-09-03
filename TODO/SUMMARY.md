# SUMMARY.md

The last session's table. ⚠ A snapshot rather than an authority:
[`PROGRESS.md`](PROGRESS.md) is the record and [`INDEX.md`](INDEX.md) is the
list. ⛔ Overwritten every session.

**Session of 2026-09-03, unattended, ended by the operator.**

---

## What moved

| | |
| --- | --- |
| entries closed | 8 |
| effort points | 22, against the 20 [`RULES.md`](RULES.md) section 10 asks for |
| entries now | 100 total, 14 open, 0 blocked, 86 done |
| tests | 380 at the start, 401 at the end |
| gate | 35 at the start with one of them skipped, 38 at the end with none skipped and `check-twins` included. `check-publish`, `check-cold-start` and `check-support-matrix` joined it, each with a twin and a comparison row |
| published | ⛔ nothing. No tag, no asset, no branch, no workflow run |

---

## The entries

| entry | effort | what closed it |
| --- | --- | --- |
| `PUB-10` | `L` | `sh scripts/common/check-publish.sh`: 3 triggers over 3 jobs, 2 job-scoped writes, no force push, no named secret, 10 refusals driven against the binary |
| `EMIT-01` | `L` | `sh scripts/common/check-support-matrix.sh`: 6 cells from a run over 6 profiles, 5 holes each resolving to a file and a line |
| `EMIT-02` | `L` | `cargo test -p b-ids-emit escape_hatch`: 5 cases. 1871 of a hello's 1983 bytes emitted and found in the raw capture exactly once |
| `LIB-01` | `M` | `cargo test -p b-ids`: 7 cases and a doctest. The corpus embedded at build time, no network, no substitute |
| `LIB-02` | `M` | `cargo test -p b-ids-cli`: 4 cases, plus a driven pass with the real binaries. 1951 of 1983 bytes identical to the browser's own |
| `VALID-04` | `M` | `cargo test -p b-ids-validator digest_vectors`: 4 cases over 16 published vectors |
| `CI-05` | `M` | `sh scripts/common/check-cold-start.sh`: 11 stages, every cache refused, 9 of 9 programs present |
| `DRIVER-06` | `M` | `cargo test -p b-ids-driver branded`: 4 cases where the command previously selected none |

---

## What each review pass found

| pass | finding |
| --- | --- |
| the door sweep | ⛔ two `git push` lines exist in this tree's workflows and the new rule governed one. Widened to every workflow, with the force-pushing count pinned at one |
| the guard mutation | ⛔ 28 planted, 27 red on the first attempt. The one that passed was this session's own force-push rule, which looked for `:+` where a forcing refspec puts the plus first |
| the claim audit | ⛔ five documents said the corpus holds five profiles and it holds six; two pasted blocks had a line removed; one byte range was read off the wrong rows. All three corrected against a re-run |
| the gate itself | ⛔ two more, from running it in full: one half of `check-placeholders` is case-insensitive where its twin is not, and shellcheck refused three lines of a check written this session |

---

## What is measured and what is not

| | |
| --- | --- |
| ⭐ measured | six profiles emit whole, 1739 to 1983 bytes; four of the six share one JA4 and the split is by major rather than by platform or browser; the specification's own `8daaf6152771` reproduced from this project's captures by three independent paths |
| ⚠ not measured | the publishing workflow has never run; the `for-testing` capture lane has never run; the cold-start job has never run on a cold machine |
| ⛔ not done | nothing was published anywhere, and `corpus/` and `raw/` were not touched |
