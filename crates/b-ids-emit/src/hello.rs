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
