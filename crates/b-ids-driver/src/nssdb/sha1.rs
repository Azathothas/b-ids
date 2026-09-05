//! SHA-1, because NSS refuses a trust record whose certificate hash is absent.
//!
//! ⛔ **This is not a security primitive here and it must not become one.**
//! The one caller writes `CKA_NSS_CERT_SHA1_HASH` into a certificate trust
//! object, and NSS compares that field against its own digest of the
//! certificate before it will believe the record at all:
//! `nssTrust_Create` in
//! `references/mozilla__nss/tree/lib/pki/certificate.c:1022` discards a trust
//! object whose hash does not match, and
//! `nssTrust_IsSafeToIgnoreCertHash` at line 916 lets the hash be absent only
//! for records that trust nothing. So a delegator record with no hash is
//! silently ignored, which is the failure this module exists to prevent.
//!
//! ⚠ **The algorithm is chosen by the reader, not by this project.**
//! `nssCryptokiTrust_GetAttributes` in
//! `references/mozilla__nss/tree/lib/dev/ckhelper.c:441` sets the mechanism to
//! `CKM_SHA_1` for a `CKO_NSS_TRUST` object, so nothing else is read.
//!
//! ⭐ **Nothing here says what its own answer should be.** Four of the vectors
//! in the tests below are FIPS 180-4's published ones and the other four came
//! from an independent implementation, so a defect fails the suite rather than
//! a browser launch.
//!
//! `docs/history/todo/driver.md`, `DRIVER-11`.

/// How many bytes a SHA-1 digest is.
pub const LEN: usize = 20;

/// The digest of `data`.
#[must_use]
pub fn sha1(data: &[u8]) -> [u8; LEN] {
    let mut state: [u32; 5] = [
        0x6745_2301,
        0xefcd_ab89,
        0x98ba_dcfe,
        0x1032_5476,
        0xc3d2_e1f0,
    ];
    let (blocks, rest) = data.as_chunks::<64>();
    for block in blocks {
        compress(&mut state, block);
    }
    // ⛔ THE PADDING IS A BIT COUNT, NOT A BYTE COUNT, and it is written
    // big-endian over eight bytes. Both halves of that have been got wrong in
    // implementations that pass on short inputs and fail past 2^29 bytes.
    let bits = (data.len() as u64).wrapping_mul(8);
    let mut tail = [0_u8; 128];
    tail[..rest.len()].copy_from_slice(rest);
    tail[rest.len()] = 0x80;
    // ⚠ One padded block when the length still fits beside the message, two
    // when it does not. A padding that assumed one is right on every vector
    // under 56 bytes and wrong on the next one.
    let used = if rest.len() < 56 { 64 } else { 128 };
    tail[used - 8..used].copy_from_slice(&bits.to_be_bytes());
    let (padded, _) = tail[..used].as_chunks::<64>();
    for block in padded {
        compress(&mut state, block);
    }

    let mut out = [0_u8; LEN];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Mix one 64-byte block into `state`.
fn compress(state: &mut [u32; 5], block: &[u8; 64]) {
    let mut w = [0_u32; 80];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([
            block[i * 4],
            block[i * 4 + 1],
            block[i * 4 + 2],
            block[i * 4 + 3],
        ]);
    }
    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }
    let [mut a, mut b, mut c, mut d, mut e] = *state;
    for (i, word) in w.iter().enumerate() {
        let (f, k) = match i {
            0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999),
            20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
            40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
            _ => (b ^ c ^ d, 0xca62_c1d6),
        };
        let t = a
            .rotate_left(5)
            .wrapping_add(f)
            .wrapping_add(e)
            .wrapping_add(k)
            .wrapping_add(*word);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = t;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔ **The expected digests are BYTES rather than hexadecimal, and that is
    /// not a style choice.** A forty-character lower-case hex run in a tracked
    /// file is what `scripts/common/check-no-secrets.sh --public` refuses,
    /// because it is the shape of a commit identifier or a token, and this
    /// project's answer to a false positive is to remove the shape rather than
    /// to widen the rule. The values are the published ones and the bytes are
    /// the same value.
    ///
    /// ⚠ **The first four are FIPS 180-4's own published vectors.** The other
    /// four are not published anywhere: they were produced on 2026-09-04 by an
    /// independent implementation, CPython 3.13.15's `hashlib`, precisely
    /// because this implementation must not be the thing that says what its own
    /// answer should be.
    fn expect(data: &[u8], digest: [u8; LEN]) {
        assert_eq!(sha1(data), digest, "over {} byte(s)", data.len());
    }

    #[test]
    fn the_fips_180_vectors_match() {
        expect(
            b"",
            [
                0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60,
                0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09,
            ],
        );
        expect(
            b"abc",
            [
                0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
                0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d,
            ],
        );
        expect(
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
            [
                0x84, 0x98, 0x3e, 0x44, 0x1c, 0x3b, 0xd2, 0x6e, 0xba, 0xae, 0x4a, 0xa1, 0xf9, 0x51,
                0x29, 0xe5, 0xe5, 0x46, 0x70, 0xf1,
            ],
        );
        expect(
            &[b'a'; 1_000_000],
            [
                0x34, 0xaa, 0x97, 0x3c, 0xd4, 0xc4, 0xda, 0xa4, 0xf6, 0x1e, 0xeb, 0x2b, 0xdb, 0xad,
                0x27, 0x31, 0x65, 0x34, 0x01, 0x6f,
            ],
        );
    }

    #[test]
    fn the_two_padding_lengths_are_both_exercised() {
        // ⛔ 55 bytes leaves room for the length in the same block and 56 does
        // not, so a padding that assumed one block is wrong at exactly this
        // step and right on every vector above it. 63 and 64 are the other
        // edge, where the message fills a block exactly.
        expect(
            &[b'x'; 55],
            [
                0xce, 0xf7, 0x34, 0xba, 0x81, 0xa0, 0x24, 0x47, 0x9e, 0x09, 0xeb, 0x5a, 0x75, 0xb6,
                0xdd, 0xae, 0x62, 0xe6, 0xab, 0xf1,
            ],
        );
        expect(
            &[b'x'; 56],
            [
                0x90, 0x13, 0x05, 0x36, 0x7c, 0x25, 0x99, 0x52, 0xf4, 0xe7, 0xaf, 0x83, 0x23, 0xf4,
                0x80, 0xd5, 0x9f, 0x81, 0x33, 0x5b,
            ],
        );
        expect(
            &[b'x'; 63],
            [
                0x0d, 0xdc, 0x4e, 0x0c, 0xcc, 0xd9, 0xa1, 0x28, 0x50, 0xde, 0xb5, 0xab, 0xb0, 0x85,
                0x3a, 0x44, 0x25, 0x55, 0x9f, 0xec,
            ],
        );
        expect(
            &[b'x'; 64],
            [
                0xbb, 0x2f, 0xa3, 0xee, 0x7a, 0xfb, 0x9f, 0x54, 0xc6, 0xdf, 0xb5, 0xd0, 0x21, 0xf1,
                0x4b, 0x1f, 0xfe, 0x40, 0xc1, 0x63,
            ],
        );
    }
}
