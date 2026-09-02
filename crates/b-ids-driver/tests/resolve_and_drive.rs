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
