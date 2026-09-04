# SUMMARY.md

⚠ **The last session's table, and it is a snapshot rather than an authority.**
[`PROGRESS.md`](PROGRESS.md) is the record. ⛔ Overwritten every session.

Session ran 2026-09-04, attended. Started `2026-09-04T00:07:59Z`.

| row | measured | from |
| --- | --- | --- |
| Elapsed | 4 h 5 min, `00:07:59Z` to `04:13:28Z` | the recorded start instant and `date -u` |
| Commits | **3**, pushed. ⚠ Two of them exist because the first push was green here and red on both runners. | `git log 601a5a4..HEAD` |
| Work | **6 completed, 2 worked and open, 0 failed, 0 deferred**. 14 effort points. | the entries, each closed in place with its acceptance command run |
| Changes | 61 files, +5132 / -384 | `git diff --shortstat 601a5a4..HEAD` |
| Size | 94,268 lines, up from 89,628. Delta +4,640 | `git ls-files` less `references/` and `vendor/`, through `wc -l` |
| Checks | gate ok, 39 passed and 1 skipped over **40**. At the start it was 38 and 1 over 39. ⛔ The skip is `check-twins`, which `--fast` skips, and it was run separately on a still tree: `check-twins exit=0`, 62 pairs, every one agreeing. So all 40 are covered rather than 39. | `check-gate.ps1 -Fast` and `check-twins.sh`, each read from the process, unpiped |
| Remote | ⭐ **`ci` green on both jobs**, run `33839660924`. ⚠ The first push of the session was green here and red on both runners; each found one defect this host could not, and both are fixed. | `gh run view`, and the two findings are in `CORPUS-02` |
| Tests | **425**, up from 401 | `cargo test --workspace` |
| Cost | no money and no bandwidth beyond `cargo` resolving the workspace. ⛔ No capture was taken, no workflow was dispatched, and nothing was pushed to any remote until the closing commit. | measured by what was run |
| Health | 4 debts cleared, 1 introduced and named, tree clean, nothing deployed | below |

---

## What the six closed entries were

| | |
| --- | --- |
| `CI-09` `M` | the Windows toolchain failure, traced to this tree's own probe and fixed from two sources |
| `PUB-11` `M` | ten of ten with the corpus moved out, plus two checks that passed by comparing something to itself |
| `PUB-04` `M` | thirty-seven generated config files, twenty-four of them refusals naming a hole at a file and a line |
| `PUB-14` `M` | the data branch check could not tell BEHIND from WRONG |
| `VALID-05` `L` | the conformance suite, with a third verdict for what a browser varies per connection |
| `HARNESS-11` `M` | the TCP layer, and the capability answer is one field of six |

⚠ **Fourteen points, under the twenty [`RULES.md`](RULES.md) section 10 asks
for.** The reason is stated rather than hidden: by the end every remaining open
entry needed an operator ruling, a capability this host does not have, or was a
large new build. ⭐ Eight rulings were taken at the close, and the work order is
now unblocked end to end.

## Debts

**Cleared:** the Windows CI failure, which had been misdiagnosed in the record
for three sessions; two checks that reported green over a tautology; a probe
that mutated the machine it was measuring; a validator message that reported
three different facts identically.

**Introduced and closed within the session:** the data branch went **behind by
37 artefacts** when `PUB-04` added a `configs/` tree. ⭐ `check-data-branch`
reported that as pending rather than failing, which is `PUB-14`, and the closing
push triggered `publish.yml`, which pushed 235 artefacts. The check now reports
`matched:true pending:0`, so the design was confirmed end to end rather than
argued.

**Deployed version:** none. No tag was pushed and no release cut.
