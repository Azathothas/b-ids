//! The profile identifier, which is derived from four keys and is not a hash.
//!
//! ⭐ It exists so a published path can be constructed without an index. A
//! consumer that knows the browser, the build, the platform and the channel can
//! build the route; one that has to look the identifier up needs a second
//! request before it can make the first.
//!
//! ⛔ It is derived, never stored as the source of truth. [`ProfileId::derive`]
//! is the one place it is computed, and a profile whose declared identifier
//! disagrees with its keys is refused rather than trusted.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{Channel, Defect, Os};

/// The platform token that appears inside a profile identifier.
///
/// ⚠ This is NOT `os` plus `arch` joined with a dash. It is the token Chrome
/// for Testing uses in its own download index, so an identifier this project
/// publishes and a build somebody downloads spell the platform the same way.
/// `linux` on `x86_64` is `linux64`, not `linux-x86_64`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlatformToken(String);

impl PlatformToken {
    /// The token for an operating system and an architecture.
    ///
    /// ⚠ An architecture this mapping does not know is joined to the operating
    /// system with a dash rather than refused. A profile for a platform nobody
    /// has published a build for is still a profile, and refusing here would
    /// design a ceiling in the one type every published path goes through.
    #[must_use]
    pub fn derive(os: Os, arch: &str) -> Self {
        let token = match (os, arch) {
            (Os::Linux, "x86_64") => "linux64".to_owned(),
            (Os::Windows, "x86_64") => "win64".to_owned(),
            (Os::Windows, "x86") => "win32".to_owned(),
            (Os::Mac, "x86_64") => "mac-x64".to_owned(),
            (Os::Mac, "aarch64") => "mac-arm64".to_owned(),
            (os, arch) => format!("{os}-{arch}"),
        };
        Self(token)
    }

    /// The token as it appears in an identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlatformToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A profile identifier: browser, exact build, platform token, channel.
///
/// ```text
/// chrome-152.0.7977.64-linux64-stable
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(String);

impl ProfileId {
    /// Build an identifier from the four keys.
    #[must_use]
    pub fn derive(
        browser: &str,
        version: &str,
        platform: &PlatformToken,
        channel: Channel,
    ) -> Self {
        Self(format!(
            "{}-{version}-{platform}-{channel}",
            browser.to_lowercase()
        ))
    }

    /// Take an identifier as written, without deriving it.
    ///
    /// ⚠ Nothing here checks that it agrees with any keys.
    /// [`crate::Profile::check`] is what compares the two.
    #[must_use]
    pub fn from_declared(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The identifier as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Refuse a version string that names a major and nothing else.
///
/// ⛔ The rule `TODO/schema.md` SCHEMA-01 states is that a version alone is not
/// an identity: a profile keyed on "Chrome 152" cannot express that two builds
/// of that major sent different bytes.
///
/// ⚠ The rule is "more than a major", not "exactly four components". Chrome
/// publishes four, Firefox publishes two or three, and requiring four here
/// would be a ceiling built in front of the second browser this project
/// captures.
pub(crate) fn check_version(field: &str, version: &str) -> Result<(), Defect> {
    if version.is_empty() {
        return Err(Defect::FieldMissing {
            field: field.to_owned(),
        });
    }
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return Err(Defect::FieldMalformed {
            field: field.to_owned(),
            why: format!(
                "{version} names a major and no build. A version alone cannot say which build sent the bytes"
            ),
        });
    }
    for part in &parts {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Defect::FieldMalformed {
                field: field.to_owned(),
                why: format!("{version} has a component that is not a number"),
            });
        }
    }
    Ok(())
}
