# b-ids (Browser-IDs)

A dated, provenance-carrying corpus of what real browsers put on a wire: TLS
`ClientHello` bytes, HTTP/2 settings and framing, and header sets in order.

⭐ **The corpus holds twelve profiles**, and every one is real: nine Chrome,
one Edge and two Firefox, across majors `151`, `152` and `154`, on Windows and
Linux, captured between 2026-09-01 and 2026-09-04 on a laptop and on hosted
runners. ⭐ Seven of them name the URL and the digest of the archive this project
fetched the browser from, rather than measuring whatever a machine happened to
have. ⭐ **Two are UNBRANDED builds**, which is what separates branding from
engine: the unbranded build sends the browser's root-store extension with an
empty list where the branded one beside it sends thirty-two identifiers. Every
profile carries the `ClientHello` bytes it was read from, published beside it. Around them are
the profile schema, a validator that refuses a combination no browser could
send, a capture oracle that terminates a TLS handshake and reads a browser's
HTTP/2 off the socket behind it, the methodology, the tooling and a reading of
the prior art.

⚠ **Six profiles across two majors and two platforms are not a matrix**, and
nothing here pretends otherwise. Every other value in the tree is a fixture or
an inherited claim: a fixture is shaped
like a capture and is not one, and an inherited value was measured somewhere
else and is recorded with its source. ⛔ Neither may be written into the corpus.

---

## Why it exists

Every impersonating HTTP client ships this data as a hand-maintained table in
its own source tree. Some of those tables are good; one project publishes 43
per-build signature files. What none of them carries is the part that makes a
measurement usable:

| missing everywhere | why it matters |
| --- | --- |
| a **capture instant** | a reader cannot tell a four-year-old value from a current one |
| **per-field provenance** | measured, assumed, substituted from another platform and copied from somebody's table are indistinguishable |
| the **raw `ClientHello` bytes** | a parser defect is unrecoverable, and some questions are answerable in no other artefact |
| a **channel** dimension | stable only, so the data is always behind the browser |
| a **published route** with a checksum | the data is reachable only by cloning somebody's client and reading its test fixtures |

The corpus serves detection exactly as much as impersonation. A server deciding
whether a caller is a real browser needs the same data as a client trying to
look like one, and "what does this build send" has one true answer regardless of
who is asking.

**The rule everything else follows from:** ⛔ **a fingerprint is measured, never
derived and never inherited.** Values this project has not measured live in
[`docs/inherited-claims.md`](docs/inherited-claims.md) and are never published
as data.

---

## Quick start

⚠ **The first three need `git` and a shell and nothing else.** Building the
crates needs the pinned toolchain in [`rust-toolchain.toml`](rust-toolchain.toml),
which `rustup` installs the first time `cargo` is run in this tree.

```bash
git clone https://github.com/Azathothas/b-ids
```

```bash
sh scripts/doctor/doctor.sh
```

Reports the host, the shell, the tools with versions and what this tree is. It
is a probe, so it exits 0 whether or not anything is missing.

```bash
sh scripts/common/check-gate.sh --fast
```

Runs every check this host can run and prints one verdict. ⚠ A missing tool is
reported as a skip and counted separately, because a skipped check is not a
passed check. On Windows, run the `.ps1` twin of either command.

### Requirements

| tool | needed for | if it is missing |
| --- | --- | --- |
| `git` | everything | nothing works |
| POSIX `sh` | the checks | run the `.ps1` twin instead |
| `pwsh` | the PowerShell half of every check | those checks report a skip |
| `node` | the file writer and the record writer | those two helpers are unavailable |
| `jq` | comparing the two halves of a check pair | that comparison reports a skip |
| `shellcheck` | linting the shell scripts | that check reports a skip |
| `gh` | studying an external repository through an authenticated route | a credential-free public route is used instead |
| the pinned Rust toolchain | building, testing and running the crates | `rustup` reads [`rust-toolchain.toml`](rust-toolchain.toml) and installs it on the first `cargo` command |

---

## Where the data lives, and where it is published

⭐ **Three branches, and each answers a different question.**

| branch | what is on it | who wants it |
| --- | --- | --- |
| the default branch | the code, the checks, the documents and the reference corpus. ⛔ **No corpus at all** since 2026-09-04 | somebody reading or changing how this works |
| ⭐ **`source`** | the canonical corpus: `corpus/`, `raw/`, `vectors/` and the licence, and nothing else | somebody reviewing a capture as a diff |
| **`data`** | what the assembler derives from `source`: the corpus, the raw bytes, every generated format, the flat fetchable routes, the per-build trust-anchor lists, an index, a manifest and a checksums file | ⭐ **a program.** This is the surface to fetch |

⭐ **The canonical corpus is committed as JSON**, one file per profile with the
raw bytes beside it, because that is what makes an automated capture reviewable:
a change shows up as a diff a person reads rather than as an opaque artefact
somebody has to fetch and compare.

⛔ **It is on its own branch so that the derivation stays checkable.** The data
branch is a FUNCTION of the source branch, and while both the source and the
derivation sat on one branch the check that asks "is what is published what the
corpus derives to" could compare the published branch against a copy of itself.
It did, once, and reported success.

⛔ **The data branch is append-only, it is never force-pushed, and a build that
would change nothing writes nothing.** A push to `source` or to the default
branch is what updates it: a capture changes the corpus, and a code change can
change what the assembler emits from an unchanged one.

```bash
git fetch origin data
```

```bash
git fetch origin source
```

### ⭐ Verifying that a release came from this repository

⛔ **A checksums file published beside the artefact proves TRANSPORT, not
authorship**, because whoever could replace one could replace the other. Every
release is attested with the runner's own identity, so a consumer can check
authorship independently:

```bash
gh attestation verify b-ids-corpus-v1.0.0.tar.gz --repo Azathothas/b-ids
```

⭐ **There is no key to fetch and none to trust.** The signature is issued
against the workflow's own OIDC identity, which names the repository, the
workflow and the commit that produced the file. ⛔ Nothing in this repository
holds a private key, and no workflow in it names a secret.

⚠ **No release has been cut yet**, so there is nothing to run that against
today. `TODO/publish.md`, `PUB-09`.

⚠ **No release has been cut**, because a pushed tag is the only thing that cuts
one. `PUB-04`, the ready-to-paste snippets for somebody else's client, is the
surface that does not exist yet. [`TODO/PROGRESS.md`](TODO/PROGRESS.md) carries
the order.

---

## What `latest` means, and what it never means

⛔ **`latest` means stable and nothing else.** A consumer following it is never
handed a pre-release build, because that is the same failure as shipping a
version nobody runs yet. The pointer file at `corpus/v1/latest.json` keeps two
maps for that reason: `latest`, keyed by browser and platform, which is built
from stable profiles alone, and `per_channel`, which carries every channel under
its own name.

⭐ **Beta and canary are published beside it, clearly labelled, and capturing
them is the mechanism rather than an extra.** The profile for the next stable is
ready the day it ships, because it was captured weeks earlier under another
name.

⛔ **A beta profile is never promoted into the stable path when it ships.** It is
a different capture of a different build, and the stable build gets its own.

⛔ **Historical versions are out of scope.** The corpus accretes going forward,
which is what a dated append-only corpus is. There is no backfill. A historical
profile contributed from outside is accepted with `vendor` provenance and stays
a draft unless somebody can capture the build, because a value nobody can
re-measure is a value nobody should trust.

```bash
sh scripts/common/check-routes.sh --assert-latest-is-stable
```

---

## What it does, and how far each part has got

| | where it stands |
| --- | --- |
| **Capture** from a real browser, off a socket, from outside the client, so the reading is what the browser sent rather than what it intended. | ⭐ done, over three hosts and two browser families |
| **Record everything the wire carried**, including the raw bytes, so a later parser can rebuild the corpus when this one turns out to be wrong. | ⭐ done. Every profile has the `ClientHello` it was read from beside it |
| **Validate** a claimed fingerprint for coherence: version against version, platform against platform, brand against build, and a handshake against the version it claims. | ⭐ done, as a library, a command and a published JSON schema |
| **Publish** in two ways that fail differently: tagged releases, and flat fetchable paths a `curl` one-liner can read with no index and no token. | ⚠ half. The flat paths are on the data branch; no release has been cut |
| **Say what it cannot do**, per stack, as a support matrix with the holes left in. A hole is the most useful cell in it. | ⭐ done, generated from a run rather than maintained by hand |
| **Hand a program a profile**, with no network in the path and no substitute for a platform nobody captured. | ⭐ done, as a crate with the corpus embedded at build time |

## What it does not do

- ⛔ **It is not about the browser fingerprint most readers mean.** Canvas,
  WebGL, audio, fonts and `Intl` are a different surface, measured from
  JavaScript inside a page. This one measures the **network** surface.
- ⛔ **It does not defeat, solve or retry past bot challenges or CAPTCHAs**, and
  it ships no scraping client. Emitters are reference snippets and libraries so
  a consumer can configure their own stack.
- ⛔ **It never redistributes a browser.** Measurements, versions, digests and
  the URL a build was fetched from. Never the binary.
- ⛔ **It never captures from a real profile.** Only a browser the harness
  launched itself, into a throwaway profile, having visited nothing. Header
  values are gated behind a switch, a credential keeps its name and loses its
  value under either policy, and the default shape carries names only.
- ⛔ **The unit is a browser build, never a person.** Nothing here profiles,
  targets or identifies an individual, and no capture is taken from anybody's
  traffic.
- ⛔ **It does not backfill history.** The corpus accretes going forward.

---

## Licence

**0BSD**, for the code and for everything the project generates.
[`LICENSE`](LICENSE) is the text. It imposes no attribution requirement, so a
consumer can embed a profile in a binary or a generated header without carrying
a notice file.

⚠ **Two limits, both real.**

- **A digest specification can carry its own terms.** JA4 itself is BSD
  3-Clause; the JA4+ family is separately licensed, restricted for monetisation,
  and patent pending.
  [`docs/reference-sweeps/findings.md`](docs/reference-sweeps/findings.md)
  finding 5 has the detail.
- **The corpus in [`references/`](references/) is not this project's.** It is
  nineteen other repositories' trees: eighteen are the evidence behind that
  document and one is a corpus of test vectors a check here runs against. Each
  is under its own licence, and 0BSD covers what this project writes and
  generates rather than what it quotes.
- **The vendored source in [`vendor/`](vendor/) is not this project's either.**
  It is compiled by this tree and patched here, under its own licence, at the
  commit [`vendor/upstream.json`](vendor/upstream.json) records.

---

## Documentation

| file | what it answers |
| --- | --- |
| [`TODO/PROGRESS.md`](TODO/PROGRESS.md) | where the work is, and what is next |
| [`docs/AGENTS.md`](docs/AGENTS.md) | how an agent works on this repository. The only router. |
| [`TODO/RULES.md`](TODO/RULES.md) | how this repository is worked on, and what each rule cost |
| [`TODO/INDEX.md`](TODO/INDEX.md) | every work item, one line each |
| [`docs/inherited-claims.md`](docs/inherited-claims.md) | every value carried from somewhere else, with its source and status |
| [`docs/glossary.md`](docs/glossary.md) | the terms, and the caveat attached to each |
| [`SECURITY.md`](SECURITY.md) | the threat model, and where to report something |
| [`docs/reference-sweeps/findings.md`](docs/reference-sweeps/findings.md) | what eighteen other repositories do, and get wrong |
| [`vendor/upstream.json`](vendor/upstream.json) | what third-party source this tree compiles, and at which commit |
| [`patches/README.md`](patches/README.md) | every local change to that source, and how to tell when a release retires one |
| [`CHANGELOG.md`](CHANGELOG.md) | what shipped, when, and where the evidence is |
| [`docs/HISTORY/README.md`](docs/HISTORY/README.md) | what was believed here and later withdrawn |

## Contributing

⚠ **The project is in beta and nobody consumes its data yet**, so there is no
process to point at. Two rules apply from the first contribution whenever it
comes: contributed data is 0BSD, and a contributed profile the project cannot
re-measure is a draft and stays one.
