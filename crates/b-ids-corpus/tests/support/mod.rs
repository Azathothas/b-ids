//! Test support: a capture built from the harness's own committed fixtures,
//! and a throwaway corpus root.
//!
//! ⛔ **The fixtures are the harness's and they are read where they live.** A
//! copy under this crate would be a second fixture that drifts from the first,
//! and the drift would be invisible: both would keep parsing.
//!
//! ⚠ **Nothing here is a measurement.** The bytes are shaped like a capture and
//! are not one. No field of anything this module produces may be written into
//! the corpus.

// ⚠ This module is compiled into every test binary in this directory and each
// one uses a different part of it, so what is unused HERE is used next door.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use b_ids_corpus::Identity;
use b_ids_harness::listener::Termination;
use b_ids_harness::{CAPTURE_SCHEMA, Capture, Protocol};
use b_ids_schema::http::ValuePolicy;
use b_ids_schema::{Channel, Os, Trust};

/// Where the harness's committed fixtures live.
#[must_use]
pub fn harness_fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("b-ids-harness")
        .join("fixtures")
}

/// Read one committed hex fixture by file name.
///
/// ⛔ Panics where the fixture is missing or is not hex. A test that silently
/// ran over an empty byte string would report green over nothing.
#[must_use]
pub fn fixture_bytes(name: &str) -> Vec<u8> {
    let path = harness_fixtures().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("the committed fixture {} is readable: {e}", path.display()));
    b_ids_harness::unhex(&text)
        .unwrap_or_else(|e| panic!("the committed fixture {} is hex: {e}", path.display()))
}

/// A capture shaped like the cold connection of a navigation: a `ClientHello`,
/// a completed handshake, and the HTTP/2 behind it.
#[must_use]
pub fn cold_capture() -> Capture {
    let hello_bytes = fixture_bytes("client-hello.hex");
    let hello = b_ids_harness::parse_record(&hello_bytes).expect("the committed hello parses");
    let h2_bytes = fixture_bytes("h2-connection.hex");
    let mut notes = Vec::new();
    let http2 = b_ids_harness::h2::parse_connection(&h2_bytes, ValuePolicy::NamesOnly, &mut notes)
        .expect("the committed HTTP/2 connection parses");

    Capture {
        schema: CAPTURE_SCHEMA.to_owned(),
        connection: 2,
        at: "2026-09-01T04:05:06Z".to_owned(),
        peer: "127.0.0.1:54321".to_owned(),
        protocol: Protocol::TlsTerminated,
        bytes_read: hello_bytes.len(),
        raw_hex: hello.raw_hex,
        tls: Some(hello.tls),
        http2: Some(http2),
        termination: Some(Termination {
            alpn: Some("h2".to_owned()),
            version: Some("TLSv1_3".to_owned()),
            cipher_suite: Some("TLS13_AES_128_GCM_SHA256".to_owned()),
            plaintext_bytes: h2_bytes.len(),
            plaintext_hex: b_ids_harness::hex(&h2_bytes),
        }),
        request_line: None,
        header_names: Vec::new(),
        header_values: Vec::new(),
        notes,
    }
}

/// The identity a test capture is labelled with.
///
/// ⛔ Not a build anybody measured. The version is deliberately not one this
/// project has ever captured, so a copy of it into the corpus would be visible.
#[must_use]
pub fn identity() -> Identity {
    Identity {
        name: "Chrome".to_owned(),
        version: "999.0.0.1".to_owned(),
        channel: Channel::Stable,
        branded: true,
        os: Os::Windows,
        arch: "x86_64".to_owned(),
        distribution: None,
        method: "host".to_owned(),
        harness: "b-ids-harness 0.0.0".to_owned(),
        operator: "test".to_owned(),
        trust: Trust::SpkiPin,
        resumption: Some(b_ids_schema::Resumption::Refused),
        switches: vec![
            "--user-data-dir=/home/user/throwaway-1234".to_owned(),
            "--no-first-run".to_owned(),
        ],
        // ⚠ None: this identity describes a browser nothing fetched.
        acquisition: None,
    }
}

/// A corpus root nobody keeps, removed when the guard drops.
///
/// ⚠ **A guard rather than a call at the end of the test.** An assertion that
/// fails returns early, and a cleanup after it would never run, so a failing
/// test would leave a tree behind on every run.
#[derive(Debug)]
pub struct Throwaway {
    /// The root a store is opened at.
    pub root: PathBuf,
}

impl Throwaway {
    /// Create one, named for the calling test so two cannot collide.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let root = std::env::temp_dir().join(format!(
            "b-ids-corpus-test-{name}-{}-{stamp}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("a throwaway corpus root is creatable");
        Self { root }
    }
}

impl Drop for Throwaway {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
