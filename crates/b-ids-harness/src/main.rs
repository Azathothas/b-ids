//! The harness command.
//!
//! ⭐ **Every switch exists because something went wrong without it**, and each
//! one below says which. A switch with no such sentence is a switch nobody
//! needs.
//!
//! ⭐ **`--ca-out` is the one switch that changes the surface**, because minting
//! an authority is only useful once the handshake is terminated. It writes the
//! authority a client is told to trust, and ⛔ nothing here tells a client to
//! skip verification: a browser launched with certificate errors ignored is a
//! browser in a different configuration from the one being measured.

use std::io::Write as _;
use std::process::ExitCode;
use std::time::Duration;

use b_ids_harness::{Capture, Config, Oracle, Protocol, parse_bind};

const USAGE: &str = "\
usage: b-ids-harness [SWITCHES]

  --raw                do not terminate TLS, and read the ClientHello only.
                       The default, because completing a handshake can change
                       what a client offers.
  --plain              cleartext: an HTTP/1.1 request, or an HTTP/2 connection
                       preface and the frames behind it, decided by the bytes
                       that arrive. The capture that works when a client cannot
                       be told to trust anything.
  --ca-out PATH        mint a certificate authority for this run and write it
                       here, then TERMINATE the handshake, so a client that
                       trusts it completes a verified one. The only surface
                       that reaches a browser's HTTP/2. It prints the
                       authority public key pin on stderr, which is what a
                       driver passes to trust this one run.
  --bind ADDR          an address to reach a browser that is not on this
                       machine. Refuses a hostname and refuses the unspecified
                       address, by name.
  --port N             the port, or 0 to let the operating system choose.
  --handshakes N       how many connections to accept before exiting. Eight by
                       default, because one handshake is not a sample: anything
                       drawn per connection means one handshake tests one draw.
  --once               stop at the first connection. Wrong for a browser, for
                       the same reason.
  --run-timeout-ms N   how long the whole run may wait for connections. Without
                       it a run whose subject never connects never returns.
  --hello-out PATH     write the raw ClientHello as one hex line. The one
                       artefact that survives every hashing scheme and every
                       parser defect.
  --header-values      record values, not only names. The one switch that can
                       log a credential, so it is off by default.
  --until-h2           stop at the first connection that reached HTTP/2. A
                       browser opens sockets it abandons, and the first one of
                       a navigation has carried no HTTP/2 at all.
  --json               one object per connection on stdout, after one line
                       carrying the base URL.
  --expect-file PATH   compare the run against a committed capture and exit 1
                       on a difference.
  --write-golden PATH  write the capture --expect-file reads.
  --timeout-ms N       how long to wait for bytes on an accepted connection.

exit 0 clean, 1 a comparison failed or a handshake did not complete,
       2 the run could not start.";

struct Args {
    config: Config,
    json: bool,
    ca_out: Option<String>,
    hello_out: Option<String>,
    expect_file: Option<String>,
    write_golden: Option<String>,
}

fn fail(message: &str) -> ExitCode {
    eprintln!("b-ids-harness: {message}");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

#[allow(clippy::too_many_lines)]
fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        config: Config::default(),
        json: false,
        ca_out: None,
        hello_out: None,
        expect_file: None,
        write_golden: None,
    };
    let mut argv = std::env::args().skip(1);
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--raw" => args.config.protocol = Protocol::TlsRaw,
            "--plain" => args.config.protocol = Protocol::Cleartext,
            "--until-h2" => args.config.until_h2 = true,
            "--header-values" => args.config.header_values = true,
            "--json" => args.json = true,
            "--once" => args.config.handshakes = 1,
            "--bind" => {
                let value = argv.next().ok_or("--bind needs an address")?;
                args.config.bind = parse_bind(&value).map_err(|e| e.to_string())?;
            }
            "--port" => {
                let value = argv.next().ok_or("--port needs a number")?;
                args.config.port = value.parse::<u16>().map_err(|e| format!("--port: {e}"))?;
            }
            "--handshakes" => {
                let value = argv.next().ok_or("--handshakes needs a number")?;
                args.config.handshakes = value
                    .parse::<u32>()
                    .map_err(|e| format!("--handshakes: {e}"))?;
                if args.config.handshakes == 0 {
                    return Err("--handshakes 0 would accept nothing".to_owned());
                }
            }
            "--run-timeout-ms" => {
                let value = argv.next().ok_or("--run-timeout-ms needs a number")?;
                let ms = value
                    .parse::<u64>()
                    .map_err(|e| format!("--run-timeout-ms: {e}"))?;
                args.config.run_timeout = Some(Duration::from_millis(ms));
            }
            "--timeout-ms" => {
                let value = argv.next().ok_or("--timeout-ms needs a number")?;
                let ms = value
                    .parse::<u64>()
                    .map_err(|e| format!("--timeout-ms: {e}"))?;
                args.config.read_timeout = Duration::from_millis(ms);
            }
            "--hello-out" => {
                args.hello_out = Some(argv.next().ok_or("--hello-out needs a path")?);
            }
            "--expect-file" => {
                args.expect_file = Some(argv.next().ok_or("--expect-file needs a path")?);
            }
            "--write-golden" => {
                args.write_golden = Some(argv.next().ok_or("--write-golden needs a path")?);
            }
            // ⛔ It selects the surface as well as naming a path. Two switches
            // for one capability would be two ways into one code path.
            "--ca-out" => {
                args.ca_out = Some(argv.next().ok_or("--ca-out needs a path")?);
                args.config.protocol = Protocol::TlsTerminated;
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(args)
}

fn main() -> ExitCode {
    let mut args = match parse_args() {
        Ok(args) => args,
        Err(message) => return fail(&message),
    };

    let bind_note = if args.config.bind.is_loopback() {
        String::new()
    } else {
        format!(" (bound to {}, which is not loopback)", args.config.bind)
    };

    // ⛔ Minted per run and written before the bind, so a caller that cannot
    // write the authority finds out before a browser is pointed anywhere.
    // ⚠ The PRIVATE key is never written: only the authority certificate is,
    // which is the half a client needs to verify.
    if let Some(path) = args.ca_out.clone() {
        let authority = match b_ids_harness::mint(args.config.bind) {
            Ok(authority) => authority,
            Err(why) => return fail(&format!("could not mint an authority: {why}")),
        };
        if let Err(err) = std::fs::write(&path, &authority.ca_pem) {
            return fail(&format!("could not write {path}: {err}"));
        }
        // ⭐ On STDERR, so the stdout contract is untouched: the base URL is
        // still the first line there and every line after it is a capture.
        // ⚠ It is not a failure line. A client that trusts this one key
        // completes a verified handshake without any trust store being
        // changed, and that is a condition of whatever is captured through it.
        eprintln!("pin: {}", authority.spki_pin());
        match authority.server_config() {
            Ok(config) => args.config.terminator = Some(config),
            Err(why) => return fail(&format!("could not build a server configuration: {why}")),
        }
    }

    let oracle = match Oracle::bind(args.config.clone()) {
        Ok(oracle) => oracle,
        Err(err) => return fail(&format!("could not bind: {err}")),
    };
    let base_url = match oracle.base_url() {
        Ok(url) => url,
        Err(err) => return fail(&format!("could not read the bound address: {err}")),
    };

    // ⭐ The base URL first, on its own line, BEFORE the accept blocks. A
    // caller cannot point a browser at a port it has not been told.
    println!("{base_url}{bind_note}");
    let _ = std::io::stdout().flush();

    let captures = match oracle.run() {
        Ok(captures) => captures,
        Err(err) => return fail(&format!("accept failed: {err}")),
    };

    // ⛔ Before anything else reads the captures. A run that did not get what
    // it asked for reports that first, because every number downstream of it
    // is over a smaller sample than the caller believes.
    let sampling = b_ids_harness::summarise(args.config.handshakes, &captures);
    let shortfall = sampling.shortfall();

    if let Some(path) = &args.hello_out {
        let hex_lines: String = captures
            .iter()
            .filter(|c| !c.raw_hex.is_empty())
            .map(|c| format!("{}\n", c.raw_hex))
            .collect();
        if let Err(err) = std::fs::write(path, hex_lines) {
            return fail(&format!("could not write {path}: {err}"));
        }
    }

    if args.json {
        for capture in &captures {
            match serde_json::to_string(capture) {
                Ok(line) => println!("{line}"),
                Err(err) => return fail(&format!("could not serialise a capture: {err}")),
            }
        }
    } else {
        for capture in &captures {
            print_capture(capture);
        }
    }

    if let Some(path) = &args.write_golden {
        match serde_json::to_string_pretty(&normalise(&captures)) {
            Ok(text) => {
                if let Err(err) = std::fs::write(path, format!("{text}\n")) {
                    return fail(&format!("could not write {path}: {err}"));
                }
            }
            Err(err) => return fail(&format!("could not serialise the golden: {err}")),
        }
    }

    if let Some(path) = &args.expect_file {
        let expected = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) => return fail(&format!("could not read {path}: {err}")),
        };
        let produced = match serde_json::to_string_pretty(&normalise(&captures)) {
            Ok(text) => text,
            Err(err) => return fail(&format!("could not serialise this run: {err}")),
        };
        if produced.trim() != expected.trim() {
            eprintln!("b-ids-harness: this run does not match {path}");
            return ExitCode::from(1);
        }
        println!("matches {path}");
    }

    if !args.json {
        println!(
            "sampling: {} of {} handshake(s) completed, {} distinct GREASE draw(s), \
             {} distinct extension order(s)",
            sampling.completed,
            sampling.requested,
            sampling.distinct_grease_draws,
            sampling.distinct_extension_orders
        );
    }

    // ⛔ A run where six of eight completed is a run that reports six, not a
    // run that reports success. It names BOTH numbers, because "some
    // handshakes failed" is a sentence nobody can act on.
    if let Some(why) = shortfall {
        eprintln!("b-ids-harness: {why}");
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

/// Drop the fields that change between two correct runs.
///
/// ⚠ The peer PORT is chosen by the operating system, so a golden that carried
/// it would fail on every run. Comparing the answer rather than the transcript
/// is the same rule the twin comparison already follows.
fn normalise(captures: &[Capture]) -> Vec<Capture> {
    captures
        .iter()
        .map(|c| Capture {
            peer: "REDACTED".to_owned(),
            ..c.clone()
        })
        .collect()
}

fn print_capture(capture: &Capture) {
    println!(
        "connection {} from {}, {} byte(s), {:?}",
        capture.connection, capture.peer, capture.bytes_read, capture.protocol
    );
    if let Some(tls) = &capture.tls {
        println!(
            "  tls: {} cipher suite(s), {} extension(s), {} GREASE slot(s)",
            tls.cipher_suites.len(),
            tls.extensions.len(),
            tls.grease.values.len()
        );
    }
    if let Some(termination) = &capture.termination {
        // ⚠ Printed as CONDITIONS of the capture. Only the selected protocol is
        // a choice the peer made; the version and the suite are this server
        // and the browser agreeing, and a reader has to be able to tell.
        println!(
            "  terminated: alpn {:?}, version {:?}, suite {:?}, {} plaintext byte(s)",
            termination.alpn,
            termination.version,
            termination.cipher_suite,
            termination.plaintext_bytes
        );
    }
    if let Some(line) = &capture.request_line {
        println!("  http: {line}");
        println!("  headers: {}", capture.header_names.join(", "));
    }
    if let Some(http2) = &capture.http2 {
        println!(
            "  http2: {} frame(s), settings {:?}, window increment {:?}",
            http2.frames.len(),
            http2.half.settings().unwrap_or_default(),
            http2.half.window_size_increment()
        );
        // ⛔ The parsed block AND the five raw bytes. A rendered string cannot
        // tell a block that was not sent from one that was not read, which is
        // why three published readings of this field disagree.
        println!(
            "  http2 priority block: {:?}, raw {:?}",
            http2.half.stream_priority,
            http2.priority_block_hex()
        );
    }
    for note in &capture.notes {
        println!("  note: {}: {}", note.field, note.why);
    }
}
