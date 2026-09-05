# Scripts

Run scripts from the repository root. Use the `.sh` form on POSIX systems and
the `.ps1` twin on Windows. Where both exist, they must produce equivalent
results and exit codes.

## Contract

- Exit `0`: the requested operation completed or the assertion passed.
- Exit `1`: the operation ran and found a failure.
- Exit `2`: prerequisites or state prevented the operation from running.
- A gate reports exit `2` as a named skip; strict CI treats skips as failures.
- Read the producing process's exit code directly. Do not infer success from
  output piped through another command.
- Checks that accept `--json` or `-Json` emit one versioned JSON object.
- Fixture modes must exercise the same implementation as the real check.

Run the complete local gate with:

```sh
sh scripts/common/check-gate.sh --strict
```

```powershell
pwsh -NoProfile -File scripts/common/check-gate.ps1 -Strict
```

## Repository checks

| command | assertion |
| --- | --- |
| `check-bindings` | generated non-Rust bindings answer identically |
| `check-catalogues` | every first-party script and document has an index entry |
| `check-changelog` | changelog entries satisfy release-record rules |
| `check-cold-start` | the cold-start workflow uses no warm or local-only state |
| `check-control-bytes` | first-party text contains no undeclared control bytes |
| `check-corpus` | versioned corpus files are append-only and profile-valid |
| `check-coverage` | capture-matrix cells and published profiles agree |
| `check-data-branch` | regenerated publication output equals the data branch |
| `check-docs` | links, cited paths, vocabulary, and document reachability are valid |
| `check-exit-codes` | scripts distinguish checked failures from inability to run |
| `check-formats` | every aggregate format regenerates from one corpus model |
| `check-gate` | all locally applicable checks run under one verdict |
| `check-generated-configs` | emitted stack configurations match snapshots |
| `check-license-consistency` | code, schema, profiles, data, and release metadata agree on 0BSD |
| `check-line-endings` | Git attributes, index bytes, and worktree bytes agree |
| `check-manual-path` | each automated operation documents a manual equivalent |
| `check-markers` | first-party text uses only the documented warning markers |
| `check-msrv` | declared Rust floor and pinned compiler satisfy the lockfile |
| `check-no-secrets` | first-party/public files contain no credential material |
| `check-notes-generator` | release notes and changelog derive from the same generator |
| `check-one-home` | long first-party prose sentences are not duplicated |
| `check-packages` | generated packages build offline and identify embedded data |
| `check-pcap` | generated packet captures contain each profile's recorded bytes |
| `check-placeholders` | template placeholders do not survive into live files |
| `check-powershell` | all PowerShell files parse and pass PSScriptAnalyzer |
| `check-pr-body` | automated capture pull requests contain required provenance |
| `check-provisioning` | browser provisioning keeps both disposable-host guards |
| `check-publish` | publication workflows declare triggers, inputs, permissions, and concurrency |
| `check-record` | the archived work index, entries, statuses, and counts agree |
| `check-release` | release assembly is deterministic and complete |
| `check-remote-items` | open pull requests, action pins, and dependency updates are reviewable |
| `check-routes` | flat field routes match the profiles they expose |
| `check-signing` | release attestations and verification instructions agree |
| `check-sources` | external version discovery retains independent sources |
| `check-staleness` | captured stable versions are compared with vendor releases |
| `check-support-matrix` | every support cell cites a run or a pinned source |
| `check-trust-anchors` | per-build anchor artifacts match extension bodies |
| `check-twins` | POSIX and PowerShell check pairs return equivalent results |
| `check-validate` | profiles validate and deterministic generation repeats byte-for-byte |
| `check-vendor` | vendored trees, patches, and upstream metadata agree |
| `check-workflows` | workflow metadata and fail-fast policy satisfy the CI contract |

## Helpers and operations

| command | purpose |
| --- | --- |
| `bindings-answers` | fixture program used by binding checks |
| `corpus-root` | resolve a local, environment-supplied, or branch-backed corpus root |
| `derive-ja4-vector` | derive a reviewable JA4 vector from one published profile |
| `doctor` | report host, toolchain, repository, shell, network, WSL, and container capabilities |
| `git-sync` | validate, commit with configured identity, push, and verify the remote SHA |
| `mine-repo` | fetch a pinned upstream repository and provenance for reference review |
| `provision-browser` | remove and install a browser build on a disposable CI host |
| `set-record` | update the archived record format when validating historical fixtures |
| `vendor-diff` | compare a vendored tree with its recorded upstream source |
| `vendor-sync` | reconstruct vendored source and apply the patch series |
| `write-file` | write content atomically with explicit encoding and line endings |

⛔ `provision-browser` is destructive and requires both `CI` and
`B_IDS_DISPOSABLE=1`; never bypass either guard on a developer machine.

⚠ `git-sync`, `mine-repo`, `vendor-sync`, and remote modes can change refs,
download content, or write repository state. Inspect their target and dry-run
surface before use. The remaining checks are read-only apart from ignored
temporary directories.
