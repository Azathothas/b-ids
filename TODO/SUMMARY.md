# SUMMARY.md

⚠ **The last session's table, and a snapshot rather than an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and the work order;
[`INDEX.md`](INDEX.md) is the entry list. ⛔ Overwritten every session.

**2026-09-01T12:32:05Z to 2026-09-01T15:20:00Z, unattended, ended by operator
interrupt.**

---

## What closed

| entry | eff | what it delivered |
| --- | --- | --- |
| `CI-01` | M | every push settles what is published, on two hosts, network off for every assertion. ⛔ Found `check-corpus`'s history leg verifying nothing in CI since the day it was written. |
| `TOOL-17` | S | the line-endings rule extracted to its own pair, reading the working tree as well as the index. ⛔ Found `check-routes.ps1` LF on disk against its own `eol=crlf`. |
| `TOOL-16` | S | a drift alongside a moved tree is UNDECIDED at exit 2. ⭐ Reproduced twice, once by accident. |
| `TOOL-15` | M | `--timings`, then the scope it named. 1056s to 636s wall, and the `check-gate` row 431s to 54s. |
| `TOOL-04` | S | the reference fetcher degrades instead of stopping; only both routes down is exit 1 |
| `DRIVER-05` | M | acquisition: routes tried in order, the one that answered recorded with the digest of what arrived |
| `CI-03` | L | the capture matrix, fanning out from a plan in the tree, every lane failing alone, with a fixture that breaks each rule once |

**13 effort points closed.** Seven entries authored and filed: `SCHEMA-12`,
`SCHEMA-13`, `SCHEMA-14`, `TOOL-15`, `TOOL-16`, `TOOL-17`, `HARNESS-14`.

## What did not close, and why

| entry | eff | the blocker |
| --- | --- | --- |
| `CORPUS-02` | L | ⛔ **Its apparatus is built and no lane has run.** The plan file, the coverage check in both halves and the fan-out that reads the plan are all here; closing it needs one run of `capture.yml` on a hosted runner and the `linux64` profile committed, which needs this session's commit on the default branch. |

## The four checks that were green over nothing

| the check | what it was not checking |
| --- | --- |
| `check-corpus` | its history leg under `actions/checkout`'s default one-commit clone, on every CI run since it was written |
| the gate's line-endings filter | git's working-tree column. Eight files went CRLF last session and it stayed green. |
| `check-twins` | whether a reported drift was a drift or a tree that moved |
| `mine-repo` | anything at all, on a host that could clone and not reach the API |

## Three defects in this session's own new code

| what | how it was found |
| --- | --- |
| an uninitialised awk variable used as a subscript is the empty string, not zero | `check-workflows` reported a job that does not exist, once per file |
| jq on this Windows host writes CRLF | `check-coverage`'s human report disagreed with its twin while the JSON matched |
| the driver cannot link the harness | the compiler, on the first build. The digest is injected now. |

## The state the tree is left in

| | |
| --- | --- |
| the gate | 25 checks and `check-twins` over 22 pairs, both halves, green |
| the suite | 256 tests in 28 files across 5 crates |
| entries | total 91, open 43, blocked 0, done 48 |
| the corpus | one profile, unchanged. ⚠ One source, not two. |
| published | nothing |
| open questions | ⭐ none. All three were put to the operator at the close and answered; the rulings are in [`PROGRESS.md`](PROGRESS.md). |
| commits | two. ⚠ The first pushed a gate transcript carrying an absolute home path, which the remote refused on both hosts; the second elides it. ⛔ Amending would need a force push, which is refused. |
