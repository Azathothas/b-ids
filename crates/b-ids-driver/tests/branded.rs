//! Branded and unbranded builds are different products, end to end.
//!
//! ⛔ **Every test name starts with `branded`, because
//! `cargo test -p b-ids-driver branded` is this entry's acceptance and a filter
//! that selects nothing exits 0 having run nothing.** That is exactly what the
//! command did for as long as this file was absent.
//!
//! ⭐ **The two halves meet here and nowhere else.** The driver knows which
//! route a build came from; the validator knows that a branded profile has a
//! vendor entry in its brand list. Neither on its own catches a profile that
//! claims a brand its build does not carry, and this suite is the pass that
//! drives an unbranded acquisition and hands the claim to the rule.
//!
//! `TODO/driver.md`, `DRIVER-06`.

use b_ids_driver::acquire::{Candidate, Route, acquire_with, plan};
use b_ids_driver::resolve::Family;
use b_ids_validator::{Outcome, check_brand};

/// The bytes a fake route hands back. ⛔ Not a browser, and never published as
/// one.
const PAYLOAD: &[u8] = b"not a browser, and never published as one";

fn digest(bytes: &[u8]) -> String {
    b_ids_harness::hex(&b_ids_harness::sha256(bytes))
}

/// Answer only for the automation index, so the acquisition that succeeds is
/// the unbranded one.
fn only_the_index(candidate: &Candidate) -> Result<Vec<u8>, String> {
    if candidate.route == Route::ChromeForTesting {
        Ok(PAYLOAD.to_vec())
    } else {
        Err("arranged failure: this lane purged the machine".to_owned())
    }
}

#[test]
fn branded_the_automation_index_serves_an_unbranded_build() {
    // ⛔ THE ROUTE IS WHAT SAYS SO, rather than a field somebody set. A lane
    // that provisions from the automation index is capturing an unbranded
    // build whether or not anybody wrote it down.
    let acquired = acquire_with(
        &plan(Family::Chrome, Some("151.0.7922.76")),
        only_the_index,
        digest,
    )
    .expect("the index answered");
    assert_eq!(acquired.route, Route::ChromeForTesting);
    assert_eq!(
        acquired.route.branded(),
        Some(false),
        "the automation index serves unbranded builds"
    );
    // ⚠ And the routes it tried first are kept rather than collapsed, so a
    // reader can tell an outage from a build that does not exist.
    assert_eq!(acquired.refusals.len(), 2, "{:?}", acquired.refusals);
}

#[test]
fn branded_a_route_that_does_not_decide_it_says_so() {
    // ⛔ NONE IS AN ANSWER. A build already on the machine is whatever somebody
    // installed, and returning `true` for it would be the synthesised brand
    // list this entry forbids.
    assert_eq!(Route::Installed.branded(), None);
    assert_eq!(Route::Cache.branded(), None);
    // ⚠ The two vendor indexes DO decide it, and they decide it differently:
    // the enterprise one serves the vendor's own product.
    assert_eq!(Route::Vendor.branded(), Some(true));
    assert_eq!(Route::EdgeEnterprise.branded(), Some(true));
    assert_eq!(Route::ChromeForTesting.branded(), Some(false));
}

#[test]
fn branded_an_unbranded_capture_claiming_a_brand_is_refused_by_the_validator() {
    // ⛔ THE ACCEPTANCE. An unbranded capture whose profile claims
    // `branded: true` is rejected by the validator with a message naming the
    // brand list.
    let acquired = acquire_with(
        &plan(Family::Chrome, Some("151.0.7922.76")),
        only_the_index,
        digest,
    )
    .expect("the index answered");
    let unbranded = acquired
        .route
        .branded()
        .expect("this route decides it")
        .eq(&false);
    assert!(unbranded, "the acquisition was not the unbranded one");

    // ⭐ The fixture's brand list is an unbranded build's: Chromium and a fake
    // brand, with no vendor entry beside them. ⛔ Nothing in it is a
    // measurement.
    //
    // ⚠ AND THE VALUES HAVE TO BE RECORDED FOR THE RULE TO RUN AT ALL. A
    // profile keeps header NAMES by default, which is SCHEMA-04's privacy rule,
    // and check 3 answers `NotCheckable` over one. A capture that wants this
    // field checked turns values on deliberately, and this is the pass that
    // says so out loud rather than the entry claiming enforcement it does not
    // have on the default profile.
    let mut profile = b_ids_schema::fixture::profile();
    assert!(
        matches!(check_brand(&profile), Outcome::NotCheckable(_)),
        "a names-only profile cannot be checked for a brand list, and saying so is the answer"
    );
    set_navigate_headers(&mut profile, &b_ids_schema::fixture::raw_headers());
    profile.browser.branded = false;
    assert_eq!(
        check_brand(&profile),
        Outcome::Passed,
        "a profile that agrees with its own brand list is coherent"
    );

    // ⛔ THE ONE FIELD, FLIPPED. Nothing else about the profile changes, so the
    // refusal can only be about the claim.
    profile.browser.branded = true;
    let outcome = check_brand(&profile);
    let Outcome::Failed(findings) = outcome else {
        panic!("an unbranded build claiming a brand was accepted: {outcome:?}");
    };
    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = &findings[0];
    assert_eq!(finding.field, "http.headers.sec-ch-ua");
    assert!(
        finding.message.contains("brand list"),
        "the message does not name the brand list: {}",
        finding.message
    );
    assert!(
        finding.message.contains("Google Chrome"),
        "the message does not name the entry that is missing: {}",
        finding.message
    );
}

#[test]
fn branded_a_branded_build_with_no_vendor_entry_is_refused_from_the_other_side() {
    // ⚠ THE SAME RULE FROM THE OTHER DIRECTION, because a check that only ever
    // fires one way is half a check. A profile that claims `branded: false`
    // while its list DOES carry the vendor entry is the mistake a lane
    // provisioning from the vendor channel would make.
    let mut profile = b_ids_schema::fixture::profile();
    profile.browser.branded = true;
    let raw = b_ids_schema::fixture::raw_headers()
        .into_iter()
        .map(|(name, value)| {
            if name == "sec-ch-ua" {
                (
                    name,
                    "\"Chromium\";v=\"152\", \"Google Chrome\";v=\"152\", \"Not?A_Brand\";v=\"24\""
                        .to_owned(),
                )
            } else {
                (name, value)
            }
        })
        .collect::<Vec<_>>();
    set_navigate_headers(&mut profile, &raw);
    assert_eq!(check_brand(&profile), Outcome::Passed);

    profile.browser.branded = false;
    let outcome = check_brand(&profile);
    let Outcome::Failed(findings) = outcome else {
        panic!("a branded build claiming to be unbranded was accepted: {outcome:?}");
    };
    assert!(
        findings[0]
            .message
            .contains("carries a Google Chrome entry"),
        "{}",
        findings[0].message
    );
}

/// Rewrite the navigate variant's recorded header values.
///
/// ⚠ **The names keep their wire order**, because that order is a fingerprint
/// in its own right and a test that reordered them would be changing two things
/// at once.
fn set_navigate_headers(profile: &mut b_ids_schema::Profile, raw: &[(String, String)]) {
    for set in &mut profile.http.variants {
        if set.variant == b_ids_schema::http::Variant::Navigate {
            for header in &mut set.headers {
                if let Some((_, value)) = raw
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&header.name))
                {
                    header.value = Some(value.clone());
                }
            }
        }
    }
}
