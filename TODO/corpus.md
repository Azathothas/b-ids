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
**Category** corpus, **Priority** P1, **Effort** L, **Status** done

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

### ⛔ 2026-09-04: the blocker above is disproved, and it has moved rather than gone

⚠ **The section above says two of the four required rows are blocked on
`b_ids_driver::Family` knowing the family at all.** ⛔ That is no longer true,
and the correction is written here rather than as an edit to it.

⭐ **The resolver knows all four families now.** Measured on this Windows host,
which has three of them installed:

```text
$ cargo run -q -p b-ids-driver -- resolve --json
{"family":"chrome","name":"Chrome","path":"C:\\Program Files\\Google/Chrome/Application/chrome.exe","version":"151.0.7922.76","answers":[["sibling-directory","151.0.7922.76"]],"disagreement":false}
{"family":"edge","name":"Edge","path":"C:\\Program Files (x86)\\Microsoft/Edge/Application/msedge.exe","version":"152.0.4191.62","answers":[["sibling-directory","152.0.4191.62"]],"disagreement":false}
{"family":"firefox","name":"Firefox","path":"C:\\Program Files\\Mozilla Firefox/firefox.exe","version":"148.0.2","answers":[["application-ini","148.0.2"]],"disagreement":false}
```

⛔ **Firefox needed a version source no Chromium layout has, and without it the
family would have been invisible while installed.** Measured in the install
directory 2026-09-04: no version-shaped sibling directory and no
`firefox.manifest`, so both existing sources answer nothing, and `resolve` drops
an executable it cannot version. `application.ini` states it, and
`Source::ApplicationIni` is what reads it.

| what changed | where |
| --- | --- |
| `Family::Chromium` and `Family::Firefox` | [`../crates/b-ids-driver/src/resolve.rs`](../crates/b-ids-driver/src/resolve.rs) |
| `Family::is_chromium`, which the launcher reads | the same file |
| `Source::ApplicationIni` and `from_application_ini` | the same file |
| candidate paths for both families, on Windows and POSIX | the same file |
| `index_route` returns `Option<Route>`, so it cannot disagree with `index_url` | [`../crates/b-ids-driver/src/acquire.rs`](../crates/b-ids-driver/src/acquire.rs) |
| `IndexRefusal::NoIndexForFamily` | the same file |
| the launcher refuses a non-Chromium family rather than passing it Chromium switches | [`../crates/b-ids-driver/src/drive.rs`](../crates/b-ids-driver/src/drive.rs) |
| a `chromium` cell, which the plan did not have while the acceptance required the row | [`../.github/capture-matrix.json`](../.github/capture-matrix.json) |

#### ⛔ Four tests used `firefox` as the example of an impossible family

⭐ **They went red on the change that fixed the gap they were written about**,
which is the most useful thing they could have done. An example chosen because
it is impossible stops testing anything on the day it becomes possible, and
nothing warns you.

| test | what it asserted |
| --- | --- |
| `resolve_and_drive_a_family_name_round_trips` | `Family::parse("firefox")` is `None` |
| `resolve_and_drive_browser_refuses_a_family_the_resolver_cannot_produce` | `resolve --browser firefox` exits 2 |
| `reachable_dimensions_a_family_the_resolver_cannot_produce_is_reported` | a `Firefox` profile is unreachable |
| `reachable_dimensions_every_profile_carrying_it_is_named` | two of them are grouped |

⚠ **All four now use `safari`**, which no branch produces on any host. ⛔ The
second one had a second defect: with `firefox` it would have become
HOST-DEPENDENT, answering 0 on a machine with Firefox installed and 2 on one
without, for a different reason each time.

#### ⛔ Two defects the runners found and this host structurally could not

⚠ **Both were pushed green from here and refused by CI**, which is the
strongest argument in this entry for the two-host gate.

**1. A Firefox version has TWO components, and a test asserted three.**
`resolve_and_drive_reports_a_build_from_a_source_it_names` required a version of
at least three dot-separated parts. Both runners ship Firefox `154.0`, and both
panicked with `154.0 is not a build`.

⛔ **It passed here for a reason that is pure luck**: this host carries
`148.0.2`, a point release. ⭐ The assertion was a Chromium assumption wearing a
general name, and `b_ids_schema::check_version` had the right rule all along:
fewer than two numeric components is refused, and two is accepted. The test
reads the schema's rule now, and a check confirmed it still refuses `154`, `""`,
`unknown`, `154.`, `154.0-beta` and `Firefox 154.0`.

**2. Two shellcheck versions disagreed about one line.** `A && B || C` in the
doctor's new fixture is `SC2015`; this host's shellcheck 0.11.0 passed it and
the `ubuntu-latest` runner's refused it. ⛔ The construct is avoided rather than
argued about, because it is genuinely not an if-then-else.

#### ⚠ Where the blocker actually is now, measured

⛔ **`firefox` is blocked on the LAUNCHER, and `chromium` on ACQUISITION.**
Neither is the resolver any more.

Measured from `firefox --help` on 148.0.2, this host, 2026-09-04: `--profile`
and `--headless` exist and there is **no certificate switch of any kind**. The
harness is a TLS terminator, so a Chromium capture is arranged with
`--ignore-certificate-errors-spki-list`; Gecko offers no command-line
equivalent, and the trust has to be arranged inside the profile instead.
⭐ `DRIVER-11` is that entry and it carries the measurement.

Chromium is blocked on there being no vendor channel serving a build
addressable by the version a profile records, which `DRIVER-10` measured and
which is why its cell says the outcome may be a recorded refusal.

#### The acceptance, still refusing, and now on the honest ground

```text
$ sh scripts/common/check-coverage.sh --require-rows chrome,edge,chromium,firefox
coverage over 9 planned cell(s):

  captured       chrome/stable/linux64              2 profile(s) required
  captured       chrome/stable/win64                3 profile(s) required
  captured       edge/stable/linux64                1 profile(s) required
  absent         chrome/for-testing/linux64         0 profile(s)
  absent         chrome/for-testing/win64           0 profile(s)
  not-attempted  chrome/stable/macos-arm64          0 profile(s)
  not-attempted  chrome/beta/linux64                0 profile(s)
  not-attempted  firefox/stable/linux64             0 profile(s)
  not-attempted  chromium/stable/linux64            0 profile(s)

3 captured, 2 absent, 4 not attempted.

coverage check failed, 2 required row(s) with no capture:

  chromium: no capture at all, on any channel or platform
  firefox: no capture at all, on any channel or platform
exit=1
```

⛔ **This entry stays open.** Two required rows have no capture, and a row is
captured when a profile exists rather than when a code path does. ⚠ The report
lists nine cells rather than eight because the `chromium` row the acceptance
requires had no cell in the plan at all until today.

---


### ⛔ Ruled by the operator 2026-09-04: a session may dispatch captures and merge the lanes

⭐ **The last thing between this entry and its acceptance is captures, and a
session may now take them.** Dispatch `capture.yml` on this repository's own
remote, let each lane open its pull request, and merge the green ones.

⛔ **It is also the only route that fixes the published `HeadlessChrome`
User-Agent.** The corpus is append-only, so `CORPUS-06`'s normalisation reaches
only the NEXT capture: one run per enabled cell is what produces profiles at new
versions carrying the normalised value. ⚠ Nothing published is edited.

⚠ **Every lane already fails alone**, so a lane that cannot capture reports
rather than breaking the run, and `check-coverage` moves the row from
`not-attempted` to `absent` rather than to `captured`.

⛔ **The two rows this entry needs are still blocked on code, not on
permission**: `firefox` on `DRIVER-11`'s launcher and `chromium` on an
acquisition route. Dispatching a lane for a family the driver cannot drive
produces an honest failure, not a profile.

### ⛔ 2026-09-04: the matrix had never added a profile, and that is why this entry looked built

⚠ **Half of this entry was called "built" and the built half did nothing.** Six
lanes ran green on 2026-09-04, run `33849934489`, six captures were actually
taken, and the run reported:

```text
corpus=pull-request requests:0 auto:0
```

⛔ **The cause: nothing ever ran `b-ids-corpus add`.**
[`../experiments/10-first-profile.sh`](../experiments/10-first-profile.sh)
deliberately writes nothing into the corpus, because the corpus is append-only
and a profile in it is permanent, so the write is a deliberate act rather than a
side effect of measuring. ⭐ That rule was applied to the SCRIPT on 2026-09-02
and never to the LANE that runs it, so the deliberate act had nowhere to happen:
each lane uploaded the checkout unchanged, `collect` merged nothing over
nothing, and the run reported success.

⚠ **It is the "step that exits 0 having done nothing it was asked to do" row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md),
at the scale of a whole workflow**, and it had been that way since the workflow
was written.

#### What the lane does now

| | |
| --- | --- |
| the add | between the capture and the upload, so the lane's artefact carries the profile it took |
| the channel and the branded flag | ⛔ from the CELL, not from the script's laptop defaults. A `for-testing` build is UNBRANDED, and a lane taking the default would publish an unbranded build labelled branded, which is a wrong value rather than a missing one |
| the operator | the run that took it, because "who or what took it" on a runner is the run |
| the method | `vm`. A hosted runner is a virtual machine, and `captured.method` is a condition rather than a label |
| an already-published build | ⛔ reported and NOT a failure. The store refuses to overwrite a published profile by design, so a lane whose runner serves a build the corpus already holds would otherwise go red on a state that is correct. ⚠ Any other refusal is still red |

#### ⭐ What run `33851238648` then produced

```text
profiles collected: 59
profile files before=9 after=14
corpus=pull-request requests:5 auto:3
```

Five profiles, merged after being verified locally rather than on the
generator's word:

| | |
| --- | --- |
| `firefox-154.0.1-linux64-stable` | ⭐ **the first Gecko capture on a runner.** `trust: trust-store`, from the certificate database `DRIVER-11` writes into the throwaway profile |
| `chrome-151.0.7922.76-linux64-for-testing` | ⭐ **the first unbranded build in the corpus**, and the first profile carrying a real `captured.acquisition`: a route, a URL and a digest |
| `chrome-151.0.7922.76-win64-for-testing` | the same build on the other platform |
| `chrome-152.0.7977.82-linux64-stable` | the vendor route, with the archive's digest recorded |
| `chrome-152.0.7977.83-win64-stable` | ⚠ and it is a DIFFERENT build from the linux one an hour apart, which is the measurement the cell's own note predicts |

⭐ **Six of nine planned cells are captured and none is absent**, up from three
captured and three absent. ⚠ The `for-testing` lane had never run before today.

#### ⛔ Three more defects, each found by the profiles actually landing

⚠ **The index was taken from a lane rather than derived over the union.** Every
lane rewrites `index.json` and `latest.json` when it adds its own profile, so
each carries only its own view and `collect` kept whichever it copied last. The
merged tree failed `b-ids-corpus verify` with `index.json does not match what the
corpus derives to`. ⭐ `collect` re-derives them now, because an aggregate is a
function of the profile set.

⚠ **The trust-anchor check called a measurement a defect.** An unbranded Chrome
for Testing build sends codepoint `0xca34` with a two-byte body, `0000`, which is
a list of length zero; the branded build beside it sends 206 bytes and 32
identifiers. ⛔ The check reported "no identifiers" and went red on the finding
the `chromium` control cell exists to produce. It distinguishes an empty list on
the wire from a decode that produced nothing now, and reports the count.

⚠ **A published profile needs a JA4 vector and nothing in the pipeline derives
one.** Five new profiles failed
`digest_vectors_every_capture_vector_matches_the_profile_it_names` until five
vectors were derived by hand with `jq` and `sha256sum`. ⛔ That derivation is
deliberately NOT this project's Rust, so automating it inside the corpus tool
would defeat the vector; a tracked derivation script is what it needs, and it is
recorded in [`PROGRESS.md`](PROGRESS.md) with a recommendation.

⚠ **And one about the pull requests themselves**: five branches were opened, one
per route, and all five carry the identical tree. Measured: every branch resolves
to tree `97248d83821e0d13bf4860a6074399938614cd22`. ⛔ A pull request titled for
one route whose diff carries five is a title a reviewer cannot act on.

⚠ **None of the five could get its required checks.** The forge does not run
workflows on a pull request created with the run's own token, so every one was
`MERGEABLE` and `BLOCKED` with no checks at all. ⭐ They were re-derived locally
instead, which is what
[`../docs/security/remote-ops.md`](../docs/security/remote-ops.md) asks for
anyway: fetched, compared against the default branch, read, merged, and then
`b-ids-corpus verify` and `validate` run over the result.

#### ⛔ Still open, and the blocker is one row

```bash
sh scripts/common/check-coverage.sh --require-rows chrome,edge,chromium,firefox
```

```text
6 captured, 1 absent, 2 not attempted, 1 outside the plan.

coverage check failed, 1 required row(s) with no capture:

  chromium: no capture at all, on any channel or platform
```

⚠ **Exit 1, read from the process, unpiped.**

#### ⭐ And the chromium blocker is MEASURED now rather than predicted

⛔ **The cell was enabled, dispatched, and read.** `capture.yml` run
`33854002345`, 2026-09-04, on `ubuntu-24.04`:

```text
{"family":"chromium","name":"Chromium","path":"/usr/bin/chromium","version":"151.0.7922.0","answers":[["version-flag","151.0.7922.0"]],"disagreement":false}
resolve exit=0
```

⭐ **So the cell is NOT blocked on resolution**, which is what the previous note
implied. The resolver finds it and reads a version from it. The launch is where
it ends:

```text
Received signal 6
#12 0x55aa97e7d6e3 content::ZygoteHostImpl::Init()
#13 0x55aa99a0e2e9 content::ContentMainRunnerImpl::Initialize()
```

⚠ **That is the snap sandbox refusing to start a zygote on that image**, and it
answers the question `DRIVER-10` left open in as many words: whether a snap can
be driven. On this image, at this version, it cannot.

⛔ **The cell is disabled again on the same day.** A lane that resolves a browser
and then cannot launch it goes red on every run, and a red lane nobody can fix
trains everybody to ignore the workflow.

⛔ **What must not be done about it: pass `--no-sandbox`.** That captures a
browser in a configuration nobody runs, which is the derived value this project
refuses wearing another costume.

⚠ **What would open it** is an acquisition route serving a real Chromium build
addressable by the version a profile records. That is `DRIVER-10`'s question
rather than this cell's, and it is the one thing between this entry and its
acceptance command.

⛔ **REFUTED THE SAME DAY, BY THE CAPTURE THIS PARAGRAPH ASKED FOR. The original
wording is kept below and it is WRONG**; the correction is in the closing
section of this entry and in
[`../docs/HISTORY/stale-documents.md`](../docs/HISTORY/stale-documents.md).

> ⚠ **And one thing the `for-testing` captures already settled that this cell was
> for.** The unbranded build publishes an EMPTY trust-anchor list and the branded
> one publishes 32 identifiers, so the bundled root store is branding rather than
> engine. ⛔ That does not retire the row: the two for-testing builds are 151 and
> the two stable ones are 152, so the comparison confounds the major with the
> branding, and a chromium capture beside a chrome one of the same major is what
> separates them.

⚠ **What was wrong with it was a confound, not the reading.** The empty list came
from a Chrome **for Testing** build, and the Chromium capture taken hours later
is equally unbranded and sends a full 206-byte list.

---

### ⭐ 2026-09-04, second session of the day: the acceptance command exits 0

```bash
sh scripts/common/check-coverage.sh --require-rows chrome,edge,chromium,firefox
```

```text
coverage over 10 planned cell(s):

  captured       chrome/stable/linux64              3 profile(s) required
  captured       chrome/stable/win64                4 profile(s) required
  captured       edge/stable/linux64                1 profile(s) required
  captured       chrome/for-testing/linux64         1 profile(s)
  captured       chrome/for-testing/win64           1 profile(s)
  not-attempted  chrome/stable/macos-arm64          0 profile(s)
  not-attempted  chrome/beta/linux64                0 profile(s)
  captured       firefox/stable/linux64             1 profile(s) required
  captured       firefox/stable/win64               2 profile(s)
  captured       chromium/stable/linux64            1 profile(s) required

8 captured, 0 absent, 2 not attempted, 0 outside the plan.
```

⚠ **Exit 0, read from the process, unpiped.** Eight of ten cells captured, none
absent, none outside the plan, and the two not attempted are the two whose
blockers this entry has always named: no runner image ships a beta channel, and
`DRIVER-05` has no route serving a macOS build.

#### ⛔ What unblocked it: three routes measured, and the third answers

⚠ **`--no-sandbox` was refused, as the note above requires.**

| route | measured | verdict |
| --- | --- | --- |
| the image's own `chromium` on `ubuntu-24.04` | `capture.yml` run `33854002345`: the resolver finds `/usr/bin/chromium` and reads `151.0.7922.0`, and the launch aborts on signal 6 inside `content::ZygoteHostImpl::Init` | ⛔ shut. It is a snap, and its zygote will not start on that image |
| Google's `chromium-browser-snapshots` | `Linux_x64/LAST_CHANGE` answers `1692381` and the zip beside it is `200` at 246,778,957 bytes | ⛔ shut for THIS row. It serves a real unpatched Chromium and it is keyed by a trunk revision, which is not the version a profile records, and a continuous trunk build belongs to no channel anybody runs |
| ⭐ an APT archive publishing `chromium` for `noble` | `Packages` is `404` and `Packages.gz` is `200`; the index names `152.0.7977.75-1xtradeb1.2404.1` with a `SHA256` and a `Size`, and the artefact it points at is `200` at exactly the `93784166` bytes the index states | ⭐ **open.** It answers by version and publishes a digest |

⭐ **And the build it serves is the one this corpus already holds branded.**
`chrome/stable/linux64/152.0.7977.75` was already published, so the control this
row exists for is a SAME-BUILD pair rather than a same-major one.

⛔ **The route is `chromium-ubuntu-ppa`**, a sixth entry in the vocabulary
`b_ids_schema::ACQUISITION_ROUTES`, `b_ids_driver::acquire::Route` and the
published JSON schema all state. ⚠ It is named for the archive rather than for
the format, because a consumer reading it should learn that this was a
distributor's repackaging and not a vendor build.

#### ⭐ The measurement the row exists for, taken

`capture.yml` run `33882426404`, every lane green, one pull request against the
source branch, merged after `verify` and `validate` were re-derived locally.
Comparing the two profiles at build `152.0.7977.75` on `linux64`, with the
per-connection GREASE draw removed because it is a draw:

| | branded Chrome | Chromium |
| --- | --- | --- |
| TLS extension set | `5,10,11,13,16,18,23,27,35,43,45,51,17613,51764,65037,65281` | ⭐ **identical** |
| cipher suites, in order | `4865,4866,4867,49195,49199,49196,49200,52393,52392,49171,49172,156,157,47,53` | ⭐ **identical** |
| the HTTP/2 half | settings, window update and priority block | ⭐ **identical** |
| header set and order | thirteen fields | ⚠ the same thirteen, shifted by one: Chromium sends `cache-control` FIRST |
| the root-store extension `0xca34` | 206 bytes, body opening `00cc0582df13` | ⚠ **206 bytes, body opening `00cc04d67909`** |

⭐ **So branding changes NOTHING in the TLS half or the HTTP/2 half at this
build.** That is the answer this row was added to get, and it is the first time
this corpus can state it without confounding branding with the major.

⚠ **The one header difference is a capture condition rather than a build
property, and it is recorded as unresolved rather than explained.**
`cache-control` at position 0 is what a navigation that is not served from cache
sends; nothing here establishes that the two runs navigated identically.

#### ⛔ AND IT REFUTES A CLAIM THIS PROJECT MADE ELEVEN HOURS EARLIER

⚠ **The record said:** "the unbranded build publishes an EMPTY trust-anchor list
and the branded one publishes 32 identifiers, so the bundled root store is
BRANDING rather than engine."

⛔ **That is wrong, and the reason is a confound rather than a mistake in the
reading.** The empty list came from a Chrome **for Testing** build, and this
Chromium build is equally unbranded and sends a full 206-byte list. So an empty
root store is a property of the automation channel's own build configuration,
not of being unbranded.

⭐ **What IS true, and it is the sharper claim:** the two 206-byte lists are
different bytes. Branded Chrome and this Chromium carry root stores of the same
size and different content, which is a build-time input rather than a branding
switch.

⚠ **What this pair does NOT separate:** the archive is a distributor's
repackaging, which may build against system libraries and disable features, so a
difference between these two is branding OR packaging. ⛔ Recorded in the cell
rather than left to be discovered.

#### ⚠ What is still open about this entry's own subject, and is not this entry

⭐ **`CI-04`'s merge condition wants agreement across two INDEPENDENT sources**,
and no cell has that yet: `firefox/stable/win64` now holds two profiles, one from
a laptop and one from a runner, but at different builds (`154.0.1` and `154.0`).
⛔ That is a coverage fact rather than an unmet acceptance, and this entry's
acceptance command is what decides whether it closes.

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
**Category** corpus, **Priority** P2, **Effort** M, **Status** done

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

### ⭐ Closed 2026-09-02. Measured here, published per build, and the trade stated

⛔ **The premise said this was measured elsewhere and inherited.** It is measured
here now: Chrome `152.0.7977.75` on `linux64`, captured on a hosted runner
2026-09-02, carries codepoint `0xca34` at 206 bytes.

```text
$ sh scripts/common/check-trust-anchors.sh
trust anchors ok: 2 of 6 profile(s) carry codepoint 0xca34, and every one has a
  published list with its capture instant. The recommendation states all three
  options and asserts no preference.
exit=0

$ sh scripts/common/check-trust-anchors.sh --json
{"schema":"check-trust-anchors/1","carriers":2,"lists":2,"profiles":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-trust-anchors.ps1 -Json
{"schema":"check-trust-anchors/1","carriers":2,"lists":2,"profiles":6,"problems":0}
```

#### ⛔ What the measurement changed about the inherited claim

| | inherited | measured here |
| --- | --- | --- |
| codepoint | `0xca34` | ⭐ the same |
| length | 206 bytes | ⭐ the same |
| shape | a length-prefixed list of identifiers | ⭐ the same: a two-byte big-endian outer length, then one-byte-length-prefixed items |
| **count** | 24 identifiers | ⛔ **32**, of 4, 5 and 8 bytes |
| the name | inferred from `draft-ietf-tls-trust-anchor-ids` | ⚠ **still inferred.** No specification was read against these bytes and this entry does not pretend otherwise |

⭐ **The count differing is the entry's premise confirmed rather than a
contradiction.** The body is a snapshot of a root store, a root store changes,
and the two numbers come from two builds. ⛔ Publishing per build with the
capture date is the only honest way to carry it, which is what this does.

⚠ **And `0x12e0` is absent from this project's own `152`**, where the origin's
`152` capture carries it. `CORPUS-05` recorded it absent from every `151` here;
that now extends to a `152`. [`../docs/inherited-claims.md`](../docs/inherited-claims.md)
section 3 carries both.

#### The two deliverables

| | |
| --- | --- |
| ⭐ **the list, per build, beside the corpus** | `b-ids-corpus anchors --out DIR` writes one file per carrying build: the profile it came from, the capture instant, the declared length and every identifier **in the browser's own order**. ⛔ The order is part of what was measured and sorting it would publish a list no browser sent. |
| ⭐ **the recommendation** | [`../docs/trust-anchors.md`](../docs/trust-anchors.md), stating all three options with the cost of each and asserting no preference: omit it and be one extension short; carry a captured list and be honest the day it was captured and a fingerprint of that day afterwards; send it empty and produce a shape no browser sends. |

```text
$ cargo run -q -p b-ids-corpus -- anchors --root . --out dist/anchors
wrote dist/anchors/chrome-152.0.7977.75-linux64.json (32 identifier(s), captured 2026-09-02T14:08:20Z)
wrote dist/anchors/chrome-152.0.7977.76-win64.json (32 identifier(s), captured 2026-09-02T14:53:12Z)
corpus=anchors lists:2 profiles:6
```

#### ⛔ The check refuses a vacuous pass, and that was proved

⚠ **Every assertion this check makes is satisfiable by an empty set.** A corpus
in which no profile carries the extension would pass rule 1 by having nothing to
check, which is the "acceptance command that cannot fail" row of
[`../docs/conventions/forbidden-patterns.md`](../docs/conventions/forbidden-patterns.md).
⭐ So the no-carrier case exits **2** and says why, and it was seen to:

```text
$ sh COPY-OF-THE-CHECK-COUNTING-A-CODEPOINT-NOTHING-CARRIES
check-trust-anchors: no profile in this corpus carries codepoint 0xca34, so
  there is nothing to publish and nothing this check can verify. That is a
  fact about the builds captured, not a pass. TODO/corpus.md, CORPUS-04.
exit=2
```

⭐ **And the recommendation's three options are asserted rather than assumed.**
With one option's heading removed from the live document:

```text
trust-anchor check failed, 1 problem(s):

  the recommendation does not state the option 'Send it empty'
exit=1
```

#### ⚠ What this leaves open, named rather than left

⛔ **The name is still inferred**, and settling it is one reading of
`draft-ietf-tls-trust-anchor-ids` against these bytes.
[`../docs/trust-anchors.md`](../docs/trust-anchors.md) says what would settle it
and [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 3 is
where its status lives. ⚠ That reading is not this entry's acceptance and was
not done here.

⚠ **This closed on one carrier, which was one sample**, measured on one build,
one platform, one day. ⭐ A second arrived the same day and the amendment below
is what it said. The blocks above are re-run against the tree as it now stands
and therefore count both.

#### ⭐ Amended 2026-09-02: a second carrier arrived within the hour, and the order is not the set

⚠ **The paragraph above says one carrier is one sample.** A second landed the
same day: Chrome `152.0.7977.76` on `win64`, from the same vendor route on a
hosted runner. ⛔ What it says is sharper than "the shape holds".

| | |
| --- | --- |
| ⭐ **the same 32 identifiers** | the SET is identical between `linux64` and `win64`. Nothing is on one platform and not the other. |
| ⭐ **the same length** | 206 bytes on both, with the same 4, 5 and 8-byte identifier lengths |
| ⛔ **and a completely different ORDER** | **all 32 positions differ.** The two bodies are not the same bytes. |

⛔ **So a client copying a captured list has a second decision the entry did not
name: whether to copy the ORDER.** A fixed order copied from one capture is a
constant, and if the order is per connection then a constant is exactly the
thing that makes a client distinguishable.

⚠ **This project cannot yet say which it is.** One capture per platform
distinguishes "the order is per platform" from "the order is per connection"
not at all, and asserting either would be a conclusion from a single sample per
side. ⭐ What would settle it is two connections of ONE navigation on ONE
platform, which is a comparison
[`../docs/trust-anchors.md`](../docs/trust-anchors.md) now names and nothing has
run.

⭐ **The published lists carry the order they arrived in**, so whichever answer
comes back, the evidence is already on disk rather than needing a re-capture.


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


---

## CORPUS-06. The headless normalisation has no caller, and six published profiles say so

**Source** found by this session's consolidation pass, 2026-09-03, reading the published routes against the code
**Category** corpus, **Priority** P1, **Effort** M, **Status** done

### Problem

Every profile in the corpus carries a User-Agent ending
`HeadlessChrome/<version> Safari/537.36`, and so does every `user-agent` route
the data branch publishes. ⛔ **A consumer who pastes one is announcing
automation**, which is the exact trap `DRIVER-03` was written to remove.

### Premise

⭐ **Measured on this tree, 2026-09-03**, and it is a dead-caller defect rather
than a wrong decision.

```bash
grep -rn "headless::normalise\|normalise(&mut" crates/*/src/*.rs
```

The function, its five tests and its provenance reason are all in
[`../crates/b-ids-driver/src/headless.rs`](../crates/b-ids-driver/src/headless.rs),
and ⛔ **nothing outside that module calls it.** The last paragraph of
[`driver.md`](driver.md)'s `DRIVER-03` names the seam this belongs at and says
it was left unwired because no capture path wrote a profile yet.
⚠ `CORPUS-01` landed and nothing went back for it.

⚠ **The profiles are not dishonest.** `captured.switches` records
`--headless=new` on every one of them, so the condition is published. What is
missing is the substitution `DRIVER-03` specified, with its provenance entry.

### Approach

Call it where `DRIVER-03` said it belongs, in the path that turns a capture into
a profile, and record what it changed.

- ⭐ **In [`../crates/b-ids-corpus/src/capture.rs`](../crates/b-ids-corpus/src/capture.rs)**,
  beside the switch redaction, which is the same shape: a published value that
  differs from the captured one, with a reason in the provenance map.
- ⛔ **The raw bytes are untouched.** `raw.http2_frames_hex` keeps the frames the
  browser sent, so the un-normalised value stays recoverable, which is what makes
  the substitution a publication choice rather than a loss.
- ⚠ **A field carrying a headless marker the module does not know is recorded
  rather than guessed at.** That rule already exists in the function and this
  entry must not weaken it.

⛔ Must not: edit a published profile. The corpus is append-only, so the six
profiles keep the value they carry and a corrected one arrives as a new capture.
⛔ Must not: normalise when the launch was not headless.

### Decision

⚠ **Two ways to reach the same published value, and the recommendation is the
first.**

| | |
| --- | --- |
| ⭐ **normalise at capture** | the profile carries the browser-shaped value with a `substituted` provenance entry, and every surface downstream is right because there is one value. This is what `DRIVER-03` specified. |
| normalise at emit | the corpus keeps the wire value and each surface decides. ⛔ It loses: a route is generated only where the corpus holds a value, so this would put the decision in every emitter rather than in one place, and a consumer reading the profile JSON directly would still get the headless token with nothing saying so. |

⚠ **What it does not settle**: whether a headless capture should be published at
all rather than captured with a window. That is `CORPUS-02`'s matrix question
and this entry does not answer it.

### Consumers

⛔ **The data branch republishes on the next push to the default branch**, so a
consumer reading `routes/user-agent/...` gets the corrected value for any
profile captured after this lands. ⚠ The six profiles published before it keep
theirs, and nothing rewrites them.

### Prove

```bash
cargo test -p b-ids-corpus headless
```

Passing means: a capture taken with `--headless=new` produces a profile whose
`user-agent` carries the windowed product token and whose provenance map marks
that field `substituted` with the headless reason, and a capture taken with a
window produces a profile the normalisation did not touch.

### Closing

**Closed 2026-09-03T13:07:00Z.** The seam is wired in
[`../crates/b-ids-corpus/src/capture.rs`](../crates/b-ids-corpus/src/capture.rs),
gated on the launch rather than on the value, and five cases hold it.

```text
$ cargo test -p b-ids-corpus headless
     Running tests\corpus.rs (target\debug\deps\corpus-f6ba89506dce22fa.exe)

running 5 tests
test headless_a_capture_taken_without_a_window_publishes_the_windowed_product_token ... ok
test headless_a_windowed_launch_is_left_alone_even_where_the_value_carries_the_token ... ok
test headless_a_windowed_value_from_a_headless_launch_is_not_marked_substituted ... ok
test headless_the_switch_that_says_so_is_published_beside_the_substitution ... ok
test headless_the_substitution_is_recorded_with_its_reason ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 0.00s
exit=0
```

⚠ The five other targets in the same invocation match nothing and print
`0 passed` each; the block above is the one that ran the cases.

### The three mutations, and each was read unpiped

⛔ **Every mutation was made against a copy of the file under the ignored
scratch directory, the live file restored from that copy, and the restored file
compared against `HEAD` before anything else ran.**

| planted | what went red |
| --- | --- |
| the `normalise` call deleted from the capture path | the product token case and the provenance case, 2 of 5 |
| the launch gate replaced with `if true` | the windowed-launch case, which is the one that says a value nobody caused is never rewritten |
| the provenance `insert` in `b_ids_driver::headless` made unreachable | the provenance case alone, with the value still rewritten, which is exactly the shape `DRIVER-03` calls unshippable |

⭐ **The second is the one worth having.** A normalisation that fired on every
capture would look correct on every profile in the corpus today, because every
one of them was taken headless.

### ⚠ What this does NOT do, and the corpus says so

⛔ **The six published profiles are unchanged**, and every `user-agent` route on
the data branch still carries `HeadlessChrome`. The corpus is append-only: a
correction is a new capture of the same build, not an edit. ⚠ So the defect is
fixed forward and the published half stays wrong until the capture lane runs
again, which is a run on a hosted runner rather than anything this session can
do on this machine.

⭐ **A reader can already tell**, and could before this entry: every one of those
profiles carries `--headless=new` in `captured.switches`, so the condition was
published even while the substitution was missing.

### ⛔ The dependency this added, stated rather than buried

`b-ids-corpus` now depends on `b-ids-driver`. ⚠ The edge is worth naming because
this tree writes its dependency directions down: the driver depends on
`b-ids-schema` and nothing else, so this adds no third-party linkage, and the
alternative was moving a module `DRIVER-03` cites at a file and a line.
