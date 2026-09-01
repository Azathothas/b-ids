# fuzz

Coverage-guided fuzzing of every parser the harness exposes to the network.
[`../TODO/harness.md`](../TODO/harness.md), `HARNESS-09`.

⭐ **The half that runs everywhere is not here.**
[`../crates/b-ids-harness/tests/hostile.rs`](../crates/b-ids-harness/tests/hostile.rs)
drives the same function over mutations of the committed captures, on every
host and every push, with no nightly toolchain and no extra tool. This
directory is the coverage-guided half, and it needs both.

⛔ **One target, not one per parser.** The list of parsers lives in
`b_ids_harness::fuzz::drive_every_parser`, so a fifth parser is covered the day
it lands. Four targets would be four lists to keep in step, and the staleness
would be invisible because each would keep passing.

---

## Running it

⚠ **`RUSTUP_TOOLCHAIN` has to be set, and finding out why cost a run.**
[`../rust-toolchain.toml`](../rust-toolchain.toml) pins this tree to an exact
stable compiler, and that file applies to every directory under the repository
root including this one. A nightly image is not enough: rustup reads the
toolchain file and downloads the pinned stable, then `-Z sanitizer` is refused
as a nightly-only option. Anything that runs this, including a CI job, overrides
the toolchain explicitly.

```bash
RUSTUP_TOOLCHAIN=nightly cargo fuzz run parsers -- -runs=1000000
```

**Seed the corpus first**, from the harness's own committed captures. ⭐ Random
input almost never survives a length check, so an unseeded run explores one
comparison; a truncated real hello reaches every field the parser reads.

```bash
mkdir -p fuzz/corpus/parsers
```

```bash
for f in crates/b-ids-harness/fixtures/*.hex; do node -e "const fs=require('fs');fs.writeFileSync(process.argv[2],Buffer.from(fs.readFileSync(process.argv[1],'utf8').replace(/\s+/g,''),'hex'))" "$f" "fuzz/corpus/parsers/$(basename "$f" .hex)"; done
```

⛔ **The corpus and the artifacts are not committed**, and
[`../.gitignore`](../.gitignore) carries the reason: one run here produced 856
corpus files, a different set on every host, and a crash input is a regression
test rather than a file to keep in a fuzz directory.

---

## ⛔ Windows cannot run this, and each route failed differently

Measured 2026-09-01 on one Windows 11 host (`10.0.26200.9168`), with
`cargo-fuzz 0.13.2` and `nightly 1.100.0`. ⚠ Three routes, three distinct
blockers, kept so the next session does not walk them again.

| route | what stopped it |
| --- | --- |
| MSVC, default sanitizer | `LNK1104: cannot open file 'clang_rt.asan_dynamic_runtime_thunk-x86_64.lib'`. The AddressSanitizer runtime is an optional Visual Studio component and this Build Tools install does not have it. ⭐ **This is the one route with a named, operator-actionable fix.** |
| MSVC, `-s none` | `LNK2001: unresolved external symbol __stop___sancov_pcs`, four times. Coverage instrumentation is still emitted, and `link.exe` provides no section-boundary symbols for the sections libFuzzer reads its counters from. |
| GNU nightly, `-s none` | libFuzzer's own `FuzzerExtFunctionsWindows.cpp` does not compile under mingw `g++`. Its Windows support is written for MSVC. |

⭐ **The fourth route works and it is what CI should use**: a Linux container,
with the toolchain overridden. One million runs finished in 295 seconds with no
crash and no timeout.

⚠ **A run leaves nothing behind on the host** when the container is removed with
it, and [`../docs/containers.md`](../docs/containers.md) is the procedure,
including naming the platform on every pull.
