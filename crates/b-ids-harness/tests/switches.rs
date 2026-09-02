//! HARNESS-02. The switches, each of which exists because something went wrong
//! without it.
//!
//! The acceptance: every switch is exercised, `--bind 0.0.0.0` is refused with
//! a message naming the reason, and the default run's output contains no header
//! value.
//!
//! ⛔ Every test name starts with `switches`, because
//! `cargo test -p b-ids-harness switches` is the entry's acceptance command.

use std::io::Write as _;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use b_ids_harness::{Config, Oracle, Protocol, parse_bind};

mod support;

use support::one_connection;

/// The compiled command, which is what a switch test has to drive.
///
/// ⚠ Driving the LIBRARY would prove the library. A switch is a property of the
/// command, and the two have been seen to disagree in other projects: a flag
/// documented, parsed and never passed through.
fn harness_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("the test binary has a path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(format!("b-ids-harness{}", std::env::consts::EXE_SUFFIX))
}

/// Run the command with the given switches, feeding one payload once it has
/// printed its base URL.
fn run_with(switches: &[&str], payload: Vec<u8>) -> (String, String, i32) {
    let mut child = Command::new(harness_bin())
        .args(switches)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the harness command runs");

    // ⭐ Read the base URL line, then connect. Blocking on the line rather than
    // sleeping is what makes this deterministic: the port is not knowable until
    // the command says so.
    let stdout = child.stdout.take().expect("stdout is piped");
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = thread::spawn(move || {
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
    let port: u16 = url
        .rsplit(':')
        .next()
        .and_then(|tail| tail.trim_end_matches(['/', '\n', '\r']).parse().ok())
        .unwrap_or_else(|| panic!("no port in the base URL line: {url}"));

    if !payload.is_empty() {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("the command is listening");
        stream.write_all(&payload).expect("the write lands");
        stream.flush().expect("flush");
        thread::sleep(Duration::from_millis(40));
    }

    let status = child.wait().expect("the command exits");
    let out = reader.join().expect("the reader finished");
    let mut err = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut stderr, &mut err);
    }
    (out, err, status.code().unwrap_or(-1))
}

fn fixture_bytes() -> Vec<u8> {
    support::fixture_bytes("client-hello.hex")
}

#[test]
fn switches_bind_refuses_the_unspecified_address_by_name() {
    // ⛔ The acceptance names this one specifically. Binding everything accepts
    // the local network as well, and a fixture that records headers does not
    // belong on every interface.
    let refused = parse_bind("0.0.0.0").expect_err("the unspecified address is refused");
    assert!(refused.why.contains("unspecified"), "{refused}");
    assert!(refused.why.contains("local network"), "{refused}");
    assert!(
        refused.to_string().starts_with("--bind 0.0.0.0:"),
        "{refused}"
    );

    // The IPv6 unspecified address is the same rule.
    assert!(parse_bind("::").is_err());
    // A literal is accepted.
    assert!(parse_bind("127.0.0.1").is_ok());
}

#[test]
fn switches_bind_refuses_a_hostname_by_name() {
    // ⚠ A leaf certificate needs a literal, so a hostname cannot go here even
    // where it resolves.
    let refused = parse_bind("localhost").expect_err("a hostname is refused");
    assert!(refused.why.contains("hostname"), "{refused}");
    assert!(refused.why.contains("literal"), "{refused}");
}

/// Run the command and require it to EXIT within a deadline.
///
/// ⛔ **A refusal test that waits forever is worse than one that fails.** Found
/// by mutation: with the unspecified-address refusal removed, the command bound
/// successfully and blocked on the accept, and the test hung instead of
/// reporting. A guard whose test cannot fail loudly is a guard nobody knows
/// works.
fn must_exit(switches: &[&str], within: Duration) -> (String, i32) {
    let mut child = Command::new(harness_bin())
        .args(switches)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the harness command runs");
    let deadline = std::time::Instant::now() + within;
    loop {
        match child.try_wait().expect("the child can be polled") {
            Some(status) => {
                let mut err = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = std::io::Read::read_to_string(&mut stderr, &mut err);
                }
                return (err, status.code().unwrap_or(-1));
            }
            None if std::time::Instant::now() > deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("the command did not exit within {within:?}: {switches:?}");
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    }
}

#[test]
fn switches_the_command_refuses_the_unspecified_address_too() {
    // ⛔ The library refusing is not the command refusing. This is the door the
    // operator actually reaches.
    let (stderr, code) = must_exit(&["--bind", "0.0.0.0"], Duration::from_secs(15));
    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("unspecified"), "{stderr}");
}

#[test]
fn switches_the_default_run_records_no_header_value() {
    // ⛔ The other half of the acceptance. --header-values is the one switch
    // that can log a credential, so the default has to be seen to withhold.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\nUser-Agent: a-fixture-value\r\n\r\n";
    let (out, _err, code) = run_with(&["--plain", "--once", "--json"], request.to_vec());
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("\"header_names\""), "{out}");
    assert!(
        !out.contains("a-fixture-value"),
        "a header value reached the output: {out}"
    );
    assert!(out.contains("\"header_values\":[]"), "{out}");
}

#[test]
fn switches_header_values_records_them_when_it_is_passed() {
    let request = b"GET / HTTP/1.1\r\nHost: example\r\nUser-Agent: a-fixture-value\r\n\r\n";
    let (out, _err, code) = run_with(
        &["--plain", "--once", "--json", "--header-values"],
        request.to_vec(),
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("a-fixture-value"), "{out}");
}

#[test]
fn switches_header_values_still_drops_a_credential() {
    // ⚠ The switch widens what is recorded; it does not lift the rule.
    // ⭐ The title stays and what is dropped changed: `SCHEMA-14` made the NAME
    // and the position recordable, so this asserts the VALUE is not there while
    // the name is.
    let request =
        b"GET / HTTP/1.1\r\nHost: example\r\nCookie: not-a-real-value\r\nAccept: */*\r\n\r\n";
    let (out, _err, code) = run_with(
        &["--plain", "--once", "--json", "--header-values"],
        request.to_vec(),
    );
    assert_eq!(code, 0, "{out}");
    assert!(!out.contains("not-a-real-value"), "{out}");
    assert!(out.contains("Cookie"), "the name is kept: {out}");
}

#[test]
fn switches_hello_out_writes_the_raw_bytes() {
    let dir = std::env::temp_dir().join(format!("b-ids-hello-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a temp directory");
    let path = dir.join("hello.hex");
    let bytes = fixture_bytes();
    let (_out, _err, code) = run_with(
        &["--once", "--hello-out", path.to_str().expect("utf-8 path")],
        bytes.clone(),
    );
    assert_eq!(code, 0);
    let written = std::fs::read_to_string(&path).expect("--hello-out wrote a file");
    assert_eq!(written.trim(), b_ids_harness::hex(&bytes));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn switches_json_prints_the_base_url_first_then_one_object_per_connection() {
    // ⭐ The base URL BEFORE the accept blocks. A caller cannot point a browser
    // at a port it has not been told.
    let (out, _err, code) = run_with(&["--once", "--json"], fixture_bytes());
    assert_eq!(code, 0, "{out}");
    let mut lines = out.lines();
    let first = lines.next().expect("a first line");
    assert!(first.starts_with("https://127.0.0.1:"), "{first}");
    let object = lines.next().expect("one object");
    assert!(
        object.starts_with('{') && object.contains(b_ids_harness::CAPTURE_SCHEMA),
        "{object}"
    );
}

#[test]
fn switches_plain_changes_the_scheme_in_the_base_url() {
    let (out, _err, _code) = run_with(&["--plain", "--once"], b"GET / HTTP/1.1\r\n\r\n".to_vec());
    assert!(out.starts_with("http://127.0.0.1:"), "{out}");
}

#[test]
fn switches_handshakes_accepts_exactly_that_many() {
    let oracle = Oracle::bind(Config {
        handshakes: 2,
        ..one_connection()
    })
    .expect("loopback binds");
    let addr = oracle.local_addr().expect("a bound address");
    let bytes = fixture_bytes();
    let sender = thread::spawn(move || {
        for _ in 0..2 {
            let mut stream = TcpStream::connect(addr).expect("connect");
            stream.write_all(&bytes).expect("write");
            stream.flush().expect("flush");
            thread::sleep(Duration::from_millis(20));
        }
    });
    let captures = oracle.run().expect("accept");
    sender.join().expect("the sender finished");
    assert_eq!(captures.len(), 2);
}

#[test]
fn switches_expect_file_exits_one_on_a_difference() {
    // ⭐ An instrument that cannot fail is research that decays. This is the
    // switch that turns the harness into a regression check.
    let dir = std::env::temp_dir().join(format!("b-ids-golden-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a temp directory");
    let golden = dir.join("golden.json");
    let path = golden.to_str().expect("utf-8 path").to_owned();

    let (_out, _err, code) = run_with(&["--once", "--write-golden", &path], fixture_bytes());
    assert_eq!(code, 0);
    assert!(golden.exists(), "--write-golden wrote nothing");

    // The same bytes match.
    let (out, _err, code) = run_with(&["--once", "--expect-file", &path], fixture_bytes());
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("matches"), "{out}");

    // ⛔ And different bytes do NOT. A comparison that has never refused is a
    // comparison nobody knows works.
    let mut different = fixture_bytes();
    let last = different.len() - 1;
    different[last] ^= 0xff;
    let (_out, err, code) = run_with(&["--once", "--expect-file", &path], different);
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("does not match"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn switches_ca_out_mints_an_authority_and_selects_the_terminated_surface() {
    // ⭐ It used to be REFUSED, because minting an authority is only useful
    // once the handshake is terminated and this tree had no TLS server.
    // `HARNESS-13` vendored one, so the switch now writes the authority and
    // selects the surface. `HARNESS-13` owns what happens over it.
    //
    // ⚠ THE DEADLINE IS LOAD-BEARING. Without it this test hangs rather than
    // fails: the command binds and blocks on an accept nobody makes. That is
    // exactly what happened when the refusal above stopped refusing, and a
    // hang has no message and no exit code.
    let dir = std::env::temp_dir().join(format!("b-ids-switches-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let ca = dir.join("ca.pem");
    let (out, err, code) = run_with(
        &[
            "--ca-out",
            &ca.display().to_string(),
            "--once",
            "--run-timeout-ms",
            "1500",
        ],
        Vec::new(),
    );

    // ⚠ 1, not 0: the run accepted nothing, so it reports a shortfall. What
    // this test asserts is the SWITCH, and the surface it selected.
    assert_eq!(code, 1, "{err}");
    assert!(out.starts_with("https://"), "{out}");
    let written = std::fs::read_to_string(&ca).expect("the run wrote its authority");
    assert!(written.contains("BEGIN CERTIFICATE"), "{written}");
    assert!(
        !written.contains("PRIVATE KEY"),
        "no private key is written"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn switches_ca_out_with_no_path_is_refused() {
    // ⛔ A switch that takes a path and was given none is refused before the
    // bind, so the failure names the switch rather than arriving as a run that
    // wrote its authority nowhere.
    let output = Command::new(harness_bin())
        .args(["--ca-out"])
        .output()
        .expect("the harness command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--ca-out needs a path"), "{stderr}");
}

#[test]
fn switches_until_h2_is_accepted_and_reaches_the_run() {
    // ⭐ It used to be refused beside `--ca-out`. Reaching HTTP/2 over
    // cleartext with prior knowledge is what made it implementable, and a
    // switch is a property of the COMMAND: this drives the compiled binary
    // rather than the library, because a flag documented, parsed and never
    // passed through is the shape this project has already been bitten by.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\n\r\n".to_vec();
    let (out, err, code) = run_with(
        &["--plain", "--until-h2", "--handshakes", "1", "--json"],
        request,
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("\"http2\":null"), "{out}");
}

#[test]
fn switches_an_unknown_argument_is_refused_with_the_usage() {
    let output = Command::new(harness_bin())
        .args(["--invented"])
        .output()
        .expect("the harness command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown argument"), "{stderr}");
    assert!(stderr.contains("usage:"), "{stderr}");
}

#[test]
fn switches_raw_is_the_default_surface() {
    // ⛔ Completing a handshake can change what a client offers, so not
    // completing one is the DEFAULT rather than a mode somebody opts into.
    assert_eq!(Config::default().protocol, Protocol::TlsRaw);
    assert!(!Config::default().header_values);
}

#[test]
fn switches_no_resumption_is_reported_so_a_caller_can_record_it() {
    // ⭐ THE CONDITION IS PRINTED, and that is the point of the switch rather
    // than a convenience. `experiments/10-first-profile.sh` reads this line back
    // into `captured.resumption`, so a profile records the configuration the run
    // actually had rather than the one the script asked for. A cold hello looks
    // the same under either policy, so nothing in the bytes could contradict a
    // wrong claim.
    //
    // ⚠ THE DEADLINE IS LOAD-BEARING, for the same reason the `--ca-out` test
    // above carries one: the command binds and blocks on an accept nobody makes.
    let dir = std::env::temp_dir().join(format!("b-ids-switches-nores-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let ca = dir.join("ca.pem");
    let (_out, err, code) = run_with(
        &[
            "--ca-out",
            &ca.display().to_string(),
            "--no-resumption",
            "--once",
            "--run-timeout-ms",
            "1500",
        ],
        Vec::new(),
    );

    // ⚠ 1, not 0: the run accepted nothing, so it reports a shortfall. What this
    // test asserts is the SWITCH and the line it prints.
    assert_eq!(code, 1, "{err}");
    assert!(
        err.contains("resumption: refused"),
        "the condition is reported on stderr: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn switches_the_default_reports_resumption_offered() {
    // ⛔ THE DEFAULT IS REPORTED TOO, and that is not symmetry for its own sake.
    // A line printed only when the switch was passed would let a caller record
    // nothing for the standing configuration, and "not recorded" and "offered"
    // are different facts.
    let dir = std::env::temp_dir().join(format!("b-ids-switches-res-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let ca = dir.join("ca.pem");
    let (_out, err, code) = run_with(
        &[
            "--ca-out",
            &ca.display().to_string(),
            "--once",
            "--run-timeout-ms",
            "1500",
        ],
        Vec::new(),
    );
    assert_eq!(code, 1, "{err}");
    assert!(
        err.contains("resumption: offered"),
        "the standing configuration is reported too: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn switches_no_resumption_without_ca_out_is_refused() {
    // ⛔ REFUSED rather than ignored. Only `--ca-out` builds a terminator, and
    // resumption is a property of one, so the switch on any other surface is a
    // flag no code reads: the "a setting or flag that no code reads" row of
    // docs/conventions/forbidden-patterns.md. A caller asking for a condition it
    // will not get finds out here rather than from a profile recording one it
    // did not have.
    let output = Command::new(harness_bin())
        .args(["--raw", "--no-resumption"])
        .output()
        .expect("the harness command runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--no-resumption is a property of the terminated surface"),
        "{stderr}"
    );
}
