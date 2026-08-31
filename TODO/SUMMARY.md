# SUMMARY.md

⚠ **The last session's table. A snapshot, never an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and
[`INDEX.md`](INDEX.md) is the list; where this file and either of those
disagree, this one is the stale copy.

⛔ Overwritten every session. Every cell is grounded in something that can be
pointed at, including the cells that say nothing moved.

---

## 2026-08-31

| row | before | after | measured by |
| --- | --- | --- | --- |
| Elapsed | 2026-08-31T14:10:15Z | 2026-08-31T16:43:25Z, about 2h33m | `date -u`, at both ends |
| Commits | 1 | 2 | `git log --oneline` |
| Work | 7 done | 22 done, 1 partial | `check-record.sh` |
| Work this session | | **15 entries closed, 21 effort points**, 1 left partial with its blocker named | [`INDEX.md`](INDEX.md) rows |
| Changes | | 77 files, +9,872 / -286 | `git diff --cached --shortstat` |
| Size | | 32,149 tracked lines outside `references/` | `git ls-files \| xargs wc -l` |
| Checks | 15, and 14 of them green with 1 skipped | ⭐ **19, all green**, both halves of every pair | `check-gate.sh`, full run |
| Twin pairs | 13 | 15 | `check-twins.sh` |
| Suite | ⛔ none existed | 107 tests across 4 crates | `cargo test --workspace` |
| Cost | | 2 Rust toolchains installed on this host, about 600 MB; 11 crates fetched from the registry | `rustup toolchain list`, `Cargo.lock` |
| Health | 2 documented-and-absent tools | ⭐ 5 defects found by running the tree, all 5 fixed | below |

## What the three review lenses found

⛔ **Three different questions, not one sweep written up three times.**

| lens | what it swept that the others did not | what it found |
| --- | --- | --- |
| 1. the door sweep | every construction site of a header field, and every caller of the credential filter, by grep rather than from memory | ⭐ **A third door, open.** Two capture paths were gated and tested; deserialisation was neither, so a profile read from a file could carry a cookie header. Fixed, with a test that reads one from JSON. |
| 2. the guard mutation | 12 guards, each planted with the defect it exists to catch and the exit code read unpiped | ⛔ **Two guards that could not fail as written**, and one **test that hung instead of failing**. All three fixed. The MSRV fixture also had to be re-planted: Cargo promotes a path dependency into the workspace, so the first attempt proved nothing. |
| 3. the claim audit | every acceptance command in `TODO/`, every timing figure in the gate headers, and every path this session wrote into a document | ⚠ **The gate's timing figures described a 13-pair, 15-check tree.** Re-measured where it could be, and the full-run figure is a dash rather than a number nobody took. Every `-p <crate>` in the tree now resolves. |

⭐ **Lens 1 is the one that paid.** It was run last, over code that lens 2 had
already mutation-proved twice, and it still found an ungated path: the two
guards were correct and the enumeration of doors was not.

## What did not move

- ⛔ **No capture has been taken and the corpus is empty.** Every value in the
  tree is a fixture or an inherited claim.
- ⛔ **Nothing is published.** `PUB-01` and `PUB-02` are untouched.
- **`HARNESS-02` is partial**, and `--ca-out` and `--until-h2` are absent rather
  than inert.
- **The full-gate wall time was not re-taken.** The run went green, all 19
  checks; the timing line was lost when the shell holding it was killed, and a
  figure nobody measured is not written down.
