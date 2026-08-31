# corpus

The canonical data: how it is stored, what it must cover, and the policies that
decide what goes in.

⭐ Coverage is what makes this a product rather than a curiosity about one
browser. One browser is a weekend; the matrix is the thing anybody uses.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## CORPUS-01. Content-addressed, append-only, never edited in place

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** corpus, **Priority** P1, **Effort** M, **Status** open

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

---

## CORPUS-03. `latest` means stable, and beta is how the project gets ahead

**Source** the founding brief. ⚠ Design reasoning, never measured.
**Category** corpus, **Priority** P2, **Effort** S, **Status** open

### Problem

A consumer following a pointer called `latest` must never be handed a
pre-release build. That is the same failure as shipping a version nobody runs
yet, and consumers will assume otherwise unless it is stated.

### Premise

Believed, and the mechanism is already available: the automation-build index is
addressable **by channel**, which is the property that lets this project be
ahead rather than perpetually behind.

### Approach

Three rules, and they go in the README as well as here because a consumer will
not read this file:

- ⛔ **`latest` means stable and nothing else.** Beta, canary and nightly are
  published beside it, in their own paths, clearly labelled.
- ⭐ **Capturing beta and canary is the mechanism.** The profile for the next
  stable is ready the day it ships, because it was captured weeks earlier under
  another name.
- ⛔ **Historical versions are out of scope.** The corpus accretes going
  forward, which is what a dated append-only corpus is. There is no backfill.
  A historical profile contributed from outside is accepted with `vendor`
  provenance and stays a draft unless somebody can capture the build, because a
  value nobody can re-measure is a value nobody should trust.

Must not: promote a beta profile into the stable path when it ships. It is a
different capture of a different build; capture the stable build.

### Prove

```bash
sh scripts/common/check-routes.sh --assert-latest-is-stable
```

Passing means: every `latest` route resolves to a profile whose channel is
stable, and a fixture corpus in which one does not fails with a message naming
the route.

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
**Category** corpus, **Priority** P3, **Effort** S, **Status** open

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
sh experiments/30-identify-extension.sh
```

Passing means: the script records what was searched and what it found, and
either the extension is named with a citation or the search is recorded as
exhausted with a list of what was ruled out.
