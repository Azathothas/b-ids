//! The smallest client, as a command.
//!
//! ⛔ **Four things and nothing else**: which profile, where to send it, and
//! the two exit codes that say what happened. `docs/history/todo/library.md`, `LIB-02`.

use std::net::ToSocketAddrs;
use std::process::ExitCode;

const USAGE: &str = "\
usage: b-ids-cli --profile ID --url URL

  --profile ID   a profile identifier from the embedded corpus, spelled as the
                 corpus spells it, for example
                 chrome-152.0.7977.75-linux64-stable. ⛔ A profile this project
                 has not captured is refused by name and never substituted.
  --url URL      where to send it, as https://HOST:PORT/. ⚠ The path is ignored:
                 this client sends a ClientHello and stops, because completing a
                 handshake needs a TLS state machine that could not emit this
                 hello in the first place.
  --list         print every profile the embedded corpus holds, and exit.
  --matrix       print the support matrix as JSON and exit: one cell per profile
                 produced by RUNNING this project own emitter, and one hole row
                 per stack this tree cannot run, each carrying the file and the
                 line it was read at. ⛔ A cell is a run and a hole is a
                 reading, and the two are different kinds rather than two
                 colours of one. docs/history/todo/emitters.md, EMIT-01.

⭐ What it proves: the bytes a profile describes can be put on a wire. Point it
   at `b-ids-harness --raw` and the harness reads back the same profile, field
   by field, which is the acceptance in docs/history/todo/library.md.

exit 0 sent, 1 refused, 2 the arguments could not be read.";

fn fail(message: &str) -> ExitCode {
    eprintln!("b-ids-cli: {message}");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

/// The host and port of an `https://HOST:PORT/` URL.
///
/// ⚠ **Deliberately not a URL parser.** This client takes one shape and refuses
/// anything else by name, which is smaller than a dependency and says more when
/// it goes wrong.
fn address(url: &str) -> Result<String, String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| format!("{url} does not start with https:// or http://"))?;
    let host_port = rest.split('/').next().unwrap_or(rest);
    if !host_port.contains(':') {
        return Err(format!(
            "{url} names no port, and this client does not assume one"
        ));
    }
    Ok(host_port.to_owned())
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let mut profile = None;
    let mut url = None;
    let mut list = false;
    let mut matrix = false;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--profile" => match argv.next() {
                Some(value) => profile = Some(value),
                None => return fail("--profile needs an identifier"),
            },
            "--url" => match argv.next() {
                Some(value) => url = Some(value),
                None => return fail("--url needs a URL"),
            },
            "--list" => list = true,
            "--matrix" => matrix = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => return fail(&format!("unknown argument: {other}")),
        }
    }

    if matrix {
        let built = b_ids_emit::support_matrix(b_ids::profiles());
        match serde_json::to_string_pretty(&built) {
            Ok(text) => println!("{text}"),
            Err(why) => return fail(&format!("serialising the matrix: {why}")),
        }
        return ExitCode::SUCCESS;
    }

    if list {
        for held in b_ids::profiles() {
            println!("{}", held.id);
        }
        println!(
            "b-ids-cli=list profiles:{} release:{}",
            b_ids::profiles().len(),
            b_ids::release().identifier
        );
        return ExitCode::SUCCESS;
    }

    let (Some(profile), Some(url)) = (profile, url) else {
        return fail("both --profile and --url are needed");
    };
    let host_port = match address(&url) {
        Ok(value) => value,
        Err(why) => return fail(&why),
    };
    let Ok(mut peers) = host_port.to_socket_addrs() else {
        return fail(&format!("{host_port} does not resolve"));
    };
    let Some(peer) = peers.next() else {
        return fail(&format!("{host_port} resolves to nothing"));
    };

    match b_ids_cli::send(&profile, peer) {
        Ok(sent) => {
            println!(
                "b-ids-cli=sent profile:{} bytes:{} peer:{}",
                sent.profile, sent.bytes, sent.peer
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("b-ids-cli: {why}");
            ExitCode::from(1)
        }
    }
}
