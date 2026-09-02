//! Rebuilding a profile's measured halves from its raw block alone.
//!
//! ⛔ **This is what makes the raw block a backstop rather than a gesture.** A
//! capture is a moment that cannot be retaken, and the reason to keep the bytes
//! is that this project's own parser will one day turn out to be wrong. A raw
//! block nobody has re-parsed is a claim rather than a backstop.
//!
//! ⚠ **It is a second ENTRY into the parser, not a second parser.** Rebuilding
//! with an independent implementation would test the two implementations
//! against each other and say nothing about whether the bytes are sufficient.
//! What this answers is the one question the raw block exists for: are the
//! stored bytes enough to produce the model again.

use b_ids_schema::http::{HeaderSet, HttpHalf, ValuePolicy, Variant};
use b_ids_schema::http2::Http2Half;
use b_ids_schema::tls::TlsHalf;
use b_ids_schema::{Profile, Raw};

use crate::bytes::unhex;
use crate::note::Note;

/// The halves a raw block reproduced.
///
/// ⚠ A half is `None` where the raw block carries no bytes for it, which is
/// not the same as a half that rebuilt to nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rebuilt {
    /// The TLS half, where the raw block carries a `ClientHello`.
    pub tls: Option<TlsHalf>,
    /// The HTTP/2 half, where the raw block carries frames.
    pub http2: Option<Http2Half>,
    /// The HTTP half, where the raw block carries a cleartext request.
    pub http: Option<HttpHalf>,
    /// What could not be read on the way back.
    pub notes: Vec<Note>,
}

/// Rebuild what a raw block carries.
///
/// ⛔ Never guesses. A half the raw block cannot produce comes back `None`,
/// and the caller compares that against what the profile claims rather than
/// against a default.
#[must_use]
pub fn rebuild(raw: &Raw, policy: ValuePolicy) -> Rebuilt {
    let mut notes = Vec::new();

    let tls =
        raw.client_hello_hex
            .as_ref()
            .and_then(|hex| match unhex(hex).map_err(|e| e.to_string()) {
                Ok(bytes) => match crate::parse_record(&bytes) {
                    Ok(capture) => {
                        notes.extend(capture.notes);
                        Some(capture.tls)
                    }
                    Err(why) => {
                        notes.push(Note::new("raw.client_hello_hex", why));
                        None
                    }
                },
                Err(why) => {
                    notes.push(Note::new("raw.client_hello_hex", why));
                    None
                }
            });

    // ⚠ The frames are stored one per entry and the reader takes a connection,
    // so the preface is put back in front of them. It is a constant of the
    // protocol rather than a captured value, which is why it is not stored.
    let http2 = if raw.http2_frames_hex.is_empty() {
        None
    } else {
        let mut bytes = Vec::from(crate::h2::PREFACE);
        let mut readable = true;
        for (index, frame) in raw.http2_frames_hex.iter().enumerate() {
            match unhex(frame) {
                Ok(decoded) => bytes.extend_from_slice(&decoded),
                Err(why) => {
                    notes.push(Note::new(format!("raw.http2_frames_hex.{index}"), why));
                    readable = false;
                }
            }
        }
        if readable {
            match crate::h2::parse_connection(&bytes, policy, &mut notes) {
                Ok(capture) => Some(capture.half),
                Err(why) => {
                    notes.push(Note::new("raw.http2_frames_hex", why));
                    None
                }
            }
        } else {
            None
        }
    };

    let http = raw
        .connection_hex
        .as_ref()
        .and_then(|hex| match unhex(hex) {
            Ok(bytes) if !crate::h2::starts_like_preface(&bytes) => {
                Some(http_half_from(&bytes, policy, &mut notes))
            }
            Ok(_) => None,
            Err(why) => {
                notes.push(Note::new("raw.connection_hex", why));
                None
            }
        });

    Rebuilt {
        tls,
        http2,
        http,
        notes,
    }
}

/// Read a cleartext HTTP/1.1 request back into a header set.
///
/// ⛔ Through [`HeaderSet::record`], which is the one construction path and the
/// one place the credential rule is enforced. A rebuild that assembled the
/// fields itself would be a fourth door into that rule.
fn http_half_from(bytes: &[u8], policy: ValuePolicy, notes: &mut Vec<Note>) -> HttpHalf {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.split("\r\n");
    let _request_line = lines.next();
    let mut fields: Vec<(String, String)> = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            notes.push(Note::new(
                "raw.connection_hex",
                format!("a header line carries no colon: {line}"),
            ));
            continue;
        };
        fields.push((name.trim().to_owned(), value.trim().to_owned()));
    }
    HttpHalf {
        variants: vec![HeaderSet::record(Variant::Navigate, fields, policy)],
        multipart_boundary: None,
    }
}

/// Every half of a profile that its own raw block does not reproduce.
///
/// ⭐ **This is the acceptance in one function.** An empty result means the
/// bytes are sufficient: nothing in the measured halves came from anywhere
/// except the wire.
#[must_use]
pub fn differences(profile: &Profile, policy: ValuePolicy) -> Vec<String> {
    let rebuilt = rebuild(&profile.raw, policy);
    let mut out = Vec::new();

    match &rebuilt.tls {
        Some(tls) if *tls == profile.tls => {}
        Some(_) => out.push("tls: the rebuilt half differs from the recorded one".to_owned()),
        None => out.push("tls: the raw block does not carry a ClientHello".to_owned()),
    }
    match &rebuilt.http2 {
        Some(http2) if *http2 == profile.http2 => {}
        Some(_) => out.push("http2: the rebuilt half differs from the recorded one".to_owned()),
        None => out.push("http2: the raw block carries no frames".to_owned()),
    }
    match &rebuilt.http {
        Some(http) if *http == profile.http => {}
        Some(_) => out.push("http: the rebuilt half differs from the recorded one".to_owned()),
        None => out.push("http: the raw block carries no cleartext request".to_owned()),
    }
    out
}
