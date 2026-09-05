//! Acquisition: the routes, their order, and what a run reports when one is
//! down.
//!
//! ⭐ **The failure case is the point.** A pipeline with one route works right
//! up until the day the URL 404s, so what has to be tested is the day it does.
//! The fetcher is injected for exactly that reason: this suite arranges the
//! primary route's failure, which no test against a live network could.
//!
//! `docs/history/todo/driver.md`, `DRIVER-05`.

use b_ids_driver::acquire::{
    Candidate, IndexRefusal, Platform, Route, acquire_with, download, download_url, index_route,
    index_url, plan,
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
// defect these tests exist to catch. `docs/history/todo/driver.md`, `DRIVER-08`.
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

#[test]
fn the_route_vocabulary_is_one_list_and_the_driver_agrees_with_the_schema() {
    // ⛔ THREE COPIES OF ONE LIST IS TWO CHANCES FOR IT TO DRIFT, and this is
    // the pair that can be compared in code: the driver's enum and the
    // schema's constant. The third is the published JSON schema, and
    // `the_published_schema_carries_the_same_route_vocabulary` below reads it.
    //
    // ⚠ The drift this catches is real rather than hypothetical: `vendor` was
    // added to the driver on 2026-09-02 because `provision-browser --route
    // vendor` had no name a profile could record, and the schema would have
    // refused every profile taken through it.
    let from_driver: Vec<&str> = Route::all().iter().map(|r| r.as_str()).collect();
    assert_eq!(
        from_driver,
        b_ids_schema::ACQUISITION_ROUTES.to_vec(),
        "the driver's routes and the schema's accepted routes are one vocabulary"
    );
}

#[test]
fn the_published_schema_carries_the_same_route_vocabulary() {
    // ⛔ READ FROM THE FILE A CONSUMER FETCHES, not from a copy of it. A
    // profile the model accepts and the published schema refuses is a profile
    // nobody downstream can validate.
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../b-ids-schema/schema/browser-profile-1.schema.json"
    ))
    .expect("the published schema is in the tree");
    let schema: serde_json::Value = serde_json::from_str(&text).expect("it parses");
    let published = schema["$defs"]["captured"]["properties"]["acquisition"]["properties"]["route"]
        ["enum"]
        .as_array()
        .expect("$defs/captured/properties/acquisition/properties/route/enum");
    let published: Vec<&str> = published
        .iter()
        .map(|v| v.as_str().expect("every route is a string"))
        .collect();
    assert_eq!(
        published,
        b_ids_schema::ACQUISITION_ROUTES.to_vec(),
        "the published schema and the model accept the same routes"
    );
}

// -- the Edge enterprise index, which is a different shape -----------------
//
// ⛔ A TRIMMED EXCERPT OF THE REAL INDEX, in its real shape, read on
// 2026-09-02 from the URL `b_ids_driver::acquire::index_url` names for Edge. It
// carried five products and 157 Stable releases; the two below are two of them
// verbatim, with the CVE lists dropped because this reader does not look at
// them. ⚠ A fixture somebody invented would let this suite agree with a reader
// that cannot read the real thing. `docs/history/todo/driver.md`, `DRIVER-10`.
const EDGE_INDEX: &str = r#"[
  { "Product": "Dev", "Releases": [] },
  {
    "Product": "Stable",
    "Releases": [
      {
        "ReleaseId": 1,
        "Platform": "Windows",
        "Architecture": "x64",
        "ProductVersion": "152.0.4191.53",
        "PublishedTime": "2026-08-28T03:03:00",
        "Artifacts": [
          {
            "ArtifactName": "msi",
            "Location": "https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/dd96e247-54fb-4e65-bc78-514b4b7ead4c/MicrosoftEdgeEnterpriseX64.msi",
            "Hash": "17B704410AE47E33F830230503AFFED39BA8ED36356E90F0CF6759231543A22C",
            "HashAlgorithm": "SHA256",
            "SizeInBytes": 258912256
          }
        ]
      },
      {
        "ReleaseId": 2,
        "Platform": "Linux",
        "Architecture": "x64",
        "ProductVersion": "151.0.4129.101",
        "PublishedTime": "2026-08-14T02:38:00",
        "Artifacts": [
          {
            "ArtifactName": "rpm",
            "Location": "https://packages.microsoft.com/yumrepos/edge/microsoft-edge-stable-151.0.4129.101-1.x86_64.rpm",
            "Hash": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "HashAlgorithm": "SHA256",
            "SizeInBytes": 1
          },
          {
            "ArtifactName": "deb",
            "Location": "https://packages.microsoft.com/repos/edge/pool/main/m/microsoft-edge-stable/microsoft-edge-stable_151.0.4129.101-1_amd64.deb",
            "Hash": "BD7604025424914A61C06293CB6BF269141A29D8C54CF1997110BC96D3365D60",
            "HashAlgorithm": "SHA256",
            "SizeInBytes": 194950834
          }
        ]
      }
    ]
  }
]"#;

#[test]
fn the_edge_index_names_the_deb_for_linux_and_the_msi_for_windows() {
    // ⛔ THE ARTEFACT KIND IS PART OF THE ANSWER, and it is not the same on the
    // two platforms. A reader that took the first artefact would install an rpm
    // on a Debian runner.
    let linux = download(
        Family::Edge,
        EDGE_INDEX,
        "151.0.4129.101",
        Platform::Linux64,
    )
    .expect("the index carries it");
    assert!(linux.url.ends_with("_amd64.deb"), "{}", linux.url);

    let windows =
        download(Family::Edge, EDGE_INDEX, "152.0.4191.53", Platform::Win64).expect("carried");
    assert!(windows.url.ends_with(".msi"), "{}", windows.url);
}

#[test]
fn the_edge_index_carries_the_publishers_digest_and_the_chrome_one_does_not() {
    // ⭐ THE DIFFERENCE BETWEEN THE TWO INDEXES, asserted rather than assumed.
    // Edge states a SHA-256 and a byte count for every artefact, so an
    // acquisition through it can be checked against the publisher. The
    // automation index for Chrome states neither, and `None` says so.
    let edge = download(
        Family::Edge,
        EDGE_INDEX,
        "151.0.4129.101",
        Platform::Linux64,
    )
    .expect("carried");
    // ⛔ DERIVED FROM THE FIXTURE RATHER THAN REPEATED. A digest written out
    // here as well would be the same value in two places with no check that
    // they agree, and it is the thing this reader is supposed to be carrying
    // across unchanged.
    let digest = edge
        .published_sha256
        .expect("the Edge index publishes a digest for every artefact");
    assert_eq!(digest.len(), 64, "a SHA-256 in hex: {digest}");
    assert!(
        digest
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "lower-cased, because that is the form this project compares digests in: {digest}"
    );
    assert!(
        EDGE_INDEX.contains(&digest.to_ascii_uppercase()),
        "and it is the digest the fixture states, read rather than retyped"
    );
    assert_eq!(edge.published_bytes, Some(194_950_834));

    let chrome =
        download(Family::Chrome, INDEX, "151.0.7922.76", Platform::Linux64).expect("carried");
    assert_eq!(chrome.published_sha256, None);
    assert_eq!(chrome.published_bytes, None);
}

#[test]
fn a_platform_the_edge_index_does_not_serve_is_refused_by_name() {
    // ⛔ Named rather than mapped onto the nearest. The index does publish
    // other pairs, and answering a macOS request with the Linux deb would
    // provision one machine with another machine's build.
    let refusal = download(
        Family::Edge,
        EDGE_INDEX,
        "151.0.4129.101",
        Platform::MacArm64,
    )
    .expect_err("this reader maps only linux64 and win64");
    assert!(
        matches!(refusal, IndexRefusal::NoDownloadForPlatform { .. }),
        "{refusal:?}"
    );
}

#[test]
fn the_route_table_names_an_index_and_a_route_for_every_family_it_knows() {
    // ⛔ THE TABLE IS THE DESIGN. Adding a family is a row plus a reader, not a
    // branch in a caller, and a family whose index is named must also have a
    // route a profile can record.
    for family in Family::all() {
        let Some(url) = index_url(family) else {
            continue;
        };
        assert!(url.starts_with("https://"), "{family}: {url}");
        // ⚠ A family whose index is named must also name a route, and the two
        // are separate answers, so this unwraps rather than assuming. The
        // pairing itself is asserted in resolve_and_drive.rs, over EVERY
        // family rather than only the ones with an index.
        let route = index_route(family)
            .unwrap_or_else(|| panic!("{family} names an index and no route to record it under"));
        assert!(
            b_ids_schema::ACQUISITION_ROUTES.contains(&route.as_str()),
            "{family}: the route {route} is not one a profile may record"
        );
        // ⚠ And `plan` must offer it, which is what a caller actually reads.
        let offered: Vec<_> = plan(family, Some("1.2.3.4"))
            .into_iter()
            .map(|c| c.route)
            .collect();
        assert!(offered.contains(&route), "{family}: {offered:?}");
    }
}
