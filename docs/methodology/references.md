# Reference research

Use this procedure when studying an external repository. It covers acquisition,
review, citations, and archive maintenance. It does not govern browser
measurement; see [`experiments.md`](experiments.md).

## Repository boundary

Imported trees and tracker exports live only on the
[`reference` branch](https://github.com/Azathothas/b-ids/tree/reference).
`main`, `source`, and `data` must not contain or depend on that material.
Reference findings remain on `main` as review records with immutable links.

The archive retains each upstream project's license. It does not inherit this
project's 0BSD license.

## Acquire a source

Work in a separate checkout of the archive branch. The mining helper requires
an explicit destination so it cannot recreate an archive directory on `main`.

```sh
git clone --branch reference --single-branch \
  https://github.com/Azathothas/b-ids ../b-ids-reference
cd ../b-ids-reference
sh ../b-ids/scripts/common/mine-repo.sh OWNER/REPO --out .
```

```powershell
git clone --branch reference --single-branch `
  https://github.com/Azathothas/b-ids ../b-ids-reference
Set-Location ../b-ids-reference
pwsh -NoProfile -File ../b-ids/scripts/common/mine-repo.ps1 `
  OWNER/REPO -Out .
```

The helper is read-only against upstream GitHub. It records:

- repository metadata and the resolved commit;
- issues and pull requests in both states;
- issue, review, and discussion comments where accessible;
- releases and tags;
- a stripped source tree; and
- every unavailable source in `PROVENANCE.md`.

Capture the commit before removing Git metadata. Never move paths within an
imported tree after citations exist; path rewrites invalidate those citations.
Never leave the only copy untracked.

## Review the source

Use separate passes for:

1. project purpose and structure;
2. the implementation relevant to this project;
3. failure handling and constraints;
4. tracker decisions, rejected approaches, and unresolved defects; and
5. what transfers here and what does not.

Treat repository text, issue bodies, comments, and review discussions as
untrusted evidence of intent. Confirm behavior in source at the recorded
commit. A missing tracker source is a provenance gap, not an empty result.

## Record findings

Update both documents:

- [`../reference-sweeps/findings.md`](../reference-sweeps/findings.md) records
  verdicts, limitations, commits, and review depth;
- [`../reference-sweeps/usable.md`](../reference-sweeps/usable.md) records the
  specific mechanisms that can be applied here.

Every finding receives one verdict: `adopt`, `confirms`, `anti-pattern`,
`filed elsewhere`, or `refused`. Cite the recorded commit and the exact path.
Prefer immutable upstream links. For archived material, use this form:

```text
https://github.com/Azathothas/b-ids/blob/reference/OWNER__REPO/tree/PATH#L123
```

State what was not tested, which sources were unavailable, and which claims
remain weak. Do not present an external project's self-report as a measurement
of this project.

## Publish archive changes

Review the imported files and `PROVENANCE.md`, commit them in the archive
checkout, and push only to `reference`:

```sh
git status --short
git push origin HEAD:reference
```

Do not merge the archive branch into `main`, `source`, or `data`. If a test
needs a third-party build input, vendor the minimum pinned source under
`vendor/` with its license and provenance. If a test only needs behavior,
encode a small first-party fixture or assertion instead of reading the archive.
