# docs.md

The document set, what each one owns, and the rules that keep it trustworthy.

[`prose.md`](prose.md) is how they are written. This is which ones exist and
what makes them true.

---

## The set this repository actually has

⛔ **Every row names a file that exists.** A row for a document nobody wrote is
worse than a missing rule: there is nothing to open, and a session that was sent
there follows whatever it expected to find. The set grows when a document earns
its place, and a role listed here with no file behind it is a defect.

| file | owns |
| --- | --- |
| ⭐ [`../AGENTS.md`](../AGENTS.md) | the router, and the ONLY one. Where you are, the absolutes, the start of a session, and what to read for which task. |
| [`../../README.md`](../../README.md) | what this is, for a competent stranger: the thing, the honest limits, the licence, and the map |
| ⭐ [`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md) | the record. The baseline, what the last session did, and the work order. Nothing else carries a work order. |
| [`../../TODO/RULES.md`](../../TODO/RULES.md) | the half of the record that does not change between sessions: the standing facts, and the rules that are this project's own |
| [`../../TODO/INDEX.md`](../../TODO/INDEX.md) | every entry, one line each, with the counts a check holds |
| [`../../TODO/ENTRY.md`](../../TODO/ENTRY.md) | the form an entry is written from |
| [`../../TODO/SUMMARY.md`](../../TODO/SUMMARY.md) | the last session's table. ⚠ A snapshot, never an authority. |
| [`../../CHANGELOG.md`](../../CHANGELOG.md) | what shipped, when, and where the evidence is |
| [`../HISTORY/RESUME.md`](../HISTORY/RESUME.md) | ⚠ written at the START of a session and refreshed as work moves, so a session that dies mid-task still hands something over. Overwritten every session, never appended to. |
| ⭐ [`../glossary.md`](../glossary.md) | the terms this project uses without stopping to define them |
| ⭐ [`../inherited-claims.md`](../inherited-claims.md) | every value this project carries that was measured somewhere else. **When a document states a fingerprint, a version or a codepoint, this file is where its provenance lives, and when the two disagree this one wins.** |
| [`../agent-tooling.md`](../agent-tooling.md) | what tool does what job, and where each one lives |
| [`../containers.md`](../containers.md) | measuring in a machine you throw away afterwards |
| [`../reference-sweeps/findings.md`](../reference-sweeps/findings.md) | what external projects were read, at which commit, and what was true in them |
| [`../reference-sweeps/usable.md`](../reference-sweeps/usable.md) | which of those findings this project can actually use, and where each one lands |
| ⭐ [`../HISTORY/README.md`](../HISTORY/README.md) | **the story.** Superseded wording, reversed decisions, dead ends, review passes. ⛔ Everything above says what is true now; this says what was believed and why that changed. |

⚠ **Three roles this set deliberately leaves empty**, because a skeleton with
nothing in it outlives the session that wrote it. There is no technical
reference for a schema nobody has written, no operator runbook for a pipeline
that does not run, and no threat model for a corpus that does not exist.
[`../../TODO/PROGRESS.md`](../../TODO/PROGRESS.md) carries each as an open
question rather than an empty file.

⛔ **`docs/HISTORY/` exists so that none of the rows above fill up with
narrative.** ⚠ The instinct to record the story is right, which is why
forbidding it does not work: a superseded explanation is worth keeping for the
reason [`prose.md`](prose.md) gives. It needed a destination.
[`../methodology/history.md`](../methodology/history.md) is that rule.

---

## The invariants

### One fact, one home

Every fact lives in exactly one document. A version string, a constant, a rate
limit, a schema: one place.

⛔ **A value in two documents with no check between them drifts**, and the copy
a reader trusts is the wrong one. If a number must appear twice, derive it from
the source, or have a check assert the two agree.

⚠ The trap is that a value which never changes cannot expose a missing check.
It sits correct for a year and drifts the first time it moves.

### The document nearest the measurement wins

⚠ **There is no single technical reference here yet, and pretending otherwise
would send a reader to a file nobody wrote.** Until a schema exists and a
document owns it, a conflict is settled by which document is nearest to the
thing that was measured: a value against
[`../inherited-claims.md`](../inherited-claims.md), a term against
[`../glossary.md`](../glossary.md), a reading of somebody else's code against
[`../reference-sweeps/findings.md`](../reference-sweeps/findings.md). Fix the
other document in the same change and say in the entry that you did.

### Documentation ships with the code it describes

⛔ Doc and code drifting apart is a forbidden pattern. The moment code changes a
documented behaviour, the document changes with it. In the same commit, not
later.

### Every claim is verified before it is written

Writing the documentation is the audit. Being forced to say precisely what
something does, and then checking whether that is true, is where a surprising
share of real defects are found.

⚠ The most confident sentence in a file is regularly the only false one. A test
file header asserting it ran "exactly as production uses it" hid the gap that
shipped a server error for six units of work.

### Prefer a shape a check can assert

Where a document names a file, a constant, a route or an identifier, prefer a
form a check can verify against the tree, so a rename fails a gate instead of
rotting quietly.

⭐ The strongest version of this is a catalogue where each entry declares which
files read it, and a check opens those files and looks. That is a document that
reviews itself.

⚠ **A document that cannot be checked is a document that drifts.** That is not
an argument against writing prose. It is an argument for making the mechanical
parts mechanical, so the reading is spent on the parts that need it.

### Say what is not true

Reserve a place for the truths that are tempting to hide. This is slower than
it looks. This has a known gap. This estimate excludes something unmeasurable.

⛔ A limit hidden is a defect filed against the user later.

---

## Where a lesson goes

⛔ **There is no separate lessons file, and that is a decision rather than an
omission.** A running log of what worked and what bit is a fourth place for a
fact to live beside the entry that found it, the convention that generalises it
and the check that holds it, and the copy a reader trusts would be whichever
they opened first.

| the lesson is | where it goes |
| --- | --- |
| mechanical, so a script can hold it | ⭐ a check under [`../../scripts/common/`](../../scripts/common/). A rule enforced by a script is a rule nobody has to remember. |
| greppable, so a reviewer can recognise it | a row in [`forbidden-patterns.md`](forbidden-patterns.md), with what it caused |
| specific to this project, and it cost something | [`../../TODO/RULES.md`](../../TODO/RULES.md) section 4 |
| a value somebody else measured | ⭐ [`../inherited-claims.md`](../inherited-claims.md), with its source and the entry that re-measures it |
| a thing that was believed and is not any more | [`../HISTORY/README.md`](../HISTORY/README.md) |
| what one reference project taught | [`../reference-sweeps/usable.md`](../reference-sweeps/usable.md) |

⚠ **A lesson that fits none of those rows is usually not a lesson yet.** It is
an observation, and it belongs in the entry that produced it until a second
occurrence says what it generalises to.

---

## The changelog

**What shipped, when, and where the evidence is.** One entry per shipped unit
of work, pointing at the record that carries the detail.

⭐ It is also the destination for what a documentation pass removes. When a
document loses the *story* of a fix, what broke and what the sentence used to
say, the story comes here. So this file is expected to grow, and its length is
not a defect.

| the text is | where it goes |
| --- | --- |
| a fact, limit or constraint a future session needs | ⛔ the document. Not here. |
| a measurement with its conditions | ⛔ the document, as a table. Not here. |
| the story of a fix, or a superseded claim kept for provenance | ⭐ here |
| the full detail of one session's work | ⛔ the entry that closed. Here goes a pointer to it. |

Four rules, and [`scripts/common/check-changelog.sh`](../../scripts/common/check-changelog.sh)
holds all four. ⭐ They were stated here and enforced by nothing for as long as
this document existed, which is the shape a rule takes on its way to becoming
a preference:

1. ⛔ **Newest first, always.** A new entry goes at the top of its section,
   never appended to the bottom.
2. ⛔ **Every heading carries a date.** Consider a full ISO 8601 UTC stamp:
   several entries sharing one date cannot be ordered from what was written
   down.
3. ⛔ **Every entry names its record**, the work item or the commit carrying
   the evidence. An entry with no record is a claim.
4. ⛔ **Every entry says whether it deployed.** "No version bump and no deploy"
   is a complete and common answer. Silence is not.

And two things an entry must not do:

- ⛔ **Do not tidy the file while shipping something else.** Reordering old
  entries in the commit that adds a new one makes both unreviewable. Tidying is
  its own commit.
- ⛔ **Do not delete an entry.** A superseded one is amended in place with a
  dated note. Amend, never silently delete.

⚠ A check can hold the order, the dates and the pointers. **It cannot check
that an entry is true.** That stays with the claim audit,
[`../methodology/reviews.md`](../methodology/reviews.md) lens 3.
