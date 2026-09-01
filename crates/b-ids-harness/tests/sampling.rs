//! HARNESS-08. One handshake is not a sample.
//!
//! The acceptance: a run configured for eight handshakes where the fixture
//! supplies six exits non-zero with a message naming both numbers.
//!
//! ⛔ Every test name starts with `sampling`, because
//! `cargo test -p b-ids-harness sampling` is the entry's acceptance command.

mod support;

use std::io::Write as _;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use b_ids_harness::sampling::{DEFAULT_HANDSHAKES, completed, summarise};
use b_ids_harness::{Config, Protocol};

use support::{feed_within, fixture_bytes, grease_values};

/// The compiled command, which is where the exit code lives.
fn harness_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().expect("the test binary has a path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join(format!("b-ids-harness{}", std::env::consts::EXE_SUFFIX))
}

#[test]
fn sampling_the_default_is_eight_handshakes_and_never_one() {
    // ⛔ Anything drawn per connection means a single handshake tests a single
    // draw. A defect that fires on three values in sixteen reaches a
    // one-handshake check four times in five, and passes.
    assert_eq!(DEFAULT_HANDSHAKES, 8);
    assert_eq!(Config::default().handshakes, 8);
}

#[test]
fn sampling_a_run_that_completed_six_of_eight_says_six_and_eight() {
    // ⛔ The acceptance. A run where six of eight completed is a run that
    // reports six, not a run that reports success, and it names BOTH numbers
    // because "some handshakes failed" is a sentence nobody can act on.
    let mut child = Command::new(harness_bin())
        .args(["--handshakes", "8", "--run-timeout-ms", "1500"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the harness command runs");

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

    // ⚠ SIX connections for a run that asked for eight. The run's deadline is
    // what turns the missing two into a report instead of a hang.
    let bytes = fixture_bytes("client-hello.hex");
    for _ in 0..6 {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("the command is listening");
        stream.write_all(&bytes).expect("the write lands");
        stream.flush().expect("flush");
        thread::sleep(Duration::from_millis(20));
    }

    let status = child.wait().expect("the command exits");
    let out = reader.join().expect("the reader finished");
    let mut err = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut stderr, &mut err);
    }

    assert_eq!(status.code(), Some(1), "stdout: {out}\nstderr: {err}");
    assert!(err.contains('6'), "{err}");
    assert!(err.contains('8'), "{err}");
    assert!(err.contains("completed"), "{err}");
    assert!(out.contains("sampling:"), "{out}");
}

#[test]
fn sampling_a_run_that_completed_every_handshake_exits_zero() {
    // ⚠ The other half. A guard that always fires is a guard nobody keeps.
    let bytes = fixture_bytes("client-hello.hex");
    let captures = feed_within(
        Config {
            handshakes: 3,
            run_timeout: Some(Duration::from_secs(5)),
            ..Config::default()
        },
        vec![bytes.clone(), bytes.clone(), bytes],
        Duration::from_secs(20),
    );
    let sampling = summarise(3, &captures);
    assert_eq!(sampling.accepted, 3);
    assert_eq!(sampling.completed, 3);
    assert!(sampling.every_one_completed());
    assert_eq!(sampling.shortfall(), None);
}

#[test]
fn sampling_an_accepted_connection_is_not_a_completed_one() {
    // ⛔ A browser opens sockets it abandons. Those are accepted, recorded,
    // and useless as a draw, and counting them as completed would report a
    // sample larger than the one that was taken.
    let bytes = fixture_bytes("client-hello.hex");
    let captures = feed_within(
        Config {
            handshakes: 2,
            read_timeout: Duration::from_millis(300),
            run_timeout: Some(Duration::from_secs(5)),
            ..Config::default()
        },
        vec![bytes, Vec::new()],
        Duration::from_secs(20),
    );
    let sampling = summarise(2, &captures);
    assert_eq!(sampling.accepted, 2);
    assert_eq!(sampling.completed, 1);
    assert!(!completed(&captures[1]));
    let why = sampling.shortfall().expect("a shortfall");
    assert!(why.contains("1 of 2"), "{why}");
    assert!(why.contains("2 accepted"), "{why}");
}

#[test]
fn sampling_reports_the_per_draw_variation() {
    // ⭐ Which values were drawn, and how many distinct orders were seen. Two
    // consecutive captures of one binary must produce two different draws, or
    // the capture is wrong, and a run that reported only a count could not say
    // whether it had seen one behaviour or eight.
    let values = grease_values();
    let payloads: Vec<Vec<u8>> = (0..3)
        .map(|i| {
            support::client_hello(&[
                (values[i], Vec::new()),
                (0x002b, vec![0x02, 0x03, 0x04]),
                (values[(i + 1) % values.len()], vec![0x00]),
            ])
        })
        .collect();
    let captures = feed_within(
        Config {
            handshakes: 3,
            run_timeout: Some(Duration::from_secs(5)),
            ..Config::default()
        },
        payloads,
        Duration::from_secs(20),
    );

    let sampling = summarise(3, &captures);
    assert_eq!(sampling.completed, 3);
    assert_eq!(sampling.grease_draws.len(), 3);
    assert_eq!(
        sampling.distinct_grease_draws, 3,
        "three different draws read as {} distinct: {:?}",
        sampling.distinct_grease_draws, sampling.grease_draws
    );
    // ⚠ The extension ORDER is the same in all three; only the values differ.
    // A run that conflated the two would report three orders where there is
    // one, and `SCHEMA-10` rests on the difference.
    assert_eq!(sampling.distinct_extension_orders, 3);
}

#[test]
fn sampling_a_run_with_a_deadline_ends_rather_than_waiting_forever() {
    // ⛔ A hang has no message and no exit code, and in continuous integration
    // it consumes the job's whole timeout and reports nothing about what went
    // wrong.
    let started = std::time::Instant::now();
    let captures = feed_within(
        Config {
            protocol: Protocol::Cleartext,
            handshakes: 8,
            run_timeout: Some(Duration::from_millis(600)),
            ..Config::default()
        },
        vec![b"GET / HTTP/1.1\r\nHost: example\r\n\r\n".to_vec()],
        Duration::from_secs(20),
    );
    assert_eq!(captures.len(), 1);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the run did not honour its deadline"
    );
    assert!(summarise(8, &captures).shortfall().is_some());
}
