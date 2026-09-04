//! Launch a resolved browser at a URL, into a profile nobody keeps.
//!
//! ⛔ **The profile directory is created per launch and removed after it.** A
//! reused profile carries a session, a cache and a set of decisions from the
//! last run, and a capture taken through one belongs to that history rather
//! than to the build. `DRIVER-01` says so in as many words.
//!
//! ⛔ **The launch has a hard time limit.** A browser that cannot complete a
//! handshake does not exit, and a driver that waited for it would hang with no
//! message and no exit code.
//!
//! ⚠ **The certificate flag names one key, and it is a condition of the
//! capture.** `--ignore-certificate-errors-spki-list` trusts the SHA-256 of one
//! subject public key for one launch. ⛔ It is not
//! `--ignore-certificate-errors`, which switches verification off, and it is
//! not a change to any trust store. `DRIVER-04` is the platform detail.
//!
//! ⚠ **`HARNESS-10` measured the capture SURFACE, not this flag.** Completing
//! the handshake changes nothing the raw surface can see; whether trusting one
//! key differs from trusting a root store is a separate question, and answering
//! it needs a change to the machine's trust store.
//!
//! ⭐ **There is a launch path per engine, and the switches are a table.** The
//! two engines disagree about all three things a launch has to say: where the
//! throwaway profile is, how to ask for no window, and how to be told what to
//! trust. The third is the one with no common answer, so
//! [`Engine::trust_route`] names which route an engine has and a launch that
//! asks for the other one is refused rather than started with an argument the
//! browser reads as a file name. `DRIVER-11`.
//!
//! `TODO/driver.md`, `DRIVER-01`.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use b_ids_schema::Trust;

use crate::resolve::{Family, Resolved};

/// The name the seeded authority carries in a Gecko profile's database.
///
/// ⚠ It is a label rather than an identifier: NSS finds the trust record by
/// issuer and serial number. It is written so that a profile left behind by a
/// failed run says what put the certificate there.
const AUTHORITY_NICKNAME: &str = "b-ids capture authority";

/// How an engine is told what to trust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustRoute {
    /// On the command line, per launch, changing no stored state.
    Switch,
    /// In the profile's own certificate database, which the launch creates.
    ProfileDatabase,
}

impl TrustRoute {
    /// The word a caller reads.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Switch => "switch",
            Self::ProfileDatabase => "profile-database",
        }
    }
}

impl std::fmt::Display for TrustRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which trust route `family` takes.
///
/// ⭐ **Asked of the driver rather than decided by a caller.** A capture script
/// that mapped a family name to a route itself would be a second family list,
/// which is the defect `DRIVER-10` records about the acquisition table: the
/// list the compiler cannot see is the one that goes stale.
#[must_use]
pub fn trust_route(family: Family) -> TrustRoute {
    Engine::for_family(family).trust_route
}

/// The switches one engine takes, as data.
///
/// ⛔ **A table rather than a second arm of a case statement**, for the reason
/// `DRIVER-10` gives about the acquisition route table: adding an engine
/// should be a row and a fixture rather than a branch in a function every
/// other engine also runs through.
#[derive(Debug, Clone, Copy)]
struct Engine {
    /// The switch that names the profile directory.
    profile: &'static str,
    /// Whether the path is joined to that switch with `=` or passed as the
    /// next argument.
    ///
    /// ⚠ Chromium takes `--user-data-dir=PATH` as one argument and Gecko takes
    /// `--profile PATH` as two. Passing either shape to the other engine
    /// leaves it looking for a file.
    profile_joined: bool,
    /// The switch that asks for no window.
    headless: &'static str,
    /// Switches that quiet a first run, in the order they are passed.
    quiet: &'static [&'static str],
    /// Which of the two trust routes this engine has.
    trust_route: TrustRoute,
}

/// Chromium's switches.
const CHROMIUM: Engine = Engine {
    profile: "--user-data-dir",
    profile_joined: true,
    headless: "--headless=new",
    quiet: &[
        "--no-first-run",
        "--no-default-browser-check",
        "--disable-search-engine-choice-screen",
        "--disable-background-networking",
        "--disable-component-update",
    ],
    trust_route: TrustRoute::Switch,
};

/// Gecko's switches.
///
/// ⚠ **Measured from `firefox --help` on 148.0.2, 2026-09-04**, not carried
/// from anywhere. `--new-instance` is there because a second Firefox started
/// while one is running hands its URL to the running one and exits, so a
/// capture would be taken through a profile nobody configured.
const GECKO: Engine = Engine {
    profile: "--profile",
    profile_joined: false,
    headless: "--headless",
    quiet: &["--new-instance"],
    trust_route: TrustRoute::ProfileDatabase,
};

impl Engine {
    /// The engine `family` is launched as.
    fn for_family(family: Family) -> Self {
        if family.is_chromium() {
            CHROMIUM
        } else {
            GECKO
        }
    }

    /// The switches that name `profile`, in order.
    fn profile_switches(self, profile: &Path) -> Vec<String> {
        if self.profile_joined {
            vec![format!("{}={}", self.profile, profile.display())]
        } else {
            vec![self.profile.to_owned(), profile.display().to_string()]
        }
    }
}

/// How often a bounded wait looks again.
///
/// ⚠ Short enough that a run ends promptly and long enough that waiting costs
/// no measurable processor time. The listener uses the same interval for the
/// same reason.
const POLL: Duration = Duration::from_millis(50);

/// How a launch is configured.
#[derive(Debug, Clone)]
pub struct Launch {
    /// Where to point the browser.
    pub url: String,
    /// The base64 SHA-256 of the one subject public key to trust, where the
    /// subject is a run of this project's own harness.
    pub spki_pin: Option<String>,
    /// Whether to run headless.
    ///
    /// ⚠ Headless changes the product token the browser announces, and
    /// `DRIVER-03` is the entry that records the substitution rather than
    /// hiding it. It is off by default because the default should be the
    /// browser a person runs.
    pub headless: bool,
    /// How long the launch may take before it is killed.
    pub timeout: Duration,
    /// Where the browser's own stdout and stderr go.
    ///
    /// ⛔ **`None` DISCARDS THEM, and that has cost a diagnosis.** On
    /// 2026-09-02 the `edge` capture lane launched Edge on a hosted runner, the
    /// browser exited after 1.4 seconds having opened no connection, and the
    /// only thing anybody could read was that it had exited: whatever Edge said
    /// about why went to `Stdio::null()`. `TODO/corpus.md`, `CORPUS-02`.
    ///
    /// ⚠ **A FILE rather than a pipe.** A pipe nobody drains fills, and a
    /// browser that filled it would block on a write while this process waits
    /// for the browser to exit.
    pub log: Option<PathBuf>,
    /// Whether to switch certificate verification off in the subject.
    ///
    /// ⛔ **A CAPTURE TOOL, AND NEVER SOMETHING TO SHIP IN A CLIENT.** It
    /// changes what the browser ACCEPTS after the handshake rather than what it
    /// SENDS, so the hello is unaffected; what it also does is remove every
    /// check the subject would otherwise make, which is why it is off by default
    /// and why `--ca-out` plus a pin is the preferred route.
    ///
    /// ⚠ **It is the way through on the platform where the browser does not
    /// read the trust store a caller can write to.** `docs/inherited-claims.md`
    /// section 8 carries the measurement, and `TODO/driver.md`, `DRIVER-04`, is
    /// the entry that reports which route completes a handshake here.
    ///
    /// ⛔ Refused together with a pin: two trust configurations at once is a
    /// capture whose condition nobody can name.
    pub disable_verification: bool,
    /// The authority to install into the profile's own certificate database,
    /// PEM encoded.
    ///
    /// ⭐ **The only trust route an engine with no certificate switch has.**
    /// Firefox takes no command-line equivalent of
    /// `--ignore-certificate-errors-spki-list`, so a capture against this
    /// project's terminator is arranged by seeding the throwaway profile's
    /// `cert9.db` instead. [`crate::nssdb`] is what writes it and
    /// `DRIVER-11` is the entry.
    ///
    /// ⚠ **It is a trust store, and the profile records it as one.** A
    /// capture taken through it carries `Trust::TrustStore` rather than
    /// `Trust::SpkiPin`, because what the subject trusts is an authority
    /// rather than one key.
    ///
    /// ⛔ Refused together with either of the two above, for the same reason
    /// those two refuse each other.
    pub ca_pem: Option<String>,
}

impl Default for Launch {
    fn default() -> Self {
        Self {
            url: String::new(),
            spki_pin: None,
            headless: false,
            timeout: Duration::from_secs(60),
            log: None,
            disable_verification: false,
            ca_pem: None,
        }
    }
}

impl Launch {
    /// The trust configuration this launch asks for.
    ///
    /// ⛔ **Exactly one, or none.** Two at once is a capture whose condition
    /// nobody can name, and the refusal names which two were asked for.
    ///
    /// # Errors
    ///
    /// A string naming the combination that was refused.
    pub fn trust(&self) -> Result<Trust, String> {
        let asked: Vec<(&str, Trust)> = [
            ("a key pin", Trust::SpkiPin, self.spki_pin.is_some()),
            (
                "an authority in the profile",
                Trust::TrustStore,
                self.ca_pem.is_some(),
            ),
            (
                "verification switched off",
                Trust::VerificationDisabled,
                self.disable_verification,
            ),
        ]
        .into_iter()
        .filter(|(_, _, asked)| *asked)
        .map(|(name, trust, _)| (name, trust))
        .collect();
        match asked.len() {
            0 => Ok(Trust::NotApplicable),
            1 => Ok(asked[0].1),
            _ => Err(format!(
                "a launch has one trust configuration or none, and this one asks for {}: \
                 a capture taken under two at once is one whose condition nobody can name",
                asked
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>()
                    .join(" and ")
            )),
        }
    }
}

/// What one launch did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Driven {
    /// The switches the browser was given, in order.
    ///
    /// ⛔ Recorded, because every one of them is a condition of whatever was
    /// captured through it.
    pub switches: Vec<String>,
    /// Whether the browser exited on its own before the deadline.
    pub exited: bool,
    /// How long the launch lasted.
    pub elapsed: Duration,
    /// Whether the throwaway profile directory was removed afterwards.
    pub profile_removed: bool,
    /// The trust configuration the launch was actually taken under.
    ///
    /// ⛔ **Reported by the launch rather than assumed by its caller.** A
    /// profile records this as `captured.trust`, and a capture whose trust
    /// nobody can name is one this corpus must not publish. `HARNESS-10`.
    pub trust: Trust,
}

/// A profile directory that removes itself.
///
/// ⚠ **A guard rather than a call at the end of the function.** An early return
/// or a panic between the launch and the cleanup would otherwise leave a
/// profile behind, and the rule is that none survives the run.
struct Throwaway {
    path: PathBuf,
}

impl Drop for Throwaway {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Launch `browser` at `launch.url`, and wait for it with a deadline.
///
/// # Errors
///
/// A string naming what the launch or the wait refused.
pub fn drive(browser: &Resolved, launch: &Launch) -> Result<Driven, String> {
    if launch.url.is_empty() {
        return Err("a launch needs a URL".to_owned());
    }
    let engine = Engine::for_family(browser.family);
    let trust = launch.trust()?;
    // ⛔ A TRUST CONFIGURATION AN ENGINE HAS NO ROUTE FOR IS REFUSED, not
    // passed and hoped over. Firefox reads an unknown argument as a file to
    // open, so `--ignore-certificate-errors-spki-list` there navigates
    // somewhere nobody asked for and the capture is of the wrong thing.
    match (engine.trust_route, trust) {
        (TrustRoute::Switch, Trust::TrustStore) => {
            return Err(format!(
                "{} takes its trust on the command line, and an authority in the profile is \
                 the route for an engine that does not. Pass a key pin instead.",
                browser.family
            ));
        }
        (TrustRoute::ProfileDatabase, Trust::SpkiPin | Trust::VerificationDisabled) => {
            return Err(format!(
                "{} takes no certificate switch at all, so a key pin and switching \
                 verification off are both unreachable there. Pass the authority instead, \
                 which is installed into the profile this launch creates.",
                browser.family
            ));
        }
        _ => {}
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let profile = std::env::temp_dir().join(format!(
        "b-ids-driver-{}-{}-{stamp}",
        browser.family,
        std::process::id()
    ));
    std::fs::create_dir_all(&profile).map_err(|e| format!("{}: {e}", profile.display()))?;
    let throwaway = Throwaway {
        path: profile.clone(),
    };

    let mut switches = engine.profile_switches(&profile);
    switches.extend(engine.quiet.iter().map(|s| (*s).to_owned()));
    if launch.headless {
        switches.push(engine.headless.to_owned());
    }
    // ⛔ SEEDED BEFORE THE LAUNCH AND INTO THE DIRECTORY CREATED ABOVE. NSS
    // reads the certificate database when the profile is opened, so a database
    // written after the browser started is one the browser has already decided
    // it did not have.
    if let Some(pem) = &launch.ca_pem {
        crate::nssdb::seed(&profile, pem, AUTHORITY_NICKNAME)?;
    }
    if let Some(pin) = &launch.spki_pin {
        switches.push(format!("--ignore-certificate-errors-spki-list={pin}"));
    }
    if launch.disable_verification {
        // ⚠ BOTH FLAGS, and the second is not decoration. Chromium ignores
        // the first on a branded build unless the run is marked as a test run,
        // which is the shape of the measurement in
        // `docs/inherited-claims.md` section 8.
        switches.push("--ignore-certificate-errors".to_owned());
        switches.push("--test-type".to_owned());
    }
    // ⛔ The URL is LAST and it is a positional argument. A switch that takes
    // the URL as its value is a mode, and passing one is how a launch ends up
    // navigating and then sitting there.
    switches.push(launch.url.clone());

    // ⛔ OPENED BEFORE THE SPAWN, so a path that cannot be written is a refusal
    // rather than a launch whose output went nowhere while a caller believed it
    // was being recorded.
    let (out, err) = match &launch.log {
        None => (Stdio::null(), Stdio::null()),
        Some(path) => {
            let file =
                std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let second = file
                .try_clone()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            (Stdio::from(file), Stdio::from(second))
        }
    };
    let started = Instant::now();
    let mut child = Command::new(&browser.path)
        .args(&switches)
        .stdout(out)
        .stderr(err)
        .spawn()
        .map_err(|e| format!("{}: {e}", browser.path.display()))?;

    let exited = wait_within(&mut child, launch.timeout)?;
    let elapsed = started.elapsed();
    if !exited {
        // ⚠ Killed rather than left. A browser pointed at a harness that has
        // already stopped listening does not exit on its own.
        let _ = child.kill();
        let _ = child.wait();
    }

    let path = throwaway.path.clone();
    drop(throwaway);
    let profile_removed = remove_profile(&path);
    Ok(Driven {
        switches,
        exited,
        elapsed,
        profile_removed,
        trust,
    })
}

/// How long a removal keeps trying after the browser was killed.
///
/// ⚠ **A ceiling on a retry, not a wait.** A browser that exited on its own
/// releases its profile before the first attempt and the loop ends there.
const REMOVE_DEADLINE: Duration = Duration::from_secs(5);

/// Remove the throwaway profile, retrying while a handle is still open on it.
///
/// ⛔ **A killed browser has not finished exiting.** Windows refuses to delete
/// a file that any process still has open, and a browser that puts each tab in
/// its own process still has those closing when the one this driver killed has
/// gone. Measured 2026-09-04 on Firefox 148.0.2: a launch killed at its
/// deadline left the whole profile behind, and the guard's single attempt
/// reported `profile_removed=false` correctly and left the directory on the
/// machine.
///
/// ⚠ The return value is measured, not assumed: it is whether the directory is
/// gone, read from the filesystem after the last attempt.
fn remove_profile(path: &Path) -> bool {
    let deadline = Instant::now() + REMOVE_DEADLINE;
    loop {
        if !path.exists() {
            return true;
        }
        let _ = std::fs::remove_dir_all(path);
        if !path.exists() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(POLL);
    }
}

/// Wait for `child`, giving up at `timeout`.
///
/// ⛔ **It blocks on the child rather than on a clock it guessed.** The deadline
/// is a ceiling, not a schedule.
fn wait_within(child: &mut Child, timeout: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(true),
            Ok(None) => {
                if Instant::now() >= deadline {
                    return Ok(false);
                }
                std::thread::sleep(POLL);
            }
            Err(err) => return Err(format!("waiting on the browser: {err}")),
        }
    }
}
