//! HARNESS-13. Terminate the handshake, and mint the authority that lets a
//! browser complete it.
//!
//! The acceptance: a client completes a VERIFIED handshake against a run started
//! with `--ca-out`, trusting only the authority that run wrote; the capture keeps
//! the raw `ClientHello` bytes and they are byte identical to what the client
//! sent; the negotiated protocol is recorded; and a run started without
//! `--ca-out` still terminates nothing.
//!
//! ⛔ Every test name starts with `termination`, because
//! `cargo test -p b-ids-harness termination` is the entry's acceptance command.
//!
//! ⚠ **The client trusts ONE certificate: the one the run just wrote.** A test
//! that disabled verification would prove the handshake and not the authority,
//! and the authority is the whole reason the switch exists.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod support;

/// The compiled command, which is what a switch test has to drive.
///
/// ⚠ Driving the LIBRARY would prove the library. `--ca-out` is a property of
/// the command, and a flag documented, parsed and never passed through is a
/// shape this project has already been bitten by.
fn harness_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("the test binary has a path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(format!("b-ids-harness{}", std::env::consts::EXE_SUFFIX))
}

/// A socket that keeps every byte the client wrote.
///
/// ⭐ **This is what lets the acceptance compare bytes rather than shapes.** The
/// capture claims to hold the `ClientHello` the client sent; without a record of
/// what was sent, the only checkable claim is that it holds *a* hello.
struct Recording {
    inner: TcpStream,
    written: Arc<Mutex<Vec<u8>>>,
}

impl Read for Recording {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for Recording {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let taken = self.inner.write(buf)?;
        self.written
            .lock()
            .expect("the recording lock is not poisoned")
            .extend_from_slice(&buf[..taken]);
        Ok(taken)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// What one driven run produced.
struct Run {
    stdout: String,
    status: i32,
    client_wrote: Vec<u8>,
}

/// Start the command, complete a verified handshake against it, and send the
/// committed HTTP/2 connection fixture over the top.
///
/// ⚠ **The fixture is the SAME bytes the cleartext surface is tested with.** A
/// second copy of a connection would let the two surfaces drift apart while both
/// tests stayed green.
fn drive(extra: &[&str], ca_path: &std::path::Path, send: &[u8]) -> Run {
    let mut switches: Vec<String> = vec![
        "--ca-out".to_owned(),
        ca_path.display().to_string(),
        "--once".to_owned(),
        "--json".to_owned(),
        "--run-timeout-ms".to_owned(),
        "20000".to_owned(),
    ];
    switches.extend(extra.iter().map(|s| (*s).to_owned()));

    let mut child = Command::new(harness_bin())
        .args(&switches)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the harness command runs");

    // ⭐ Block on the base URL line rather than sleeping. The port is not
    // knowable until the command says so.
    let stdout = child.stdout.take().expect("stdout is piped");
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        use std::io::{BufRead as _, BufReader};
        let mut reader = BufReader::new(stdout);
        let mut first = String::new();
        let _ = reader.read_line(&mut first);
        let _ = tx.send(first.clone());
        let mut rest = String::new();
        let _ = std::io::Read::read_to_string(&mut reader, &mut rest);
        format!("{first}{rest}")
    });
    let url = rx
        .recv_timeout(Duration::from_secs(20))
        .expect("the command prints its base URL before it accepts");
    assert!(
        url.starts_with("https://"),
        "the terminated surface offers https: {url}"
    );
    let port: u16 = url
        .rsplit(':')
        .next()
        .and_then(|tail| tail.trim_end_matches(['/', '\n', '\r']).parse().ok())
        .unwrap_or_else(|| panic!("no port in the base URL line: {url}"));

    let written = Arc::new(Mutex::new(Vec::new()));
    let socket = Recording {
        inner: TcpStream::connect(("127.0.0.1", port)).expect("the command is listening"),
        written: Arc::clone(&written),
    };
    let ca_pem = std::fs::read(ca_path).expect("the run wrote its authority");
    speak_h2(socket, &ca_pem, send);

    let status = child.wait().expect("the command exits");
    let stdout = reader.join().expect("the reader finished");
    let client_wrote = written
        .lock()
        .expect("the recording lock is not poisoned")
        .clone();
    Run {
        stdout,
        status: status.code().unwrap_or(-1),
        client_wrote,
    }
}

/// Complete a verified handshake and write `payload` over it.
fn speak_h2(socket: Recording, ca_pem: &[u8], payload: &[u8]) {
    use rustls::pki_types::pem::PemObject as _;

    let mut roots = rustls::RootCertStore::empty();
    let anchor = rustls::pki_types::CertificateDer::from_pem_slice(ca_pem)
        .expect("the authority the run wrote is PEM");
    roots
        .add(anchor)
        .expect("the authority the run wrote is a usable trust anchor");

    let mut config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec()];

    let server = rustls::pki_types::ServerName::try_from("127.0.0.1")
        .expect("the loopback address is a server name");
    let mut connection = rustls::ClientConnection::new(Arc::new(config), server)
        .expect("the client connection is buildable");
    let mut socket = socket;
    let mut stream = rustls::Stream::new(&mut connection, &mut socket);
    stream
        .write_all(payload)
        .expect("the payload lands over the completed handshake");
    stream.flush().expect("the flush lands");
    // ⚠ Hold the socket open long enough for the server to read the payload. The
    // server stops on its own completeness rule; dropping here would race it.
    std::thread::sleep(Duration::from_millis(200));
}

/// The one capture line of a `--json` run.
fn capture_of(run: &Run) -> serde_json::Value {
    let line = run
        .stdout
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or_else(|| panic!("no capture object in the output:\n{}", run.stdout));
    serde_json::from_str(line).expect("the capture line is JSON")
}

#[test]
fn termination_completes_a_verified_handshake_and_reads_http2() {
    let dir = std::env::temp_dir().join(format!("b-ids-termination-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let ca = dir.join("ca.pem");

    let fixture = support::fixture_bytes("h2-connection.hex");
    let run = drive(&[], &ca, &fixture);

    let capture = capture_of(&run);
    assert_eq!(
        capture["protocol"], "tls_terminated",
        "the switch selects the surface: {}",
        run.stdout
    );
    assert!(
        !capture["tls"].is_null(),
        "the hello is parsed from the bytes the listener read: {}",
        run.stdout
    );
    let termination = &capture["termination"];
    assert_eq!(
        termination["alpn"], "h2",
        "the peer selected h2 and it is recorded: {}",
        run.stdout
    );
    assert!(
        termination["version"].is_string(),
        "the negotiated version is recorded as a condition of the capture: {}",
        run.stdout
    );
    assert!(
        termination["cipher_suite"].is_string(),
        "the negotiated suite is recorded as a condition of the capture: {}",
        run.stdout
    );

    // ⛔ The HTTP/2 half comes from the SAME reader the cleartext surface uses.
    let frames = capture["http2"]["frames"].as_array().unwrap_or_else(|| {
        panic!(
            "no HTTP/2 frames over a terminated connection:\n{}",
            run.stdout
        )
    });
    assert!(
        !frames.is_empty(),
        "the terminated surface reached HTTP/2: {}",
        run.stdout
    );

    // ⭐ The claim that matters: the recorded hello is the bytes the client put
    // on the wire, not a re-encoding of what a TLS library made of them.
    let sent = &run.client_wrote;
    assert!(sent.len() > 5, "the client wrote a record");
    let declared = usize::from(u16::from(sent[3]) << 8 | u16::from(sent[4]));
    let first_record = &sent[..5 + declared];
    assert_eq!(
        capture["raw_hex"].as_str().expect("raw_hex is a string"),
        b_ids_harness::hex(first_record),
        "the raw block is the first record the client sent, byte for byte"
    );
    assert_eq!(
        capture["bytes_read"]
            .as_u64()
            .expect("bytes_read is a number"),
        first_record.len() as u64
    );

    assert_eq!(run.status, 0, "a completed run exits 0: {}", run.stdout);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn termination_records_the_authority_and_never_its_key() {
    let dir = std::env::temp_dir().join(format!("b-ids-authority-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let ca = dir.join("ca.pem");
    let fixture = support::fixture_bytes("h2-connection.hex");
    let _ = drive(&[], &ca, &fixture);

    let written = std::fs::read_to_string(&ca).expect("the run wrote its authority");
    assert!(
        written.contains("BEGIN CERTIFICATE"),
        "the authority is written as a certificate"
    );
    // ⛔ A private key on disk is what docs/security/secrets.md refuses, and the
    // client needs only the certificate to verify.
    assert!(
        !written.contains("PRIVATE KEY"),
        "no private key is written beside the authority"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn termination_is_absent_on_the_surfaces_that_do_not_terminate() {
    // ⛔ The default surface reads a hello and closes. A capture that carried a
    // termination record there would be claiming a handshake nobody completed.
    let captures = support::feed(
        support::one_connection(),
        vec![support::client_hello(&[(0x0000, vec![0x00, 0x00])])],
    );
    assert_eq!(captures.len(), 1);
    assert!(
        captures[0].termination.is_none(),
        "the raw surface terminates nothing"
    );
    assert!(captures[0].tls.is_some(), "and it still reads the hello");
}

#[test]
fn termination_refuses_a_terminated_surface_with_no_server_configuration() {
    // ⛔ A protocol that says it terminates beside no configuration would be a
    // mode that silently did nothing, which is the "flag no code reads" defect
    // from docs/conventions/forbidden-patterns.md. It is refused at the bind.
    let config = b_ids_harness::Config {
        protocol: b_ids_harness::Protocol::TlsTerminated,
        terminator: None,
        ..support::one_connection()
    };
    let refused = b_ids_harness::Oracle::bind(config).expect_err("the combination is refused");
    assert!(
        refused.to_string().contains("server configuration"),
        "the refusal names what is missing: {refused}"
    );
}
