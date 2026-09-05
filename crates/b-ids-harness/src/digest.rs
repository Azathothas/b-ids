//! JA4, the hashed form, computed where SHA-256 already lives.
//!
//! ⭐ **The split is deliberate and it is the one this tree already makes.**
//! `b_ids_schema::tls::TlsHalf::ja4_lists` renders the lists, because rendering
//! is pure logic over the model and `Http2Half::akamai_text` is the precedent.
//! Hashing them belongs here, because [`crate::sha256`] has one home and a
//! second implementation of it for one digest is what that home exists to
//! prevent.
//!
//! ⛔ **Implemented from the published specification** at
//! `references/FoxIO-LLC__ja4/tree/technical_details/JA4.md`, never by copying
//! source. JA4 is BSD-3 and FoxIO states no patent claim over it;
//! `docs/reference-sweeps/findings.md` finding 5 has the split, and ⛔ **no
//! member of the JA4+ family is computed anywhere in this tree** because that
//! licence question has no answer written down yet.
//!
//! ⚠ **JA3 is not here and is not coming.** It is an MD5, this tree links no
//! MD5, and the record's own rule is to record JA3 and never assert on it: it
//! preserves wire order and browsers shuffle, so it is unstable per connection.
//! `docs/inherited-claims.md` section 10.
//!
//! `docs/history/todo/validator.md`, `VALID-04`.

use b_ids_schema::tls::TlsHalf;

use crate::bytes::{hex, sha256};

/// How many characters of a SHA-256 a JA4 section carries.
///
/// ⚠ **Twelve, and the specification says so.** A truncation length that drifts
/// produces a fingerprint that looks right and matches nothing.
pub const TRUNCATION: usize = 12;

/// What a section with no values carries instead of a hash.
///
/// ⛔ **Twelve zeroes rather than the hash of an empty string**, which is the
/// specification's own reasoning: it makes it clear to a reader that the field
/// had no values, where a hash of nothing looks like a hash of something.
pub const EMPTY: &str = "000000000000";

/// The truncated SHA-256 a JA4 section carries for one list.
///
/// ⛔ **Lowercase**, which the specification requires of every hash it defines.
#[must_use]
pub fn section(list: &str) -> String {
    if list.is_empty() {
        return EMPTY.to_owned();
    }
    let mut digest = hex(&sha256(list.as_bytes()));
    digest.truncate(TRUNCATION);
    digest
}

/// The JA4 fingerprint of one `ClientHello`.
///
/// ⚠ **Derived from the model rather than from bytes**, so one parser answers
/// for every consumer. A caller holding raw bytes parses them with
/// [`crate::parse_record`] first, and `b-ids-corpus verify` is what asserts
/// that the stored bytes and the stored model agree.
#[must_use]
pub fn ja4(tls: &TlsHalf) -> String {
    let lists = tls.ja4_lists();
    format!(
        "{}_{}_{}",
        lists.prefix,
        section(&lists.ciphers_sorted),
        section(&lists.extensions_sorted)
    )
}

/// JA4's `-o` form: the same prefix over the lists in their original order.
///
/// ⚠ **A different fingerprint from [`ja4`], not a rendering of it.** The
/// specification renames the field to `ja4_o` when this form is asked for,
/// precisely so the two cannot be mistaken for each other.
#[must_use]
pub fn ja4_o(tls: &TlsHalf) -> String {
    let lists = tls.ja4_lists();
    format!(
        "{}_{}_{}",
        lists.prefix,
        section(&lists.ciphers_original),
        section(&lists.extensions_original)
    )
}
