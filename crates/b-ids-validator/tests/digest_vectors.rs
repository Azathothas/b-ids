//! `VALID-04`. The published JA4 vectors, checked against this project's own
//! implementation.
//!
//! ⛔ **Every test name contains `digest_vectors`**, because
//! `cargo test -p b-ids-validator digest_vectors` is this entry's acceptance
//! and a filter that selects nothing exits 0 having run nothing.
//!
//! ⭐ **Two kinds of vector, and the difference is where the expected value
//! came from.** A `specification` vector's expected value is published in the
//! specification beside the list it belongs to. A `capture` vector's is derived
//! from a profile with `jq` and `sha256sum`, which is not this project's code.
//! ⛔ Neither came from running the implementation this suite checks, which is
//! the rule the entry states and the reason the vectors are worth anything.

use std::path::{Path, PathBuf};

use b_ids_harness::digest::{ja4, section};
use b_ids_schema::Profile;
use b_ids_schema::tls::TlsHalf;
use serde::Deserialize;

/// One published vector.
#[derive(Debug, Clone, Deserialize)]
struct Vector {
    kind: String,
    #[serde(default)]
    section: Option<String>,
    #[serde(default)]
    list: Option<String>,
    #[serde(default)]
    expect: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    ja4: Option<String>,
    #[serde(default)]
    ja4_r: Option<String>,
    #[serde(default)]
    ja4_ro: Option<String>,
}

/// The published vector file.
#[derive(Debug, Clone, Deserialize)]
struct Published {
    schema: String,
    vectors: Vec<Vector>,
}

/// ⛔ **Resolved, never assumed, and the VECTORS come from the same root.**
/// `PUB-13` moved `corpus/`, `raw/` and `vectors/` onto the source branch
/// together, because `b_ids_corpus::publish::build` reads the vector file from
/// the corpus root and because this suite asserts one vector per published
/// profile: a profile and its vector on two different branches leave the gate
/// red until both land, and there is no order of two merges that avoids it.
fn corpus_root() -> PathBuf {
    b_ids_schema::root::corpus_root_or_explain(Path::new(env!("CARGO_MANIFEST_DIR")))
}

fn published() -> Published {
    let path = corpus_root().join("vectors").join("ja4").join("v1.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every published profile, by identifier.
fn profiles() -> Vec<Profile> {
    let corpus = corpus_root().join("corpus").join("v1");
    let mut found = Vec::new();
    let mut stack = vec![corpus];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "index.json" || name == "latest.json" || !name.ends_with(".json") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a published profile");
            found.push(serde_json::from_str(&text).expect("a published profile parses"));
        }
    }
    found
}

/// A TLS half carrying one ALPN and nothing else, for the ALPN rule alone.
fn only_alpn(alpn: &str) -> TlsHalf {
    let mut tls = b_ids_schema::fixture::profile().tls;
    tls.alpn = if alpn.is_empty() {
        Vec::new()
    } else {
        vec![alpn.to_owned()]
    };
    tls
}

#[test]
fn digest_vectors_every_specification_vector_reproduces_its_published_value() {
    let file = published();
    assert_eq!(file.schema, "ja4-vectors/1");
    let mut checked = 0_usize;
    for vector in file.vectors.iter().filter(|v| v.kind == "specification") {
        let list = vector.list.clone().unwrap_or_default();
        let expect = vector
            .expect
            .clone()
            .expect("a specification vector expects");
        match vector.section.as_deref() {
            Some("ciphers" | "extensions") => {
                assert_eq!(section(&list), expect, "over {list}");
            }
            Some("alpn") => {
                assert_eq!(only_alpn(&list).ja4_alpn(), expect, "over {list:?}");
            }
            other => panic!("a vector names an unknown section: {other:?}"),
        }
        checked += 1;
    }
    // ⛔ THE COUNT IS ASSERTED. A vector file that lost half its rows would
    // otherwise pass every remaining one and report nothing.
    assert_eq!(checked, 10, "the specification vectors are not all here");
}

#[test]
fn digest_vectors_every_capture_vector_matches_the_profile_it_names() {
    let file = published();
    let corpus = profiles();
    assert!(!corpus.is_empty(), "the corpus published nothing to check");
    let mut checked = 0_usize;
    for vector in file.vectors.iter().filter(|v| v.kind == "capture") {
        let id = vector.id.clone().expect("a capture vector names a profile");
        let profile = corpus
            .iter()
            .find(|p| p.id.to_string() == id)
            .unwrap_or_else(|| panic!("{id} is not in the corpus"));
        assert_eq!(
            ja4(&profile.tls),
            vector.ja4.clone().expect("a capture vector expects a JA4"),
            "{id}"
        );
        assert_eq!(
            profile.tls.ja4_r(),
            vector.ja4_r.clone().expect("and a raw form"),
            "{id}"
        );
        assert_eq!(
            profile.tls.ja4_ro(),
            vector.ja4_ro.clone().expect("and an ordered raw form"),
            "{id}"
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        corpus.len(),
        "every published profile has a vector, and it does not"
    );
}

#[test]
fn digest_vectors_a_corrupted_vector_fails() {
    // ⛔ A COMPARISON NOBODY HAS SEEN REFUSE IS THEATRE. The vector is corrupted
    // one character at a time, and each corruption has to be caught.
    let file = published();
    let vector = file
        .vectors
        .iter()
        .find(|v| v.kind == "specification" && v.section.as_deref() == Some("ciphers"))
        .expect("a cipher vector");
    let list = vector.list.clone().expect("a list");
    let expect = vector.expect.clone().expect("an expected value");
    assert_eq!(section(&list), expect);

    // ⚠ One character of the EXPECTED value.
    let mut corrupted = expect.clone();
    corrupted.replace_range(0..1, "0");
    assert_ne!(
        section(&list),
        corrupted,
        "a corrupted expectation was accepted"
    );

    // ⚠ And one character of the INPUT, which is the corruption a re-ordering
    // or a dropped cipher would look like.
    let moved = list.replacen("002f", "002e", 1);
    assert_ne!(list, moved, "the corruption changed nothing");
    assert_ne!(
        section(&moved),
        expect,
        "a corrupted list produced the same hash"
    );
}

#[test]
fn digest_vectors_the_raw_forms_carry_what_the_hashed_one_hides() {
    // ⭐ THE ORDER-PRESERVING FORM SHOWS ORDER AND NOTHING ABOUT GREASE, which
    // is the correction docs/inherited-claims.md section 10 records. So a
    // profile whose extensions are shuffled has a stable JA4 and a moving
    // JA4_ro, and both facts are worth being able to state.
    for profile in profiles() {
        let lists = profile.tls.ja4_lists();
        assert!(
            !lists.ciphers_sorted.is_empty(),
            "{} has no ciphers at all",
            profile.id
        );
        // ⛔ SNI AND ALPN ARE IN THE ORDERED LIST AND NOT IN THE SORTED ONE.
        // The specification removes them from the hashed extension list and
        // keeps them in the original-order form.
        for excluded in ["0000", "0010"] {
            assert!(
                !lists
                    .extensions_sorted
                    .split(['_', ','])
                    .any(|c| c == excluded),
                "{} carries {excluded} in the hashed extension list",
                profile.id
            );
        }
        // ⚠ Every GREASE codepoint is gone from every list, which the
        // specification requires everywhere it appears.
        for part in lists
            .ciphers_original
            .split(',')
            .chain(lists.extensions_original.split(['_', ',']))
        {
            let Ok(value) = u16::from_str_radix(part, 16) else {
                continue;
            };
            assert!(
                !b_ids_schema::tls::is_grease_value(value),
                "{} leaves GREASE {part} in a JA4 list",
                profile.id
            );
        }
    }
}
