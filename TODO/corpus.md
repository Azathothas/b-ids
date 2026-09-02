# corpus

The canonical data: how it is stored, what it must cover, and the policies that
decide what goes in.

⭐ Coverage is what makes this a product rather than a curiosity about one
browser. One browser is a weekend; the matrix is the thing anybody uses.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## CORPUS-01. Content-addressed, append-only, never edited in place

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** corpus, **Priority** P1, **Effort** M, **Status** done

### Problem

A corpus that is edited in place cannot be used as evidence. A consumer who
pinned a value has no way to tell whether it changed, and a reader has no way to
tell what it used to say.

### Premise

Believed. Reinforced by a measurement from the sweep: two published copies of
one dataset, both carrying the same version number and both naming the same
upstream, contain a different number of entries.
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
has it, and [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md)
section 9 draws the conclusion about what a version number does and does not
guarantee.

### Approach

- **The canonical corpus is committed on the default branch**, as JSON, one file
  per profile, reviewable in a diff. That is what makes the automated pull
  requests in `CI-04` reviewable: a human sees the profile change.
- **A profile is immutable once published.** A correction is a **new** profile
  carrying a `supersedes` field and a reason.
- **Raw captures are committed beside them**, per `SCHEMA-06`.
- Publish at least one index, and one latest-per-key pointer file. Sign the
  index, per `PUB-12`.

The layout, which is also what the routes in `PUB-03` are generated from:

```text
corpus/v1/chrome/stable/linux64/152.0.7977.64.json
raw/v1/chrome/stable/linux64/152.0.7977.64.hello.hex
```

Must not: edit a published profile, and must not delete a superseded one.

### Prove

```bash
sh scripts/common/check-corpus.sh
```

Passing means: every profile validates, every `supersedes` names a profile that
exists, no committed profile has ever been modified after its first commit, and
the check is run over the whole history rather than the working tree.

### Closing

**Closed 2026-09-01.** ⭐ **The corpus is not empty any more.** One profile,
measured off Chrome `151.0.7922.76` on this machine, with the bytes it was read
from beside it.

```text
$ sh scripts/common/check-corpus.sh
corpus ok: 1 profile(s), nothing edited after publication, index and
pointers agree with the tree.
rc=0
```

```text
$ cargo test -p b-ids-corpus corpus
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

⚠ **That block is the run this entry closed on, and the count has since moved to
24**: `CORPUS-03` added two tests to the same file later the same day. The
figure is left as it was measured rather than re-pasted, and the reason it no
longer matches is written here. Re-running it today reports 24 passed.

#### What was built, and where each of the three words in the title is enforced

| the word | what enforces it |
| --- | --- |
| **content-addressed** | `corpus/v1/index.json` carries the SHA-256 and the byte count of every published file. `Store::verify` recomputes both. A version number that does not pin its bytes pins nothing. |
| **append-only** | `Store::add` refuses a route that already holds a profile, and names `supersedes` as what to do instead. |
| **never edited in place** | ⛔ **Not a question the working tree can answer**, because an edited file and a file that was always that way are identical on disk. `scripts/common/check-corpus` asks git, over the whole history, with `--diff-filter=MDR`. |

The layout is the one the entry specified, and it is derived in one place,
`b_ids_corpus::route`:

```text
corpus/v1/chrome/stable/win64/151.0.7922.76.json
raw/v1/chrome/stable/win64/151.0.7922.76.hello.hex
```

#### ⭐ The first profile, and the conditions it was taken under

[`../experiments/10-first-profile.sh`](../experiments/10-first-profile.sh) is
the script that took it, so the run is repeatable rather than a transcript.
What it reported:

```text
chrome 151.0.7922.76 exited=false elapsed_ms=45048 profile_removed=true
b-ids-harness: 6 of 8 handshake(s) completed, from 7 accepted connection(s)
connections recorded: 7
7 connection(s): 1 cold, 4 resumed, 1 further cold, 1 abandoned
```

⭐ **That split is the inherited trap reproducing exactly**, on a different
browser build from the one it was measured on: the navigation opened seven
sockets, one carried no HTTP/2 at all, and four resumed.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8 has the
original reading. The profile is built from the cold one and nothing else is
averaged into it.

The conditions, which are part of the result:

| | |
| --- | --- |
| host | Windows 11 `10.0.26200.9168`, `x86_64`, native |
| browser | Chrome `151.0.7922.76`, stable, branded, headful |
| trust | ⚠ `spki-pin`. One key, for one launch, no trust store changed. |
| header values | recorded, deliberately. Below. |
| taken | 2026-09-01T08:26:33Z, from the harness's own clock at the moment the connection was accepted |

#### ⚠ Header values were recorded, and that is a decision rather than a default

The default capture shape is names only, and open question 3 says to keep it and
take a coherence capture with `--header-values` deliberately. A published
profile is that case: **four of the validator's eight checks read a header
value**, so a corpus profile taken under the default is one the validator cannot
check. Taken with values, three of them run and pass:

```text
$ b-ids-validator corpus/v1/chrome/stable/win64/151.0.7922.76.json
ok    version
ok    platform
ok    brand
SKIP  handshake -- deciding whether this hello came from a 151 build needs a per-build corpus to compare against, and none exists yet
ok    grease
SKIP  encoding -- the caller did not say what the consuming client can decode
SKIP  absence -- the caller named no target stack
ok    provenance
```

⚠ **The three skips are structural rather than a gap in this profile.** Two want
a caller to name a target stack, and one wants a second profile of the same
build to compare against. `CORPUS-02` is what produces the second.

#### ⭐ Open question 1 is answered, and it was answered by placement

The credential rule's third door is the decrypted first message a terminated
capture holds. The conversion writes it into `raw.connection_hex`, which is the
field `b_ids_schema::Raw::check` already scans for a `cookie` or `authorization`
header line, so **the refusal fires at the moment a capture becomes a profile**,
which is what the recommendation asked for. No new rule was written: the
existing one was routed through.

⛔ **The bytes are never edited to make the refusal go away.** A profile that
carries one is refused whole, and the operator decides what to do with the
capture.

#### What else this entry had to build, and why each was in the way

| | why it could not be skipped |
| --- | --- |
| `captured.trust`, a four-value enum | ⛔ A measurement carries its conditions. Every capture goes through a per-launch key pin, `HARNESS-10` exists to measure whether that changed the answer, and it has nothing to compare across unless every profile records it. `Profile::check` refuses `not-applicable` on a profile carrying both a hello and HTTP/2 frames, because those frames arrived inside the session that hello opened. |
| `captured.switches` | Every switch the browser was given is a condition of what was captured through it, and the driver already recorded them for nobody. |
| `Capture::at`, and `harness-capture/4` | `captured.at` is never optional, so something has to produce it. ⛔ Stamped by the thing that took the capture: a reader that stamped one later would record when it read the file. |
| `b_ids_schema::instant` | The formatter lives beside the checker that validates the shape. Two modules, one writing the format and one validating it, is two places for the format to be defined. |
| `RawFrame::wire_hex` | `raw.http2_frames_hex` needs the frames back as bytes, and the frame head layout already lives in `h2.rs`. ⛔ It writes the DECLARED length and the ARRIVED payload even where they disagree, because that is what the wire carried. |
| `sha256` moved to `bytes.rs` | It had two callers with unrelated jobs: the certificate pin and the content address. A digest computed in two places is two places for it to be computed differently, in the one field whose purpose is that two parties agree. |

#### ⚠ Two things the secret scan refused, and the rule was not widened

The first profile made `check-no-secrets --public` red, exactly as `TOOL-03`
predicted it would for a different shape. Two more narrow exclusions, both
halves, each by name or by path-and-shape:

- a hex run assigned to an identifier named `sha256`, which is a declared
  content address and the same shape as the `checksum` exclusion already there;
- a line under `corpus/` or `raw/` that is **nothing but** a quoted lower-case
  hex run and an optional comma, which is how pretty-printed JSON writes an
  element of `http2_frames_hex` and is the one place the field name is on a
  different line from the value.

⛔ **Mutation-proved, both halves, exit codes read unpiped.** A 48-hex value
planted inside the corpus profile under a field named `planted_token` is still
refused by both:

```text
$ sh scripts/common/check-no-secrets.sh --public
== a long hex identifier ==
corpus/v1/chrome/stable/win64/151.0.7922.76.json:3:  "planted_token": "deadbeef...",
⛔ 1 category/categories matched.
rc=1
```

⚠ The value above is abbreviated at the ellipsis for the same reason `TOOL-03`'s
block is: written out it is a credential-shaped string in a tracked file, which
is what the check refuses.

⭐ **And those bytes have a second gate**, which is what makes the array
exclusion acceptable: `Raw::check` decodes the recorded bytes and refuses the
profile if they spell out a credential header. The one class of credential that
could hide inside a frame array is the one already checked by the model itself.

#### ⛔ The history leg refused the derived files, and that was its own defect

⭐ **Found the moment a SECOND commit touched the corpus**, which is the earliest
it could have been found and is why it was not visible when this entry closed.
`check-corpus` reported:

```text
corpus check failed: 1 published file(s) modified, deleted or renamed after
their first commit.

commit af11973

M	corpus/v1/latest.json
```

⛔ **`index.json` and `latest.json` are DERIVED and change by construction.** They
are regenerated from the tree every time a profile is added, so a rule refusing
their modification would refuse the second profile this corpus ever gets. The
rule belongs to a published PROFILE and its raw sidecar, and to nothing else.

⚠ **Nothing goes unchecked by excluding them.** Their content is asserted by the
other leg, which re-derives both from the profiles and compares; a hand-edited
index is refused there and `CORPUS-03` carries that mutation.

⛔ **Mutation-proved in a throwaway repository, both halves, three directions:**

| what was committed | edits | exit |
| --- | --- | --- |
| the derived index and pointer, regenerated | 0 | ⭐ allowed |
| a published profile, edited in place | 1 | refused |
| a raw sidecar, deleted | 2 | refused |

#### ⚠ What is NOT in the first profile, and is not hidden

| | |
| --- | --- |
| `digests` | empty. Nothing in this tree computes JA3 or JA4 yet; `VALID-04` is the entry that does, with published test vectors. A digest from an unverified implementation would be a fabricated field. |
| `raw.record_layer` | `null`. The harness does not read the record layer as its own block, and deriving one here from bytes this crate happens to hold would be a derivation wearing a measurement's label. |
| a second platform, a second browser, a second build | `CORPUS-02`. One profile is not a matrix and this entry never claimed to be one. |

#### The gate

```text
$ pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
gate ok: 20 passed, but 1 SKIPPED on this host: check-twins
```

⚠ `check-twins` is skipped by `--fast`, and `check-corpus` is registered as a
pair in it. Both halves were also run directly, which is where the figure below
comes from rather than from the comparison:

```text
$ sh scripts/common/check-corpus.sh --json
{"schema":"check-corpus/1","corpus":true,"profiles":1,"edits":0,"problems":0}
sh exit=0
$ pwsh -NoProfile -File scripts/common/check-corpus.ps1 -Json
{"schema":"check-corpus/1","corpus":true,"profiles":1,"edits":0,"problems":0}
ps exit=0
```

⛔ **The history leg was mutation-proved in a throwaway repository**, because
this tree had nothing committed under `corpus/` to edit yet. A published profile
edited and committed a second time is refused by both halves, with `edits:1` and
exit 1.

---

## CORPUS-02. The capture matrix: browsers, channels and hosts

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** corpus, **Priority** P1, **Effort** L, **Status** open

### Problem

Coverage decides whether this project is useful. It also decides whether
automated merging is possible at all, because agreement across two independent
sources is only satisfiable when the same build is captured on more than one
host.

### Premise

Believed, and one measurement exists: one browser version on two platforms
produced the same digests. ⚠ That is one version and two platforms, and it is
nowhere near enough to conclude that the TLS half is platform-independent. The
matrix exists to answer that rather than to assume it.

### Approach

**Browsers**, ordered by value per unit of effort. Everything after the first is
cheaper than it looks, because the harness does not change: only acquisition and
the driver flags do.

| browser | why it earns a lane |
| --- | --- |
| Chrome | the reference. Most impersonated, best published acquisition path. |
| Edge | the same engine with a different brand list and User-Agent, so it isolates branding from engine at almost no cost |
| Chromium, unbranded | the control that proves which fields are branding and which are engine |
| **Firefox** | ⭐ a genuinely different TLS stack: different cipher list, different extension set and order, different HTTP/2 settings. The highest-value non-Chrome lane. |
| Firefox extended support | long-lived, widely deployed in enterprises, so it is what a lot of real traffic actually is, and nobody publishes it |
| Safari | one platform family, version welded to the operating system, hardest to automate, least served and most requested |
| Brave, Opera, Vivaldi | forks that change things on purpose. One job each, opportunistic. |
| mobile browsers | the most-requested and least-available data in the field. Emulator and simulator lanes, nightly rather than per push. |
| ⭐ browser-automation bundled builds | **the negative control**: what automation looks like, which is what the detection side needs. Nearly free, same pipeline. |

⭐ **The negative control is not an afterthought.** A corpus that only says what
a real browser sends is half a tool. Saying what a driven one sends is what lets
anybody tell the two apart.

**Hosts**: Linux on both common architectures, Windows on both, macOS on both
while both exist, a different libc or distribution in a container because that
is a real source of variance for a different TLS stack, and emulator lanes for
mobile.

**Channels**: stable, beta and canary are required for the Chromium family; dev
is opportunistic because the same index carries it. Release, beta, nightly and
extended support for Firefox. Release plus a preview where a runner has it for
Safari.

Must not: treat a lane as covered because a neighbouring lane passed. The point
of the matrix is that platform dependence is a measurement.

### Prove

```bash
sh scripts/common/check-coverage.sh --require-rows chrome,edge,chromium,firefox
```

Passing means: the coverage report lists every planned cell, marks each as
captured, failed or not attempted, and exits non-zero when a required row has no
capture at all.


### ⚠ Open, with the blocker named. What landed 2026-09-01

⛔ **Half of this entry is built and the other half needs a runner.** The
apparatus is here and no lane has run, so the entry stays open rather than
closing on machinery.

| what exists now | |
| --- | --- |
| ⭐ [`../.github/capture-matrix.json`](../.github/capture-matrix.json) | eight planned cells, three enabled, each with the reason it is or is not attempted yet, and each naming the `route` it gets its browser by. It is the ONE place the plan lives. |
| ⭐ `check-coverage`, both halves | every planned cell reported as `captured`, `absent` or `not-attempted`, with `--require-rows` for the caller's own assertion |
| ⭐ [`../.github/workflows/capture.yml`](../.github/workflows/capture.yml) | the fan-out, from `CI-03`, reading the plan above through `fromJSON` |

```text
$ sh scripts/common/check-coverage.sh
coverage over 6 planned cell(s):

  absent         chrome/stable/linux64              0 profile(s) required
  captured       chrome/stable/win64                1 profile(s) required
  not-attempted  edge/stable/linux64                0 profile(s)
  not-attempted  chrome/stable/macos-arm64          0 profile(s)
  not-attempted  chrome/beta/linux64                0 profile(s)
  not-attempted  firefox/stable/linux64             0 profile(s)

1 captured, 1 absent, 4 not attempted.
exit=0

$ sh scripts/common/check-coverage.sh --require-rows chrome,edge,chromium,firefox
coverage check failed, 3 required row(s) with no capture:

  edge: no capture at all, on any channel or platform
  chromium: no capture at all, on any channel or platform
  firefox: no capture at all, on any channel or platform
exit=1
```

⭐ **The acceptance command already refuses**, which is the half of this entry a
host with no runner can prove.

### ⛔ What would close it

**One run of `capture.yml` on a hosted runner, and the `linux64` profile
committed.** That needs the workflow on the default branch, which needs this
session's commit pushed, which is why it is the first thing the next session
does rather than something waiting on a decision.

⚠ **The `win64` cell already reads `captured`, and that profile came from a
laptop rather than a runner.** ⛔ It is not a second source: `CI-04`'s
merge condition wants agreement across two INDEPENDENT sources, and one laptop
capture is one source. ⭐ The single highest-value capture available is still
two profiles of ONE build on TWO platforms, and `VALID-01`'s handshake check
reports `NotCheckable` until it exists.

### ⚠ A defect this found in its own tooling, on this host

⛔ **jq on Windows writes CRLF**, so the plan's third field arrived as `true`
followed by a carriage return, which is not `true`. The sh half's human report
dropped the word `required` from every required row while its JSON, which does
not carry that field, matched its PowerShell twin exactly. ⚠ A divergence the
twin comparison structurally could not see, found by reading the two human
outputs side by side. Both jq reads strip the carriage return now.

### ⭐ 2026-09-02: the matrix ran, and it found why a lane can capture nothing

⭐ **`capture.yml` has run on hosted runners and every job passed.** Run
`33579619515`, dispatched with the authenticated `gh` on the default branch:
the plan job, both browser lanes, the fuzz lane and the collect job, all green.
That is the first time the capture matrix has ever run.

| what the runners served | |
| --- | --- |
| `ubuntu-latest`, image `ubuntu24/20260823.283` | Chrome `151.0.7922.173`, and ⭐ **Edge `151.0.4129.101` also resolved**, at `/usr/bin/microsoft-edge` |
| `windows-latest`, image `win25-vs2026/20260824.214` | Chrome `151.0.7922.174` |

⛔ **The two runners do not carry the same Chrome build**, so two profiles of
ONE build on TWO platforms is not obtainable from the preinstalled browser. It
needs pinned acquisition, which is `DRIVER-05`'s route rather than this lane's.
⚠ The premise above says one browser version on two platforms produced the same
digests; nothing here confirms or refutes it, because no version was shared.

#### ⛔ The `linux64` lane captured nothing, twice, and the reason is resumption

⚠ **Reproduced rather than raced.** Runs `33579619515` and `33580371329`
produced the same shape: Chrome on `ubuntu-latest` abandoned both of the
connections that were not resumed and resumed every one it kept, so the
navigation had **no cold connection** and there was nothing to publish. More
connections do not help, because the first completed handshake leaves a ticket
and everything after it resumes.

⛔ **And the report said `1 cold` on the line above the refusal saying there was
none.** `b-ids-corpus add` carried the word behind a hardcoded `1` in its format
string: the "a hardcoded or synthetic status, progress or metric" row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md).
⭐ Fixed by moving the line to `Selection::report`, where a test can reach it,
and the guard was seen to fail: reverting `cold_count` to a literal `1` takes
`connection_selection_reports_no_cold_connection_when_every_one_resumed` red at
exit 101.

#### ⭐ The fix, and the control that says it is safe

`b-ids-harness --no-resumption` issues no session tickets, so the subject cannot
resume and every hello is a cold one. ⚠ **`experiments/10-first-profile.sh`
stopped passing it on 2026-09-02**, when `HARNESS-15` made the two halves
selectable independently: the switch is a CONTROL for
`experiments/30-resumption-control.sh` and it is no longer a condition every
published profile is taken under. The harness still **reports** the
configuration on stderr and the script still reads that line back into
`captured.resumption` rather than typing it.

⭐ **Measured, not argued.** `experiments/30-resumption-control.sh`, three
rounds on this Windows host against Chrome `151.0.7922.76`, headless:

```text
offered: 4 cold, 11 resumed, 3 abandoned
refused: 15 cold, 0 resumed, 3 abandoned
modes=agree differing:0 not_comparable:2 fields:19
```

⚠ The two not-comparable fields are `tls.cipher_suites` and
`tls.extensions.order`, both of which carry a per-connection GREASE draw or
shuffle, and `b_ids_harness::modes` reports those as not comparable rather than
as findings. ⭐ **So the switch changes WHICH connections are cold, not WHAT a
cold hello is**, which is the claim the code makes about itself.

#### What the corpus holds now

⭐ **Three profiles, and two of them were captured on machines nobody owns.**

| route | where it came from | `captured.resumption` |
| --- | --- | --- |
| `chrome/stable/win64/151.0.7922.76` | a laptop, 2026-09-01 | absent: the field did not exist |
| `chrome/stable/win64/151.0.7922.174` | `windows-latest`, run `33579619515` | ⚠ absent: that run predates the switch, and the harness reported nothing to read back. ⛔ Not filled in afterwards; a condition nobody read is not a condition somebody measured. |
| `chrome/stable/linux64/151.0.7922.173` | `ubuntu-latest`, run `33582975294` | `refused` |

```text
$ cargo run -q -p b-ids-corpus -- verify --root .
corpus=profiles:3 problems:0

$ cargo run -q -p b-ids-corpus -- validate --root .
corpus=validate profiles:3 findings:0 notcheckable:9
```

⭐ **The `linux64` lane publishes now**, and the run that did it reported
`1 cold, 0 resumed, 6 further cold, 1 abandoned` where the two runs before it
reported `0 cold`.

#### ⭐ The matrix's `browser` column reaches the driver now

⛔ **It reached nothing until 2026-09-02.** `b-ids-driver drive` took
`browsers.first()`, so an `edge` lane would have driven Chrome, and the capture
script wrote the literal `"Chrome"` into every identity file it produced. The
corpus derives a route by lower-casing that name, so the lane would have
published Chrome under Chrome's route while the artefact was called `edge`.

⭐ What changed, in one direction each so nothing is decided twice:

| | |
| --- | --- |
| `b-ids-driver --browser NAME` | selects the family for `resolve` and for `drive`. ⛔ A name with no branch is refused and names what it knows; a family this machine lacks exits **2**, because "no browser here" and "the capture failed" are different facts. |
| `Family::vendor_name` | the spelling a profile records, derived rather than typed. A test asserts it lower-cases to the family, which is what the route is built from. |
| `Resolved.name` | reported in the driver's JSON, so the capture script reads it instead of carrying its own table. |
| `capture.yml` | both the resolve step and the capture step pass `${{ matrix.browser }}`. |
| the plan file | `edge/stable/linux64` is **enabled and required**, with the measurement that unblocked it written into its `why`. |

#### ⛔ What still blocks this entry

The acceptance command names four rows:

| row | state |
| --- | --- |
| `chrome` | ⭐ captured, on `linux64` and `win64`, from runners |
| `edge` | ⭐ **captured 2026-09-02**, `151.0.4129.101` on `linux64`, provisioned from the vendor's enterprise index rather than taken from the image. `DRIVER-10` is the entry and its closing carries the run. |
| `chromium` | ⛔ `b_ids_driver::Family` has two variants, `Chrome` and `Edge`. Nothing can resolve Chromium, so no lane can produce it. |
| `firefox` | ⛔ the same. `VALID-03` is the check that says so from the corpus side. |

⚠ **So two of the four required rows are captured and two are blocked on the
same thing**: `b_ids_driver::Family` knowing the family at all. That is
`DRIVER-10`'s steps 2 and 3, and it is the whole of what stands between this
entry and its acceptance command.

```text
$ sh scripts/common/check-coverage.sh --require-rows chrome,edge,chromium,firefox
coverage check failed, 2 required row(s) with no capture:

  chromium: no capture at all, on any channel or platform
  firefox: no capture at all, on any channel or platform
exit=1
```

⭐ **The refusal is down from three rows to two**, which is the half of this
entry that moved on 2026-09-02.

---

## CORPUS-03. `latest` means stable, and beta is how the project gets ahead

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** corpus, **Priority** P2, **Effort** S, **Status** done

### Problem

A consumer following a pointer called `latest` must never be handed a
pre-release build. That is the same failure as shipping a version nobody runs
yet, and consumers will assume otherwise unless it is stated.

### Premise

Believed, and the mechanism is already available: the automation-build index is
addressable **by channel**, which is the property that lets this project be
ahead rather than perpetually behind.

### Approach

Three rules, and ⛔ **they live in [`../README.md`](../README.md)**, because a
consumer will not read this file and that is where they are aimed:

- `latest` means stable and nothing else, with the pre-release channels
  published beside it under their own names;
- capturing beta and canary is the mechanism that gets this project ahead of a
  release rather than perpetually behind it;
- historical versions are out of scope, because the corpus accretes forward.

⚠ **This paragraph used to restate all three here as well, and the tree's own
check refused it.** `scripts/common/check-one-home.sh` found two sentences in
both documents, which is the rule that one fact lives in one place. The
restatement is a pointer now and the README carries the wording. ⭐ The
correction is recorded rather than made silently; the closing has it.

Must not: promote a beta profile into the stable path when it ships. It is a
different capture of a different build; capture the stable build.

### Prove

```bash
sh scripts/common/check-routes.sh --assert-latest-is-stable
```

Passing means: every `latest` route resolves to a profile whose channel is
stable, and a fixture corpus in which one does not fails with a message naming
the route.

### Closing

**Closed 2026-09-01.** ⭐ **`latest` cannot name a pre-release build, and it
cannot because of how it is built rather than because something checks it
afterwards.**

```text
$ sh scripts/common/check-routes.sh --assert-latest-is-stable
routes ok: 1 single-value file(s), none ends with a line ending, and every latest pointer names a stable profile
rc=0
```

```text
$ cargo test -p b-ids-corpus corpus
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

#### ⭐ The rule is unrepresentable rather than tested for

The pointer file carries two maps now, and the split is the whole design:

```json
{
  "schema": "corpus-latest/2",
  "latest": { "chrome/win64": "corpus/v1/chrome/stable/win64/151.0.7922.76.json" },
  "per_channel": { "chrome/stable/win64": "corpus/v1/chrome/stable/win64/151.0.7922.76.json" }
}
```

`Store::pointers` builds `latest` from stable profiles alone, so the derivation
has no way to put a beta build in it. `Store::verify` then compares the written
file against the derivation, so a hand-edited pointer file is refused as well.
⭐ **A class of defect that cannot be represented is stronger than one that is
tested for**, and the test below covers the one path that remains: somebody
editing the published file.

⚠ **The schema is `corpus-latest/2`.** Version 1 had one map keyed by channel
and left what `latest` means undecided. A field added or repurposed without a
version bump is a positional format that mis-reads silently.

#### ⛔ Mutation-proved, and the fixture is a corpus where the beta is NEWER

The test corpus holds stable `152.0.7977.64` and beta `153.0.8010.12`, so a
pointer that took the newest of everything would take the beta. It does not.

Editing the published file to point `latest` at the beta is refused twice, by
two different questions:

```text
latest/chrome/win64 resolves to corpus/v1/chrome/beta/win64/153.0.8010.12.json, whose channel is beta. A pointer called latest means stable and nothing else
```

⭐ **And `check-corpus` refuses it too**, because the written file no longer
matches what the tree derives to. Two independent refusals of one edit is what
the construction-plus-comparison design buys.

#### ⚠ The entry asked for the rules in two places, and this tree refuses that

⛔ **A finding against this entry's own Approach.** It said the three rules "go
in the README as well as here because a consumer will not read this file". Doing
that put two twelve-word sentences in both documents, and
`scripts/common/check-one-home.sh` refused the tree: one fact lives in one
document.

⭐ **The README owns the wording**, because that is where a consumer is; the
Approach above is a pointer now, and the amendment is recorded there rather than
made silently. ⚠ The instinct behind the original wording was right and the
mechanism was wrong: a fact that two audiences need is pointed at twice, not
written twice.

#### What is not covered

| | |
| --- | --- |
| a beta or canary profile in the real corpus | There is one profile and it is stable. `CORPUS-02` is the matrix, and the beta lane is what makes the second map carry anything. |
| a `latest` that resolves over HTTP | These are routes inside the tree. `PUB-03` is the entry that serves them, and it generates from the same pointer file rather than from a second answer. |
| nightly | Named in the rule and absent from the vocabulary's use here, because no browser this project captures publishes one under that name yet. `Channel` carries it. |

---

## CORPUS-04. Per-build trust-anchor lists, and a recommendation

**Source** the founding brief; the two codepoints are [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 3
**Category** corpus, **Priority** P2, **Effort** M, **Status** open

### Problem

One extension carries a snapshot of the browser's own root store, so a client
carrying one build's list is advertising which build it copied. What a client
with no store of its own can do instead is a genuine trade with three answers,
and [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 3
states all three.

### Premise

Measured elsewhere and inherited: the codepoint, the length and the body shape
are in [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 3.
⚠ The **name** attached to that codepoint is inferred rather than read against a
specification, and the entry says so.

### Approach

Two deliverables:

- **publish the list per build, with its capture date**, as its own artefact
  rather than buried in a profile, because it changes on a different schedule
  from everything else;
- **write the recommendation**, with the trade stated rather than a preference
  asserted: omit it and be one extension short of a real browser; carry a
  captured list and be honest the day it was captured and a fingerprint of that
  day afterwards; send it empty and produce a shape no browser sends.

⭐ **Nobody currently provides this and every impersonating client will need
it.**

Also settle the inferred name: read the draft specification against the bytes.
That is one afternoon and it removes an inferred claim from the tree.

Must not: state the recommendation as settled. It is a trade, and the entry's
job is to make the trade legible.

### Prove

```bash
sh scripts/common/check-trust-anchors.sh
```

Passing means: every profile carrying that extension has a corresponding
published list with a capture date, and the published document states all three
options with the cost of each.

---

## CORPUS-05. Name the unidentified extension

**Source** the founding brief; the two codepoints are [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 3
**Category** corpus, **Priority** P3, **Effort** S, **Status** done

### Problem

One extension codepoint observed in a shipped browser is unidentified. It is two
zero bytes and trivially reproducible, so nothing is blocked by it, but an
unnamed field in a published corpus is a question every consumer will ask.

### Premise

Measured elsewhere and inherited: codepoint, length two, body two zero bytes,
seen at position seven in one capture.
[`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 3.

### Approach

Search the browser engine's source for the codepoint, read the specification
drafts registered near it, and record what it is, with the evidence. If it
cannot be named, record that it could not be named and what was searched, so the
next attempt does not repeat the search.

⭐ **It is recorded either way**, which is why it can be identified later by
somebody who is not this project. An extension nobody can name still gets its
codepoint, its length and its body kept verbatim.

Must not: guess a name from a codepoint's neighbours. That is how the other
extension in that capture acquired an inferred name that this tree now has to
carry as inferred: its body is measured and its name is not.

### Prove

```bash
sh experiments/60-identify-extension.sh
```

Passing means: the script records what was searched and what it found, and
either the extension is named with a citation or the search is recorded as
exhausted with a list of what was ruled out.

### ⚠ The acceptance names `60-` rather than `30-`, and the reason is a rule

⛔ **A number is never reused and `30-` was taken** by
[`../experiments/30-resumption-control.sh`](../experiments/30-resumption-control.sh),
written after this entry was authored.
[`../docs/methodology/experiments.md`](../docs/methodology/experiments.md) says a
citation has to keep meaning what it meant, so the script is
[`../experiments/60-identify-extension.sh`](../experiments/60-identify-extension.sh)
and the Prove block above is corrected rather than the file misnumbered to match.

### Closing

**Closed 2026-09-02T04:30:00Z.** The search ran, is recorded, and is re-runnable.
⛔ **The extension is NOT named**, and what was searched is written down so the
next attempt does not repeat it.

```text
$ sh experiments/60-identify-extension.sh
searching for extension 0x12e0 (4832 decimal)

-- what this project measured itself --
  absent   chrome-151.0.7922.173-linux64-stable
  absent   chrome-151.0.7922.174-win64-stable
  absent   chrome-151.0.7922.76-win64-stable
  0 of 3 published profile(s) carry it

-- reference captures whose extension list carries it --
  references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json

-- verdict --
  ⛔ NOT NAMED. No specification was read against these bytes, and the
     browser engine source is not a tree this project keeps.
exit=0
```

### ⭐ The search produced a measurement, which the entry did not expect

⛔ **Chrome `151` does not send `0x12e0`, on either platform, in any of the three
profiles this project has captured.** The origin's capture of Chrome
`152.0.7977.64` on `linux64` does: its `ja4_r` extension list reads
`...,0033,12e0,44cd,ca34,fe0d,ff01`, at
[`../references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json`](../references/Azathothas__bit-cli/tree/bench/browser-fingerprint-cft-152.json).

⭐ **So it is a codepoint Chrome added between 151 and 152**, which narrows the
search from "somewhere in an engine" to "a change in one release". ⚠ The same is
true of `0xca34`, the inferred trust-anchor extension: present in the 152
capture, absent in all three 151 profiles here.

⚠ **This is a measurement of ABSENCE and it is worth what an absence is worth.**
It says Chrome 151 does not send the codepoint. It does not say Chrome 152 does:
that is a reading of somebody else's capture at a named commit, and this project
has captured no 152 build.

### ⛔ What the script is allowed to conclude, and what it is not

| it searches | what that is |
| --- | --- |
| every profile under `corpus/v1/` | ⭐ a **measurement**: whether the codepoint was on a wire this project read |
| every tracked reference tree, three spellings | a **reading** of somebody else's repository at a named commit |
| the reference captures whose extension list carries it | the highest-signal reading there is, because it names the build |

⛔ **It does not guess a name from a neighbour.** That is how `0xca34` acquired
an inferred name this tree carries as inferred: its body is measured and its
name is not.

⚠ **The decimal spelling is low signal and is searched anyway.** Four digits
occur in unrelated JSON across the reference corpus, so a hit there is a place to
look rather than a finding. ⛔ Narrowing the search to make the output tidy would
be narrowing it to get the answer that fits.

### ⛔ Why the engine source was not searched

**A claim about a repository is not written until that repository is in
[`../references/`](../references/) at a named commit**, and a browser engine
checkout is not a tree this project keeps.
[`RULES.md`](RULES.md) section 3 is the rule and it cost this repository its most
expensive defect. ⭐ So the verdict names what would settle it: the engine source
at a named commit here, or a specification draft read against the recorded body.

