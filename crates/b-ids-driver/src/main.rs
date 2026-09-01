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
       b-ids-driver versions [--channel CH] [--json]

  resolve          find a browser on this machine and report its build, with
                   what each source answered and whether they disagreed
  versions         ask the vendor which build is SERVING on a channel, which
                   during a staged rollout is not the highest build it knows.
                   It names every source, what each answered, the chosen build
                   with its rollout fraction, the highest build and ITS
                   fraction, and any disagreement.
  --channel CH     the release channel to ask about. stable by default.
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

/// Ask the vendor what is serving on a channel.
///
/// ⛔ **Exit 0 when at least one source answered, 1 when none did.** A run
/// during somebody else's outage has degraded rather than found a problem, and
/// one that reached nothing has verified nothing.
fn versions(channel: &str, json: bool) -> ExitCode {
    let report = b_ids_driver::discover(channel);
    if json {
        match serde_json::to_string(&report) {
            Ok(line) => println!("{line}"),
            Err(err) => return fail(&format!("could not serialise: {err}")),
        }
    } else {
        for answer in &report.answers {
            match (&answer.version, &answer.error) {
                (Some(version), _) => println!("  {}: {version}", answer.source.as_str()),
                (None, Some(why)) => println!("  {}: {why}", answer.source.as_str()),
                (None, None) => println!("  {}: no answer and no error", answer.source.as_str()),
            }
        }
        match &report.chosen {
            Some(chosen) => {
                println!("chosen {} fraction {:?}", chosen.version, chosen.fraction);
                // ⭐ Beside the answer, always, so a reader can check the
                // choice rather than take it. Where these differ, a rollout is
                // in progress and the naive query would have said the second.
                println!(
                    "highest known {} fraction {:?}",
                    chosen.highest_known, chosen.highest_fraction
                );
            }
            None => println!("no source named a build"),
        }
        if report.disagreement {
            // ⚠ A finding, not an error. Two first-party sources disagreeing
            // is how the defect this command exists for was found.
            println!("the sources disagree, and neither is preferred");
        }
    }
    if report.answered() {
        ExitCode::SUCCESS
    } else {
        eprintln!("b-ids-driver: no source answered, so nothing was discovered");
        ExitCode::from(1)
    }
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let Some(command) = argv.next() else {
        return fail("no command");
    };

    let mut json = false;
    let mut channel = "stable".to_owned();
    let mut launch = Launch::default();
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--headless" => launch.headless = true,
            "--channel" => {
                let Some(value) = argv.next() else {
                    return fail("--channel needs a value");
                };
                channel = value;
            }
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

    // ⛔ BEFORE the resolver runs, because this command asks the vendor rather
    // than the machine. A host with no browser installed can still be told what
    // is serving, and routing it through a resolver that refuses would report a
    // fact about the host as a fact about the channel.
    if command == "versions" {
        return versions(&channel, json);
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
