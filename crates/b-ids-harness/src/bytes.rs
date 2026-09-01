//! Reading bytes without slicing past the end, and the hex both directions.
//!
//! ⛔ **One read path.** The `ClientHello` reader and the HTTP/2 frame reader
//! are two parsers over hostile input and they share this cursor rather than
//! carrying two copies of it. Two copies acquire different defects, and a fix
//! to one never reaches the other.
//!
//! ⛔ **Every method returns `None` at the end of input rather than slicing
//! past it.** This is the whole defence against a malformed message taking down
//! the one component that faces the network.

/// A bounds-checked reader over a byte slice.
#[derive(Debug)]
pub struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    /// Start at the beginning of `bytes`.
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    /// Take `n` bytes, or `None` where fewer than `n` remain.
    pub fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(n)?;
        let slice = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(slice)
    }

    /// One byte.
    pub fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    /// Two bytes, big-endian.
    pub fn u16(&mut self) -> Option<u16> {
        let b = self.take(2)?;
        Some(u16::from(b[0]) << 8 | u16::from(b[1]))
    }

    /// Three bytes, big-endian, which is how HTTP/2 and TLS both write a
    /// length.
    pub fn u24(&mut self) -> Option<u32> {
        let b = self.take(3)?;
        Some(u32::from(b[0]) << 16 | u32::from(b[1]) << 8 | u32::from(b[2]))
    }

    /// Four bytes, big-endian.
    pub fn u32(&mut self) -> Option<u32> {
        let b = self.take(4)?;
        Some(u32::from(b[0]) << 24 | u32::from(b[1]) << 16 | u32::from(b[2]) << 8 | u32::from(b[3]))
    }

    /// How many bytes are left.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.at)
    }

    /// How far in the cursor has read.
    #[must_use]
    pub fn position(&self) -> usize {
        self.at
    }
}

/// The base64 alphabet, as RFC 4648 section 4 defines it.
const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Base64-encode bytes, padded.
///
/// ⚠ **Encode only, and that is the whole requirement.** The one caller is the
/// certificate pin a client is given, which is produced here and consumed by
/// another program. A decoder with no caller would be machinery nothing asks
/// for.
#[must_use]
pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).copied().map_or(0, u32::from);
        let b2 = chunk.get(2).copied().map_or(0, u32::from);
        let triple = (b0 << 16) | (b1 << 8) | b2;
        for shift in [18_u32, 12, 6, 0] {
            let index = ((triple >> shift) & 0x3f) as usize;
            out.push(char::from(BASE64[index]));
        }
        // ⛔ Padding is written by TRUNCATING the four characters this chunk
        // produced, never by skipping them. A short chunk still contributes
        // its high bits to the character before the padding.
        if chunk.len() < 3 {
            out.truncate(out.len() - (3 - chunk.len()));
            for _ in 0..(3 - chunk.len()) {
                out.push('=');
            }
        }
    }
    out
}

/// SHA-256, from the crypto provider this tree already links.
///
/// ⛔ **No second crypto library for one digest, and no second copy of this
/// function.** It has two callers with unrelated jobs: the certificate pin in
/// [`crate::tls`], and the content address `b-ids-corpus` writes beside every
/// published file. A digest computed in two places is two places for it to be
/// computed differently, in the one field whose whole purpose is that two
/// parties agree on it.
///
/// ⚠ Reaching it through a cipher suite is a public path rather than a patch to
/// the vendored tree.
#[must_use]
pub fn sha256(data: &[u8]) -> Vec<u8> {
    let suite = rustls::crypto::ring::cipher_suite::TLS13_AES_128_GCM_SHA256;
    let rustls::SupportedCipherSuite::Tls13(tls13) = suite else {
        // ⛔ Unreachable by construction: the constant above IS a TLS 1.3
        // suite. It returns an empty digest rather than panicking, and every
        // caller is a comparison nobody could match against an empty one.
        return Vec::new();
    };
    tls13.common.hash_provider.hash(data).as_ref().to_vec()
}

/// Hex-encode bytes, lower case, no separators.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
        out.push(char::from_digit(u32::from(byte & 0x0f), 16).unwrap_or('0'));
    }
    out
}

/// Decode a hex string, ignoring whitespace.
///
/// # Errors
///
/// Returns the offending character where the input is not hex, or a message
/// where the length is odd.
pub fn unhex(text: &str) -> Result<Vec<u8>, String> {
    let digits: Vec<u8> = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| {
            c.to_digit(16)
                .map(|d| u8::try_from(d).unwrap_or(0))
                .ok_or_else(|| format!("{c} is not a hex digit"))
        })
        .collect::<Result<_, _>>()?;
    if !digits.len().is_multiple_of(2) {
        return Err(format!("{} hex digits is an odd number", digits.len()));
    }
    Ok(digits.chunks(2).map(|p| (p[0] << 4) | p[1]).collect())
}
