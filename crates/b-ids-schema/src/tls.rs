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
        /// How many DISTINCT orders those draws produced.
        ///
        /// ⛔ **A count, never the orders themselves.** A profile is ONE
        /// connection, and carrying the other connections' orders inside it
        /// would fold a set of captures into one the way
        /// `docs/inherited-claims.md` section 8 says never to.
        ///
        /// ⛔ **Fewer than two is a contradiction and
        /// [`crate::Profile::check`] refuses it.** A state that says the order
        /// differed while reporting one order is a claim its own field denies.
        /// `docs/history/todo/schema.md`, `SCHEMA-10`.
        ///
        /// ⚠ Defaulted on the way in so a profile written before the field
        /// existed still reads, and 0 then means "not recorded".
        #[serde(default)]
        distinct_orders: u32,
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

/// The lists a JA4 fingerprint is built from, before anything is hashed.
///
/// ⭐ **Separated because the hash is not this crate's job.** Rendering a list
/// is pure logic over the model, which is where `akamai_text` already lives;
/// SHA-256 has one home in this tree and it is not here. A caller that wants
/// the hashed form asks `b_ids_harness::ja4`.
///
/// ⛔ **Every list has GREASE removed**, which the specification requires
/// everywhere it appears. `docs/history/todo/validator.md`, `VALID-04`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ja4Lists {
    /// `t13d1516h2`: transport, version, SNI, the two counts and the ALPN pair.
    pub prefix: String,
    /// The ciphers, sorted by hex value, comma delimited.
    pub ciphers_sorted: String,
    /// The extensions, sorted by hex value, with SNI and ALPN removed, then an
    /// underscore and the signature algorithms in wire order.
    ///
    /// ⚠ **No underscore where the hello carried no signature algorithms**,
    /// which the specification states and which changes the hash.
    pub extensions_sorted: String,
    /// The ciphers in wire order, GREASE removed.
    pub ciphers_original: String,
    /// The extensions in wire order, GREASE removed, with SNI and ALPN KEPT,
    /// then the signature algorithms.
    pub extensions_original: String,
}

impl TlsHalf {
    /// The lists a JA4 fingerprint is built from.
    ///
    /// ⛔ **Implemented from the published specification**, at
    /// `references/FoxIO-LLC__ja4/tree/technical_details/JA4.md`, and never by
    /// copying source. JA4 itself is BSD-3 with no patent claim;
    /// `docs/reference-sweeps/findings.md` finding 5 is the split, and ⛔ no
    /// member of the JA4+ family is computed anywhere in this tree.
    #[must_use]
    pub fn ja4_lists(&self) -> Ja4Lists {
        let ciphers: Vec<u16> = self
            .cipher_suites
            .iter()
            .copied()
            .filter(|c| !is_grease_value(*c))
            .collect();
        let extensions: Vec<u16> = self
            .extensions
            .iter()
            .map(|e| e.codepoint)
            .filter(|c| !is_grease_value(*c))
            .collect();
        let sigalgs: Vec<u16> = self
            .signature_algorithms
            .iter()
            .copied()
            .filter(|s| !is_grease_value(*s))
            .collect();

        let mut sorted_ciphers = ciphers.clone();
        sorted_ciphers.sort_unstable();
        // ⛔ SNI AND ALPN ARE REMOVED FROM THE HASHED LIST AND NOT FROM THE
        // COUNT. The specification says so in as many words: they are already
        // in the prefix, so one application produces the same third section
        // whether it went to a domain or to an address.
        let mut sorted_extensions: Vec<u16> = extensions
            .iter()
            .copied()
            .filter(|c| *c != 0x0000 && *c != 0x0010)
            .collect();
        sorted_extensions.sort_unstable();

        let sig_text = join_hex(&sigalgs);
        // ⚠ NO TRAILING UNDERSCORE WHERE THERE ARE NO SIGNATURE ALGORITHMS.
        let with_sigs = |list: String| {
            if sig_text.is_empty() {
                list
            } else {
                format!("{list}_{sig_text}")
            }
        };

        Ja4Lists {
            prefix: self.ja4_prefix(),
            ciphers_sorted: join_hex(&sorted_ciphers),
            extensions_sorted: with_sigs(join_hex(&sorted_extensions)),
            ciphers_original: join_hex(&ciphers),
            extensions_original: with_sigs(join_hex(&extensions)),
        }
    }

    /// JA4's raw form, with both lists sorted.
    #[must_use]
    pub fn ja4_r(&self) -> String {
        let lists = self.ja4_lists();
        format!(
            "{}_{}_{}",
            lists.prefix, lists.ciphers_sorted, lists.extensions_sorted
        )
    }

    /// JA4's order-preserving raw form.
    ///
    /// ⚠ **It shows order and nothing about GREASE**, because the specification
    /// strips GREASE here too. The raw `ClientHello` is the one artefact in
    /// which a GREASE question is answerable at all, which is what
    /// `docs/inherited-claims.md` section 10 corrects.
    #[must_use]
    pub fn ja4_ro(&self) -> String {
        let lists = self.ja4_lists();
        format!(
            "{}_{}_{}",
            lists.prefix, lists.ciphers_original, lists.extensions_original
        )
    }

    /// The first section of a JA4 fingerprint.
    ///
    /// ⚠ **`t` and never `q` or `d`.** This project captures TLS over TCP; a
    /// QUIC or DTLS capture would need the transport recorded, and neither is
    /// something this harness can take today.
    #[must_use]
    pub fn ja4_prefix(&self) -> String {
        let version = self.ja4_version();
        let sni = if self.extensions.iter().any(|e| e.codepoint == 0x0000) {
            'd'
        } else {
            'i'
        };
        let ciphers = self
            .cipher_suites
            .iter()
            .filter(|c| !is_grease_value(**c))
            .count();
        // ⚠ SNI AND ALPN ARE COUNTED HERE, and removed from the hashed list.
        let extensions = self.extensions.iter().filter(|e| !e.is_grease()).count();
        format!(
            "t{version}{sni}{:02}{:02}{}",
            ciphers.min(99),
            extensions.min(99),
            self.ja4_alpn()
        )
    }

    /// The two characters the ALPN contributes.
    ///
    /// ⛔ **The byte rule, not the string rule, and the difference is the whole
    /// of it.** Where the first or the last byte of the first protocol is not
    /// ASCII alphanumeric, the pair becomes the FIRST character of that first
    /// byte's hex and the LAST character of the last byte's hex. Derived from
    /// the eight worked examples in the specification, all of which this
    /// reading reproduces and one of which reads as a counter-example until
    /// the rule is stated per BYTE rather than per string.
    #[must_use]
    pub fn ja4_alpn(&self) -> String {
        let Some(first) = self.alpn.first() else {
            return "00".to_owned();
        };
        let bytes = first.as_bytes();
        let (Some(head), Some(tail)) = (bytes.first(), bytes.last()) else {
            return "00".to_owned();
        };
        if head.is_ascii_alphanumeric() && tail.is_ascii_alphanumeric() {
            return format!("{}{}", char::from(*head), char::from(*tail));
        }
        let head_hex = format!("{head:02x}");
        let tail_hex = format!("{tail:02x}");
        format!(
            "{}{}",
            head_hex.chars().next().unwrap_or('0'),
            tail_hex.chars().last().unwrap_or('0')
        )
    }

    /// The two characters the TLS version contributes.
    ///
    /// ⚠ **`supported_versions` wins where it exists, and the fallback is the
    /// HANDSHAKE version rather than the record layer's.** The specification's
    /// wording is ambiguous about which; the reference implementation resolves
    /// it at `references/FoxIO-LLC__ja4/tree/rust/ja4/src/tls.rs:573`, whose
    /// own comment says the field is not to be confused with the record
    /// version. ⛔ Read to settle an ambiguity, never copied.
    #[must_use]
    pub fn ja4_version(&self) -> &'static str {
        let highest = self
            .extensions
            .iter()
            .find(|e| e.codepoint == 0x002b)
            .and_then(|e| supported_versions(&e.body_hex))
            .unwrap_or(self.legacy_version);
        match highest {
            0x0304 => "13",
            0x0303 => "12",
            0x0302 => "11",
            0x0301 => "10",
            0x0300 => "s3",
            0x0002 => "s2",
            0xfeff => "d1",
            0xfefd => "d2",
            0xfefc => "d3",
            _ => "00",
        }
    }
}

/// Four-character lowercase hex codepoints, comma delimited.
fn join_hex(values: &[u16]) -> String {
    values
        .iter()
        .map(|v| format!("{v:04x}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// The highest non-GREASE version a `supported_versions` body offers.
///
/// ⚠ **A client's body is a one-byte length then two-byte versions.** A body
/// that does not parse produces [`None`] rather than a guess, and the caller
/// then falls back to the handshake version, which is the specification's own
/// rule for an absent extension.
fn supported_versions(body_hex: &str) -> Option<u16> {
    let bytes = decode_hex(body_hex)?;
    let (&declared, rest) = bytes.split_first()?;
    if usize::from(declared) != rest.len() || rest.len() % 2 != 0 {
        return None;
    }
    rest.as_chunks::<2>()
        .0
        .iter()
        .map(|pair| (u16::from(pair[0]) << 8) | u16::from(pair[1]))
        .filter(|v| !is_grease_value(*v))
        .max()
}

/// Decode an even-length hex string, or nothing.
fn decode_hex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    let raw = text.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        let hi = char::from(raw[i]).to_digit(16)?;
        let lo = char::from(raw[i + 1]).to_digit(16)?;
        out.push(u8::try_from(hi * 16 + lo).ok()?);
        i += 2;
    }
    Some(out)
}
