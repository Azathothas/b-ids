//! The `ClientHello` side of the emitter.
//!
//! ⛔ **The type here cannot represent a declared length that disagrees with
//! its body**, and that is structural rather than checked. The parser's
//! [`b_ids_schema::tls::Extension`] carries a `length` field beside a
//! `body_hex`, deliberately, because a capture that recorded a disagreement has
//! recorded a finding. [`EmittableExtension`] carries bytes and derives the
//! length when it writes them, so an emitter cannot put a length on the wire
//! that its own body does not have.
//!
//! ⚠ **The conversion between them is FALLIBLE, and that is the seam.** Every
//! capture the emitter refuses is a capture the parser was right to keep.

use b_ids_schema::tls::{Extension, TlsHalf};

/// One extension in the form an emitter can put on the wire.
///
/// ⛔ No declared length. See the module documentation: the length is derived
/// from the body at the moment of writing, so the two cannot disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittableExtension {
    /// The extension type, as the two bytes on the wire.
    ///
    /// ⚠ A plain number rather than an enum. A codepoint learned at run time
    /// has no variant, and an enum here is what stopped a real version bump in
    /// two separate projects. `docs/inherited-claims.md` section 9.
    pub codepoint: u16,
    /// The body, exactly.
    ///
    /// ⭐ **Arbitrary, including on a GREASE codepoint, including one byte.** A
    /// browser sends a trailing GREASE extension carrying a single zero byte,
    /// and a model that forced a GREASE body to be empty would refuse about one
    /// handshake in five.
    pub body: Vec<u8>,
}

impl EmittableExtension {
    /// The bytes this extension puts on the wire: the codepoint, the length,
    /// then the body.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.body.len());
        out.extend_from_slice(&self.codepoint.to_be_bytes());
        // ⛔ Derived here and nowhere else. A length that came from anywhere but
        // the body is a length that can disagree with it.
        let length = u16::try_from(self.body.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&length.to_be_bytes());
        out.extend_from_slice(&self.body);
        out
    }
}

/// Why one extension of a captured profile cannot be emitted exactly.
///
/// ⛔ **A refusal, never an approximation.** An emitter that wrote its best
/// guess would produce a hello that exists nowhere, and a client announcing one
/// version over a hello nobody sends is more distinguishing than an honestly
/// old one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unreproducible {
    /// The recorded body is not hex, so there are no bytes to write.
    BodyNotHex {
        /// The extension it is about.
        codepoint: u16,
        /// What the decoder said.
        why: String,
    },
    /// The capture recorded a declared length its body does not have.
    ///
    /// ⚠ This is a real capture shape rather than a hypothetical: a truncated
    /// or padded hello produces exactly it, and the parser records the
    /// disagreement rather than repairing it. An emitter has to choose which of
    /// the two numbers to believe, and the answer is neither.
    LengthDisagrees {
        /// The extension it is about.
        codepoint: u16,
        /// The length the wire declared.
        declared: u16,
        /// How many bytes the recorded body actually holds.
        actual: usize,
    },
    /// The body is longer than a length field can express.
    BodyTooLong {
        /// The extension it is about.
        codepoint: u16,
        /// How many bytes the recorded body holds.
        actual: usize,
    },
}

impl core::fmt::Display for Unreproducible {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BodyNotHex { codepoint, why } => write!(
                f,
                "extension 0x{codepoint:04x}: the recorded body is not hex: {why}"
            ),
            Self::LengthDisagrees {
                codepoint,
                declared,
                actual,
            } => write!(
                f,
                "extension 0x{codepoint:04x}: the capture declares {declared} byte(s) and its \
                 body holds {actual}, so an emitter would have to believe one of them"
            ),
            Self::BodyTooLong { codepoint, actual } => write!(
                f,
                "extension 0x{codepoint:04x}: a body of {actual} byte(s) is longer than the \
                 two-byte length field can express"
            ),
        }
    }
}

/// Turn one captured extension into an emittable one.
///
/// # Errors
///
/// [`Unreproducible`] where the capture cannot be put on the wire byte for
/// byte. ⛔ There is no lossy path: the caller gets the bytes or the reason.
pub fn extension(captured: &Extension) -> Result<EmittableExtension, Unreproducible> {
    let body = decode_hex(&captured.body_hex).map_err(|why| Unreproducible::BodyNotHex {
        codepoint: captured.codepoint,
        why,
    })?;
    if body.len() > usize::from(u16::MAX) {
        return Err(Unreproducible::BodyTooLong {
            codepoint: captured.codepoint,
            actual: body.len(),
        });
    }
    if usize::from(captured.length) != body.len() {
        return Err(Unreproducible::LengthDisagrees {
            codepoint: captured.codepoint,
            declared: captured.length,
            actual: body.len(),
        });
    }
    Ok(EmittableExtension {
        codepoint: captured.codepoint,
        body,
    })
}

/// Turn a whole captured TLS half's extension list into emittable ones.
///
/// # Errors
///
/// Every reason, not the first one. ⚠ An emitter that stopped at the first
/// refusal would send its author back for one more run per defect, and a
/// capture cannot be retaken.
pub fn extensions(tls: &TlsHalf) -> Result<Vec<EmittableExtension>, Vec<Unreproducible>> {
    let mut out = Vec::with_capacity(tls.extensions.len());
    let mut refusals = Vec::new();
    for captured in &tls.extensions {
        match extension(captured) {
            Ok(emittable) => out.push(emittable),
            Err(why) => refusals.push(why),
        }
    }
    if refusals.is_empty() {
        Ok(out)
    } else {
        Err(refusals)
    }
}

/// The whole extensions block, as a `ClientHello` carries it: a two-byte total
/// length, then every extension in wire order.
///
/// ⭐ **This is the escape hatch.** Any emitter that reproduces a browser
/// faithfully needs an ordered list of codepoint-and-body pairs, and a model
/// with one typed field per extension cannot hold one. Retrofitting the list
/// into such a model is the largest change in this space, so it is the shape
/// this project started from. `docs/history/todo/emitters.md`, `EMIT-02`.
///
/// ⛔ **The order is the capture's, never sorted and never normalised.** A
/// browser's extension order is a fingerprint in its own right, and an emitter
/// that reordered it would produce a hello that exists nowhere.
///
/// # Errors
///
/// Every [`Unreproducible`] reason, not the first one.
pub fn extensions_block(tls: &TlsHalf) -> Result<Vec<u8>, Vec<Unreproducible>> {
    let emittable = extensions(tls)?;
    let mut body = Vec::new();
    for extension in &emittable {
        body.extend_from_slice(&extension.encode());
    }
    // ⛔ DERIVED FROM WHAT WAS WRITTEN, like every other length here. A total
    // taken from anywhere but the bytes is a total that can disagree with them.
    let Ok(length) = u16::try_from(body.len()) else {
        return Err(vec![Unreproducible::BodyTooLong {
            codepoint: 0,
            actual: body.len(),
        }]);
    };
    let mut out = Vec::with_capacity(2 + body.len());
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// The codepoints in a capture that no field of the model names.
///
/// ⭐ **The list is what the escape hatch exists for**, and it is derived from
/// the schema rather than typed here: a codepoint the model gives a field to is
/// one an emitter could write without the list, and everything else is only
/// reachable through it.
///
/// ⚠ **`supported_versions` is NOT on the named list even though the model
/// reads it**, because reading a value out of a body is not the same as being
/// able to write that body back. The test is whether the model could reproduce
/// the extension without its recorded bytes.
#[must_use]
pub fn unnamed_codepoints(tls: &TlsHalf) -> Vec<u16> {
    // The extensions whose whole content the model carries as a typed field, so
    // an emitter could rebuild the body without the recorded bytes.
    const NAMED: [u16; 4] = [
        0x000a, // supported_groups: TlsHalf::key_exchange_groups
        0x000d, // signature_algorithms: TlsHalf::signature_algorithms
        0x0010, // application_layer_protocol_negotiation: TlsHalf::alpn
        0x0033, // key_share: TlsHalf::key_shares
    ];
    tls.extensions
        .iter()
        .map(|e| e.codepoint)
        .filter(|c| !NAMED.contains(c))
        .collect()
}

/// A whole `ClientHello`, in one TLS record, from a captured profile.
///
/// ⛔ **The random is the CALLER'S**, and it is the one field of a hello this
/// project does not record. A per-connection random carries no fingerprint, so
/// the model steps over it at capture time; an emitter therefore cannot
/// reproduce a capture byte for byte and must not pretend to. Everything else
/// on the wire comes from the profile. `docs/history/todo/emitters.md`, `EMIT-02`, and
/// `docs/history/todo/library.md`, `LIB-02`.
///
/// ⚠ **The lengths are all derived from what was written**, at three levels:
/// the extensions block, the handshake body and the record. A length taken from
/// anywhere but the bytes is a length that can disagree with them.
///
/// # Errors
///
/// Every [`Unreproducible`] reason, plus the two the shape itself can produce:
/// a session id whose declared length disagrees with its bytes, and a body
/// longer than its length field.
pub fn client_hello(tls: &TlsHalf, random: &[u8; 32]) -> Result<Vec<u8>, Vec<Unreproducible>> {
    let block = extensions_block(tls)?;
    let session_id = decode_hex(&tls.session_id_hex).map_err(|why| {
        vec![Unreproducible::BodyNotHex {
            codepoint: 0,
            why: format!("the session id: {why}"),
        }]
    })?;
    if usize::from(tls.session_id_len) != session_id.len() {
        return Err(vec![Unreproducible::LengthDisagrees {
            codepoint: 0,
            declared: u16::from(tls.session_id_len),
            actual: session_id.len(),
        }]);
    }

    let mut body = Vec::new();
    body.extend_from_slice(&tls.legacy_version.to_be_bytes());
    body.extend_from_slice(random);
    body.push(tls.session_id_len);
    body.extend_from_slice(&session_id);

    let ciphers_len = tls.cipher_suites.len() * 2;
    let Ok(ciphers_len) = u16::try_from(ciphers_len) else {
        return Err(vec![Unreproducible::BodyTooLong {
            codepoint: 0,
            actual: ciphers_len,
        }]);
    };
    body.extend_from_slice(&ciphers_len.to_be_bytes());
    for suite in &tls.cipher_suites {
        body.extend_from_slice(&suite.to_be_bytes());
    }

    let Ok(compression_len) = u8::try_from(tls.compression_methods.len()) else {
        return Err(vec![Unreproducible::BodyTooLong {
            codepoint: 0,
            actual: tls.compression_methods.len(),
        }]);
    };
    body.push(compression_len);
    body.extend_from_slice(&tls.compression_methods);
    body.extend_from_slice(&block);

    // The handshake header: type 1, then a THREE-byte length.
    let Ok(body_len) = u32::try_from(body.len()) else {
        return Err(vec![Unreproducible::BodyTooLong {
            codepoint: 0,
            actual: body.len(),
        }]);
    };
    let mut handshake = Vec::with_capacity(4 + body.len());
    handshake.push(1);
    handshake.extend_from_slice(&body_len.to_be_bytes()[1..]);
    handshake.extend_from_slice(&body);

    // ⚠ The record layer's version is the CAPTURE'S, not a constant. A hello
    // that announced a different record version from the one measured would be
    // a different fingerprint in the one byte pair a middlebox reads first.
    let Ok(record_len) = u16::try_from(handshake.len()) else {
        return Err(vec![Unreproducible::BodyTooLong {
            codepoint: 0,
            actual: handshake.len(),
        }]);
    };
    let mut record = Vec::with_capacity(5 + handshake.len());
    record.push(0x16);
    record.extend_from_slice(&tls.record_version.to_be_bytes());
    record.extend_from_slice(&record_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    Ok(record)
}

/// Decode a hex body.
///
/// ⚠ Deliberately not shared with the harness's decoder. This crate does not
/// depend on the harness and must not: an emitter that reached into the capture
/// tool would be one component with two jobs, and the schema is the seam
/// between them.
fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.len().is_multiple_of(2) {
        return Err(format!("{} hex digit(s) is an odd number", text.len()));
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    for pair in bytes.chunks(2) {
        let high = hex_digit(pair[0])?;
        let low = hex_digit(pair[1])?;
        out.push((high << 4) | low);
    }
    Ok(out)
}

fn hex_digit(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        other => Err(format!(
            "{:?} is not a hex digit",
            char::from_u32(u32::from(other)).unwrap_or('?')
        )),
    }
}
