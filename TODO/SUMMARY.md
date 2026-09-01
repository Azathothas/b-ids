# SUMMARY.md

The last session's table. ⚠ **A snapshot, never an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and the work order;
[`INDEX.md`](INDEX.md) is the list. When this file and either of those disagree,
this one is stale.

⛔ Overwritten at the end of every session.

---

## 2026-09-01, 03:47:48Z to 07:00Z

| row | measured |
| --- | --- |
| Elapsed | 3h12m, from the recorded start instant to the last commit |
| Commits | the session work squashed to 1, then 3 more: the review record, a red-build fix, and this table. 4 pushed on `main` from `bd02855`. |
| Work | **7 entries completed**, 15 effort points. 0 deferred, 0 failed. Operator interrupt ended the session before the twenty-point quota. |
| Changes | 273 files changed, 73,058 insertions, 343 deletions. ⚠ 210 of those files are the vendored tree; 63 are this project's own. |
| Size | 43,268 lines tracked outside `references/` and `vendor/` |
| Checks | ⭐ green, local and remote. 20 checks, 19 passed and `check-twins` skipped by `--fast`; run separately, 1025s, every pair agrees. At the start: 19 checks, same shape. ⚠ The remote went red once, on a test of this session's own, and was fixed before the session ended. |
| Tests | 192 across 22 test files in 4 crates, up from 166 in 16. ⚠ One of them, the driven capture, is opt-in behind an environment variable and is gate part (b). |
| Cost | no money. Network: one shallow clone of rustls, two crates.io resolutions, one reference-corpus read. |
| Health | ⛔ 1 fabricated number found in this session's own writing and corrected in three files. 3 traps paid for and written down. 1 stale document claim fixed. Tree clean, nothing deployed, nothing published. |

---

## What closed

| entry | effort | what it now does |
| --- | --- | --- |
| `VENDOR-01` | L | rustls vendored at `v/0.23.43` and compiled here, with a manifest, a change record, a derived patch series and a two-legged scan |
| `HARNESS-13` | L | `--ca-out` mints an authority, terminates the handshake, and the harness reads a browser's HTTP/2 through it |
| `HARNESS-02` | M | all nine switches, after one day at `partial` |
| `HARNESS-05` | S | the priority block measured on two browsers: `80000000ff` on all thirteen HEADERS frames |
| `VALID-02` | S | ten exhibits in two public repositories, each with a file, a line and the check it fails |
| `DRIVER-01` | M | resolve a browser with the source that answered, drive it into a profile nobody keeps |
| `DRIVER-03` | S | the headless product token, measured rather than inherited, and the substitution recorded |

---

## ⛔ What is still not true

- **The corpus is empty.** Captures were taken and nothing writes a profile.
  `CORPUS-01` is the top of the work order for that reason.
- **Nothing is published.** No release, no data branch, no route.
- **One quantity is `measured-here`** and it is not published either.
