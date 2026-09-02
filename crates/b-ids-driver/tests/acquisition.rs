//! Acquisition: the routes, their order, and what a run reports when one is
//! down.
//!
//! ⭐ **The failure case is the point.** A pipeline with one route works right
//! up until the day the URL 404s, so what has to be tested is the day it does.
//! The fetcher is injected for exactly that reason: this suite arranges the
//! primary route's failure, which no test against a live network could.
//!
//! `TODO/driver.md`, `DRIVER-05`.

use b_ids_driver::acquire::{
    Candidate, IndexRefusal, Platform, Route, acquire_with, download_url, plan,
};
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

// -- the automation index, read rather than constructed --------------------
//
// ⛔ THE FIXTURE IS A TRIMMED EXCERPT OF THE REAL INDEX, in its real shape,
// with URLs copied from it. Read on 2026-09-02 from the index
// `b_ids_driver::acquire` names: it carried 2497 builds, and the two entries
// below are two of them verbatim. ⚠ A fixture somebody invented would let this
// suite agree with a reader that cannot read the real thing, which is the
// defect these tests exist to catch. `TODO/driver.md`, `DRIVER-08`.
const INDEX: &str = r#"{
  "timestamp": "2026-09-02T09:03:41.795Z",
  "versions": [
    {
      "version": "151.0.7922.71",
      "revision": "1654411",
      "downloads": {
        "chrome": [
          {
            "platform": "linux64",
            "url": "https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.71/linux64/chrome-linux64.zip"
          }
        ]
      }
    },
    {
      "version": "151.0.7922.76",
      "revision": "1654411",
      "downloads": {
        "chrome": [
          {
            "platform": "linux64",
            "url": "https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.76/linux64/chrome-linux64.zip"
          },
          {
            "platform": "mac-arm64",
            "url": "https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.76/mac-arm64/chrome-mac-arm64.zip"
          },
          {
            "platform": "win64",
            "url": "https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.76/win64/chrome-win64.zip"
          }
        ]
      }
    }
  ]
}"#;

#[test]
fn the_index_names_the_archive_for_one_build_on_one_platform() {
    let url = download_url(INDEX, "151.0.7922.76", Platform::Win64).expect("the index has it");
    assert_eq!(
        url,
        "https://storage.googleapis.com/chrome-for-testing-public/151.0.7922.76/win64/chrome-win64.zip"
    );
}

#[test]
fn the_index_is_read_by_name_rather_than_by_position() {
    // ⛔ The wanted build is second in the list and its wanted archive is third
    // in that build's own list, so a reader taking either by position answers
    // with a different build or a different platform and looks right.
    let url = download_url(INDEX, "151.0.7922.76", Platform::Linux64).expect("the index has it");
    assert!(url.contains("/151.0.7922.76/linux64/"), "{url}");
    let other = download_url(INDEX, "151.0.7922.71", Platform::Linux64).expect("the index has it");
    assert!(other.contains("/151.0.7922.71/linux64/"), "{other}");
}

#[test]
fn a_build_the_index_does_not_publish_is_refused_with_the_nearest_it_has() {
    // ⚠ THE COMMON CASE, and it is a fact about the vendor's catalogue rather
    // than an error here. Measured 2026-09-02: the hosted runner images served
    // Chrome 151.0.7922.173 and 151.0.7922.174, and the automation index
    // publishes neither, so provisioning to an exact build cannot reproduce
    // what the images happened to install.
    let refusal = download_url(INDEX, "151.0.7922.173", Platform::Linux64)
        .expect_err("the index does not publish it");
    match &refusal {
        IndexRefusal::NoSuchBuild {
            version,
            known,
            nearest,
        } => {
            assert_eq!(version, "151.0.7922.173");
            assert_eq!(*known, 2);
            // ⭐ The near misses, because a caller told only "no" has to fetch
            // the whole index again to find out what it could have asked for.
            assert_eq!(nearest, &["151.0.7922.71", "151.0.7922.76"]);
        }
        other => panic!("wrong refusal: {other:?}"),
    }
    let said = refusal.to_string();
    assert!(said.contains("subset"), "{said}");
}

#[test]
fn a_platform_the_build_has_no_archive_for_is_a_different_refusal() {
    // ⛔ Three facts kept apart. "Not published", "not for this platform" and
    // "the bytes did not parse" send a caller to three different places.
    let refusal = download_url(INDEX, "151.0.7922.71", Platform::Win64)
        .expect_err("that build is linux64 only in this fixture");
    match &refusal {
        IndexRefusal::NoDownloadForPlatform {
            version,
            platform,
            had,
        } => {
            assert_eq!(version, "151.0.7922.71");
            assert_eq!(*platform, Platform::Win64);
            assert_eq!(had, &["linux64"]);
        }
        other => panic!("wrong refusal: {other:?}"),
    }
}

#[test]
fn bytes_that_are_not_the_index_are_refused_rather_than_read_as_an_empty_one() {
    // ⚠ An index served as an error page is the failure that looks like a build
    // nobody published. It has to be told apart from one.
    let refusal =
        download_url("<html>502</html>", "151.0.7922.76", Platform::Linux64).expect_err("not JSON");
    assert!(
        matches!(refusal, IndexRefusal::Unparsable(_)),
        "{refusal:?}"
    );

    let no_array = download_url(r#"{"timestamp":"now"}"#, "151.0.7922.76", Platform::Linux64)
        .expect_err("JSON, and not this index");
    assert!(
        matches!(no_array, IndexRefusal::Unparsable(_)),
        "{no_array:?}"
    );
}

#[test]
fn the_platform_names_are_the_indexs_own_spellings() {
    // ⛔ Read from the index rather than chosen, and the corpus spells one of
    // them differently: the capture matrix says `macos-arm64` where the index
    // says `mac-arm64`, so a caller crossing the two translates deliberately.
    assert_eq!(Platform::MacArm64.as_str(), "mac-arm64");
    assert_eq!(Platform::parse("mac-arm64"), Some(Platform::MacArm64));
    assert_eq!(Platform::parse("macos-arm64"), None);
    assert_eq!(Platform::parse("linux64"), Some(Platform::Linux64));
}
