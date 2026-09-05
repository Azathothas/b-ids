# Experiment history

These scripts record one-off measurements completed during implementation.
They are retained for reproducibility and are not called by current workflows.
See [`../../methodology/experiments.md`](../../methodology/experiments.md) for
the current evidence requirements.

## Graduated entry points

Two repeatable procedures became maintained capture tooling:

| historical experiment | current entry point |
| --- | --- |
| `10-first-profile.sh` | [`scripts/capture/profile.sh`](../../../scripts/capture/profile.sh) |
| `50-trust-anchor.sh` | [`scripts/capture/trust-anchor.sh`](../../../scripts/capture/trust-anchor.sh) |

## Archived experiments

| script | question measured |
| --- | --- |
| [`20-compare-capture-modes.sh`](20-compare-capture-modes.sh) | whether completing a handshake changes the pre-handshake capture |
| [`30-resumption-control.sh`](30-resumption-control.sh) | whether refusing tickets changes the cold hello |
| [`40-trust-paths.sh`](40-trust-paths.sh) | which trust route allows browser capture on each platform |
| [`60-identify-extension.sh`](60-identify-extension.sh) | whether an unidentified TLS extension could be named |

Each script writes evidence under the ignored scratch directory and records its
host, tool, browser, sample, and trust conditions. The resulting architectural
findings are summarized in [`../../architecture.md`](../../architecture.md).
