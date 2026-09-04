//! A packet capture per profile, synthesised from the bytes it already carries.
//!
//! ⛔ **Every network engineer already has a tool for one format and this
//! project did not produce it.** `TODO/publish.md`, `PUB-06`.
//!
//! ⭐ **SYNTHESISED FROM THE PROFILE, never captured a second time.** The
//! `ClientHello` is already stored byte for byte under `raw/v1/`, so this is a
//! generated format like every other rather than a second capture path that
//! could disagree with the first.
//!
//! # ⛔ What is real here, and what is not
//!
//! The entry's own rule is that a synthesised capture must not be
//! indistinguishable from a real one, so the answer is a file that says what it
//! is, in a field a standard tool displays.
//!
//! | in the file | where it comes from |
//! | --- | --- |
//! | the TLS record | ⭐ **the profile's own `raw/v1/` bytes, verbatim** |
//! | everything else | ⛔ **synthesised, and named as such in three comments** |
//!
//! ⚠ **The values chosen for the synthesised fields are chosen to be visibly
//! impossible or visibly reserved**, rather than plausible:
//!
//! - the addresses are `192.0.2.1` and `192.0.2.2`, which RFC 5737 reserves for
//!   documentation and which no host on the internet has;
//! - the timestamp is zero, which displays as 1970;
//! - ⛔ the time to live, the window size and the source port are **zero**,
//!   and each is a field `HARNESS-11` measured that this project cannot read.
//!   Writing a plausible value into any of them would be publishing a
//!   measurement nobody took, which is `TODO/RULES.md` rule 1;
//! - the destination port is 443, which is the protocol's own well-known port
//!   and the one thing here chosen so the file dissects at all.
//!
//! ⭐ **The header checksums ARE computed**, because a checksum is derived from
//! bytes rather than invented, and a wrong one would make every tool that opens
//! the file report a defect that is not in the data.
//!
//! ⚠ **pcapng rather than pcap**, and that is the whole reason the format was
//! chosen: pcapng carries comments, classic pcap does not. A file that could not
//! say it was synthesised would be the thing this entry forbids.

use b_ids_schema::Profile;

/// The marker every synthesised capture carries, in its section comment.
///
/// ⛔ **Checked for by `scripts/common/check-pcap`**, so a build that stopped
/// writing it fails rather than publishing a file that looks captured.
pub const SYNTHESISED_MARKER: &str = "SYNTHESISED BY b-ids";

/// The section comment, in full.
const SECTION_COMMENT: &str = "\
SYNTHESISED BY b-ids. This is NOT a captured file. The TLS record in the one \
packet below is the ClientHello this project measured, byte for byte, from the \
profile named in the packet comment. Every other byte in this file was \
generated: the addresses are RFC 5737 documentation addresses, the timestamp is \
zero, and the source port, the window size and the time to live are zero \
because this project has not measured them. Read the profile for what was \
measured; read this file only to feed the bytes to a tool that wants a capture.";

/// What one profile synthesises to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Synthesised {
    /// The profile this describes.
    pub profile_id: String,
    /// The published path, relative to the tree root.
    pub path: String,
    /// The file.
    pub bytes: Vec<u8>,
    /// How many bytes of it are the profile's own `ClientHello`.
    pub hello_bytes: usize,
}

/// Why a profile produced no capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotSynthesised {
    /// The profile records no raw `ClientHello`.
    ///
    /// ⚠ **Absent rather than a refusal.** A profile taken before the raw
    /// sidecar existed carries none, and a build that refused would refuse the
    /// whole corpus over one old profile.
    NoRawHello,
    /// The recorded hex is not hex, or is an odd length.
    NotHex(String),
}

impl std::fmt::Display for NotSynthesised {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRawHello => f.write_str("the profile records no raw ClientHello"),
            Self::NotHex(why) => write!(f, "the recorded ClientHello is not hex: {why}"),
        }
    }
}

/// Decode a lower-case hex run.
fn unhex(text: &str) -> Result<Vec<u8>, String> {
    let trimmed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    if !trimmed.len().is_multiple_of(2) {
        return Err(format!("{} character(s) is an odd length", trimmed.len()));
    }
    (0..trimmed.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&trimmed[i..i + 2], 16)
                .map_err(|_| format!("{:?} is not a hex pair", &trimmed[i..i + 2]))
        })
        .collect()
}

/// The one's-complement sum a header checksum is.
fn ones_complement(bytes: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let (pairs, rest) = bytes.as_chunks::<2>();
    for pair in pairs {
        sum += u32::from(u16::from_be_bytes(*pair));
    }
    // ⚠ AN ODD-LENGTH BUFFER IS PADDED WITH A ZERO BYTE ON THE RIGHT, which is
    // the algorithm's own rule rather than a convenience: a TCP payload of odd
    // length is ordinary and dropping its last byte would compute a checksum
    // over bytes nobody sent.
    if let [last] = rest {
        sum += u32::from(u16::from_be_bytes([*last, 0]));
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Append a pcapng option, padded to four bytes.
fn option(out: &mut Vec<u8>, code: u16, value: &[u8]) {
    out.extend_from_slice(&code.to_le_bytes());
    out.extend_from_slice(&(u16::try_from(value.len()).unwrap_or(u16::MAX)).to_le_bytes());
    out.extend_from_slice(value);
    let pad = (4 - value.len() % 4) % 4;
    out.extend(std::iter::repeat_n(0u8, pad));
}

/// Wrap a body in a pcapng block, with the length written at both ends.
///
/// ⛔ **The trailing length is what makes a pcapng file readable BACKWARDS**,
/// and a writer that omitted it produces a file some tools open and others
/// refuse. It is the same value, written twice, by construction.
fn block(kind: u32, body: &[u8]) -> Vec<u8> {
    let total = u32::try_from(body.len() + 12).unwrap_or(u32::MAX);
    let mut out = Vec::with_capacity(total as usize);
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&total.to_le_bytes());
    out.extend_from_slice(body);
    out.extend_from_slice(&total.to_le_bytes());
    out
}

/// The IPv4 and TCP headers the record is carried in.
///
/// ⛔ Every field here is synthesised and the module header says which and why.
fn synthetic_headers(payload: &[u8]) -> Vec<u8> {
    let total_len = u16::try_from(20 + 20 + payload.len()).unwrap_or(u16::MAX);
    let mut ip = Vec::with_capacity(20);
    ip.push(0x45); // IPv4, five 32-bit words of header
    ip.push(0x00); // no differentiated services
    ip.extend_from_slice(&total_len.to_be_bytes());
    ip.extend_from_slice(&[0x00, 0x00]); // identification
    ip.extend_from_slice(&[0x40, 0x00]); // don't fragment
    ip.push(0x00); // ⛔ time to live: unmeasured, and zero is impossible on the wire
    ip.push(0x06); // TCP
    ip.extend_from_slice(&[0x00, 0x00]); // checksum, filled below
    ip.extend_from_slice(&[192, 0, 2, 1]); // RFC 5737, reserved for documentation
    ip.extend_from_slice(&[192, 0, 2, 2]);
    let checksum = ones_complement(&ip);
    ip[10..12].copy_from_slice(&checksum.to_be_bytes());

    let mut tcp = Vec::with_capacity(20);
    tcp.extend_from_slice(&0u16.to_be_bytes()); // ⛔ source port: unmeasured
    tcp.extend_from_slice(&443u16.to_be_bytes());
    tcp.extend_from_slice(&0u32.to_be_bytes()); // sequence
    tcp.extend_from_slice(&0u32.to_be_bytes()); // acknowledgement
    tcp.push(0x50); // five 32-bit words of header, no options
    tcp.push(0x18); // PSH, ACK
    tcp.extend_from_slice(&0u16.to_be_bytes()); // ⛔ window: unmeasured
    tcp.extend_from_slice(&[0x00, 0x00]); // checksum, filled below
    tcp.extend_from_slice(&0u16.to_be_bytes()); // urgent pointer

    // ⭐ The TCP checksum covers a pseudo-header of the addresses, the protocol
    // and the segment length, which is why it is computed after the IP header.
    let mut pseudo = Vec::with_capacity(12 + tcp.len() + payload.len());
    pseudo.extend_from_slice(&ip[12..20]);
    pseudo.push(0);
    pseudo.push(0x06);
    pseudo.extend_from_slice(
        &u16::try_from(tcp.len() + payload.len())
            .unwrap_or(u16::MAX)
            .to_be_bytes(),
    );
    pseudo.extend_from_slice(&tcp);
    pseudo.extend_from_slice(payload);
    let tcp_checksum = ones_complement(&pseudo);
    tcp[16..18].copy_from_slice(&tcp_checksum.to_be_bytes());

    let mut out = ip;
    out.extend_from_slice(&tcp);
    out.extend_from_slice(payload);
    out
}

/// Synthesise one profile's capture.
///
/// `hello_hex` is the profile's raw sidecar, exactly as `raw/v1/` stores it.
///
/// # Errors
///
/// [`NotSynthesised`], for a profile with no raw hello or one whose recorded
/// bytes are not hex.
pub fn synthesise(profile: &Profile, hello_hex: &str) -> Result<Synthesised, NotSynthesised> {
    if hello_hex.trim().is_empty() {
        return Err(NotSynthesised::NoRawHello);
    }
    let hello = unhex(hello_hex).map_err(NotSynthesised::NotHex)?;

    let mut out = Vec::new();

    // -- the section header --------------------------------------------------
    let mut shb = Vec::new();
    shb.extend_from_slice(&0x1A2B_3C4Du32.to_le_bytes()); // byte-order magic
    shb.extend_from_slice(&1u16.to_le_bytes()); // major
    shb.extend_from_slice(&0u16.to_le_bytes()); // minor
    shb.extend_from_slice(&u64::MAX.to_le_bytes()); // section length: not known
    option(&mut shb, 1, SECTION_COMMENT.as_bytes()); // opt_comment
    option(&mut shb, 4, b"b-ids"); // shb_userappl
    option(&mut shb, 0, &[]); // opt_endofopt
    out.extend_from_slice(&block(0x0A0D_0D0A, &shb));

    // -- the interface -------------------------------------------------------
    //
    // ⚠ LINKTYPE_RAW rather than Ethernet, deliberately: an Ethernet frame
    // needs two hardware addresses this project has not measured and has no
    // reserved range to borrow. Raw IP needs neither.
    let mut idb = Vec::new();
    idb.extend_from_slice(&101u16.to_le_bytes()); // LINKTYPE_RAW
    idb.extend_from_slice(&0u16.to_le_bytes()); // reserved
    idb.extend_from_slice(&0u32.to_le_bytes()); // snaplen: no limit
    option(
        &mut idb,
        1,
        b"SYNTHESISED BY b-ids. There was no interface: raw IP, so that no \
          hardware address had to be invented.",
    );
    option(&mut idb, 0, &[]);
    out.extend_from_slice(&block(0x0000_0001, &idb));

    // -- the one packet ------------------------------------------------------
    let packet = synthetic_headers(&hello);
    let mut epb = Vec::new();
    epb.extend_from_slice(&0u32.to_le_bytes()); // interface 0
    epb.extend_from_slice(&0u32.to_le_bytes()); // timestamp, high
    epb.extend_from_slice(&0u32.to_le_bytes()); // timestamp, low
    let len = u32::try_from(packet.len()).unwrap_or(u32::MAX);
    epb.extend_from_slice(&len.to_le_bytes()); // captured length
    epb.extend_from_slice(&len.to_le_bytes()); // original length
    epb.extend_from_slice(&packet);
    let pad = (4 - packet.len() % 4) % 4;
    epb.extend(std::iter::repeat_n(0u8, pad));
    option(
        &mut epb,
        1,
        format!(
            "SYNTHESISED BY b-ids from profile {}. The {} byte(s) of TLS record are that \
             profile's measured ClientHello, verbatim. The IPv4 and TCP headers around them were \
             generated: the addresses are RFC 5737 documentation addresses, and the source port, \
             the window size and the time to live are zero because this project has not measured \
             them.",
            profile.id,
            hello.len()
        )
        .as_bytes(),
    );
    option(&mut epb, 0, &[]);
    out.extend_from_slice(&block(0x0000_0006, &epb));

    Ok(Synthesised {
        profile_id: profile.id.to_string(),
        path: format!(
            "pcap/v1/{}/{}/{}/{}.pcapng",
            profile.browser.name.to_ascii_lowercase(),
            profile.browser.channel.as_str(),
            profile.platform_token().as_str(),
            profile.browser.version
        ),
        bytes: out,
        hello_bytes: hello.len(),
    })
}
