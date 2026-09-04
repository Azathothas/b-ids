//! `EMIT-02`. The ordered list of codepoint-and-body pairs, put back on a wire.
//!
//! ⛔ **Every test name contains `escape_hatch`**, because
//! `cargo test -p b-ids-emit escape_hatch` is this entry's acceptance and a
//! filter that selects nothing exits 0 having run nothing.
//!
//! ⭐ **The comparison is against the RAW BYTES, not against the model.**
//! Emitting from a model and comparing with that same model asks only whether
//! the emitter agrees with itself. What this asks is whether the bytes it
//! writes are the bytes the browser sent.

use std::path::{Path, PathBuf};

use b_ids_emit::{extensions_block, unnamed_codepoints};
use b_ids_schema::Profile;

/// ⛔ **Resolved, never assumed.** `corpus/` left the default branch in
/// `PUB-13`, and this suite walked up from its own manifest to find it.
/// `b_ids_schema::root` is the one place that question is answered now.
fn corpus_root() -> PathBuf {
    b_ids_schema::root::corpus_root_or_explain(Path::new(env!("CARGO_MANIFEST_DIR")))
}

/// Every published profile that carries a raw `ClientHello`.
fn profiles() -> Vec<Profile> {
    let mut found = Vec::new();
    let mut stack = vec![corpus_root().join("corpus").join("v1")];
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
            let profile: Profile = serde_json::from_str(&text).expect("it parses");
            if profile.raw.client_hello_hex.is_some() {
                found.push(profile);
            }
        }
    }
    found
}

/// Decode a hex string.
fn unhex(text: &str) -> Vec<u8> {
    let raw = text.as_bytes();
    assert!(raw.len().is_multiple_of(2), "an odd number of hex digits");
    raw.chunks(2)
        .map(|pair| {
            let hi = char::from(pair[0]).to_digit(16).expect("hex");
            let lo = char::from(pair[1]).to_digit(16).expect("hex");
            u8::try_from(hi * 16 + lo).expect("a byte")
        })
        .collect()
}

/// How many times `needle` occurs in `haystack`.
fn occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || needle.len() > haystack.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
}

#[test]
fn escape_hatch_every_capture_carries_codepoints_the_model_does_not_name() {
    // ⛔ THE PREMISE OF THE WHOLE ENTRY, asserted rather than believed. A model
    // with one typed field per extension could not hold these at all, and every
    // real capture in this corpus carries several.
    for profile in profiles() {
        let unnamed = unnamed_codepoints(&profile.tls);
        assert!(
            unnamed.len() >= 2,
            "{} carries {} unnamed codepoint(s), so this profile cannot exercise the escape hatch",
            profile.id,
            unnamed.len()
        );
    }
}

#[test]
fn escape_hatch_the_emitted_block_is_the_bytes_the_browser_sent() {
    // ⛔ THE ACCEPTANCE. The extensions are emitted in order, with their bodies
    // intact, and the emitted bytes are found in the raw hello exactly once.
    //
    // ⚠ FOUND RATHER THAN SLICED, and the difference matters: slicing would
    // need a second parser of the hello, and a second parser tests two
    // implementations against each other rather than testing the bytes. A block
    // of several hundred bytes occurring exactly once is not a coincidence.
    let all = profiles();
    assert!(!all.is_empty(), "no published profile carries a raw hello");
    for profile in &all {
        let block =
            extensions_block(&profile.tls).unwrap_or_else(|why| panic!("{}: {why:?}", profile.id));
        let hello = unhex(
            profile
                .raw
                .client_hello_hex
                .as_deref()
                .expect("this profile carries one"),
        );
        assert_eq!(
            occurrences(&hello, &block),
            1,
            "{}: the emitted extensions block of {} byte(s) is not in the {} byte hello exactly \
             once",
            profile.id,
            block.len(),
            hello.len()
        );
    }
}

#[test]
fn escape_hatch_an_unnamed_codepoint_keeps_its_body_and_its_place() {
    // ⭐ THE TWO PROPERTIES SEPARATELY, so a failure says which one broke.
    for profile in profiles() {
        let block = extensions_block(&profile.tls).expect("it emits");
        let mut offset = 2_usize;
        for captured in &profile.tls.extensions {
            let body = unhex(&captured.body_hex);
            let head = &block[offset..offset + 4];
            assert_eq!(
                u16::from_be_bytes([head[0], head[1]]),
                captured.codepoint,
                "{}: an extension moved",
                profile.id
            );
            assert_eq!(
                usize::from(u16::from_be_bytes([head[2], head[3]])),
                body.len(),
                "{}: a length was not derived from its body",
                profile.id
            );
            assert_eq!(
                &block[offset + 4..offset + 4 + body.len()],
                body.as_slice(),
                "{}: extension 0x{:04x} lost its body",
                profile.id,
                captured.codepoint
            );
            offset += 4 + body.len();
        }
        assert_eq!(offset, block.len(), "{}: the block has a tail", profile.id);
    }
}

#[test]
fn escape_hatch_a_reordered_list_is_a_different_block() {
    // ⛔ A COMPARISON NOBODY HAS SEEN REFUSE IS THEATRE. The order is a
    // fingerprint, so swapping two extensions has to change the bytes, and the
    // swapped block must NOT be found in the hello.
    let profile = profiles().into_iter().next().expect("a profile");
    let original = extensions_block(&profile.tls).expect("it emits");
    let mut swapped = profile.clone();
    swapped.tls.extensions.swap(0, 1);
    let moved = extensions_block(&swapped.tls).expect("it emits");
    assert_ne!(original, moved, "swapping two extensions changed nothing");
    assert_eq!(original.len(), moved.len(), "the swap changed the length");

    let hello = unhex(profile.raw.client_hello_hex.as_deref().expect("one"));
    assert_eq!(occurrences(&hello, &original), 1);
    assert_eq!(
        occurrences(&hello, &moved),
        0,
        "a reordered block was found in the hello anyway"
    );
}

#[test]
fn escape_hatch_a_body_the_capture_could_not_write_is_refused() {
    // ⛔ A REFUSAL, NEVER AN APPROXIMATION. An emitter that wrote its best guess
    // would produce a hello that exists nowhere.
    let mut profile = profiles().into_iter().next().expect("a profile");
    profile.tls.extensions[0].length += 1;
    let refusals = extensions_block(&profile.tls).expect_err("a disagreement is refused");
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    let said = refusals[0].to_string();
    assert!(said.contains("believe one of them"), "{said}");
}
