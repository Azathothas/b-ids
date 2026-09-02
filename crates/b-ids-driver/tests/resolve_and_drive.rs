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

use b_ids_driver::{Launch, drive, resolve};

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
        assert!(
            browser.version.split('.').count() >= 3,
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
    assert_eq!(b_ids_driver::Family::parse("firefox"), None);
    assert_eq!(b_ids_driver::Family::parse("Chrome"), None);
    assert_eq!(b_ids_driver::Family::parse(""), None);
}

#[test]
fn resolve_and_drive_browser_refuses_a_family_the_resolver_cannot_produce() {
    // ⛔ REFUSED WITH THE LIST, not silently ignored. A lane asking for a family
    // that has no branch would otherwise capture whatever resolved first and
    // label it with the name it asked for, which is the corpus's worst outcome:
    // a profile that is wrong in a way nothing notices.
    let output = std::process::Command::new(driver_bin())
        .args(["resolve", "--browser", "firefox"])
        .output()
        .expect("the driver command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no branch for that family"),
        "the refusal names the problem: {stderr}"
    );
    assert!(
        stderr.contains("chrome, edge"),
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
    };
    let refused = drive(browser, &launch).expect_err("an unwritable log is refused");
    assert!(
        refused.contains("browser.log"),
        "the refusal names the path: {refused}"
    );
}
