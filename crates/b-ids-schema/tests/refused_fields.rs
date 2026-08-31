//! SCHEMA-07. What must never be in the model.
//!
//! The acceptance: a profile whose identifier is derived from a digest is
//! rejected, and a profile carrying a session ticket in its TLS half is
//! rejected, both with messages naming the field.
//!
//! ⛔ Every test name starts with `refused_fields`, because
//! `cargo test -p b-ids-schema refused_fields` is the entry's acceptance
//! command.

mod support;

use b_ids_schema::tls::Extension;
use b_ids_schema::{Defect, ProfileId};
use support::{fixture, messages};

#[test]
fn refused_fields_a_well_formed_profile_refuses_nothing() {
    assert!(fixture().refused_fields().is_empty());
}

#[test]
fn refused_fields_an_identifier_taken_from_a_digest_is_rejected() {
    // ⛔ A digest is DERIVED from a profile. Keying on one lets a consumer
    // round-trip a profile through a value that cannot reconstruct it, and it
    // makes the identity move every time the browser reshuffles.
    let mut profile = fixture();
    let digest = "t13i1517h2_8daaf6152771_4980c97edce0";
    profile.digests.ja4 = Some(digest.to_owned());
    profile.id = ProfileId::from_declared(digest);

    let defects = profile.check();
    assert!(
        defects.contains(&Defect::DigestUsedAsIdentity {
            field: "digests.ja4".to_owned()
        }),
        "{}",
        messages(&defects)
    );
    let text = messages(&defects);
    assert!(text.contains("digests.ja4"), "{text}");
    assert!(text.contains("never derived from a digest"), "{text}");
}

#[test]
fn refused_fields_a_digest_stored_beside_the_identity_is_fine() {
    // ⚠ STORING a digest is correct and is what `digests` is for. What is
    // refused is keying on one, so this has to be seen to pass or the test
    // above proves only that something was refused.
    let mut profile = fixture();
    profile.digests.ja4 = Some("t13i1517h2_8daaf6152771_4980c97edce0".to_owned());
    profile.digests.akamai = Some(profile.http2.akamai_text());
    assert!(profile.refused_fields().is_empty(), "{:?}", profile.check());
}

#[test]
fn refused_fields_a_session_ticket_in_the_tls_half_is_rejected() {
    // ⛔ Connection state, not identity. A resumed handshake carries real ticket
    // bytes and a cold one carries the codepoint with an empty body.
    let mut profile = fixture();
    profile.tls.extensions.push(Extension {
        codepoint: 0x0023,
        length: 4,
        body_hex: "a1b2c3d4".to_owned(),
    });
    let defects = profile.check();
    let text = messages(&defects);
    assert!(text.contains("tls.extensions.0x0023"), "{text}");
    assert!(text.contains("session ticket"), "{text}");
    assert!(text.contains("learned from the network"), "{text}");
}

#[test]
fn refused_fields_a_pre_shared_key_is_rejected_too() {
    // ⚠ Both codepoints, not one. A resumed handshake offers pre_shared_key
    // where a fresh one offers session_ticket, and a rule that caught one would
    // pass over exactly the connection this project must not average in.
    let mut profile = fixture();
    profile.tls.extensions.push(Extension {
        codepoint: 0x0029,
        length: 2,
        body_hex: "beef".to_owned(),
    });
    let text = messages(&profile.check());
    assert!(text.contains("tls.extensions.0x0029"), "{text}");
    assert!(text.contains("pre-shared key"), "{text}");
}

#[test]
fn refused_fields_an_empty_session_ticket_extension_is_kept() {
    // ⭐ The distinction the whole entry rests on. PRESENCE of the codepoint is
    // identity: a browser sends session_ticket empty on a cold connection and
    // the extension being there at all is part of the fingerprint. Its CONTENTS
    // are connection state.
    let mut profile = fixture();
    profile.tls.extensions.push(Extension {
        codepoint: 0x0023,
        length: 0,
        body_hex: String::new(),
    });
    assert!(
        profile.refused_fields().is_empty(),
        "an empty ticket extension is a measurement, not connection state: {:?}",
        profile.check()
    );
    assert!(
        profile.tls.extensions.iter().any(|e| e.codepoint == 0x0023),
        "and it is kept rather than dropped"
    );
}

#[test]
fn refused_fields_the_bytes_still_live_in_the_raw_capture() {
    // ⛔ "Must not: silently drop these at capture time." The refusal is about
    // promoting them into a parsed field; the raw hello is not edited, because
    // a capture is a moment that cannot be retaken.
    let mut profile = fixture();
    profile.raw.client_hello_hex = Some("160301aabbccdd0023000401020304".to_owned());
    assert!(
        profile.refused_fields().is_empty(),
        "the raw capture is never the thing refused"
    );
    assert!(profile.raw.client_hello_hex.is_some());
}
