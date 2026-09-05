//! The acquisition block a profile may carry, and what refuses a bad one.
//!
//! ⭐ **The door sweep found nothing read this field.** The published schema
//! constrained it and `Profile::check` did not, so a profile could claim a
//! route no driver can produce and every check in the tree would have passed
//! it. `docs/history/todo/driver.md`, `DRIVER-05`.

mod support;

use b_ids_schema::Acquisition;
use support::{fixture, messages};

/// A digest of the right SHAPE, BUILT rather than pasted.
///
/// ⚠ What these tests are about is the shape: 64 lower-case hex. A pasted
/// sixty-four-character literal is also what `check-no-secrets` refuses on
/// sight, and narrowing that rule for a test fixture would be widening a
/// security check to suit a convenience.
fn digest_shaped() -> String {
    "0123456789abcdef".repeat(4)
}

#[test]
fn acquisition_absent_acquisition_is_not_a_defect() {
    // ⚠ A build already installed was not obtained by this project and has no
    // route. Absent is correct and is not a finding.
    let profile = fixture();
    assert!(profile.captured.acquisition.is_none());
    assert!(profile.check().is_empty(), "{:?}", profile.check());
}

#[test]
fn acquisition_a_well_formed_acquisition_is_accepted() {
    let mut profile = fixture();
    profile.captured.acquisition = Some(Acquisition {
        route: "chrome-for-testing".to_owned(),
        url: Some("https://example.invalid/build".to_owned()),
        sha256: digest_shaped(),
        bytes: 1234,
    });
    assert!(profile.check().is_empty(), "{:?}", profile.check());
}

#[test]
fn acquisition_a_route_no_driver_can_produce_is_refused() {
    let mut profile = fixture();
    profile.captured.acquisition = Some(Acquisition {
        route: "somebody-mailed-it-to-me".to_owned(),
        url: None,
        sha256: digest_shaped(),
        bytes: 1,
    });
    let defects = profile.check();
    assert!(
        messages(&defects).contains("captured.acquisition.route"),
        "{defects:?}"
    );
}

#[test]
fn acquisition_a_digest_that_is_not_one_is_refused() {
    let mut profile = fixture();
    profile.captured.acquisition = Some(Acquisition {
        route: "cache".to_owned(),
        // ⛔ Upper case and too short. Both are refused, and the message says
        // what a digest is rather than only that this is not one.
        sha256: "ABC123".to_owned(),
        url: None,
        bytes: 1,
    });
    let defects = profile.check();
    assert!(
        messages(&defects).contains("captured.acquisition.sha256"),
        "{defects:?}"
    );
}
