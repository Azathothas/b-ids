//! Headless changes what a browser announces, and normalising it is reported.
//!
//! ⭐ **Silently rewriting a captured value is the failure this whole project is
//! about**, so the reporting is the point rather than a nicety. A normalised
//! field is marked `substituted` in the profile's provenance map with a reason,
//! and a consumer can see exactly which values were not measured as they stand.
//!
//! ⚠ **Measured here on 2026-09-01, Chrome `151.0.7922.76` on Windows.** One
//! header changes and one does not:
//!
//! | header | headful | headless |
//! | --- | --- | --- |
//! | `user-agent` | `... Chrome/151.0.0.0 Safari/537.36` | `... HeadlessChrome/151.0.0.0 Safari/537.36` |
//! | `sec-ch-ua` | `"Not=A?Brand";v="99", "Google Chrome";v="151", "Chromium";v="151"` | ⭐ identical |
//!
//! ⛔ **That refines what this entry inherited**, which said the substitution
//! reaches the brand list on some builds. On the build measured here it does
//! not, and nothing is rewritten in a field nothing was seen to change.
//!
//! `TODO/driver.md`, `DRIVER-03`.

use b_ids_schema::{Profile, ProvenanceEntry, ProvenanceKind};

/// The product token a headless build announces.
const HEADLESS_TOKEN: &str = "HeadlessChrome/";

/// The product token the same build announces with a window.
const HEADFUL_TOKEN: &str = "Chrome/";

/// The reason a normalised field carries in the provenance map.
///
/// ⚠ **One string, so a consumer can filter on it.** A reason written twice in
/// two shapes is a reason nobody can select.
pub const REASON: &str = "headless-product-token";

/// What normalising one value did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Normalisation {
    /// The value after normalising, which equals the input where nothing
    /// changed.
    pub value: String,
    /// Whether anything changed.
    pub changed: bool,
}

/// Replace a headless product token with the one a windowed build announces.
///
/// ⛔ **It changes the product token and nothing else.** The version, the
/// platform block and every other token are the browser's own, and rewriting
/// one this function was not sure about would be the defect the entry names.
#[must_use]
pub fn normalise_user_agent(value: &str) -> Normalisation {
    if !value.contains(HEADLESS_TOKEN) {
        return Normalisation {
            value: value.to_owned(),
            changed: false,
        };
    }
    Normalisation {
        value: value.replace(HEADLESS_TOKEN, HEADFUL_TOKEN),
        changed: true,
    }
}

/// Whether a value carries a headless marker this module does not normalise.
///
/// ⚠ **An uncertain case is reported rather than rewritten.** The entry says an
/// uncertain field is `unreproducible` with a reason, and the first step is
/// noticing one at all.
#[must_use]
pub fn carries_headless_marker(value: &str) -> bool {
    value.to_ascii_lowercase().contains("headless")
}

/// Normalise every header value in `profile` that a headless run changed, and
/// record each substitution in the provenance map.
///
/// Returns the provenance keys it wrote, in the order it wrote them.
///
/// ⛔ **The value changes and the fact that it changed is published beside it.**
/// A normalisation with no provenance entry is a rewritten capture, which is the
/// one thing this project cannot ship.
pub fn normalise(profile: &mut Profile) -> Vec<String> {
    let mut written = Vec::new();
    for set in &mut profile.http.variants {
        for header in &mut set.headers {
            let Some(value) = header.value.as_ref() else {
                continue;
            };
            if header.name != "user-agent" {
                // ⚠ Any OTHER header carrying a headless marker is recorded as
                // unreproducible rather than guessed at. On the build measured
                // here there is none, and a rule that only fires on what was
                // measured is a rule that misses the next build quietly.
                if carries_headless_marker(value) {
                    let key = format!("http.headers.{}", header.name);
                    profile.provenance.insert(
                        &key,
                        ProvenanceEntry {
                            kind: ProvenanceKind::Unreproducible,
                            reason: Some(REASON.to_owned()),
                        },
                    );
                    written.push(key);
                }
                continue;
            }
            let normalised = normalise_user_agent(value);
            if !normalised.changed {
                continue;
            }
            header.value = Some(normalised.value);
            let key = format!("http.headers.{}", header.name);
            profile.provenance.insert(
                &key,
                ProvenanceEntry {
                    kind: ProvenanceKind::Substituted,
                    reason: Some(REASON.to_owned()),
                },
            );
            written.push(key);
        }
    }
    written
}
