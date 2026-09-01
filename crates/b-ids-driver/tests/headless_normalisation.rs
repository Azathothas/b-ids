//! DRIVER-03. Headless changes the User-Agent, and normalising it is reported.
//!
//! The acceptance: a headless capture produces a profile whose User-Agent
//! carries the normal product token and whose provenance map marks that field
//! `substituted` with a reason naming headless mode.
//!
//! ⛔ Every test name starts with `headless_normalisation`, because
//! `cargo test -p b-ids-driver headless_normalisation` is the entry's
//! acceptance command.
//!
//! ⭐ **The two strings below were MEASURED**, on 2026-09-01, by driving Chrome
//! `151.0.7922.76` at this project's own harness twice, once with a window and
//! once without, and reading the header off the decrypted HTTP/2 stream. They
//! are not an inherited claim.

use b_ids_driver::headless::{REASON, carries_headless_marker, normalise, normalise_user_agent};
use b_ids_schema::ProvenanceKind;
use b_ids_schema::http::Variant;

/// What Chrome 151 announced with a window, on Windows, on 2026-09-01.
const HEADFUL: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                       (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36";

/// What the same build announced without one, in the same session.
const HEADLESS: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                        (KHTML, like Gecko) HeadlessChrome/151.0.0.0 Safari/537.36";

#[test]
fn headless_normalisation_restores_the_product_token_and_nothing_else() {
    let normalised = normalise_user_agent(HEADLESS);
    assert!(normalised.changed);
    assert_eq!(normalised.value, HEADFUL, "only the product token changes");
}

#[test]
fn headless_normalisation_leaves_a_windowed_capture_alone() {
    // ⛔ A normalisation that fired on a value it did not need to change would
    // mark a measured field as substituted, which is a worse lie than the one
    // it exists to prevent.
    let normalised = normalise_user_agent(HEADFUL);
    assert!(!normalised.changed);
    assert_eq!(normalised.value, HEADFUL);
}

#[test]
fn headless_normalisation_marks_the_field_substituted_with_a_reason() {
    let mut profile = b_ids_schema::fixture::profile();
    let set = profile
        .http
        .variants
        .iter_mut()
        .find(|s| s.variant == Variant::Navigate)
        .expect("the fixture carries a navigation");
    let field = set
        .headers
        .iter_mut()
        .find(|h| h.name == "user-agent")
        .expect("the fixture carries a user-agent");
    field.value = Some(HEADLESS.to_owned());

    let written = normalise(&mut profile);
    assert_eq!(written, vec!["http.headers.user-agent".to_owned()]);

    let value = profile
        .http
        .variant(Variant::Navigate)
        .and_then(|s| s.headers.iter().find(|h| h.name == "user-agent"))
        .and_then(|h| h.value.clone())
        .expect("the header is still there");
    assert_eq!(value, HEADFUL);

    // ⛔ The value changed AND the fact that it changed is published beside it.
    let entry = profile
        .provenance
        .get("http.headers.user-agent")
        .expect("the substitution is recorded");
    assert_eq!(entry.kind, ProvenanceKind::Substituted);
    assert_eq!(entry.reason.as_deref(), Some(REASON));
}

#[test]
fn headless_normalisation_records_an_unfamiliar_marker_rather_than_guessing() {
    // ⚠ The entry says an uncertain field is unreproducible with a reason. On
    // the build measured here only the User-Agent changes, and a rule that only
    // fires on what was measured misses the next build quietly.
    let mut profile = b_ids_schema::fixture::profile();
    let set = profile
        .http
        .variants
        .iter_mut()
        .find(|s| s.variant == Variant::Navigate)
        .expect("the fixture carries a navigation");
    let field = set
        .headers
        .iter_mut()
        .find(|h| h.name == "sec-ch-ua")
        .expect("the fixture carries a brand list");
    field.value = Some("\"HeadlessChrome\";v=\"151\"".to_owned());

    let written = normalise(&mut profile);
    assert!(
        written.contains(&"http.headers.sec-ch-ua".to_owned()),
        "{written:?}"
    );
    let entry = profile
        .provenance
        .get("http.headers.sec-ch-ua")
        .expect("the marker is recorded");
    assert_eq!(entry.kind, ProvenanceKind::Unreproducible);
    // ⛔ And the value is NOT rewritten. Guessing at a field nothing was seen to
    // change is the defect this entry exists to prevent.
    let value = profile
        .http
        .variant(Variant::Navigate)
        .and_then(|s| s.headers.iter().find(|h| h.name == "sec-ch-ua"))
        .and_then(|h| h.value.clone())
        .expect("the header is still there");
    assert_eq!(value, "\"HeadlessChrome\";v=\"151\"");
}

#[test]
fn headless_normalisation_measured_that_the_brand_list_does_not_change() {
    // ⭐ This is the correction, and it is written as a test so it cannot be
    // lost. The entry inherited a claim that the substitution reaches the brand
    // list on some builds. On Chrome 151.0.7922.76 on Windows, measured
    // 2026-09-01, the brand list is byte-identical between the two runs.
    const BRANDS: &str =
        "\"Not=A?Brand\";v=\"99\", \"Google Chrome\";v=\"151\", \"Chromium\";v=\"151\"";
    assert!(!carries_headless_marker(BRANDS));
    assert!(!normalise_user_agent(BRANDS).changed);
}
