//! The profile itself, and the four keys its identity is made of.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::id::check_version;
use crate::{
    Defect, PlatformToken, ProfileId, Provenance, SCHEMA_ID, http::HttpHalf, http2::Http2Half,
    tls::TlsHalf,
};

/// A release channel.
///
/// ⛔ `latest` is not one of these. It means stable and nothing else, and a
/// channel field that could hold it would be a field with two spellings for one
/// value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    /// The channel most people run.
    Stable,
    /// The channel ahead of stable, and how this project gets ahead of it.
    Beta,
    /// Developer channel.
    Dev,
    /// Nightly builds.
    Nightly,
    /// Canary builds.
    Canary,
    /// Extended support release.
    Esr,
}

impl Channel {
    /// Every channel, in the order the vocabulary is written down.
    #[must_use]
    pub fn all() -> [Self; 6] {
        [
            Self::Stable,
            Self::Beta,
            Self::Dev,
            Self::Nightly,
            Self::Canary,
            Self::Esr,
        ]
    }

    /// The word as it is written in a profile.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Dev => "dev",
            Self::Nightly => "nightly",
            Self::Canary => "canary",
            Self::Esr => "esr",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// macOS.
    Mac,
    /// Android.
    Android,
    /// iOS.
    Ios,
}

impl Os {
    /// Every operating system, in the order the vocabulary is written down.
    #[must_use]
    pub fn all() -> [Self; 5] {
        [
            Self::Linux,
            Self::Windows,
            Self::Mac,
            Self::Android,
            Self::Ios,
        ]
    }

    /// The word as it is written in a profile.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Mac => "mac",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which browser, at which exact build, in which channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Browser {
    /// The browser's name, as its vendor spells it.
    pub name: String,
    /// ⛔ The EXACT build, never a major alone. Two builds of one major have
    /// sent different bytes, and a major cannot say which produced a value.
    pub version: String,
    /// The major, carried beside the version so a consumer filtering by major
    /// does not have to parse.
    ///
    /// ⚠ Checked against `version` rather than trusted: a value in two places
    /// with no check between them drifts.
    pub major: u32,
    /// The release channel.
    pub channel: Channel,
    /// Whether this build carries its vendor's own entry in its brand list.
    ///
    /// ⚠ An unbranded build cannot produce a branded profile. Chrome for
    /// Testing builds are unbranded, and they are the ones automation reaches
    /// for first.
    pub branded: bool,
}

/// Which machine the capture was taken on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Platform {
    /// The operating system.
    pub os: Os,
    /// The architecture, as the machine reports it.
    pub arch: String,
    /// The distribution or release, where it is knowable.
    ///
    /// ⚠ Recorded because a root store is a per-distribution fact, and the
    /// trust-anchors extension carries a snapshot of it.
    pub distribution: Option<String>,
}

/// When the capture was taken, by what, and how.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Captured {
    /// ⛔ The instant, ISO 8601 UTC, and never optional. A capture with no
    /// instant cannot be ordered against the build it claims to describe.
    pub at: String,
    /// How the browser was run: `container`, `host`, `vm`.
    pub method: String,
    /// The harness that read the bytes, with its version.
    pub harness: String,
    /// Who or what took it.
    pub operator: String,
}

/// The derived digests, siblings of the measured halves.
///
/// ⛔ Derived, and visibly so. ⭐ None of them is a key: a profile is never
/// derived from a digest and nothing round-trips through one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digests {
    /// JA3, an MD5 over `ClientHello` fields in wire order, GREASE stripped.
    ///
    /// ⚠ Unstable per connection for any browser that shuffles, which is what
    /// makes it unfit to assert on.
    pub ja3: Option<String>,
    /// JA4.
    pub ja4: Option<String>,
    /// JA4's raw form.
    pub ja4_r: Option<String>,
    /// JA4's order-preserving raw form.
    pub ja4_ro: Option<String>,
    /// The rendered Akamai fingerprint.
    pub akamai: Option<String>,
}

/// The bytes, kept because a capture is a moment that cannot be retaken.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Raw {
    /// The whole `ClientHello`, hex-encoded.
    pub client_hello_hex: Option<String>,
    /// The SETTINGS frame, hex-encoded.
    pub settings_frame_hex: Option<String>,
}

/// One browser, one build, one platform, one channel, one instant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// The schema this profile is written against.
    pub schema: String,
    /// The identifier, derived from the four keys.
    pub id: ProfileId,
    /// Which browser, at which build, in which channel.
    pub browser: Browser,
    /// Which machine.
    pub platform: Platform,
    /// When, by what, and how.
    pub captured: Captured,
    /// The TLS half.
    pub tls: TlsHalf,
    /// The HTTP/2 half.
    pub http2: Http2Half,
    /// The HTTP half.
    pub http: HttpHalf,
    /// Derived digests, sibling to the measured halves.
    pub digests: Digests,
    /// The raw bytes, sibling to the measured halves.
    pub raw: Raw,
    /// Per-field provenance.
    pub provenance: Provenance,
    /// The profile this one replaces, where it replaces one.
    ///
    /// ⛔ A published profile is immutable. A correction is a NEW profile naming
    /// the one it replaces, never an edit of the old one.
    pub supersedes: Option<String>,
}

impl Profile {
    /// The platform token this profile's identifier is built from.
    #[must_use]
    pub fn platform_token(&self) -> PlatformToken {
        PlatformToken::derive(self.platform.os, &self.platform.arch)
    }

    /// The identifier the four keys derive to, whatever `id` says.
    #[must_use]
    pub fn derived_id(&self) -> ProfileId {
        ProfileId::derive(
            &self.browser.name,
            &self.browser.version,
            &self.platform_token(),
            self.browser.channel,
        )
    }

    /// Every way this profile is malformed on its own terms.
    ///
    /// ⛔ Every defect, not the first, so one pass names everything a caller has
    /// to fix.
    ///
    /// ⚠ This is not the coherence question. Whether a well-formed profile
    /// could have come from a real browser belongs to `b-ids-validator`.
    ///
    /// # Errors
    ///
    /// Returns every [`Defect`] found. An empty vector means well-formed.
    #[must_use]
    pub fn check(&self) -> Vec<Defect> {
        let mut defects = Vec::new();

        if self.schema != SCHEMA_ID {
            defects.push(Defect::FieldMalformed {
                field: "schema".to_owned(),
                why: format!("expected {SCHEMA_ID}, found {}", self.schema),
            });
        }

        if let Err(defect) = check_version("browser.version", &self.browser.version) {
            defects.push(defect);
        } else {
            let declared_major = self
                .browser
                .version
                .split('.')
                .next()
                .and_then(|m| m.parse::<u32>().ok());
            if declared_major != Some(self.browser.major) {
                defects.push(Defect::FieldMalformed {
                    field: "browser.major".to_owned(),
                    why: format!(
                        "{} does not match the major in browser.version {}",
                        self.browser.major, self.browser.version
                    ),
                });
            }
        }

        if self.browser.name.trim().is_empty() {
            defects.push(Defect::FieldMissing {
                field: "browser.name".to_owned(),
            });
        }
        if self.platform.arch.trim().is_empty() {
            defects.push(Defect::FieldMissing {
                field: "platform.arch".to_owned(),
            });
        }

        // ⛔ Never optional, and checked for content rather than presence: an
        // empty string deserialises fine and orders against nothing.
        if self.captured.at.trim().is_empty() {
            defects.push(Defect::FieldMissing {
                field: "captured.at".to_owned(),
            });
        } else if let Err(why) = check_instant(&self.captured.at) {
            defects.push(Defect::FieldMalformed {
                field: "captured.at".to_owned(),
                why,
            });
        }
        if self.captured.harness.trim().is_empty() {
            defects.push(Defect::FieldMissing {
                field: "captured.harness".to_owned(),
            });
        }

        let derived = self.derived_id();
        if derived != self.id {
            defects.push(Defect::IdMismatch {
                declared: self.id.to_string(),
                derived: derived.to_string(),
            });
        }

        if self.http.variants.is_empty() {
            defects.push(Defect::FieldMissing {
                field: "http.variants".to_owned(),
            });
        }

        defects.extend(self.http2.check_units());
        defects.extend(self.refused_fields());
        defects.extend(self.provenance.check());
        defects
    }

    /// The two classes of value that look like identity and are not.
    ///
    /// ⛔ **A digest is derived and connection state is learned**, and storing
    /// either as identity makes a profile that changes for reasons nothing in
    /// the corpus can explain. `SCHEMA-07`.
    #[must_use]
    pub fn refused_fields(&self) -> Vec<Defect> {
        let mut defects = Vec::new();

        // ⛔ Never key on a digest. Nothing round-trips through one.
        let id = self.id.as_str();
        for (name, value) in [
            ("ja3", &self.digests.ja3),
            ("ja4", &self.digests.ja4),
            ("ja4_r", &self.digests.ja4_r),
            ("ja4_ro", &self.digests.ja4_ro),
            ("akamai", &self.digests.akamai),
        ] {
            if let Some(value) = value
                && !value.is_empty()
                && value == id
            {
                defects.push(Defect::DigestUsedAsIdentity {
                    field: format!("digests.{name}"),
                });
            }
        }

        // ⛔ THE THIRD DOOR INTO THE CREDENTIAL RULE, and it was open.
        //
        // `HeaderSet::record` filters at capture time and the harness filters
        // on its own path, and both were tested. DESERIALISATION is neither:
        // serde builds a `HeaderField` field by field, so a profile read from
        // disk could carry a cookie header that no capture would have
        // produced. Found by the door sweep at the end of the session that
        // wrote all three.
        //
        // ⚠ A capture-time filter cannot hold a rule about a FILE. This is the
        // gate on the read path, and it is why the rule is checked rather than
        // only enforced where the bytes are first seen.
        for set in &self.http.variants {
            for field in &set.headers {
                if crate::http::is_never_recorded(&field.name) {
                    defects.push(Defect::ConnectionStateInIdentity {
                        field: format!("http.variants.{}.{}", set.variant, field.name),
                        what: "a credential header, which no capture records".to_owned(),
                    });
                }
            }
        }

        // ⚠ PRESENCE of these codepoints is identity: a browser sends
        // session_ticket empty on a cold connection, and the extension being
        // there at all is part of the fingerprint. ⛔ Their CONTENTS are
        // connection state, and that is what is refused.
        for (codepoint, what) in [
            (0x0023_u16, "a session ticket"),
            (0x0029_u16, "a pre-shared key"),
        ] {
            for extension in self
                .tls
                .extensions
                .iter()
                .filter(|e| e.codepoint == codepoint && !e.body_hex.is_empty())
            {
                defects.push(Defect::ConnectionStateInIdentity {
                    field: format!("tls.extensions.0x{:04x}", extension.codepoint),
                    what: what.to_owned(),
                });
            }
        }

        defects
    }

    /// Whether this profile carries any field copied from somebody else's
    /// table.
    ///
    /// ⭐ A profile with any is a draft, whatever else is true of it.
    #[must_use]
    pub fn is_draft(&self) -> bool {
        !self.provenance.vendor_fields().is_empty()
    }
}

/// An ISO 8601 UTC instant, to the shape `captured.at` requires.
///
/// ⚠ Shape rather than calendar. This refuses `2026-08-30 03:53:11`, which no
/// consumer can sort against an ISO column, and it does not refuse February
/// the thirtieth. A date library would refuse both, and it is not worth a
/// dependency in the one field a capture always fills in from a clock.
fn check_instant(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let digits_at = |i: usize| bytes.get(i).is_some_and(u8::is_ascii_digit);
    let char_at = |i: usize, c: u8| bytes.get(i) == Some(&c);

    let shaped = bytes.len() >= 20
        && (0..4).all(digits_at)
        && char_at(4, b'-')
        && (5..7).all(digits_at)
        && char_at(7, b'-')
        && (8..10).all(digits_at)
        && char_at(10, b'T')
        && (11..13).all(digits_at)
        && char_at(13, b':')
        && (14..16).all(digits_at)
        && char_at(16, b':')
        && (17..19).all(digits_at)
        && bytes.last() == Some(&b'Z');

    if shaped {
        Ok(())
    } else {
        Err(format!(
            "{value} is not an ISO 8601 UTC instant of the form 2026-08-30T03:53:11Z"
        ))
    }
}
