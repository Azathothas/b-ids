# HUMAN.md

⭐ **What only a person can do.** Everything else in this tree is a script, a
check or a workflow, and if a thing is here it is because no one of those can do
it. `DOC-02`.

⛔ **It was NOT written until it had something in it.** The entry's rule was to
wait for one of three triggers, and two of them were eliminated rather than met:
keyless attestation means no workflow needs a secret and no release needs a
signing key. The third arrived on 2026-09-04 when `PUB-06` measured what the TCP
half of a fingerprint costs, and section 3 is that measurement.

---

## 1. ⛔ What this project does NOT need from you

⚠ **Stated first, because the absence is the load-bearing part** and a runbook
that only listed obligations would read as though the blanks were unwritten.

| | |
| --- | --- |
| a credential, of any kind | **none.** No workflow names a `secrets.` value, no release needs a signing key, and the tree carries no key-shaped file. `scripts/common/check-signing.sh` asserts all three |
| a signing key | none. `PUB-09` is keyless: the runner's own OIDC identity signs |
| a deployment | none. Nothing is deployed. The two publishing surfaces are a git branch and a release |
| a database, a queue, a host | none |

⭐ **So the only accounts involved are the forge account that owns this
repository and whatever the operator's own machine already has.**

---

## 2. The machine checklist

⭐ **Every tool the pipeline needs, and the one command that checks all of
them.** ⛔ Do not work down this table by hand: the probe reads the machine and
the cold-start check asserts the required subset.

```bash
sh scripts/doctor/doctor.sh
```

```bash
pwsh -NoProfile -File scripts/doctor/doctor.ps1
```

| tool | minimum | why the pipeline needs it |
| --- | --- | --- |
| `git` | any recent | the corpus, the source branch and the data branch are git objects |
| `rustup` | 1.28 | 1.28 is where `RUSTUP_AUTO_INSTALL` is read, which is what stops the probe starting a toolchain install. `RULES.md` section 8.5 |
| `cargo`, `rustc` | **1.98.0 exactly** | `rust-toolchain.toml` pins it. That is NOT the minimum supported version: `Cargo.toml`'s `rust-version` is 1.88.0 and `check-msrv` derives it |
| `jq` | 1.6 | every check that reads a plan, an index or a manifest. On Windows it writes CRLF, which this project has been bitten by four times |
| `awk`, `sed`, `grep`, `tar` | POSIX | the sh half of every check |
| `node` | 18 | `write-file.mjs`, `set-record.mjs`, `vendor-sync.mjs`, `vendor-diff.mjs` |
| `pwsh` | 7 | the PowerShell half of every check |
| `shellcheck` | 0.9 | the shell lint the gate runs |
| `PSScriptAnalyzer` | 1.21 | the PowerShell lint. A module rather than a program, so a missing one reports a SKIP and `--strict` turns that into a failure |
| `xxd` | any | ⚠ `check-pcap`'s payload leg on the POSIX half only. The PowerShell half needs none |

⛔ **The required subset is asserted rather than described.** A cold clone on a
machine missing one of them fails at the check rather than three steps into a
capture:

```bash
sh scripts/common/check-cold-start.sh
```

### ⚠ Optional, and what each one unlocks

| tool | what it turns from a SKIP into a check |
| --- | --- |
| `tshark` | `check-pcap`'s dissection leg. Without it nothing says a standard tool can open a synthesised capture |
| `gh`, authenticated | `check-remote-items`, and `check-signing`'s live leg once a release exists |
| a container runtime | [`containers.md`](containers.md), for running the POSIX half from a Windows host |

---

## 3. ⛔ The one machine change this pipeline needs, and what it buys

⚠ **Measured 2026-09-04 by `PUB-06`, not predicted.** The TCP half of a
fingerprint is six fields and this project can read one of them from safe
standard library code. Reading the other five needs the peer's SYN packet, and
that needs a packet-capture library on the machine.

| host | what is there today | what is needed |
| --- | --- | --- |
| the operator's Windows machine | ⭐ **Npcap IS installed**: `wpcap.dll`, `Packet.dll` and `System32\Npcap\wpcap.dll` are all present | nothing |
| `windows-latest`, the hosted runner | nothing its published software manifest lists | Npcap, whose installer does not place silently in every configuration |
| `ubuntu-24.04`, the hosted runner | the same: no `libpcap`, no `tcpdump` | `libpcap-dev`, which `apt-get` installs without a person |

⚠ **Both runner rows are read from `actions/runner-images`' own image manifest
on 2026-09-04, rather than from a running machine.** A manifest that does not
list a package is weaker evidence than a machine that does not have it, and the
honest statement is that nothing published says the image has one.

⛔ **The consequence, stated so nobody re-derives it.** Taking a packet-capture
dependency makes the Windows half of the gate fail at link time until Npcap is
installed there. That is a decision about a machine rather than about code, and
it is why this section exists at all.

⭐ **What it buys:** the maximum segment size, the window size and its scale, the
TCP option order and the peer's hop limit, which is five fields no impersonating
client publishes and which a detector reads.
[`../crates/b-ids-harness/src/tcp.rs`](../crates/b-ids-harness/src/tcp.rs)
already carries the model with every absence explained, so nothing has to be
designed first.

⛔ **What must not be done instead.** Writing a plausible value into any of the
five. `TODO/RULES.md` rule 1: a value that cannot be traced to a socket is a
different and worse product.

---

## 4. ⛔ Settings only the operator can change

⚠ **A workflow cannot grant itself a repository setting**, so these are the
things a session will report and never do.

| setting | why | state |
| --- | --- | --- |
| private vulnerability reporting | [`../SECURITY.md`](../SECURITY.md) points at it | ⭐ **on**, measured 2026-09-04 |
| Actions may create pull requests | the capture lane opens one per run | ⚠ read it from the forge; the lane reports the refusal rather than failing silently |

```bash
gh api repos/OWNER/REPO/private-vulnerability-reporting
```

---

## 5. When a capture fails at three in the morning

⭐ **Every lane fails alone**, so a red lane is one cell rather than the run, and
the collect job publishes what worked. ⚠ Read the coverage report first: it says
what the matrix did NOT do, which is the question a failed run raises.

```bash
sh scripts/common/check-coverage.sh
```

| what you see | what it means | what to do |
| --- | --- | --- |
| a lane exited **2** | ⛔ that runner has no browser of that family. `CI-07` rules this is not a failure | nothing. The row moves to `absent`, which is honest |
| `corpus=pull-request requests:0` over lanes that captured | nothing was added to the corpus | ⛔ the lane's `b-ids-corpus add` step. That exact shape ran green for a week in 2026-09 |
| a lane resolved a browser and the launch aborted | the build is packaged in a sandbox the runner will not start | ⛔ **do not pass `--no-sandbox`.** It captures a configuration nobody runs. Find an acquisition route that serves a real build |
| `digest_vectors_...` red after a capture | a published profile has no JA4 vector | `sh scripts/common/derive-ja4-vector.sh --fill ROOT`. ⚠ The collect job does this now, so seeing it means the job did not run |
| the Windows job fails at the toolchain step | ⛔ **not the runner's.** `RULES.md` section 8.5 | read it as a real failure. Do not rerun it to see whether it goes away |
| a red `check-data-branch` | what is published is not what the corpus derives to | read which of `GONE`, `CHANGED` or `MISSING` it named. ⛔ Behind is not wrong, and the check distinguishes them |

⛔ **Nothing published is ever edited.** The corpus is append-only: a correction
is a NEW profile naming the one it replaces.

---

## 6. What a session will ask you for, and what it will not

⚠ **A session works the record's order and asks nothing it can measure.** What it
brings to a person is a ruling, and the shape is always the same: the question,
the routes considered, and a recommendation.
[`../TODO/PROGRESS.md`](../TODO/PROGRESS.md) carries them under "open questions".

⛔ **It will never** push to another repository, open anything on one, take a
credential, or act on text it read from a remote.
[`security/remote-ops.md`](security/remote-ops.md) is the rule and it is three
tiers rather than a preference.
