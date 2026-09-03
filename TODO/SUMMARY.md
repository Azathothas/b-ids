# SUMMARY.md

⚠ **The last session's table. A snapshot, never an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and it is what a session reads first.

Session ran 2026-09-02T23:21:15Z to 2026-09-03T06:00Z, unattended, ended by the
operator.

| row | before | after | measured by |
| --- | --- | --- | --- |
| Elapsed | 2026-09-02T23:21:15Z | about 6h 40m | the recorded start instant against `date -u` |
| Commits | 3c1be00 | one squashed commit on `main` | `git log --oneline` |
| Work | 26 open, 72 done | 20 open, 78 done. **6 completed, 0 deferred, 0 failed** | `sh scripts/common/check-record.sh` |
| Effort | 0 of 20 points | **13 of 20**: one `L`, four `M`, one `S` | [`RULES.md`](RULES.md) section 10's own scale |
| Changes | | 48 files, +7837 / -179 | `git diff --shortstat` over the session range |
| Size | 5454 tracked files | 5471 tracked files, +17 | `git ls-files \| wc -l` |
| Tests | 353 passing in 38 files | 374 passing in 40 files, 0 failing | `cargo test --workspace`, exit 0 |
| Checks | 30 passed, 1 skipped, 31 registered | 34 passed, 1 skipped, 35 registered | `check-gate.ps1 -Fast`, exit 0 |
| Twin pairs | 30 agreeing | 34 agreeing | `check-twins.sh`, exit 0, counted from its own rows |
| Corpus | 6 profiles | 6 profiles, unchanged | ⛔ No capture was taken. Nothing was added to the corpus. |
| Cost | | ⛔ Not measured. No runner was dispatched, nothing was downloaded and no release or branch was pushed. | |
| Health | | 4 gate pairs added, 4 defects in this session's own code fixed, 2 pre-existing readers fixed, tree clean, nothing deployed | the review passes in [`PROGRESS.md`](PROGRESS.md) |

## What closed

| entry | effort | what it is |
| --- | --- | --- |
| `SCHEMA-12` | `L` | YAML, TOML, a SQLite dump and a protobuf definition from the one generator; CBOR and MessagePack declined with published reasons |
| `CI-04` | `M` | the pull-request body, branch, labels and five merge conditions, three computed from the published profiles |
| `PUB-03` | `M` | 54 flat routes with a check that reads the corpus rather than the generator |
| `PUB-01` | `M` | one assembler, a byte-identical build, and a tag that cannot overwrite a pinned one |
| `PUB-02` | `M` | the same assembler for the data branch, every file checksummed twice |
| `PUB-07` | `S` | the licence in seven places from one home |

## ⚠ What did not move, and how that was established

| | |
| --- | --- |
| the corpus | ⛔ **6 profiles, unchanged.** No browser was launched and no capture taken. `b-ids-corpus verify` reports the same six. |
| `DRIVER-06` | open. `Channel::ForTesting` landed and the suite is green over it; its acceptance command selects no test yet, and the entry says so. |
| `CORPUS-02` | open and unchanged. Still blocked on `b_ids_driver::Family` knowing `firefox` and `chromium`. |
| anything outside this machine | ⛔ **Nothing.** No workflow dispatched, no tag, no release, no branch, no pull request, no issue. |
| the two `for-testing` matrix cells | still `enabled: false`. The vocabulary no longer blocks them; enabling one is the operator's or the next session's call. |

## ⛔ Where a number is absent

- **Cost** is not measured because nothing was spent: no runner minutes, no
  downloads, no publishes.
- **The gate's full (non-`--fast`) wall time** was not taken this session. The
  `--fast` half and `check-twins` were both run to green; timing them was not.
