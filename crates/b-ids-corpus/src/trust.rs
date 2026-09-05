//! The trust-anchor extension, read out of a profile as its own artefact.
//!
//! ⛔ **One extension carries a snapshot of the browser's own root store**, so a
//! client carrying one build's list is advertising which build it copied. It
//! changes on a different schedule from everything else in a profile, which is
//! why it is published beside the corpus rather than left buried in one.
//! `docs/history/todo/corpus.md`, `CORPUS-04`.
//!
//! ⚠ **The codepoint's body is measured and its NAME is inferred.**
//! `docs/inherited-claims.md` section 3 carries that split, and nothing here
//! settles it: this module reads the bytes and says what shape they have. A
//! reader that asserted the name would be publishing an inference as a
//! measurement.

use b_ids_schema::Profile;

/// The codepoint the trust-anchor list is carried at.
///
/// ⚠ **Measured, and the name attached to it is not.**
/// `docs/inherited-claims.md` section 3.
pub const TRUST_ANCHORS: u16 = 0xca34;

/// One build's list, as it was found on the wire.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AnchorList {
    /// The profile this came out of.
    pub profile_id: String,
    /// The browser, as its vendor spells it.
    pub browser: String,
    /// The exact build.
    pub version: String,
    /// The platform token the corpus routes by.
    pub platform: String,
    /// ⛔ **When the bytes were read**, which is the whole reason this is
    /// published separately. A list with no date is a list nobody can place.
    pub captured_at: String,
    /// The declared extension length, in bytes.
    ///
    /// ⚠ A `u16` on the wire and here, because that is what the record layer
    /// gives an extension. Widening it would invent a range the format has not.
    pub extension_length: u16,
    /// Every identifier, lowercase hex, in the browser's own order.
    ///
    /// ⛔ **In the order they arrived.** The order is part of what was
    /// measured, and sorting it here would publish a list no browser sent.
    pub identifiers: Vec<String>,
}

/// Why a profile's extension body could not be read as a list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAList {
    /// The profile does not carry the extension at all.
    ///
    /// The ordinary case, and not an error. Extension absence is distinct from
    /// a present two-byte body that encodes an empty list.
    Absent,
    /// The body is present and does not have the shape this reads.
    Malformed {
        /// What went wrong, in terms of the bytes.
        why: String,
    },
}

impl core::fmt::Display for NotAList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Absent => f.write_str("carries no trust-anchor extension"),
            Self::Malformed { why } => write!(f, "the trust-anchor body is malformed: {why}"),
        }
    }
}

/// Read one profile's trust-anchor list.
///
/// ⭐ **The shape, measured on 2026-09-02 against Chrome `152.0.7977.75` on
/// `linux64`**: a two-byte big-endian length, then that many bytes of
/// one-byte-length-prefixed identifiers. That build carried 206 bytes, an outer
/// length of 204, and **32** identifiers of 4, 5 and 8 bytes.
///
/// ⚠ **`docs/inherited-claims.md` section 3 says 24 identifiers**, inherited
/// from a different build's capture. ⛔ Both can be right, and that is the
/// entry's premise: the list is per build. Nothing here reconciles them.
///
/// # Errors
///
/// [`NotAList::Absent`] where the profile does not carry the extension, which
/// is not a defect, and [`NotAList::Malformed`] where the body does not decode.
pub fn anchor_list(profile: &Profile) -> Result<AnchorList, NotAList> {
    let extension = profile
        .tls
        .extensions
        .iter()
        .find(|e| e.codepoint == TRUST_ANCHORS)
        .ok_or(NotAList::Absent)?;

    let bytes = decode_hex(&extension.body_hex).map_err(|why| NotAList::Malformed { why })?;
    if bytes.len() < 2 {
        return Err(NotAList::Malformed {
            why: format!("{} byte(s), and the outer length alone is two", bytes.len()),
        });
    }
    let declared = usize::from(u16::from_be_bytes([bytes[0], bytes[1]]));
    let body = &bytes[2..];
    if declared != body.len() {
        // ⛔ REFUSED rather than truncated or padded. A body whose declared
        // length disagrees with what arrived is one this reader does not
        // understand, and guessing which is right would publish a list the
        // browser did not send.
        return Err(NotAList::Malformed {
            why: format!(
                "the outer length says {declared} and {} byte(s) followed it",
                body.len()
            ),
        });
    }

    let mut identifiers = Vec::new();
    let mut at = 0_usize;
    while at < body.len() {
        let length = usize::from(body[at]);
        at += 1;
        if at + length > body.len() {
            return Err(NotAList::Malformed {
                why: format!(
                    "an identifier of {length} byte(s) at offset {at} runs past the {} the body \
                     has",
                    body.len()
                ),
            });
        }
        identifiers.push(hex(&body[at..at + length]));
        at += length;
    }

    Ok(AnchorList {
        profile_id: profile.id.to_string(),
        browser: profile.browser.name.clone(),
        version: profile.browser.version.clone(),
        platform: profile.platform_token().as_str().to_owned(),
        captured_at: profile.captured.at.clone(),
        extension_length: extension.length,
        identifiers,
    })
}

/// Every list a set of profiles carries, in the order the profiles were given.
///
/// Profiles without the extension are skipped. A malformed extension stops the
/// batch so corruption cannot be reported as absence.
///
/// # Errors
///
/// Returns [`NotAList::Malformed`] for the first malformed extension.
pub fn anchor_lists(profiles: &[Profile]) -> Result<Vec<AnchorList>, NotAList> {
    let mut lists = Vec::new();
    for profile in profiles {
        match anchor_list(profile) {
            Ok(list) => lists.push(list),
            Err(NotAList::Absent) => {}
            Err(error @ NotAList::Malformed { .. }) => return Err(error),
        }
    }
    Ok(lists)
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) {
        return Err(format!("{} hex characters, which is odd", text.len()));
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        let pair = &text[at..at + 2];
        out.push(u8::from_str_radix(pair, 16).map_err(|_| format!("{pair} is not a hex byte"))?);
        at += 2;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{NotAList, TRUST_ANCHORS, anchor_lists};

    #[test]
    fn batches_skip_absence_keep_empty_lists_and_refuse_malformed_bodies() {
        let mut absent = b_ids_schema::fixture::profile();
        absent
            .tls
            .extensions
            .retain(|extension| extension.codepoint != TRUST_ANCHORS);
        assert!(
            anchor_lists(&[absent])
                .expect("absence is not an error")
                .is_empty()
        );

        let mut empty = b_ids_schema::fixture::profile();
        let extension = empty
            .tls
            .extensions
            .iter_mut()
            .find(|extension| extension.codepoint == TRUST_ANCHORS)
            .expect("the fixture carries the extension");
        extension.length = 2;
        extension.body_hex = "0000".to_owned();
        let lists = anchor_lists(&[empty]).expect("an encoded empty list is valid");
        assert_eq!(lists.len(), 1);
        assert!(lists[0].identifiers.is_empty());

        let malformed = b_ids_schema::fixture::profile();
        assert!(matches!(
            anchor_lists(&[malformed]),
            Err(NotAList::Malformed { .. })
        ));
    }
}
