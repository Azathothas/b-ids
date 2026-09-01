//! HARNESS-10. Check whether measuring changed what was measured.
//!
//! ⛔ Every test name starts with `modes`, because
//! `cargo test -p b-ids-harness modes` is the suite half of this entry. The
//! other half is `experiments/20-compare-capture-modes.sh`, which drives a real
//! browser twice; this half proves the comparison can tell a per-connection
//! draw from a mode effect, which is the one thing the driven run cannot
//! establish about itself.

mod support;

use b_ids_harness::modes::{Stability, Verdict};
use b_ids_harness::{CAPTURE_SCHEMA, Capture, Protocol, compare};
use b_ids_schema::tls::TlsHalf;

/// A capture carrying one TLS half and nothing else.
///
/// ⚠ Built from the committed hello and then mutated, so every field except the
/// one a test changes is a real parse of real bytes.
fn capture(connection: u32, tls: TlsHalf, protocol: Protocol) -> Capture {
    Capture {
        schema: CAPTURE_SCHEMA.to_owned(),
        connection,
        at: "2026-09-01T00:00:00Z".to_owned(),
        peer: "REDACTED".to_owned(),
        protocol,
        bytes_read: 0,
        raw_hex: String::new(),
        tls: Some(tls),
        http2: None,
        termination: None,
        request_line: None,
        header_names: Vec::new(),
        header_values: Vec::new(),
        notes: Vec::new(),
    }
}

/// The committed hello, parsed.
fn half() -> TlsHalf {
    let bytes = support::fixture_bytes("client-hello.hex");
    b_ids_harness::parse_record(&bytes)
        .expect("the committed hello parses")
        .tls
}

fn verdict<'a>(comparison: &'a b_ids_harness::Comparison, field: &str) -> &'a Verdict {
    &comparison
        .fields
        .iter()
        .find(|f| f.field == field)
        .unwrap_or_else(|| panic!("{field} is one of the compared fields"))
        .verdict
}

#[test]
fn modes_a_field_stable_in_both_runs_and_equal_agrees() {
    let raw = vec![
        capture(1, half(), Protocol::TlsRaw),
        capture(2, half(), Protocol::TlsRaw),
    ];
    let terminated = vec![
        capture(1, half(), Protocol::TlsTerminated),
        capture(2, half(), Protocol::TlsTerminated),
    ];
    let comparison = compare(&raw, &terminated);
    assert!(
        comparison.differing().is_empty(),
        "{:?}",
        comparison.differing()
    );
    assert!(matches!(
        verdict(&comparison, "tls.legacy_version"),
        Verdict::Agrees(_)
    ));
}

#[test]
fn modes_a_field_stable_in_both_runs_and_different_is_a_mode_effect() {
    // ⭐ The finding this entry exists to be able to report. A browser that
    // offered a different ALPN list when the handshake completes would make
    // every profile taken through the terminating surface a reading of the
    // harness rather than of the browser.
    let raw = vec![
        capture(1, half(), Protocol::TlsRaw),
        capture(2, half(), Protocol::TlsRaw),
    ];
    let changed = || {
        let mut tls = half();
        tls.alpn = vec!["http/1.1".to_owned()];
        tls
    };
    let terminated = vec![
        capture(1, changed(), Protocol::TlsTerminated),
        capture(2, changed(), Protocol::TlsTerminated),
    ];
    let comparison = compare(&raw, &terminated);
    match verdict(&comparison, "tls.alpn") {
        Verdict::Differs { raw, terminated } => {
            assert_eq!(raw, "h2,http/1.1");
            assert_eq!(terminated, "http/1.1");
        }
        other => panic!("expected a difference, got {other:?}"),
    }
    assert_eq!(comparison.differing().len(), 1);
}

#[test]
fn modes_a_field_drawn_per_connection_is_not_comparable_rather_than_a_finding() {
    // ⛔ THE MUTATION THIS DESIGN EXISTS FOR. A browser draws GREASE per
    // connection, so two runs differ on it with no mode change involved. A
    // comparison that reported that as a difference would produce a list that
    // reads like evidence and is not.
    let drawn = |value: u16| {
        let mut tls = half();
        tls.grease.values = vec![value];
        tls.cipher_suites[0] = value;
        tls
    };
    let raw = vec![
        capture(1, drawn(0x0a0a), Protocol::TlsRaw),
        capture(2, drawn(0x1a1a), Protocol::TlsRaw),
    ];
    let terminated = vec![
        capture(1, drawn(0x2a2a), Protocol::TlsTerminated),
        capture(2, drawn(0x3a3a), Protocol::TlsTerminated),
    ];
    let comparison = compare(&raw, &terminated);

    match verdict(&comparison, "tls.cipher_suites") {
        Verdict::NotComparable { raw, terminated } => {
            assert_eq!(*raw, Stability::Varies { distinct: 2 });
            assert_eq!(*terminated, Stability::Varies { distinct: 2 });
        }
        other => panic!("a per-connection draw is not comparable, got {other:?}"),
    }
    assert!(
        comparison.differing().is_empty(),
        "a draw is never reported as a mode effect: {:?}",
        comparison.differing()
    );
}

#[test]
fn modes_stripping_grease_is_what_makes_the_cipher_list_comparable_at_all() {
    // ⭐ The same input as the test above. The raw list cannot be compared and
    // the same list with GREASE removed agrees, which is the whole reason both
    // are carried as separate fields.
    let drawn = |value: u16| {
        let mut tls = half();
        tls.grease.values = vec![value];
        tls.cipher_suites[0] = value;
        tls
    };
    let raw = vec![
        capture(1, drawn(0x0a0a), Protocol::TlsRaw),
        capture(2, drawn(0x1a1a), Protocol::TlsRaw),
    ];
    let terminated = vec![
        capture(1, drawn(0x2a2a), Protocol::TlsTerminated),
        capture(2, drawn(0x3a3a), Protocol::TlsTerminated),
    ];
    let comparison = compare(&raw, &terminated);
    assert!(matches!(
        verdict(&comparison, "tls.cipher_suites.no_grease"),
        Verdict::Agrees(_)
    ));
}

#[test]
fn modes_one_connection_per_mode_is_reported_as_thin() {
    // ⛔ One connection cannot establish stability, so every field reads as
    // stable and the comparison is weaker than it looks. The caller is told.
    let raw = vec![capture(1, half(), Protocol::TlsRaw)];
    let terminated = vec![capture(1, half(), Protocol::TlsTerminated)];
    let comparison = compare(&raw, &terminated);
    assert!(comparison.thin());

    let wide = vec![
        capture(1, half(), Protocol::TlsRaw),
        capture(2, half(), Protocol::TlsRaw),
    ];
    assert!(!compare(&wide, &wide).thin());
}

#[test]
fn modes_a_run_with_no_hello_counts_none_rather_than_agreeing_with_everything() {
    let raw: Vec<Capture> = Vec::new();
    let terminated = vec![
        capture(1, half(), Protocol::TlsTerminated),
        capture(2, half(), Protocol::TlsTerminated),
    ];
    let comparison = compare(&raw, &terminated);
    assert_eq!(comparison.raw_hellos, 0);
    assert_eq!(comparison.terminated_hellos, 2);
    assert!(comparison.differing().is_empty(), "nothing to differ from");
    // ⚠ And every field is not comparable rather than agreeing, which is what
    // stops an empty run reading as a clean result.
    assert_eq!(
        comparison.not_comparable().len(),
        comparison.fields.len(),
        "an absent side makes every field not comparable"
    );
}

#[test]
fn modes_every_compared_field_renders_on_a_real_hello() {
    // ⛔ A field named in the list and unrenderable would be reported as absent
    // on every capture forever, which is a comparison that quietly covers less
    // than its own list says.
    let one = vec![capture(1, half(), Protocol::TlsRaw)];
    let comparison = compare(&one, &one);
    for field in &comparison.fields {
        assert!(
            !matches!(
                field.verdict,
                Verdict::NotComparable {
                    raw: Stability::Absent,
                    ..
                }
            ),
            "{} renders on a real hello",
            field.field
        );
    }
    assert_eq!(comparison.fields.len(), b_ids_harness::modes::FIELDS.len());
}
