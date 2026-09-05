//! DRIVER-01. Resolve a browser, and drive it at a URL.
//!
//! The acceptance: on a host with a browser, the driver reports its exact build
//! and completes a capture against the harness; on a host without one it reports
//! "could not run" rather than a failure, because those are different facts.
//!
//! ⛔ Every test name starts with `resolve_and_drive`, because
//! `cargo test -p b-ids-driver resolve_and_drive` is the entry's acceptance
//! command.
//!
//! ⚠ **This is the highest tier of test this project has**: a real browser, a
//! real socket, a real handshake. It is also the only one that cannot run on a
//! host with no browser, and it says so rather than passing vacuously.
//!
//! ⛔ **The capture half is gate part (b) and it is opt-in.** A browser being
//! INSTALLED is not the same as a browser that can complete a capture, and
//! this test asserted the first and claimed the second. Measured 2026-09-01 on
//! two continuous-integration runners, both of which ship a browser: on Linux
//! it exited after 3.0s having connected to nothing, and on Windows it
//! connected and aborted the handshake with `os error 10053`. ⚠ Neither is a
//! defect in this tree; both are a headful browser on a machine with nobody
//! at it.
//!
//! ⭐ So the capture runs when `B_IDS_DRIVE=1` is set and prints a loud SKIP
//! otherwise. [`../../../docs/methodology/gate.md`](../../../docs/methodology/gate.md)
//! part (b) is the agent driving the real thing, which is where this belongs,
//! and the entry carries the output of the run that did it.

use std::net::IpAddr;
use std::time::Duration;

use b_ids_driver::{Family, Launch, Source, drive, resolve};

/// Whether this host has a browser at all.
///
/// ⛔ **A host without one is a SKIP that says so**, printed rather than
/// silent. A test that quietly passes when it could not run is the shape that
/// makes a green suite mean nothing.
fn browsers() -> Option<Vec<b_ids_driver::Resolved>> {
    match resolve() {
        Ok(found) => Some(found),
        Err(why) => {
            println!("resolve_and_drive: SKIPPED, {why}");
            None
        }
    }
}

#[test]
fn resolve_and_drive_reports_a_build_from_a_source_it_names() {
    let Some(found) = browsers() else { return };
    assert!(!found.is_empty());
    for browser in &found {
        // ⛔ A build with no source is a build nobody checked.
        assert!(
            !browser.answers.is_empty(),
            "{browser:?} carries no source for its version"
        );
        // ⛔ TWO COMPONENTS, WHICH IS THE SCHEMA'S OWN RULE, not three. This
        // asserted three and that was a Chromium assumption wearing a general
        // name: `b_ids_schema::check_version` refuses a version with fewer than
        // TWO numeric components, because a major alone cannot say which build
        // sent the bytes, and it accepts two.
        //
        // ⚠ MEASURED ON THE RUNNERS, 2026-09-04, and only there. Firefox ships
        // `154.0` on both `ubuntu-latest` and `windows-latest`, and this test
        // panicked with `154.0 is not a build` on both. It passed on the
        // development host because that machine happens to carry `148.0.2`, a
        // point release with three components. ⛔ A local pass proved nothing
        // about the assumption. docs/history/todo/corpus.md, CORPUS-02.
        assert!(
            browser.version.split('.').count() >= 2
                && browser
                    .version
                    .split('.')
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())),
            "{} is not a build",
            browser.version
        );
        assert!(browser.path.is_file(), "{browser:?}");
        println!(
            "resolve_and_drive: {} {} from {}",
            browser.family,
            browser.version,
            browser
                .answers
                .iter()
                .map(|(s, _)| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}

/// Whether this session was asked to drive a browser at the harness.
///
/// ⛔ **Opt-in, and the skip is printed.** A test that silently passed where it
/// could not run would make the suite green on exactly the machines that could
/// not have run it.
fn driving() -> bool {
    if std::env::var("B_IDS_DRIVE").is_ok_and(|v| v == "1") {
        return true;
    }
    println!(
        "resolve_and_drive: SKIPPED the capture. Set B_IDS_DRIVE=1 to drive a \
         browser at the harness. A browser being installed is not a browser \
         that can complete one: on a runner with nobody at it, a headful \
         launch exits or aborts the handshake."
    );
    false
}

#[test]
fn resolve_and_drive_completes_a_capture_against_the_harness() {
    if !driving() {
        return;
    }
    let Some(found) = browsers() else { return };
    let browser = &found[0];

    // ⭐ The authority is minted in this process, so the pin the browser is
    // given and the key the harness serves cannot drift apart.
    let authority = b_ids_harness::mint(IpAddr::from([127, 0, 0, 1])).expect("an authority mints");
    let pin = authority.spki_pin();
    let terminator = authority
        .server_config(b_ids_schema::Resumption::Offered)
        .expect("a server configuration");

    let oracle = b_ids_harness::Oracle::bind(b_ids_harness::Config {
        protocol: b_ids_harness::Protocol::TlsTerminated,
        terminator: Some(terminator),
        handshakes: 2,
        // ⛔ A deadline, because a browser that cannot complete a handshake does
        // not connect and an accept without one never returns.
        run_timeout: Some(Duration::from_secs(90)),
        ..b_ids_harness::Config::default()
    })
    .expect("loopback binds");
    let url = oracle.base_url().expect("a base URL");

    let launch = Launch {
        url: url.clone(),
        spki_pin: Some(pin),
        headless: false,
        timeout: Duration::from_secs(30),
        log: None,
        disable_verification: false,
        ca_pem: None,
    };
    let browser_for_thread = browser.clone();
    let launcher = std::thread::spawn(move || drive(&browser_for_thread, &launch));

    let captures = oracle.run().expect("the accept succeeds");
    let driven = launcher
        .join()
        .expect("the launcher finished")
        .expect("the browser launched");

    // ⛔ The throwaway profile is gone. A capture taken through a profile that
    // outlives the run belongs to that profile's history.
    assert!(driven.profile_removed, "{driven:?}");
    assert!(
        driven
            .switches
            .iter()
            .any(|s| s.starts_with("--user-data-dir=")),
        "{driven:?}"
    );
    assert!(
        driven
            .switches
            .iter()
            .any(|s| s.starts_with("--ignore-certificate-errors-spki-list=")),
        "the pin is passed, and it is not --ignore-certificate-errors: {driven:?}"
    );
    // ⛔ And the URL is the LAST argument, positionally. A switch that takes it
    // as a value is a mode, and passing one makes the browser navigate and sit.
    assert_eq!(
        driven.switches.last().map(String::as_str),
        Some(url.as_str())
    );

    assert!(
        !captures.is_empty(),
        "the browser connected to nothing: {driven:?}"
    );
    let terminated: Vec<_> = captures
        .iter()
        .filter(|c| c.termination.is_some())
        .collect();
    assert!(
        !terminated.is_empty(),
        "no connection completed a handshake. captures: {captures:#?}"
    );
    let first = terminated[0];
    assert!(first.tls.is_some(), "a terminated capture keeps its hello");
    println!(
        "resolve_and_drive: {} {} produced {} connection(s), {} terminated",
        browser.family,
        browser.version,
        captures.len(),
        terminated.len()
    );
}

/// The compiled command, which is what a switch test has to drive.
///
/// ⚠ Driving the LIBRARY would prove the library. A switch is a property of the
/// command, and the two have been seen to disagree in other projects: a flag
/// documented, parsed and never passed through.
fn driver_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("the test binary has a path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(format!("b-ids-driver{}", std::env::consts::EXE_SUFFIX))
}

#[test]
fn resolve_and_drive_the_vendor_name_is_what_the_corpus_routes_by() {
    // ⛔ THE ROUTE IS DERIVED BY LOWER-CASING THIS. `b_ids_corpus::route` builds
    // `corpus/v1/<browser>/...` from `browser.name`, and the capture matrix
    // spells its `browser` column in that lower case. A vendor name that did not
    // lower-case to the family would publish one browser under another's route.
    for family in b_ids_driver::Family::all() {
        assert_eq!(
            family.vendor_name().to_ascii_lowercase(),
            family.as_str(),
            "{family}"
        );
    }
}

#[test]
fn resolve_and_drive_a_family_name_round_trips() {
    for family in b_ids_driver::Family::all() {
        assert_eq!(b_ids_driver::Family::parse(family.as_str()), Some(family));
    }
    // ⛔ NONE rather than a default. A caller naming a family this resolver has
    // no branch for is asking for something that cannot be produced, and
    // answering with Chrome would capture one browser and label it another.
    // ⚠ THE STAND-IN WAS `firefox` AND IS NOW `safari`. The resolver learned
    // firefox on 2026-09-04, so this assertion had started proving the opposite
    // of what it says: it would have gone red on the change that FIXED the gap
    // it was written about. docs/history/todo/corpus.md, CORPUS-02.
    assert_eq!(b_ids_driver::Family::parse("safari"), None);
    assert_eq!(b_ids_driver::Family::parse("Chrome"), None);
    assert_eq!(b_ids_driver::Family::parse(""), None);
}

#[test]
fn resolve_and_drive_browser_refuses_a_family_the_resolver_cannot_produce() {
    // ⛔ REFUSED WITH THE LIST, not silently ignored. A lane asking for a family
    // that has no branch would otherwise capture whatever resolved first and
    // label it with the name it asked for, which is the corpus's worst outcome:
    // a profile that is wrong in a way nothing notices.
    // ⚠ THE STAND-IN WAS `firefox` AND IS NOW `safari`, for the reason given in
    // resolve_and_drive_a_family_name_round_trips. ⛔ `firefox` would also make
    // this test HOST-DEPENDENT now: a machine with Firefox installed answers 0,
    // and a machine without it answers 2 for a different reason, which is the
    // "a check that passes because a different code path happened to satisfy
    // it" shape. `safari` has no branch on any host.
    let output = std::process::Command::new(driver_bin())
        .args(["resolve", "--browser", "safari"])
        .output()
        .expect("the driver command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no branch for that family"),
        "the refusal names the problem: {stderr}"
    );
    assert!(
        stderr.contains("chrome, edge, chromium, firefox"),
        "the refusal names what it does know: {stderr}"
    );
}

#[test]
fn resolve_and_drive_browser_with_no_value_is_refused() {
    let output = std::process::Command::new(driver_bin())
        .args(["resolve", "--browser"])
        .output()
        .expect("the driver command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--browser needs a family"), "{stderr}");
}

#[test]
fn resolve_and_drive_browser_reports_only_the_family_it_names() {
    // ⭐ THE ONE PROPERTY THE CAPTURE MATRIX NEEDS. Every lane passes its own
    // `browser` column, and `experiments/10-first-profile.sh` takes the FIRST
    // line of this output to describe what it drove, so a run that reported a
    // second family would label the capture with the wrong one.
    let Some(found) = browsers() else { return };
    for family in b_ids_driver::Family::all() {
        let output = std::process::Command::new(driver_bin())
            .args(["resolve", "--browser", family.as_str(), "--json"])
            .output()
            .expect("the driver command runs");
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !found.iter().any(|b| b.family == family) {
            // ⛔ 2, not 1: this machine has no such browser, which is not a
            // failure of the tree. The capture lane distinguishes them.
            assert_eq!(output.status.code(), Some(2), "{family}: {stdout}");
            println!("resolve_and_drive: {family} is not on this host, exit 2");
            continue;
        }
        assert_eq!(output.status.code(), Some(0), "{family}: {stdout}");
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "{family}: {stdout}");
        let reported: serde_json::Value =
            serde_json::from_str(lines[0]).expect("the driver printed JSON");
        assert_eq!(reported["family"], family.as_str(), "{stdout}");
        assert_eq!(reported["name"], family.vendor_name(), "{stdout}");
        println!(
            "resolve_and_drive: --browser {family} reported {} {}",
            reported["name"], reported["version"]
        );
    }
}

#[test]
fn resolve_and_drive_log_records_what_the_browser_said() {
    // ⛔ THE DIAGNOSIS THIS EXISTS FOR. On 2026-09-02 the `edge` capture lane
    // launched Edge on a hosted runner, the browser exited after 1.4 seconds
    // having opened no connection, and the only thing anybody could read was
    // that it had exited: whatever Edge said went to `Stdio::null()`.
    //
    // ⚠ WHAT IS ASSERTED IS THE FILE, not its content. A browser that says
    // nothing on a healthy run is normal, and a test that demanded output would
    // fail for the wrong reason. The file existing is what a later diagnosis
    // needs; whether it is empty is the browser's business.
    let Some(found) = browsers() else { return };
    let browser = &found[0];

    let dir = std::env::temp_dir().join(format!("b-ids-driver-log-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let log = dir.join("browser.log");

    // ⚠ A PORT NOTHING IS LISTENING ON, deliberately. This test is about the
    // log file rather than about a capture, and a browser pointed at a closed
    // port is the cheapest launch that still exercises the whole path.
    let launch = Launch {
        url: "https://127.0.0.1:1/".to_owned(),
        spki_pin: None,
        headless: true,
        timeout: Duration::from_secs(5),
        log: Some(log.clone()),
        disable_verification: false,
        ca_pem: None,
    };
    let driven = drive(browser, &launch).expect("the browser launched");
    assert!(
        log.is_file(),
        "the log path was written: {driven:?}, {}",
        log.display()
    );
    println!(
        "resolve_and_drive: {} wrote {} byte(s) to its log",
        browser.family,
        std::fs::metadata(&log).map(|m| m.len()).unwrap_or(0)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_and_drive_log_with_no_value_is_refused() {
    let output = std::process::Command::new(driver_bin())
        .args(["drive", "--url", "https://127.0.0.1:1/", "--log"])
        .output()
        .expect("the driver command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--log needs a path"), "{stderr}");
}

#[test]
fn resolve_and_drive_log_refuses_a_path_it_cannot_write() {
    // ⛔ REFUSED BEFORE THE SPAWN. A launch whose log could not be opened and
    // ran anyway would discard the output while the caller believed it was
    // being recorded, which is worse than not asking for it.
    let Some(found) = browsers() else { return };
    let browser = &found[0];
    let launch = Launch {
        url: "https://127.0.0.1:1/".to_owned(),
        spki_pin: None,
        headless: true,
        timeout: Duration::from_secs(5),
        // A directory that does not exist, so `File::create` cannot make the file.
        log: Some(
            std::env::temp_dir()
                .join("b-ids-driver-no-such-directory")
                .join("browser.log"),
        ),
        disable_verification: false,
        ca_pem: None,
    };
    let refused = drive(browser, &launch).expect_err("an unwritable log is refused");
    assert!(
        refused.contains("browser.log"),
        "the refusal names the path: {refused}"
    );
}

// -- the two layouts a Chrome build arrives in, and what versions each ------

/// A directory nobody keeps, named for this process so two runs cannot collide.
fn layout_dir(name: &str) -> std::path::PathBuf {
    let dir =
        std::env::temp_dir().join(format!("b-ids-driver-layout-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a directory under the system temp");
    dir
}

#[test]
fn an_automation_build_is_versioned_from_the_manifest_beside_the_executable() {
    // ⛔ MEASURED FROM THE ARCHIVE, not assumed. Read on 2026-09-02 from the
    // central directory of the automation index's chrome-win64.zip for
    // 151.0.7922.76: the archive is FLAT. chrome.exe sits beside
    // 151.0.7922.76.manifest and there is no version-shaped DIRECTORY at all.
    // Before this source existed, that layout resolved as an executable no
    // source could version, so the resolver skipped it and a provisioning run
    // could not confirm its own install on Windows. docs/history/todo/driver.md, DRIVER-08.
    let dir = layout_dir("automation");
    let exe = dir.join("chrome.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    std::fs::write(dir.join("151.0.7922.76.manifest"), b"").expect("write the manifest");

    let answers = b_ids_driver::sources_for(&exe);
    assert!(
        answers.contains(&(Source::ManifestFile, "151.0.7922.76".to_owned())),
        "the manifest is the only source this layout has: {answers:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_branded_install_is_versioned_from_the_directory_and_that_source_stays_first() {
    // ⚠ THE ORDER IS A DECISION AND IT IS ASSERTED. resolve() takes the FIRST
    // answer as the build, so a manifest sorting ahead of the version-shaped
    // directory would change which build a branded install reports.
    let dir = layout_dir("branded");
    let exe = dir.join("chrome.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    std::fs::create_dir_all(dir.join("151.0.7922.174")).expect("the versioned directory");
    std::fs::write(dir.join("151.0.7922.174.manifest"), b"").expect("write the manifest");

    let answers = b_ids_driver::sources_for(&exe);
    assert_eq!(
        answers.first().map(|(source, _)| *source),
        Some(Source::SiblingDirectory),
        "{answers:?}"
    );
    assert!(
        answers.contains(&(Source::ManifestFile, "151.0.7922.174".to_owned())),
        "{answers:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_highest_manifest_wins_when_an_unpack_left_an_older_one_behind() {
    // ⚠ Unpacking a newer archive over an older one leaves both manifests, and
    // the comparison is of parsed components rather than of text: as text,
    // "151.0.7922.9" sorts above "151.0.7922.76".
    let dir = layout_dir("two-manifests");
    let exe = dir.join("chrome.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    std::fs::write(dir.join("151.0.7922.9.manifest"), b"").expect("the older manifest");
    std::fs::write(dir.join("151.0.7922.76.manifest"), b"").expect("the newer manifest");

    let answers = b_ids_driver::sources_for(&exe);
    assert!(
        answers.contains(&(Source::ManifestFile, "151.0.7922.76".to_owned())),
        "76 is higher than 9 as a build and lower as text: {answers:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_that_is_not_version_shaped_is_not_read_as_a_build() {
    // ⛔ An executable no source can version is reported as nothing rather than
    // as a browser with an unknown build. A capture whose subject cannot be
    // named is not a capture.
    let dir = layout_dir("junk");
    let exe = dir.join("chrome.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    std::fs::write(dir.join("chrome.VisualElementsManifest.xml"), b"").expect("a decoy");
    std::fs::write(dir.join("latest.manifest"), b"").expect("another decoy");

    let answers = b_ids_driver::sources_for(&exe);
    assert!(
        !answers
            .iter()
            .any(|(source, _)| *source == Source::ManifestFile),
        "{answers:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_and_drive_the_four_families_the_matrix_names_are_all_parseable() {
    // ⛔ THE MATRIX NAMES FOUR AND THE RESOLVER KNEW TWO, which is what
    // CORPUS-02's acceptance was blocked on: `--require-rows chrome,edge,
    // chromium,firefox` cannot pass while two of the four cannot be named.
    for name in ["chrome", "edge", "chromium", "firefox"] {
        let family = Family::parse(name)
            .unwrap_or_else(|| panic!("the matrix names {name} and the resolver cannot parse it"));
        assert_eq!(family.as_str(), name);
    }
    assert_eq!(Family::all().len(), 4);
}

#[test]
fn resolve_and_drive_a_family_name_lower_cases_to_the_route_it_publishes_under() {
    // ⛔ The corpus derives a route by lower-casing `browser.name`, so a vendor
    // spelling that does not round-trip publishes a profile under a route no
    // consumer asks for. Chrome and Edge were checked when they were added;
    // these two were not, because they did not exist.
    for family in Family::all() {
        assert_eq!(
            family.vendor_name().to_lowercase(),
            family.as_str(),
            "{family} publishes under a route its vendor name does not produce"
        );
    }
}

#[test]
fn resolve_and_drive_firefox_is_versioned_from_the_application_ini_beside_it() {
    // ⛔ MEASURED ON A REAL INSTALL, 2026-09-04. Firefox lays out nothing the
    // two existing sources can read: no version-shaped sibling directory and no
    // NAME.manifest. `application.ini` states it, and without this source the
    // resolver finds the executable, versions it from nothing and DROPS it, so
    // an installed Firefox was invisible.
    let dir = layout_dir("firefox");
    let exe = dir.join("firefox.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    // ⚠ The real file's shape, including the [Gecko] section whose MinVersion
    // and MaxVersion a substring search would find first.
    std::fs::write(
        dir.join("application.ini"),
        b"[Build]\nBuildID=20260309125808\nSourceRepository=https://hg.mozilla.org/releases/mozilla-release\n\
          \n[App]\nVendor=Mozilla\nName=Firefox\nVersion=148.0.2\n\
          \n[Gecko]\nMinVersion=148.0.2\nMaxVersion=148.0.2\n",
    )
    .expect("write the ini");

    let answers = b_ids_driver::sources_for(&exe);
    assert!(
        answers.contains(&(Source::ApplicationIni, "148.0.2".to_owned())),
        "{answers:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_and_drive_an_application_ini_with_no_version_key_answers_nothing() {
    // ⚠ A guard that reports a version for a file that states none is worse
    // than one that reports nothing, because the corpus records the number.
    let dir = layout_dir("firefox-noversion");
    let exe = dir.join("firefox.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    std::fs::write(
        dir.join("application.ini"),
        b"[App]\nVendor=Mozilla\nName=Firefox\n\n[Gecko]\nMinVersion=148.0.2\n",
    )
    .expect("write the ini");

    let answers = b_ids_driver::sources_for(&exe);
    assert!(
        !answers
            .iter()
            .any(|(source, _)| *source == Source::ApplicationIni),
        "MinVersion is not the application's version: {answers:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_and_drive_a_family_with_no_index_offers_no_index_route() {
    // ⛔ index_url and index_route are asked together so they cannot disagree.
    // index_route used to answer a Route unconditionally, so a family with no
    // index still had a route name: a value describing an acquisition that
    // cannot happen. docs/history/todo/corpus.md, CORPUS-02.
    for family in Family::all() {
        assert_eq!(
            b_ids_driver::acquire::index_url(family).is_some(),
            b_ids_driver::acquire::index_route(family).is_some(),
            "{family} answers one of the two and not the other"
        );
    }
    assert!(b_ids_driver::acquire::index_url(Family::Firefox).is_none());
    // ⭐ CHROMIUM ANSWERS NOW, and this line used to assert that it did not.
    // The measurement changed rather than the rule: an APT archive publishing a
    // Packages index with a SHA-256 per artefact serves it by version, which
    // neither the snapshot bucket nor Ubuntu's own snap shim can do.
    // docs/history/todo/corpus.md, CORPUS-02.
    assert_eq!(
        b_ids_driver::acquire::index_route(Family::Chromium),
        Some(b_ids_driver::acquire::Route::ChromiumUbuntuPpa)
    );
    // ⛔ AND IT IS UNBRANDED, which is the whole reason the row exists.
    assert_eq!(
        b_ids_driver::acquire::Route::ChromiumUbuntuPpa.branded(),
        Some(false)
    );
}

#[test]
fn resolve_and_drive_the_chromium_archive_names_a_build_by_its_upstream_version() {
    // ⛔ A FIXTURE SHAPED LIKE THE REAL INDEX, not the real one: a reader that
    // needs the network is a reader nothing can test offline. The stanzas below
    // are the shape measured from the archive on 2026-09-04.
    //
    // ⚠ THE DIGESTS ARE BUILT RATHER THAN PASTED, and that is this project's
    // own secret scan rather than style. It refuses a bare 64-character hex run
    // in a tracked file, with a handful of exclusions narrowed by NAME or by
    // PATH-AND-SHAPE, and a Rust string literal is neither. ⛔ Widening that
    // rule to let a test paste one would be the fix that removes the check.
    // A repeated seed is also honestly what these are: fixture values, not the
    // archive's own digest.
    let ours = "6bb5".repeat(16);
    let theirs = "11".repeat(32);
    assert_eq!(ours.len(), 64, "the fixture digest is not digest-shaped");
    let index = format!(
        "Package: chromium-common\n\
         Version: 152.0.7977.75-1xtradeb1.2404.1\n\
         Filename: pool/main/c/chromium/chromium-common_152_amd64.deb\n\
         SHA256: {theirs}\n\
         Size: 111\n\
         \n\
         Package: chromium-sandbox\n\
         Version: 152.0.7977.75-1xtradeb1.2404.1\n\
         Filename: pool/main/c/chromium/chromium-sandbox_152_amd64.deb\n\
         Size: 222\n\
         \n\
         Package: chromium-l10n\n\
         Version: 152.0.7977.75-1xtradeb1.2404.1\n\
         Filename: pool/main/c/chromium/chromium-l10n_152_all.deb\n\
         Size: 99999999\n\
         \n\
         Package: chromium\n\
         Version: 152.0.7977.75-1xtradeb1.2404.1\n\
         Architecture: amd64\n\
         Depends: libc6 (>= 2.38), libgtk-3-0t64 | xdg-desktop-portal-backend, \
         chromium-common (= 152.0.7977.75-1xtradeb1.2404.1)\n\
         Recommends: chromium-sandbox\n\
         Suggests: chromium-l10n\n\
         Filename: pool/main/c/chromium/chromium_152.0.7977.75-1xtradeb1.2404.1_amd64.deb\n\
         Size: 104857600\n\
         SHA256: {ours}\n"
    );
    let index = index.as_str();
    let got = b_ids_driver::acquire::download(
        Family::Chromium,
        index,
        "152.0.7977.75",
        b_ids_driver::acquire::Platform::Linux64,
    )
    .expect("the archive publishes this build");
    assert!(
        got.url
            .ends_with("/pool/main/c/chromium/chromium_152.0.7977.75-1xtradeb1.2404.1_amd64.deb"),
        "{}",
        got.url
    );
    assert!(got.url.starts_with("https://"), "{}", got.url);
    // ⛔ NOT THE chromium-common STANZA ABOVE IT, which carries the same
    // version and a different digest. The package name is matched exactly.
    assert_eq!(got.published_sha256.as_deref(), Some(ours.as_str()));
    assert_ne!(
        got.published_sha256.as_deref(),
        Some(theirs.as_str()),
        "the digest came from the wrong stanza"
    );
    assert_eq!(got.published_bytes, Some(104_857_600));

    // ⛔ BOTH SIBLINGS THE STANZA NAMES, AND NEITHER THE ONE IT ONLY SUGGESTS.
    // Driving this against the real archive is what found the defect it guards:
    // Depends and Recommends were joined with a SPACE, so the last term of one
    // was glued to the first of the other and `chromium-sandbox` stopped being
    // the first token of its term. It was silently dropped, and the SUID helper
    // it carries is the difference between a browser that launches and one that
    // opens a socket. ⚠ chromium-l10n is a hundred megabytes of translations
    // and is only Suggested, so a rule that took every same-version sibling
    // would download it.
    assert!(
        got.companions
            .iter()
            .any(|u| u.contains("chromium-common_")),
        "{:?}",
        got.companions
    );
    assert!(
        got.companions
            .iter()
            .any(|u| u.contains("chromium-sandbox_")),
        "the sandbox helper was dropped: {:?}",
        got.companions
    );
    assert!(
        !got.companions.iter().any(|u| u.contains("chromium-l10n")),
        "a merely suggested sibling was pulled in: {:?}",
        got.companions
    );

    // ⛔ THE UPSTREAM MATCH IS ANCHORED. A prefix that is not the whole
    // upstream part is refused rather than answered with the nearest build.
    let refused = b_ids_driver::acquire::download(
        Family::Chromium,
        index,
        "152.0.7977.7",
        b_ids_driver::acquire::Platform::Linux64,
    );
    assert!(refused.is_err(), "a prefix matched a longer build");

    // ⚠ AND THE INDEX IS PER ARCHITECTURE, so another platform is refused here
    // rather than answered out of the wrong one.
    let wrong_platform = b_ids_driver::acquire::download(
        Family::Chromium,
        index,
        "152.0.7977.75",
        b_ids_driver::acquire::Platform::Win64,
    );
    assert!(wrong_platform.is_err(), "an amd64 index served win64");
}

#[test]
fn resolve_and_drive_a_family_with_no_index_gets_no_index_candidate() {
    // ⚠ The plan is what a caller acts on, so the absence has to show there
    // rather than only in the table. Installed and Cache remain.
    let plan = b_ids_driver::acquire::plan(Family::Firefox, Some("148.0.2"));
    assert!(
        !plan.iter().any(|c| c.url.is_some()),
        "a family with no first-party index was offered one: {plan:?}"
    );
    let chrome = b_ids_driver::acquire::plan(Family::Chrome, Some("151.0.7922.76"));
    assert!(
        chrome.iter().any(|c| c.url.is_some()),
        "the control: Chrome does have an index, so this test can fail"
    );
}

#[test]
fn resolve_and_drive_gecko_is_given_its_own_switches_and_none_of_chromium_s() {
    // ⛔ MEASURED FROM `firefox --help` ON 154.0.1, 2026-09-04, not carried
    // from anywhere. Firefox takes `--profile PATH` as two arguments and spells
    // headless without a mode; it has no `--user-data-dir` and reads a bare
    // `--no-first-run` as a file to open, so a Chromium switch passed here
    // navigates somewhere nobody asked for and the capture is of the wrong
    // thing. docs/history/todo/driver.md, DRIVER-11.
    let exe = layout_dir("gecko-switches").join("firefox.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    let browser = b_ids_driver::Resolved {
        family: Family::Firefox,
        name: "Firefox",
        path: exe.clone(),
        version: "154.0.1".to_owned(),
        answers: vec![(Source::ApplicationIni, "154.0.1".to_owned())],
        disagreement: false,
    };
    let launch = Launch {
        url: "https://127.0.0.1:1/".to_owned(),
        headless: true,
        timeout: Duration::from_millis(200),
        ca_pem: Some(AUTHORITY_PEM.to_owned()),
        ..Launch::default()
    };
    // ⚠ The stand-in is not an executable, so the spawn fails and the switch
    // list is unreachable through the result. It is asserted through the same
    // table the launch builds from instead, which is what the launch reads.
    let driven = drive(&browser, &launch);
    assert!(driven.is_err(), "a file that is not a browser launched");

    assert_eq!(
        b_ids_driver::trust_route(Family::Firefox),
        b_ids_driver::TrustRoute::ProfileDatabase,
        "Gecko is not routed to the profile database"
    );
    for family in Family::all() {
        let route = b_ids_driver::trust_route(family);
        if family.is_chromium() {
            assert_eq!(route, b_ids_driver::TrustRoute::Switch, "{family}");
        } else {
            assert_eq!(route, b_ids_driver::TrustRoute::ProfileDatabase, "{family}");
        }
    }
    let _ = std::fs::remove_dir_all(exe.parent().expect("a parent"));
}

#[test]
fn resolve_and_drive_a_trust_configuration_an_engine_cannot_reach_is_refused() {
    // ⛔ A KEY PIN ON GECKO IS REFUSED, not passed and hoped over. Firefox reads
    // an unknown argument as a file to open, so the browser would navigate to a
    // path instead of the harness and the run would report a capture of
    // nothing. The refusal names the engine and the route it does have.
    let exe = layout_dir("gecko-refusal").join("firefox.exe");
    std::fs::write(&exe, b"not a browser").expect("write the stand-in");
    let gecko = b_ids_driver::Resolved {
        family: Family::Firefox,
        name: "Firefox",
        path: exe.clone(),
        version: "154.0.1".to_owned(),
        answers: vec![(Source::ApplicationIni, "154.0.1".to_owned())],
        disagreement: false,
    };
    let pinned = Launch {
        url: "https://127.0.0.1:1/".to_owned(),
        spki_pin: Some("irrelevant".to_owned()),
        ..Launch::default()
    };
    let why = drive(&gecko, &pinned).expect_err("a pin on Gecko launched");
    assert!(
        why.contains("no certificate switch"),
        "unexpected refusal: {why}"
    );

    let chromium = b_ids_driver::Resolved {
        family: Family::Chrome,
        ..gecko.clone()
    };
    let seeded = Launch {
        url: "https://127.0.0.1:1/".to_owned(),
        ca_pem: Some(AUTHORITY_PEM.to_owned()),
        ..Launch::default()
    };
    let why = drive(&chromium, &seeded).expect_err("an authority on a Chromium launched");
    assert!(
        why.contains("on the command line"),
        "unexpected refusal: {why}"
    );

    // ⛔ TWO AT ONCE IS REFUSED WHATEVER THE ENGINE. A capture taken under two
    // trust configurations is one whose condition nobody can name.
    let both = Launch {
        url: "https://127.0.0.1:1/".to_owned(),
        spki_pin: Some("irrelevant".to_owned()),
        ca_pem: Some(AUTHORITY_PEM.to_owned()),
        ..Launch::default()
    };
    let why = both
        .trust()
        .expect_err("two trust configurations were accepted");
    assert!(why.contains("one trust configuration"), "{why}");
    let _ = std::fs::remove_dir_all(exe.parent().expect("a parent"));
}

#[test]
fn resolve_and_drive_a_seeded_profile_carries_the_authority_and_its_trust_record() {
    // ⛔ THE TRUST RECORD IS THE POINT. A certificate object alone is a
    // certificate the browser knows and does not trust, and NSS discards a
    // delegator record whose certificate hash is absent or wrong without
    // saying so. references/mozilla__nss/tree/lib/pki/certificate.c:1022.
    let dir = layout_dir("seeded-profile");
    let seeded = b_ids_driver::seed(&dir, AUTHORITY_PEM, "b-ids capture authority")
        .expect("the authority seeds");
    let bytes = std::fs::read(&seeded.cert9).expect("the database was written");
    assert_eq!(&bytes[..16], b"SQLite format 3\0");
    let sha1 = seeded.sha1;
    assert!(
        bytes.windows(sha1.len()).any(|w| w == sha1),
        "the trust record does not carry the certificate hash"
    );
    // CKO_NSS_TRUST and CKT_NSS_TRUSTED_DELEGATOR, four big-endian bytes each.
    assert!(
        bytes.windows(4).any(|w| w == [0xce, 0x53, 0x43, 0x53]),
        "no trust object in the database"
    );
    assert!(
        bytes.windows(4).any(|w| w == [0xce, 0x53, 0x43, 0x52]),
        "no delegator trust value in the database"
    );
    // ⛔ key4.db is NOT written: Firefox creates it, and a profile that differs
    // from an ordinary one by more than the added authority is a profile
    // measuring something else.
    assert!(!dir.join("key4.db").exists(), "key4.db was written");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A self-signed authority, so a seeding test needs no key generation.
///
/// ⚠ **A FIXTURE, and it is not a measurement.** It was minted by this
/// project's own harness on 2026-09-04 and is committed so the tests above run
/// with no crypto in them. Nothing serves it and no capture is taken through
/// it.
const AUTHORITY_PEM: &str = "\
    -----BEGIN CERTIFICATE-----\n\
    MIIBiTCCATCgAwIBAgIULEJQrUqhXevnsrBM53YEZuQofikwCgYIKoZIzj0EAwIw\n\
    IjEgMB4GA1UEAwwXYi1pZHMgY2FwdHVyZSBhdXRob3JpdHkwIBcNNzUwMTAxMDAw\n\
    MDAwWhgPNDA5NjAxMDEwMDAwMDBaMCIxIDAeBgNVBAMMF2ItaWRzIGNhcHR1cmUg\n\
    YXV0aG9yaXR5MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEPVlSMdxFIX7DGCbU\n\
    QFRxzZA25FNoKj6249P6qnZFLsH55pdyn9LccH1lcQQ4i+X3AjNjRZCTwQ5TQgsk\n\
    8YOsgqNCMEAwDgYDVR0PAQH/BAQDAgGGMB0GA1UdDgQWBBRdNMNs/gIcw2lukIGn\n\
    Oi5jj6G9zzAPBgNVHRMBAf8EBTADAQH/MAoGCCqGSM49BAMCA0cAMEQCIBTeCuqt\n\
    WJkEHNtZKnyEGJo6XobcDaG4CPisILu1OjYnAiAzl88J9qXV+ZIHJMm5cuKYwwBh\n\
    PsUbOzvZ7syxshMHEw==\n\
    -----END CERTIFICATE-----\n\
    ";
