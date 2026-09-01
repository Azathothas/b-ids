# patches

Every local change to a vendored tree, one section each.

⛔ **These files are OUTPUT.** Nothing here applies them. The vendored tree under
[`../vendor/`](../vendor/) is the truth, the series is regenerated from it by
`scripts/common/vendor-diff.mjs`, and editing a patch changes nothing about what
is built. [`../docs/methodology/vendoring.md`](../docs/methodology/vendoring.md)
carries the reasoning and
[`../TODO/vendor.md`](../TODO/vendor.md) is the work.

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
