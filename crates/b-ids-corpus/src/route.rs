//! Where a profile lives, and where its bytes live beside it.
//!
//! ⛔ **One derivation, and everything that needs a path calls it.** The writer
//! places a file, the verifier walks the tree and asks whether each file is
//! where its own contents say it should be, and `PUB-03` will generate routes
//! from the same function. A second answer to "where does this profile go" is a
//! corpus whose index and whose tree disagree, which is the defect
//! `docs/reference-sweeps/usable.md` section 9 measured in somebody else's
//! published dataset.
//!
//! ⚠ **The path is derived from the profile's own keys, never stored in it.**
//! A stored path is a second copy of the identity, and the copy a reader trusts
//! is the wrong one.

use std::path::PathBuf;

use b_ids_schema::Profile;

/// The corpus layout version, which is part of every published path.
///
/// ⛔ **In the path rather than implied by the reader.** A consumer pins a
/// route; changing the layout under one is how a pin stops meaning anything.
/// A new layout is a new prefix served beside this one, never an edit of it.
pub const LAYOUT: &str = "v1";

/// The directory the profiles live under.
pub const CORPUS_DIR: &str = "corpus";

/// The directory the raw bytes live under.
///
/// ⛔ **A sibling of the corpus rather than a child of it.** Everything under
/// [`CORPUS_DIR`] is a profile, so a consumer fetching the corpus does not
/// fetch megabytes of hex it did not ask for, and a walk of the corpus needs no
/// rule for which files to skip.
pub const RAW_DIR: &str = "raw";

/// Why a profile has no route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoRoute {
    /// Which key was unusable.
    pub field: String,
    /// What is wrong with it.
    pub why: String,
}

impl core::fmt::Display for NoRoute {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.field, self.why)
    }
}

/// Where one profile and its bytes are published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// The profile, as JSON.
    pub profile: PathBuf,
    /// The `ClientHello`, as one hex line with no trailing newline.
    pub hello: PathBuf,
}

/// A path component that cannot escape the corpus root or collide with a
/// directory separator.
///
/// ⛔ **Refused rather than sanitised.** A browser named `../..` is not a
/// browser this project measured, and quietly rewriting it into something
/// path-safe would publish a profile under a name that is not its name. Fail
/// loud; the caller fixes the identity.
///
/// ⚠ Lower-cased, because the corpus is served over routes and two files whose
/// names differ only in case are one file on a case-insensitive filesystem and
/// two over HTTP.
fn component(field: &str, value: &str) -> Result<String, NoRoute> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(NoRoute {
            field: field.to_owned(),
            why: "is empty, and an empty path component would publish this profile at the \
                  directory above it"
                .to_owned(),
        });
    }
    if trimmed == "." || trimmed == ".." {
        return Err(NoRoute {
            field: field.to_owned(),
            why: format!("is {trimmed}, which names a directory rather than a value"),
        });
    }
    if let Some(bad) = trimmed
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')))
    {
        return Err(NoRoute {
            field: field.to_owned(),
            why: format!(
                "carries {bad:?}, and a published path component is ASCII letters, digits, dot, \
                 dash and underscore"
            ),
        });
    }
    Ok(trimmed.to_ascii_lowercase())
}

/// Where this profile is published.
///
/// # Errors
///
/// A [`NoRoute`] naming the key that cannot appear in a path and why.
pub fn route(profile: &Profile) -> Result<Route, NoRoute> {
    let browser = component("browser.name", &profile.browser.name)?;
    let channel = component("browser.channel", profile.browser.channel.as_str())?;
    let platform = component("platform", profile.platform_token().as_str())?;
    let version = component("browser.version", &profile.browser.version)?;

    let leaf = format!("{browser}/{channel}/{platform}");
    Ok(Route {
        profile: PathBuf::from(CORPUS_DIR)
            .join(LAYOUT)
            .join(&leaf)
            .join(format!("{version}.json")),
        hello: PathBuf::from(RAW_DIR)
            .join(LAYOUT)
            .join(&leaf)
            .join(format!("{version}.hello.hex")),
    })
}

/// The key a latest-per-key pointer is keyed by: browser, channel, platform.
///
/// ⚠ **The channel is part of the key rather than resolved away.** A pointer
/// that had to decide what `latest` means across channels would be answering
/// `CORPUS-03`'s question, which is that `latest` means stable and nothing
/// else. Keyed this way it never has to.
///
/// # Errors
///
/// A [`NoRoute`] naming the key that cannot appear in a path and why.
pub fn pointer_key(profile: &Profile) -> Result<String, NoRoute> {
    let browser = component("browser.name", &profile.browser.name)?;
    let channel = component("browser.channel", profile.browser.channel.as_str())?;
    let platform = component("platform", profile.platform_token().as_str())?;
    Ok(format!("{browser}/{channel}/{platform}"))
}

/// The published path as a route reads it: forward slashes on every host.
///
/// ⛔ **Never `Path::display`.** That prints a backslash on Windows, so a corpus
/// written on one host and an index written on another would disagree about
/// every path in it, and the difference would be invisible to a reader looking
/// at two strings that name the same file.
#[must_use]
pub fn as_route(path: &std::path::Path) -> String {
    path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

/// Order two build strings by their numeric components, highest last.
///
/// ⭐ **Re-exported, never re-implemented.** The corpus's latest pointer and the
/// driver's rollout choice ask the same question, and a comparison written twice
/// is two orderings. `b_ids_schema::version_order` is the one implementation and
/// carries the reason.
pub use b_ids_schema::version_order;
