# AGENTS.md

These instructions apply to the entire repository.

## Precedence

Follow, in order:

1. the operator's current request;
2. this file;
3. the document that owns the affected subsystem;
4. nearby code comments.

When two sources disagree, stop relying on the weaker source and correct the
contradiction in the same change.

## Start here

1. Read this file and inspect the working tree.
2. Run the host doctor:

   ```sh
   sh scripts/doctor/doctor.sh
   ```

   On Windows:

   ```powershell
   pwsh -NoProfile -File scripts/doctor/doctor.ps1
   ```

3. Run the fast local gate before editing:

   ```sh
   sh scripts/common/check-gate.sh --fast
   ```

   On Windows:

   ```powershell
   pwsh -NoProfile -File scripts/common/check-gate.ps1 -Fast
   ```

4. Read the owning document from the routing table below. For a continued
   multi-session change, also read `docs/history/RESUME.md`.

## Repository invariants

- A browser fingerprint is measured from a real browser at a named build and
  instant. Fixtures, inferred names, and inherited claims are never published
  as measurements.
- Preserve raw capture bytes beside normalized profiles. Parser changes must be
  recoverable from those bytes.
- `main` contains code, tests, documentation, vendored source, and evidence;
  it contains no published corpus.
- `source` contains the canonical reviewed profiles, raw captures, vectors,
  and license.
- `data` is generated from `source`; do not edit it by hand or force-push it.
- `latest` selects stable profiles only. Pre-release channels remain explicit.
- Published packages embed data and must work without network access.
- Preserve measured values, failures, samples, conditions, and provenance.
  Formatting or route changes do not authorize semantic data changes.
- Scripts use exit code 0 for success, 1 for a checked failure, and 2 when the
  check could not run. A skipped check is never described as passed.
- Keep POSIX and PowerShell twins behaviorally equivalent. Run
  `scripts/common/check-twins.sh` after changing either half.
- Use the Rust version pinned by `rust-toolchain.toml`, keep the lockfile
  reproducible, and deny compiler and linter warnings.
- Third-party workflow actions use immutable commit SHAs.
- Keep imported evidence under `references/` byte-preserved and vendored
  build inputs under `vendor/`; neither inherits this project's license.

## Layout

| path | purpose |
| --- | --- |
| `crates/` | Rust schema, capture, validation, generation, library, and CLI code |
| `experiments/` | live capture and identification instruments used by workflows |
| `references/` | pinned upstream evidence and external conformance vectors |
| `vendor/` | third-party source compiled by this workspace |
| `patches/` | local changes applied to vendored source |
| `scripts/common/` | checks, generators, provisioning, and repository operations |
| `scripts/doctor/` | read-only host capability probe |
| `docs/history/` | retired plans, reviews, and superseded documentation |
| `.github/workflows/` | CI, capture, publication, provisioning, and staleness jobs |

## Documentation routes

| subject | owner |
| --- | --- |
| architecture and branch model | `architecture.md` |
| human review boundary | `HUMAN.md` |
| terminology | `glossary.md` |
| inherited facts and provenance | `inherited-claims.md` |
| trust-anchor observations | `trust-anchors.md` |
| container validation | `containers.md` |
| code conventions | `conventions/code.md` |
| document conventions | `conventions/docs.md` |
| forbidden patterns | `conventions/forbidden-patterns.md` |
| Git conventions | `conventions/git.md` |
| prose conventions | `conventions/prose.md` |
| shell conventions | `conventions/shell.md` |
| authoring method | `methodology/authoring.md` |
| experiment method | `methodology/experiments.md` |
| validation gate | `methodology/gate.md` |
| repository initialization | [`methodology/initialize.md`](docs/methodology/initialize.md) |
| agent-tool limitations | `agent-tooling.md` |
| historical-record policy | `methodology/history.md` |
| reference acquisition | `methodology/references.md` |
| review method | `methodology/reviews.md` |
| session handoff | `methodology/sessions.md` |
| vendoring method | `methodology/vendoring.md` |
| archived work-record method | `methodology/work-todo.md` |
| upstream findings | `reference-sweeps/findings.md` |
| usable upstream techniques | `reference-sweeps/usable.md` |
| remote-operation safety | `security/remote-ops.md` |
| secret handling | `security/secrets.md` |
| retired material | `history/README.md` |

All paths in this table are relative to `docs/`. Script contracts are indexed
by `scripts/README.md`; vendored patches are indexed by `patches/README.md`.

## Validation

Run the complete gate before committing:

```sh
sh scripts/common/check-gate.sh --strict
```

On Windows:

```powershell
pwsh -NoProfile -File scripts/common/check-gate.ps1 -Strict
```

Also run the supported disposable-Linux procedure in `docs/containers.md`
when changing platform behavior, release machinery, shell code, or the gate.
Fix failures at their source; do not reduce coverage, relax assertions, or
convert a failure to a skip.

## Remote changes

Inspect authentication, repository state, and the exact target before changing
GitHub settings or refs. Use the configured Git author and committer identity.
Do not add tool, assistant, or co-author attribution. Push only a clean,
validated tree and verify the remote SHA afterwards.
