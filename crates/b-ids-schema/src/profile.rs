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

/// How the subject came to trust the harness that measured it.
///
/// ⛔ **A condition of the measurement, not a detail of the run.** A capture
/// taken through a handshake the subject completed only because it was told to
/// trust one key is a capture taken under a configuration no ordinary browser
/// is in, and a corpus that cannot say which profile was taken under which
/// cannot answer whether the configuration changed the answer. `HARNESS-10` is
/// the entry that measures the difference, and it has nothing to compare
/// across unless every profile records this.
///
/// ⚠ **An enum rather than free text, and an unknown value is REFUSED rather
/// than read.** This field exists to be compared across profiles, and a
/// comparison over free text fails silently on a spelling. A profile written by
/// a later version under a trust configuration this reader has no name for must
/// fail loudly, because reading it as one of these would be reporting a
/// condition that was not the condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Trust {
    /// No handshake was completed, so nothing had to be trusted.
    ///
    /// ⚠ This is the raw and the cleartext surfaces, where the question does
    /// not arise. It is not "the subject trusted nothing".
    NotApplicable,
    /// The subject was given the SHA-256 of one subject public key, for one
    /// launch, and verified against it.
    ///
    /// ⛔ Not the same as verification being switched off, and not the same as
    /// a trusted root. No trust store was changed.
    SpkiPin,
    /// The harness authority was installed in a trust store the subject reads.
    TrustStore,
    /// Verification was switched off in the subject.
    ///
    /// ⛔ This changes the SUBJECT rather than the condition, and no profile in
    /// this corpus is taken this way. The variant exists so that a capture
    /// taken that way can be labelled honestly rather than mislabelled as one
    /// of the others.
    VerificationDisabled,
}

impl Trust {
    /// The value a profile written before the field existed reads back as.
    fn not_applicable() -> Self {
        Self::NotApplicable
    }

    /// Every trust configuration, in the order the vocabulary is written down.
    #[must_use]
    pub fn all() -> [Self; 4] {
        [
            Self::NotApplicable,
            Self::SpkiPin,
            Self::TrustStore,
            Self::VerificationDisabled,
        ]
    }

    /// The word as it is written in a profile.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not-applicable",
            Self::SpkiPin => "spki-pin",
            Self::TrustStore => "trust-store",
            Self::VerificationDisabled => "verification-disabled",
        }
    }
}

impl fmt::Display for Trust {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether the harness offered the subject a way to resume a session.
///
/// ⛔ **A condition of the measurement, and it decides whether a cold
/// handshake is obtainable at all.** A server that issues session tickets lets
/// the subject resume, and a resumed hello is a different hello: it offers a
/// pre-shared key where a cold one offers an empty session ticket. WARN
/// **Measured on hosted runners 2026-09-02**, over two independent runs of
/// `capture.yml`: Chrome on `ubuntu-latest` abandoned both of its first two
/// connections after the handshake and every later one resumed, so the
/// navigation produced NO cold connection and nothing could be published from
/// it. `TODO/corpus.md`, `CORPUS-02`.
///
/// ⚠ **An enum rather than a boolean, and absent rather than defaulted.** A
/// profile written before this field existed did not record the condition, and
/// reading one as `offered` would be a condition nobody measured reported as
/// one somebody did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Resumption {
    /// The harness issued session tickets, so the subject could resume.
    ///
    /// ⚠ This is the standing configuration and it is what every profile
    /// taken before 2026-09-02 was captured under.
    Offered,
    /// The harness issued no session tickets, so every hello is a cold one.
    ///
    /// ⭐ **It removes the resumed connections from the sample rather than
    /// changing what a cold hello looks like.** A subject with no ticket for an
    /// origin sends the hello a fresh client sends, which is the one this
    /// corpus publishes.
    Refused,
}

impl Resumption {
    /// Every resumption configuration, in the order the vocabulary is written
    /// down.
    #[must_use]
    pub fn all() -> [Self; 2] {
        [Self::Offered, Self::Refused]
    }

    /// The word as it is written in a profile.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Offered => "offered",
            Self::Refused => "refused",
        }
    }
}

impl fmt::Display for Resumption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
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
    /// How the subject came to trust the harness.
    ///
    /// ⚠ Defaulted on the way in so a profile written before this field existed
    /// still reads, and then REFUSED by [`Profile::check`] unless the surface
    /// says the question does not arise. A silent `not-applicable` on a
    /// terminated capture would be a condition nobody recorded reading as a
    /// condition somebody did.
    #[serde(default = "Trust::not_applicable")]
    pub trust: Trust,
    /// Whether the harness offered the subject a way to resume a session.
    ///
    /// ⛔ **Absent where it was not recorded, never defaulted.** A profile
    /// written before the field existed did not measure the condition, and
    /// `None` says exactly that. ⭐ Every profile written since carries it,
    /// read from what the harness reported rather than typed beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumption: Option<Resumption>,
    /// The switches the subject was launched with, in order.
    ///
    /// ⛔ Every one of them is a condition of what was captured through it.
    /// ⚠ Empty where the subject was not launched by this project's driver,
    /// which is a different fact from a launch with no switches; the driver
    /// always passes at least the throwaway profile directory.
    #[serde(default)]
    pub switches: Vec<String>,
    /// Where the build came from, when this project fetched it.
    ///
    /// ⭐ **The digest is what makes an acquisition reproducible after the
    /// artefact stops being served.** Every download URL will one day 404, and
    /// a later reader still needs to be able to say whether two captures used
    /// the same bytes. `TODO/driver.md`, `DRIVER-05`.
    ///
    /// ⚠ **Absent where nothing was fetched**, which is a different fact from
    /// an acquisition that failed: a build already installed on the machine was
    /// not obtained by this project and has no route or digest to record.
    ///
    /// ⛔ **This one field is omitted from the serialised form when absent**,
    /// which nothing else here does, and the reason is the corpus rather than
    /// taste: it is append-only, so a profile published before this field
    /// existed has to keep serialising exactly as it was published.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquisition: Option<Acquisition>,
}

/// The routes a build can come from, as [`Profile::check`] accepts them.
///
/// ⛔ **Kept identical to the published schema's enum and to
/// `b_ids_driver::acquire::Route`.** Three copies of one list is two chances
/// for it to drift, and the two that can be compared are compared: a profile
/// carrying a route outside this list is refused here, and the schema refuses
/// it to a consumer.
pub const ACQUISITION_ROUTES: [&str; 3] = ["installed", "cache", "chrome-for-testing"];

/// Where a build came from, and the digest of what arrived.
///
/// ⛔ **The URL is recorded and the artefact never is.** This project publishes
/// measurements, versions, digests and where a build was fetched from; the
/// binary is the vendor's to serve. `TODO/driver.md`, `DRIVER-05`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acquisition {
    /// Which route answered: `installed`, `cache` or `chrome-for-testing`.
    pub route: String,
    /// The URL it answered from, where there was one.
    ///
    /// ⚠ `None` for a route that asks the machine rather than a URL.
    pub url: Option<String>,
    /// The digest of what arrived, lowercase hex.
    pub sha256: String,
    /// How many bytes arrived.
    pub bytes: usize,
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
///
/// ⛔ **Everything the wire carried, and it is the backstop against this
/// project's own parser being wrong**, which it will be. A field dropped
/// because nobody could imagine a consumer is a field nobody can recover: the
/// build will be gone, the download will stop being served, and the machine
/// will be reimaged.
///
/// ⭐ **A profile is rebuildable from this block alone.** That is asserted by a
/// test rather than intended, and it is what makes the raw block a backstop
/// rather than a gesture.
///
/// ⚠ **The schema is additive.** Fields are added, never removed and never
/// repurposed. Removing one is a new major, and a new major is a promise to
/// keep serving the old one.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Raw {
    /// The whole `ClientHello`, hex-encoded.
    ///
    /// ⭐ The one artefact in which a GREASE question is answerable at all.
    /// Every digest, JA4_ro included, strips GREASE before it is computed.
    pub client_hello_hex: Option<String>,
    /// The SETTINGS frame, hex-encoded.
    ///
    /// ⚠ Kept for the profiles written before `http2_frames_hex` existed. It
    /// is the first entry of that list wherever both are present, and
    /// [`Raw::check`] asserts they agree.
    pub settings_frame_hex: Option<String>,
    /// Every HTTP/2 frame the connection opened with, in arrival order,
    /// hex-encoded, head and payload together.
    ///
    /// ⛔ Every frame, including a frame type this project has no name for. A
    /// sequence that silently omits one is a sequence nobody can compare.
    #[serde(default)]
    pub http2_frames_hex: Vec<String>,
    /// The bytes of the HTTP/1.1 request line, exactly.
    ///
    /// ⚠ The BYTES rather than the text. A request line is not guaranteed to
    /// be UTF-8 and a capture that stored it as text could not reproduce one
    /// that was not.
    #[serde(default)]
    pub request_line_hex: Option<String>,
    /// The whole first message of the connection, hex-encoded, before anything
    /// was made of it.
    ///
    /// ⭐ The widest backstop there is: whatever the parser got wrong, this is
    /// what it read.
    #[serde(default)]
    pub connection_hex: Option<String>,
    /// What the TLS record layer itself carried.
    #[serde(default)]
    pub record_layer: Option<RecordLayer>,
}

/// The TLS record layer, which is a fingerprint surface of its own.
///
/// ⚠ **Its version is not the handshake's version and not the negotiated
/// one.** Three quantities called "the version" live in one hello, and a
/// profile that carried only one of them cannot say which.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordLayer {
    /// The version in the record header.
    pub version: u16,
    /// The length the record header declared.
    pub declared_length: u16,
    /// ⛔ How many bytes actually arrived after the header. Counting what
    /// arrived rather than trusting what was declared is the rule; recording
    /// both is what makes a disagreement visible instead of repaired.
    pub bytes_arrived: usize,
    /// Whether the hello arrived spread over more than one record.
    ///
    /// ⚠ A client that fragments its hello is a client that stands out, and a
    /// reassembling parser loses the fact unless it is recorded here.
    pub fragmented: bool,
}

impl Raw {
    /// Whether this block carries enough to rebuild anything at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.client_hello_hex.is_none()
            && self.settings_frame_hex.is_none()
            && self.http2_frames_hex.is_empty()
            && self.request_line_hex.is_none()
            && self.connection_hex.is_none()
    }

    /// Every place two fields of this block disagree.
    ///
    /// ⛔ **A value in two places needs a check that they agree**, or the copy
    /// a reader trusts is the wrong one.
    #[must_use]
    pub fn check(&self) -> Vec<crate::Defect> {
        let mut defects = Vec::new();
        if let (Some(settings), Some(first)) = (
            self.settings_frame_hex.as_ref(),
            self.http2_frames_hex.first(),
        ) && settings != first
        {
            defects.push(crate::Defect::FieldMalformed {
                field: "raw.settings_frame_hex".to_owned(),
                why: format!(
                    "is {settings} and the first recorded frame is {first}. They are one frame in \
                     two fields and the older one is kept only for profiles written before the \
                     list existed"
                ),
            });
        }
        // ⛔ THE CREDENTIAL RULE REACHES THE RAW BLOCK, and it did not until
        // this check existed. `SCHEMA-04` says a capture carries no `cookie`
        // and no `authorization`; `SCHEMA-07` says the raw bytes are never
        // edited. On a CLEARTEXT surface those two rules collide: the parsed
        // fields drop the credential and the bytes beside them still spell it
        // out, hex-encoded, where a grep for the plaintext finds nothing.
        //
        // ⛔ The profile is REFUSED rather than repaired. Editing the bytes
        // would destroy the one artefact that survives every parser defect,
        // and dropping them silently is the failure this whole project is
        // about. Fail loud; the operator decides what to do with the capture.
        //
        // ⚠ Only the cleartext fields are scanned. A `ClientHello` carries no
        // header lines, and scanning it would be a rule firing on entropy.
        for (field, hex) in [
            ("raw.connection_hex", self.connection_hex.as_ref()),
            ("raw.request_line_hex", self.request_line_hex.as_ref()),
        ] {
            if let Some(hex) = hex
                && let Some(name) = credential_header_in_hex(hex)
            {
                defects.push(crate::Defect::FieldMalformed {
                    field: field.to_owned(),
                    why: format!(
                        "carries a {name} header line. A capture records no credential, and these \
                         bytes spell one out even though the parsed fields do not"
                    ),
                });
            }
        }

        if let Some(record) = &self.record_layer
            && let Some(hello) = &self.client_hello_hex
        {
            // ⚠ Two hex digits per byte, and the five-byte record head is part
            // of what `client_hello_hex` carries.
            let bytes = hello.len() / 2;
            let expected = record.bytes_arrived + 5;
            if bytes != expected {
                defects.push(crate::Defect::FieldMalformed {
                    field: "raw.record_layer.bytes_arrived".to_owned(),
                    why: format!(
                        "says {} byte(s) after a five-byte head, and raw.client_hello_hex holds \
                         {bytes}",
                        record.bytes_arrived
                    ),
                });
            }
        }
        defects
    }
}

/// Which credential header a run of hex spells out, if any.
///
/// ⚠ **Case-insensitive, and it looks for the header LINE rather than the
/// word.** HTTP/1.1 does not lower-case its names and HTTP/2 does, so a rule
/// holding one spelling holds nothing on the other wire; and the bare word
/// appears in ordinary text, where a colon after it does not.
fn credential_header_in_hex(hex: &str) -> Option<&'static str> {
    let Ok(bytes) = decode_hex(hex) else {
        return None;
    };
    let text = String::from_utf8_lossy(&bytes).to_ascii_lowercase();
    crate::http::NEVER_RECORDED
        .iter()
        .find(|name| text.contains(&format!("{name}:")))
        .copied()
}

/// Decode a hex run, refusing anything that is not one.
fn decode_hex(text: &str) -> Result<Vec<u8>, ()> {
    if !text.len().is_multiple_of(2) {
        return Err(());
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks(2) {
        let high = char::from(pair[0]).to_digit(16).ok_or(())?;
        let low = char::from(pair[1]).to_digit(16).ok_or(())?;
        out.push(u8::try_from(high * 16 + low).map_err(|_| ())?);
    }
    Ok(out)
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

        // ⛔ AN ACQUISITION IS CHECKED WHERE IT IS PRESENT, and it was not until
        // the door sweep asked who reads this field. The published schema
        // constrains the route to an enum and the object to four fields;
        // nothing on this side did, so a profile could claim a route no driver
        // can produce and a digest that is not one, and every check in the tree
        // would have passed it. `TODO/driver.md`, `DRIVER-05`.
        //
        // ⚠ ABSENT IS CORRECT AND IS NOT CHECKED. A build already installed on
        // the machine was not obtained by this project and has no route.
        if let Some(acquisition) = &self.captured.acquisition {
            if !ACQUISITION_ROUTES.contains(&acquisition.route.as_str()) {
                defects.push(Defect::FieldMalformed {
                    field: "captured.acquisition.route".to_owned(),
                    why: format!(
                        "is {}, and the routes a build can come from are {}",
                        acquisition.route,
                        ACQUISITION_ROUTES.join(", ")
                    ),
                });
            }
            // ⛔ The same shape the corpus index uses for every published file:
            // 64 lower-case hex. A digest that is not one is a value nobody can
            // compare against anything.
            if acquisition.sha256.len() != 64
                || !acquisition
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                defects.push(Defect::FieldMalformed {
                    field: "captured.acquisition.sha256".to_owned(),
                    why: format!(
                        "is {:?}, and a digest is 64 lower-case hex characters",
                        acquisition.sha256
                    ),
                });
            }
        }

        // ⛔ A CONDITION NOBODY RECORDED MUST NOT READ AS A CONDITION SOMEBODY
        // DID. `captured.trust` defaults on the way in so a profile written
        // before the field existed still deserialises, and that default is
        // exactly the value a terminated capture may not carry: HTTP/2 frames
        // that arrived after a `ClientHello` arrived INSIDE the session that
        // hello opened, so something had to have trusted the harness for them
        // to exist at all. `HARNESS-10` compares profiles on this field, and a
        // silent default here is what would make that comparison meaningless.
        if !self.tls.cipher_suites.is_empty()
            && !self.http2.frames.is_empty()
            && self.captured.trust == Trust::NotApplicable
        {
            defects.push(Defect::FieldMalformed {
                field: "captured.trust".to_owned(),
                why: "is not-applicable on a profile carrying both a ClientHello and HTTP/2 \
                      frames. Those frames arrived inside the session that hello opened, so the \
                      handshake completed and something had to trust this harness"
                    .to_owned(),
            });
        }

        // ⛔ The raw block's own internal agreement. It is the backstop against
        // this parser being wrong, so a backstop that disagrees with itself is
        // worse than none.
        defects.extend(self.raw.check());

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
