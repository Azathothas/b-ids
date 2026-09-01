//! Test support: the committed fixtures, and one way to feed bytes to a bound
//! oracle over a real loopback socket.
//!
//! ⛔ **One read path, and this is it.** Three test binaries in this directory
//! drive the listener and each one used to carry its own copy of the fixture
//! reader. Copy-pasted IO becomes copies that acquire different defects, and a
//! fix to one never reaches the others.

// ⚠ This module is compiled into every test binary in this directory and each
// one uses a different part of it, so what is unused HERE is used next door.
#![allow(dead_code)]

use std::io::Write as _;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use b_ids_harness::{Capture, Config, Oracle, Protocol};

/// How long a fed run may take before the accept is unblocked deliberately.
pub const DEADLINE: Duration = Duration::from_secs(20);

/// Where the committed fixtures live.
#[must_use]
pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Read one committed hex fixture by file name.
///
/// ⛔ Panics where the fixture is missing or is not hex. A test that silently
/// ran over an empty byte string would report green over nothing.
#[must_use]
pub fn fixture_bytes(name: &str) -> Vec<u8> {
    let path = fixtures().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the committed fixture {} is readable: {e}", path.display()));
    b_ids_harness::unhex(&text)
        .unwrap_or_else(|e| panic!("the committed fixture {} is hex: {e}", path.display()))
}

/// Bind, feed each payload on its own connection, and return what the oracle
/// read.
///
/// ⚠ An empty payload is a socket opened and abandoned, which a browser really
/// does.
#[must_use]
pub fn feed(config: Config, payloads: Vec<Vec<u8>>) -> Vec<Capture> {
    feed_within(config, payloads, DEADLINE)
}

/// The same, with the deadline named.
///
/// ⛔ **A run that would block forever is unblocked rather than left to hang.**
/// A test configured for more connections than it feeds is exactly what a
/// broken stop condition produces, and a hang has no message and no exit code:
/// in continuous integration it consumes the job's whole timeout and reports
/// nothing about which assertion would have failed. One extra connection turns
/// that into a count that is wrong, which an assertion can read.
#[must_use]
pub fn feed_within(config: Config, payloads: Vec<Vec<u8>>, within: Duration) -> Vec<Capture> {
    let oracle = Oracle::bind(config).expect("loopback binds");
    let addr = oracle.local_addr().expect("a bound address");

    let sender = thread::spawn(move || {
        for payload in payloads {
            let mut stream = TcpStream::connect(addr).expect("the oracle is listening");
            if payload.is_empty() {
                drop(stream);
                continue;
            }
            stream.write_all(&payload).expect("the write lands");
            stream.flush().expect("the flush lands");
            // Hold the socket open long enough for the oracle to read it.
            thread::sleep(Duration::from_millis(20));
        }
    });

    let (finished, wait) = std::sync::mpsc::channel();
    let runner = thread::spawn(move || {
        let captures = oracle.run().expect("the accept succeeds");
        let _ = finished.send(());
        captures
    });

    if wait.recv_timeout(within).is_err() {
        let _ = TcpStream::connect(addr);
    }
    sender.join().expect("the sender finished");
    runner.join().expect("the runner finished")
}

/// Feed ONE connection in several writes, with a pause between them.
///
/// ⚠ **This is the shape that catches a completeness rule which is wrong.** A
/// message delivered in one write is complete on the first read whatever the
/// rule says, so a rule that stops too early passes every single-write test and
/// truncates every real client. Both protocols this harness reads arrive split
/// in practice.
#[must_use]
pub fn feed_in_chunks(config: Config, chunks: Vec<Vec<u8>>, gap: Duration) -> Vec<Capture> {
    let oracle = Oracle::bind(config).expect("loopback binds");
    let addr = oracle.local_addr().expect("a bound address");
    let sender = thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).expect("the oracle is listening");
        for chunk in chunks {
            stream.write_all(&chunk).expect("the write lands");
            stream.flush().expect("the flush lands");
            thread::sleep(gap);
        }
    });
    let captures = oracle.run().expect("the accept succeeds");
    sender.join().expect("the sender finished");
    captures
}

/// The sixteen values RFC 8701 reserves, in order.
///
/// ⚠ Derived rather than listed. A typed list of sixteen constants is sixteen
/// chances to write one of them wrong, and the rule is one line: both bytes
/// equal, low nibble `a`.
#[must_use]
pub fn grease_values() -> Vec<u16> {
    (0..16_u16)
        .map(|n| {
            let byte = (n << 4) | 0x0a;
            (byte << 8) | byte
        })
        .collect()
}

/// Frame a minimal `ClientHello` record carrying exactly these extensions.
///
/// ⚠ **Not shared with `examples/make-fixture.rs`, and the reason is that they
/// build different things.** That example builds ONE hello whose every shape is
/// deliberate and whose bytes are committed and reviewed. This builds a hello
/// that varies per test case and whose bytes nobody reads. The framing they
/// have in common is the TLS record format, which is fixed by a specification
/// and cannot drift.
#[must_use]
pub fn client_hello(extensions: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&0x0303_u16.to_be_bytes()); // legacy_version
    body.extend_from_slice(&[0_u8; 32]); // random, zeroes so the fixture is stable
    body.push(32);
    body.extend_from_slice(&[0x11_u8; 32]); // session id
    body.extend_from_slice(&4_u16.to_be_bytes());
    body.extend_from_slice(&0x0a0a_u16.to_be_bytes()); // GREASE leads the cipher list
    body.extend_from_slice(&0x1301_u16.to_be_bytes());
    body.push(1);
    body.push(0); // one compression method, null

    let mut block = Vec::new();
    for (codepoint, ext_body) in extensions {
        block.extend_from_slice(&codepoint.to_be_bytes());
        let length = u16::try_from(ext_body.len()).expect("a fixture extension is short");
        block.extend_from_slice(&length.to_be_bytes());
        block.extend_from_slice(ext_body);
    }
    let block_len = u16::try_from(block.len()).expect("a fixture extension block is short");
    body.extend_from_slice(&block_len.to_be_bytes());
    body.extend_from_slice(&block);

    let mut handshake = vec![0x01];
    let body_len = u32::try_from(body.len()).expect("a fixture hello is short");
    handshake.extend_from_slice(&body_len.to_be_bytes()[1..]);
    handshake.extend_from_slice(&body);

    let mut record = vec![0x16, 0x03, 0x01];
    let handshake_len = u16::try_from(handshake.len()).expect("a fixture record is short");
    record.extend_from_slice(&handshake_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

/// A constructed navigation of thirteen connections, in the shape one was
/// measured in.
///
/// ⛔ **CONSTRUCTED, not captured.** Driving a browser at the probe produced
/// thirteen connections, the first carrying no HTTP/2 at all and every one
/// after the second offering a pre-shared key. That reading is inherited and
/// lives in `docs/inherited-claims.md` section 8; this rebuilds its SHAPE so
/// the selection rule can be exercised. ⛔ No value here is a measurement.
///
/// ⚠ It needs the handshake terminated to arise from a real browser, because
/// only a terminated connection carries both a `ClientHello` and HTTP/2 frames.
/// The shape is representable today and the rule is written against it now, so
/// that the first terminated capture is selected rather than eyeballed.
#[must_use]
pub fn thirteen_connection_navigation() -> Vec<Capture> {
    let session_ticket = b_ids_harness::select::SESSION_TICKET;
    let pre_shared_key = b_ids_harness::select::PRE_SHARED_KEY;

    let hello = |resumed: bool| {
        let marker = if resumed {
            pre_shared_key
        } else {
            session_ticket
        };
        let bytes = client_hello(&[
            (0x6a6a, Vec::new()),
            (0x0000, vec![0x00, 0x00]),
            (marker, vec![0x01]),
            (0x4a4a, vec![0x00]),
        ]);
        b_ids_harness::parse_record(&bytes).expect("a constructed hello parses")
    };

    let mut notes = Vec::new();
    let http2 = b_ids_harness::h2::parse_connection(
        &fixture_bytes("h2-connection.hex"),
        b_ids_schema::http::ValuePolicy::NamesOnly,
        &mut notes,
    )
    .expect("the committed HTTP/2 fixture parses");

    (1..=13_u32)
        .map(|connection| {
            let resumed = connection >= 3;
            let parsed = hello(resumed);
            Capture {
                schema: b_ids_harness::CAPTURE_SCHEMA.to_owned(),
                connection,
                at: "2026-09-01T00:00:00Z".to_owned(),
                peer: "REDACTED".to_owned(),
                protocol: Protocol::TlsRaw,
                bytes_read: parsed.raw_hex.len() / 2,
                raw_hex: parsed.raw_hex.clone(),
                tls: Some(parsed.tls),
                // ⚠ The FIRST connection carried no HTTP/2 at all. It is the
                // preconnect the browser abandoned, and a run that dropped it
                // would under-report what a navigation does.
                http2: (connection > 1).then(|| http2.clone()),
                // ⚠ CONSTRUCTED from a raw surface, which terminates nothing. The
                // shape this exercises is the SELECTION rule, and that reads the
                // hello and the frames rather than what a handshake negotiated.
                termination: None,
                request_line: None,
                header_names: Vec::new(),
                header_values: Vec::new(),
                notes: Vec::new(),
            }
        })
        .collect()
}

/// A configuration that accepts exactly one connection.
///
/// ⛔ **The default is EIGHT**, because one handshake is not a sample. A test
/// that feeds one connection has to say so, and saying so here rather than in
/// twenty places is what stops the next test from quietly waiting for seven
/// connections nobody is going to make.
#[must_use]
pub fn one_connection() -> Config {
    Config {
        handshakes: 1,
        ..Config::default()
    }
}
