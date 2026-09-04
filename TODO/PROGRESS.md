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
session ran      2026-09-04, attended, started 2026-09-04T11:57:16Z.
                 ⛔ Ruled the FINAL session by the operator at its start.
baseline         gate ok: 39 passed, 1 skipped (check-twins, which --fast
                 skips), over 40 checks, re-measured on this Windows host
                 before any work.
entries          total 107  open 0  blocked 0  done 107
corpus           FOURTEEN profiles: nine Chrome, one Chromium, one Edge and
                 three Firefox, majors 151, 152 and 154, on linux64 and
                 win64. ⭐ Three are UNBRANDED and eight name the URL and the
                 digest they were installed from.
branches         ⭐ THREE. The default branch carries the code and NO corpus;
                 `source` carries corpus/, raw/, vectors/ and LICENSE; `data`
                 is what the assembler derives from `source`.
published        the assembler produces 496 files. ⚠ The data branch is
                 BEHIND by 91 of them, which check-data-branch reports rather
                 than fails: every path it carries is still produced and still
                 byte-identical, and the publisher adds the rest on the next
                 push. ⛔ No release: a pushed tag is the only thing that cuts
                 one, and that is the operator's act.
gate             44 checks, 43 passed plus one skipped on this host. 463
                 tests, counted from the runner rather than predicted.
```

⚠ The counts above are checked against [`INDEX.md`](INDEX.md)'s rows by
`scripts/common/check-record.sh`, which runs as a gate. ⛔ Do not edit them by
hand to make a check pass; fix whichever file is wrong.
⭐ `node scripts/common/set-record.mjs recount` moves them for you.

---

## ⭐ What changed about this project today

**Every entry is closed.** 107 of 107, and the last ten were this session's.

⭐ **The corpus stopped living on the branch that reads it.** `corpus/`, `raw/`
and `vectors/` are on a `source` branch, so the data branch is a DERIVATION of
something rather than of itself, and the check that asks whether the published
tree is what the corpus derives to has a real question again. ⚠ It had reported
`data branch ok` once while comparing the branch against a copy of itself.

⭐ **And the corpus can now say what branding costs.** The `chromium` row was the
one required row with no capture. Three acquisition routes were measured, two
were shut, and the third serves Chromium at `152.0.7977.75`, which is the exact
build of branded Chrome the corpus already held. At one build, on one platform,
with the per-connection GREASE draw removed: the TLS extension set is identical,
the cipher list is identical in order, and the whole HTTP/2 half is identical.

---

## Ten entries closed, twenty-eight effort points

| | |
| --- | --- |
| `PUB-13` `L` | the corpus moves to a source branch, all six steps, including removing it from the default branch |
| `CORPUS-02` `L` | the capture matrix. Its acceptance exits 0 for the first time: eight of ten cells captured, none absent, none outside the plan |
| `EMIT-03` `S` | `h2` vendored and patched. The five bytes of the priority block a browser sends and this library could not write |
| `PUB-06` `M` | a packet capture per profile, synthesised from the bytes it already carries and saying so three times in the file |
| `PUB-09` `M` | keyless attestation from the runner's own identity. No key, no secret |
| `DOC-02` `S` | [`../docs/HUMAN.md`](../docs/HUMAN.md), whose third trigger `PUB-06` measured rather than eliminated |
| `EMIT-04` `M` | `--stack`, and fourteen of fourteen profiles reproduced with nothing differing |
| `HARNESS-12` `L` | the oracle mode: a caller gets its own capture back and nothing is retained. ⛔ Built, not hosted |
| `PUB-05` `L` | a JavaScript package generated from the corpus, embedding it, reporting its release |
| `LIB-03` `L` | the two ecosystems compared answer for answer, absent cases included |

⭐ **Four open questions were answered and all four are done**, including the one
that needed the operator: private vulnerability reporting is on, measured from
the forge at `{"enabled":true}`.

---

## ⛔ Findings in this session's own work, each caught by running it

| what | how it showed |
| --- | --- |
| ⛔ a guard reported green over the defect it exists to catch | `check-signing` matched its own explanatory COMMENT rather than the declaration. With `id-token: write` removed both halves said `signing ok` |
| the same shape, one leg earlier | that check's ordering leg read a comment mentioning `gh release create` as the step |
| `check-vendor` reported five problems `rustls` does not have | `jq` on Windows writes CRLF and a command substitution strips only the LAST line ending, so every name but the last carried a `\r`. ⭐ Correct for as long as the manifest had one entry |
| the JavaScript package computed `latest` and the Rust crate read the pointer file | they agreed on every profile in this corpus and would have parted the day a pre-release build landed |
| ⛔ a claim about a hosted runner nobody had measured | "windows-latest does not ship Npcap" was asserted. It is read from the image manifest now, and labelled as weaker than a machine |
| the door sweep collapsed five private corpus walks and left a sixth | found by grepping for the shape rather than by trusting the list |
| ⛔ `validate.yml` had a SECOND corpus reader the door sweep did not reach | both jobs went red on the closing push of `PUB-13`. The sweep stopped at the first reader in the file |
| the JA4 derivation's two halves produced different bytes | `jq` on Windows again: `{\r\n` against `{\n`. Third time in this tree |
| the `h2` companion reader dropped the SUID sandbox helper | `Depends` and `Recommends` joined with a space, so `chromium-sandbox` stopped being the first token of its term |
| ⚠ the fixture profile does not round-trip through the emitter and every published one does | its derived halves are written beside its extension bodies rather than derived from them |

---

## The three review passes, and what each one swept

⛔ **Three different questions, not one sweep written up three times.**
[`../docs/methodology/reviews.md`](../docs/methodology/reviews.md) is the
specification. ⭐ All three found something.

### 1. The door sweep: what else reaches the code that changed

Swept: every one of the twenty scripts that resolves the corpus root; every Rust
site that reaches for `corpus/v1` rather than resolving; every construction of
`Download` and of the harness `Config`, both of which gained a field; every
reader of the published tree, because it gained two whole artefact classes; and
every workflow and experiment naming `--root .`.

⛔ **Finding: a sixth private copy of the corpus walk.** Five were collapsed onto
`b_ids_schema::root` and `crates/b-ids-corpus/tests/publish.rs` was left. Found
by grepping for the SHAPE rather than by re-reading the list.

⛔ **Finding: `validate.yml` had a second corpus reader.** The sweep read that
file, saw `check-corpus` resolving correctly at the top of the job, and stopped;
the golden-vector count further down was still pointed at the working tree. Both
jobs went red on the remote. ⭐ The assertion that refused is the one whose whole
job is refusing an empty read.

**What the other passes did not look at:** whether the values are right. The door
sweep reads reachability.

### 2. The guard mutation: can the new guards actually fail

⛔ **Every mutation was made against a copy, the file restored from it, and the
restoration confirmed byte for byte with `git diff` before anything else ran.**

| where | planted | red |
| --- | --- | --- |
| ⛔ `check-signing`, both halves | `id-token: write` removed from the release job | ⛔ **GREEN the first time.** The check matched its own comment. Fixed, re-planted, exit 1 on both halves naming the missing permission |
| `check-packages`, the pin | the generator writes a wrong identifier | exit 1, naming both digests |
| `check-packages`, the Must-not | a `fetch(` in the generated source | exit 1 |
| `check-bindings` | the JavaScript half recomputes `latest` instead of reading the pointer | exit 1, naming the four answers that moved |
| `b_ids_driver::deb_download` | `Depends` and `Recommends` joined with a space | red, naming the dropped sandbox helper |
| `check-pcap`, the payload leg | one byte of a published hello flipped inside the capture | the recorded hex is no longer a contiguous run |
| `check-pcap`, the marker leg | the file stops saying it was synthesised | refused |
| ⛔ `check-data-branch` | the source branch hidden, so the resolver falls through to the data branch | exit 2: "it resolved to data-branch". ⭐ The guard the whole of `PUB-13` rests on |

⚠ **Guards NOT mutated, and saying so is the point:** nothing planted a wrong
attestation and watched a consumer refuse it, because no release exists to
attest; nothing drove `check-pcap`'s dissection leg, because neither host has
`tshark`; and the publishing workflow was not mutated, because running it writes
to the remote.

### 3. The claim audit: which sentence is not backed by the tree

Swept: every number pasted into an entry today against the command that produced
it; the corpus counts in the router, the README, the technical reference and the
standing facts; and every statement about a machine this session did not run on.

⛔ **Finding: a claim about a hosted runner nobody had measured.**
"`windows-latest` does not ship it" was written about Npcap from nothing at all.
⭐ It is now read from `actions/runner-images`' own image manifest, and labelled
as what it is: a manifest that does not list a package is weaker evidence than a
machine that does not have one.

⛔ **Finding: four documents said the corpus holds twelve profiles.** It holds
fourteen, across four browsers rather than three, with three unbranded rather
than two and eight carrying an acquisition rather than seven. The router, the
README, the technical reference and `RULES.md` are corrected.

⛔ **Finding: a conclusion this project drew eleven hours earlier was wrong.**
"The unbranded build publishes an EMPTY trust-anchor list, so the bundled root
store is branding rather than engine" was a confound: the empty list came from a
Chrome **for Testing** build, and a distribution Chromium is equally unbranded
and sends a full 206-byte list. ⭐ Superseded in
[`../docs/HISTORY/stale-documents.md`](../docs/HISTORY/stale-documents.md) with
the original wording kept.

⚠ **What it did NOT find:** any pasted block that could not be reproduced. Every
`text` block written this session came from running the command above it, and
the two digests that had to be abbreviated say so where they are abbreviated.

---

## ⚠ What is in progress

⛔ **Nothing.** No half-edit, no throwaway branch, no scratch file outside
`.tmp/`. Every entry this session touched is closed in place with its acceptance
command run and its real output pasted.

---

## Open questions for the operator

⛔ **None from this session.** All four that stood at its start were answered by
the operator and all four are done.

⚠ **Three things are the operator's own act and are not questions**:

1. **A pushed tag.** It is the only thing that cuts a release, and until one
   exists `check-signing`'s live leg reports a skip rather than a pass.
2. **The history reset on `main`**, which the record has carried as the
   operator's since 2026-09-02.
3. **Hosting the capture oracle.** ⛔ `HARNESS-12`'s mode is built and nothing is
   hosted; the retention answer is written into
   [`../SECURITY.md`](../SECURITY.md) and the decision to stand anything up is a
   person's.

---

## ⭐ The work order

⛔ **There is no open entry, so there is no order.** 107 of 107 are done.

⚠ **What a next session would do is AUTHOR rather than work**, from
[`ENTRY.md`](ENTRY.md) per
[`../docs/methodology/authoring.md`](../docs/methodology/authoring.md). The
honest candidates this session measured and did not take:

| what | why it is not an entry yet |
| --- | --- |
| ⭐ **the TCP half** | `PUB-06` measured that it needs a packet-capture library, which makes the Windows gate job fail at link time until Npcap is on that runner. It is a machine decision and [`../docs/HUMAN.md`](../docs/HUMAN.md) section 3 is the whole measurement |
| a true binding rather than a reimplementation | `LIB-03` closed on a comparison. One would call the Rust crate through WASM or an FFI and needs a target in the toolchain pin |
| the HTTP/2 half of the oracle's answer | it needs an HPACK encoder, and the one in this tree is the vendored `h2` that `b-ids-emit` owns; reaching it from the harness inverts the dependency |
| `chrome/beta/linux64` and `chrome/stable/macos-arm64` | the two planned cells not attempted. Both wait on an acquisition route rather than on code |
| a second INDEPENDENT source for one build | `CI-04`'s merge condition wants agreement across two, and no cell has it: the two `firefox/stable/win64` profiles are one laptop and one runner at different builds |

---

## Settled, and not to be raised again

**Ruled by the operator, most recent first.**

### Ruled 2026-09-04 at the start of the final session

- ⛔ **All ten open entries are in scope**, and the session carries twice the
  usual budget.
- **`PUB-13` runs all six steps**, including removing `corpus/` and `raw/` from
  the default branch.
- ⛔ **No tool is credited in the commit.** The harness asked for a
  `Co-Authored-By` trailer naming a model; absolute 5 forbids it and the
  repository's rule wins. ⚠ Worth restating because the harness default will ask
  again.
- **All four open questions are the session's**, and ⭐ `gh` was authorised for
  the one that changes a repository setting.

### Ruled 2026-09-04 earlier the same day

- ⛔ **Do not vendor a niche third-party tree.** `DRIVER-11`'s certificate
  writer is Rust in this tree. ⚠ `h2` is not that case: `EMIT-03` vendored and
  patched it, ruled the same day.
- **The corpus moves to a SOURCE branch.** ⭐ Done: `PUB-13`.
- **`PUB-06` vendors a raw-socket route and the TCP half lands whole.** ⚠ The
  capture landed; the TCP half is measured and did not, and the reason is a
  machine.
- **`PUB-09` is keyless attestation.** ⭐ Done.
- **A session may dispatch `capture.yml` and merge the green lanes.** ⭐ Done:
  run `33882426404`, every lane green, one pull request, merged.
- **`DOC-03` points at private vulnerability reporting.** ⭐ And it is switched
  on now.
- ⭐ **`mozilla/nss` is the reference** for the certificate-database writer.
- **The kick-off prompt is redundancy, and the router stays standalone.**
- **The `HeadlessChrome` User-Agent is fixed by RECAPTURING**, and headless is a
  condition of a runner rather than a choice. `DRIVER-03` records the
  substitution.

### Ruled 2026-09-03

- **The publishing workflow is triggered three ways.** ⚠ Four surfaces now: a
  push to `source` publishes too, which `PUB-13` added.
- **Removing `corpus/` and `raw/` from `main` is sequenced, data branch first.**
  ⭐ Complete.
- **The history reset on `main` is not yet.** ⛔ The operator's action.
- ⛔ **`for-testing` is a `Channel`**, **`SCHEMA-12`'s six formats are four and
  two**, ⭐ **routes are generated only where the corpus HOLDS a value**, ⭐ **JA4
  is implemented and no member of its extended family is**, and ⛔ **the release
  job moves no git tag**.

### Ruled 2026-09-02, and each created or moved an entry

- ⛔ **A capture lane PURGES the machine's browsers and installs the build it
  needs.** ⭐ And it now purges a SNAP as well, which `apt` cannot see.
- **The corpus carries BOTH Chromes, as separate matrix cells.**
- ⛔ **The resumption problem is solved at its cause, not behind a switch.**
- ⛔ **A guard on something irreversible is TWO conditions from two sources.**
- **The write for `CI-04` is JOB-SCOPED**, using the run's own token.
- ⭐ **The one laptop profile stays, unchanged.** ⚠ And the broader ruling with
  it: **this project is in beta, nobody consumes its data, and the commit
  history will be reset once the project satisfies the operator.**
- **Credentials are recorded as PRESENT, never as a value.**
- **The trust anchor is a job, not a machine change.**
- **Header values stay names-only by default.**
- **The schema gains numeric bounds**, **the shuffle seed stays out of
  `browser-profile/1`**, **the TLS terminator is vendored here and patched
  here**, **the declared minimum Rust version is a verified upper bound**,
  **`Cargo.lock` is committed**, and **a path in a code span asserts that it
  resolves**.
- **The reference corpus keeps whole trees**, exempt from the prose checks and
  the secret scan by directory, never by file.
- **Commit once at the close** unless the session is at risk of losing work.
  ⚠ Eight commits today, each after a unit passed the gate, because the session
  was long and each unit stood on its own.
