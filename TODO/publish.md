# publish

Three delivery surfaces, because each fails differently, and the routes a
program with nothing but `curl` can read.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## PUB-01. Releases, tagged and versioned and immutable

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P1, **Effort** M, **Status** open

### Problem

A consumer that pins a release and gets different bytes later has been broken
silently. A consumer that has to list releases before it can fetch one has been
given an API instead of a file.

### Premise

Believed. Nothing has been released.

### Approach

- **Tag shape**: the schema major, the date, and a counter, for example
  `v1.2026.08.30.1`. The first component says which schema a consumer is getting
  without opening anything, the date says how fresh the data is, the counter
  allows more than one release a day, and the whole thing sorts correctly and is
  a valid tag.
- **Moving tags beside it**: one per schema major, and one overall, so a script
  can fetch without listing.
- **Assets**: one archive per format from `SCHEMA-08`, a checksums file, and a
  build provenance attestation. Never an unversioned archive as the only asset.
- ⛔ **A published release is immutable.** Never re-upload an asset. Cut a new
  release that supersedes it and say so in the body. Consumers pin releases, and
  a mutated asset breaks them silently.
- **Reproducible**: the same inputs produce byte-identical archives. Sort
  everything, zero the timestamps, pin the generator.

Must not: put a token in the fetch path. A release asset is public and a
consumer should need nothing.

### Prove

```bash
sh scripts/common/check-release.sh --dry-run
```

Passing means: two runs of the release build over the same corpus produce
byte-identical archives and identical checksums, and a run that would overwrite
an existing tag exits non-zero.

---

## PUB-02. The data branch, over raw file serving

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P1, **Effort** M, **Status** open

### Problem

Releases alone are not enough, and the reason is practical rather than
theoretical. A releases API is rate-limited, needs authentication for serious
polling, has its own downtime, and returns a document a consumer has to walk
before it can fetch a file. Raw file serving takes a path, is fronted by a
cache, needs no token, and is what every install script already uses.

### Premise

Believed, and one reference project's practice supports it: a dataset published
as flat files over a content delivery network, refreshed weekly by its own
automation, with no index in the fetch path.

### Approach

Publish **both**, from the same build, with the same content.

- an **orphan branch** carrying only generated artefacts and no source history,
  so it stays small and clones fast;
- **dated snapshots and a `latest` pointer**, both. The snapshot is what a
  reproducible build pins; the pointer is what a script follows;
- ⛔ **append-only, never force-pushed.** A consumer pinning a commit on this
  branch keeps working forever, and that property is free;
- **an index and a checksums file**, so a consumer can verify what it fetched
  without any API;
- **document the cache** next to the URL. A raw file service caches for some
  minutes, and a consumer polling for a change must not be surprised.

⛔ The source tree, any vendored dependency and the reference corpus stay off
this branch. A consumer of the data never has to reason about somebody else's
licence, because none of it is in what they downloaded.

Must not: generate the branch from anything but the canonical corpus, and must
not hand-edit a file on it.

### Prove

```bash
sh scripts/common/check-data-branch.sh
```

Passing means: the branch's content is regenerated and compares byte-identical
to what is published, every file has a checksum in the index, and a push that
would rewrite history is refused.

---

## PUB-03. Routes a program with nothing but `curl` can read

**Source** the operator; [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) section 9
**Category** publish, **Priority** P1, **Effort** M, **Status** open

### Problem

A consumer that wants one value should fetch one path and get one value. Today
that consumer would have to fetch a profile, parse it, and extract a field, which
is three dependencies for a string.

### Premise

⭐ **Measured, and the model has the defect.** The reference the operator named
uses exactly this naming shape, and its single-value files end with a newline:
`od -c` over two of them shows a trailing `\n`. So a consumer has to strip it,
which is the burden this entry exists to remove.
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
has the measurement.

### Approach

Generate a flat route tree from the canonical corpus, one file per value, at
every useful permutation of the axes the corpus carries.

```text
user-agent/chrome/stable/latest/windows.txt
user-agent/chrome/stable/152.0.7977.64/windows.txt
user-agent/chrome/beta/latest/linux64.txt
ja4/chrome/stable/latest/linux64.txt
akamai/chrome/stable/latest/linux64.txt
sec-ch-ua/chrome/stable/latest/windows.txt
accept-language/firefox/release/latest/macos-arm64.txt
header-order/chrome/stable/latest/windows/navigate.txt
client-hello-hex/chrome/stable/latest/linux64.txt
```

Rules the generator holds:

- ⛔ **a single-value file contains the value and nothing else.** No trailing
  newline, no leading whitespace, no quotes. A consumer must never need to strip
  anything.
- **a multi-value file is newline-delimited and says so by its extension**, so
  the two are distinguishable without fetching. A list file is `.list.txt`.
- **every axis is a directory**, so a path is constructible without an index:
  property, browser, channel, version-or-`latest`, platform, and the request
  variant where a property has one.
- **`latest` is a real file, not a redirect**, so one fetch is one round trip.
- ⭐ **an `index.txt` per directory** listing what is beside it, for a consumer
  that wants to discover rather than construct.
- the axes come from the corpus, so a property this project stops measuring
  stops having routes, rather than serving a stale value forever.

⚠ **Decide the permutation policy explicitly**, because the cross product grows
fast: every property times every browser times every channel times every version
times every platform times every variant. Generating all of them is cheap in
bytes and expensive in file count.

Must not: generate a route for a value that does not exist in the corpus, and
must not fall back to a neighbouring platform's value when one is missing. A
missing route is a 404, which is a fact; a substituted value is a lie.

### Decision

Whether to generate every permutation or only those with a value.

Recommendation: only those with a value, and publish the `index.txt` so a
consumer can see what exists. A route that resolves to a plausible-looking wrong
value is worse than one that 404s, and this is the same rule as the corpus's.

### Prove

```bash
sh scripts/common/check-routes.sh
```

Passing means: for every generated single-value route the last byte is not a
newline; every route resolves to a value present in the corpus; `latest`
resolves to a stable channel; and a fixture route deliberately given a trailing
newline fails the check.

---

## PUB-04. The formats that are not data files

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** M, **Status** open

### Problem

Most of the day-to-day value of a corpus is in the artefact somebody can paste
into their own tool, and none of the generated data formats is that.

### Premise

Believed.

### Approach

Generate, from the canonical corpus, alongside the data formats in `SCHEMA-08`:

- **ready-to-use client configuration**: a flag line for a general-purpose HTTP
  client, an invocation for an impersonating one, a snippet for each TLS library
  this project targets. A consumer copies one line and is done.
- **detection-side artefacts**: digest allowlists, and rule snippets for the
  common reverse proxies and web application firewalls. ⭐ Same data, other
  direction, and it is half the reason the project is acceptable to everybody.

Must not: generate a configuration for a stack that cannot actually emit the
profile. That is what the support matrix in `EMIT-01` is for, and a snippet that
silently approximates is worse than no snippet.

### Prove

```bash
sh scripts/common/check-generated-configs.sh
```

Passing means: every generated snippet is for a (profile, stack) pair the
support matrix marks as emittable, and a pair marked as a hole generates a
comment naming the hole instead of a snippet.

---

## PUB-05. Language packages that embed the corpus

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** L, **Status** open

### Problem

Fetching and parsing a corpus is work. A dependency line is not.

### Premise

Believed. ⭐ Stated in the design brief as the single biggest quality-of-life
win in the project.

### Approach

One package per major ecosystem, each embedding the corpus at a pinned release
and published on the same schedule as the release. The Rust one is `LIB-01` and
is the reference implementation; the others follow its shape.

Each package states which corpus release it embeds, in a field a program can
read, so a consumer can tell how old their data is without leaving their own
language.

Must not: publish a package that fetches at runtime by default. A package that
needs the network to answer is a package that fails in the environment its
consumers care most about.

### Prove

```bash
sh scripts/common/check-packages.sh
```

Passing means: each package builds offline, reports the corpus release it
embeds, and a test asserts that release matches the one the build was cut from.

---

## PUB-06. A packet capture per profile

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P3, **Effort** M, **Status** open

### Problem

Every network engineer already has a tool for one format, and this project does
not produce it.

### Premise

Believed, and it depends on whether the harness can synthesise one from the
recorded bytes rather than needing to have captured one.

### Approach

Synthesise a capture file per profile from the raw bytes the profile already
carries, so it is a generated format like any other rather than a second
capture path.

⚠ Establish first whether a synthesised capture is faithful enough to be worth
publishing. A capture with invented timing and invented sequence numbers may
mislead a reader who opens it expecting a real one, and if so the answer is to
label it as synthesised in the file itself.

Must not: publish a synthesised capture that is indistinguishable from a real
one.

### Prove

```bash
sh scripts/common/check-pcap.sh
```

Passing means: a synthesised capture parses in a standard tool, its
`ClientHello` bytes compare equal to the profile's raw hex, and the file carries
a comment identifying it as synthesised.

---

## PUB-07. The licence stated in three places

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P1, **Effort** S, **Status** open

### Problem

A file that travels alone still has to say what it is. A consumer who downloads
one profile should not have to find this repository to learn they may use it.

### Premise

Believed. The licence choice itself is settled and its reasoning is in
[`../README.md`](../README.md).

### Approach

State it in three places, and have a check assert all three agree:

- a `LICENSE` file on the data branch;
- a `"license"` field in every profile and in the index;
- a line in every release body.

⛔ **Separate the trees.** The harness, the emitters and any vendored upstream
live in the source repository under their own terms. The data branch and the
release assets carry only generated data. A consumer of the data never has to
reason about a vendored dependency's licence because none of it is in what they
downloaded.

⚠ **One caveat that is not settled and must not be published as though it
were.** A digest specification can carry its own terms. Nothing in this project
emits a restricted digest variant until `VALID-04`'s licence question has a
written answer.

Must not: put a licence identifier in a profile that disagrees with the branch's
`LICENSE` file. A check holds that.

### Prove

```bash
sh scripts/common/check-license-consistency.sh
```

Passing means: the three statements agree, and a fixture where one disagrees
fails with a message naming all three.

---

## PUB-08. One generator for the release body and the changelog

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** S, **Status** done

### Problem

Release notes and a changelog that are written separately drift, and the reader
who trusts the wrong one is the one who was doing something careful.

### Premise

Believed.

### Approach

One generator, two outputs, so the two cannot disagree by construction rather
than by discipline. The same code writes the automated pull request body in
`CI-04`.

What the body must contain, so a reader can answer "what changed and how do I
know" without leaving the page: what changed per browser, channel and platform,
**field-level** rather than "updated a browser"; new and superseded profiles;
the validator's summary; capture provenance for every profile that moved; and
the run identifier.

Must not: write a release body by hand, and must not let the changelog be
generated from a different query than the body.

### Prove

```bash
sh scripts/common/check-notes-generator.sh
```

Passing means: the generator is run twice over one corpus change and produces a
release body and a changelog entry that agree field for field, and a fixture
where they are generated from different inputs fails.

---

### ⭐ Closed 2026-09-02. One model, two renderers, and the comparison can fail

⛔ **The two cannot disagree by construction rather than by discipline.**
`b_ids_corpus::notes::model` computes what changed between two corpus states,
and `release_body` and `changelog_entry` are the only things that turn it into
text. Neither computes anything of its own.

```text
$ sh scripts/common/check-notes-generator.sh
notes generator ok: 6 case(s). The release body and the changelog entry
  are rendered from one model, they carry every fact it holds, a no-op
  change renders nothing, and two outputs from different inputs are
  asserted NOT to agree.
exit=0

$ sh scripts/common/check-notes-generator.sh --json
{"schema":"check-notes-generator/1","cases":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-notes-generator.ps1 -Json
{"schema":"check-notes-generator/1","cases":6,"problems":0}
```

#### What makes "agree" checkable rather than a sentence

⭐ **`notes::facts` returns every fact the model holds**, and the suite asserts
both renderings contain every one of them. ⛔ That is a function rather than a
paragraph, so a renderer that quietly dropped a movement fails a test rather
than passing a review.

| assertion | why it is there |
| --- | --- |
| both outputs carry every fact | the entry's requirement, made mechanical |
| a no-op renders **nothing** | ⛔ silence is the correct output for "the browser did not change". A bot that writes on a schedule trains people to ignore it, and `CI-04` states the same rule |
| a version move is **field-level** | "updated a browser" is what a reader cannot act on. ⚠ The fields come from `b_ids_validator::diff` rather than a second comparison here |
| a first profile is an **addition**, not a diff | listing every field of a first profile as changed would be a diff against nothing |
| two runs produce identical text | a body that diffs on every run is one nobody can review |
| ⛔ two outputs from **different inputs** do NOT agree | the negative case the acceptance names. A check that only ever sees agreement is one nobody knows works |

#### ⛔ The pair earned itself before it was registered

⚠ **The two halves disagreed the first time they were run against each other**,
and the cause is one this tree's own conventions warn about:

```text
$ sh scripts/common/check-notes-generator.sh --json
{"schema":"check-notes-generator/1","cases":6,"problems":0}
$ pwsh -NoProfile -File scripts/common/check-notes-generator.ps1 -Json
{"schema":"check-notes-generator/1","cases":0,"problems":0}
```

⛔ **A backslash was lost crossing a shell**, so the PowerShell half's
`'^running (\d+) tests'` arrived as `'^running (d+) tests'`, matched nothing and
reported zero cases.
[`../docs/conventions/shell.md`](../docs/conventions/shell.md) section 1 names
exactly this: "A backslash escape that survives one hop loses a backslash on the
next." ⚠ It was written by passing a payload through a shell, which is what that
section says not to do, and it was found by running both halves rather than by
reading them.

⭐ **Neither half's answer was wrong about the suite**; they were wrong about
each other, which is the drift a twin comparison exists to catch and the reason
`check-notes-generator` has a row in `check-twins` rather than only a gate line.

#### ⚠ What this does not do

⛔ **It writes no release and no changelog entry into the tree.** There is no
release to write notes for: `PUB-01` is that surface and it does not exist.
⭐ What exists is the generator both it and `CI-04` will call, proved before
either of them can be wrong about it.

⚠ **And the model compares two corpus STATES rather than reading git.** A caller
holding a before and an after gets an answer; deriving those two states from a
revision range is the caller's job and `CI-04` is where it lands.

---

## PUB-09. Signed and attested captures

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** publish, **Priority** P2, **Effort** M, **Status** open

### Problem

A consumer currently has to take this project's word that a value was measured
rather than typed. Provenance in a file is a claim by whoever wrote the file.

### Premise

Believed, and there is a caution worth carrying: a checksums file published in
the same release as the artefact proves transport rather than authorship,
because whoever could replace one could replace the other. A signature the
consumer verifies independently is the check that proves authorship, and it
applies on top.

### Approach

Sign the index and attest the build, so a consumer can distinguish a capture
from an assertion without trusting a file that travelled with the artefact.

Publish what verifying looks like, as a command a consumer runs, next to the
route it verifies.

Must not: describe a checksums file as proof of authorship. It is not, and
saying so is the kind of claim this project exists to stop making.

### Prove

```bash
sh scripts/common/check-signing.sh
```

Passing means: a published index verifies against the project's key, a corpus
file altered after signing fails verification, and the published verification
command is the one the check runs.
