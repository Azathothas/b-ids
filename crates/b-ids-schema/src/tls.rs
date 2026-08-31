//! The TLS half, in wire order, with unknown codepoints kept.
//!
//! ⛔ **An ordered list of codepoint-and-body pairs, never a struct of named
//! fields and never a set of booleans.** A model that enumerates extensions
//! cannot record an extension nobody has enumerated, and two such codepoints
//! are already known to exist in a shipped browser. The shape is `utls`'s
//! `ClientHelloSpec`; `docs/reference-sweeps/usable.md` section 1 is the
//! reading, including the two alternatives and why each fails.
//!
//! ⛔ **This is not academic.** A version bump in another repository stopped
//! because its model could name neither `0x12e0` nor `0xca34` in the newer
//! hello. That is what this model exists to survive.
//!
//! ⚠ **No GREASE codepoint is a typed field**, because a GREASE extension
//! carries an arbitrary body and a typed field for it makes that body
//! unparseable. GREASE is recorded as [`Grease`], which describes where the
//! values were and what they carried, beside the extension list that holds them
//! like any other codepoint.

use serde::{Deserialize, Serialize};

/// One `ClientHello` extension, as it appeared on the wire.
///
/// ⭐ `codepoint` is a number rather than an enum on purpose. A codepoint
/// learned at runtime has no variant, and a parser that cannot name it must
/// still be able to keep it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Extension {
    /// The extension type, as the two bytes on the wire.
    pub codepoint: u16,
    /// The body length the wire declared.
    ///
    /// ⚠ Recorded as declared AND checkable against `body_hex`. Trusting a
    /// declared length instead of counting what arrived is a forbidden pattern;
    /// so is discarding it, because a declared length that disagrees with the
    /// body is itself a finding.
    pub length: u16,
    /// The body, hex-encoded, empty where the extension has none.
    pub body_hex: String,
}

impl Extension {
    /// Whether the declared length agrees with the recorded body.
    ///
    /// ⭐ Two bytes of hex per byte of body. A mismatch is not repaired here:
    /// the profile records what the wire carried and the disagreement is the
    /// measurement.
    #[must_use]
    pub fn length_agrees(&self) -> bool {
        self.body_hex.len() == usize::from(self.length) * 2
    }

    /// Whether this codepoint is one of RFC 8701's sixteen reserved values.
    ///
    /// Both bytes equal, low nibble `a`: `0x0a0a`, `0x1a1a` ... `0xfafa`.
    #[must_use]
    pub fn is_grease(&self) -> bool {
        is_grease_value(self.codepoint)
    }
}

/// Whether a two-byte value is one of RFC 8701's sixteen reserved values.
///
/// ⚠ The same predicate covers cipher suites, groups and extension types,
/// because GREASE is drawn from one table for all of them.
#[must_use]
pub fn is_grease_value(value: u16) -> bool {
    let high = (value >> 8) as u8;
    let low = (value & 0xff) as u8;
    high == low && (low & 0x0f) == 0x0a
}

/// One key-share entry.
///
/// ⚠ The entry LENGTH is recorded beside the group, because two builds sending
/// the same group with different key sizes are two different handshakes and a
/// group identifier alone cannot say so.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyShare {
    /// The named group.
    pub group: u16,
    /// The length of the key exchange entry, in bytes.
    pub entry_len: u16,
}

/// Encrypted Client Hello, as the hello carried it.
///
/// ⚠ ECH GREASE and RFC 8701 GREASE are different mechanisms that share a word.
/// This is the first one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ech {
    /// The mode byte, `0` for outer and `1` for inner in the current draft.
    pub mode: u8,
    /// The key exchange identifier the extension declared.
    pub kem_id: u16,
}

/// Whether the extension order varies per connection, and how many draws said
/// so.
///
/// ⛔ `Fixed` after one draw is not a finding, it is an absence of one. The
/// draw count is carried so nobody can read a single sample as a property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum Shuffle {
    /// The order differed between draws.
    Observed {
        /// How many handshakes were compared.
        draws: u32,
    },
    /// The order was the same in every draw taken.
    Fixed {
        /// How many handshakes were compared.
        draws: u32,
    },
    /// Fewer than two handshakes were taken, so nothing can be said.
    Unknown,
}

/// Where GREASE appeared and what it carried.
///
/// ⭐ `distinct` is recorded because a browser draws GREASE independently per
/// slot and a client that reuses one value at both ends is distinguishable by
/// that alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grease {
    /// The zero-based positions in the extension list that held a GREASE
    /// codepoint.
    pub extension_positions: Vec<usize>,
    /// The GREASE codepoints drawn, in the order they appeared.
    pub values: Vec<u16>,
    /// Whether the drawn values were all different from each other.
    pub distinct: bool,
    /// The bodies those extensions carried, hex-encoded, in the same order.
    ///
    /// ⛔ A GREASE extension may carry a byte. A model that assumes an empty
    /// body cannot record one that does.
    pub bodies_hex: Vec<String>,
}

/// The TLS half of a profile: everything read out of one `ClientHello`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsHalf {
    /// The version in the record layer header.
    pub record_version: u16,
    /// The legacy version inside the handshake message.
    pub legacy_version: u16,
    /// The session id length the hello declared.
    pub session_id_len: u8,
    /// The session id, hex-encoded.
    pub session_id_hex: String,
    /// Cipher suites, in wire order, GREASE included.
    pub cipher_suites: Vec<u16>,
    /// Compression methods, in wire order.
    pub compression_methods: Vec<u8>,
    /// Extensions, in wire order, unknown codepoints kept with their bytes.
    pub extensions: Vec<Extension>,
    /// Supported groups, in wire order, GREASE included.
    pub key_exchange_groups: Vec<u16>,
    /// Key shares, with their entry lengths.
    pub key_shares: Vec<KeyShare>,
    /// Signature algorithms, in wire order.
    pub signature_algorithms: Vec<u16>,
    /// Signature algorithms for certificates, where the hello carried them.
    pub signature_algorithms_cert: Option<Vec<u16>>,
    /// ALPN protocol identifiers, in wire order.
    pub alpn: Vec<String>,
    /// Encrypted Client Hello, where the hello carried it.
    pub ech: Option<Ech>,
    /// The padding extension's length, where the hello carried one.
    pub padding_len: Option<u16>,
    /// Whether the extension order varies per connection.
    pub shuffled: Shuffle,
    /// Where GREASE appeared and what it carried.
    pub grease: Grease,
}

impl TlsHalf {
    /// The extensions whose codepoint is a GREASE value, with their positions.
    #[must_use]
    pub fn grease_extensions(&self) -> Vec<(usize, &Extension)> {
        self.extensions
            .iter()
            .enumerate()
            .filter(|(_, e)| e.is_grease())
            .collect()
    }

    /// Every extension whose declared length disagrees with its recorded body.
    #[must_use]
    pub fn length_disagreements(&self) -> Vec<&Extension> {
        self.extensions
            .iter()
            .filter(|e| !e.length_agrees())
            .collect()
    }
}
