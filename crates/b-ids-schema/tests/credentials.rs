//! SCHEMA-14. A credential's presence is a fingerprint, and it is currently a
//! hole.
//!
//! The acceptance: a capture carrying `cookie` produces a profile whose header
//! list holds an entry at that position marked as a withheld credential with no
//! value field at all; the serialised profile contains none of the credential's
//! bytes, asserted by searching the serialised text; and a profile hand-built
//! with a value on such an entry is refused.
//!
//! ⛔ Every test name starts with `credentials`, because
//! `cargo test -p b-ids-schema credentials` is the entry's acceptance command.
//!
//! ⛔ **Nothing here adds a way to record a credential's value.** Every test
//! below asserts the absence of one, and the one that supplies a value asserts
//! that it is refused.

mod support;

use b_ids_schema::http::{HeaderSet, ValuePolicy, Variant};

use support::{schema, validate};

/// A navigation's headers with a credential in the middle of the order.
///
/// ⚠ The value is a marker nothing else in this tree contains, so a search of
/// the serialised profile for it cannot pass by coincidence.
const SECRET: &str = "zzz-never-published-zzz";

fn headers() -> Vec<(&'static str, String)> {
    vec![
        ("sec-ch-ua", "\"Chromium\";v=\"151\"".to_owned()),
        ("cookie", format!("session={SECRET}")),
        ("accept", "text/html".to_owned()),
        ("authorization", format!("Bearer {SECRET}")),
        ("accept-language", "en-GB".to_owned()),
    ]
}

#[test]
fn credentials_are_recorded_as_present_in_their_wire_position() {
    // ⭐ THE HOLE THIS CLOSES. Before 2026-09-02 the credential entries were
    // dropped, name and all, so the recorded order read
    // sec-ch-ua, accept, accept-language and a consumer believed it had the
    // whole sequence.
    let set = HeaderSet::record(Variant::Navigate, headers(), ValuePolicy::WithValues);
    let names: Vec<&str> = set.names().collect();
    assert_eq!(
        names,
        vec![
            "sec-ch-ua",
            "cookie",
            "accept",
            "authorization",
            "accept-language"
        ],
        "the order is whole, including the credentials"
    );

    let withheld: Vec<&str> = set.withheld().map(|h| h.name.as_str()).collect();
    assert_eq!(withheld, vec!["cookie", "authorization"]);
}

#[test]
fn credentials_carry_no_value_field_at_all() {
    // ⛔ NOT AN EMPTY STRING, and not a placeholder. `value` is absent, which is
    // the same shape a names-only capture produces, and `withheld` is what says
    // the absence was deliberate rather than a policy.
    for policy in [ValuePolicy::NamesOnly, ValuePolicy::WithValues] {
        let set = HeaderSet::record(Variant::Navigate, headers(), policy);
        for field in set.withheld() {
            assert!(field.value.is_none(), "{field:?} under {policy:?}");
        }
        let serialised = serde_json::to_string(&set).expect("the set serialises");
        assert!(
            !serialised.contains(SECRET),
            "the credential's bytes are nowhere in the serialised set under {policy:?}"
        );
        assert!(
            serialised.contains("\"withheld\":true"),
            "the marker is serialised under {policy:?}: {serialised}"
        );
    }
}

#[test]
fn credentials_an_ordinary_header_carries_no_marker() {
    // ⛔ THE MARKER MEANS ONE THING. A `withheld` on every entry would say a
    // value was suppressed where none was, and a reader could no longer tell a
    // credential from a names-only capture.
    let set = HeaderSet::record(Variant::Navigate, headers(), ValuePolicy::NamesOnly);
    let ordinary: Vec<&str> = set
        .headers
        .iter()
        .filter(|h| !h.withheld)
        .map(|h| h.name.as_str())
        .collect();
    assert_eq!(ordinary, vec!["sec-ch-ua", "accept", "accept-language"]);
    let serialised = serde_json::to_string(&set).expect("the set serialises");
    assert_eq!(
        serialised.matches("\"withheld\":true").count(),
        2,
        "only the two credentials carry it: {serialised}"
    );
}

#[test]
fn credentials_a_profile_carrying_a_credential_value_is_refused() {
    // ⛔ THE READ PATH, not only the capture path. A capture-time filter cannot
    // hold a rule about a FILE, and a profile arriving from disk could carry
    // anything.
    let mut profile = b_ids_schema::fixture::profile();
    profile.http.variants[0]
        .headers
        .push(b_ids_schema::http::HeaderField {
            name: "cookie".to_owned(),
            value: Some(format!("session={SECRET}")),
            withheld: true,
        });
    let defects = profile.check();
    assert!(
        defects
            .iter()
            .any(|d| d.to_string().contains("a credential header with a value")),
        "{defects:?}"
    );
}

#[test]
fn credentials_a_credential_without_the_marker_is_refused() {
    // ⛔ Otherwise a credential header read from disk would look like an
    // ordinary one whose value simply was not recorded, and those are different
    // facts about what the wire carried.
    let mut profile = b_ids_schema::fixture::profile();
    profile.http.variants[0]
        .headers
        .push(b_ids_schema::http::HeaderField {
            name: "authorization".to_owned(),
            value: None,
            withheld: false,
        });
    let defects = profile.check();
    assert!(
        defects
            .iter()
            .any(|d| d.to_string().contains("not marked withheld")),
        "{defects:?}"
    );
}

#[test]
fn credentials_the_marker_on_an_ordinary_header_is_refused() {
    let mut profile = b_ids_schema::fixture::profile();
    profile.http.variants[0]
        .headers
        .push(b_ids_schema::http::HeaderField {
            name: "accept".to_owned(),
            value: None,
            withheld: true,
        });
    let defects = profile.check();
    assert!(
        defects
            .iter()
            .any(|d| d.to_string().contains("only a credential header is")),
        "{defects:?}"
    );
}

#[test]
fn credentials_the_published_schema_carries_the_marker() {
    // ⭐ THE CONTRACT, not only the type. A consumer validating against the file
    // this project publishes has to be able to read the marker, and a schema
    // with `additionalProperties: false` and no `withheld` would refuse every
    // profile that carries one.
    let schema = schema();
    let mut profile =
        serde_json::to_value(b_ids_schema::fixture::profile()).expect("the fixture serialises");
    profile["http"]["variants"][0]["headers"]
        .as_array_mut()
        .expect("the fixture has a header list")
        .push(serde_json::json!({ "name": "cookie", "withheld": true }));
    assert!(
        validate(&schema, &profile).is_empty(),
        "{:?}",
        validate(&schema, &profile)
    );
}
