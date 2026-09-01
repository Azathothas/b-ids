//! Acquisition: the routes, their order, and what a run reports when one is
//! down.
//!
//! ⭐ **The failure case is the point.** A pipeline with one route works right
//! up until the day the URL 404s, so what has to be tested is the day it does.
//! The fetcher is injected for exactly that reason: this suite arranges the
//! primary route's failure, which no test against a live network could.
//!
//! `TODO/driver.md`, `DRIVER-05`.

use b_ids_driver::acquire::{Candidate, Route, acquire_with, plan};
use b_ids_driver::resolve::Family;

/// The bytes a fake route hands back, and the digest they are known to have.
const PAYLOAD: &[u8] = b"not a browser, and never published as one";

fn refuses(_c: &Candidate) -> Result<Vec<u8>, String> {
    Err("arranged failure".to_owned())
}

/// The digest the caller supplies. ⚠ It is the harness's, which is a DEV
/// dependency here for the reason the driver's manifest gives: a driver that
/// imported the harness would be one component with two jobs.
fn digest(bytes: &[u8]) -> String {
    b_ids_harness::hex(&b_ids_harness::sha256(bytes))
}

#[test]
fn acquisition_plans_the_installed_route_first_and_the_index_last() {
    let routes: Vec<Route> = plan(Family::Chrome, Some("151.0.7922.76"))
        .into_iter()
        .map(|c| c.route)
        .collect();
    assert_eq!(
        routes,
        vec![Route::Installed, Route::Cache, Route::ChromeForTesting],
        "the order is the design: a machine that already has the build cannot fail for a \
         network reason"
    );
}

#[test]
fn acquisition_leaves_out_the_exact_build_route_when_no_build_was_named() {
    // ⛔ The automation index is keyed by build. A plan with no version says so
    // by leaving the route out, never by offering a URL that cannot answer.
    let routes: Vec<Route> = plan(Family::Chrome, None)
        .into_iter()
        .map(|c| c.route)
        .collect();
    assert!(!routes.contains(&Route::ChromeForTesting), "{routes:?}");
}

#[test]
fn acquisition_falls_back_when_the_primary_route_is_down() {
    let candidates = plan(Family::Chrome, Some("151.0.7922.76"));
    let mut seen = Vec::new();
    let acquired = acquire_with(
        &candidates,
        |candidate| {
            seen.push(candidate.route);
            // ⚠ Everything before the index refuses, which is the arranged outage.
            if candidate.route == Route::ChromeForTesting {
                Ok(PAYLOAD.to_vec())
            } else {
                Err(format!("{} is arranged down", candidate.route))
            }
        },
        digest,
    )
    .expect("the last route answered");

    assert_eq!(acquired.route, Route::ChromeForTesting);
    assert_eq!(acquired.bytes, PAYLOAD.len());
    assert_eq!(
        acquired.refusals.len(),
        2,
        "every route tried before the one that answered is reported: {:?}",
        acquired.refusals
    );
    assert_eq!(
        seen,
        vec![Route::Installed, Route::Cache, Route::ChromeForTesting],
        "the routes are tried in the planned order"
    );
    // ⭐ The digest is what makes the acquisition reproducible after the
    // artefact stops being served, so it is asserted rather than assumed
    // present.
    assert_eq!(acquired.sha256, digest(PAYLOAD));
}

#[test]
fn acquisition_reports_every_refusal_when_no_route_answers() {
    let candidates = plan(Family::Chrome, Some("151.0.7922.76"));
    let refusals = acquire_with(&candidates, refuses, digest).expect_err("no route answered");
    assert_eq!(refusals.len(), candidates.len());
    // ⛔ Not the last refusal alone. A caller shown only the final failure
    // cannot tell an outage from a build that does not exist.
    let routes: Vec<Route> = refusals.iter().map(|r| r.route).collect();
    assert_eq!(
        routes,
        vec![Route::Installed, Route::Cache, Route::ChromeForTesting]
    );
}

#[test]
fn acquisition_treats_an_empty_answer_as_a_refusal() {
    // ⚠ A route that answers with nothing has not answered. Accepting it would
    // record a digest of the empty string as the digest of a browser.
    let candidates = vec![
        Candidate {
            route: Route::Cache,
            url: None,
        },
        Candidate {
            route: Route::ChromeForTesting,
            url: Some("https://example.invalid/build".to_owned()),
        },
    ];
    let acquired = acquire_with(
        &candidates,
        |candidate| {
            if candidate.route == Route::Cache {
                Ok(Vec::new())
            } else {
                Ok(PAYLOAD.to_vec())
            }
        },
        digest,
    )
    .expect("the second route answered");
    assert_eq!(acquired.route, Route::ChromeForTesting);
    assert_eq!(acquired.refusals.len(), 1);
    assert!(acquired.refusals[0].why.contains("no bytes"));
}
