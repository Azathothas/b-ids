//! HARNESS-03. Read HTTP/2 settings, the window update and the priority block.
//!
//! The acceptance: a fixture of frame bytes produces a profile whose settings
//! list is in arrival order, and a fixture that omits one settings key produces
//! a profile that records it as absent rather than as its default.
//!
//! ⛔ Every test name starts with `http2`, because
//! `cargo test -p b-ids-harness http2` is the entry's acceptance command.

mod support;

use b_ids_harness::h2::{self, FLAG_END_HEADERS, FLAG_PADDED, FLAG_PRIORITY, PREFACE};
use b_ids_harness::{Config, Note, Protocol};
use b_ids_schema::http::ValuePolicy;
use b_ids_schema::http2::Frame;
use std::time::Duration;

use support::{feed, fixture_bytes, one_connection};

/// The identifier for `SETTINGS_MAX_FRAME_SIZE`, whose protocol default is
/// 16384 and which the fixtures never send.
const MAX_FRAME_SIZE: u16 = 5;

/// The identifier the `missing-setting` fixture leaves out and the `full` one
/// carries.
const OMITTED_BY_ONE_FIXTURE: u16 = 4;

fn read(name: &str) -> (h2::Http2Capture, Vec<Note>) {
    let bytes = fixture_bytes(name);
    let mut notes = Vec::new();
    let capture =
        h2::parse_connection(&bytes, ValuePolicy::NamesOnly, &mut notes).unwrap_or_else(|why| {
            panic!("the committed fixture {name} is an HTTP/2 connection: {why}")
        });
    (capture, notes)
}

#[test]
fn http2_reads_the_settings_in_the_order_they_arrived() {
    // ⛔ Order is part of the fingerprint, which is why the half is a frame
    // sequence and not a settings map. The fixture sends 6, 1, 4, 2 precisely
    // because nothing sorts to that: a reader that lost the order would come
    // out ascending and pass a test written against an ascending fixture.
    let (capture, _notes) = read("h2-connection.hex");
    let settings = capture.half.settings().expect("a SETTINGS frame arrived");
    let ids: Vec<u16> = settings.iter().map(|e| e.id).collect();
    assert_eq!(ids, vec![6, 1, 4, 2], "the arrival order is not preserved");

    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_ne!(ids, sorted, "the fixture must not be in sorted order");

    let values: Vec<u32> = settings.iter().map(|e| e.value).collect();
    assert_eq!(values, vec![0x0001_0001, 0x0000_2000, 0x0030_0000, 0]);
}

#[test]
fn http2_records_an_absent_setting_as_absent_rather_than_as_its_default() {
    // ⛔ The other half of the acceptance, and the reason a map was refused. A
    // map cannot say which settings were absent, and a client that omits one
    // and a client that sends it at the protocol default are two visibly
    // different connections.
    let (full, _) = read("h2-connection.hex");
    let (missing, _) = read("h2-connection-missing-setting.hex");

    assert!(full.half.sends_setting(OMITTED_BY_ONE_FIXTURE));
    assert!(!missing.half.sends_setting(OMITTED_BY_ONE_FIXTURE));

    let entries = missing.half.settings().expect("a SETTINGS frame arrived");
    assert_eq!(entries.len(), 3);
    assert!(
        entries.iter().all(|e| e.id != OMITTED_BY_ONE_FIXTURE),
        "the omitted identifier came back: {entries:?}"
    );

    // ⛔ And a setting neither fixture sends is absent in both, rather than
    // present at the value the protocol would have assumed.
    for capture in [&full, &missing] {
        assert!(!capture.half.sends_setting(MAX_FRAME_SIZE));
        assert!(
            capture
                .half
                .settings()
                .expect("a SETTINGS frame")
                .iter()
                .all(|e| e.id != MAX_FRAME_SIZE),
            "an unsent setting was filled in"
        );
    }
}

#[test]
fn http2_reads_the_window_update_increment_and_never_the_window() {
    // ⛔ The increment is what the wire carried. The window is the same
    // quantity in the human unit, it differs by the protocol's own 65,535, and
    // one shipped database holds both meanings in one field with seven entries
    // 65,535 short.
    let (capture, _) = read("h2-connection.hex");
    assert_eq!(capture.half.window_size_increment(), Some(0x00ab_cdef));
    assert_eq!(
        capture.half.connection_window, None,
        "a capture records the increment and derives the window, never the reverse"
    );
    assert!(
        capture.half.check_units().is_empty(),
        "{:?}",
        capture.half.check_units()
    );
}

#[test]
fn http2_reads_the_priority_block_as_bytes_and_reports_the_raw_five() {
    // ⛔ Never only as a rendered string. That string cannot distinguish a
    // block that was not sent from one that was not read, and two of the three
    // published readings of this field were reading a tool that could not write
    // the block rather than a browser.
    let (capture, _) = read("h2-connection.hex");
    let priority = capture
        .half
        .stream_priority
        .expect("the fixture sets the priority flag");
    assert!(priority.exclusive);
    assert_eq!(priority.stream_dependency, 0);
    assert_eq!(priority.weight_wire, 255);
    // ⚠ One quantity in two units: the wire carries the weight minus one.
    assert_eq!(priority.weight_spec(), 256);

    assert_eq!(
        capture.priority_block_hex().as_deref(),
        Some("80000000ff"),
        "the five raw bytes are reported beside the parse"
    );

    let headers = capture
        .half
        .frames
        .iter()
        .find_map(|f| match f {
            Frame::Headers {
                stream_id,
                has_priority_block,
            } => Some((*stream_id, *has_priority_block)),
            _ => None,
        })
        .expect("a HEADERS frame arrived");
    assert_eq!(headers, (1, true));
}

#[test]
fn http2_skips_the_pad_length_byte_before_the_priority_block() {
    // ⚠ The trap. The pad length byte comes FIRST, so a reader that took the
    // five bytes straight after the frame head is right on every unpadded frame
    // and silently wrong on a padded one, reporting a dependency built from the
    // pad length and four bytes of the real block.
    let mut bytes = Vec::from(PREFACE);
    let mut payload = vec![0x04_u8]; // pad length
    payload.extend_from_slice(&[0x80, 0x00, 0x00, 0x00, 0xff]);
    payload.extend_from_slice(&[0x82]); // the header block fragment
    payload.extend_from_slice(&[0x00; 4]); // the padding itself
    let length = u32::try_from(payload.len()).expect("small");
    bytes.extend_from_slice(&length.to_be_bytes()[1..]);
    bytes.push(0x1);
    bytes.push(FLAG_PADDED | FLAG_PRIORITY | FLAG_END_HEADERS);
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    bytes.extend_from_slice(&payload);

    let mut notes = Vec::new();
    let capture = h2::parse_connection(&bytes, ValuePolicy::NamesOnly, &mut notes)
        .expect("a well-framed connection");
    let priority = capture.half.stream_priority.expect("a priority block");
    assert!(priority.exclusive, "the pad byte was read as the block");
    assert_eq!(priority.stream_dependency, 0);
    assert_eq!(priority.weight_wire, 255);
    assert_eq!(capture.priority_block_hex().as_deref(), Some("80000000ff"));
}

#[test]
fn http2_keeps_a_frame_type_it_has_no_name_for() {
    // ⛔ The same rule as an unknown TLS extension. A sequence that silently
    // omits a frame is a sequence nobody can compare.
    let (capture, _) = read("h2-connection.hex");
    let other = capture
        .half
        .frames
        .iter()
        .find_map(|f| match f {
            Frame::Other {
                frame_type,
                payload_hex,
            } if *frame_type == 0x63 => Some(payload_hex.clone()),
            _ => None,
        })
        .expect("the unnamed frame is kept");
    assert_eq!(other, "dead");

    let raw = capture
        .frames
        .iter()
        .find(|f| f.frame_type == 0x63)
        .expect("and it is in the raw frame list too");
    assert_eq!(raw.declared_length, 2);
    assert_eq!(raw.bytes_arrived, 2);
}

#[test]
fn http2_records_a_standalone_priority_frame_separately_from_the_block() {
    // ⚠ The standalone frame and the block inside HEADERS are two seams, and a
    // model that merged them could not say which one a client used.
    let (capture, _) = read("h2-connection.hex");
    assert_eq!(capture.half.priority_frames.len(), 1);
    let frame = capture.half.priority_frames[0];
    assert_eq!(frame.stream_id, 3);
    assert!(!frame.priority.exclusive);
    assert_eq!(frame.priority.weight_wire, 200);

    // ⛔ The parsed frame and the raw frame it came from must agree. A value in
    // two places with no check between them drifts, and the copy a reader
    // trusts is the wrong one.
    let raw = capture
        .frames
        .iter()
        .find(|f| f.frame_type == h2::FRAME_PRIORITY)
        .expect("the raw frame is kept");
    assert_eq!(raw.stream_id, frame.stream_id);
    assert_eq!(raw.payload_hex, "00000000c8");
}

#[test]
fn http2_the_frame_list_is_the_arrival_sequence() {
    let (capture, _) = read("h2-connection.hex");
    let types: Vec<u8> = capture.frames.iter().map(|f| f.frame_type).collect();
    assert_eq!(types, vec![0x4, 0x8, 0x63, 0x2, 0x1]);
    assert!(
        capture.frames.iter().all(|f| !f.reserved_bit),
        "the fixture sets no reserved bit"
    );
}

#[test]
fn http2_refuses_bytes_that_are_not_a_connection_preface() {
    // ⚠ An error, not a note. The permissive rule covers what is unreadable
    // INSIDE a well-framed connection; bytes that are not one at all are a
    // different surface and saying so is how the listener picks the other
    // reader.
    let mut notes = Vec::new();
    let why = h2::parse_connection(
        b"GET / HTTP/1.1\r\nHost: example\r\n\r\n",
        ValuePolicy::NamesOnly,
        &mut notes,
    )
    .expect_err("an HTTP/1.1 request is not an HTTP/2 connection");
    assert!(why.contains("preface"), "{why}");
    assert!(notes.is_empty(), "{notes:?}");
}

#[test]
fn http2_notes_a_truncated_frame_rather_than_padding_it() {
    // ⛔ Count what arrived; do not trust what was declared. Padding to match a
    // declared length would record bytes nobody sent.
    let mut bytes = Vec::from(PREFACE);
    bytes.extend_from_slice(&[0x00, 0x00, 0x0c]); // declares twelve
    bytes.push(0x4);
    bytes.push(0x0);
    bytes.extend_from_slice(&0_u32.to_be_bytes());
    bytes.extend_from_slice(&[0x00, 0x06, 0x00, 0x01, 0x00, 0x01]); // six arrive

    let mut notes = Vec::new();
    let capture = h2::parse_connection(&bytes, ValuePolicy::NamesOnly, &mut notes)
        .expect("a well-framed connection");
    assert_eq!(capture.frames.len(), 1);
    assert_eq!(capture.frames[0].declared_length, 12);
    assert_eq!(capture.frames[0].bytes_arrived, 6);
    assert!(
        notes.iter().any(|n| n.why.contains("declares 12")),
        "{notes:?}"
    );
    let entries = capture.half.settings().expect("what did arrive is kept");
    assert_eq!(entries.len(), 1);
}

#[test]
fn http2_reaches_the_listener_over_a_cleartext_socket() {
    // ⭐ The driven half. The surface is cleartext and the PEER decides which
    // protocol it speaks, so the reader is chosen by the bytes rather than by a
    // flag the operator passed.
    let captures = feed(
        Config {
            protocol: Protocol::Cleartext,
            ..one_connection()
        },
        vec![fixture_bytes("h2-connection.hex")],
    );
    let capture = &captures[0];
    assert!(capture.reached_h2(), "{:?}", capture.notes);
    let http2 = capture.http2.as_ref().expect("the HTTP/2 half is recorded");
    assert_eq!(http2.frames.len(), 5);
    assert!(
        capture.request_line.is_none(),
        "it is not an HTTP/1.1 request"
    );
    // ⛔ The raw bytes are kept whatever the parser made of them.
    assert_eq!(capture.bytes_read, fixture_bytes("h2-connection.hex").len());
}

#[test]
fn http2_reassembles_a_connection_split_across_two_reads() {
    // ⛔ The preface carries a blank line at byte 16, so a completeness rule
    // that asks the HTTP/1.1 question first stops there. A client that sends
    // its preface and settings in one write and its HEADERS frame in the next
    // is the ordinary case, and a single-write test cannot see the difference:
    // the whole message is already in the buffer by the time the rule runs.
    let bytes = fixture_bytes("h2-connection.hex");
    let (head, tail) = bytes.split_at(PREFACE.len());
    let captures = support::feed_in_chunks(
        Config {
            protocol: Protocol::Cleartext,
            ..one_connection()
        },
        vec![head.to_vec(), tail.to_vec()],
        Duration::from_millis(60),
    );
    let capture = &captures[0];
    assert_eq!(capture.bytes_read, bytes.len(), "the read stopped early");
    let http2 = capture.http2.as_ref().expect("the HTTP/2 half is recorded");
    assert_eq!(http2.frames.len(), 5);
    assert!(capture.reached_h2());
}

#[test]
fn http2_a_cleartext_http1_request_is_still_read_as_http1() {
    // ⚠ The preface carries a blank line at byte 16, so the detection has to
    // ask the HTTP/2 question first. This is the other side of that: adding the
    // second reader must not have taken the first one away.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\nAccept: */*\r\n\r\n";
    let captures = feed(
        Config {
            protocol: Protocol::Cleartext,
            ..one_connection()
        },
        vec![request.to_vec()],
    );
    let capture = &captures[0];
    assert_eq!(capture.request_line.as_deref(), Some("GET / HTTP/1.1"));
    assert_eq!(capture.header_names, vec!["Host", "Accept"]);
    assert!(capture.http2.is_none());
    assert!(!capture.reached_h2());
}

#[test]
fn http2_until_h2_stops_at_the_first_connection_that_reached_it() {
    // ⚠ A browser opens sockets it abandons, and the first connection of a
    // navigation has been measured carrying no HTTP/2 at all. The run keeps
    // accepting until one does.
    let request = b"GET / HTTP/1.1\r\nHost: example\r\n\r\n".to_vec();
    let captures = feed(
        Config {
            protocol: Protocol::Cleartext,
            handshakes: 8,
            until_h2: true,
            ..one_connection()
        },
        vec![
            request.clone(),
            request,
            fixture_bytes("h2-connection.hex"),
            // ⛔ A fourth payload nothing should reach. If the run did not stop
            // it would block here until the accept timed out the test.
        ],
    );
    assert_eq!(captures.len(), 3, "the run did not stop at the HTTP/2 one");
    assert!(!captures[0].reached_h2());
    assert!(!captures[1].reached_h2());
    assert!(captures[2].reached_h2());
}

#[test]
fn http2_the_akamai_string_is_derived_from_the_frames() {
    // ⛔ A digest is derived from a profile and a profile is never derived from
    // a digest, so nothing stores this.
    let (capture, _) = read("h2-connection.hex");
    assert_eq!(
        capture.half.akamai_text(),
        "6:65537;1:8192;4:3145728;2:0|11259375|1:1:0:255|m,a,s,p"
    );

    // ⚠ And the string is what the model refuses to be. An absent block renders
    // `0`, which is also what a source reading only the string reports when it
    // could not read one, and that ambiguity is why three published readings of
    // this field disagree.
    let mut without = capture.half.clone();
    without.stream_priority = None;
    assert_eq!(
        without.akamai_text(),
        "6:65537;1:8192;4:3145728;2:0|11259375|0|m,a,s,p"
    );
}

#[test]
fn http2_every_frame_re_encodes_to_the_bytes_it_was_read_from() {
    // ⭐ THE ROUND TRIP THAT MAKES THE CORPUS'S RAW BLOCK A BACKSTOP. A profile
    // stores its frames as `RawFrame::wire_hex`, and a re-encoding that lost or
    // moved a byte would produce a raw block that reparses to something else
    // while looking complete.
    //
    // ⚠ Found untested by the guard-mutation pass: it was exercised only
    // indirectly, through the corpus rebuild, which cannot say WHICH byte moved.
    let bytes = support::fixture_bytes("h2-connection.hex");
    let mut notes = Vec::new();
    let capture = h2::parse_connection(&bytes, ValuePolicy::NamesOnly, &mut notes)
        .expect("the committed connection parses");

    let rebuilt: String = capture.frames.iter().map(h2::RawFrame::wire_hex).collect();
    let after_preface = b_ids_harness::hex(&bytes[PREFACE.len()..]);
    assert_eq!(
        rebuilt, after_preface,
        "the frames re-encode to the bytes behind the preface, byte for byte"
    );
}

#[test]
fn http2_re_encoding_keeps_a_declared_length_that_disagrees_with_what_arrived() {
    // ⛔ A frame that declared more than it delivered is what the wire carried,
    // and re-encoding it with the arrived length would produce bytes no client
    // sent while looking tidier. The disagreement IS the measurement.
    let frame = h2::RawFrame {
        declared_length: 0x00ff_ffff,
        bytes_arrived: 2,
        frame_type: 0x04,
        flags: 0x01,
        stream_id: 1,
        reserved_bit: false,
        payload_hex: "0000".to_owned(),
    };
    assert_eq!(
        frame.wire_hex(),
        "ffffff040100000001".to_owned() + "0000",
        "the head carries the DECLARED length and the arrived payload follows"
    );
}

#[test]
fn http2_re_encoding_puts_the_reserved_bit_back_where_it_was_read_from() {
    // ⚠ The specification says a receiver ignores it, so a sender that sets it
    // is a sender that stands out. A re-encoding that dropped it would erase
    // exactly that.
    let frame = h2::RawFrame {
        declared_length: 0,
        bytes_arrived: 0,
        frame_type: 0x00,
        flags: 0x00,
        stream_id: 1,
        reserved_bit: true,
        payload_hex: String::new(),
    };
    assert_eq!(frame.wire_hex(), "00000000008000_0001".replace('_', ""));
}
