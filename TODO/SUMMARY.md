# SUMMARY.md

The last session's table. ⚠ **A snapshot, never an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record and it is what a session reads first.

⛔ Overwritten every session. Every cell here is grounded in something a reader
can point at, including the cells that say nothing moved.

---

## 2026-09-02, unattended, from 11:22:03Z

| | |
| --- | --- |
| entries closed | **8**, for **20** effort points on [`RULES.md`](RULES.md) section 10's scale |
| entries now | total 98, open 26, blocked 0, done 72 |
| corpus | 3 profiles to **6**. Two browsers, two majors, two platforms |
| gate | 26 checks to **30**, plus `check-twins`, and about 600 seconds to **179** |
| tests | 309 to **344**, in 37 files |
| runner runs | 9 dispatched, 9 read |

### What closed

| entry | effort | what it took to close it |
| --- | --- | --- |
| `DRIVER-08` | L | the purge and the install executed on hosted runners, both platforms and both routes. `provision.yml` run 33628209454 |
| `DRIVER-10` | L | a second browser family, from a route table rather than branches. Edge captured for the first time |
| `SCHEMA-08` | L | five formats, one generator, a reader for each round trip |
| `HARNESS-15` | M | the two halves selected independently, proved against the navigation that could not publish under the old rule |
| `TOOL-18` | M | the gate's cost, and the cause was not the one the premise named |
| `CORPUS-04` | M | the trust-anchor list measured here and published per build, with the trade stated and no preference asserted |
| `HARNESS-16` | S | `certutil -addstore -user Root` returns **124** on `windows-latest`: it never answers |
| `PUB-08` | S | one model, two renderers, and a check whose comparison was seen to fail |

### What moved without closing

| entry | what changed about it |
| --- | --- |
| `CORPUS-02` | its acceptance refused three rows this morning and refuses two now. Both are blocked on `b_ids_driver::Family` knowing two families |
| `EMIT-03` | its measurement is in: all six profiles carry the priority block and agree exactly, so it takes the branch that needs the HTTP/2 library vendored |
| `CI-08` | amended: `check-manual-path` read tracked files only, so a workflow written and never staged escaped it |
| `DRIVER-06` | unblocked by nothing and blocked by the open question below |

### ⛔ What is not true, said plainly

| | |
| --- | --- |
| the corpus is not a matrix | 6 profiles, two families of four planned. `chromium` and `firefox` cannot be resolved at all |
| no capture has gone through an unbranded build | the two `for-testing` cells are planned and not attempted, on the open question below |
| `captured.operator` is still typed | the identity writer leaves it empty and a person fills it in, this session's three profiles included |
| `Shuffle::Observed` is still never written | a field the model carries that the capture path never fills |
| the PowerShell half of the gate did not get `TOOL-18`'s speedup | `check-control-bytes.ps1` is 27 s against its twin's 1 s |
| ⛔ one commit bypassed `git-sync` | it is compliant, verified afterwards, and that is luck rather than method |

### The one open question

**Where does an unbranded build live in the corpus?** The published route is
`browser/channel/platform/version` and carries no `branded`, and `Channel` is a
closed vocabulary that does not include the vendor's automation channel, so a
branded and an unbranded build of one version publish at one path.
**Recommendation: add `for-testing` to the `Channel` vocabulary.**
[`PROGRESS.md`](PROGRESS.md) carries the reasoning and the alternative that
loses.
