# SUMMARY.md

The last session's table. ⚠ **A snapshot, never an authority.**
[`PROGRESS.md`](PROGRESS.md) is what is true now; a session that reads this file
and acts on it is acting on what was true last time.

---

## 2026-08-31, unattended

| | before | after |
| --- | --- | --- |
| Elapsed | 2026-08-31T00:30:00Z | to the commit below |
| Commits | ⛔ none. The tree was not a git repository. | 1, squashed |
| Work | 76 entries, 3 closed | 81 entries, 7 closed. 5 filed this session, 4 of them closed, 0 deferred, 0 failed |
| Changes | a tree describing a repository nobody had fetched | 1 reference tree added, 20 MiB; 3 research documents rewritten; 5 records rewritten; 2 files deleted; 58 entry source lines re-pointed |
| Size | 4,476 tracked files: 87 this project's own | ⭐ unchanged in shape, 87 own files and 4,389 corpus files. `check-docs` reads 48 of them as documents. |
| Checks | ⛔ **the gate had never passed.** `check-docs` 11 problems, `check-twins` 12 drifts | ⭐ 15 of 15, 0 skipped, both halves of every pair, and green on both runners |
| Cost | no money | 1 repository fetched over an authenticated route, 20 MiB kept |
| Health | 11 broken links, 5 documents naming files that did not exist, 60 values citing nothing | tree clean, record consistent, nothing deployed |

### The gate, on this host

⚠ **This host is not the host a contributor will have.** It has `pwsh`,
`shellcheck` and the PowerShell analyzer, so both halves of every pair ran
before pushing. A host without them reports skips, and a skip is not a pass.

| check | result |
| --- | --- |
| `check-docs` | pass. 48 files, 662 links, 129 shell blocks |
| `check-markers` | pass. 83 files, 1,990 markers, densest 28 per 100 lines |
| `check-one-home` | pass |
| `check-placeholders` | pass |
| `check-control-bytes` | pass |
| `check-record` | pass, 81 entries, counts agree with rows |
| `check-no-secrets --public` | pass |
| `check-changelog` | pass |
| line endings | pass |
| `sh -n` over every tracked shell script | pass |
| `shellcheck` | pass |
| PowerShell parse over every tracked `.ps1` | pass |
| PowerShell analyzer | pass |
| the probe | pass |
| `check-twins` | pass |
| `check-remote-items` | pass. ⚠ It ran: this host has an authenticated `gh` |
| the test suite | ⛔ **absent.** There is no code. `TOOL-02` is the entry. |

⭐ **And on the two runners**, which is where the twins earn their keep:

| job | result |
| --- | --- |
| `gate (ubuntu)` | pass. The full gate under `--strict`, plus the probe and a yaml parse over every workflow |
| `gate (windows)` | pass. The PowerShell half under `-Fast`, plus an assertion that the analyzer was not skipped |

⛔ **Both were red on the first push and the failures were real.** A corpus 92
files short, and a lint the Windows runner did not have. `TOOL-12` and
`TOOL-13`.

⛔ **One of those rows is not a pass and the table says so.** An absent suite is
not a green suite.

### What moved

- ⭐ **The origin repository was fetched, at a named commit, and read.** Every
  inherited value now cites a file in it. Before this session, none of them
  could be checked at all.
- **Five claims changed on that reading**, one of them a refutation of the
  founding brief by the capture the brief was quoting.
- **Two failing checks fixed**, one by deleting a tool seven files described
  and nothing needed.
- ⭐ **A guard that did not exist was found by planting the defect it was
  supposed to catch**, and implemented in both halves. `TOOL-11`.
- ⛔ **The first push found the corpus was 92 files short**, because a mined
  tree's own ignore rules had swallowed them, one being the capture every
  inherited value is cited against. `TOOL-12`.
- **Five entries filed**, `TOOL-09` through `TOOL-13`. Four close here;
  `TOOL-10` is the check that would have prevented this session and it is open.

### ⚠ What was not measured

- **Anything on a wire.** No browser was launched and no socket was opened, by
  instruction. Every value in this tree is inherited or read from source.
- **The gate's own cost.** It was run repeatedly and never timed. `TOOL-07`.
- **Whether the origin's numbers reproduce.**
  [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 11 says
  what one capture from one instrument does and does not establish.
