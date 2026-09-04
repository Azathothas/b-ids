# SECURITY.md

How to report something in this repository, and what this project's own attack
surface actually is.

[`TODO/docs.md`](TODO/docs.md), `DOC-03`, is the entry that asked for this.

---

## Reporting

⭐ **Report privately, through this repository's own private vulnerability
reporting.** It is on the repository's Security tab, it carries no address that
outlives whoever wrote it, and nothing you send through it is public.

⭐ **It is switched on**, measured from the forge on 2026-09-04 rather than
assumed:

```bash
gh api repos/OWNER/REPO/private-vulnerability-reporting
```

```text
{"enabled":true}
```

⚠ **That was `{"enabled":false}` when this document was written**, which is why
the paragraph below exists and why it stays.

⚠ **If that route is not offered to you, open an issue that says only that you
have a report and asks for a private channel.** ⛔ Do not put the detail in it.
A one-line issue with no detail is safe to write in public and is the fallback
this document deliberately keeps, because a route that is switched on at the
repository is a route that can be switched off.

⛔ **No timeline is promised here.** Nobody has agreed to one, and a document
that promises a response somebody has not agreed to is worse than a document
that says nothing.

**What a report is most useful with:** the commit, the command you ran, what
happened, and what you expected. A profile identifier or a route path where one
applies. ⚠ If a capture is involved, the raw ClientHello hex is the artefact
that survives every parser defect in this tree.

---

## ⭐ What this project is, in one paragraph, because it changes what counts

This repository is a corpus of **browser network fingerprints** and the harness
that captures them. It publishes data: what a named browser build put on the
wire, with the bytes it was read from. ⭐ **It serves detection exactly as much
as impersonation**, because "what does this build send" has one true answer
whichever side is asking. [`README.md`](README.md) says the same thing at
greater length.

⚠ **A wrong value here is a defect and it is the interesting kind.** A profile
that names one browser version over another version's handshake describes a
combination that exists nowhere, and a consumer that ships it is more
distinguishable than one that shipped nothing.
[`crates/b-ids-validator`](crates/b-ids-validator) is what refuses those, and a
coherence rule it does not have is a real gap.

---

## The threat model

⛔ **Two things here are unlike an ordinary repository, and both are in the
capture path.**

### 1. ⭐ The harness accepts connections from anything

[`crates/b-ids-harness`](crates/b-ids-harness) binds a listener and reads
whatever arrives. That is not a mode; it is the whole design, because the
subject being measured is a browser and a browser is not going to authenticate
itself.

| | |
| --- | --- |
| what it binds | loopback by default. `--bind` takes an address and ⛔ **refuses a hostname and refuses the unspecified address, by name**, so widening it is a deliberate act with an address typed into it. |
| what it parses | attacker-shaped input by construction: a `ClientHello`, HTTP/2 frames, and HPACK, all read off the socket before anything is trusted. |
| what holds it | the parsers are fuzzed, and ⛔ **a panic in one is treated as unacceptable rather than as a crash.** [`TODO/harness.md`](TODO/harness.md), `HARNESS-09`. |
| what it does not do | it runs no user code, executes nothing it reads, and writes only into paths a caller named. |

⚠ **A run bound to a public address is a service, and this project does not
operate one.** If you are pointing this harness at the internet you have taken
on that decision yourself.

### 2. ⚠ A hosted capture oracle would receive other people's traffic

[`TODO/harness.md`](TODO/harness.md), `HARNESS-12`, is the entry for a public
capture endpoint. ⛔ **The MODE is built and nothing is hosted**, and the
difference is the whole of this section: `b-ids-harness --serve` answers a
caller on a machine somebody chose to run it on, and no endpoint of this
project's is reachable from anywhere.

### ⭐ The retention answer, settled before the mode was built

⛔ **Nothing is retained. That is the default and it is enforced rather than
promised.**

| the question the entry asked | the answer |
| --- | --- |
| what is logged | the capture goes to the socket that produced it and to the run's own stdout. Nothing else |
| for how long | ⛔ the life of the process |
| whether anything is retained at all | ⛔ **no.** `--no-retain` refuses `--ca-out`, `--hello-out` and `--write-golden` by name, at parse time, before a socket is opened |
| how that is checked | a test runs the binary in an empty directory and counts what is in the directory afterwards. ⚠ Over the DIRECTORY rather than over a list of switches, because a list can go stale and a directory cannot |
| what a caller gets | its own capture, in full, with the raw bytes. ⭐ That is the Problem: every existing hosted service returns a subset and no raw hello |

⛔ **A capture from a visitor's browser never becomes a corpus entry.** The
corpus is captures the harness took of browsers it launched itself, and there is
no path from the serve mode into `b-ids-corpus add`.

⚠ **`--serve` without `--no-retain` is allowed and warns on stderr**, because
pairing an oracle with a switch that writes is a decision somebody may have
reasons for. What it must not be is silent.

⭐ **What it would receive is a fingerprint, which is exactly the thing this
project publishes.** That is the point and it is also the hazard: a header set
carries more than a handshake does, and one of the fields a browser sends is a
credential. ⛔ [`crates/b-ids-schema`](crates/b-ids-schema) refuses a profile
whose recorded bytes spell out a cookie or an authorization header, and header
values are recorded **names-only by default**, which is the setting that exists
so a model whose natural form carries values cannot one day publish one.

⚠ **That check is a backstop and not a licence.** It knows the shapes it knows.

---

## What this repository does not have

⛔ **No credential of any kind**, and that is a property the tooling holds
rather than a claim: [`scripts/common/check-no-secrets.sh`](scripts/common/check-no-secrets.sh)
runs in the gate over every tracked file, with a stricter set of rules for a
public repository. Every workflow that writes uses the run's own scoped token
and never a personal one.

⛔ **No network service, no deployment and no user data.** What is published is
a git branch of static files and, when a tag is pushed, a release of the same.

⚠ **The vendored and mined trees are somebody else's code**, under their own
licences, in [`vendor/`](vendor/) and [`references/`](references/). A defect in
one of them is theirs; ⛔ this project's rule is to fix it here rather than
wait, and [`patches/README.md`](patches/README.md) records every such change.
⭐ Report it here anyway if you found it through this repository, and say which
tree it is in.

---

## Scope, plainly

| in scope | out of scope |
| --- | --- |
| a parser in this tree that panics, hangs, or reads out of bounds on input from a socket | a browser's own defect. Report that to its vendor. |
| a published profile whose values do not describe the build it names | a consumer's misuse of correct data |
| a credential or a fingerprint of a private system reaching a published artefact | the fact that fingerprint data exists |
| a check that reports green over the thing it exists to catch | a missing feature. That is [`TODO/PROGRESS.md`](TODO/PROGRESS.md). |

⭐ **The third row is the one to read twice.** A published artefact is
permanent: the corpus is append-only and a data branch is never rewritten, so
something that should not have been published cannot be taken back by deleting
it. That makes it the report this project most wants to receive early.
