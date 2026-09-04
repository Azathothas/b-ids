//! `HARNESS-12`. The oracle mode: hand a caller its own capture back, and keep
//! nothing.
//!
//! ⛔ **Every test name starts with `oracle`**, because
//! `cargo test -p b-ids-harness --test oracle` is what runs this file alone.
//!
//! ⛔ **THE MODE IS BUILT AND IT IS NOT HOSTED.** The entry's own decision says
//! so: a hosted endpoint receives traffic from people, which is the one thing
//! this project's scope boundary says it does not do, and that question needs
//! an answer written down and a person's approval before anything is stood up.
//! ⚠ Everything here binds the loopback address and is torn down by the test
//! that started it.

use std::io::{Read, Write};
use std::net::TcpStream;

// ⭐ THE TYPE WAS ALREADY CALLED `Oracle`, before this entry existed. The
// listener has been the shape a public one needs since HARNESS-01 ruled the
// oracle is a server rather than a client; what HARNESS-12 adds is the answer.
use b_ids_harness::listener::{Config, Oracle, Protocol};

/// A listener in the oracle mode, on a port the operating system chooses.
fn oracle(handshakes: u32) -> Oracle {
    let mut config = Config {
        protocol: Protocol::Cleartext,
        serve: true,
        handshakes,
        ..Config::default()
    };
    config.run_timeout = Some(std::time::Duration::from_secs(20));
    Oracle::bind(config).expect("a loopback listener")
}

#[test]
fn oracle_a_caller_gets_its_own_capture_back() {
    let listener = oracle(1);
    let addr = listener.local_addr().expect("a bound address");

    // ⭐ THE CALLER IS A REAL SOCKET SPEAKING REAL HTTP/1.1, not a function
    // call. What this asserts is what a browser pointed at the endpoint gets.
    let caller = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).expect("the oracle is listening");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: oracle.invalid\r\nAccept: */*\r\n\r\n")
            .expect("the request goes out");
        let mut answer = String::new();
        stream
            .read_to_string(&mut answer)
            .expect("an answer comes back");
        answer
    });

    let captures = listener.run().expect("the run completes");
    let answer = caller.join().expect("the caller finished");

    assert_eq!(captures.len(), 1, "one connection was accepted");

    // ⛔ A REAL RESPONSE, with a status line and a length a client can act on.
    assert!(answer.starts_with("HTTP/1.1 200 OK\r\n"), "{answer:.60}");
    assert!(
        answer.contains("Content-Type: application/json"),
        "the answer is not typed as JSON"
    );
    assert!(
        answer.contains("Cache-Control: no-store"),
        "a capture of somebody's own browser must not be cached by anything in between"
    );

    let body = answer
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("a body follows the head");
    let parsed: serde_json::Value = serde_json::from_str(body).expect("the body is JSON");

    // ⭐ THE FULL MODEL, RAW BYTES INCLUDED, which is the whole of the Problem:
    // every existing hosted service returns a subset and no raw hello.
    assert_eq!(parsed["connection"], 1);
    assert_eq!(parsed["protocol"], "cleartext");
    let raw = parsed["raw_hex"]
        .as_str()
        .expect("the raw bytes are returned");
    assert!(!raw.is_empty(), "the answer carried no raw bytes");
    assert!(
        parsed["request_line"]
            .as_str()
            .is_some_and(|line| line.starts_with("GET / HTTP/1.1")),
        "the answer does not carry what the caller sent"
    );

    // ⚠ AND THE ANSWER IS THE CAPTURE THE RUN ALSO REPORTS, not a second one
    // built for the socket. A serve path that answered from its own model would
    // be a second answer to the question this project exists to answer once.
    assert_eq!(
        raw, captures[0].raw_hex,
        "the caller got a different capture"
    );
}

#[test]
fn oracle_an_http2_caller_is_told_why_it_gets_no_answer() {
    // ⛔ TOLD, RATHER THAN LEFT WAITING. Answering over HTTP/2 needs an HPACK
    // encoder and this crate has a decoder, so the honest outcome is a note
    // naming that rather than a socket that hangs.
    let listener = oracle(1);
    let addr = listener.local_addr().expect("a bound address");
    let caller = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).expect("the oracle is listening");
        // The connection preface, then an empty SETTINGS frame and a HEADERS
        // frame with an END_HEADERS flag, which is what makes the read complete.
        let mut out = Vec::from(&b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"[..]);
        out.extend_from_slice(&[0, 0, 0, 0x04, 0, 0, 0, 0, 0]);
        out.extend_from_slice(&[0, 0, 1, 0x01, 0x04, 0, 0, 0, 1, 0x82]);
        let _ = stream.write_all(&out);
        let _ = stream.flush();
        let mut answer = Vec::new();
        let _ = stream.read_to_end(&mut answer);
        answer
    });

    let captures = listener.run().expect("the run completes");
    let answer = caller.join().expect("the caller finished");

    assert_eq!(captures.len(), 1);
    assert!(
        answer.is_empty(),
        "an HTTP/2 caller was sent {} byte(s), and nothing here can encode one",
        answer.len()
    );
    assert!(
        captures[0]
            .notes
            .iter()
            .any(|n| n.field == "serve" && n.why.contains("HPACK encoder")),
        "the capture does not say why the caller got no answer: {:?}",
        captures[0].notes
    );
}

#[test]
fn oracle_the_mode_is_off_unless_it_is_asked_for() {
    // ⛔ A HARNESS THAT ANSWERED EVERY CALLER BY DEFAULT would be an oracle
    // nobody chose to run, and this project's scope boundary says it does not
    // receive traffic from people.
    assert!(!Config::default().serve, "the oracle mode is on by default");

    let mut config = Config {
        protocol: Protocol::Cleartext,
        handshakes: 1,
        ..Config::default()
    };
    config.run_timeout = Some(std::time::Duration::from_secs(20));
    let listener = Oracle::bind(config).expect("a loopback listener");
    let addr = listener.local_addr().expect("a bound address");
    let caller = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).expect("listening");
        let _ = stream.write_all(b"GET / HTTP/1.1\r\nHost: x\r\n\r\n");
        let mut answer = Vec::new();
        let _ = stream.read_to_end(&mut answer);
        answer
    });
    let captures = listener.run().expect("the run completes");
    let answer = caller.join().expect("the caller finished");
    assert_eq!(captures.len(), 1, "the control captured nothing");
    assert!(
        answer.is_empty(),
        "a run that was not asked to serve answered anyway with {} byte(s)",
        answer.len()
    );
}

#[test]
fn oracle_no_retain_creates_no_file() {
    // ⛔ THE ENTRY'S OWN WORDING: a test asserts the no-retain default by
    // checking that the process created no file. ⭐ It runs the BINARY in an
    // empty directory and counts what is in that directory afterwards, rather
    // than asking the code what it believes it wrote. A list of writing
    // switches can go stale; a directory cannot.
    let exe = env!("CARGO_BIN_EXE_b-ids-harness");
    let dir = std::env::temp_dir().join(format!(
        "b-ids-oracle-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));
    std::fs::create_dir_all(&dir).expect("a scratch directory");

    let out = std::process::Command::new(exe)
        .current_dir(&dir)
        .args([
            "--plain",
            "--serve",
            "--no-retain",
            "--once",
            "--json",
            "--run-timeout-ms",
            "1200",
        ])
        .output()
        .expect("the harness runs");

    let left: Vec<String> = std::fs::read_dir(&dir)
        .expect("the scratch directory is readable")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        left.is_empty(),
        "a --no-retain run created {left:?}, and it must keep nothing"
    );
    // ⚠ AND IT RAN. A binary that refused its arguments would also create no
    // file, and the two must not read the same.
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(
        !said.contains("usage: b-ids-harness"),
        "the run was refused rather than performed: {said}"
    );
}

#[test]
fn oracle_no_retain_refuses_every_switch_that_writes() {
    // ⛔ REFUSED BY NAME, AT PARSE TIME, before a socket is opened. A flag that
    // merely INTENDED to keep nothing while --hello-out sat beside it would be
    // the "a setting that no code reads" row of forbidden-patterns.md.
    let exe = env!("CARGO_BIN_EXE_b-ids-harness");
    for flag in ["--ca-out", "--hello-out", "--write-golden"] {
        let out = std::process::Command::new(exe)
            .args(["--no-retain", flag, "would-be-written"])
            .output()
            .expect("the harness runs");
        assert_eq!(
            out.status.code(),
            Some(2),
            "{flag} beside --no-retain was not refused"
        );
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(
            said.contains(flag) && said.contains("--no-retain"),
            "the refusal does not name both switches: {said}"
        );
    }
}
