# b-ids (Browser-IDs)

A dated, provenance-carrying corpus of what real browsers put on a wire: TLS
`ClientHello` bytes, HTTP/2 settings and framing, and header sets in order.

⭐ **The corpus holds six profiles**, and every one is real: Chrome
`151.0.7922.76` on Windows captured 2026-09-01 on a laptop, Chrome
`151.0.7922.174` and `151.0.7922.173` captured 2026-09-02 on hosted runners on
Windows and Linux, and three more from 2026-09-02 whose builds this project chose
rather than inherited: Chrome `152.0.7977.75` on Linux and `152.0.7977.76` on
Windows from the vendor's own channel, and Edge `151.0.4129.101` on Linux from
the vendor's enterprise index, each naming the URL and the digest it was
installed from. Every profile carries the `ClientHello` bytes it was read from,
published beside it. Around them are
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

⚠ **These are all this repository can do today**: run its own checks, and read.

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

---

## Where the data lives, and where it will be published

⭐ **The canonical corpus is committed on the default branch**, as JSON, one file
per profile, with the raw bytes beside it. That is deliberate and it is what
makes an automated capture reviewable: a change to a profile shows up as a diff
a person reads, rather than as an opaque artefact somebody has to fetch and
compare.

⛔ **The default branch is not the publishing surface.** Nothing is published
from this repository yet. When it is, it goes to an orphan **data branch**
carrying only generated artefacts, with dated snapshots, a `latest` pointer, an
index and a checksums file, append-only and never force-pushed. The generated
formats, the flat fetchable routes and the ready-to-paste client snippets are
generated from the canonical corpus rather than maintained beside it.

⚠ **None of that exists today**, and the entries that build it are named so the
gap is visible rather than implied: `SCHEMA-08` and `SCHEMA-12` are the
generator and its nine formats, `PUB-01` the releases, `PUB-02` the data branch,
`PUB-03` the routes, and `PUB-04` the artefacts that are not data files.
[`TODO/PROGRESS.md`](TODO/PROGRESS.md) carries the order.

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

## What it will do

- **Capture** from a real browser, off a socket, from outside the client, so
  the reading is what the browser sent rather than what it intended.
- **Record everything the wire carried**, including the raw bytes, so a later
  parser can rebuild the corpus when this one turns out to be wrong.
- **Validate** a claimed fingerprint for coherence: version against version,
  platform against platform, brand against build, and a handshake against the
  version it claims.
- **Publish** in two ways that fail differently: tagged releases, and flat
  fetchable paths a `curl` one-liner can read with no index and no token.
- **Say what it cannot do**, per stack, as a support matrix with the holes left
  in. A hole is the most useful cell in it.

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
  eighteen other repositories' trees, kept as the evidence behind that document,
  each under its own licence. 0BSD covers what this project writes and
  generates, not what it quotes.
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
| [`docs/reference-sweeps/findings.md`](docs/reference-sweeps/findings.md) | what eighteen other repositories do, and get wrong |
| [`vendor/upstream.json`](vendor/upstream.json) | what third-party source this tree compiles, and at which commit |
| [`patches/README.md`](patches/README.md) | every local change to that source, and how to tell when a release retires one |
| [`CHANGELOG.md`](CHANGELOG.md) | what shipped, when, and where the evidence is |
| [`docs/HISTORY/README.md`](docs/HISTORY/README.md) | what was believed here and later withdrawn |

## Contributing

⚠ **Not yet.** There is nothing to contribute to until the schema and the
harness exist. When that changes, two rules apply from the first contribution:
contributed data is 0BSD, and a contributed profile the project cannot
re-measure is a draft and stays one.
