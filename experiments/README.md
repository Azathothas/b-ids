# experiments

The measurements this project took itself, each by a script anybody can re-run.

⭐ **A measurement that lives only in a session transcript is re-derived every
time somebody wants it.** The number gets quoted, the conditions do not, and
nobody can tell whether the difference between two runs is the change or the
machine. [`../docs/methodology/experiments.md`](../docs/methodology/experiments.md)
is the specification: what a script owes, what a number owes, and where the
result goes.

⛔ **Numbered in the order they were run, and a number is never reused.** A
citation of `10-` has to keep meaning what it meant, so a replaced experiment
keeps its file and the replacement takes the next number.

⛔ **Nothing here cleans up its own output.** The evidence is the point. Every
script writes under the repository's ignored scratch directory, and says on the
way out where it left things.

---

## The scripts

| | question it answers |
| --- | --- |
| [`10-first-profile.sh`](10-first-profile.sh) | what does the browser on this machine put on the wire, and does the corpus hold it? |
| [`20-compare-capture-modes.sh`](20-compare-capture-modes.sh) | does completing the handshake change what the browser offers before it? |
| [`30-resumption-control.sh`](30-resumption-control.sh) | does refusing session tickets change the cold hello, or only which connections are cold? |
| [`40-trust-paths.sh`](40-trust-paths.sh) | which trust route lets a browser complete a handshake with this harness, on this platform? |
| [`50-trust-anchor.sh`](50-trust-anchor.sh) | does trusting a real root change what the browser puts on the wire, against trusting one key for one launch? |
| [`60-identify-extension.sh`](60-identify-extension.sh) | can the unidentified TLS extension codepoint be named, and if not, what has been searched? |

---

## ⚠ What every one of these is a measurement OF

⛔ **One machine on one day is one machine on one day.** Each script prints its
conditions on the way out: the host, the tool versions, the browser build, the
sample count and the trust configuration the capture was taken under. A number
without those cannot be compared to anything, which makes it worse than an
absence.

⚠ **And the trust configuration is not a detail.** Every capture this project
has taken went through a per-launch key pin rather than a trust store, which is
a condition of the measurement. ⭐ `20-` measured the capture SURFACE and found
it changes nothing a raw capture can see; what a real trust anchor would do is
still unmeasured, and answering it needs a root installed into the host's own
trust store.
