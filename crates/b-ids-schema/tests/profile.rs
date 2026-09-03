//! SCHEMA-01. The profile: one browser, one build, one platform, one channel,
//! one instant.
//!
//! The acceptance: a hand-written profile validates against the published
//! schema, a profile missing `captured.at` is rejected with a message naming
//! that field, and a profile whose `id` disagrees with its four keys is
//! rejected.

mod support;

use b_ids_schema::{Channel, Defect, Os, PlatformToken, ProfileId, ProvenanceKind, SCHEMA_ID};
use support::{as_json, check_schema_is_supported, fixture, messages, schema, validate};

#[test]
fn the_checker_implements_every_keyword_the_schema_uses() {
    // ⭐ This test is why the checker beside it is worth having. Without it, a
    // keyword added to the published schema and implemented by nobody would be
    // a constraint the schema states and nothing enforces, and every test below
    // would still pass.
    let unknown = check_schema_is_supported(&schema());
    assert!(
        unknown.is_empty(),
        "the schema uses keywords this checker does not implement, so those constraints are unenforced:\n  {}",
        unknown.join("\n  ")
    );
}

#[test]
fn a_hand_written_profile_validates_against_the_published_schema() {
    let profile = fixture();
    let problems = validate(&schema(), &as_json(&profile));
    assert!(
        problems.is_empty(),
        "the fixture does not validate:\n  {}",
        problems.join("\n  ")
    );
}

#[test]
fn the_schema_check_can_fail() {
    // ⛔ A validator that has never refused anything is a validator nobody knows
    // works. An undeclared property is the cheapest thing to plant.
    let mut json = as_json(&fixture());
    json.as_object_mut()
        .expect("a profile is an object")
        .insert("invented".to_owned(), serde_json::Value::Bool(true));
    let problems = validate(&schema(), &json);
    assert!(
        problems.iter().any(|p| p.contains("invented")),
        "an undeclared property was accepted: {problems:?}"
    );
}

#[test]
fn a_well_formed_profile_has_no_defects() {
    let defects = fixture().check();
    assert!(defects.is_empty(), "{}", messages(&defects));
}

#[test]
fn a_profile_with_no_capture_instant_is_rejected_naming_the_field() {
    let mut profile = fixture();
    profile.captured.at = String::new();
    let defects = profile.check();
    assert!(
        defects.contains(&Defect::FieldMissing {
            field: "captured.at".to_owned()
        }),
        "{}",
        messages(&defects)
    );
    assert!(messages(&defects).contains("captured.at"));
}

#[test]
fn a_capture_instant_that_is_not_iso_8601_utc_is_rejected() {
    // ⚠ A database's own "now" often produces this shape, and it never
    // string-compares against an ISO column.
    let mut profile = fixture();
    profile.captured.at = "2026-08-30 03:53:11".to_owned();
    let defects = profile.check();
    assert!(messages(&defects).contains("captured.at"), "{defects:?}");
}

#[test]
fn an_id_that_disagrees_with_the_four_keys_is_rejected() {
    let mut profile = fixture();
    profile.id = ProfileId::from_declared("chrome-152.0.7977.64-win64-stable");
    let defects = profile.check();
    let text = messages(&defects);
    assert!(
        text.contains("win64") && text.contains("linux64"),
        "the message has to show both, so a reader can see which key moved: {text}"
    );
}

#[test]
fn changing_any_one_key_changes_the_id() {
    // ⛔ Four keys, so four ways for the identity to move. A test that checked
    // one would pass over an id that ignored the other three.
    let base = fixture().derived_id();
    let mut version = fixture();
    version.browser.version = "152.0.7977.65".to_owned();
    let mut channel = fixture();
    channel.browser.channel = Channel::Beta;
    let mut platform = fixture();
    platform.platform.os = Os::Windows;
    let mut name = fixture();
    name.browser.name = "Firefox".to_owned();

    for (label, moved) in [
        ("version", version.derived_id()),
        ("channel", channel.derived_id()),
        ("platform", platform.derived_id()),
        ("name", name.derived_id()),
    ] {
        assert_ne!(base, moved, "moving {label} left the id unchanged");
    }
}

#[test]
fn a_version_with_no_build_is_rejected() {
    let mut profile = fixture();
    profile.browser.version = "152".to_owned();
    let defects = profile.check();
    assert!(
        messages(&defects).contains("browser.version"),
        "{defects:?}"
    );
}

#[test]
fn a_major_that_disagrees_with_the_version_is_rejected() {
    // ⚠ A value in two places with no check between them drifts, and the copy a
    // reader trusts is the wrong one.
    let mut profile = fixture();
    profile.browser.major = 151;
    let defects = profile.check();
    assert!(messages(&defects).contains("browser.major"), "{defects:?}");
}

#[test]
fn the_schema_version_is_part_of_the_data() {
    let mut profile = fixture();
    profile.schema = "browser-profile/2".to_owned();
    let defects = profile.check();
    assert!(messages(&defects).contains(SCHEMA_ID), "{defects:?}");
}

#[test]
fn a_profile_round_trips_through_json_unchanged() {
    let profile = fixture();
    let text = serde_json::to_string(&profile).expect("serialises");
    let back: b_ids_schema::Profile = serde_json::from_str(&text).expect("deserialises");
    assert_eq!(profile, back);
    // ⚠ And twice, because an unordered map would only show up on the second
    // pass.
    let again = serde_json::to_string(&back).expect("serialises");
    assert_eq!(text, again);
}

#[test]
fn the_platform_token_is_the_one_a_download_index_uses() {
    // ⚠ Not os plus arch joined with a dash. A published path and a downloaded
    // build have to spell the platform the same way.
    assert_eq!(
        PlatformToken::derive(Os::Linux, "x86_64").as_str(),
        "linux64"
    );
    assert_eq!(
        PlatformToken::derive(Os::Mac, "aarch64").as_str(),
        "mac-arm64"
    );
    // An architecture the mapping does not know is joined rather than refused,
    // because refusing here would design a ceiling in every published path.
    assert_eq!(
        PlatformToken::derive(Os::Linux, "riscv64").as_str(),
        "linux-riscv64"
    );
}

#[test]
fn digests_and_raw_are_siblings_of_the_measured_halves() {
    // ⛔ A derived value inside a measured block is how the two stop being
    // distinguishable. This asserts the shape rather than trusting the doc.
    let json = as_json(&fixture());
    for half in ["tls", "http2", "http"] {
        let block = json.get(half).expect("the half is present");
        for derived in ["digests", "ja3", "ja4", "akamai", "raw"] {
            assert!(
                block.get(derived).is_none(),
                "{half} carries {derived}, which is a derived value inside a measured block"
            );
        }
    }
    assert!(json.get("digests").is_some());
    assert!(json.get("raw").is_some());
}

#[test]
fn a_profile_with_a_vendor_field_is_a_draft() {
    let mut profile = fixture();
    assert!(!profile.is_draft());
    profile.provenance.insert(
        "http2.stream_priority",
        b_ids_schema::ProvenanceEntry {
            kind: ProvenanceKind::Vendor,
            reason: None,
        },
    );
    assert!(profile.is_draft());
    assert_eq!(
        profile.provenance.vendor_fields(),
        vec!["http2.stream_priority"]
    );
}

#[test]
fn profile_a_freshly_written_one_carries_the_licence_and_an_old_one_reads_it_back() {
    // ⛔ THE LEG THE CORPUS CANNOT PROVE TODAY. Every profile published before
    // 2026-09-03 predates the field and the corpus is append-only, so a check
    // over the published set finds nobody carrying it. This is where the rule
    // is held instead: what the writer produces, and what a reader does with a
    // profile written before the field existed. TODO/publish.md, PUB-07.
    let profile = b_ids_schema::fixture::profile();
    let text = serde_json::to_string(&profile).expect("it serialises");
    assert!(
        text.contains(&format!("\"license\":\"{}\"", b_ids_schema::LICENSE)),
        "a freshly written profile does not carry the licence: {text}"
    );

    // ⚠ AND AN OLD ONE STILL READS. A required field would refuse every profile
    // in the published corpus, which is why the schema lists it as optional.
    let mut value: serde_json::Value = serde_json::from_str(&text).expect("it parses");
    value
        .as_object_mut()
        .expect("a profile is an object")
        .remove("license");
    let older: b_ids_schema::Profile =
        serde_json::from_value(value).expect("a profile written before the field still reads");
    assert_eq!(older.license, b_ids_schema::LICENSE);
}
