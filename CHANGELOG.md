# CHANGELOG

What shipped, when, and where the evidence is. Newest first.

⛔ **Nothing in this project has been released.** The entries below are
repository changes, not published artefacts, and every one says so.
[`TODO/PROGRESS.md`](TODO/PROGRESS.md) is where the work stands.

---

## 2026-08-31T00:30:00Z - the repository is initialised

**Record:** [`TODO/PROGRESS.md`](TODO/PROGRESS.md), and
[`TODO/INDEX.md`](TODO/INDEX.md) for the 77 entries this created.
**Deployed:** no. Nothing is published from this repository yet, and
[`TODO/RULES.md`](TODO/RULES.md) records that as a standing fact rather than an
omission.

What landed:

- **The methodology, the conventions and the security rules.** Every document
  that named a file this project does not have was rewritten rather than
  inherited, and three roles were deliberately left unwritten rather than
  shipped as empty skeletons.
- **The gate**, in both its POSIX shell and PowerShell halves, plus the probe,
  the record checker and its writer, the file writer, the commit helper and the
  reference fetcher. ⚠ The gate contains no test suite, because there is no
  code, and both halves of the runner say so in a comment rather than reporting
  green over an absence.
- ⭐ **A sweep of eighteen repositories**, at named commits. The trees are in
  [`references/`](references/) and the write-up is in
  [`docs/reference-sweeps/`](docs/reference-sweeps/). ⭐ One of the eighteen,
  `Azathothas/bit-cli`, is the origin every inherited value was measured in,
  rather than prior art.
- **[`docs/inherited-claims.md`](docs/inherited-claims.md)**, which records every
  value this project carries that it did not measure, each cited at a file in
  the tree it was measured in.
- **[`docs/glossary.md`](docs/glossary.md)**, with the caveat attached to each
  term rather than to the page that uses it.
- **77 work entries** across eleven categories, four of which close.
- **The 0BSD licence**, a bare README, and the repository's own router at
  [`docs/AGENTS.md`](docs/AGENTS.md).

⛔ **Four inherited claims were refuted during the reading**, before any of them
had been acted on. [`docs/HISTORY/README.md`](docs/HISTORY/README.md) lists each
with the reading that took it away. One changes what this project claims about
itself, and [`README.md`](README.md) makes the narrower claim; one was refuted by
the capture the claim was quoting.

⚠ **Two defects in this repository's own tooling were found by its own checks**
and both are recorded rather than quietly repaired. Five places described a
licence filler that was not on disk, which `TOOL-09` closes by deleting the
description; and the reference fetcher stops before cloning when one of its two
routes is down, which is `TOOL-04` and is still open.

**The design brief this repository started from was never committed.** `DOC-04`
carries the table of where each part of it went, and
[`docs/inherited-claims.md`](docs/inherited-claims.md) is what carries its
measurements now.
