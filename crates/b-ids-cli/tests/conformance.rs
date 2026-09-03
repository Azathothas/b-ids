//! `LIB-02`. Point the smallest client at the harness and compare, field by
//! field, what it sent against what it claimed.
//!
//! ⛔ **The listener runs IN PROCESS.** A test that spawned the harness binary
//! and parsed its stdout would be testing the orchestration, and the first
//! thing to go wrong would be the orchestration.
//!
//! ⚠ **This is the only pass that says the corpus is USABLE rather than merely
//! accurate**, which is why the entry is worth more than its priority suggests.

use std::net::SocketAddr;
use std::time::Duration;

use b_ids_cli::{NotSent, differences, send};
use b_ids_harness::listener::{Config, Oracle, Protocol};

/// A listener on a port the operating system chooses, reading one connection.
fn harness() -> (Oracle, SocketAddr) {
    let config = Config {
        protocol: Protocol::TlsRaw,
        port: 0,
        handshakes: 1,
        run_timeout: Some(Duration::from_secs(20)),
        read_timeout: Duration::from_secs(5),
        ..Config::default()
    };
    let listener = Oracle::bind(config).expect("a listener on a free port");
    let addr = listener.local_addr().expect("its address");
    (listener, addr)
}

/// Send one profile at a fresh listener and give back what arrived.
fn round_trip() -> b_ids_harness::listener::Capture {
    let (listener, addr) = harness();
    let sender = std::thread::spawn(move || send(id_for_thread(), addr));
    let captures = listener.run().expect("the listener ran");
    let sent = sender
        .join()
        .expect("the client thread")
        .expect("the client sent");
    assert!(sent.bytes > 100, "a hello of {} bytes", sent.bytes);
    assert_eq!(captures.len(), 1, "one connection was expected");
    captures.into_iter().next().expect("the one capture")
}

/// The profile every case here uses.
///
/// ⚠ **Read from the embedded corpus rather than typed**, because a test that
/// named a build would go stale the first time the corpus grew.
fn id_for_thread() -> &'static str {
    static ID: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    ID.get_or_init(|| {
        b_ids::latest_stable("chrome", "linux64")
            .expect("the corpus publishes one")
            .id
            .to_string()
    })
}

#[test]
fn the_harness_reads_back_the_profile_the_client_claimed() {
    // ⛔ THE ACCEPTANCE. Field by field, never a digest comparison: two profiles
    // can share a digest and differ in a field the digest sorts away.
    let capture = round_trip();
    let sent = capture
        .tls
        .as_ref()
        .expect("the harness read a ClientHello");
    let claimed = &b_ids::profiles()
        .iter()
        .find(|p| p.id.to_string() == id_for_thread())
        .expect("the profile")
        .tls;

    let differing = differences(claimed, sent);
    assert!(
        differing.is_empty(),
        "the client sent a hello that differs from the profile it claimed:\n  {}",
        differing.join("\n  ")
    );
}

#[test]
fn the_bytes_on_the_wire_are_the_captured_bytes_but_for_the_random() {
    // ⭐ THE STRONGEST FORM THE MODEL ALLOWS. The profile's own raw hello and
    // the one this client sent are the same length and differ in exactly the
    // thirty-two bytes the model does not record.
    let capture = round_trip();
    let profile = b_ids::profiles()
        .iter()
        .find(|p| p.id.to_string() == id_for_thread())
        .expect("the profile");
    let original = profile
        .raw
        .client_hello_hex
        .as_deref()
        .expect("the profile publishes its bytes");
    let arrived = &capture.raw_hex;
    assert_eq!(
        original.len(),
        arrived.len(),
        "the hello this client sent is a different length from the one captured"
    );

    // The random is the 32 bytes after the 5-byte record header, the 4-byte
    // handshake header and the 2-byte legacy version: bytes 11..43.
    let differing: Vec<usize> = original
        .as_bytes()
        .iter()
        .zip(arrived.as_bytes())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i / 2)
        .collect();
    let unique: std::collections::BTreeSet<usize> = differing.into_iter().collect();
    assert!(
        unique.iter().all(|&b| (11..43).contains(&b)),
        "bytes outside the random differ: {unique:?}"
    );
    assert!(
        !unique.is_empty(),
        "not one byte of the random differs, so this client is sending a constant"
    );
}

#[test]
fn a_profile_the_corpus_does_not_hold_is_refused_by_name() {
    // ⛔ NEVER A SUBSTITUTE. The same rule as the published routes: a missing
    // profile is a fact and a neighbouring one is a lie.
    let (_listener, addr) = harness();
    let refused = send("netscape-4.7-os2-stable", addr).expect_err("no such profile");
    assert_eq!(
        refused,
        NotSent::NoSuchProfile {
            wanted: "netscape-4.7-os2-stable".to_owned()
        }
    );
    assert!(refused.to_string().contains("never substitutes"));
}

#[test]
fn the_comparison_reports_a_field_that_moved() {
    // ⛔ A COMPARISON NOBODY HAS SEEN REFUSE IS THEATRE. One extension is moved
    // and the report has to name the field it moved in.
    let profile = b_ids::latest_stable("chrome", "linux64").expect("a profile");
    let mut moved = profile.tls.clone();
    moved.extensions.swap(0, 1);
    let differing = differences(&profile.tls, &moved);
    assert!(
        differing
            .iter()
            .any(|d| d.starts_with("tls.extensions.order")),
        "{differing:?}"
    );

    // ⚠ And a cipher list that lost one entry is a different field again, so
    // the report says which rather than that something changed.
    let mut shorter = profile.tls.clone();
    shorter.cipher_suites.pop();
    let differing = differences(&profile.tls, &shorter);
    assert!(
        differing.iter().any(|d| d.starts_with("tls.cipher_suites")),
        "{differing:?}"
    );
}
