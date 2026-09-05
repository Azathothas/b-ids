//! Embed the corpus at build time, from the index rather than from a walk.
//!
//! ⛔ **The index is the one statement of what is published**, so the list of
//! files embedded here is read out of it. A walk of the tree would be a second
//! answer to the same question, and a stray file under `corpus/` would then be
//! embedded as though somebody had published it.
//!
//! ⭐ **`b-ids-harness` is a BUILD dependency and deliberately not a real
//! one.** The digest that pins a corpus has exactly one home in this tree, and
//! a second implementation of SHA-256 for one identifier is the thing that home
//! exists to prevent. A build dependency is not in a consumer's runtime graph,
//! so the read-only path `b-ids-corpus` names in its own manifest stays
//! read-only. `docs/history/todo/library.md`, `LIB-01`.

use std::path::{Path, PathBuf};

/// The environment variable that names a corpus root explicitly.
///
/// ⭐ **The seam `PUB-11` needs.** When `corpus/` leaves the default branch,
/// this points at a fetched copy of the data branch and nothing else about this
/// crate changes.
const ROOT_ENV: &str = "B_IDS_CORPUS_ROOT";

fn main() {
    println!("cargo::rerun-if-env-changed={ROOT_ENV}");

    let root = corpus_root();
    let index_path = root.join("corpus").join("v1").join("index.json");
    let pointer_path = root.join("corpus").join("v1").join("latest.json");
    println!("cargo::rerun-if-changed={}", forward(&index_path));
    println!("cargo::rerun-if-changed={}", forward(&pointer_path));

    let index_bytes = std::fs::read(&index_path)
        .unwrap_or_else(|e| panic!("b-ids: cannot read {}: {e}", index_path.display()));
    // ⛔ THE IDENTIFIER IS A DIGEST OF THE INDEX, and the index carries a digest
    // of every published file, so pinning the index pins the corpus
    // transitively. One value, derived here, asserted independently by the
    // suite.
    let release = b_ids_harness::hex(&b_ids_harness::sha256(&index_bytes));

    let index: serde_json::Value = serde_json::from_slice(&index_bytes)
        .unwrap_or_else(|e| panic!("b-ids: {} is not an index: {e}", index_path.display()));
    let entries = index
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("b-ids: {} lists no profiles", index_path.display()));

    let mut generated = String::new();
    generated.push_str(&format!(
        "pub(crate) const RELEASE: &str = \"{release}\";\n"
    ));
    generated.push_str(&format!(
        "pub(crate) const POINTERS_JSON: &str = include_str!(\"{}\");\n",
        forward(&pointer_path)
    ));
    generated.push_str("pub(crate) const PROFILES: &[(&str, &str)] = &[\n");
    let mut embedded = 0_usize;
    for entry in entries {
        let path = entry
            .get("profile")
            .and_then(|p| p.get("path"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("b-ids: an index entry names no profile path"));
        let full = root.join(path);
        // An indexed file that is missing is a build failure. Otherwise the
        // embedded corpus would be smaller than the index it reports.
        assert!(
            full.is_file(),
            "b-ids: the index names {path} and there is no file at {}",
            full.display()
        );
        println!("cargo::rerun-if-changed={}", forward(&full));
        generated.push_str(&format!(
            "    (\"{path}\", include_str!(\"{}\")),\n",
            forward(&full)
        ));
        embedded += 1;
    }
    generated.push_str("];\n");
    assert!(embedded > 0, "b-ids: the index embedded no profile at all");

    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(out.join("corpus.rs"), generated.as_bytes())
        .unwrap_or_else(|e| panic!("b-ids: cannot write the generated corpus: {e}"));
}

/// Forward slashes, because the generated file is Rust source and a Windows
/// path in a string literal is a run of escape sequences.
fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Where the corpus is, named explicitly or found by walking up.
///
/// ⚠ **Walking up stops at the first directory that HAS a corpus**, rather than
/// at the first that has a `.git`. A crate built from a checkout inside another
/// checkout would otherwise embed the wrong one.
fn corpus_root() -> PathBuf {
    if let Some(named) = std::env::var_os(ROOT_ENV) {
        return PathBuf::from(named);
    }
    let manifest =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut here: &Path = &manifest;
    loop {
        if here.join("corpus").join("v1").join("index.json").is_file() {
            return here.to_path_buf();
        }
        match here.parent() {
            Some(parent) => here = parent,
            // ⛔ THE MESSAGE NAMES THE COMMAND, because since PUB-13 this is
            // the ORDINARY state of a fresh checkout rather than a mistake:
            // corpus/ lives on the source branch and the default branch carries
            // none. scripts/common/corpus-root.sh is the one thing that answers
            // where it is, and check-gate exports exactly this variable around
            // its three cargo steps. A build script that resolved a branch
            // itself would be a third copy of that order.
            None => panic!(
                "b-ids: no corpus above {}. corpus/ lives on the source branch: \
                 set {ROOT_ENV}=$(sh scripts/common/corpus-root.sh), or run the \
                 gate, which does it for you.",
                manifest.display()
            ),
        }
    }
}
