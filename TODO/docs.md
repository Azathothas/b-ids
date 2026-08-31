# docs

Documents this project will need and does not have, and the claims in the ones
it does have that are waiting on a measurement.

⛔ **An empty skeleton outlives the session that wrote it**, so each of these
stays unwritten until there is something true to put in it. The entry is what
records that the decision was made rather than forgotten.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## DOC-01. There is no technical reference, and one document is pretending not to notice

**Source** found while adopting the conventions, 2026-08-30
**Category** docs, **Priority** P2, **Effort** S, **Status** open

### Problem

The convention that governs documents says a conflict between two of them is
settled by the technical reference. This tree has no technical reference,
because it has no schema, so that rule currently points at nothing.

### Premise

Measured by reading, this session: the section was rewritten to say that a
conflict is settled by whichever document is nearest the measurement, and to say
in as many words that no single reference exists yet.

### Approach

When `SCHEMA-01` lands, write docs/architecture.md: the schema, the pipeline
between the five components, the state a capture passes through, and the limits.
Then rewrite the conflict rule to name it, and move the interim wording to
[`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) rather than deleting
it.

⛔ Do not write it before the schema. A reference that describes a schema nobody
has written is the claim this project exists to stop making.

Must not: leave both rules live. Amend in place, and move the superseded wording
to the history directory.

### Prove

```bash
sh scripts/common/check-docs.sh
```

Passing means: docs/architecture.md exists, every claim in it names a file in
the tree, the conflict rule names it, and the interim wording is in the history
directory with its date.

---

## DOC-02. The operator's side is unwritten because there is nothing to operate

**Source** found while adopting the template, 2026-08-30
**Category** docs, **Priority** P2, **Effort** S, **Status** open

### Problem

There is no document saying what only a human can do: which machine setup is
needed, which credentials exist and where they live, and which runbook to follow
when a capture fails at three in the morning.

### Premise

Measured: nothing in this tree needs a credential, nothing is deployed, and
nothing runs on a schedule. So the document would be empty, and an empty
skeleton is read by a later session as a document somebody forgot to finish.

### Approach

Write `HUMAN.md` when the first of these becomes true: a workflow needs a secret,
a release needs a signing key, or a capture lane needs a machine somebody has to
set up.

⭐ **The machine checklist is the part that earns it**: every tool the pipeline
needs, its minimum version, and the one-line command that checks it. That exists
so a unit of work does not stall three tasks in on a missing program.

Must not: write it now and fill it with placeholders.

### Prove

```bash
sh scripts/common/check-placeholders.sh
```

Passing means: when the file is written it contains no unfilled marker, and
until then the check passes over a tree that does not have it.

---

## DOC-03. There is no threat model, and publishing a corpus will need one

**Source** found while adopting the template, 2026-08-30
**Category** docs, **Priority** P2, **Effort** S, **Status** open

### Problem

A public repository that publishes an artefact should say where to report a
vulnerability and what a reporter should expect. Without it, a finder's options
are a public issue or silence, and both are bad.

### Premise

Measured: this repository publishes nothing yet, so there is no artefact to
report against. ⚠ The argument gets stronger the moment `PUB-01` cuts a release,
and stronger again if `HARNESS-12` ever hosts an endpoint.

### Approach

Write `SECURITY.md` when the first release ships: where to send a report, what
to include, and what happens next. Keep it short and do not promise a timeline
that will not be met.

⭐ **Writing the threat model is the audit**, and this project has two specific
things to think about that a normal project does not: a capture harness accepts
connections from anything, and a hosted oracle would receive other people's
traffic.

Must not: promise a response time nobody has agreed to.

### Prove

```bash
sh scripts/common/check-docs.sh
```

Passing means: the file exists when a release does, it names a contact route,
and every claim in it is one somebody has agreed to.

---

## DOC-04. The founding brief is retired, and this entry records what replaced it

**Source** the operator
**Category** docs, **Priority** P2, **Effort** S, **Status** done

### Problem

This repository began as a single design brief describing a project that did not
exist. A brief is not a work list, nothing in the tree could be checked against
it, and leaving it beside the tree would give a later session two accounts of
what the project is.

### Premise

⛔ **The first attempt at this entry closed on a reading that had not been
done.** It routed the brief's content into this tree without fetching the
repository the brief's measurements were taken in, so about sixty values arrived
with a provenance tag that resolved to nothing, five claims went unchecked, and
one of them was wrong.
[`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md)
carries the corrections.

Measured, on 2026-08-31: the origin repository is fetched at a named commit into
[`../references/Azathothas__bit-cli/`](../references/Azathothas__bit-cli/),
every measured claim is cited against a file in it, and the brief's own sections
were read against those files rather than against the brief.

### Approach

Where each part of it went:

| the brief's content | destination |
| --- | --- |
| the governing rule, and the three consequences | [`../README.md`](../README.md), and absolute 1 in [`../docs/AGENTS.md`](../docs/AGENTS.md) |
| the prior-art table | [`../docs/reference-sweeps/findings.md`](../docs/reference-sweeps/findings.md), re-derived from the trees rather than inherited |
| ⭐ the repository every measurement was taken in | [`../references/Azathothas__bit-cli/`](../references/Azathothas__bit-cli/), tracked, at the commit its `PROVENANCE.md` names |
| the data model, sections 2.1 to 2.6 | `SCHEMA-01` through `SCHEMA-07` |
| the architecture and the capture harness | `HARNESS-01` through `HARNESS-04`, `DRIVER-01`, and [`../docs/reference-sweeps/usable.md`](../docs/reference-sweeps/usable.md) sections 14 and 15 |
| every measured fingerprint, codepoint and constant | [`../docs/inherited-claims.md`](../docs/inherited-claims.md), each cited at a file in the origin tree |
| the validator's eight checks | `VALID-01` |
| the emitter holes | `EMIT-01`, corrected where the sweep found them stale |
| automation, staleness and the capture matrix | `CI-01` through `CI-04`, and `CORPUS-02` |
| durability and redundancy | `CI-05` through `CI-08`, and `DRIVER-05` |
| publishing, formats and the licence | `PUB-01` through `PUB-09` |
| the traps | [`../docs/inherited-claims.md`](../docs/inherited-claims.md) section 8, and the entries that own each one |
| the layout, toolchain, definition of done and test strategy | [`RULES.md`](RULES.md), `TOOL-01`, and the methodology this tree already carries |
| the scope boundary | [`../README.md`](../README.md) |
| the glossary | [`../docs/glossary.md`](../docs/glossary.md), with the entries a reading corrected |
| its own provenance section, naming what was unverified | [`../docs/inherited-claims.md`](../docs/inherited-claims.md), which is that section generalised into a document with a rule attached |

⭐ **Four of its claims were refuted during the reading**, and all four are in
[`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) with the reading that
took each away.

Must not: reintroduce it, or cite it as a path. ⛔ **"The founding brief" is
provenance and not a file**, defined once in
[`../docs/inherited-claims.md`](../docs/inherited-claims.md), and nothing in
this tree needs it to be readable.

### Prove

```bash
sh scripts/common/check-docs.sh
```

Passing means: every link in the tree resolves and every cited path exists, so
no document depends on a file that is not here. The brief is one of them.

### Closing

**Closed 2026-08-31T00:00:00Z.** The brief was read against the origin tree
section by section, routed as the table above records, and removed from the
working tree in the same change that created its destinations. It was never
committed.

```text
$ sh scripts/common/check-docs.sh
docs ok: 48 files, 662 relative links, 129 shell blocks. Links and prose clean.
```

⚠ **The three counts move with the tree** and the exit code does not. A later
run reporting different figures over zero problems is this entry still passing,
not a defect.

⚠ **What this entry does not claim.** The routing was checked by reading, not by
a mechanical diff, because prose has no such check. What is mechanical is
narrower and it is what the acceptance runs: no link and no cited path in this
tree resolves to the retired file, because none names it.
