//! Find a browser on this machine, and work out which build it is.
//!
//! ⭐ **Resolving and driving are two jobs and this file is only the first.**
//! A resolver that also launched would be a component with two reasons to fail,
//! and the one that matters here is that the build is read rather than assumed.
//!
//! ⛔ **The build is read from more than one place and a disagreement is a
//! finding.** A version taken from one source is a version nobody checked, and
//! `DRIVER-02` is the entry that says why: a first-party endpoint answered with
//! a build almost nobody was running.
//!
//! ⚠ **It finds what is installed. It does not acquire anything.** `DRIVER-05`
//! is acquisition, and a resolver that downloaded a browser would change the
//! machine it was asked to describe.
//!
//! `TODO/driver.md`, `DRIVER-01`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

/// A browser family this resolver knows how to look for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    /// Google Chrome.
    Chrome,
    /// Microsoft Edge.
    Edge,
}

impl Family {
    /// Every family, in the order they are reported.
    #[must_use]
    pub fn all() -> [Self; 2] {
        [Self::Chrome, Self::Edge]
    }

    /// The family's name, as a report prints it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Edge => "edge",
        }
    }
}

impl core::fmt::Display for Family {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where a version string came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// A version-shaped directory beside the executable, which is how the
    /// Chromium installers lay a build out on Windows.
    SiblingDirectory,
    /// The executable's own `--version` output.
    ///
    /// ⛔ **Not asked on Windows at all, and that was measured here.** Running
    /// `chrome.exe --version` on Windows does not print a version and exit: it
    /// LAUNCHES THE BROWSER, into the person's own profile, and never returns.
    /// A resolver that waited for it hangs, and one that killed it has still
    /// opened somebody's browser to read a number that is in a directory name.
    VersionFlag,
}

impl Source {
    /// The source's name, as a report prints it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SiblingDirectory => "sibling-directory",
            Self::VersionFlag => "version-flag",
        }
    }
}

/// One browser found on this machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolved {
    /// Which family it is.
    pub family: Family,
    /// Where the executable is.
    pub path: PathBuf,
    /// The build, as the sources agreed on it.
    pub version: String,
    /// What each source answered, in the order they were asked.
    ///
    /// ⛔ Kept whatever the answer is, so a reader can check the choice rather
    /// than take it.
    pub answers: Vec<(Source, String)>,
    /// Whether two sources answered and disagreed.
    ///
    /// ⚠ A finding rather than an error. It is how a stale install beside a
    /// newer one gets noticed at all.
    pub disagreement: bool,
}

/// Why nothing could be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotResolved {
    /// The paths that were looked at.
    pub looked_at: Vec<PathBuf>,
}

impl core::fmt::Display for NotResolved {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "no browser found. Looked at {} path(s)",
            self.looked_at.len()
        )
    }
}

/// The places a browser is installed, per platform.
///
/// ⚠ **Read from the environment rather than written out**, because a program
/// files directory is not the same string on every Windows install and a home
/// directory is nobody's to hardcode.
fn candidates(family: Family) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let tail: &[&str] = match family {
        Family::Chrome => &["Google/Chrome/Application/chrome.exe"],
        Family::Edge => &["Microsoft/Edge/Application/msedge.exe"],
    };
    for key in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        let Ok(root) = std::env::var(key) else {
            continue;
        };
        for suffix in tail {
            out.push(Path::new(&root).join(suffix));
        }
    }
    // ⚠ The POSIX names, so this compiles and runs to a useful answer on a host
    // that is not Windows rather than reporting that nothing exists.
    let posix: &[&str] = match family {
        Family::Chrome => &[
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/opt/google/chrome/chrome",
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        ],
        Family::Edge => &[
            "/usr/bin/microsoft-edge",
            "/opt/microsoft/msedge/msedge",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ],
    };
    out.extend(posix.iter().map(PathBuf::from));
    out
}

/// Whether a directory name looks like a build.
fn version_shaped(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// The build from a version-shaped directory beside the executable.
fn from_sibling(path: &Path) -> Option<String> {
    let dir = path.parent()?;
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| version_shaped(n))
        .collect();
    // ⚠ Several can be present while an update is staged. The highest is the
    // one that runs, and sorting numerically rather than as text is why this is
    // a comparison of parsed components.
    found.sort_by_key(|n| {
        n.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect::<Vec<u64>>()
    });
    found.pop()
}

/// The build from the executable's own `--version`.
///
/// ⛔ **Windows is excluded by platform rather than by timeout.** Measured on
/// 2026-09-01: `chrome.exe --version` opened the browser in the operator's own
/// profile and did not exit. A bounded wait would have stopped the hang and
/// not the side effect, and a resolver must not touch the machine it is
/// describing. The sibling directory answers on that platform anyway.
fn from_flag(path: &Path) -> Option<String> {
    if cfg!(windows) {
        return None;
    }
    let output = Command::new(path).arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.split_whitespace()
        .find(|token| version_shaped(token))
        .map(str::to_owned)
}

/// Find every browser this resolver knows how to look for.
///
/// # Errors
///
/// [`NotResolved`], naming how many paths were examined. ⛔ It is a "could not
/// run" rather than a failure: a host with no browser has not failed a capture,
/// it cannot take one.
pub fn resolve() -> Result<Vec<Resolved>, NotResolved> {
    let mut out = Vec::new();
    let mut looked_at = Vec::new();
    for family in Family::all() {
        for path in candidates(family) {
            looked_at.push(path.clone());
            if !path.is_file() {
                continue;
            }
            let mut answers = Vec::new();
            if let Some(version) = from_sibling(&path) {
                answers.push((Source::SiblingDirectory, version));
            }
            if let Some(version) = from_flag(&path) {
                answers.push((Source::VersionFlag, version));
            }
            let Some((_, version)) = answers.first().cloned() else {
                // ⛔ An executable no source could version is reported as
                // nothing rather than as a browser with an unknown build. A
                // capture whose subject cannot be named is not a capture.
                continue;
            };
            let disagreement = answers.iter().any(|(_, v)| *v != version);
            out.push(Resolved {
                family,
                path,
                version,
                answers,
                disagreement,
            });
            // ⚠ One per family. A second install of one family is a different
            // entry: DRIVER-06 is branded against unbranded builds.
            break;
        }
    }
    if out.is_empty() {
        return Err(NotResolved { looked_at });
    }
    Ok(out)
}
