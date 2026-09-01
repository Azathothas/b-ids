//! The certificate pin an authority hands a client, and the encoding under it.
//!
//! ⭐ **A pin is what lets a client trust exactly one run without touching a
//! trust store.** Installing a root certificate changes a machine's security
//! configuration; a pin names one key, for one launch, and is a condition of
//! any capture taken through it.
//!
//! ⚠ **The encoder is checked against the specification's own vectors**, not
//! against itself. An encoder tested by decoding its own output agrees with
//! every defect it has.
//!
//! `TODO/driver.md`, `DRIVER-01`.

use std::net::IpAddr;

use b_ids_harness::bytes::base64;

#[test]
fn pin_encoder_matches_the_specification_vectors() {
    // RFC 4648 section 10. ⛔ Every one of the four padding cases is here,
    // because a three-byte encoder is right on aligned input and wrong on the
    // tail, and the tail is where the digest ends.
    for (input, expected) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(input.as_bytes()), expected, "input {input:?}");
    }
}

#[test]
fn pin_encoder_emits_the_alphabet_in_order() {
    // ⚠ Every one of the 64 characters, so a transcription error in the table
    // fails here rather than in a client that cannot match a pin.
    //
    // ⭐ The input is the 64 six-bit values packed end to end, which is exactly
    // 48 bytes, so the expected output is the alphabet itself with no padding.
    // ⚠ A first attempt used the bytes 0 to 47 and covered 39 of the 64: an
    // encoder is fed BYTES and the table is indexed by six-bit groups, and
    // those are not the same range.
    let mut packed = Vec::new();
    let mut accumulator = 0_u32;
    let mut bits = 0_u32;
    for value in 0..64_u32 {
        accumulator = (accumulator << 6) | value;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            packed.push(u8::try_from((accumulator >> bits) & 0xff).expect("a byte"));
        }
    }
    assert_eq!(packed.len(), 48);
    assert_eq!(
        base64(&packed),
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
    );
}

#[test]
fn pin_is_a_sha256_and_it_is_stable_for_one_authority() {
    let authority = b_ids_harness::mint(IpAddr::from([127, 0, 0, 1])).expect("an authority mints");
    let pin = authority.spki_pin();
    // 32 bytes of digest are 44 base64 characters, the last of which is padding.
    assert_eq!(pin.len(), 44, "{pin}");
    assert!(pin.ends_with('='), "{pin}");
    assert_eq!(
        pin,
        authority.spki_pin(),
        "the pin does not change under one authority"
    );
}

#[test]
fn pin_differs_between_two_runs() {
    // ⛔ An authority is minted per run and never reused, so two of them share
    // no key. A pin that repeated would mean a key that repeated, which is a
    // long-lived private key this project refuses to have.
    let one = b_ids_harness::mint(IpAddr::from([127, 0, 0, 1])).expect("an authority mints");
    let two = b_ids_harness::mint(IpAddr::from([127, 0, 0, 1])).expect("an authority mints");
    assert_ne!(one.spki_pin(), two.spki_pin());
}
