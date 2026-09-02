//! HARNESS-01. The oracle is a server, not a client.
//!
//! The acceptance: a committed fixture of raw bytes is fed to the listener over
//! a loopback socket and produces the capture committed beside it, byte for
//! byte, with no browser involved.
//!
//! ⛔ Every test name starts with `listener`, because
//! `cargo test -p b-ids-harness listener` is the entry's acceptance command.

mod support;

use std::io::Write as _;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use b_ids_harness::{Config, Oracle, Protocol};

use support::{feed, fixtures, one_connection};

/// The `ClientHello` fixture, which is what most of these tests feed.
fn fixture_bytes() -> Vec<u8> {
    support::fixture_bytes("client-hello.hex")
}

#[test]
fn listener_reads_the_committed_fixture_and_produces_the_committed_capture() {
    let captures = feed(
        Config {
            handshakes: 1,
            ..one_connection()
        },
        vec![fixture_bytes()],
    );
    assert_eq!(captures.len(), 1);

    let mut capture = captures.into_iter().next().expect("one capture");
    // ⚠ The peer port is chosen by the operating system and the instant is a
    // clock, so neither is part of what the golden can assert. Everything else
    // is.
    //
    // ⛔ REDACTED FROM THE COMPARISON, never from the capture. Both fields are
    // still recorded and still printed; a golden that carried either would fail
    // on every run, and a golden that fails always is a golden somebody
    // regenerates without reading.
    capture.peer = "REDACTED".to_owned();
    capture.at = "REDACTED".to_owned();

    let produced = serde_json::to_string_pretty(&capture).expect("serialises");
    let golden_path = fixtures().join("client-hello.capture.json");

    // ⭐ Regenerate with B_IDS_WRITE_GOLDEN=1, and read the diff before
    // committing it. A golden a test rewrites silently is not a golden.
    if std::env::var("B_IDS_WRITE_GOLDEN").is_ok() {
        std::fs::write(&golden_path, format!("{produced}\n")).expect("the golden is writable");
    }

    let golden = std::fs::read_to_string(&golden_path).expect("the golden is committed");
    assert_eq!(
        produced.trim(),
        golden.trim(),
        "the capture changed. Re-read the diff, then B_IDS_WRITE_GOLDEN=1 cargo test"
    );
}

#[test]
fn listener_keeps_the_raw_bytes_whatever_else_happens() {
    // ⛔ The one artefact that survives every hashing scheme and every parser
    // defect. A capture is a moment that cannot be retaken.
    let bytes = fixture_bytes();
    let captures = feed(one_connection(), vec![bytes.clone()]);
    let capture = &captures[0];
    assert_eq!(capture.bytes_read, bytes.len());
    assert_eq!(capture.raw_hex, b_ids_harness::hex(&bytes));
    assert!(capture.tls.is_some());
}

#[test]
fn listener_records_a_socket_that_was_opened_and_abandoned() {
    // ⚠ A browser opens sockets it abandons. A run that dropped them would
    // under-report what a navigation does, so the connection still produces a
    // capture, with its note.
    let captures = feed(
        Config {
            handshakes: 1,
            read_timeout: Duration::from_millis(300),
            ..one_connection()
        },
        vec![Vec::new()],
    );
    let capture = &captures[0];
    assert_eq!(capture.bytes_read, 0);
    assert!(capture.tls.is_none());
    assert!(
        capture.notes.iter().any(|n| n.why.contains("sent nothing")),
        "{:?}",
        capture.notes
    );
}

#[test]
fn listener_reassembles_a_record_split_across_two_reads() {
    // ⚠ A TLS record can arrive split, and a parser fed one read's worth of a
    // two-read hello reports a truncation that never happened. The length in
    // the record header is what says when to stop.
    let bytes = fixture_bytes();
    let oracle = Oracle::bind(one_connection()).expect("loopback binds");
    let addr = oracle.local_addr().expect("a bound address");
    let split = bytes.clone();
    let sender = thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).expect("the oracle is listening");
        let (head, tail) = split.split_at(40);
        stream.write_all(head).expect("the first write lands");
        stream.flush().expect("flush");
        thread::sleep(Duration::from_millis(60));
        stream.write_all(tail).expect("the second write lands");
        stream.flush().expect("flush");
        thread::sleep(Duration::from_millis(20));
    });
    let captures = oracle.run().expect("the accept succeeds");
    sender.join().expect("the sender finished");

    assert_eq!(captures[0].bytes_read, bytes.len());
    let tls = captures[0].tls.as_ref().expect("the hello parsed");
    assert_eq!(tls.extensions.len(), 12);
    assert!(
        captures[0].notes.is_empty(),
        "a split read produced notes: {:?}",
        captures[0].notes
    );
}

#[test]
fn listener_reports_more_than_one_connection_in_order() {
    // ⛔ One navigation is not one connection: driving a browser at a probe has
    // produced thirteen. The count is a switch and the order is recorded.
    let bytes = fixture_bytes();
    let captures = feed(
        Config {
            handshakes: 3,
            ..one_connection()
        },
        vec![bytes.clone(), bytes.clone(), bytes],
    );
    assert_eq!(captures.len(), 3);
    assert_eq!(
        captures.iter().map(|c| c.connection).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn listener_reads_a_cleartext_request_when_that_is_the_surface() {
    // ⭐ The seam that makes this multi-protocol from the first commit. A third
    // surface is a variant and a match arm, not a second listener.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\nUser-Agent: fixture\r\n\
                    Cookie: not-a-real-value\r\nAccept: */*\r\n\r\n";
    let captures = feed(
        Config {
            protocol: Protocol::Cleartext,
            ..one_connection()
        },
        vec![request.to_vec()],
    );
    let capture = &captures[0];
    assert_eq!(capture.request_line.as_deref(), Some("GET / HTTP/1.1"));
    // ⭐ THE CREDENTIAL KEEPS ITS NAME AND ITS POSITION. `SCHEMA-14`: whether
    // it was sent, and where in the order, is a fingerprint signal that carries
    // no secret, and a name dropped here left the order closed over an unmarked
    // gap.
    assert_eq!(
        capture.header_names,
        vec!["Host", "User-Agent", "Cookie", "Accept"]
    );
    // ⛔ The VALUE filter runs here as well as in the model: one gate per
    // action, and this is a different door into the same one.
    assert!(capture.header_values.is_empty(), "names only by default");
}

#[test]
fn listener_base_url_names_the_bound_port() {
    let oracle = Oracle::bind(one_connection()).expect("loopback binds");
    let url = oracle.base_url().expect("a base url");
    let port = oracle.local_addr().expect("a bound address").port();
    assert!(url.starts_with("https://127.0.0.1:"), "{url}");
    assert!(url.ends_with(&format!(":{port}/")), "{url}");

    let plain = Oracle::bind(Config {
        protocol: Protocol::Cleartext,
        ..one_connection()
    })
    .expect("loopback binds");
    assert!(
        plain.base_url().expect("a base url").starts_with("http://"),
        "the scheme follows the surface"
    );
}
