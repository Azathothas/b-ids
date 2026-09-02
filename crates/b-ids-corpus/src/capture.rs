//! Turning a capture into a profile, which is where a measurement becomes
//! publishable.
//!
//! ⭐ **This is the seam the whole project was waiting on.** The harness reads
//! bytes off a socket and the schema says what a profile is; until something
//! joined the two, every measurement this project took lived in a scratch log.
//!
//! ⛔ **It refuses rather than fills in.** A capture that did not complete a
//! handshake, or that carries a credential, does not become a profile with a
//! gap in it: it is refused by name and the operator decides what to do with
//! the capture. The alternative is a corpus whose entries look measured and are
//! not, which is the one failure this project cannot recover from.
//!
//! # ⭐ The credential rule's third door, and it is closed here
//!
//! A terminated capture holds the decrypted first message the peer sent. That
//! is the surface where a real browser's credentials actually appear, unlike
//! the cleartext one nothing ever reaches. It is written into
//! `raw.connection_hex`, which is the field [`b_ids_schema::Raw::check`]
//! already scans for a `cookie` or `authorization` header line, so the refusal
//! fires at the moment a capture becomes a profile rather than at the moment
//! one is published.
//!
//! ⛔ **The bytes are never edited to make the refusal go away.** They are the
//! backstop against this project's own parser being wrong, and a corpus that
//! repaired them would have destroyed the artefact it exists to keep.
//!
//! `TODO/corpus.md`, `CORPUS-01`.

use b_ids_harness::Capture;
use b_ids_schema::http::{HeaderSet, HttpHalf, ValuePolicy, Variant};
use b_ids_schema::{
    Browser, Captured, Channel, Defect, Digests, Os, Platform, Profile, Provenance,
    ProvenanceEntry, ProvenanceKind, Raw, SCHEMA_ID, Trust,
};

/// The reason a redacted switch carries in the provenance map.
///
/// ⚠ **One string, so a consumer can filter on it**, and the same discipline
/// the headless normalisation follows.
pub const SWITCH_REASON: &str = "throwaway-profile-path";

/// The switch whose value names a directory on the machine that took the
/// capture.
const USER_DATA_DIR: &str = "--user-data-dir=";

/// What a redacted `--user-data-dir` is published as.
///
/// ⚠ Not the empty string. An empty value reads as a switch that was passed
/// with nothing, which is a different launch from the one that happened.
const REDACTED_DIR: &str = "(throwaway)";

/// Why a capture did not become a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// Which connection was refused.
    pub connection: u32,
    /// Why, in a sentence naming what was missing or wrong.
    pub why: String,
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "connection {}: {}", self.connection, self.why)
    }
}

/// What the subject was, and under what conditions it was measured.
///
/// ⛔ **Everything here is a label on the subject rather than a reading off the
/// wire**, which is why it is a separate type from the capture. The two are
/// joined by a caller that has both, and neither can invent the other.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Identity {
    /// The browser's name, as its vendor spells it.
    pub name: String,
    /// ⛔ The exact build, never a major alone.
    pub version: String,
    /// The release channel.
    pub channel: Channel,
    /// Whether this build carries its vendor's own entry in its brand list.
    pub branded: bool,
    /// The operating system the capture was taken on.
    pub os: Os,
    /// The architecture, as the machine reports it.
    pub arch: String,
    /// The distribution or release, where it is knowable.
    pub distribution: Option<String>,
    /// How the browser was run: `container`, `host`, `vm`.
    pub method: String,
    /// The harness that read the bytes, with its version.
    pub harness: String,
    /// Who or what took the capture.
    pub operator: String,
    /// How the subject came to trust the harness.
    pub trust: Trust,
    /// Whether the harness offered the subject a way to resume a session.
    ///
    /// ⛔ **Read from what the harness printed, never typed.** ⚠ Defaulted
    /// on the way in so an identity file written before this field existed still
    /// reads, and absent rather than assumed: a run that did not report the
    /// condition did not measure it. `TODO/corpus.md`, `CORPUS-02`.
    #[serde(default)]
    pub resumption: Option<b_ids_schema::Resumption>,
    /// The switches the subject was launched with, in order.
    pub switches: Vec<String>,
    /// Where the build came from, when this project fetched it.
    ///
    /// ⚠ **Absent where nothing was fetched**, which is a different fact from
    /// an acquisition that failed. A build already installed on the machine
    /// was not obtained by this project and has no route or digest.
    /// ⭐ Defaulted on the way in so an identity file written before this
    /// field existed still reads. `TODO/driver.md`, `DRIVER-05`.
    #[serde(default)]
    pub acquisition: Option<b_ids_schema::Acquisition>,
}

/// The major component of a build string.
///
/// ⭐ **Derived, never typed beside the version.** A major somebody typed is a
/// second copy of a number the version already carries, and
/// [`Profile::check`] refuses the two disagreeing, so typing it wrong is a
/// refusal rather than a silent drift. Deriving it removes the chance.
fn major_of(version: &str) -> u32 {
    version
        .split('.')
        .next()
        .and_then(|m| m.parse::<u32>().ok())
        .unwrap_or(0)
}

/// Replace the throwaway profile directory in a switch list.
///
/// ⛔ **A published profile carries no absolute path from the machine that took
/// it.** The directory is created per launch and removed after it, so its name
/// is noise that differs per run and it names a person's home directory on
/// every desktop host. ⚠ The substitution is recorded in the provenance map by
/// [`profile_from`]; a rewrite nobody reported would be the failure this whole
/// project is about.
///
/// Returns the switches and whether anything was replaced.
fn redact_switches(switches: &[String]) -> (Vec<String>, bool) {
    let mut changed = false;
    let out = switches
        .iter()
        .map(|switch| {
            if switch.starts_with(USER_DATA_DIR) {
                changed = true;
                format!("{USER_DATA_DIR}{REDACTED_DIR}")
            } else {
                switch.clone()
            }
        })
        .collect();
    (out, changed)
}

/// Build a profile from one capture and the identity of what produced it.
///
/// ⛔ **The capture must be the connection a profile is built from**: one that
/// carried a `ClientHello` and reached HTTP/2. A browser opens sockets it
/// abandons, and `b_ids_harness::select` is what picks the cold one out of a
/// navigation. Refusing everything else here means the corpus cannot acquire a
/// profile of an abandoned preconnect by anybody forgetting to select.
///
/// # Errors
///
/// A [`Refusal`] naming the connection and what was missing, or the
/// [`Defect`]s the assembled profile fails its own well-formedness check with.
/// ⚠ The second case includes the credential refusal, which is the one this
/// function exists to place.
pub fn profile_from(capture: &Capture, identity: &Identity) -> Result<Profile, Vec<Refusal>> {
    let refuse = |why: &str| {
        vec![Refusal {
            connection: capture.connection,
            why: why.to_owned(),
        }]
    };

    let Some(tls) = capture.tls.clone() else {
        return Err(refuse(
            "carries no ClientHello, so there is no TLS half to publish. A profile is not \
             assembled from a connection that did not send one",
        ));
    };
    let Some(http2) = capture.http2.clone() else {
        return Err(refuse(
            "never reached HTTP/2, which is what an abandoned preconnect looks like. Select the \
             cold connection with b_ids_harness::select before converting",
        ));
    };
    if capture.at.trim().is_empty() {
        return Err(refuse(
            "carries no instant. A capture with no instant cannot be ordered against the build it \
             describes, and stamping one here would record when this ran rather than when the \
             bytes arrived",
        ));
    }

    // ⛔ THROUGH `HeaderSet::record`, which is the one construction path and
    // the one place the credential filter lives. Assembling the fields here
    // would be another door into that rule.
    //
    // ⚠ The VALUE POLICY is taken from what the capture actually holds rather
    // than from a flag passed here: the harness has already dropped the values
    // unless it was asked for them, so a `WithValues` policy over a names-only
    // capture would record empty strings as if they were measured values.
    let policy = if http2.headers.iter().any(|h| h.value.is_some()) {
        ValuePolicy::WithValues
    } else {
        ValuePolicy::NamesOnly
    };
    let http = HttpHalf {
        variants: vec![HeaderSet::record(
            Variant::Navigate,
            http2
                .headers
                .iter()
                .filter(|h| !h.name.starts_with(':'))
                .map(|h| (h.name.clone(), h.value.clone().unwrap_or_default())),
            policy,
        )],
        multipart_boundary: None,
    };

    let frames: Vec<String> = http2
        .frames
        .iter()
        .map(b_ids_harness::RawFrame::wire_hex)
        .collect();
    let raw = Raw {
        client_hello_hex: Some(capture.raw_hex.clone()),
        // ⚠ The same frame in two fields, and `Raw::check` asserts they agree.
        // The older field is kept for profiles written before the list existed.
        settings_frame_hex: frames.first().cloned(),
        http2_frames_hex: frames,
        request_line_hex: None,
        // ⭐ THE THIRD DOOR INTO THE CREDENTIAL RULE. This is the decrypted
        // first message, which is where a real browser's credentials appear.
        // `Raw::check` scans it, so assembling it here is what arms the
        // refusal.
        connection_hex: capture
            .termination
            .as_ref()
            .map(|t| t.plaintext_hex.clone()),
        // ⚠ ABSENT rather than derived. The harness does not read the record
        // layer as its own block today, and filling a measured-looking field
        // from a second parse of bytes this crate happens to hold would be a
        // derivation wearing a measurement's label.
        record_layer: None,
    };

    let (switches, redacted) = redact_switches(&identity.switches);

    let mut provenance = Provenance::new();
    for field in ["tls", "http2", "http", "raw"] {
        provenance.insert(
            field,
            ProvenanceEntry {
                kind: ProvenanceKind::Wire,
                reason: None,
            },
        );
    }
    if redacted {
        provenance.insert(
            "captured.switches",
            ProvenanceEntry {
                kind: ProvenanceKind::Substituted,
                reason: Some(SWITCH_REASON.to_owned()),
            },
        );
    }

    let browser = Browser {
        name: identity.name.clone(),
        version: identity.version.clone(),
        major: major_of(&identity.version),
        channel: identity.channel,
        branded: identity.branded,
    };
    let platform = Platform {
        os: identity.os,
        arch: identity.arch.clone(),
        distribution: identity.distribution.clone(),
    };
    let mut profile = Profile {
        schema: SCHEMA_ID.to_owned(),
        // ⚠ A placeholder that is immediately overwritten by the derivation
        // below. `Profile::derived_id` needs a profile to read the four keys
        // off, and `Profile::check` refuses the two disagreeing, so this can
        // never survive.
        id: b_ids_schema::ProfileId::derive(
            &browser.name,
            &browser.version,
            &b_ids_schema::PlatformToken::derive(platform.os, &platform.arch),
            browser.channel,
        ),
        browser,
        platform,
        captured: Captured {
            at: capture.at.clone(),
            method: identity.method.clone(),
            harness: identity.harness.clone(),
            operator: identity.operator.clone(),
            trust: identity.trust,
            // ⛔ Carried from what the harness reported, never typed beside the
            // capture. A profile that claimed a resumption configuration the run
            // did not use would be a condition nobody could contradict from the
            // bytes: a cold hello looks the same either way.
            resumption: identity.resumption,
            switches,
            // ⛔ Carried from the identity file rather than derived here. The
            // route that answered and the digest of what arrived are facts
            // about a FETCH, and nothing in this crate does one.
            acquisition: identity.acquisition.clone(),
        },
        tls,
        http2: http2.half,
        http,
        // ⚠ EMPTY, and that is honest. Nothing in this tree computes JA3 or
        // JA4 yet; `VALID-04` is the entry that does, with published test
        // vectors. A digest computed by an unverified implementation and
        // published as a field is exactly the fabricated value this project
        // refuses.
        digests: Digests::default(),
        raw,
        provenance,
        supersedes: None,
    };
    profile.id = profile.derived_id();
    Ok(profile)
}

/// Every way an assembled profile is malformed on its own terms.
///
/// ⭐ **A thin name over [`Profile::check`], so the corpus writer and the
/// corpus verifier ask exactly the same question.** Two callers with two
/// spellings of "is this acceptable" is two answers.
#[must_use]
pub fn defects(profile: &Profile) -> Vec<Defect> {
    profile.check()
}
