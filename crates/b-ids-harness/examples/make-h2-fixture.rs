//! Build the HTTP/2 connection fixtures the frame-reader test feeds, and print
//! one of them as a hex line.
//!
//! ```text
//! cargo run -p b-ids-harness --example make-h2-fixture -- full > crates/b-ids-harness/fixtures/h2-connection.hex
//! cargo run -p b-ids-harness --example make-h2-fixture -- missing-setting > crates/b-ids-harness/fixtures/h2-connection-missing-setting.hex
//! ```
//!
//! ⛔ **THE BYTES ARE CONSTRUCTED, NOT CAPTURED.** No value here is a
//! measurement and none of it may enter the corpus. A measured value lives in a
//! profile; a value somebody else measured lives in `docs/inherited-claims.md`
//! with its source.
//!
//! ⭐ **The settings arrive in an order nothing would sort to, and the values
//! are not any client's.** Both are deliberate. An arrival order that happens
//! to be ascending cannot tell a reader that preserves order from one that
//! sorts, and a fixture carrying a browser's real numbers is a fixture somebody
//! will one day quote as a measurement.
//!
//! ⚠ The two variants differ by exactly one settings entry, which is what makes
//! "records it as absent" separable from "records it as its default".

const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// The settings this connection sends, in the order it sends them.
///
/// ⚠ Identifiers 1, 2, 4 and 6 with 3 and 5 missing is the SHAPE a browser has.
/// The values beside them are not.
const SETTINGS: [(u16, u32); 4] = [
    (6, 0x0001_0001),
    (1, 0x0000_2000),
    (4, 0x0030_0000),
    (2, 0x0000_0000),
];

/// The identifier the `missing-setting` variant leaves out.
const OMITTED: u16 = 4;

fn frame(out: &mut Vec<u8>, frame_type: u8, flags: u8, stream_id: u32, payload: &[u8]) {
    let length = u32::try_from(payload.len()).expect("a fixture frame is small");
    out.extend_from_slice(&length.to_be_bytes()[1..]);
    out.push(frame_type);
    out.push(flags);
    out.extend_from_slice(&stream_id.to_be_bytes());
    out.extend_from_slice(payload);
}

/// An exclusive bit, a 31-bit dependency, and one weight byte.
///
/// ⛔ The weight is AS ENCODED, which the protocol defines as the weight minus
/// one.
fn priority_block(exclusive: bool, dependency: u32, weight_wire: u8) -> [u8; 5] {
    let head = (u32::from(exclusive) << 31) | dependency;
    let [a, b, c, d] = head.to_be_bytes();
    [a, b, c, d, weight_wire]
}

fn connection(omit: Option<u16>) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(PREFACE);

    let mut settings = Vec::new();
    for (id, value) in SETTINGS {
        if omit == Some(id) {
            continue;
        }
        settings.extend_from_slice(&id.to_be_bytes());
        settings.extend_from_slice(&value.to_be_bytes());
    }
    frame(&mut out, 0x4, 0x0, 0, &settings);

    // The connection-level WINDOW_UPDATE. ⛔ The INCREMENT, which is what the
    // wire carries. ⚠ The number is deliberately one no client sends, so that
    // nobody can read this fixture as a measurement of one.
    frame(&mut out, 0x8, 0x0, 0, &0x00ab_cdef_u32.to_be_bytes());

    // ⚠ A frame type this project has no name for, sent BEFORE the header
    // block so that a reader which stops at the first HEADERS frame still meets
    // it. A sequence that silently omits a frame is a sequence nobody can
    // compare.
    frame(&mut out, 0x63, 0x0, 0, &[0xde, 0xad]);

    // A standalone PRIORITY frame, which is a different seam from the block
    // inside a HEADERS frame.
    frame(&mut out, 0x2, 0x0, 3, &priority_block(false, 0, 200));

    // HEADERS on stream 1, with END_STREAM, END_HEADERS and PRIORITY set.
    let mut headers = Vec::new();
    headers.extend_from_slice(&priority_block(true, 0, 255));
    headers.extend_from_slice(&header_block());
    frame(&mut out, 0x1, 0x01 | 0x04 | 0x20, 1, &headers);

    out
}

/// A literal string, uncoded, with its length in front of it.
///
/// ⚠ Uncoded on purpose. A fixture whose strings are Huffman-coded proves the
/// decoder against bytes this file produced, and the Huffman table is proved
/// against a fetched vector corpus instead.
fn literal(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    assert!(bytes.len() < 127, "a fixture string is short");
    out.push(u8::try_from(bytes.len()).expect("checked above"));
    out.extend_from_slice(bytes);
}

/// A prefixed integer, for the four-bit and six-bit forms this fixture uses.
fn prefixed(out: &mut Vec<u8>, pattern: u8, mask: u8, value: u8) {
    assert!(value < mask + 128, "a fixture index is small");
    if value < mask {
        out.push(pattern | value);
    } else {
        out.push(pattern | mask);
        out.push(value - mask);
    }
}

/// The HPACK block the first request carries.
///
/// ⭐ **The pseudo-headers are in the order a browser sends them**, which is the
/// shape the Akamai string renders as `m,a,s,p`. ⛔ The values are not any
/// client's.
///
/// ⭐ **It carries a `cookie` field that is ADDED TO THE DYNAMIC TABLE**, and
/// two indexed fields after it. The decoder has to see the credential, because
/// the table would otherwise be wrong for every later index, and the capture
/// has to drop it. Those two requirements are only separable with a fixture
/// where the second field can only resolve if the first one was stored: index
/// 63 below names the marker header if and only if the credential took slot
/// 62.
fn header_block() -> Vec<u8> {
    let mut out = Vec::new();
    out.push(0x82); // indexed: :method GET
    out.push(0x41); // literal, incremental, name index 1: :authority
    literal(&mut out, "fixture.invalid");
    out.push(0x86); // indexed: :scheme http
    out.push(0x84); // indexed: :path /

    out.push(0x40); // literal, incremental, a name not in either table
    literal(&mut out, "x-fixture-marker");
    literal(&mut out, "kept");

    prefixed(&mut out, 0x00, 0x0f, 58); // without indexing, name index 58: user-agent
    literal(&mut out, "a-fixture-value");

    prefixed(&mut out, 0x40, 0x3f, 32); // INCREMENTAL, name index 32: cookie
    literal(&mut out, "not-a-real-value");

    prefixed(&mut out, 0x80, 0x7f, 62); // indexed: the credential, from the table
    prefixed(&mut out, 0x80, 0x7f, 63); // indexed: the marker, one slot further down
    out
}

fn main() {
    let variant = std::env::args().nth(1).unwrap_or_default();
    let bytes = match variant.as_str() {
        "full" => connection(None),
        "missing-setting" => connection(Some(OMITTED)),
        other => {
            eprintln!("make-h2-fixture: variant {other:?} is not full or missing-setting");
            std::process::exit(2);
        }
    };
    println!("{}", b_ids_harness::hex(&bytes));
}
