//! `EMIT-03`. The HTTP/2 PRIORITY block, emitted and read back.
//!
//! ⛔ **Every test name contains `priority_block`**, because
//! `cargo test -p b-ids-emit priority_block` is this entry's acceptance and a
//! filter that selects nothing exits 0 having run nothing.
//!
//! ⭐ **THE READER IS NOT THE WRITER, and that is what makes this a comparison
//! rather than the emitter checking its own arithmetic.** The bytes are written
//! by the vendored and patched `h2` and read by `b_ids_harness::h2`, which is
//! this project's own frame reader, written here and used on every capture in
//! the corpus. ⛔ Neither knows about the other.

use b_ids_emit::priority::{
    DEFAULT_MAX_FRAME_SIZE, Refusal, headers_with_priority, opening_with_priority,
};
use b_ids_harness::h2::parse_connection;
use b_ids_schema::http::ValuePolicy;
use b_ids_schema::http2::StreamPriority;
use h2::frame::Pseudo;
use http::{HeaderMap, HeaderValue, Method, uri::Uri};

/// What every profile in this corpus carries, at every route.
///
/// ⭐ **Measured, not chosen.** Six of six on 2026-09-02 and twelve of twelve
/// since: two browsers, three majors, two platforms, one value.
fn what_browsers_send() -> StreamPriority {
    StreamPriority {
        exclusive: true,
        stream_dependency: 0,
        weight_wire: 255,
    }
}

fn a_request() -> (Pseudo, HeaderMap) {
    let pseudo = Pseudo::request(
        Method::GET,
        Uri::from_static("https://example.invalid/"),
        None,
    );
    let mut fields = HeaderMap::new();
    fields.insert("accept", HeaderValue::from_static("*/*"));
    (pseudo, fields)
}

#[test]
fn priority_block_the_first_headers_frame_carries_the_flag_and_the_five_bytes() {
    let (pseudo, fields) = a_request();
    let want = what_browsers_send();
    let wire = opening_with_priority(1, &fields, pseudo, &want, DEFAULT_MAX_FRAME_SIZE)
        .expect("a client stream and a frame size a peer must accept");

    let mut notes = Vec::new();
    let capture = parse_connection(&wire, ValuePolicy::NamesOnly, &mut notes)
        .expect("this project's own reader parses what the patched library wrote");

    // ⛔ THE FLAG AND THE BLOCK TOGETHER. `priority_block_hex` returns None when
    // the flag is clear, so a frame carrying five bytes with no flag reads here
    // exactly like one carrying nothing, which is the failure the patch's
    // single setter exists to make unrepresentable.
    let got = capture
        .priority_block_hex()
        .expect("the flag is set and the five bytes are behind it");
    assert_eq!(got.len(), 10, "five bytes is ten hex characters: {got}");

    // ⭐ THE VALUE, BYTE FOR BYTE. Exclusive is the top bit of the 31-bit
    // dependency, so 1:0:255 is 80000000 followed by ff.
    assert_eq!(got, "80000000ff", "the block is not what a browser sends");
}

#[test]
fn priority_block_the_harness_reads_back_what_the_schema_asked_for() {
    // ⚠ THE ROUND TRIP THROUGH THE MODEL, not through a hex string somebody
    // typed. The value goes in as a StreamPriority and comes back out of the
    // reader as bytes, and this is where the two are compared.
    for want in [
        what_browsers_send(),
        StreamPriority {
            exclusive: false,
            stream_dependency: 3,
            weight_wire: 0,
        },
        StreamPriority {
            exclusive: true,
            stream_dependency: 0x7fff_ffff,
            weight_wire: 1,
        },
    ] {
        let (pseudo, fields) = a_request();
        let wire = opening_with_priority(1, &fields, pseudo, &want, DEFAULT_MAX_FRAME_SIZE)
            .expect("a client stream");
        let mut notes = Vec::new();
        let capture =
            parse_connection(&wire, ValuePolicy::NamesOnly, &mut notes).expect("a connection");
        let got = capture.priority_block_hex().expect("a block");

        let mut expected = want.stream_dependency;
        if want.exclusive {
            expected |= 1 << 31;
        }
        let bytes = [
            expected.to_be_bytes()[0],
            expected.to_be_bytes()[1],
            expected.to_be_bytes()[2],
            expected.to_be_bytes()[3],
            want.weight_wire,
        ];
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(got, hex, "over {want:?}");
    }
}

#[test]
fn priority_block_the_frame_length_is_right_across_a_continuation_split() {
    // ⛔ THE CASE THE SEAM MAKES FREE, AND THE ONE WORTH CHECKING ANYWAY. The
    // five bytes are written before the header block and the payload length is
    // computed after, so a header block that overflows one frame has to split
    // with the five bytes counted in the FIRST frame's length and in no other.
    // ⚠ A patch that wrote them after the block, or that ran the closure once
    // per continuation, would produce a connection that parses until it does
    // not.
    let (pseudo, mut fields) = a_request();
    // ⚠ Big enough to need a continuation at the smallest frame size every peer
    // must accept, and built rather than pasted so the size is visible.
    for i in 0..40 {
        let name = format!("x-padding-{i:02}");
        fields.insert(
            http::header::HeaderName::from_bytes(name.as_bytes()).expect("a header name"),
            HeaderValue::from_str(&"v".repeat(600)).expect("a header value"),
        );
    }

    let want = what_browsers_send();
    let wire = opening_with_priority(1, &fields, pseudo, &want, DEFAULT_MAX_FRAME_SIZE)
        .expect("a client stream");

    let mut notes = Vec::new();
    let capture = parse_connection(&wire, ValuePolicy::NamesOnly, &mut notes)
        .expect("a split connection still parses");

    // ⛔ IT ACTUALLY SPLIT. A test that asserted the split case over a block
    // that fitted in one frame would be green and would check nothing.
    let continuations = capture
        .frames
        .iter()
        .filter(|f| f.frame_type == 0x09)
        .count();
    assert!(
        continuations >= 1,
        "the header block did not need a continuation, so this case checked nothing: {} frame(s)",
        capture.frames.len()
    );

    // ⭐ AND THE BLOCK IS STILL THERE, IN THE FIRST FRAME, unchanged.
    assert_eq!(
        capture.priority_block_hex().as_deref(),
        Some("80000000ff"),
        "the block did not survive the split"
    );

    // ⚠ EVERY FRAME IS WITHIN THE SIZE THE PEER MUST ACCEPT, which is what
    // "the frame length is correct" means for a split.
    for frame in &capture.frames {
        assert!(
            frame.payload_hex.len() / 2 <= DEFAULT_MAX_FRAME_SIZE,
            "a frame of {} byte(s) exceeds the size every peer must accept",
            frame.payload_hex.len() / 2
        );
    }
}

#[test]
fn priority_block_a_stream_a_client_cannot_open_is_refused() {
    // ⛔ REFUSED HERE RATHER THAN ON A SOCKET. Stream 0 is the connection
    // control stream and a client's own streams are odd, so both of these are
    // frames a server closes the connection over.
    let (pseudo, fields) = a_request();
    let want = what_browsers_send();
    assert_eq!(
        headers_with_priority(0, &fields, pseudo, &want, DEFAULT_MAX_FRAME_SIZE),
        Err(Refusal::NotAClientStream(0))
    );

    let (pseudo, fields) = a_request();
    assert_eq!(
        headers_with_priority(2, &fields, pseudo, &want, DEFAULT_MAX_FRAME_SIZE),
        Err(Refusal::NotAClientStream(2))
    );

    // ⚠ AND A STREAM THAT DEPENDS ON ITSELF, which RFC 7540 section 5.3.1 calls
    // a connection error and which this library's own reader refuses on load,
    // so emitting one would produce a frame this project could not read back.
    let (pseudo, fields) = a_request();
    let self_dependent = StreamPriority {
        exclusive: false,
        stream_dependency: 1,
        weight_wire: 16,
    };
    assert_eq!(
        headers_with_priority(1, &fields, pseudo, &self_dependent, DEFAULT_MAX_FRAME_SIZE),
        Err(Refusal::DependsOnItself(1))
    );

    // ⚠ AND A FRAME SIZE BELOW THE FLOOR EVERY PEER MUST ACCEPT.
    let (pseudo, fields) = a_request();
    assert_eq!(
        headers_with_priority(1, &fields, pseudo, &want, 1024),
        Err(Refusal::TooSmallToFrame(1024))
    );
}

#[test]
fn priority_block_the_patch_is_what_puts_it_there() {
    // ⛔ THE CONTROL, AND WITHOUT IT THIS SUITE PROVES NOTHING ABOUT THE PATCH.
    // An unpatched `h2` passes `|_| {}` into the same closure, so a frame built
    // without `set_stream_priority` must carry neither the flag nor the bytes.
    // ⚠ If this case ever goes red with the others green, the block is coming
    // from somewhere that is not the patch.
    use bytes::BufMut;
    let (pseudo, fields) = a_request();
    let mut frame = h2::frame::Headers::new(h2::frame::StreamId::from(1), pseudo, fields);
    frame.set_end_headers();
    let mut encoder = h2::hpack::Encoder::new(4096, 0);
    let mut out = bytes::BytesMut::new();
    let mut buf = (&mut out).limit(DEFAULT_MAX_FRAME_SIZE);
    let rest = frame.encode(&mut encoder, &mut buf);
    assert!(rest.is_none(), "the control fitted in one frame");

    let mut wire = Vec::new();
    wire.extend_from_slice(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
    wire.extend_from_slice(&[0, 0, 0, 0x04, 0, 0, 0, 0, 0]);
    wire.extend_from_slice(&out);

    let mut notes = Vec::new();
    let capture =
        parse_connection(&wire, ValuePolicy::NamesOnly, &mut notes).expect("a connection");
    assert!(capture.opened_a_stream(), "the control sent no HEADERS");
    assert_eq!(
        capture.priority_block_hex(),
        None,
        "a frame built without the patch's setter carried a priority block"
    );
}
