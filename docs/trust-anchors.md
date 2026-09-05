# Trust-anchor extension

Several Chromium-family profiles carry TLS extension codepoint `0xca34`. The
wire data establishes the codepoint, length, body bytes, and capture conditions.
The name `draft-ietf-tls-trust-anchor-ids` remains an inference until the draft
encoding is checked against these bytes.

## Encoding

The measured body begins with a two-byte big-endian payload length. The payload
is a sequence of one-byte-length-prefixed identifiers. Identifiers are
published as lowercase hexadecimal in browser order.

Malformed lengths or identifiers are publication errors. A profile that lacks
the extension is distinct from a profile that carries a two-byte empty list.

## Initial-release measurements

| browser | version | platform | extension bytes | identifiers |
| --- | --- | --- | ---: | ---: |
| Chrome | `151.0.7922.76` | `linux64` | 2 | 0 |
| Chrome | `151.0.7922.76` | `win64` | 2 | 0 |
| Chrome | `152.0.7977.75` | `linux64` | 206 | 32 |
| Chrome | `152.0.7977.76` | `win64` | 206 | 32 |
| Chrome | `152.0.7977.82` | `linux64` | 206 | 32 |
| Chrome | `152.0.7977.83` | `win64` | 206 | 32 |
| Chromium | `152.0.7977.75` | `linux64` | 206 | 32 |

The generated artifacts are published under
[`anchors/`](https://github.com/Azathothas/b-ids/tree/data/anchors). Each file
records its profile, browser, version, platform, capture instant, declared
extension length, and ordered identifiers.

These measurements establish a difference between the captured 151 and 152
builds. They do not isolate branding: the empty and non-empty observations are
not a controlled same-version branded/unbranded comparison.

## Consumer guidance

To reproduce a measured build exactly, use the extension body from that build's
profile. Reusing a list from another build advertises the other build's root
store snapshot. Omitting an extension that the target build sent changes its
ordered extension list. Sending an empty list is correct only for a build whose
measurement contains that empty body.

This page asserts no preference among the three implementation options:

### Omit the extension

This changes the extension set for a build measured with `0xca34`, but may be
appropriate when exact reproduction is not required.

### Carry a captured list

This reproduces one build's measured body. It must be refreshed with the target
build because a root-store snapshot ages independently of other fields.

### Send it empty

This reproduces the measured Chrome 151 profiles listed above. It is not a
substitute for captured 152 profiles, which carry 32 identifiers.

⚠ Identifier order may vary by connection. A single capture per build cannot
distinguish a stable build order from a per-connection shuffle, so fixed-order
claims require a controlled repeated-connection measurement.

## Inference boundary

[`inherited-claims.md`](inherited-claims.md) records the proposed extension name
and its evidence status. Until the relevant specification is read against the
measured encoding, code and documentation must describe the name as inferred.
Raw bytes remain authoritative.
