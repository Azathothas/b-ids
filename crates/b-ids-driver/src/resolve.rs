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
//! ⚠ **It finds what is installed. It does not acquire anything.**
//! [`crate::acquire`] is where a build is obtained, and a resolver that
//! downloaded one would change the machine it was asked to describe.
//! ⭐ The two meet at [`crate::acquire::Route::Installed`], which is this
//! module's answer offered as an acquisition route.
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
    /// Chromium, unbranded.
    ///
    /// ⚠ **The control that separates branding from engine.** Whatever differs
    /// between this and [`Self::Chrome`] on one host is branding, and whatever
    /// does not is the engine. `TODO/corpus.md`, `CORPUS-02`.
    Chromium,
    /// Mozilla Firefox.
    ///
    /// ⭐ **The one family here that is not a Chromium.** A different TLS
    /// stack, a different extension set and order, and different HTTP/2
    /// settings, which is what makes it the highest-value non-Chrome lane.
    Firefox,
}

impl Family {
    /// Every family, in the order they are reported.
    #[must_use]
    pub fn all() -> [Self; 4] {
        [Self::Chrome, Self::Edge, Self::Chromium, Self::Firefox]
    }

    /// The family's name, as a report prints it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Edge => "edge",
            Self::Chromium => "chromium",
            Self::Firefox => "firefox",
        }
    }

    /// Whether the family is a Chromium at heart.
    ///
    /// ⛔ **Asked rather than assumed, because the switches differ.** Every
    /// launch flag this driver passes is a Chromium switch, and Firefox takes
    /// none of them: it spells headless `-headless` and has no equivalent for
    /// most of the rest. A caller that treated all four alike would launch
    /// Firefox with arguments it reads as file names.
    /// `TODO/corpus.md`, `CORPUS-02`.
    #[must_use]
    pub fn is_chromium(self) -> bool {
        match self {
            Self::Chrome | Self::Edge | Self::Chromium => true,
            Self::Firefox => false,
        }
    }

    /// The vendor's own spelling, which is what a profile records.
    ///
    /// ⛔ **Derived here rather than typed where a profile is built.** The
    /// corpus derives a route from `browser.name` by lower-casing it, so a name
    /// somebody typed is a second copy of this value with nothing checking that
    /// the two agree. `TODO/corpus.md`, `CORPUS-02`.
    #[must_use]
    pub fn vendor_name(self) -> &'static str {
        match self {
            Self::Chrome => "Chrome",
            Self::Edge => "Edge",
            Self::Chromium => "Chromium",
            Self::Firefox => "Firefox",
        }
    }

    /// Read a family from the name a caller wrote.
    ///
    /// ⛔ **An unknown name is `None` rather than a default.** A caller naming a
    /// family this resolver has no branch for is asking for something that
    /// cannot be produced, and answering with Chrome would capture one browser
    /// and label it another. `TODO/validator.md`, `VALID-03`, is the check that
    /// says the same thing from the corpus side.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|f| f.as_str() == name)
    }

    /// Every family's name, for a message that has to say what is available.
    #[must_use]
    pub fn names() -> String {
        Self::all()
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(", ")
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
    /// A version-shaped `NAME.manifest` file beside the executable.
    ///
    /// ⛔ **Measured, and it is the only source an automation build has on
    /// Windows.** The archive the automation index serves is FLAT: read from
    /// the central directory of `chrome-win64.zip` for `151.0.7922.76` on
    /// 2026-09-02, `chrome.exe` sits beside `151.0.7922.76.manifest` and there
    /// is no version-shaped DIRECTORY at all. [`Source::SiblingDirectory`]
    /// therefore answers nothing for that layout, and
    /// [`Source::VersionFlag`] is not asked on Windows, so without this an
    /// automation build would resolve as an executable nobody could version
    /// and be skipped. `TODO/driver.md`, `DRIVER-08`.
    ManifestFile,
    /// The executable's own `--version` output.
    ///
    /// ⛔ **Not asked on Windows at all, and that was measured here.** Running
    /// `chrome.exe --version` on Windows does not print a version and exit: it
    /// LAUNCHES THE BROWSER, into the person's own profile, and never returns.
    /// A resolver that waited for it hangs, and one that killed it has still
    /// opened somebody's browser to read a number that is in a directory name.
    VersionFlag,
    /// `Version=` in the `application.ini` beside the executable.
    ///
    /// ⛔ **Firefox's only on-disk source, and neither of the two above
    /// answers for it.** Measured on this host 2026-09-04: the install
    /// directory holds `browser/`, `defaults/`, `fonts/` and `uninstall/` and
    /// NO version-shaped directory, so [`Source::SiblingDirectory`] finds
    /// nothing, and there is no `firefox.manifest`, so
    /// [`Source::ManifestFile`] finds nothing either. `application.ini` states
    /// it:
    ///
    /// ```text
    /// Version=148.0.2
    /// BuildID=20260309125808
    /// ```
    ///
    /// ⚠ A resolver without this source finds the executable, versions it from
    /// nothing, and drops it, which is the "an executable no source could
    /// version is reported as nothing" branch in [`resolve`]. Firefox would
    /// have been invisible while being installed. `TODO/corpus.md`,
    /// `CORPUS-02`.
    ApplicationIni,
}

impl Source {
    /// The source's name, as a report prints it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SiblingDirectory => "sibling-directory",
            Self::ManifestFile => "manifest-file",
            Self::VersionFlag => "version-flag",
            Self::ApplicationIni => "application-ini",
        }
    }
}

/// One browser found on this machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Resolved {
    /// Which family it is.
    pub family: Family,
    /// The vendor's own spelling of the name, which is what a profile records.
    ///
    /// ⛔ **Reported rather than left for a caller to map.**
    /// `experiments/10-first-profile.sh` writes the identity file this ends up
    /// in, and a shell script carrying its own family-to-name table would be
    /// the same value in two places with no check that they agree.
    pub name: &'static str,
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
        Family::Chromium => &["Chromium/Application/chrome.exe"],
        // ⚠ NOT UNDER AN `Application` DIRECTORY, and that is not a typo. The
        // Chromium installers put the executable one level down beside its
        // version directories; Firefox puts it at the top of its own install
        // directory. Measured on this host 2026-09-04.
        Family::Firefox => &["Mozilla Firefox/firefox.exe"],
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
        Family::Chromium => &[
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            // ⚠ The runner image serves Chromium as a snap, which is a
            // different install mechanism and a different sandbox. The path is
            // listed so the resolver can REPORT one that is there; whether a
            // snap can be driven is a separate measurement and DRIVER-10 says
            // so. TODO/driver.md, DRIVER-10.
            "/snap/bin/chromium",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ],
        Family::Firefox => &[
            "/usr/bin/firefox",
            "/usr/lib/firefox/firefox",
            "/snap/bin/firefox",
            "/Applications/Firefox.app/Contents/MacOS/firefox",
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

/// The build from a version-shaped `NAME.manifest` file beside the executable.
///
/// ⛔ **A FILE, where [`from_sibling`] wants a DIRECTORY, and the difference is
/// the whole reason this exists.** The branded installers lay a build out in a
/// version-named directory; the automation archive is flat and names the build
/// in a manifest file instead. A reader that looked for one shape found nothing
/// in the other and reported an executable it could not version.
///
/// ⚠ **Highest wins, for the same reason as [`from_sibling`].** Nothing stops
/// two manifests sitting in one directory after an unpack over an older build,
/// and the comparison is of parsed components rather than of text so that
/// `7922.9` sorts below `7922.76`.
fn from_manifest(path: &Path) -> Option<String> {
    let dir = path.parent()?;
    let mut found: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.strip_suffix(".manifest").map(str::to_owned))
        .filter(|stem| version_shaped(stem))
        .collect();
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

/// What every source answers about the executable at one path, in the order
/// they are asked.
///
/// ⭐ **One reader of "which build is this", and [`resolve`] is a caller rather
/// than a second copy.** Three sources answer three different layouts and the
/// order between them is a decision; a second place that asked two of them
/// would disagree with this one the first time a layout moved.
///
/// ⚠ **Empty means no source could version it**, which [`resolve`] treats as
/// not a browser: a capture whose subject cannot be named is not a capture.
#[must_use]
pub fn sources_for(path: &Path) -> Vec<(Source, String)> {
    let mut answers = Vec::new();
    if let Some(version) = from_sibling(path) {
        answers.push((Source::SiblingDirectory, version));
    }
    // ⚠ After the directory and before the flag, which is the order of
    // decreasing authority for a BRANDED install: a staged update creates the
    // new directory before the manifest beside it moves. For an automation
    // build it is the only source on Windows.
    if let Some(version) = from_manifest(path) {
        answers.push((Source::ManifestFile, version));
    }
    if let Some(version) = from_flag(path) {
        answers.push((Source::VersionFlag, version));
    }
    // ⚠ LAST, AND IT IS THE ONLY ONE THAT ANSWERS FOR FIREFOX. Asked for every
    // path rather than gated on the family, because a source is a property of
    // the layout on disk and gating it would be a second place that has to
    // agree about which family a path belongs to.
    if let Some(version) = from_application_ini(path) {
        answers.push((Source::ApplicationIni, version));
    }
    answers
}

/// The build from `Version=` in the `application.ini` beside the executable.
///
/// ⚠ **The first `Version=` in the file, and the key is matched at the start of
/// a line.** `application.ini` also carries a `[Gecko]` section with its own
/// `MinVersion` and `MaxVersion`, and a substring search finds those first.
fn from_application_ini(path: &Path) -> Option<String> {
    let ini = path.parent()?.join("application.ini");
    let text = std::fs::read_to_string(ini).ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("Version="))
        .map(str::trim)
        .filter(|v| !v.is_empty())
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
            let answers = sources_for(&path);
            let Some((_, version)) = answers.first().cloned() else {
                // ⛔ An executable no source could version is reported as
                // nothing rather than as a browser with an unknown build. A
                // capture whose subject cannot be named is not a capture.
                continue;
            };
            let disagreement = answers.iter().any(|(_, v)| *v != version);
            out.push(Resolved {
                family,
                name: family.vendor_name(),
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
