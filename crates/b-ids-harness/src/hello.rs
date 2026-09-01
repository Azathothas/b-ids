//! Reading a `ClientHello` off the wire.
//!
//! ⛔ **Parse permissively, emit exactly.** A capture harness is pointed at
//! hostile bytes by definition, and a parser that refuses a hello it does not
//! recognise has thrown away the capture. Every field this module cannot read
//! becomes a [`Note`] on the capture rather than an error, and the raw bytes
//! are kept whatever happens, because they are the one artefact that survives
//! every parser defect.
//!
//! ⛔ **Nothing here panics on input.** Every read is bounds-checked through
//! [`crate::bytes::Cursor`], which returns `None` at the end rather than
//! slicing past it. A panic in this module is a denial of service in the one
//! component that faces the network.

use b_ids_schema::tls::{Ech, Extension, Grease, KeyShare, Shuffle, TlsHalf, is_grease_value};

use crate::bytes::{Cursor, hex, unhex};
use crate::note::Note;

/// What one `ClientHello` produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelloCapture {
    /// The TLS half of a profile, as far as it could be read.
    pub tls: TlsHalf,
    /// The whole record, hex-encoded, whatever else happened.
    pub raw_hex: String,
    /// Fields that could not be read.
    pub notes: Vec<Note>,
}

/// Parse a TLS record carrying a `ClientHello`.
///
/// ⚠ Takes the WHOLE record including its five-byte header, because
/// `record_version` is part of the fingerprint and lives there rather than in
/// the handshake message.
///
/// # Errors
///
/// Only for input that is not a `ClientHello` record at all. Everything the
/// parser cannot read INSIDE a well-framed hello becomes a note.
pub fn parse_record(record: &[u8]) -> Result<HelloCapture, String> {
    let raw_hex = hex(record);
    let mut cursor = Cursor::new(record);

    let content_type = cursor.u8().ok_or("empty input")?;
    if content_type != 0x16 {
        return Err(format!(
            "content type 0x{content_type:02x} is not handshake (0x16)"
        ));
    }
    let record_version = cursor.u16().ok_or("truncated before the record version")?;
    let declared = cursor.u16().ok_or("truncated before the record length")?;

    let mut notes = Vec::new();
    // ⛔ Count what arrived; do not trust what was declared. A declared length
    // longer than the body is exactly the shape a truncation attack has, and
    // padding to match it would record bytes nobody sent.
    let available = cursor.remaining();
    if usize::from(declared) != available {
        notes.push(Note {
            field: "record.length".to_owned(),
            why: format!("declares {declared} bytes and {available} arrived"),
        });
    }

    let handshake_type = cursor.u8().ok_or("truncated before the handshake type")?;
    if handshake_type != 0x01 {
        return Err(format!(
            "handshake type 0x{handshake_type:02x} is not client_hello (0x01)"
        ));
    }
    let _handshake_len = cursor
        .u24()
        .ok_or("truncated before the handshake length")?;

    let legacy_version = cursor.u16().ok_or("truncated before the legacy version")?;
    cursor.take(32).ok_or("truncated inside the random")?;

    let session_id_len = cursor
        .u8()
        .ok_or("truncated before the session id length")?;
    let session_id = cursor
        .take(usize::from(session_id_len))
        .ok_or("truncated inside the session id")?;

    let cipher_len = cursor.u16().ok_or("truncated before the cipher suites")?;
    let cipher_bytes = cursor
        .take(usize::from(cipher_len))
        .ok_or("truncated inside the cipher suites")?;
    let cipher_suites = u16_list(cipher_bytes);

    let comp_len = cursor
        .u8()
        .ok_or("truncated before the compression methods")?;
    let compression_methods = cursor
        .take(usize::from(comp_len))
        .ok_or("truncated inside the compression methods")?
        .to_vec();

    let mut extensions = Vec::new();
    if cursor.remaining() >= 2 {
        let ext_total = cursor.u16().unwrap_or(0);
        let ext_bytes = cursor.take(usize::from(ext_total)).unwrap_or(&[]);
        if ext_bytes.len() != usize::from(ext_total) {
            notes.push(Note {
                field: "extensions.length".to_owned(),
                why: format!("declares {ext_total} bytes and {} arrived", ext_bytes.len()),
            });
        }
        let mut inner = Cursor::new(ext_bytes);
        while inner.remaining() >= 4 {
            let Some(codepoint) = inner.u16() else { break };
            let Some(length) = inner.u16() else { break };
            let Some(body) = inner.take(usize::from(length)) else {
                notes.push(Note {
                    field: format!("extensions.0x{codepoint:04x}"),
                    why: format!("declares {length} bytes and the list ends first"),
                });
                break;
            };
            extensions.push(Extension {
                codepoint,
                length,
                body_hex: hex(body),
            });
        }
    }

    let find = |codepoint: u16| extensions.iter().find(|e| e.codepoint == codepoint);
    let body_of = |codepoint: u16| find(codepoint).map(|e| unhex(&e.body_hex).unwrap_or_default());

    let key_exchange_groups = body_of(0x000a)
        .map(|b| u16_list_with_header(&b))
        .unwrap_or_default();
    let signature_algorithms = body_of(0x000d)
        .map(|b| u16_list_with_header(&b))
        .unwrap_or_default();
    let signature_algorithms_cert = body_of(0x0032).map(|b| u16_list_with_header(&b));
    let alpn = body_of(0x0010).map(|b| alpn_list(&b)).unwrap_or_default();
    let key_shares = body_of(0x0033)
        .map(|b| key_share_list(&b))
        .unwrap_or_default();
    let padding_len = find(0x0015).map(|e| e.length);
    let ech = body_of(0xfe0d).and_then(|b| {
        let mut c = Cursor::new(&b);
        Some(Ech {
            mode: c.u8()?,
            kem_id: c.u16()?,
        })
    });
    if find(0xfe0d).is_some() && ech.is_none() {
        notes.push(Note {
            field: "tls.ech".to_owned(),
            why: "the extension is present and its body is shorter than a mode and a kem id"
                .to_owned(),
        });
    }

    let grease_positions: Vec<usize> = extensions
        .iter()
        .enumerate()
        .filter(|(_, e)| is_grease_value(e.codepoint))
        .map(|(i, _)| i)
        .collect();
    let grease_values: Vec<u16> = extensions
        .iter()
        .filter(|e| is_grease_value(e.codepoint))
        .map(|e| e.codepoint)
        .collect();
    let grease_bodies: Vec<String> = extensions
        .iter()
        .filter(|e| is_grease_value(e.codepoint))
        .map(|e| e.body_hex.clone())
        .collect();
    let distinct = {
        let mut seen = grease_values.clone();
        seen.sort_unstable();
        seen.dedup();
        seen.len() == grease_values.len()
    };

    Ok(HelloCapture {
        tls: TlsHalf {
            record_version,
            legacy_version,
            session_id_len,
            session_id_hex: hex(session_id),
            cipher_suites,
            compression_methods,
            extensions,
            key_exchange_groups,
            key_shares,
            signature_algorithms,
            signature_algorithms_cert,
            alpn,
            ech,
            padding_len,
            // ⛔ One handshake is not a sample, so the shuffle is UNKNOWN from
            // one hello and never Fixed. HARNESS-08 is what takes more than one.
            shuffled: Shuffle::Unknown,
            grease: Grease {
                extension_positions: grease_positions,
                values: grease_values,
                distinct,
                bodies_hex: grease_bodies,
            },
        },
        raw_hex,
        notes,
    })
}

fn u16_list(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| u16::from(p[0]) << 8 | u16::from(p[1]))
        .collect()
}

/// A list whose first two bytes are its own length, which is how most
/// `ClientHello` extension bodies are framed.
fn u16_list_with_header(bytes: &[u8]) -> Vec<u16> {
    let mut cursor = Cursor::new(bytes);
    let Some(len) = cursor.u16() else {
        return Vec::new();
    };
    // ⚠ Take what arrived rather than what was declared, and take the declared
    // amount only when it is there. Either way nothing slices past the end.
    let body = cursor
        .take(usize::from(len))
        .unwrap_or_else(|| bytes.get(2..).unwrap_or(&[]));
    u16_list(body)
}

fn alpn_list(bytes: &[u8]) -> Vec<String> {
    let mut cursor = Cursor::new(bytes);
    let Some(total) = cursor.u16() else {
        return Vec::new();
    };
    let body = cursor
        .take(usize::from(total))
        .unwrap_or_else(|| bytes.get(2..).unwrap_or(&[]));
    let mut inner = Cursor::new(body);
    let mut out = Vec::new();
    while inner.remaining() > 0 {
        let Some(len) = inner.u8() else { break };
        let Some(name) = inner.take(usize::from(len)) else {
            break;
        };
        out.push(String::from_utf8_lossy(name).into_owned());
    }
    out
}

fn key_share_list(bytes: &[u8]) -> Vec<KeyShare> {
    let mut cursor = Cursor::new(bytes);
    let Some(total) = cursor.u16() else {
        return Vec::new();
    };
    let body = cursor
        .take(usize::from(total))
        .unwrap_or_else(|| bytes.get(2..).unwrap_or(&[]));
    let mut inner = Cursor::new(body);
    let mut out = Vec::new();
    while inner.remaining() >= 4 {
        let Some(group) = inner.u16() else { break };
        let Some(entry_len) = inner.u16() else { break };
        if inner.take(usize::from(entry_len)).is_none() {
            // ⚠ Recorded anyway. The declared length is part of the
            // fingerprint even when the key itself was truncated.
            out.push(KeyShare { group, entry_len });
            break;
        }
        out.push(KeyShare { group, entry_len });
    }
    out
}
