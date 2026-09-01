//! The driver command.
//!
//! ⛔ **Exit 0 done, 1 the launch failed, 2 nothing could be run.** A host with
//! no browser has not failed a capture; it cannot take one, and those are
//! different facts.
//!
//! `TODO/driver.md`, `DRIVER-01`.

use std::process::ExitCode;
use std::time::Duration;

use b_ids_driver::{Launch, drive, resolve};

const USAGE: &str = "\
usage: b-ids-driver resolve [--json]
       b-ids-driver drive --url URL [--pin PIN] [--headless] [--timeout-ms N]

  resolve          find a browser on this machine and report its build, with
                   what each source answered and whether they disagreed
  drive            launch it into a throwaway profile, point it at URL, and
                   wait for it with a hard time limit
  --pin PIN        the base64 SHA-256 of the one subject public key to trust.
                   b-ids-harness --ca-out prints it on stderr as `pin: VALUE`.
                   It is not a change to any trust store.
  --headless       run headless. Off by default, because headless changes the
                   product token the browser announces.
  --timeout-ms N   the ceiling on the launch. Sixty seconds by default.
  --json           one object, on stdout.

exit 0 done, 1 the launch failed, 2 nothing could be run.";

fn fail(message: &str) -> ExitCode {
    eprintln!("b-ids-driver: {message}");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let Some(command) = argv.next() else {
        return fail("no command");
    };

    let mut json = false;
    let mut launch = Launch::default();
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--headless" => launch.headless = true,
            "--url" => {
                let Some(value) = argv.next() else {
                    return fail("--url needs a URL");
                };
                launch.url = value;
            }
            "--pin" => {
                let Some(value) = argv.next() else {
                    return fail("--pin needs a value");
                };
                launch.spki_pin = Some(value);
            }
            "--timeout-ms" => {
                let Some(value) = argv.next() else {
                    return fail("--timeout-ms needs a number");
                };
                match value.parse::<u64>() {
                    Ok(ms) => launch.timeout = Duration::from_millis(ms),
                    Err(err) => return fail(&format!("--timeout-ms: {err}")),
                }
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => return fail(&format!("unknown argument: {other}")),
        }
    }

    let browsers = match resolve() {
        Ok(browsers) => browsers,
        Err(why) => {
            // ⛔ 2, not 1. A host with no browser verified nothing.
            eprintln!("b-ids-driver: {why}");
            return ExitCode::from(2);
        }
    };

    match command.as_str() {
        "resolve" => {
            for browser in &browsers {
                if json {
                    match serde_json::to_string(browser) {
                        Ok(line) => println!("{line}"),
                        Err(err) => return fail(&format!("could not serialise: {err}")),
                    }
                } else {
                    println!("{} {}", browser.family, browser.version);
                    for (source, value) in &browser.answers {
                        println!("  {}: {value}", source.as_str());
                    }
                    if browser.disagreement {
                        // ⚠ A finding, not an error. Two sources disagreeing is
                        // how a staged update gets noticed at all.
                        println!("  the sources disagree, and the first one was taken");
                    }
                }
            }
            ExitCode::SUCCESS
        }
        "drive" => {
            let Some(browser) = browsers.first() else {
                return fail("no browser resolved");
            };
            match drive(browser, &launch) {
                Ok(driven) => {
                    println!(
                        "{} {} exited={} elapsed_ms={} profile_removed={}",
                        browser.family,
                        browser.version,
                        driven.exited,
                        driven.elapsed.as_millis(),
                        driven.profile_removed
                    );
                    for switch in &driven.switches {
                        println!("  {switch}");
                    }
                    ExitCode::SUCCESS
                }
                Err(why) => {
                    eprintln!("b-ids-driver: {why}");
                    ExitCode::from(1)
                }
            }
        }
        other => fail(&format!("unknown command: {other}")),
    }
}
