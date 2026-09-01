# vendor

Third-party source that lives in this tree and is compiled by it.

[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) is the
rule. This is the work.

[`INDEX.md`](INDEX.md) is the list. [`ENTRY.md`](ENTRY.md) is the form.

---

## VENDOR-01. The vendored tree, and the four things that keep it honest

**Source** the operator, 2026-09-01, ruling that the TLS terminator is vendored here and patched here
**Category** vendor, **Priority** P1, **Effort** L, **Status** done

### Problem

The harness cannot terminate a TLS handshake, so `--ca-out` is absent,
`HARNESS-05` cannot reach a browser's HTTP/2, `HARNESS-10` has nothing to
compare, `DRIVER-01` has no URL worth pointing a browser at, and no profile in
this corpus can ever carry a real `ClientHello` taken through a completed
handshake.

Nothing in this tree speaks TLS. `crates/b-ids-harness/src/listener.rs` reads a
`ClientHello` off the socket and closes, and its own module comment says so.

### Premise

⭐ **Measured, not read.** On this host, 2026-09-01: a scratch package
depending on `rustls` 0.23.43 with `default-features = false` and the features
`std`, `ring`, `tls12` and `logging`, beside `rcgen` 0.14 with `pem` and
`ring`, resolved 65 packages, compiled 23 of them, and finished in 14.95s. The
`ring` build script found the Visual Studio Build Tools 17.14 install that
`cl.exe` lives in, without `cl` being on `PATH`.

The workspace declares two dependencies today, `serde` and `serde_json`, so
this is the change that gives the tree a compiled third-party surface.

⚠ **What was NOT measured** is a browser completing a handshake against it.
That is `HARNESS-13`, and it is a separate entry for that reason.

### Approach

Follow the practice the operator named, which is `Azathothas/bit-cli`'s and is
in the reference corpus at
[`../references/Azathothas__bit-cli/`](../references/Azathothas__bit-cli/). Its
shape is a manifest, a record of every local change, a derived patch series,
and a scan. Four deliverables, in that order.

**1. The manifest.** A vendor directory holding `upstream.json`, schema
versioned, one object per vendored upstream carrying its name, its repository,
its directory, the ref it was taken at, the commit that ref resolved to, the
instant it was vendored, and an exclude list where every exclusion carries its
own reason. bit-cli's file is the model and it is readable at
[`../references/Azathothas__bit-cli/tree/vendor/upstream.json`](../references/Azathothas__bit-cli/tree/vendor/upstream.json).

**2. The tree.** Upstream `rustls/rustls` at tag `v/0.23.43`, which resolves to
commit `fcf61cdbba30913cfd5b40aefa83989c6233812d`, read from the GitHub refs
API on 2026-09-01. Only the `rustls` crate is depended on; the workspace
members this tree never builds are excluded with a reason each, and the
excluded set starts from bit-cli's, which was derived against the same
upstream.

⛔ **An agent instruction file of any name is excluded**, per the vendoring
rule's own table. So is anything this repository's ignore rules would swallow,
because a file that lands on disk and never reaches a commit makes a fresh
clone build a different tree from the one that was tested. `TOOL-12` is what
that cost here already.

**3. The patch record and the derived series.** A patches directory with a
README carrying, per local change, what it is in one line, the entry it
unblocks, why it cannot be done outside the vendored tree, and ⭐ **the command
that reproduces the defect**, so the next reconciliation can run it against a
new copy and delete the patch when it exits zero. The series itself is
generated from the tree by a helper and never applied to anything: the tree is
the truth.

⚠ **The series is empty on the first vendoring, and the machinery still
lands.** A patch generator written the day the first patch is needed is a
generator nobody has seen work.

**4. The scan.** A check with two legs, the way `scripts/common/check-msrv.sh`
already has two. The offline leg asserts that the manifest describes the tree:
every entry names a directory that exists, every vendored directory has an
entry, every crate the manifest names resolves to a manifest declaring that
name, every recorded base is a 40-character commit, and every patch names a
file the tree has. The network leg fetches the recorded ref and reports whether
upstream has moved past it.

⛔ **Only the offline leg runs in the gate.** A gate that needs the network
fails on a machine that has none, and a check that cannot run is reported as a
skip rather than a pass.

Both halves of the check exist, per rule 4 of [`RULES.md`](RULES.md), and the
pair gets a `check-twins` row in the same change. The two helpers are node, so
they need no twin for the reason `scripts/README.md` gives for the two that are
already there.

⛔ **What this must not do.** It must not vendor `ring`, `rcgen` or anything
else this tree does not intend to patch: the ruling is about the TLS
terminator, and vendoring a dependency nobody edits buys a reconciliation cost
and nothing else. It must not build a pristine-tree-plus-patches layout, which
the vendoring rule has already rejected with its reasons. It must not make the
vendored crate a workspace member, because the workspace lints would then apply
to code nobody here wrote.

### Decision

**Which upstream, and how much of its closure comes with it.**

⭐ **Recommendation: vendor `rustls` alone, from upstream at a release tag, and
take `ring` and `rcgen` from the registry.** Ruled here on the practice the
operator named rather than left open, because bit-cli answers it: it vendors
the trees it patches and takes the rest from crates.io.

Three alternatives, and why each lost:

| alternative | why it lost |
| --- | --- |
| vendor `apify/rustls`, the fork already in the reference corpus | its fork carries client-side hello shaping this project does not use in a server, and its version reads `0.24.0-dev.0`, so every reconciliation would be against a moving branch rather than a release |
| vendor the whole closure, so the tree keeps its zero-registry-dependency shape | four trees to reconcile instead of one, and `ring` alone is several megabytes of pre-generated assembly nobody here will ever edit |
| a registry dependency on `rustls`, with no vendoring | the operator ruled against it on 2026-09-01, and the ruling is recorded in [`RULES.md`](RULES.md) |

### Prove

⛔ **The acceptance, and it is a command.**

```bash
sh scripts/common/check-vendor.sh
```

Passing means exit 0 with the manifest agreeing with the tree on every field
the offline leg reads, over a tree that has at least one vendored upstream in
it. The three that go with it, each read unpiped:

```bash
pwsh -NoProfile -File scripts/common/check-vendor.ps1
```

```bash
cargo build -p b-ids-harness
```

```bash
node scripts/common/vendor-diff.mjs --check
```

The last exits 0 when the series regenerated from the tree matches the series
on disk, which is what makes the record a derived artefact rather than a second
copy of the truth.

⚠ The network leg is run and its output pasted in the closing, and it is not in
the gate:

```bash
sh scripts/common/check-vendor.sh --upstream
```

---

## Closing

**Closed 2026-09-01T04:33:47Z.** rustls is vendored at `v/0.23.43`, compiled by
this tree, and the manifest, the change record, the derived series and the scan
all exist. The four acceptance commands, each read from the process that
produced it:

```text
$ sh scripts/common/check-vendor.sh
vendor ok: 1 upstream(s), 1 crate(s), 1 patch(es), manifest agrees with the tree
exit=0

$ pwsh -NoProfile -File scripts/common/check-vendor.ps1
vendor ok: 1 upstream(s), 1 crate(s), 1 patch(es), manifest agrees with the tree
exit=0

$ node scripts/common/vendor-diff.mjs --check
rustls: 1 patch(es), 0 difference(s)
exit=0

$ sh scripts/common/check-vendor.sh --upstream
upstream rustls: ref v/0.23.43 still resolves to the recorded base
upstream rustls: no newer release tag

vendor ok: 1 upstream(s), 1 crate(s), 1 patch(es), manifest agrees with the tree
exit=0
```

And the build, which is what says the tree is a dependency rather than a
directory. ⚠ Read over an up-to-date tree: a compiling line names its crate
directory absolutely, and `check-no-secrets --public` refuses an absolute home
path in a tracked file. The cold build of the same target, measured the same
day, compiled 23 crates including the vendored one in 13.26s.

```text
$ cargo build -p b-ids-harness
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
exit=0
```

### ⛔ The premise about an empty series was wrong, and the first patch was forced

The approach said the series would be empty on the first vendoring and that the
machinery should land anyway. **It is not empty.** Cargo resolves a path
dependency against the OUTERMOST workspace that does not exclude it, so the
vendored workspace manifest is loaded and its member list has to name only what
was vendored. Fifteen of upstream's seventeen members are excluded here, so the
unpatched manifest names directories that are not present and nothing in the
workspace resolves at all.

⭐ **So the machinery was proved by a patch that had to exist rather than by one
written to exercise it.** [`../patches/README.md`](../patches/README.md) carries
the change, the reason, and the command that reproduces the defect with its two
exit codes.

### ⚠ A second finding from the same measurement, and it is a trap

The failure a nested workspace produces does not name the workspace at all. With
`vendor` absent from this repository's own `exclude` list, the error was
`dependency.aws-lc-rs was not found in workspace.dependencies`, pointing at a key
this tree has never had. The root `Cargo.toml` now carries the exclusion and the
measurement beside it.

### What the vendored tree cost the checks, measured rather than predicted

Five of the checks that read the whole tree failed on the first run after it
landed: `check-docs`, `check-markers`, `check-one-home`, `check-placeholders`
and `check-no-secrets`. Two did not: `check-control-bytes` and the line-endings
check both passed over all 210 files, so both keep the vendored tree in scope.

⭐ **The exemption is per vendored tree and per derived series, never the whole
directory.** [`../vendor/upstream.json`](../vendor/upstream.json) and
[`../patches/README.md`](../patches/README.md) are this project's own writing and
stay in scope, and that boundary fired the same session: the secret scan refused
the patch record because a pasted cargo failure carried an absolute home path
into a public repository. The record now describes the failure and gives its exit
codes, which are what the acceptance reads.

⛔ **The secret scan was read before it was narrowed.** 38 hits over the vendored
tree with `--public --scope vendor`, in three categories, all of them read: four
doc comments naming a PEM header as text, four published author addresses in
licence and README files, and 30 long hex runs of which ten are commit ids
inside links to other public repositories and the other twenty are literal test
payloads. Not one is a live credential. The reading is in the check's own header.

⚠ **Those three numbers were wrong when this closing was first written**, taken
from a read of the output rather than from a count of it, and the claim audit
caught them before the push. ⛔ A number nobody counted is the defect
[`../docs/conventions/prose.md`](../docs/conventions/prose.md) names in as many
words, and this project put one in a security check's own header.

### Mutation-proved

⛔ **A guard that has never been seen to refuse is a guard nobody knows works.**
Every one of these was planted, run, and read unpiped, on both halves where a
pair exists.

| what was planted | what happened |
| --- | --- |
| an excluded path put back into the vendored tree | both halves: `website is listed as excluded and is present in vendor/rustls`, exit 1 |
| a vendored directory with no manifest entry | both halves: `vendor/ghost exists and no upstream in vendor/upstream.json names it`, exit 1 |
| the vendored file a patch names, removed | both halves, byte-identical message naming the patch and the missing file, exit 1 |
| the patch record removed | `patches/README.md does not exist, so no local change has a reason recorded`, exit 1 |
| a line appended to the vendored manifest without regenerating | `vendor-diff.mjs --check`: `differs: 0001-Cargo.toml.patch`, exit 1, and 0 again once restored |
| the recorded ref set to an older release | the network leg reported the moved commit AND `3 newer release tag(s), newest v/0.23.43` |
| a 40-hex run under a key called `token` in the manifest | both halves of the secret scan reported it, so the narrowing is by NAME and not by file |

### ⭐ The twin comparison found a divergence before the pair was ever committed

`ConvertFrom-Json` turns a string that parses as a date into a `[datetime]`, so
the PowerShell half was validating `09/01/2026 09:45:00` against a rule about ISO
8601 text and refusing a manifest that was correct. The sh half read the same
field through `jq` and got the string. ⚠ Neither half was wrong about its own
input; they were reading different things. The PowerShell half now reads the
stamps out of the raw JSON, which is what `jq` was already doing.

⚠ **A second Windows trap in the same file**: `jq` writes CRLF here, so a path
read through a pipe keeps the carriage return and
`vendor/rustls/rustls` plus a stray byte is not a directory. The check reported a
missing crate over a tree that was correct. Command substitution hides it and a
pipe does not.

### What is NOT here, named rather than left to be discovered

- ⛔ **No handshake is terminated.** This entry puts the library in the tree and
  proves it compiles and links. `HARNESS-13` is the wiring, `--ca-out` is still
  absent, and `HARNESS-02` stays `partial` until it lands.
- **Nothing but `rustls` is vendored.** `ring` and, later, the certificate
  minter come from the registry, pinned in the committed `Cargo.lock`, because
  the ruling is about the terminator and vendoring a dependency nobody edits
  buys a reconciliation cost and nothing else.
- ⚠ **The three agent-instruction exclusions matched nothing upstream**, which
  the materialise step reports rather than passing over. They are written
  defensively so that a release adding one does not land silently.
- **`.patch` files are outside `check-markers`'s extension list**, so the
  non-breaking space upstream carries in its own manifest comment travels into
  the generated series unchecked. That is the existing scope of that check
  rather than something this entry changed.
