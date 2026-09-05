# Architecture

`b-ids` captures a browser's network identity, validates the result, stores the
canonical measurement, and deterministically publishes consumer formats.

## Profile model

A profile represents one browser, build, channel, platform, and capture instant.
Its published JSON contains:

| section | contents |
| --- | --- |
| `tls` | `ClientHello` fields in wire order, including unknown codepoints |
| `http2` | ordered frames, settings, connection window, and priority data |
| `http` | ordered request headers for each captured request kind |
| `raw` | the source bytes required to reproduce parser output |
| `captured` | time, tool, acquisition, trust, resumption, and connection conditions |
| `provenance` | source classification for each recorded field |

The machine contract is
[`browser-profile-1.schema.json`](../crates/b-ids-schema/schema/browser-profile-1.schema.json).
`Profile::check` enforces cross-field invariants before a profile can be stored
or published.

Digests are derived from a profile and are not stored as measurements. The raw
`ClientHello` remains authoritative for questions a digest discards, including
GREASE values and shuffled extension order.

## Workspace components

| crate | responsibility |
| --- | --- |
| [`b-ids-schema`](../crates/b-ids-schema/) | profile types, invariants, schema, routes, and digest input rendering |
| [`b-ids-harness`](../crates/b-ids-harness/) | server-side TLS and HTTP/2 capture from bytes received on the socket |
| [`b-ids-driver`](../crates/b-ids-driver/) | browser discovery, acquisition metadata, and isolated launch |
| [`b-ids-corpus`](../crates/b-ids-corpus/) | capture conversion, append-only storage, indexing, routes, packages, and publication assembly |
| [`b-ids-validator`](../crates/b-ids-validator/) | coherence validation and comparison against supported dimensions |
| [`b-ids-emit`](../crates/b-ids-emit/) | byte/configuration emission and support-matrix generation |
| [`b-ids`](../crates/b-ids/) | offline library with corpus data embedded at build time |
| [`b-ids-cli`](../crates/b-ids-cli/) | minimal profile-backed connection client |
| [`b-ids-conformance`](../crates/b-ids-conformance/) | field-level comparison of an observed client with a claimed profile |

`b-ids-harness` is a server, not a browser client. `b-ids-driver` launches a
browser but does not parse captures. `b-ids-validator` has no network access.
`b-ids-cli` intentionally omits general HTTP-client behavior such as cookies,
redirects, and retries.

## Capture flow

```text
browser build
  | b-ids-driver resolves and launches an isolated profile
  v
local b-ids-harness socket
  | reads TLS records and HTTP/2 frames as received
  v
connection captures
  | selection chooses the first cold TLS hello and first HTTP/2 connection
  v
normalized Profile
  | Profile::check and Store::add
  v
source branch: corpus/v1/... + raw/v1/...
  | deterministic publication assembly
  v
data branch and tagged release archive
```

A browser may open, abandon, or resume several connections during one
navigation. TLS and HTTP/2 halves can therefore come from different connection
numbers; the profile records both. Resumed TLS handshakes are not substituted
for cold handshakes.

Capture uses a disposable browser profile and a local endpoint. The trust route
and browser acquisition are recorded as capture conditions. Provisioning scripts
refuse hosts that do not present both CI and explicit disposable-runner guards.

## Branch authority

| branch | authority |
| --- | --- |
| `main` | implementation, tests, workflows, documentation, reference evidence, and vendored code |
| `source` | canonical reviewed profiles, raw captures, vectors, and license |
| `data` | generated consumer tree derived from `source` |

`main` deliberately carries no profile corpus. All readers resolve the corpus
root through the shared shell/PowerShell helpers or the Rust root resolver.
This prevents a validator from accidentally comparing generated output with a
second copy of itself.

The `source` branch is append-only for versioned profile and raw-capture paths.
Derived indexes may be regenerated. The `data` branch is replaced only by a
successful deterministic assembly and is never force-pushed.

## Publication

`b-ids-corpus::publish` produces one sorted tree containing:

- canonical profiles and raw captures;
- current and per-channel pointers;
- flat routes for individual fields;
- aggregate formats and language/package bindings;
- generated client and detector configuration;
- trust-anchor lists and packet captures;
- schema vectors, `MANIFEST.json`, and `SHA256SUMS`.

The assembler reads no clock and performs no remote write. Workflows publish its
output only after validation. A no-change rebuild produces no data-branch
commit.

## Failure behavior

- Existing versioned profile routes are not overwritten. A correction is a new
  profile that records what it supersedes.
- Missing dimensions return no profile; they are not approximated from another
  browser or platform.
- Malformed measured bytes fail conversion or publication. Absence is accepted
  only where the schema explicitly permits it.
- Conformance reports `conforms`, `differs`, or `not-checkable`; per-connection
  variation is never counted as a pass.
- Generator output is sorted and checked against a second regeneration.
- Script exit code 2 means the check could not run and is reported as a skip,
  never success.

## External source boundaries

`references/` contains pinned evidence and conformance vectors. Tests read the
NSS and HPACK reference trees directly, so the directory is an operational
input as well as review evidence. `vendor/` contains code compiled by the
workspace, and `patches/` records local modifications. Each retains its
upstream license and provenance.

See [`inherited-claims.md`](inherited-claims.md) for facts not measured by this
project, [`trust-anchors.md`](trust-anchors.md) for the inferred-name boundary,
and [`reference-sweeps/findings.md`](reference-sweeps/findings.md) for upstream
review evidence.
