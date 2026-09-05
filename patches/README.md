# patches

Every local change to a vendored tree, one section each.

⛔ **These files are OUTPUT.** Nothing here applies them. The vendored tree under
[`../vendor/`](../vendor/) is the truth, the series is regenerated from it by
`scripts/common/vendor-diff.mjs`, and editing a patch changes nothing about what
is built. [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md)
carries the reasoning and
[`../docs/history/todo/vendor.md`](../docs/history/todo/vendor.md) is the work.

⚠ **A section here is not derived and it does not regenerate.** The generator
knows what changed; only a person can say why, and why it cannot be done outside
the vendored tree.

```bash
node scripts/common/vendor-diff.mjs --check
```

That exits 1 when the series on disk no longer matches what the tree produces.
The offline half, whether every patch still names a file the tree has, is
`scripts/common/check-vendor.sh` and it runs in the gate.

---

## What every section carries

Four things, and a section without all four is incomplete:

1. **What the change is**, in one line.
2. **The entry it unblocks.**
3. **Why it cannot be done outside the vendored tree**, which is usually a seam
   the published interface does not expose.
4. ⭐ **The command that reproduces the defect.**
   [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) owns
   the rule and says what it buys at the next reconciliation.

---

## rustls

Vendored from `https://github.com/rustls/rustls` at tag `v/0.23.43`, commit
`fcf61cdbba30913cfd5b40aefa83989c6233812d`. The manifest is
[`../vendor/upstream.json`](../vendor/upstream.json).

### `0001-Cargo.toml.patch`

**What.** The vendored workspace's member list is trimmed to the two crates this
tree vendors, `rustls` and `rustls-test`, and the workspace dependency naming a
crate this tree does not vendor is dropped with them.

**Unblocks.** `VENDOR-01`, and through it `HARNESS-13`. Without it nothing in
this workspace can be resolved at all, because loading any package in this tree
loads the vendored manifest.

**Why it cannot be done outside the tree.** A workspace's member list lives in
its own manifest and cannot be narrowed from outside it. Fifteen of upstream's
seventeen members are excluded by
[`../vendor/upstream.json`](../vendor/upstream.json), each with its own reason,
so the unpatched manifest names members whose directories are not here.

**Could a release retire it.** ⛔ No. It exists because of what this tree chose
not to vendor, so a new release recreates it rather than removing the need for
it. ⚠ A release that adds or renames a member changes which lines it touches,
and the reconciliation regenerates it.

**The reproduction.** Restore upstream's manifest over the vendored one and ask
cargo to read the workspace:

```bash
cargo metadata --manifest-path vendor/rustls/Cargo.toml --no-deps --format-version 1 > /dev/null
```

Measured 2026-09-01 on one Windows 11 machine. With upstream's manifest in
place it exits **101**; with the patch in place it exits **0**. The first line
of the failure reads "failed to load manifest for workspace member bogo",
referred to a workspace member whose directory the exclude list removed, and
ends in operating system error 3.

⚠ **The failure is described rather than pasted.** Cargo names every path
absolutely, so the four lines it prints carry the operator's home directory,
and `scripts/common/check-no-secrets.sh --public` refuses an absolute home path
in a tracked file of a public repository. ⭐ The guard refused this section on
its first draft, which is the check working rather than a nuisance. The exit
code is what the acceptance reads and it is machine-independent.

---

## h2

Vendored from `https://github.com/hyperium/h2` at tag `v0.4.19`, commit
`d57d1b852fec9dda6d42d3454502006d52104da8`. The manifest is
[`../vendor/upstream.json`](../vendor/upstream.json).

⭐ **All four patches serve one change: a client can send the HTTP/2 PRIORITY
block a browser sends.** Every profile in this corpus carries the block and they
agree exactly, at `exclusive: true, stream_dependency: 0, weight_wire: 255`,
across two browsers, three majors and two platforms. A client that omits it
carries a zero in one field of four in a widely-read fingerprint.

⛔ **Upstream will not take it, and the reason is stated rather than guessed at.**
[`../docs/history/todo/emitters.md`](../docs/history/todo/emitters.md), `EMIT-03`, owns that sentence.
⚠ What it means HERE is that these patches are permanent rather than pending, and
[`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md) settles
that upstreaming is not a topic either way.

### `0001-Cargo.toml.patch`

**What.** The vendored workspace's member list is emptied, the dev-dependencies
are dropped, and the benchmark target is removed.

**Unblocks.** `EMIT-03`. Without it nothing in this workspace resolves at all,
because loading any package in this tree loads the vendored manifest.

**Why it cannot be done outside the tree.** A workspace's member list, its
dev-dependencies and its target table live in its own manifest and cannot be
narrowed from outside it. Upstream's five members are its test rig, its fuzz
target and two fixture generators, all excluded by
[`../vendor/upstream.json`](../vendor/upstream.json) with their reasons; the
dev-dependencies serve those same excluded directories, and `[[bench]]` names a
file under a directory the exclude list removed.

⚠ **The `[workspace]` table itself STAYS, emptied rather than deleted.** Cargo
resolves a path dependency against the outermost workspace that does not exclude
it, so a vendored crate with no table of its own inherits this repository's root
table and fails on the first key it does not carry. `VENDOR-01` measured exactly
that with rustls.

**Could a release retire it.** ⛔ No. It exists because of what this tree chose
not to vendor, so a new release recreates it.

**The reproduction.** Restore upstream's manifest over the vendored one and ask
cargo to read the workspace:

```bash
cargo metadata --manifest-path vendor/h2/Cargo.toml --no-deps --format-version 1 > /dev/null
```

Measured 2026-09-04 on one Windows 11 machine. With upstream's manifest in place
it exits **101** and the first line reads "failed to load manifest for workspace
member", naming `tests/h2-fuzz`, and ends in operating system error 3. With the
patch in place it exits **0**.

### `0002-src-frame-headers.rs.patch`

**What.** Three things, and they are one change: `HeadersFlag::set_priority`,
`Headers::set_stream_priority`, and `Headers::encode` writing the dependency
into the closure it already ran.

**Unblocks.** `EMIT-03`, and it is the whole of it.

**Why it cannot be done outside the tree.** `Headers::stream_dep` is a private
field, both send-path constructors hardcode it to `None`, and the closure
`EncodingHeaderBlock::encode` takes is passed `|_| {}` at the one call site that
matters. None of the three is reachable from outside the crate at any feature
level.

⭐ **The setter sets BOTH halves and that is deliberate.** ⛔ A head carrying the
PRIORITY flag with no block is a frame a peer cannot parse, and a block with no
flag is five bytes of header block that decodes as garbage. Two setters would
make the pair somebody's to remember.

⚠ **The frame length and any CONTINUATION split follow for free.** The closure
runs after the frame head and before the header block, and the payload length is
computed after it, so the five bytes are counted in the first frame and in no
other. That is why this entry was ever an `S`.

**Could a release retire it.** ⛔ No, and this is the one where that is a
statement about upstream's intent rather than about this tree's choices. The
reason is in the section heading above.

**The reproduction.** The suite's own control case builds a frame WITHOUT the
setter and requires that no block comes back:

```bash
cargo test -p b-ids-emit priority_block
```

`priority_block_the_patch_is_what_puts_it_there` is that case. Measured
2026-09-04: five cases pass, and reverting this patch takes four of the five red
while the control stays green, which is what says the block comes from here.

### `0003-src-frame-priority.rs.patch`

**What.** `StreamDependency::encode`, the half `load` never had.

**Unblocks.** `EMIT-03`. `0002` calls it.

**Why it cannot be done outside the tree.** `StreamDependency`'s three fields
are private and it exposes only `dependency_id`, so the exclusive bit and the
weight cannot be read out of it, let alone written. Upstream parses the block on
receive and has no path that writes one.

⚠ **The weight is written as the byte on the wire**, in `[0, 255]`, which is one
less than the `[1, 256]` the specification defines. That offset is upstream's own
convention on this type and this does not re-apply it.

**Could a release retire it.** ⛔ No, for `0002`'s reason.

**The reproduction.** `priority_block_the_harness_reads_back_what_the_schema_asked_for`
in the suite above puts three different dependencies through the encoder and
compares the bytes this project's own reader returns.

### `0004-src-lib.rs.patch`

**What.** `hpack` becomes public under the `unstable` feature.

**Unblocks.** `EMIT-03`.

**Why it cannot be done outside the tree.** ⭐ **It is upstream's own idiom,
applied to one more module.** `frame` and `proto` are already public under
`unstable`; `hpack` was not, and `frame::Headers::encode` takes
`&mut hpack::Encoder`, so the frame encoder was reachable in NAME and not in
USE. ⚠ Nothing about the gate changes: it is still `unstable`, which upstream
documents as an API with no backwards-compatibility promise, which is exactly
the trade a vendored tree exists to make.

**Could a release retire it.** ⚠ **Yes, and that would be the good outcome.** A
release that adds `hpack` to the `unstable` block makes this patch a no-op, and
the reconciliation would drop it. Nothing about it is contentious upstream; it
is an omission rather than a decision.

**The reproduction.** Remove the `#[cfg(feature = "unstable")] pub mod hpack;`
arm and build the emitter:

```bash
cargo build -p b-ids-emit
```

It fails on `module `hpack` is private` at the emitter's import.
