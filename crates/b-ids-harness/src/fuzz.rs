//! Feeding hostile bytes to every parser, from one place.
//!
//! ⛔ **The harness reads bytes chosen by whoever connects to it.** A parser
//! that panics on a truncated or malformed record is a parser anybody who can
//! reach the listener can stop. There is no authentication in front of it and
//! there cannot be: the whole point is to accept what an unknown client sends.
//!
//! ⭐ **One function drives every parser, and both the fuzz target and the
//! suite call it.** A fuzz target with its own list of parsers is a list that
//! goes stale the day a fifth parser lands, and the staleness is invisible
//! because the target keeps passing.
//!
//! ⚠ **The property is the ABSENCE of a panic, not the result.** Every parser
//! here returns a `Result` or an `Option`, so any outcome is acceptable; what is
//! not acceptable is unwinding. Nothing below inspects what came back, on
//! purpose: an assertion about the value would make this a test of the parse
//! rather than of the process surviving.
//!
//! `docs/history/todo/harness.md`, `HARNESS-09`.

use b_ids_schema::http::ValuePolicy;

use crate::hpack::Decoder;

/// Feed one byte string to every parser this crate exposes to the network.
///
/// ⛔ **Every parser, and the list is here rather than in a caller.** The four
/// the entry names are the record layer, the `ClientHello`, the HTTP/2 frame
/// reader and the HPACK decoder; the Huffman decoder is the fifth, reached both
/// through the HPACK decoder and directly, because a caller can hand it a
/// literal that is Huffman-coded and nothing else.
///
/// ⚠ **The HTTP/2 reader is fed twice**, once with the bytes as they are and
/// once with the connection preface in front of them. Without the second, every
/// input would be refused at the preface check and the frame reader behind it
/// would never be reached at all, which is a fuzz target that exercises one
/// comparison.
pub fn drive_every_parser(bytes: &[u8]) {
    let _ = crate::hello::parse_record(bytes);

    let _ = crate::h2::starts_like_preface(bytes);
    let _ = crate::h2::first_header_block_complete(bytes);

    let mut notes = Vec::new();
    let _ = crate::h2::parse_connection(bytes, ValuePolicy::NamesOnly, &mut notes);

    let mut with_preface = Vec::with_capacity(crate::h2::PREFACE.len() + bytes.len());
    with_preface.extend_from_slice(crate::h2::PREFACE);
    with_preface.extend_from_slice(bytes);
    let mut notes = Vec::new();
    let _ = crate::h2::parse_connection(&with_preface, ValuePolicy::WithValues, &mut notes);
    let _ = crate::h2::first_header_block_complete(&with_preface);

    let _ = crate::hpack::decode_huffman(bytes);

    // ⚠ A FRESH DECODER each time. The dynamic table is connection state, and a
    // decoder carried across inputs would make every case depend on the ones
    // before it, so a crash would not be reproducible from its own input.
    let mut decoder = Decoder::default();
    let _ = decoder.decode(bytes);

    // ⚠ And one with a small table limit, because the eviction path is only
    // reached when the table is full and the default limit is large enough that
    // random input never fills it.
    let mut small = Decoder::with_settings_max(64);
    let _ = small.decode(bytes);
}

/// A deterministic pseudo-random source.
///
/// ⛔ **Seeded and reproducible, never a clock.** A randomised test whose input
/// depends on when it ran is a test that fails once and cannot be re-run, which
/// is the worst kind: the failure is real and the evidence is gone.
///
/// ⚠ **xorshift64, not a cryptographic generator.** What is needed is a spread
/// of bytes that is the same on every host, and a dependency for that would be
/// a dependency in the one crate that faces the network.
#[derive(Debug)]
pub struct Rng(u64);

impl Rng {
    /// Start from a seed. ⚠ Zero is replaced, because xorshift is stuck there.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(if seed == 0 {
            0x9e37_79b9_7f4a_7c15
        } else {
            seed
        })
    }

    /// The next value.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// The next byte.
    pub fn next_u8(&mut self) -> u8 {
        u8::try_from(self.next_u64() & 0xff).unwrap_or(0)
    }

    /// A value below `bound`, or zero where `bound` is zero.
    pub fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            usize::try_from(self.next_u64() % bound as u64).unwrap_or(0)
        }
    }
}

/// Every input a corpus of seeds produces: truncations, byte flips, and random
/// bytes.
///
/// ⭐ **Mutations of real captures, not only random bytes.** Random input almost
/// never survives the first length check, so a target fed only random bytes
/// exercises one comparison and reports success. A truncated real hello reaches
/// every field the parser reads, which is where a slice past the end lives.
///
/// ⚠ **Truncation is the single highest-value mutation here**, because every
/// parser in this crate reads a declared length and then takes bytes behind it.
#[must_use]
pub fn cases(seeds: &[Vec<u8>], rng: &mut Rng, random_cases: usize) -> Vec<Vec<u8>> {
    let mut out = Vec::new();

    for seed in seeds {
        // Every prefix, including the empty one.
        for take in 0..=seed.len() {
            out.push(seed[..take].to_vec());
        }
        // One flipped bit at every byte, at a position the generator chooses,
        // so the set is wide without being the whole cross product.
        for index in 0..seed.len() {
            let mut mutated = seed.clone();
            mutated[index] ^= 1 << (rng.next_u8() % 8);
            out.push(mutated);
        }
        // A byte replaced with an extreme value, which is what a length field
        // claiming more than arrived looks like.
        for index in 0..seed.len() {
            for value in [0x00_u8, 0xff] {
                let mut mutated = seed.clone();
                mutated[index] = value;
                out.push(mutated);
            }
        }
    }

    for _ in 0..random_cases {
        let len = rng.below(600);
        out.push((0..len).map(|_| rng.next_u8()).collect());
    }
    out
}
