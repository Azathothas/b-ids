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
//! `TODO/driver.md`, `DRIVER-01`.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::resolve::Resolved;

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

    let mut switches = vec![
        format!("--user-data-dir={}", profile.display()),
        "--no-first-run".to_owned(),
        "--no-default-browser-check".to_owned(),
        "--disable-search-engine-choice-screen".to_owned(),
        "--disable-background-networking".to_owned(),
        "--disable-component-update".to_owned(),
    ];
    if launch.headless {
        switches.push("--headless=new".to_owned());
    }
    if launch.spki_pin.is_some() && launch.disable_verification {
        return Err(
            "a launch trusts one key or it verifies nothing, never both: two trust \
             configurations at once is a capture whose condition nobody can name"
                .to_owned(),
        );
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
    Ok(Driven {
        switches,
        exited,
        elapsed,
        profile_removed: !path.exists(),
    })
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
