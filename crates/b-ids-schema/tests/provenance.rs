//! SCHEMA-05. Provenance is per field, with four kinds and no more.
//!
//! The acceptance: a profile with an unreasoned `substituted` field is rejected
//! with a message naming that field, and a profile with a `vendor` field
//! validates as a draft and fails the published-profile check.
//!
//! ⛔ Every test name starts with `provenance`, because
//! `cargo test -p b-ids-schema provenance` is the entry's acceptance command.

mod support;

use b_ids_schema::{Defect, Profile, ProvenanceEntry, ProvenanceKind};
use support::{as_json, fixture, messages, schema, validate};

#[test]
fn provenance_an_unreasoned_substituted_field_is_rejected_naming_the_field() {
    let mut profile = fixture();
    profile.provenance.insert(
        "http.headers.sec-ch-ua-platform",
        ProvenanceEntry {
            kind: ProvenanceKind::Substituted,
            reason: None,
        },
    );
    let defects = profile.check();
    assert!(
        defects.contains(&Defect::ProvenanceReasonMissing {
            field: "http.headers.sec-ch-ua-platform".to_owned(),
            kind: ProvenanceKind::Substituted,
        }),
        "{}",
        messages(&defects)
    );
    assert!(messages(&defects).contains("sec-ch-ua-platform"));
}

#[test]
fn provenance_an_unreasoned_unreproducible_field_is_rejected_too() {
    // ⚠ Both kinds that require a reason, not one. A test over one of them
    // passes over a rule applied to half its subject.
    let mut profile = fixture();
    profile.provenance.insert(
        "tls.extensions.0xca34",
        ProvenanceEntry {
            kind: ProvenanceKind::Unreproducible,
            reason: None,
        },
    );
    let defects = profile.check();
    assert!(messages(&defects).contains("0xca34"), "{defects:?}");
}

#[test]
fn provenance_wire_and_vendor_need_no_reason() {
    let mut profile = fixture();
    profile.provenance.insert(
        "tls.cipher_suites",
        ProvenanceEntry {
            kind: ProvenanceKind::Wire,
            reason: None,
        },
    );
    profile.provenance.insert(
        "http2.stream_priority",
        ProvenanceEntry {
            kind: ProvenanceKind::Vendor,
            reason: None,
        },
    );
    let defects = profile.check();
    let reason_defects: Vec<&Defect> = defects
        .iter()
        .filter(|d| matches!(d, Defect::ProvenanceReasonMissing { .. }))
        .collect();
    assert!(reason_defects.is_empty(), "{reason_defects:?}");
}

#[test]
fn provenance_a_vendor_field_makes_the_profile_a_draft() {
    let mut profile = fixture();
    assert!(!profile.is_draft(), "the fixture starts publishable");
    profile.provenance.insert(
        "http2.stream_priority",
        ProvenanceEntry {
            kind: ProvenanceKind::Vendor,
            reason: None,
        },
    );
    // ⛔ Still well formed. A draft is not malformed: it is a profile that may
    // not be published, and the two are different questions.
    assert!(profile.check().is_empty(), "{}", messages(&profile.check()));
    assert!(profile.is_draft());
    assert_eq!(
        profile.provenance.vendor_fields(),
        vec!["http2.stream_priority"]
    );
}

#[test]
fn provenance_a_fifth_kind_is_refused() {
    // ⛔ Four is the whole vocabulary. A fifth is how a provenance model stops
    // meaning anything, so the parser refuses rather than carrying a string.
    let defect = ProvenanceEntry::parse("tls.cipher_suites", "guessed")
        .expect_err("a fifth kind is refused");
    assert_eq!(
        defect,
        Defect::ProvenanceKindUnknown {
            field: "tls.cipher_suites".to_owned(),
            found: "guessed".to_owned(),
        }
    );
    assert!(
        defect
            .to_string()
            .contains("wire, substituted, vendor, unreproducible")
    );
}

#[test]
fn provenance_parses_the_kind_and_the_kind_colon_reason_forms() {
    let plain = ProvenanceEntry::parse("f", "wire").expect("parses");
    assert_eq!(plain.kind, ProvenanceKind::Wire);
    assert_eq!(plain.reason, None);

    let reasoned = ProvenanceEntry::parse("f", "substituted:platform-token").expect("parses");
    assert_eq!(reasoned.kind, ProvenanceKind::Substituted);
    assert_eq!(reasoned.reason.as_deref(), Some("platform-token"));
    assert_eq!(reasoned.to_wire(), "substituted:platform-token");

    // A trailing colon with nothing after it is not a reason.
    let empty = ProvenanceEntry::parse("f", "substituted:").expect("parses");
    assert_eq!(empty.reason, None);
}

#[test]
fn provenance_round_trips_through_json_in_the_flat_form() {
    // ⭐ A sibling map of strings rather than a wrapper around every scalar.
    // Wrapping makes every consumer pay for a field most never read.
    let profile = fixture();
    let json = as_json(&profile);
    let map = json
        .get("provenance")
        .and_then(serde_json::Value::as_object)
        .expect("provenance is an object");
    assert_eq!(
        map.get("http.headers.sec-ch-ua-platform")
            .and_then(serde_json::Value::as_str),
        Some("substituted:platform-token")
    );

    let text = serde_json::to_string(&profile).expect("serialises");
    let back: Profile = serde_json::from_str(&text).expect("deserialises");
    assert_eq!(profile.provenance, back.provenance);
}

#[test]
fn provenance_a_fifth_kind_in_json_fails_to_deserialise() {
    let mut json = as_json(&fixture());
    json.pointer_mut("/provenance")
        .and_then(serde_json::Value::as_object_mut)
        .expect("provenance is an object")
        .insert(
            "tls.alpn".to_owned(),
            serde_json::Value::String("guessed".to_owned()),
        );
    let err = serde_json::from_value::<Profile>(json).expect_err("a fifth kind is refused");
    assert!(err.to_string().contains("tls.alpn"), "{err}");
}

#[test]
fn provenance_the_schema_enum_and_the_type_agree() {
    // ⛔ A fifth kind added to the type and not to the published schema, or the
    // reverse, is a vocabulary with two definitions. The schema declares
    // provenance values as free strings, so the agreement asserted here is that
    // every kind the type knows round-trips through the published shape.
    let mut profile = fixture();
    for kind in ProvenanceKind::all() {
        let reason = if kind.requires_reason() {
            Some("a-reason".to_owned())
        } else {
            None
        };
        profile
            .provenance
            .insert(format!("tls.{kind}"), ProvenanceEntry { kind, reason });
    }
    let problems = validate(&schema(), &as_json(&profile));
    assert!(problems.is_empty(), "{}", problems.join("\n  "));
    assert!(profile.check().is_empty(), "{}", messages(&profile.check()));
    assert_eq!(profile.provenance.len(), 2 + ProvenanceKind::all().len());
}
