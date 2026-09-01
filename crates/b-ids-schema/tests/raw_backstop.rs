//! SCHEMA-06. Record everything the wire carried, from the first commit.
//!
//! The acceptance: a profile is rebuilt from its `raw` block alone by a second
//! code path, and the result compares equal to the parsed profile field by
//! field.
//!
//! ⛔ Every test name starts with `raw_backstop`, because
//! `cargo test -p b-ids-schema raw_backstop` is the entry's acceptance command.
//!
//! ⚠ **The bytes come from the harness's own committed fixtures rather than
//! from a copy.** A fixture copied into a second place is two fixtures that
//! drift, and this test is about whether the STORED bytes are sufficient, so it
//! has to use the ones a capture stores.

use std::path::{Path, PathBuf};

use b_ids_harness::rebuild::{differences, rebuild};
use b_ids_harness::{hex, unhex};
use b_ids_schema::http::ValuePolicy;
use b_ids_schema::{Profile, Raw, RecordLayer};

/// The harness's fixture directory, which is where the committed bytes live.
fn harness_fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../b-ids-harness/fixtures")
}

fn fixture_bytes(name: &str) -> Vec<u8> {
    let path = harness_fixtures().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the committed fixture {} is readable: {e}", path.display()));
    unhex(&text).unwrap_or_else(|e| panic!("{} is hex: {e}", path.display()))
}

/// A profile whose measured halves were produced BY parsing the raw block it
/// carries, which is the shape a capture writes.
fn profile_from_bytes() -> Profile {
    let hello = fixture_bytes("client-hello.hex");
    let connection = fixture_bytes("h2-connection.hex");

    let parsed = b_ids_harness::parse_record(&hello).expect("the committed hello parses");
    let mut notes = Vec::new();
    let http2 =
        b_ids_harness::h2::parse_connection(&connection, ValuePolicy::NamesOnly, &mut notes)
            .expect("the committed connection parses");

    let mut profile = b_ids_schema::fixture::profile();
    profile.tls = parsed.tls;
    profile.http2 = http2.half.clone();
    profile.raw = Raw {
        client_hello_hex: Some(hex(&hello)),
        settings_frame_hex: http2.frames.first().map(frame_hex),
        http2_frames_hex: http2.frames.iter().map(frame_hex).collect(),
        request_line_hex: None,
        connection_hex: Some(hex(&connection)),
        record_layer: Some(RecordLayer {
            version: profile.tls.record_version,
            declared_length: u16::try_from(hello.len() - 5).expect("a fixture record is short"),
            bytes_arrived: hello.len() - 5,
            fragmented: false,
        }),
    };
    profile
}

/// One frame's head and payload together, as the raw block stores it.
fn frame_hex(frame: &b_ids_harness::RawFrame) -> String {
    let mut head = Vec::new();
    head.extend_from_slice(&frame.declared_length.to_be_bytes()[1..]);
    head.push(frame.frame_type);
    head.push(frame.flags);
    let identifier = (u32::from(frame.reserved_bit) << 31) | frame.stream_id;
    head.extend_from_slice(&identifier.to_be_bytes());
    format!("{}{}", hex(&head), frame.payload_hex)
}

#[test]
fn raw_backstop_rebuilds_the_tls_half_from_the_raw_block_alone() {
    // ⛔ The acceptance. A raw block nobody has re-parsed is a claim rather
    // than a backstop.
    let profile = profile_from_bytes();
    let rebuilt = rebuild(&profile.raw, ValuePolicy::NamesOnly);
    let tls = rebuilt.tls.expect("the raw block carries a ClientHello");
    assert_eq!(tls, profile.tls);
}

#[test]
fn raw_backstop_rebuilds_the_http2_half_from_the_raw_block_alone() {
    let profile = profile_from_bytes();
    let rebuilt = rebuild(&profile.raw, ValuePolicy::NamesOnly);
    let http2 = rebuilt.http2.expect("the raw block carries frames");
    assert_eq!(http2, profile.http2);
}

#[test]
fn raw_backstop_reports_every_half_the_raw_block_does_not_reproduce() {
    // ⭐ The measured halves are the ones the bytes produce, and the report
    // names any that are not. The HTTP half of this profile is not rebuildable
    // from these bytes, because the fixture connection is HTTP/2 and carries no
    // cleartext request: that is reported rather than passed over.
    let profile = profile_from_bytes();
    let missing = differences(&profile, ValuePolicy::NamesOnly);
    assert_eq!(
        missing,
        vec!["http: the raw block carries no cleartext request".to_owned()],
        "an unexpected half failed to rebuild"
    );
}

#[test]
fn raw_backstop_rebuilds_a_cleartext_request_through_the_one_construction_path() {
    // ⛔ Through HeaderSet::record, which is where the credential rule lives. A
    // rebuild that assembled the fields itself would be a fourth door into it.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\nCookie: not-a-real-value\r\n\
                    Accept: */*\r\n\r\n";
    let raw = Raw {
        connection_hex: Some(hex(request)),
        ..Raw::default()
    };
    let rebuilt = rebuild(&raw, ValuePolicy::WithValues);
    let http = rebuilt.http.expect("the raw block carries a request");
    let names: Vec<&str> = http.variants[0].names().collect();
    assert_eq!(names, vec!["Host", "Accept"]);
    let serialised = serde_json::to_string(&http).expect("serialises");
    assert!(!serialised.contains("not-a-real-value"), "{serialised}");
}

#[test]
fn raw_backstop_refuses_a_raw_block_that_disagrees_with_itself() {
    // ⛔ A backstop that disagrees with itself is worse than none. The settings
    // frame is kept in two fields for profiles written before the list existed,
    // and a check asserts they are the same frame.
    let mut profile = profile_from_bytes();
    assert!(profile.check().is_empty(), "{:?}", profile.check());

    profile.raw.settings_frame_hex = Some("00".to_owned());
    let defects = profile.check();
    assert_eq!(defects.len(), 1, "{defects:?}");
    assert!(
        defects[0].to_string().contains("raw.settings_frame_hex"),
        "{defects:?}"
    );
}

#[test]
fn raw_backstop_refuses_a_record_layer_that_disagrees_with_the_hello() {
    let mut profile = profile_from_bytes();
    let record = profile.raw.record_layer.as_mut().expect("a record layer");
    record.bytes_arrived += 1;
    let defects = profile.check();
    assert_eq!(defects.len(), 1, "{defects:?}");
    assert!(
        defects[0].to_string().contains("bytes_arrived"),
        "{defects:?}"
    );
}

#[test]
fn raw_backstop_keeps_a_frame_type_the_model_has_no_name_for() {
    // ⛔ An unknown thing is not an absent thing. A codepoint nobody can name
    // still gets its length and its body recorded verbatim, and the raw block
    // is where verbatim lives.
    let profile = profile_from_bytes();
    assert!(
        profile
            .raw
            .http2_frames_hex
            .iter()
            .any(|f| f.contains("63") && f.ends_with("dead")),
        "{:?}",
        profile.raw.http2_frames_hex
    );
}

#[test]
fn raw_backstop_the_schema_is_additive() {
    // ⚠ Fields are added, never removed and never repurposed. A profile
    // written before the list existed still reads, because the new fields
    // default rather than being required.
    let older = r#"{"client_hello_hex":"1603","settings_frame_hex":null}"#;
    let raw: Raw = serde_json::from_str(older).expect("an older raw block still reads");
    assert_eq!(raw.client_hello_hex.as_deref(), Some("1603"));
    assert!(raw.http2_frames_hex.is_empty());
    assert!(raw.connection_hex.is_none());
    assert!(raw.record_layer.is_none());
}

#[test]
fn raw_backstop_reports_a_half_that_disagrees_with_its_own_bytes() {
    // ⛔ Found by mutation: every other test here exercises the ABSENT branch,
    // so a comparison that accepted any rebuild at all passed the whole file.
    // A backstop that never reported a difference is a backstop nobody has
    // seen work.
    let mut profile = profile_from_bytes();
    assert_eq!(
        differences(&profile, ValuePolicy::NamesOnly).len(),
        1,
        "only the HTTP half is expected to be unrebuildable here"
    );

    // A measured half that the stored bytes do not produce. This is exactly
    // what a parser change, or a hand edit, looks like from the outside.
    profile.tls.cipher_suites.push(0x1301);
    let reported = differences(&profile, ValuePolicy::NamesOnly);
    assert!(
        reported
            .iter()
            .any(|d| d == "tls: the rebuilt half differs from the recorded one"),
        "{reported:?}"
    );

    profile
        .http2
        .pseudo_header_order
        .push(":protocol".to_owned());
    let reported = differences(&profile, ValuePolicy::NamesOnly);
    assert!(
        reported
            .iter()
            .any(|d| d == "http2: the rebuilt half differs from the recorded one"),
        "{reported:?}"
    );
}

#[test]
fn raw_backstop_refuses_a_raw_block_whose_bytes_spell_out_a_credential() {
    // ⛔ FOUND BY THE DOOR SWEEP, and it is the fourth door into the credential
    // rule. The parsed fields drop `cookie`; the bytes beside them spell it
    // out, hex-encoded, so a grep for the plaintext finds nothing and the
    // credential is in the capture anyway.
    //
    // ⚠ The profile is REFUSED rather than repaired. Editing the raw bytes
    // would destroy the one artefact that survives every parser defect.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\nCookie: not-a-real-value\r\n\r\n";
    let mut profile = profile_from_bytes();
    profile.raw.connection_hex = Some(hex(request));

    let defects = profile.check();
    assert_eq!(defects.len(), 1, "{defects:?}");
    let said = defects[0].to_string();
    assert!(said.contains("raw.connection_hex"), "{said}");
    assert!(said.contains("cookie"), "{said}");

    // ⚠ And the same on the other wire, where the name is lower-cased and the
    // header is `authorization`. A rule holding one spelling holds nothing on
    // the other.
    profile.raw.connection_hex = Some(hex(
        b"GET / HTTP/1.1\r\nauthorization: Bearer not-a-real-token\r\n\r\n",
    ));
    let defects = profile.check();
    assert_eq!(defects.len(), 1, "{defects:?}");
    assert!(defects[0].to_string().contains("authorization"));
}

#[test]
fn raw_backstop_a_clean_cleartext_capture_is_not_refused() {
    // ⚠ A guard that always fires is a guard somebody switches off. The word
    // has to be a header LINE, not a word in a body or a path.
    let mut profile = profile_from_bytes();
    profile.raw.connection_hex = Some(hex(
        b"GET /cookie-policy HTTP/1.1\r\nHost: example\r\nAccept: */*\r\n\r\n",
    ));
    assert!(profile.check().is_empty(), "{:?}", profile.check());
}
