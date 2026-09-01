//! HPACK: the header compression HTTP/2 header order sits behind.
//!
//! ⛔ **Header order is a first-class part of the fingerprint and it is not
//! readable without this.** Names arrive Huffman-coded and indexed against a
//! table the connection builds as it goes, so a reader without a decoder sees
//! opaque bytes.
//!
//! ⛔ **Whether each field was Huffman-coded is RECORDED, not discarded.** It
//! is a choice the encoder made, it differs between clients, and it is exactly
//! the kind of thing that cannot be added later because the capture is gone.
//! The same goes for which of the four indexing forms the encoder picked.
//!
//! ⚠ **This decoder is checked against a fetched vector corpus rather than
//! against itself.** A Huffman decoder that is subtly wrong produces plausible
//! header names, which is the failure that survives review. The corpus is
//! `references/http2jp__hpack-test-case/` at the commit its `PROVENANCE.md`
//! names.
//!
//! ⛔ **Nothing here filters a credential, and that is deliberate.** The
//! dynamic table has to see every field the encoder inserted or every later
//! index is wrong, so a decoder that dropped `cookie` would decode the rest of
//! the connection into nonsense. The credential rule is applied where the
//! capture is BUILT, and there is a test that the table saw what the capture
//! did not.

use std::collections::VecDeque;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// The protocol's own starting size for the dynamic table, in octets.
pub const DEFAULT_TABLE_SIZE: usize = 4096;

/// The per-entry overhead the specification adds to a name and a value.
pub const ENTRY_OVERHEAD: usize = 32;

/// The static table, index 1 through 61.
///
/// ⚠ It contains `cookie`, `authorization` and `set-cookie`, so a decoder that
/// resolves an index can produce a credential's name from one byte on the wire.
/// The rule that keeps those out of a capture is applied at the capture, not
/// here.
pub const STATIC_TABLE: [(&str, &str); 61] = [
    (":authority", ""),
    (":method", "GET"),
    (":method", "POST"),
    (":path", "/"),
    (":path", "/index.html"),
    (":scheme", "http"),
    (":scheme", "https"),
    (":status", "200"),
    (":status", "204"),
    (":status", "206"),
    (":status", "304"),
    (":status", "400"),
    (":status", "404"),
    (":status", "500"),
    ("accept-charset", ""),
    ("accept-encoding", "gzip, deflate"),
    ("accept-language", ""),
    ("accept-ranges", ""),
    ("accept", ""),
    ("access-control-allow-origin", ""),
    ("age", ""),
    ("allow", ""),
    ("authorization", ""),
    ("cache-control", ""),
    ("content-disposition", ""),
    ("content-encoding", ""),
    ("content-language", ""),
    ("content-length", ""),
    ("content-location", ""),
    ("content-range", ""),
    ("content-type", ""),
    ("cookie", ""),
    ("date", ""),
    ("etag", ""),
    ("expect", ""),
    ("expires", ""),
    ("from", ""),
    ("host", ""),
    ("if-match", ""),
    ("if-modified-since", ""),
    ("if-none-match", ""),
    ("if-range", ""),
    ("if-unmodified-since", ""),
    ("last-modified", ""),
    ("link", ""),
    ("location", ""),
    ("max-forwards", ""),
    ("proxy-authenticate", ""),
    ("proxy-authorization", ""),
    ("range", ""),
    ("referer", ""),
    ("refresh", ""),
    ("retry-after", ""),
    ("server", ""),
    ("set-cookie", ""),
    ("strict-transport-security", ""),
    ("transfer-encoding", ""),
    ("user-agent", ""),
    ("vary", ""),
    ("via", ""),
    ("www-authenticate", ""),
];

/// The Huffman code, as `(code, bit length)` per symbol, 0 through 255 and then
/// the end-of-string symbol.
///
/// ⛔ Transcribed from the specification's own table and checked against the
/// vector corpus rather than by reading. A wrong row here produces a plausible
/// header name, which is the defect that survives a review.
const HUFFMAN: [(u32, u8); 257] = [
    (0x1ff8, 13),
    (0x7fffd8, 23),
    (0xfffffe2, 28),
    (0xfffffe3, 28),
    (0xfffffe4, 28),
    (0xfffffe5, 28),
    (0xfffffe6, 28),
    (0xfffffe7, 28),
    (0xfffffe8, 28),
    (0xffffea, 24),
    (0x3ffffffc, 30),
    (0xfffffe9, 28),
    (0xfffffea, 28),
    (0x3ffffffd, 30),
    (0xfffffeb, 28),
    (0xfffffec, 28),
    (0xfffffed, 28),
    (0xfffffee, 28),
    (0xfffffef, 28),
    (0xffffff0, 28),
    (0xffffff1, 28),
    (0xffffff2, 28),
    (0x3ffffffe, 30),
    (0xffffff3, 28),
    (0xffffff4, 28),
    (0xffffff5, 28),
    (0xffffff6, 28),
    (0xffffff7, 28),
    (0xffffff8, 28),
    (0xffffff9, 28),
    (0xffffffa, 28),
    (0xffffffb, 28),
    (0x14, 6),
    (0x3f8, 10),
    (0x3f9, 10),
    (0xffa, 12),
    (0x1ff9, 13),
    (0x15, 6),
    (0xf8, 8),
    (0x7fa, 11),
    (0x3fa, 10),
    (0x3fb, 10),
    (0xf9, 8),
    (0x7fb, 11),
    (0xfa, 8),
    (0x16, 6),
    (0x17, 6),
    (0x18, 6),
    (0x0, 5),
    (0x1, 5),
    (0x2, 5),
    (0x19, 6),
    (0x1a, 6),
    (0x1b, 6),
    (0x1c, 6),
    (0x1d, 6),
    (0x1e, 6),
    (0x1f, 6),
    (0x5c, 7),
    (0xfb, 8),
    (0x7ffc, 15),
    (0x20, 6),
    (0xffb, 12),
    (0x3fc, 10),
    (0x1ffa, 13),
    (0x21, 6),
    (0x5d, 7),
    (0x5e, 7),
    (0x5f, 7),
    (0x60, 7),
    (0x61, 7),
    (0x62, 7),
    (0x63, 7),
    (0x64, 7),
    (0x65, 7),
    (0x66, 7),
    (0x67, 7),
    (0x68, 7),
    (0x69, 7),
    (0x6a, 7),
    (0x6b, 7),
    (0x6c, 7),
    (0x6d, 7),
    (0x6e, 7),
    (0x6f, 7),
    (0x70, 7),
    (0x71, 7),
    (0x72, 7),
    (0xfc, 8),
    (0x73, 7),
    (0xfd, 8),
    (0x1ffb, 13),
    (0x7fff0, 19),
    (0x1ffc, 13),
    (0x3ffc, 14),
    (0x22, 6),
    (0x7ffd, 15),
    (0x3, 5),
    (0x23, 6),
    (0x4, 5),
    (0x24, 6),
    (0x5, 5),
    (0x25, 6),
    (0x26, 6),
    (0x27, 6),
    (0x6, 5),
    (0x74, 7),
    (0x75, 7),
    (0x28, 6),
    (0x29, 6),
    (0x2a, 6),
    (0x7, 5),
    (0x2b, 6),
    (0x76, 7),
    (0x2c, 6),
    (0x8, 5),
    (0x9, 5),
    (0x2d, 6),
    (0x77, 7),
    (0x78, 7),
    (0x79, 7),
    (0x7a, 7),
    (0x7b, 7),
    (0x7ffe, 15),
    (0x7fc, 11),
    (0x3ffd, 14),
    (0x1ffd, 13),
    (0xffffffc, 28),
    (0xfffe6, 20),
    (0x3fffd2, 22),
    (0xfffe7, 20),
    (0xfffe8, 20),
    (0x3fffd3, 22),
    (0x3fffd4, 22),
    (0x3fffd5, 22),
    (0x7fffd9, 23),
    (0x3fffd6, 22),
    (0x7fffda, 23),
    (0x7fffdb, 23),
    (0x7fffdc, 23),
    (0x7fffdd, 23),
    (0x7fffde, 23),
    (0xffffeb, 24),
    (0x7fffdf, 23),
    (0xffffec, 24),
    (0xffffed, 24),
    (0x3fffd7, 22),
    (0x7fffe0, 23),
    (0xffffee, 24),
    (0x7fffe1, 23),
    (0x7fffe2, 23),
    (0x7fffe3, 23),
    (0x7fffe4, 23),
    (0x1fffdc, 21),
    (0x3fffd8, 22),
    (0x7fffe5, 23),
    (0x3fffd9, 22),
    (0x7fffe6, 23),
    (0x7fffe7, 23),
    (0xffffef, 24),
    (0x3fffda, 22),
    (0x1fffdd, 21),
    (0xfffe9, 20),
    (0x3fffdb, 22),
    (0x3fffdc, 22),
    (0x7fffe8, 23),
    (0x7fffe9, 23),
    (0x1fffde, 21),
    (0x7fffea, 23),
    (0x3fffdd, 22),
    (0x3fffde, 22),
    (0xfffff0, 24),
    (0x1fffdf, 21),
    (0x3fffdf, 22),
    (0x7fffeb, 23),
    (0x7fffec, 23),
    (0x1fffe0, 21),
    (0x1fffe1, 21),
    (0x3fffe0, 22),
    (0x1fffe2, 21),
    (0x7fffed, 23),
    (0x3fffe1, 22),
    (0x7fffee, 23),
    (0x7fffef, 23),
    (0xfffea, 20),
    (0x3fffe2, 22),
    (0x3fffe3, 22),
    (0x3fffe4, 22),
    (0x7ffff0, 23),
    (0x3fffe5, 22),
    (0x3fffe6, 22),
    (0x7ffff1, 23),
    (0x3ffffe0, 26),
    (0x3ffffe1, 26),
    (0xfffeb, 20),
    (0x7fff1, 19),
    (0x3fffe7, 22),
    (0x7ffff2, 23),
    (0x3fffe8, 22),
    (0x1ffffec, 25),
    (0x3ffffe2, 26),
    (0x3ffffe3, 26),
    (0x3ffffe4, 26),
    (0x7ffffde, 27),
    (0x7ffffdf, 27),
    (0x3ffffe5, 26),
    (0xfffff1, 24),
    (0x1ffffed, 25),
    (0x7fff2, 19),
    (0x1fffe3, 21),
    (0x3ffffe6, 26),
    (0x7ffffe0, 27),
    (0x7ffffe1, 27),
    (0x3ffffe7, 26),
    (0x7ffffe2, 27),
    (0xfffff2, 24),
    (0x1fffe4, 21),
    (0x1fffe5, 21),
    (0x3ffffe8, 26),
    (0x3ffffe9, 26),
    (0xffffffd, 28),
    (0x7ffffe3, 27),
    (0x7ffffe4, 27),
    (0x7ffffe5, 27),
    (0xfffec, 20),
    (0xfffff3, 24),
    (0xfffed, 20),
    (0x1fffe6, 21),
    (0x3fffe9, 22),
    (0x1fffe7, 21),
    (0x1fffe8, 21),
    (0x7ffff3, 23),
    (0x3fffea, 22),
    (0x3fffeb, 22),
    (0x1ffffee, 25),
    (0x1ffffef, 25),
    (0xfffff4, 24),
    (0xfffff5, 24),
    (0x3ffffea, 26),
    (0x7ffff4, 23),
    (0x3ffffeb, 26),
    (0x7ffffe6, 27),
    (0x3ffffec, 26),
    (0x3ffffed, 26),
    (0x7ffffe7, 27),
    (0x7ffffe8, 27),
    (0x7ffffe9, 27),
    (0x7ffffea, 27),
    (0x7ffffeb, 27),
    (0xffffffe, 28),
    (0x7ffffec, 27),
    (0x7ffffed, 27),
    (0x7ffffee, 27),
    (0x7ffffef, 27),
    (0x7fffff0, 27),
    (0x3ffffee, 26),
    (0x3fffffff, 30),
];

/// The end-of-string symbol, which may never appear inside a coded string.
const EOS: u16 = 256;

/// The canonical decoding tables, derived once from [`HUFFMAN`].
///
/// ⭐ Derived rather than transcribed a second time. The code is canonical, so
/// a first code and a first index per bit length is enough to decode, and
/// re-deriving it means the two halves cannot disagree.
struct Canonical {
    /// Symbols sorted by bit length and then by code.
    symbols: Vec<u16>,
    /// How many codes have each bit length, indexed by length.
    count: [u16; 31],
    /// The first code of each bit length.
    first_code: [u32; 31],
    /// Where each bit length's symbols begin in `symbols`.
    first_index: [usize; 31],
}

fn canonical() -> &'static Canonical {
    static TABLES: OnceLock<Canonical> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut order: Vec<u16> = (0..257).collect();
        order.sort_by_key(|&s| {
            let (code, bits) = HUFFMAN[s as usize];
            (bits, code)
        });
        let mut count = [0_u16; 31];
        for &(_, bits) in &HUFFMAN {
            count[bits as usize] += 1;
        }
        let mut first_code = [0_u32; 31];
        let mut first_index = [0_usize; 31];
        let mut index = 0_usize;
        for bits in 1..31 {
            first_index[bits] = index;
            if count[bits] > 0 {
                // ⛔ Read the TRANSCRIBED code of the lowest-coded symbol at
                // this length rather than deriving it from the counts. A code
                // derived purely from the counts makes the code column of
                // [`HUFFMAN`] load-bearing only through its sort order, so a
                // wrong row that happens to preserve that order decodes
                // correctly and nothing can see the mistake. Found by mutating
                // one row and watching the whole vector corpus still pass.
                first_code[bits] = HUFFMAN[order[index] as usize].0;
            }
            index += count[bits] as usize;
        }
        Canonical {
            symbols: order,
            count,
            first_code,
            first_index,
        }
    })
}

/// Check that the transcribed table is a canonical Huffman code.
///
/// ⭐ **The decoder rests on this and nothing else states it.** Canonical
/// decoding counts forward from the first code at each bit length, which is
/// only correct where the codes at each length are consecutive and ascending in
/// symbol order. The specification's table is; this says so out of the table
/// rather than out of a comment.
///
/// # Errors
///
/// The first symbol whose transcribed code disagrees with the construction.
pub fn check_table_is_canonical() -> Result<(), String> {
    let tables = canonical();
    let mut code = 0_u32;
    let mut index = 0_usize;
    for bits in 1..31_usize {
        code <<= 1;
        for slot in 0..usize::from(tables.count[bits]) {
            let symbol = tables.symbols[index + slot];
            let (transcribed, transcribed_bits) = HUFFMAN[symbol as usize];
            let expected = code + u32::try_from(slot).map_err(|_| "a slot past u32".to_owned())?;
            if usize::from(transcribed_bits) != bits || transcribed != expected {
                return Err(format!(
                    "symbol {symbol} is transcribed as {transcribed:#x} over {transcribed_bits} \
                     bit(s), and the canonical construction puts it at {expected:#x} over {bits}"
                ));
            }
        }
        code += u32::from(tables.count[bits]);
        index += usize::from(tables.count[bits]);
    }
    Ok(())
}

/// Decode a Huffman-coded string.
///
/// # Errors
///
/// A code that is not in the table, an end-of-string symbol inside the string,
/// or padding that is longer than seven bits or is not all ones. All three are
/// decoding errors the specification names, and each is a client doing
/// something a capture should not paper over.
pub fn decode_huffman(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let tables = canonical();
    let mut out = Vec::with_capacity(bytes.len() * 8 / 5);
    let mut code = 0_u32;
    let mut bits = 0_usize;
    for byte in bytes {
        for shift in (0..8).rev() {
            code = (code << 1) | u32::from((byte >> shift) & 1);
            bits += 1;
            if bits > 30 {
                return Err(format!(
                    "no Huffman code is longer than 30 bits, and {bits} were read"
                ));
            }
            let length = bits;
            if tables.count[length] == 0 {
                continue;
            }
            let offset = code.wrapping_sub(tables.first_code[length]);
            if offset >= u32::from(tables.count[length]) {
                continue;
            }
            let symbol = tables.symbols[tables.first_index[length] + offset as usize];
            if symbol == EOS {
                return Err("the end-of-string symbol appears inside the string".to_owned());
            }
            out.push(u8::try_from(symbol).map_err(|_| "a symbol above 255".to_owned())?);
            code = 0;
            bits = 0;
        }
    }
    // ⛔ The remainder must be the most significant bits of the end-of-string
    // code, which are all ones, and there must be fewer than eight of them.
    if bits >= 8 {
        return Err(format!(
            "{bits} bits of padding, and padding is under eight"
        ));
    }
    if bits > 0 && code != (1 << bits) - 1 {
        return Err(format!(
            "{bits} bits of padding that are not all ones, so they are not the end-of-string code"
        ));
    }
    Ok(out)
}

/// Which of the four forms the encoder used for one field.
///
/// ⛔ Recorded because it is a choice the encoder made. Two clients sending the
/// same headers by different forms are two visibly different connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Indexing {
    /// Name and value both came from a table index.
    Indexed,
    /// A literal, added to the dynamic table.
    Incremental,
    /// A literal, not added to the dynamic table.
    WithoutIndexing,
    /// A literal an intermediary may never add to a table.
    NeverIndexed,
}

/// One header field, with what HPACK did to it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderRecord {
    /// The field name.
    pub name: String,
    /// The field value.
    ///
    /// ⛔ `None` under the names-only policy, which is the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Whether the NAME was Huffman-coded, where it was sent as a literal.
    ///
    /// ⚠ `None` means the name came from an index and was not coded at all,
    /// which is a third state rather than a false.
    pub name_huffman: Option<bool>,
    /// Whether the VALUE was Huffman-coded, where it was sent as a literal.
    pub value_huffman: Option<bool>,
    /// Which form the encoder used.
    pub indexing: Indexing,
}

/// A decoded header block, with the connection state it changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    /// The fields, in the order they arrived.
    pub fields: Vec<HeaderRecord>,
    /// Every dynamic table size update the block carried, in order.
    ///
    /// ⭐ A choice the encoder made about a table it owns, and one that a
    /// settings value does not predict.
    pub table_size_updates: Vec<usize>,
    /// Field positions whose bytes were not valid UTF-8.
    ///
    /// ⚠ HPACK carries octets. A name or value that is not text is kept as the
    /// lossy conversion here, and the raw bytes are still in the capture.
    pub not_utf8: Vec<usize>,
}

/// The decoder for one connection.
///
/// ⛔ **One decoder per connection, never one per frame.** The dynamic table is
/// connection state: a decoder reset between header blocks resolves every later
/// index against the wrong table and produces plausible nonsense.
#[derive(Debug, Clone)]
pub struct Decoder {
    dynamic: VecDeque<(Vec<u8>, Vec<u8>)>,
    size: usize,
    max_size: usize,
    settings_max: usize,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::with_settings_max(DEFAULT_TABLE_SIZE)
    }
}

impl Decoder {
    /// A decoder whose peer may size the table up to `settings_max` octets.
    #[must_use]
    pub fn with_settings_max(settings_max: usize) -> Self {
        Self {
            dynamic: VecDeque::new(),
            size: 0,
            max_size: settings_max,
            settings_max,
        }
    }

    /// The dynamic table's current entry count.
    #[must_use]
    pub fn entries(&self) -> usize {
        self.dynamic.len()
    }

    /// The dynamic table's current size in octets.
    #[must_use]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Raise or lower the size the peer's encoder may ask for.
    ///
    /// ⛔ **The CEILING only.** The table's own maximum moves when the wire
    /// carries a size update and at no other time, so changing it here would
    /// evict entries the encoder still has and mis-resolve every later index.
    ///
    /// ⚠ This is what a SETTINGS frame from OUR side would say. The harness
    /// sends none, so a peer assumes the protocol default until told otherwise.
    pub fn set_settings_max(&mut self, settings_max: usize) {
        self.settings_max = settings_max;
    }

    /// Decode one header block.
    ///
    /// # Errors
    ///
    /// An index that names nothing, a truncated field, a size update above what
    /// the peer was allowed, or a Huffman string that does not decode. Each is
    /// a connection error by the specification rather than something to guess
    /// past: a wrong guess here silently mis-decodes every later field.
    pub fn decode(&mut self, block: &[u8]) -> Result<Decoded, String> {
        let mut at = 0_usize;
        let mut out = Decoded {
            fields: Vec::new(),
            table_size_updates: Vec::new(),
            not_utf8: Vec::new(),
        };
        while at < block.len() {
            let first = block[at];
            if first & 0x80 != 0 {
                let index = read_integer(block, &mut at, 0x7f)?;
                let (name, value) = self.lookup(index)?;
                self.push_field(&mut out, name, value, None, None, Indexing::Indexed);
            } else if first & 0xc0 == 0x40 {
                let index = read_integer(block, &mut at, 0x3f)?;
                let (name, name_huffman) = self.name_of(block, &mut at, index)?;
                let (value, value_huffman) = read_string(block, &mut at)?;
                self.insert(name.clone(), value.clone());
                self.push_field(
                    &mut out,
                    name,
                    value,
                    name_huffman,
                    Some(value_huffman),
                    Indexing::Incremental,
                );
            } else if first & 0xe0 == 0x20 {
                let requested = read_integer(block, &mut at, 0x1f)?;
                let requested = usize::try_from(requested).unwrap_or(usize::MAX);
                if requested > self.settings_max {
                    return Err(format!(
                        "a dynamic table size update asks for {requested} octets and the peer was \
                         allowed {}",
                        self.settings_max
                    ));
                }
                self.resize(requested);
                out.table_size_updates.push(requested);
            } else {
                let never = first & 0xf0 == 0x10;
                let index = read_integer(block, &mut at, 0x0f)?;
                let (name, name_huffman) = self.name_of(block, &mut at, index)?;
                let (value, value_huffman) = read_string(block, &mut at)?;
                self.push_field(
                    &mut out,
                    name,
                    value,
                    name_huffman,
                    Some(value_huffman),
                    if never {
                        Indexing::NeverIndexed
                    } else {
                        Indexing::WithoutIndexing
                    },
                );
            }
        }
        Ok(out)
    }

    fn push_field(
        &self,
        out: &mut Decoded,
        name: Vec<u8>,
        value: Vec<u8>,
        name_huffman: Option<bool>,
        value_huffman: Option<bool>,
        indexing: Indexing,
    ) {
        let position = out.fields.len();
        if std::str::from_utf8(&name).is_err() || std::str::from_utf8(&value).is_err() {
            out.not_utf8.push(position);
        }
        out.fields.push(HeaderRecord {
            name: String::from_utf8_lossy(&name).into_owned(),
            value: Some(String::from_utf8_lossy(&value).into_owned()),
            name_huffman,
            value_huffman,
            indexing,
        });
    }

    /// A name taken from an index, or read as a literal where the index is 0.
    fn name_of(
        &self,
        block: &[u8],
        at: &mut usize,
        index: u64,
    ) -> Result<(Vec<u8>, Option<bool>), String> {
        if index == 0 {
            let (name, huffman) = read_string(block, at)?;
            return Ok((name, Some(huffman)));
        }
        let (name, _) = self.lookup(index)?;
        // ⚠ A name taken from an index was not CODED at all, so the flag is
        // absent rather than false. False would say the encoder chose plain
        // text, and it made no such choice.
        Ok((name, None))
    }

    fn lookup(&self, index: u64) -> Result<(Vec<u8>, Vec<u8>), String> {
        if index == 0 {
            return Err("index 0 names no header field".to_owned());
        }
        let index = usize::try_from(index).unwrap_or(usize::MAX);
        if index <= STATIC_TABLE.len() {
            let (name, value) = STATIC_TABLE[index - 1];
            return Ok((name.as_bytes().to_vec(), value.as_bytes().to_vec()));
        }
        let dynamic_index = index - STATIC_TABLE.len() - 1;
        self.dynamic.get(dynamic_index).cloned().ok_or_else(|| {
            format!(
                "index {index} is past the end of a dynamic table holding {} entr(ies)",
                self.dynamic.len()
            )
        })
    }

    fn insert(&mut self, name: Vec<u8>, value: Vec<u8>) {
        let entry_size = name.len() + value.len() + ENTRY_OVERHEAD;
        // ⛔ An entry larger than the whole table empties it and is not stored.
        // That is the specification's rule and it is not an error.
        while self.size + entry_size > self.max_size {
            match self.dynamic.pop_back() {
                Some((old_name, old_value)) => {
                    self.size = self
                        .size
                        .saturating_sub(old_name.len() + old_value.len() + ENTRY_OVERHEAD);
                }
                None => break,
            }
        }
        if entry_size > self.max_size {
            return;
        }
        self.size += entry_size;
        self.dynamic.push_front((name, value));
    }

    fn resize(&mut self, max_size: usize) {
        self.max_size = max_size;
        while self.size > self.max_size {
            match self.dynamic.pop_back() {
                Some((name, value)) => {
                    self.size = self
                        .size
                        .saturating_sub(name.len() + value.len() + ENTRY_OVERHEAD);
                }
                None => break,
            }
        }
    }
}

/// Read a prefixed integer, advancing past it.
///
/// ⚠ The prefix is a MASK rather than a width, because the specification writes
/// the pattern that identifies the field in the same byte.
fn read_integer(block: &[u8], at: &mut usize, mask: u8) -> Result<u64, String> {
    let first = *block
        .get(*at)
        .ok_or_else(|| "an integer ran off the end of the block".to_owned())?;
    *at += 1;
    let mut value = u64::from(first & mask);
    if value < u64::from(mask) {
        return Ok(value);
    }
    let mut shift = 0_u32;
    loop {
        let byte = *block
            .get(*at)
            .ok_or_else(|| "a multi-byte integer ran off the end of the block".to_owned())?;
        *at += 1;
        // ⛔ Refuse rather than wrap. An integer that overflows here is a peer
        // asking for a table of impossible size, and a wrapped value would be
        // acted on as though it were small.
        let addend = u64::from(byte & 0x7f)
            .checked_shl(shift)
            .ok_or_else(|| "an integer longer than this decoder will represent".to_owned())?;
        value = value
            .checked_add(addend)
            .ok_or_else(|| "an integer longer than this decoder will represent".to_owned())?;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
        shift += 7;
        if shift > 56 {
            return Err("an integer longer than this decoder will represent".to_owned());
        }
    }
}

/// Read a string literal, returning its bytes and whether it was Huffman-coded.
fn read_string(block: &[u8], at: &mut usize) -> Result<(Vec<u8>, bool), String> {
    let huffman = block
        .get(*at)
        .ok_or_else(|| "a string ran off the end of the block".to_owned())?
        & 0x80
        != 0;
    let length = read_integer(block, at, 0x7f)?;
    let length = usize::try_from(length)
        .map_err(|_| "a string longer than this platform can address".to_owned())?;
    let end = at
        .checked_add(length)
        .ok_or_else(|| "a string length that overflows the block".to_owned())?;
    let raw = block.get(*at..end).ok_or_else(|| {
        format!(
            "a string declares {length} byte(s) and {} remain in the block",
            block.len().saturating_sub(*at)
        )
    })?;
    *at = end;
    if huffman {
        Ok((decode_huffman(raw)?, true))
    } else {
        Ok((raw.to_vec(), false))
    }
}
