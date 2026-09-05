# b-ids

`b-ids` is a dated, provenance-carrying corpus of the network identity emitted
by real browser builds: TLS `ClientHello` bytes, HTTP/2 settings and framing,
and ordered request headers. It includes capture, validation, publication, and
offline-consumption tooling.

## Current data

The primary result set is
[`corpus/v1/latest.json`](https://raw.githubusercontent.com/Azathothas/b-ids/data/corpus/v1/latest.json).
It maps each supported browser/platform pair to its current stable profile and
lists current profiles for every captured channel. Consumers should follow that
map instead of hard-coding a version.

Direct current results:

| result | browser/platform | link |
| --- | --- | --- |
| User-Agent | Chrome stable / Linux | [`navigate.txt`](https://raw.githubusercontent.com/Azathothas/b-ids/data/routes/user-agent/chrome/stable/latest/linux64/navigate.txt) |
| User-Agent | Firefox stable / Windows | [`navigate.txt`](https://raw.githubusercontent.com/Azathothas/b-ids/data/routes/user-agent/firefox/stable/latest/win64/navigate.txt) |
| ClientHello | Chrome stable / Linux | [`linux64.txt`](https://raw.githubusercontent.com/Azathothas/b-ids/data/routes/client-hello-hex/chrome/stable/latest/linux64.txt) |
| header order | Chrome stable / Linux | [`navigate.list.txt`](https://raw.githubusercontent.com/Azathothas/b-ids/data/routes/header-order/chrome/stable/latest/linux64/navigate.list.txt) |
| complete route index | all fields and profiles | [`routes.json`](https://raw.githubusercontent.com/Azathothas/b-ids/data/routes/routes.json) |

Aggregate formats are published as
[`JSON`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.json),
[`NDJSON`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.ndjson),
[`CSV`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.csv),
[`TSV`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.tsv),
[`YAML`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.yaml),
[`TOML`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.toml),
[`SQL`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.sql), and
[`Protocol Buffers`](https://raw.githubusercontent.com/Azathothas/b-ids/data/formats/corpus.proto).
Specialized outputs remain discoverable under the data branch's
[`configs/`](https://github.com/Azathothas/b-ids/tree/data/configs),
[`anchors/`](https://github.com/Azathothas/b-ids/tree/data/anchors),
[`packages/`](https://github.com/Azathothas/b-ids/tree/data/packages), and
[`pcap/`](https://github.com/Azathothas/b-ids/tree/data/pcap) directories.

## Data guarantees

- A profile is measured from a real browser at a named build and capture
  instant. Inherited values and fixtures are not published as measurements.
- Every normalized profile retains the raw `ClientHello` bytes from which it
  was parsed.
- Per-field provenance distinguishes measured, assumed, substituted, and
  imported values.
- `latest` selects stable profiles only; pre-release channels remain explicit.
- The `data` branch is generated from `source` and includes `MANIFEST.json` and
  `SHA256SUMS` for integrity checks.
- Published packages embed the corpus and do not fetch data at runtime.

⚠ Coverage is the set named by `latest.json`, not a complete browser matrix.
A missing browser, channel, platform, or version is absent data and must not be
silently substituted.

## Repository branches

| branch | contents | authority |
| --- | --- | --- |
| `main` | code, tests, documentation, workflows, vendored source, and pinned evidence | implementation |
| `source` | reviewed profiles, raw captures, vectors, and license | canonical measurements |
| `data` | deterministic publication output derived from `source` | consumer surface |

⛔ Do not edit `data` directly or force-push `source` or `data`. A capture
changes `source`; the publication workflow regenerates `data`.

## Verify a release

Release archives are checksummed and receive GitHub build-provenance
attestations. For the initial release:

```bash
gh release download v0.0.1 --repo Azathothas/b-ids --pattern 'b-ids-corpus-v0.0.1.tar.gz'
gh attestation verify b-ids-corpus-v0.0.1.tar.gz --repo Azathothas/b-ids
```

A checksum beside an archive verifies transport; the attestation binds the
archive to this repository, workflow, and commit.

## Build and validate

The Rust version is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). The
repository also requires Git, PowerShell for `.ps1` checks, a POSIX shell for
`.sh` checks, `jq`, Node.js, ShellCheck, and PSScriptAnalyzer for the complete
cross-platform gate.

Start with the read-only host probe:

```bash
sh scripts/doctor/doctor.sh
```

On Windows:

```powershell
pwsh -NoProfile -File scripts/doctor/doctor.ps1
```

Run the complete gate before publishing:

```bash
sh scripts/common/check-gate.sh --strict
```

```powershell
pwsh -NoProfile -File scripts/common/check-gate.ps1 -Strict
```

The gate covers formatting, warning-denied linting, tests, release builds,
script parsing, ShellCheck, links, schemas, manifests, generated snapshots,
aggregate regeneration, and check self-tests. See
[`docs/methodology/gate.md`](docs/methodology/gate.md) for individual commands.

## Capture and validation

`experiments/10-first-profile.sh` is the workflow-backed capture entry point.
It launches a browser in a new disposable profile and records only the network
traffic sent to the local harness. It does not inspect a user's browser profile
or capture unrelated traffic.

⚠ Run provisioning and live capture only on a disposable runner or container.
The scripts refuse unsafe hosts, preserve raw bytes, and separate capture from
publication review.

The schema and coherence validator live in `b-ids-schema` and
`b-ids-validator`. The remaining workspace crates provide the listener, browser
driver, corpus assembler, configuration emitters, conformance CLI, and embedded
library. [`docs/architecture.md`](docs/architecture.md) describes their
boundaries and data flow.

## Scope

This project measures browser network identity. It does not:

- measure canvas, WebGL, audio, font, locale, or other page-visible identity;
- bypass bot challenges or CAPTCHAs;
- ship a scraping client;
- redistribute browser binaries;
- capture from a user's existing browser profile or identify a person;
- fabricate a profile for an uncaptured platform.

## Documentation

| document | purpose |
| --- | --- |
| [`AGENTS.md`](AGENTS.md) | repository invariants, layout, and maintenance routes |
| [`docs/architecture.md`](docs/architecture.md) | technical model and branch data flow |
| [`docs/inherited-claims.md`](docs/inherited-claims.md) | claims from other projects and their provenance |
| [`docs/trust-anchors.md`](docs/trust-anchors.md) | observed trust-anchor extension behavior |
| [`docs/reference-sweeps/findings.md`](docs/reference-sweeps/findings.md) | review of pinned upstream evidence |
| [`scripts/README.md`](scripts/README.md) | script contracts and exit semantics |
| [`SECURITY.md`](SECURITY.md) | threat model and vulnerability reporting |
| [`docs/history/README.md`](docs/history/README.md) | completed plans, reviews, and superseded claims |

## License

Project code and generated data are released under
[`0BSD`](LICENSE). Imported material under `references/` and compiled third-party
source under `vendor/` retain their own licenses.
