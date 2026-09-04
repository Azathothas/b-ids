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
**Category** docs, **Priority** P2, **Effort** S, **Status** done

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

### Closing

**Closed 2026-09-02T04:45:00Z.**
[`../docs/architecture.md`](../docs/architecture.md) exists, the conflict rule
names it, and the interim wording is in
[`../docs/HISTORY/README.md`](../docs/HISTORY/README.md) with its date.

```text
$ sh scripts/common/check-docs.sh
docs ok: 53 files, 880 relative links, 86 cited paths, 155 shell blocks. Links, paths and prose clean.
exit=0
```

#### What the reference carries

| section | what it settles |
| --- | --- |
| the product is a profile | the four halves, the conditions block, and that `digests` is empty on purpose |
| five components | what each crate does and, per row, ⛔ what it is not |
| the state a capture passes through | the pipeline as a diagram, with the four arrows a value can be lost at and what holds each |
| what is published, and what is derived | which files are append-only and which are rewritten from the tree |
| ⛔ the limits | stated rather than discovered by a reader |

#### ⛔ It was not written before the schema, and that was the entry's own rule

`SCHEMA-01` through `SCHEMA-09` are closed, so the page describes a model that
exists and every claim in it names a file the check resolves. ⭐ **A reference
describing a schema nobody had written is the claim this project exists to stop
making**, and `check-docs` asserts every cited path resolves, so the rule is held
by a gate rather than by memory.

#### The conflict rule was amended in place, not doubled

⛔ **Both rules live is the failure mode the entry named.** The interim wording
is gone from [`../docs/conventions/docs.md`](../docs/conventions/docs.md) and
kept verbatim in the history directory with the date it was withdrawn and what
replaced it.

⭐ **Its three exceptions survived the replacement, deliberately.** A value this
project did not measure, a term, and a reading of somebody else's code at a named
commit each have a document NEARER the thing measured than a reference page can
be, and the new rule names all three.

#### ⚠ Two stale claims this found on the way

⛔ **The router said the corpus holds one profile.** It holds three, and
`docs/AGENTS.md` said one because the sentence was written the day the first
landed. Corrected in the same change.

⛔ **The reference nearly said four crates when there are nine.** `b-ids` and
`b-ids-cli` are scaffolding whose own first lines say so, and a page that listed
five components and stopped would have implied the other four do something. They
are named as scaffolding, with the entries that fill them.

---

## DOC-02. The operator's side is unwritten because there is nothing to operate

**Source** found while adopting the template, 2026-08-30
**Category** docs, **Priority** P2, **Effort** S, **Status** done

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

### ⚠ 2026-09-04: the third trigger was eliminated rather than met, and this entry stays open

⛔ **The Approach names three triggers and two were already gone.** Keyless
attestation means no workflow needs a secret and no release needs a signing key,
so what remained was "a capture lane needs a machine somebody has to set up".

⭐ **`DRIVER-11` closed without needing one.** The Gecko lane arranges its trust
inside the throwaway profile the launcher already creates, with a certificate
database written by this tree's own Rust. ⛔ Nothing is installed, no trust store
is changed, and no person sets anything up. So that candidate is **eliminated**,
not triggered, and this entry is further from its trigger than it was this
morning rather than nearer.

⚠ **One candidate is left and it is not this entry's to force.** `PUB-06`
vendors a raw-socket route, and capturing at that layer can need a privilege or
a capability on the capture machine. ⛔ If it does, that is the trigger and
`HUMAN.md` gets written then, with the machine checklist that earns it.

⛔ **Still open, and still not written.** The Approach forbids writing it now and
filling it with placeholders, and a skeleton is exactly what an empty operator
runbook would be.

---


### ⭐ 2026-09-04, later the same day: the third trigger arrived, and it is written

⚠ **The note above says this entry was FURTHER from its trigger than it was that
morning**, with one candidate left: `PUB-06`'s raw-socket route, and only if
capturing at that layer needs a privilege or a capability on the machine.

⛔ **It does, and `PUB-06` measured it rather than predicting it.**

| host | what is there | what a packet-capture dependency would need |
| --- | --- | --- |
| the operator's Windows machine | Npcap IS installed: `wpcap.dll`, `Packet.dll` and `System32\Npcap\wpcap.dll` | nothing |
| `windows-latest` | nothing | Npcap, whose installer does not place silently in every configuration |
| `ubuntu-24.04` | unmeasured from here | `libpcap-dev`, which `apt-get` installs without a person |

⭐ **So taking the dependency makes the Windows half of the gate fail at link
time until somebody installs Npcap on that runner.** That is a decision about a
machine rather than about code, which is this entry's trigger in its own words.

#### The acceptance, run

```bash
sh scripts/common/check-placeholders.sh
```

```text
no placeholders survived in 325 files (TODO/ENTRY.md is exempt)
```

⚠ **Exit 0, read from the process, unpiped**, over a tree that now contains
[`../docs/HUMAN.md`](../docs/HUMAN.md). ⛔ The Approach forbade writing it with
placeholders in it, and the acceptance is exactly the check that would catch one.

#### ⭐ What earned it, and what it deliberately leads with

⛔ **Section 1 is what this project does NOT need from you**, and it is first on
purpose: a runbook that only listed obligations would read as though the blanks
were unwritten rather than as though the answer is none. No credential, no
signing key, no deployment, no host. `check-signing` asserts the first two.

⭐ **Section 2 is the machine checklist the Approach called the part that earns
it**: every tool, its minimum, and why the pipeline needs it, with the two
commands that read the machine rather than a table somebody works down by hand.
⚠ The optional tools are listed separately with what each one turns from a SKIP
into a check, because "install this and a leg starts running" is a different
fact from "install this or nothing works".

⭐ **Section 5 is the runbook**, and every row in it is an incident this
repository actually had rather than a failure somebody imagined: a lane that
exits 2, a run that reported `requests:0` over six real captures, a launch that
aborts in a snap sandbox, a capture that leaves the digest suite red, and a
Windows toolchain failure that is not the runner's.

⚠ **What it does NOT contain**, so nobody looks for it: no credential
inventory, because there are none; no deployment procedure, because nothing is
deployed; and no on-call rotation, because nobody is on call.

---

## DOC-03. There is no threat model, and publishing a corpus will need one

**Source** found while adopting the template, 2026-08-30
**Category** docs, **Priority** P2, **Effort** S, **Status** done

### Problem

A public repository that publishes an artefact should say where to report a
vulnerability and what a reporter should expect. Without it, a finder's options
are a public issue or silence, and both are bad.

### Premise

Measured: this repository publishes nothing yet, so there is no artefact to
report against. ⚠ The argument gets stronger the moment `PUB-01` cuts a release,
and stronger again if `HARNESS-12` ever hosts an endpoint.

⛔ **That premise is disproved as of 2026-09-04, and the correction is here
rather than as an edit to it.** The `data` branch has carried a tree a consumer
fetches since 2026-09-03, and `PUB-04` added a `configs/` tree to it: snippets
somebody pastes into their own tool, which is a materially different artefact
from a data file. ⚠ There IS something to report against now, and no release has
been cut, so the Approach's trigger and the Problem's condition have come apart.

⚠ **And the entry is still blocked, on a different thing.** The contact route is
the operator's to choose and every option is a decision plus, for two of them, a
setting on the remote: private vulnerability reporting on the forge, an address,
or a `security.txt`. ⛔ A session must not pick one, and must not publish an
address nobody offered. The question is in [`PROGRESS.md`](PROGRESS.md) with a
recommendation.

⭐ **What does not need the ruling is the threat model itself**, which the
Approach already says is the part that earns the document: a capture harness
accepts connections from anything, and a hosted oracle would receive other
people's traffic. Those two sentences are true today and are what a reporter
needs to know they have found something.

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


### ⛔ Ruled by the operator 2026-09-04: private vulnerability reporting on the forge

⭐ **One setting on the remote and no address in the tree.** `SECURITY.md`
points a reporter at the forge's private reporting route, so nothing published
here is an address that outlives the session that wrote it.

⛔ **Promise no timeline.** The Approach's own rule, and the ruling does not
change it.

⭐ **The threat model is the part that earns the document** and it needed no
ruling: a capture harness accepts connections from anything, and a hosted oracle
would receive other people's traffic. ⚠ The second becomes real if `HARNESS-12`
lands, which the operator has now put in scope.

### ⭐ Closed 2026-09-04. [`../SECURITY.md`](../SECURITY.md) exists, and it degrades rather than lying

⛔ **The ruling's route is measurably switched off, and the document is written
so that this does not matter to a reporter.** Measured 2026-09-04, read from the
forge rather than assumed:

```text
$ gh api repos/Azathothas/b-ids/private-vulnerability-reporting
{"enabled":false}
```

⚠ **A session must not switch it on.** It is a setting on the remote and this
session's authorisation names branches, workflow dispatches and merges, not
repository settings. ⭐ Enabling it is one action for the operator and it is
recorded in [`PROGRESS.md`](PROGRESS.md).

⭐ **So the document names the private route AND keeps a fallback that works
whether or not it is on:** open an issue saying only that you have a report and
asking for a private channel, with no detail in it. ⛔ That publishes nothing,
needs no address in the tree, and is true today. A document that told a reporter
to use a button that is not there would be the "most confident sentence in a
file is the only false one" case, in the file where it matters most.

#### What earned the document, which is the half that needed no ruling

| | |
| --- | --- |
| the harness accepts connections from anything | it is the design rather than a mode: the subject is a browser and a browser will not authenticate itself. Loopback by default, `--bind` refuses a hostname and the unspecified address by name, and the parsers are fuzzed with a panic treated as unacceptable |
| a hosted oracle would receive other people's traffic | `HARNESS-12` is open and nothing like it runs today. What it would receive is a fingerprint, which is what this project publishes, and one field a browser sends is a credential |
| what holds the second | the schema refuses a profile whose recorded bytes spell out a cookie or an authorization header, and header values are names-only by default. ⚠ Stated as a backstop rather than a licence |

⭐ **And the scope table names what is NOT a report here**, including the one
this project expects most: a browser's own defect belongs to its vendor.

⚠ **The row worth reading twice is the third**, and writing it was the audit:
a published artefact is permanent, because the corpus is append-only and the
data branch is never rewritten, so something that should not have been published
cannot be taken back by deleting it.

#### Prove

```bash
sh scripts/common/check-docs.sh
```

```text
docs ok: 56 files, 1171 relative links, 116 cited paths, 187 shell blocks. Links, paths and prose clean.
```

⚠ **Exit 0 read from the process, unpiped.** The file count moved from 55 to 56,
which is the new document, and its every link and cited path resolves. ⭐ The
block above is the run taken AFTER this section was written: closing an entry
adds links and a shell block of its own, so the first run's counts were already
stale by the time they were pasted.

⛔ **What this does not prove is that a claim in it is true.** That is a reading,
and the two the reading turned up are above: the reporting route is off, and the
hosted oracle does not exist.

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
