//! The driver command.
//!
//! ⛔ **Exit 0 done, 1 the launch failed, 2 nothing could be run.** A host with
//! no browser has not failed a capture; it cannot take one, and those are
//! different facts.
//!
//! `TODO/driver.md`, `DRIVER-01`.

use std::process::ExitCode;
use std::time::Duration;

use b_ids_driver::acquire::{Platform, Route, download_url};
use b_ids_driver::{Family, Launch, drive, plan, resolve};

const USAGE: &str = "\
usage: b-ids-driver resolve [--browser NAME] [--json]
       b-ids-driver drive --url URL [--browser NAME] [--pin PIN] [--headless]
                          [--timeout-ms N] [--log PATH] [--disable-verification]
       b-ids-driver versions [--channel CH] [--json]
       b-ids-driver acquire --browser NAME --version V --index PATH
                            [--platform P] [--json]

  resolve          find a browser on this machine and report its build, with
                   what each source answered and whether they disagreed
  versions         ask the vendor which build is SERVING on a channel, which
                   during a staged rollout is not the highest build it knows.
                   It names every source, what each answered, the chosen build
                   with its rollout fraction, the highest build and ITS
                   fraction, and any disagreement.
  --channel CH     the release channel to ask about. stable by default.
  acquire          read the automation-build index and print the archive URL
                   for one exact build on one platform. ⛔ IT FETCHES
                   NOTHING and touches no machine: it reads an index a caller
                   already has and names a URL. ⚠ The index publishes a
                   SUBSET of builds, so a build the vendor shipped may not be
                   in it, and the refusal says so and names the nearest.
  --index PATH     the automation-build index, as JSON, already on disk.
  --index-url      print the URL that index is served at and exit, so a caller
                   can fetch it with whatever tool the platform has. ⛔ IT IS
                   ASKED FOR RATHER THAN SPELLED TWICE: a fetcher carrying its
                   own copy is a value in two places with no check.
  --version V      the exact build to look up. Required: the index is keyed
                   by build and there is no nearest-match.
  --platform P     which archive: linux64, win64, win32, mac-arm64, mac-x64.
                   The host's own platform by default. ⚠ These are the
                   INDEX's spellings and the corpus spells one of them
                   differently.
  drive            launch it into a throwaway profile, point it at URL, and
                   wait for it with a hard time limit
  --pin PIN        the base64 SHA-256 of the one subject public key to trust.
                   b-ids-harness --ca-out prints it on stderr as `pin: VALUE`.
                   It is not a change to any trust store.
  --browser NAME   which family to resolve or drive: chrome or edge. Without
                   it, the first family that resolved is taken, which is the
                   order the resolver reports. ⛔ A NAME THAT IS NOT A
                   FAMILY IS REFUSED, and a family this machine does not have
                   exits 2: no browser here, and the capture failed, are
                   different facts.
  --headless       run headless. Off by default, because headless changes the
                   product token the browser announces.
  --timeout-ms N   the ceiling on the launch. Sixty seconds by default.
  --disable-verification
                   switch certificate verification OFF in the subject,
                   with --test-type so a branded build honours it.⛔ A
                   CAPTURE TOOL AND NEVER SOMETHING TO SHIP IN A CLIENT: it
                   changes what the browser ACCEPTS after the handshake
                   rather than what it SENDS. Refused together with
                   --pin, and --ca-out plus a pin is the preferred route.
  --log PATH       write the browser's own stdout and stderr here. Without
                   it they are discarded, and a launch that captured
                   nothing then carries no word from the browser about
                   why.
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

/// Name the archive one exact build is published at, from an index on disk.
///
/// ⛔ **It reads and it does not fetch.** Keeping the network out of this
/// command is what lets `provision-browser` fetch with the one tool a platform
/// already has, and what lets this be tested without a day the network agrees.
///
/// ⛔ **Exit 1 is the index answering no, and exit 2 is not being able to
/// ask.** A build the index does not publish is a fact about the vendor's
/// catalogue; a missing index file is a fact about this run, and a caller that
/// could not tell them apart would retry the wrong one.
fn acquire(
    family: Option<Family>,
    version: Option<&str>,
    platform: Option<Platform>,
    index: Option<&std::path::Path>,
    index_url: bool,
    json: bool,
) -> ExitCode {
    let Some(family) = family else {
        return fail(&format!(
            "acquire needs --browser. It knows {}",
            Family::names()
        ));
    };
    // ⛔ ASKED FOR RATHER THAN SPELLED TWICE. A caller has to FETCH the index
    // before it can be read, and a fetcher that carried its own copy of the URL
    // would be a value in two places with no check that they agree. The version
    // is not needed to name the index, so it is not required here.
    if index_url {
        let Some(candidate) = plan(family, Some("0.0.0.0"))
            .into_iter()
            .find(|c| c.route == Route::ChromeForTesting)
        else {
            eprintln!(
                "b-ids-driver: there is no automation-build route for {family}, so there is \
                 no index to fetch."
            );
            return ExitCode::from(2);
        };
        match candidate.url {
            Some(url) => {
                println!("{url}");
                return ExitCode::SUCCESS;
            }
            // ⛔ Unreachable through `plan` today and refused rather than
            // unwrapped: a route offered with no URL is a defect in the plan,
            // and a panic here would report it as a crash.
            None => return fail("the automation-build route was planned with no index URL"),
        }
    }
    let Some(version) = version else {
        return fail("acquire needs --version: the index is keyed by build");
    };
    let Some(index) = index else {
        return fail("acquire needs --index, the automation index as JSON on disk");
    };

    // ⛔ THE PLAN DECIDES WHETHER THE ROUTE EXISTS, rather than this command
    // knowing which families have an automation index. A second answer here
    // would be a copy of `plan`'s branch with nothing checking that they agree.
    let Some(candidate) = plan(family, Some(version))
        .into_iter()
        .find(|c| c.route == Route::ChromeForTesting)
    else {
        eprintln!(
            "b-ids-driver: there is no automation-build route for {family}. \
             It is the one route that serves an exact build, and it is Chrome only."
        );
        return ExitCode::from(2);
    };

    let Some(platform) = platform.or_else(Platform::host) else {
        return fail(&format!(
            "acquire could not name this host's platform, so --platform is needed. \
             The index publishes {}",
            Platform::names()
        ));
    };

    let text = match std::fs::read_to_string(index) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("b-ids-driver: could not read {}: {err}", index.display());
            return ExitCode::from(2);
        }
    };

    match download_url(&text, version, platform) {
        Ok(url) => {
            if json {
                // ⛔ SERIALISED, never formatted. A URL carrying a character
                // that has to be escaped would otherwise emit JSON that does
                // not parse.
                let object = serde_json::json!({
                    "schema": "acquire/1",
                    "route": candidate.route.as_str(),
                    "index": candidate.url,
                    "browser": family.as_str(),
                    "version": version,
                    "platform": platform.as_str(),
                    "url": url,
                });
                match serde_json::to_string(&object) {
                    Ok(line) => println!("{line}"),
                    Err(err) => return fail(&format!("could not serialise: {err}")),
                }
            } else {
                println!("{url}");
            }
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("b-ids-driver: {refusal}");
            ExitCode::from(1)
        }
    }
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let Some(command) = argv.next() else {
        return fail("no command");
    };

    let mut json = false;
    // ⛔ NONE IS "the first that resolved", never "chrome". A default family
    // written here would be a second place the resolver's order is decided.
    let mut wanted: Option<Family> = None;
    let mut channel = "stable".to_owned();
    let mut version: Option<String> = None;
    let mut index: Option<std::path::PathBuf> = None;
    let mut index_url = false;
    // ⛔ NONE IS "this host's platform", resolved at the point of use rather
    // than here, so a host the index does not publish for is refused with a
    // message instead of silently taking the nearest.
    let mut platform: Option<Platform> = None;
    let mut launch = Launch::default();
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--headless" => launch.headless = true,
            "--browser" => {
                let Some(value) = argv.next() else {
                    return fail("--browser needs a family");
                };
                let Some(family) = Family::parse(&value) else {
                    return fail(&format!(
                        "--browser {value}: this resolver has no branch for that family. \
                         It knows {}",
                        Family::names()
                    ));
                };
                wanted = Some(family);
            }
            "--channel" => {
                let Some(value) = argv.next() else {
                    return fail("--channel needs a value");
                };
                channel = value;
            }
            "--version" => {
                let Some(value) = argv.next() else {
                    return fail("--version needs a build");
                };
                version = Some(value);
            }
            "--index-url" => index_url = true,
            "--index" => {
                let Some(value) = argv.next() else {
                    return fail("--index needs a path");
                };
                index = Some(std::path::PathBuf::from(value));
            }
            "--platform" => {
                let Some(value) = argv.next() else {
                    return fail("--platform needs a name");
                };
                let Some(parsed) = Platform::parse(&value) else {
                    return fail(&format!(
                        "--platform {value}: the automation index has no branch for that. \
                         It publishes {}",
                        Platform::names()
                    ));
                };
                platform = Some(parsed);
            }
            "--url" => {
                let Some(value) = argv.next() else {
                    return fail("--url needs a URL");
                };
                launch.url = value;
            }
            "--disable-verification" => launch.disable_verification = true,
            "--log" => {
                let Some(value) = argv.next() else {
                    return fail("--log needs a path");
                };
                launch.log = Some(std::path::PathBuf::from(value));
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

    // ⛔ ALSO BEFORE THE RESOLVER, and for a sharper reason than `versions`
    // has. This command is what a provisioning run calls immediately AFTER
    // purging every browser off the machine, so routing it through a resolver
    // that exits 2 when nothing is installed would refuse it exactly when it is
    // needed. TODO/driver.md, DRIVER-08.
    if command == "acquire" {
        return acquire(
            wanted,
            version.as_deref(),
            platform,
            index.as_deref(),
            index_url,
            json,
        );
    }

    let browsers = match resolve() {
        Ok(browsers) => browsers,
        Err(why) => {
            // ⛔ 2, not 1. A host with no browser verified nothing.
            eprintln!("b-ids-driver: {why}");
            return ExitCode::from(2);
        }
    };

    // ⛔ THE FILTER IS APPLIED ONCE, for both commands. A `drive` that chose a
    // family and a `resolve` that reported every one would be two answers to
    // "which browser is this run about", and `experiments/10-first-profile.sh`
    // reads the second to describe what the first captured.
    // ⚠ 2, not 1: a machine without the named family has no browser, which is
    // not a failure of this tree. The capture lane distinguishes them.
    let browsers: Vec<_> = match wanted {
        None => browsers,
        Some(family) => {
            let kept: Vec<_> = browsers
                .into_iter()
                .filter(|b| b.family == family)
                .collect();
            if kept.is_empty() {
                eprintln!("b-ids-driver: {family} did not resolve on this machine");
                return ExitCode::from(2);
            }
            kept
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
