//! SCHEMA-04. The HTTP half, its variants, and the one privacy rule.
//!
//! The acceptance: a capture taken with no switch contains no header value at
//! all, and a capture taken with the switch on contains no `cookie` and no
//! `authorization`. ⭐ Both are asserted, and the first is asserted over a
//! fixture that DOES contain values, so the test can fail.
//!
//! ⛔ Every test name starts with `header_privacy`, because
//! `cargo test -p b-ids-schema header_privacy` is the entry's acceptance
//! command.

mod support;

use b_ids_schema::http::{HeaderSet, HttpHalf, ValuePolicy, Variant, is_never_recorded};
use support::raw_headers;

#[test]
fn header_privacy_the_input_fixture_carries_values_and_a_credential() {
    // ⭐ Asserted first, and separately, because every test below is vacuous if
    // the input has nothing to drop. A privacy test over an empty input passes
    // forever and proves nothing.
    let raw = raw_headers();
    assert!(raw.iter().all(|(_, v)| !v.is_empty()));
    assert!(raw.iter().any(|(n, _)| n == "cookie"));
    assert!(raw.iter().any(|(n, _)| n == "authorization"));
}

#[test]
fn header_privacy_the_default_records_no_value_at_all() {
    let set = HeaderSet::record(Variant::Navigate, raw_headers(), ValuePolicy::default());
    assert!(!set.carries_values());
    for field in &set.headers {
        assert!(field.value.is_none(), "{} carried a value", field.name);
    }
    // And nothing of the value survives into the serialised form either.
    let text = serde_json::to_string(&set).expect("serialises");
    assert!(!text.contains("Mozilla/5.0"), "{text}");
    assert!(!text.contains("not-a-real-value"), "{text}");
}

#[test]
fn header_privacy_names_only_is_the_default_policy() {
    // ⛔ A switch that has to be turned OFF for safety is a switch that ships
    // on. This asserts the default rather than the documentation of it.
    assert_eq!(ValuePolicy::default(), ValuePolicy::NamesOnly);
}

#[test]
fn header_privacy_values_on_still_drops_cookie_and_authorization() {
    // ⭐ THE TITLE STILL HOLDS AND THE SHAPE CHANGED. `SCHEMA-14` made a
    // credential recordable as PRESENT, in its wire position, so a header order
    // stops closing over a gap nothing marks. What is dropped is the VALUE, and
    // that is what this test asserts.
    let set = HeaderSet::record(Variant::Navigate, raw_headers(), ValuePolicy::WithValues);
    assert!(set.carries_values(), "the switch is on, so values are kept");
    let names: Vec<&str> = set.names().collect();
    assert!(names.contains(&"cookie"), "the NAME is kept: {names:?}");
    assert!(names.contains(&"authorization"), "{names:?}");
    for field in set.headers.iter().filter(|h| h.withheld) {
        assert!(field.value.is_none(), "{} carried a value", field.name);
    }
    let withheld: Vec<&str> = set.withheld().map(|h| h.name.as_str()).collect();
    assert_eq!(withheld, vec!["cookie", "authorization"]);

    let text = serde_json::to_string(&set).expect("serialises");
    assert!(!text.contains("not-a-real-value"), "{text}");
    assert!(!text.contains("not-a-real-token"), "{text}");
    assert!(
        text.contains("Mozilla/5.0"),
        "other values are kept: {text}"
    );
}

#[test]
fn header_privacy_the_credential_filter_is_case_insensitive() {
    // ⚠ HTTP/2 lower-cases header names and an HTTP/1.1 read does not. A rule
    // that only catches one spelling catches nothing on the other wire.
    assert!(is_never_recorded("Cookie"));
    assert!(is_never_recorded("AUTHORIZATION"));
    assert!(!is_never_recorded("cookie-policy"));

    let set = HeaderSet::record(
        Variant::Navigate,
        [("Cookie", "a=b"), ("Authorization", "Bearer x")],
        ValuePolicy::WithValues,
    );
    // ⛔ Both are recognised whatever their case, and both are withheld rather
    // than dropped. A case the filter missed would carry a VALUE, which is the
    // failure this test exists for.
    assert_eq!(set.headers.len(), 2, "{:?}", set.headers);
    assert!(set.headers.iter().all(|h| h.withheld), "{:?}", set.headers);
    assert!(
        set.headers.iter().all(|h| h.value.is_none()),
        "{:?}",
        set.headers
    );
    let text = serde_json::to_string(&set).expect("serialises");
    assert!(!text.contains("a=b"), "{text}");
    assert!(!text.contains("Bearer x"), "{text}");
}

#[test]
fn header_privacy_a_set_says_which_request_kind_produced_it() {
    // ⚠ Designed rather than derived: this project has no measured example of
    // two kinds differing, and the first capture of two kinds at one version is
    // what turns this into evidence. A set that does not say which kind
    // produced it is uncomparable whether or not any two happen to differ.
    let navigate = HeaderSet::record(Variant::Navigate, raw_headers(), ValuePolicy::NamesOnly);
    let subresource =
        HeaderSet::record(Variant::Subresource, raw_headers(), ValuePolicy::NamesOnly);
    assert_ne!(navigate, subresource);

    let half = HttpHalf {
        variants: vec![navigate, subresource],
    };
    assert!(half.variant(Variant::Navigate).is_some());
    assert!(half.variant(Variant::Reload).is_none());
}

#[test]
fn header_privacy_header_order_is_kept() {
    let set = HeaderSet::record(Variant::Navigate, raw_headers(), ValuePolicy::NamesOnly);
    let names: Vec<&str> = set.names().collect();
    assert_eq!(names.first(), Some(&"sec-ch-ua"));
    assert_eq!(names.last(), Some(&"accept-language"));
    // ⭐ EVERY NAME, INCLUDING THE CREDENTIALS. `SCHEMA-14`: the order is a
    // fingerprint signal and a gap nothing marks is a sequence a consumer
    // believes is whole and is not.
    assert_eq!(names.len(), raw_headers().len());
}

#[test]
fn header_privacy_a_credential_read_from_disk_is_refused() {
    // ⛔ THE THIRD DOOR, found by the door sweep. `HeaderSet::record` filters at
    // capture time and the harness filters on its own path; DESERIALISATION is
    // neither, because serde builds a HeaderField field by field. A profile
    // read from a file could carry a cookie header that no capture would have
    // produced, and a capture-time filter cannot hold a rule about a file.
    let mut json = serde_json::to_value(b_ids_schema::fixture::profile()).expect("serialises");
    json.pointer_mut("/http/variants/0/headers")
        .and_then(serde_json::Value::as_array_mut)
        .expect("the fixture carries a header set")
        .push(serde_json::json!({ "name": "cookie", "value": "session=x" }));

    let profile: b_ids_schema::Profile = serde_json::from_value(json).expect("it deserialises");
    let defects = profile.check();
    let text = defects
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("cookie"), "{text}");
    assert!(
        text.contains("navigate"),
        "the message names the variant: {text}"
    );
}
