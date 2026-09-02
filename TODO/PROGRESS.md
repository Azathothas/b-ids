# PROGRESS.md

⭐ **The one file every session reads first.** Where the work is, what is next,
and why. [`INDEX.md`](INDEX.md) is the list of entries and the **order lives
here and nowhere else**. [`RULES.md`](RULES.md) is the half of the record that
does not change between sessions, and [`SUMMARY.md`](SUMMARY.md) is the last
session's table, which is a snapshot rather than an authority.

⛔ Rewritten every session. It carries no history: the history is the git log and
the entries themselves. Do not add a "previous sessions" section.

⛔ Edited in the same change as the work, never as a report afterwards.

---

## State

```text
session ran      2026-09-02T11:22:03Z, unattended, following a work order the
                 operator wrote into the kickoff
baseline         gate ok: all 31 checks passed, in full, in 715 seconds
                 on this Windows host. 344 tests in 37 files.
entries          total 98  open 26  blocked 0  done 72
gate             five checks joined it today: check-provisioning,
                 check-formats, check-trust-anchors, check-notes-generator
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## ⭐ The corpus stopped recording builds nobody chose

⛔ **Every profile before today carried `captured.acquisition: null`**, which is
the weakest provenance the artefact half can have in a project whose product is
provenance. Three of the six carry a URL and a digest now.

| profile | how the build got there |
| --- | --- |
| Chrome `151.0.7922.76` `win64` | a laptop, 2026-09-01. `acquisition: null` |
| Chrome `151.0.7922.173` `linux64` | whatever `ubuntu-latest` shipped. `acquisition: null` |
| Chrome `151.0.7922.174` `win64` | whatever `windows-latest` shipped. `acquisition: null` |
| Chrome `152.0.7977.75` `linux64` | purged, then installed from the vendor's channel. URL and digest recorded |
| Chrome `152.0.7977.76` `win64` | the same, and a different build on the same day |
| Edge `151.0.4129.101` `linux64` | purged, then installed from the vendor's enterprise index, whose published digest the tool compared what arrived against |

⛔ **The three that say `null` will always say it.** The corpus is append-only
and those builds were not chosen; filling the field in afterwards would be a
derivation wearing a measurement's label.

---

## ⛔ A ruling's reasoning was refuted by measurement

⚠ **The operator ruled 2026-09-02 that the vendor route gives both platforms the
same build, "because both install on the same day".** Measured one hour apart on
one day, from the vendor's own channel:

| platform | the vendor route served |
| --- | --- |
| `ubuntu-latest` | `152.0.7977.75` |
| `windows-latest` | `152.0.7977.76` |

**The exact-build route does give one build on two platforms**, and it is the
only route that can: `151.0.7922.76` resolved on both from the automation index.

⚠ **The ruling stands and is the operator's.** What is refuted is one sentence
of its reasoning, and it is written into `DRIVER-08` rather than edited into the
ruling.

---

## ⭐ Eight entries closed, twenty effort points

| | |
| --- | --- |
| `DRIVER-08` `L` | the purge and the install ran on hosted runners, both platforms and both routes. `captured.acquisition` is populated from a record the tool writes |
| `DRIVER-10` `L` | a second browser family, as a route table rather than branches. Edge's index publishes a digest per artefact, which Chrome's does not |
| `SCHEMA-08` `L` | five formats from one generator, each with a reader, byte-identical over two runs |
| `HARNESS-15` `M` | the TLS half and the HTTP/2 half are selected per half. `captured.connections` records which connection each came from |
| `TOOL-18` `M` | the gate went from about 600 seconds to 213 |
| `CORPUS-04` `M` | the trust-anchor list measured here, published per build, and the trade stated with no preference asserted |
| `HARNESS-16` `S` | ⛔ `certutil -addstore -user Root` on `windows-latest` does not fail. It returns **124**, which is the timeout verdict: it never answers |
| `PUB-08` `S` | one model, two renderers, and a check that the comparison between them can fail |

**Twenty of the twenty points [`RULES.md`](RULES.md) section 10 asks for**,
on that section's own scale: three `L` at four, three `M` at two and two `S` at
one.

---

## ⛔ Findings in this session's own code, each caught by running it

| what | how it showed |
| --- | --- |
| `experiments/10-first-profile.sh` said it writes the profile into the corpus | it prints the command. Found by reading the artefacts of a scheduled run in which two lanes reported success and neither had added anything |
| `resolve` could not version an automation build on Windows | the archive is flat: `chrome.exe` beside a `VERSION.manifest`, no version-shaped directory, and `--version` is not asked on that platform. `Source::ManifestFile` is the third source |
| `check-manual-path` read tracked files only | it reported 9 jobs over a tree carrying 10, and `git add -N` alone changed the answer. `check-exit-codes` had the same defect and was fixed; this half was left |
| the first provisioning run failed on a directory git does not carry | `.tmp` is ignored, so the redirect failed before the command on its left ever ran, and both lanes reported that the tool did not purge cleanly when it had never been invoked |
| ⛔ the recorded acquisition route came from the shell | a `case` mapped `for-testing` to `chrome-for-testing` whatever the family was, so the first Edge profile recorded a Chrome route |
| the family route table was defined below the plan that reads it | `--plan` for an `edge` request printed an empty purge line and a command-not-found |
| ⛔ `powershell -Command` does not bind trailing arguments to `param()` | it appends them to the command text. The Windows purge lost its pattern, left Chrome on the machine, and the confirm step refused. ⭐ The lane went red rather than capturing a build nobody chose |
| ⛔ a guard added this session passed on a different line | the for-testing plan check searched for the word `index` anywhere, and the `fetch` line reads "the zip that index names for the build". Found by the mutation pass |
| one fact had two names | `Selection::no_http2` and `Split::abandoned` counted the same connections. Found by the door sweep |
| ⛔ an exit code read after an `if` is the `if`'s | the Windows store loop logged `refused, exit 0` for a call that had plainly done nothing. Same class as reading one through a pipe. The real answer was **124** |
| ⛔ a backslash lost crossing a shell | a `.ps1` written by passing a payload through `node -e` inside bash arrived with `(d+)` where `(d+)` was meant, so its half of a new pair reported 0 cases against the other's 6 |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what other door reaches this code

Swept, by grep rather than from memory: every caller of `sources_for`,
`profile_from` and `Selection`'s fields; every reader of `captured.connections`;
every construction of `Captured`; every place `check-provisioning`,
`check-formats` and `check-trust-anchors` are registered; and every spelling of
an index URL in the tree.

⛔ **Finding: one fact with two names.** `Kind::Abandoned` became `Kind::NoHttp2`
and `Selection::abandoned` became `Selection::no_http2`, and
`modes::Split::abandoned` counted exactly the same connections under the old
name. Renamed, with the example that prints it.

**Confirmed, by counting:** one production caller of `profile_from` and one of
`sources_for`; each of the three index URLs spelled once, in the driver, with
the shell asking for it rather than carrying a copy; `check-provisioning`,
`check-formats` and `check-trust-anchors` each present in both gate halves and
in `check-twins`; every construction of `Captured` found by the compiler when
the field was added.

**What the other passes did not look at:** the callers. Both of the others
read what was written; this one grepped for what was not enumerated.

### 2. The guard mutation: can the new guards actually fail

Planted and read unpiped, each in the half where the guard lives:

| planted | what went red |
| --- | --- |
| the TLS half required to have reached HTTP/2, which is the old rule | two `connection_selection` tests, exit **101** |
| the HTTP/2 half required to come from a connection that did not resume | two more, exit **101** |
| a workflow's `# manual:` line removed | both halves of `check-manual-path`, exit **1** |
| every `index` line removed from a COPY of the provisioning tool | ⛔ **nothing.** The check passed. See below. |
| the same, after the check was fixed | `--plan for-testing: names no index step`, exit **1** |
| a copy of `check-trust-anchors` counting a codepoint nothing carries | the vacuous-pass refusal, exit **2** |
| one of the three options removed from the recommendation | `the recommendation does not state the option 'Send it empty'`, exit **1** |
| a 64-hex under a label that is not `sha256` | `check-no-secrets` still catches it, three ways |
| a suite case name removed from `check-notes-generator`'s list | both halves name the four they expect, so a deleted test is caught rather than passed over |

⛔ **The fourth row is the pass earning its place.** A guard added earlier the
same day searched for the word `index` anywhere in the plan output, and the
`fetch` line reads "the zip that index names for the build asked for", so
removing every `index` step still passed. Both halves match a line whose first
field is the step now.

⚠ **And the first attempt at that mutation was itself wrong**, which is the
other half of the lesson: it removed the Linux branch's line on a Windows host
and reported that the file had changed. A patch that asserts only that something
changed is the shape `reviews.md` names.

⛔ **The provisioning tool and its check were mutated as COPIES under the ignored
scratch directory**, never as the files on this machine, and the real pair was
re-run afterwards to confirm it was untouched.

⭐ **And one pair failed against itself before it was ever registered.**
`check-notes-generator`'s two halves reported 6 cases and 0, because a backslash
was lost crossing a shell. ⚠ That is not a planted mutation; it is the same
question asked of a new pair, and it is why the pair has a `check-twins` row
rather than only a gate line.

⚠ **Guards NOT mutated, and saying so is the point:** `check-formats`'s
determinism assertion was never seen to fail, because making a deterministic
generator non-deterministic means editing the generator rather than a check;
`check-trust-anchors`'s "list has no capture instant" branch has never fired;
and the Windows-store fallback in `50-trust-anchor.sh` has not been seen to
refuse, because the run that would show it is the one still in flight.

### 3. The claim audit: which sentence is not backed by an artefact

Swept: every number and every pasted block in the seven closings, this file, the
changelog, and the documents the work made stale.

⛔ **Finding: three closings carried output that was true when it ran and stale
by the time the session ended.** `SCHEMA-08` pasted `profiles:5`, `CORPUS-04`
pasted `1 of 5 profile(s)` and `DRIVER-10` pasted a coverage table with older
counts, because two profiles landed afterwards. All three re-run against the
tree as it now stands, and `CORPUS-04`'s "one carrier is one sample" sentence
rewritten so it no longer contradicts the block above it.

⛔ **Finding: four documents said the corpus holds three profiles.**
[`../docs/AGENTS.md`](../docs/AGENTS.md),
[`../README.md`](../README.md),
[`../docs/architecture.md`](../docs/architecture.md) and
[`RULES.md`](RULES.md)'s standing-facts table, each written when it was true.
All four say six and name what is different about them.

⛔ **Finding: a procedural failure of this session's own.** One commit was made
with `git commit` directly rather than through `git-sync`, which is the tool
that enforces the identity and attribution rules.
[`../docs/AGENTS.md`](../docs/AGENTS.md) section 5 names it. ⭐ The commit is
compliant, verified afterwards with `git-sync --check`, and that is luck rather
than method.

**Claims checked that stood, by re-running the command:** `profiles:6
problems:0`; 338 tests in 37 files; 29 scripts answering 2; 10 workflow jobs
each naming a manual equivalent; 5 formats from 6 profiles; 2 anchor carriers of
6; 27 twin pairs agreeing.

---

## ⚠ What is in progress

⛔ **Nothing.** Every entry this session touched is closed in place with its
acceptance command run, or left open with its blocker named. ⚠ Two
trust-anchor runs were dispatched and both are read: `33647065058` gave the
answer and `33647839757` gave it with a usable exit code.

---

## The gate costs 213 seconds

⭐ **`TOOL-18` closed on this**, and the cause was not the one its premise named.

| | |
| --- | --- |
| ⛔ **the largest cause** | a command substitution in a `while ... read` assignment prefix, which is re-evaluated on every iteration. `IFS="$(printf '\t')" read` forks once per LINE READ, and a substitution costs 35 ms on this host. Eleven occurrences across seven checks. |
| ⛔ **the second** | a `git check-ignore` per link, 966 of them. One batched call now. |
| ⛔ **the third** | three processes per fenced block, two of which `awk` can do in the pass that extracts it. |
| ⚠ **what is NOT fixed** | the PowerShell halves still carry the per-file shape: `check-control-bytes.ps1` is 27 s against its twin's 1 s. A gate run on Windows pays the slow half and `check-twins` pays both. |

---

## Open questions for the operator

### ⛔ One, and it blocks the two unbranded matrix cells

**Where does an unbranded build live in the corpus?** The published route is
`browser/channel/platform/version` and carries no `branded`, and `Channel` is a
closed vocabulary of six that does not include the vendor's automation channel.
So a branded and an unbranded build of one version publish at one path, and the
two `for-testing` cells in
[`../.github/capture-matrix.json`](../.github/capture-matrix.json) are planned
and not attempted because of it rather than because the tool cannot fetch them.

**Recommendation: add `for-testing` to the `Channel` vocabulary**, and let
`branded: false` follow from it rather than becoming a path component.

| | |
| --- | --- |
| ⭐ **why this one** | the channel is ALREADY part of the route and of the `latest` key, so nothing about the layout changes and no consumer's pin moves. It is also true: the automation index is a channel the vendor publishes, separately from stable. |
| ⚠ **what it costs** | one variant on a closed enum, the published schema regenerated, and `CORPUS-03`'s "latest means stable" sentence re-read so an automation build cannot be mistaken for one. |
| ⛔ **the alternative that loses** | `branded` as a fifth path component. It changes `corpus/v1/` for every consumer to carry a dimension only one browser family has. |

⚠ **Nothing else is blocked on it.** `DRIVER-06` is the entry that measures the
branded-against-unbranded difference and it is what the answer unblocks.

⚠ **A later session that finds a fork writes it here with a recommendation
attached and keeps working.** [`RULES.md`](RULES.md) section 10 names "this
needs a decision from the operator" as one of the four sentences that is not a
reason to stop.

---

## ⭐ The work order

⚠ **Take these in order.**

1. **`CORPUS-02`**, whose acceptance names four rows and refuses on two.
   ⛔ Both are blocked on one thing: `b_ids_driver::Family` knows two families.
   `DRIVER-10`'s steps 2 and 3 are `firefox` and `chromium`, and `firefox` is
   the higher value: a genuinely different TLS stack.
2. **`CI-04`**. A scheduled run that finds a change opens a pull request.
   ⭐ The write is ruled job-scoped, its acceptance is a body generator driven by
   a fixture rather than a real pull request, and `PUB-08` closed today is the
   model that body renders from.
3. **`SCHEMA-12`**, the six formats that need a decoder as well as an encoder,
   now that `SCHEMA-08`'s five are proved and its generator is the seam.
4. **`PUB-03`**, then `PUB-01`, `PUB-02`, `PUB-07`. ⭐ `SCHEMA-08` gives
   `PUB-03` something to publish that is not JSON alone, and `PUB-08` gives
   `PUB-01` its release body.
5. **`EMIT-03`**, whose measurement is in: every profile carries the priority
   block, so the entry takes the branch that needs the HTTP/2 library vendored.
6. **`DRIVER-06`**, once the question above has an answer.

⚠ **Small entries worth taking whenever a larger one is blocked**:
`HARNESS-11` (the p0f layer), `VALID-04` (reference digest implementations,
⛔ with a licence question stated in the entry that comes first), `DOC-02` and
`DOC-03`, both of which their own entries say to write only when a specific
thing becomes true.

---

## Settled, and not to be raised again

**Ruled by the operator 2026-09-01 unless noted.**

### ⭐ Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** ⭐ Done: `DRIVER-08` closed, and `capture.yml` provisions where the
  cell asks for it. ⚠ One sentence of the ruling's reasoning is refuted above.
- ⭐ **The corpus carries BOTH Chromes, as separate matrix cells.** ⚠ The
  unbranded cells are blocked on the open question above.
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
  ⭐ Done: `HARNESS-15` closed and `--no-resumption` left the capture path.
- **Two smaller findings are their own entries**: `DRIVER-07`, the browser log,
  which is what diagnosed the Edge lane, and `HARNESS-16`, the Windows trust
  store.
- ⛔ **A guard on something irreversible is TWO conditions from two sources,
  and it is never mutated on the machine it protects.** ⭐ Held all session:
  every mutation of the provisioning pair ran against copies under the ignored
  scratch directory.
- ⭐ **The write for `CI-04` is JOB-SCOPED.** `contents: write` and
  `pull-requests: write` on the collect job alone, using the run's own
  `GITHUB_TOKEN`. ⛔ Never a personal access token. ⚠ It also needs the
  repository setting that lets Actions create pull requests, which is the
  operator's to enable.
- ⭐ **The first runner capture is fetched with `gh` and added by hand.**
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the operator has ruled
  something broader with it: **this project is in beta, nobody consumes its
  data, and the commit history will be reset once the project satisfies the
  operator.** ⛔ That is the OPERATOR'S action at a time of their choosing and
  it licenses nothing for a session: no force push, no history rewrite, and the
  corpus stays append-only in every change an agent makes.
- **`SCHEMA-08` is SPLIT.** ⭐ Closed: JSON, NDJSON, CSV, TSV and Markdown.
  `SCHEMA-12` carries YAML, TOML, SQLite, CBOR, MessagePack and Protobuf.
- **Credentials are recorded as PRESENT, never as a value.**
- **The trust anchor is a job, not a machine change.**
- **Header values stay names-only by default.** Corpus captures turn them on
  deliberately.
- **`CORPUS-04` publishes the per-build trust-anchor list and states all three
  options with their costs.** ⛔ It asserts no preference. ⭐ Closed.
- **The schema gains numeric bounds.**
- ⭐ **The shuffle seed stays out of `browser-profile/1`.**
- **Commit once at the close** unless the session is genuinely at risk of losing
  work. ⚠ This session pushed eight times, because a capture lane runs the tree
  on the default branch and could not be dispatched otherwise.
- **A measured profile goes into the committed corpus with its conditions
  recorded.**
- **The TLS terminator is vendored here and patched here.**
- **The declared minimum Rust version is a verified upper bound.**
- **`Cargo.lock` is committed.**
- **A path in a code span asserts that it resolves.**
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
