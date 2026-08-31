//! SCHEMA-02. The TLS half, in wire order, with unknown codepoints kept.
//!
//! The acceptance: a fixture containing an extension the parser has no name for
//! round-trips to identical bytes, and a fixture whose GREASE extension carries
//! one zero byte parses rather than erroring.
//!
//! ⛔ Every test name in this file starts with `tls_extensions`, because
//! `cargo test -p b-ids-schema tls_extensions` is the entry's acceptance
//! command and it filters on the test name.

mod support;

use b_ids_schema::tls::{Extension, TlsHalf, is_grease_value};
use support::{as_json, fixture_tls, schema, validate};

#[test]
fn tls_extensions_unknown_codepoint_round_trips_to_identical_bytes() {
    // ⛔ This is the whole reason the model is an ordered list of
    // codepoint-and-body pairs. A version bump in another repository stopped
    // because its model could name neither 0x12e0 nor 0xca34.
    let half = fixture_tls();
    assert!(
        half.extensions.iter().any(|e| e.codepoint == 0xca34),
        "the fixture has to contain a codepoint with no name, or this proves nothing"
    );

    let text = serde_json::to_string(&half).expect("serialises");
    let back: TlsHalf = serde_json::from_str(&text).expect("deserialises");
    assert_eq!(half, back);
    assert_eq!(text, serde_json::to_string(&back).expect("serialises"));

    let unknown = back
        .extensions
        .iter()
        .find(|e| e.codepoint == 0xca34)
        .expect("the unknown codepoint survived");
    assert_eq!(unknown.body_hex, "deadbeef");
    assert_eq!(unknown.length, 4);
}

#[test]
fn tls_extensions_keep_wire_order() {
    // A set would lose this, and order is part of the fingerprint.
    let half = fixture_tls();
    let order: Vec<u16> = half.extensions.iter().map(|e| e.codepoint).collect();
    let text = serde_json::to_string(&half).expect("serialises");
    let back: TlsHalf = serde_json::from_str(&text).expect("deserialises");
    let round_tripped: Vec<u16> = back.extensions.iter().map(|e| e.codepoint).collect();
    assert_eq!(order, round_tripped);
    assert_ne!(
        order,
        {
            let mut sorted = order.clone();
            sorted.sort_unstable();
            sorted
        },
        "the fixture's order has to differ from its sorted form, or this test cannot fail"
    );
}

#[test]
fn tls_extensions_grease_carrying_one_zero_byte_parses() {
    // ⚠ A GREASE codepoint may carry an arbitrary body. A model that assumed an
    // empty body would refuse this, and a real browser sends it.
    let half = TlsHalf {
        extensions: vec![Extension {
            codepoint: 0x5a5a,
            length: 1,
            body_hex: "00".to_owned(),
        }],
        ..fixture_tls()
    };
    let text = serde_json::to_string(&half).expect("serialises");
    let back: TlsHalf = serde_json::from_str(&text).expect("deserialises");
    let grease = &back.extensions[0];
    assert!(grease.is_grease());
    assert_eq!(grease.body_hex, "00");
    assert!(grease.length_agrees());
}

#[test]
fn tls_extensions_grease_predicate_covers_the_sixteen_and_nothing_else() {
    // ⛔ RFC 8701: both bytes equal, low nibble a. Sixteen values, and the test
    // counts them rather than spot-checking two.
    let found: Vec<u16> = (0..=u16::MAX).filter(|v| is_grease_value(*v)).collect();
    assert_eq!(found.len(), 16, "found {found:?}");
    assert_eq!(found.first(), Some(&0x0a0a));
    assert_eq!(found.last(), Some(&0xfafa));
    // Near misses, so the predicate is not simply "high == low".
    assert!(!is_grease_value(0x0b0b));
    assert!(!is_grease_value(0x0a0b));
    assert!(!is_grease_value(0x1301));
}

#[test]
fn tls_extensions_declared_length_that_disagrees_with_the_body_is_reported() {
    // ⛔ Not repaired. The profile records what the wire carried, and the
    // disagreement is itself the measurement. Padding or truncating to make the
    // two agree is the forbidden pattern.
    let half = TlsHalf {
        extensions: vec![Extension {
            codepoint: 0x002b,
            length: 9,
            body_hex: "0303".to_owned(),
        }],
        ..fixture_tls()
    };
    let bad = half.length_disagreements();
    assert_eq!(bad.len(), 1);
    assert_eq!(bad[0].codepoint, 0x002b);
    // And a well-formed one is not reported.
    assert!(fixture_tls().length_disagreements().is_empty());
}

#[test]
fn tls_extensions_key_shares_record_entry_lengths() {
    // Two builds sending one group with different key sizes are two different
    // handshakes, and a group identifier alone cannot say so.
    let half = fixture_tls();
    assert!(half.key_shares.iter().any(|k| k.entry_len == 32));
    assert!(half.key_shares.iter().any(|k| k.group == 0x0a0a));
}

#[test]
fn tls_extensions_validate_against_the_published_schema() {
    let json = as_json(&support::fixture());
    let problems = validate(&schema(), &json);
    assert!(problems.is_empty(), "{}", problems.join("\n  "));
    let extensions = json
        .pointer("/tls/extensions")
        .and_then(serde_json::Value::as_array)
        .expect("the TLS half carries an extension array");
    assert!(extensions.len() >= 3);
}
