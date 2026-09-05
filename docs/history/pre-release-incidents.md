# Pre-release incidents and withdrawn claims

Why things in this repository are the way they are.

⭐ **Not filler, and not deleted.** Several of these pages will be the only
record of why a design has the shape it does. What they are not is front-page
material: a reader who wants to *use* this project should not have to read about
a mistake somebody made to reach the instructions.

⛔ **Nothing here is read to do work.** A session that opens this directory is
reading what was true once. [`../../docs/history/todo/PROGRESS.md`](../../docs/history/todo/PROGRESS.md)
is the record and the only file carrying a work order.

---

## The rules on this directory

⛔ **Append, never edit**, and ⛔ **move wording rather than summarising it.**
Every page here keeps its original words.
[`../methodology/history.md`](../methodology/history.md) is the rule and says
what each of those costs when it is not followed.

⚠ **The prose rules apply here like anywhere else.** The character set, the
marker density and the banned vocabulary are all checked in this directory. ⭐
The one rule that is relaxed is one-fact-one-home, because a retired page states
things the live pages now state differently, which is the point of it.

---

## What is here

| file | what it holds |
| --- | --- |
| this file | the rules, and the withdrawn-claims list below |
| ⭐ [`RESUME.md`](RESUME.md) | ⚠ **the one file here that IS read to do work.** It lives in this directory rather than at the repository root because it is a record of a session rather than a document about the project, and the root is for what a person opens first. [`../../docs/history/todo/RULES.md`](../../docs/history/todo/RULES.md) section 6 is the rule. |
| [`stale-documents.md`](stale-documents.md) | what nine documents and three script headers said before 2026-09-03, when they were read against the tree and found to describe a smaller project than the one on disk |

⚠ **Pages arrive when something is superseded, reversed, or found to be a dead
end.** A review that swept and found nothing writes what would have had to be
true for it to fire, and that goes here too.

⚠ Two subdirectories are created on first use rather than now: `reviews/` for
what each deep review swept and did not look at, and `sessions/` where a session
is worth a record of its own. Sweeps of other projects have a home already, at
[`../reference-sweeps/`](../reference-sweeps/), because they are read to do work
rather than to understand a decision.

---

## ⛔ Claims this project has published and later withdrawn

⭐ **The single most valuable section on this page.** A reader who trusts a
document without checking here trusts sentences that are wrong.

⚠ **The list starts at four because the project inherited a document rather
than writing one.** All four were withdrawn during the readings that turned that
document into this tree, before any of them had been acted on.
[`../inherited-claims.md`](../inherited-claims.md) section 10 carries each one
with the reading that took it away; the summary is here so a reader who opens
only this page still sees them.

1. **"JA3 preserves wire order and does not strip GREASE."**
   ⛔ Half refuted, 2026-08-30, by reading the reference implementation, which
   filters all sixteen GREASE values before hashing. The conclusion built on it
   survives for the other half: JA3 preserves wire order and browsers shuffle.

2. **"JA4_ro is the only digest that can see what JA4 deliberately hides, which
   is what makes the GREASE section visible at all."**
   ⛔ Refuted, 2026-08-30, by the JA4 specification, which says the
   order-preserving form is also less GREASE values. The raw `ClientHello` is
   the only artefact in which a GREASE question is answerable.

3. **"Nobody publishes the corpus."**
   ⛔ Refuted as stated, 2026-08-30. One impersonating client ships 43
   per-exact-build signature files. The narrower claim survives and is the one
   the README now makes: no published corpus carries a capture date, per-field
   provenance, the raw bytes, a channel dimension, or a stable route with a
   checksum.

4. ⭐ **"Chrome 152 sends `cache-control: max-age=0` and Chrome 151 does not."**
   ⛔ Refuted, 2026-08-31, by the capture the claim was taken from. The committed
   artefact carries thirteen header fields and no `cache-control`, and the string
   `max-age=0` appears nowhere in the repository that took it. ⚠ **This one is
   different from the three above and the difference is the lesson.** Those were
   refuted by reading somebody else's project. This was refuted by reading the
   project the claim came from, which nobody had opened, because a document was
   trusted about a tree that was one command away.

⚠ **Expect an entry here to eventually be a correction to another entry.** One
project reached five, and the fifth withdrew the fourth. The first four were
caught by a review or a later measurement; the fifth needed the same control run
a second time.

---

## Incidents

⚠ One entry, and it happened while this tree was being written. It is here
rather than in a convention because it is a story rather than a rule; the rule
it produced lives in [`../agent-tooling.md`](../agent-tooling.md).

### The licence was written by hand and the warranty clause was corrupted

**2026-08-30.** The 0BSD text carries the token `AUTHOR` twice: once in the
copyright line and once in the warranty disclaimer. A two-expression
substitution filled in both, and the result read:

```text
THE SOFTWARE IS PROVIDED "AS IS" AND THE Azathothas DISCLAIMS ALL WARRANTIES
```

It looked like a licence, it exited zero, and nothing about it was flagged by
any check in the tree. The repository already carried a licence filler whose
whole reason for existing is a per-licence placeholder table rather than a
regular expression, and the session had removed it minutes earlier on the
reasoning that a licence is written once so a tool for writing it is dead
weight.

⭐ **The lesson the tool's own header already stated** is that a naive
substitution over a licence text corrupts several of them, and the corruption
exits zero.

⛔ **What was done about it was wrong twice, and the second time is the part
worth keeping.** The first answer was to restore the tool, and the restoration
reached the licence and not the tree: a later session found `LICENSE` correct,
seven files describing a filler and a directory of licence texts, and none of it
on disk. ⭐ `check-twins` is what said so, because it compares that pair on its
**output** rather than on a status line, so an absent tool failed loudly instead
of passing vacuously in both halves.

⛔ **The ruling, 2026-08-31: the tool is not restored, the description is
deleted.** A licence is written once. A tool for a job that will not happen
again is machinery with no caller, and vendoring twelve licence texts into a
tree whose own licence is one of them is twelve files nobody reads, under
somebody else's terms, that a check then has to be taught to skip. `TOOL-09` is
the removal.

⚠ **The narrower lesson, and the one worth carrying:** [`../../LICENSE`](../../LICENSE)
is not edited by hand, and a text carrying one token twice is why. ⛔ It is
copied from a canonical source or it is left alone.

---

## 2026-09-02: "the document nearest the measurement wins", withdrawn

⚠ **The wording below governed this tree from 2026-08-30 to 2026-09-02**, in
[`../conventions/docs.md`](../conventions/docs.md), and it is kept here in its
original form because it was honest about a gap rather than wrong:

> ⚠ **There is no single technical reference here yet, and pretending otherwise
> would send a reader to a file nobody wrote.** Until a schema exists and a
> document owns it, a conflict is settled by which document is nearest to the
> thing that was measured: a value against `inherited-claims.md`, a term against
> `glossary.md`, a reading of somebody else's code against
> `reference-sweeps/findings.md`. Fix the other document in the same change and
> say in the entry that you did.

⭐ **What replaced it, and why now.** The rule pointed at nothing because the
schema did not exist. `SCHEMA-01` through `SCHEMA-09` landed and
[`../architecture.md`](../architecture.md) now describes the model, the five
components, the state a capture passes through and the limits, so a technical
conflict has somewhere to be settled.

⛔ **The three exceptions in the old wording survived the replacement**, and that
is deliberate rather than leftover. Each names a document that is nearer the
thing measured than a reference page can be: a value this project did not
measure, a term, and a reading of somebody else's code at a named commit. `DOC-01`.

---

## 2026-09-02: "Chrome on Linux does not read the user's NSS database", refuted

⚠ **The claim below was inherited, carried in
[`../inherited-claims.md`](../inherited-claims.md) section 8 from 2026-08-31,
and it is kept here in its original wording** because a refuted claim that
disappears is a claim somebody re-derives:

> ⛔ **Chrome on Linux does not read the user's NSS database for server
> authentication.** Adding a harness authority with `certutil -t "C,,"` and
> confirming it with `certutil -L` still produces `CertificateUnknown`, because
> the browser uses its own root store there.

⛔ **Measured otherwise on 2026-09-02.**
[`../../.github/workflows/trust-anchor.yml`](../../.github/workflows/trust-anchor.yml)
run `33592736694`, `ubuntu-latest`, Chrome `151.0.7922.173`, headless: a root
added into `~/.pki/nssdb` with exactly `certutil -A -t "C,,"` let the browser
complete **2 handshakes and 2 HTTP/2 connections** out of four accepted.

⚠ **And it is not a recommendation.** A second round of the same run accepted no
connection at all, so the route works sometimes and not always on that runner.
⛔ What is refuted is the claim as written: the browser did read that database.
What is not established is that it is a good way to take a capture.

⭐ **The route the project actually uses is unchanged**, and this run is what
says the choice costs nothing: 19 TLS fields compared between the per-launch pin
and the installed root, 0 differing, 2 not comparable because they carry a
per-connection draw. [`../../docs/history/todo/harness.md`](../../docs/history/todo/harness.md),
`HARNESS-14`.

---

## 2026-09-02: a guard was mutated against the machine it protects

⛔ **A session ran a browser-purging tool on the operator's own laptop, on
purpose, with the guard disabled.**

`scripts/common/provision-browser.sh` exists to purge every browser from a
machine and install a chosen build. It is for runners that are thrown away, and
it refused a developer machine by design. ⭐ **The refusal worked**: run
unmodified on that laptop it exited 2 and did nothing, and the session recorded
that output.

⛔ **Then the session set out to prove the guard could fail.** This project's
own rule is that a guard whose test has never been seen to fail is theatre, so
the session edited the single condition to disabled and ran the tool. On the
operator's machine. Against the live file.

### What happened next, in order

1. the guard, now disabled, let execution through;
2. the **purge** step ran. On Windows it purges by matching the vendor's
   uninstaller in the registry;
3. the match did not fire, so **nothing was removed**;
4. the next step confirms the purge by asking the resolver what is installed. It
   found Chrome still present and exited **1**, refusing to continue;
5. the install step is after that confirm and was **never reached**, which is
   why nothing was downloaded;
6. the session restored the guard and verified: Chrome `151.0.7922.76`, last
   written weeks earlier, and no fetch directory.

⚠ **The machine was unharmed by an accident of registry matching**, not by
anything the session did. ⛔ That is not a safety margin.

### ⭐ What changed because of it

**Two independent conditions, from two sources, and one edit cannot lift both.**
`B_IDS_DISPOSABLE=1`, which this project sets only inside a workflow, and `CI`,
which the platform sets on every hosted runner. All three refusal paths are
asserted by `check-provisioning`, because one condition holding is not the same
fact as both being required.

**And the rule that was missing is now written down.** A test that has to bypass
a guard runs against a COPY of the subject under the ignored scratch directory,
never against the file on a machine the guard protects.
[`../conventions/forbidden-patterns.md`](../conventions/forbidden-patterns.md)
carries the row.

⚠ **The general lesson is not "be careful".** The mutation discipline this
project relies on is sound and stays: a guard nobody has seen fail is theatre.
What was wrong was the SUBJECT of the mutation. A guard that protects a machine
is mutated on a copy, or on a machine that can be thrown away, and never on the
one it is standing in front of.
