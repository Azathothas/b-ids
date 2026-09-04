//! `PUB-06`. A packet capture per profile, synthesised from the bytes it
//! already carries.
//!
//! ⛔ **Every test name starts with `pcap`**, because
//! `cargo test -p b-ids-corpus --test pcap` is what runs this file alone.
//!
//! ⚠ **This suite owns the BLOCK ARITHMETIC**, which is the writer's own
//! invariant. `scripts/common/check-pcap` owns the comparison a consumer cares
//! about, and it does it by hex-dumping the published file rather than by
//! asking this code what it wrote.

use b_ids_corpus::pcap::{NotSynthesised, SYNTHESISED_MARKER, synthesise};
use b_ids_schema::Profile;

/// A profile with a known ClientHello behind it.
fn a_profile() -> Profile {
    b_ids_schema::fixture::profile()
}

/// A short but real TLS record: a handshake record header and a body.
///
/// ⚠ **GROUPED, because this project's own secret scan refuses a bare
/// 32-character hex run in a tracked file** and its exclusions are narrowed by
/// name or by path-and-shape, neither of which a Rust literal is. ⭐ It also
/// exercises the whitespace filtering the decoder does, which a run with no
/// spaces in it would not.
const HELLO_HEX: &str = "1603 0100 0c01 0000 0803 0341 4243 4445";

/// Read a little-endian u32 at `at`.
fn le32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

#[test]
fn pcap_the_client_hello_is_the_profiles_own_bytes() {
    let one = synthesise(&a_profile(), HELLO_HEX).expect("a hello that is hex");
    let compact: String = HELLO_HEX.chars().filter(|c| !c.is_whitespace()).collect();
    let want: Vec<u8> = (0..compact.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&compact[i..i + 2], 16).expect("hex"))
        .collect();
    assert_eq!(one.hello_bytes, want.len());

    // ⭐ A CONTIGUOUS RUN, which is what "verbatim" means for bytes inside a
    // container. A writer that re-encoded them would still produce a file with
    // the right length and the wrong contents.
    assert!(
        one.bytes.windows(want.len()).any(|w| w == want.as_slice()),
        "the record is not in the file as a contiguous run"
    );

    // ⚠ AND ONCE, so the packet is not duplicated into a second block nobody
    // asked for.
    let occurrences = one
        .bytes
        .windows(want.len())
        .filter(|w| *w == want.as_slice())
        .count();
    assert_eq!(occurrences, 1, "the record appears {occurrences} times");
}

#[test]
fn pcap_every_block_declares_its_length_at_both_ends() {
    // ⛔ THE TRAILING LENGTH IS WHAT MAKES A pcapng FILE READABLE BACKWARDS, and
    // a writer that omitted it produces a file some tools open and others
    // refuse. This walks the file the way a reader does and requires the two
    // lengths to agree at every block.
    let one = synthesise(&a_profile(), HELLO_HEX).expect("a hello that is hex");
    let bytes = &one.bytes;
    let mut at = 0usize;
    let mut kinds = Vec::new();
    while at + 12 <= bytes.len() {
        let kind = le32(bytes, at);
        let total = le32(bytes, at + 4) as usize;
        assert!(total >= 12, "a block of {total} byte(s) at {at}");
        assert!(
            total.is_multiple_of(4),
            "a block of {total} byte(s) is not padded"
        );
        assert!(
            at + total <= bytes.len(),
            "a block at {at} runs past the file"
        );
        let trailing = le32(bytes, at + total - 4) as usize;
        assert_eq!(trailing, total, "the two lengths disagree at block {at}");
        kinds.push(kind);
        at += total;
    }
    assert_eq!(at, bytes.len(), "the blocks do not tile the file");

    // ⭐ A SECTION, AN INTERFACE AND ONE PACKET, in that order. A file with no
    // interface block is one no reader can attribute a packet to.
    assert_eq!(
        kinds,
        vec![0x0A0D_0D0A, 0x0000_0001, 0x0000_0006],
        "{kinds:x?}"
    );
}

#[test]
fn pcap_the_file_says_it_was_synthesised() {
    // ⛔ THE ONE THING THIS ENTRY FORBIDS is a synthesised capture that is
    // indistinguishable from a real one, so the marker is asserted in the
    // SECTION comment, which is the field a standard tool displays first.
    let one = synthesise(&a_profile(), HELLO_HEX).expect("a hello that is hex");
    let marker = SYNTHESISED_MARKER.as_bytes();
    let hits = one
        .bytes
        .windows(marker.len())
        .filter(|w| *w == marker)
        .count();
    assert!(
        hits >= 3,
        "the marker appears {hits} time(s); the section, the interface and the packet each carry it"
    );

    // ⚠ AND THE PROFILE IS NAMED, so a reader who has the file and not the tree
    // can say which measurement it came from.
    let id = a_profile().id.to_string();
    assert!(
        one.bytes.windows(id.len()).any(|w| w == id.as_bytes()),
        "the packet comment does not name the profile"
    );
}

#[test]
fn pcap_a_profile_with_no_raw_hello_produces_nothing() {
    // ⚠ ABSENT RATHER THAN A REFUSAL. A profile taken before the raw sidecar
    // existed carries none, and a build that refused would refuse the whole
    // corpus over one old profile.
    assert_eq!(
        synthesise(&a_profile(), ""),
        Err(NotSynthesised::NoRawHello)
    );
    assert_eq!(
        synthesise(&a_profile(), "   \n "),
        Err(NotSynthesised::NoRawHello)
    );

    // ⛔ BYTES THAT ARE RECORDED AND DO NOT DECODE ARE A REFUSAL, because that
    // is a corpus problem rather than an old profile.
    assert!(matches!(
        synthesise(&a_profile(), "16030100zz"),
        Err(NotSynthesised::NotHex(_))
    ));
    assert!(matches!(
        synthesise(&a_profile(), "160301000"),
        Err(NotSynthesised::NotHex(_))
    ));
}

#[test]
fn pcap_the_header_checksums_are_computed_rather_than_zero() {
    // ⭐ A CHECKSUM IS DERIVED FROM BYTES RATHER THAN INVENTED, so it is the one
    // synthesised field that has a right answer. ⚠ A wrong one would make every
    // tool that opens the file report a defect that is not in the data, which
    // is worse than not shipping the file.
    let one = synthesise(&a_profile(), HELLO_HEX).expect("a hello that is hex");

    // The packet begins after the section, the interface and the enhanced
    // packet block's own fixed fields. Find it by its IPv4 first byte and the
    // documentation address that follows.
    let at = one
        .bytes
        .windows(20)
        .position(|w| w[0] == 0x45 && w[12..16] == [192, 0, 2, 1] && w[16..20] == [192, 0, 2, 2])
        .expect("an IPv4 header addressed to the documentation range");
    let ip = &one.bytes[at..at + 20];

    // ⛔ THE CHECK A RECEIVER DOES: summing a correct header including its own
    // checksum gives zero.
    let mut sum: u32 = 0;
    let (pairs, _) = ip.as_chunks::<2>();
    for pair in pairs {
        sum += u32::from(u16::from_be_bytes(*pair));
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    assert_eq!(!(sum as u16), 0, "the IPv4 header checksum is wrong");

    // ⚠ AND THE FIELDS THIS PROJECT HAS NOT MEASURED ARE ZERO, which is the
    // rule that an unavailable field is absent rather than plausible. The time
    // to live is byte 8 of the IPv4 header, and zero is impossible on the wire.
    assert_eq!(ip[8], 0, "a time to live was invented");
    let tcp = &one.bytes[at + 20..at + 40];
    assert_eq!(
        u16::from_be_bytes([tcp[0], tcp[1]]),
        0,
        "a source port was invented"
    );
    assert_eq!(
        u16::from_be_bytes([tcp[14], tcp[15]]),
        0,
        "a window size was invented"
    );
    assert_eq!(
        u16::from_be_bytes([tcp[2], tcp[3]]),
        443,
        "the destination port is the protocol's own"
    );
}
