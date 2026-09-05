//! The HTTP/2 PRIORITY block, emitted.
//!
//! ⛔ **A client that omits it carries a zero in one field of four** in a
//! widely-read HTTP/2 fingerprint, and that is one of the fields an origin can
//! still tell apart. `docs/history/todo/emitters.md`, `EMIT-03`.
//!
//! ⭐ **The measurement is in and it is unanimous.** Every profile this corpus
//! holds carries the block, and every one of them carries the same value:
//! `exclusive: true, stream_dependency: 0, weight_wire: 255`, across two
//! browsers, three majors and two platforms. So the Approach's second branch,
//! which would have closed the entry on a negative result, does not apply.
//!
//! ⚠ **The library this builds on had no way to write it**, and upstream will
//! not take the change for a stated reason rather than by neglect: RFC 9113
//! deprecates stream priority, so a release adding a way to send one would add
//! a way to do what the specification tells clients not to do. The library is
//! vendored under [`../../../vendor/h2`] and patched there; `patches/README.md`
//! records what changed.
//!
//! ⛔ **This module writes a frame. It opens no socket and speaks to nothing.**
//! `b-ids-cli` is the one thing in this tree that puts bytes on a wire, and
//! `docs/architecture.md` says it must never grow into a general-purpose HTTP
//! client.

use b_ids_schema::http2::StreamPriority;
use bytes::{BufMut, BytesMut};
use h2::frame::{Headers, Pseudo, StreamId};
use h2::hpack::Encoder;
use http::HeaderMap;

/// The default a peer must accept, from RFC 9113 section 6.5.2.
///
/// ⚠ **A floor rather than a preference.** A frame larger than this needs the
/// peer to have advertised a larger `SETTINGS_MAX_FRAME_SIZE`, so a caller that
/// does not know what the peer advertised uses this.
pub const DEFAULT_MAX_FRAME_SIZE: usize = 16_384;

/// What a caller got wrong, rather than what the wire did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The stream identifier a client may not open a stream on.
    ///
    /// ⛔ Stream 0 is the connection control stream and carries no HEADERS,
    /// and a client's streams are odd. A frame on an even one is a frame a
    /// server refuses, which is a defect worth catching here rather than on a
    /// socket.
    NotAClientStream(u32),
    /// A stream cannot depend on itself.
    ///
    /// ⚠ RFC 7540 section 5.3.1 calls this a `PROTOCOL_ERROR` on the
    /// connection, and this library's own reader refuses it on load, so
    /// emitting one would produce a frame this project could not read back.
    DependsOnItself(u32),
    /// The frame size a peer is not required to accept.
    TooSmallToFrame(usize),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAClientStream(id) => write!(
                f,
                "{id} is not a stream a client opens: 0 is the control stream and a client's own \
                 streams are odd"
            ),
            Self::DependsOnItself(id) => write!(
                f,
                "stream {id} cannot depend on itself, which is a connection error and which this \
                 library's own reader refuses on load"
            ),
            Self::TooSmallToFrame(size) => write!(
                f,
                "a maximum frame size of {size} is below the 16384 every peer must accept"
            ),
        }
    }
}

impl std::error::Error for Refusal {}

/// One HEADERS frame carrying the PRIORITY block, ready for a socket.
///
/// ⭐ **The five bytes and the flag, from one call.** ⛔ A head carrying the
/// flag with no block is a frame a peer cannot parse, and a block with no flag
/// is five bytes of header block that decodes as garbage; the patched library
/// sets both in `Headers::set_stream_priority` so the pair is not a caller's to
/// remember.
///
/// ⚠ **The frame length and any CONTINUATION split follow for free**, because
/// the patch writes into a closure the encoder already ran between the frame
/// head and the header block, and the payload length is computed after it. That
/// is the seam, and it is why this entry was ever an `S`.
///
/// ⚠ **`weight_wire` is the byte on the wire**, in `[0, 255]`, which is one
/// less than the `[1, 256]` weight the specification defines. `SCHEMA-09` is
/// the rule that every field is named for the wire, and this crosses no
/// boundary where it could be re-applied.
///
/// # Errors
///
/// [`Refusal`], for a stream identifier or a frame size a caller should not
/// have asked for. ⛔ Nothing here fails for a reason about the bytes.
pub fn headers_with_priority(
    stream_id: u32,
    fields: &HeaderMap,
    pseudo: Pseudo,
    priority: &StreamPriority,
    max_frame_size: usize,
) -> Result<Vec<u8>, Refusal> {
    if stream_id == 0 || stream_id.is_multiple_of(2) {
        return Err(Refusal::NotAClientStream(stream_id));
    }
    if priority.stream_dependency == stream_id {
        return Err(Refusal::DependsOnItself(stream_id));
    }
    if max_frame_size < DEFAULT_MAX_FRAME_SIZE {
        return Err(Refusal::TooSmallToFrame(max_frame_size));
    }

    let mut frame = Headers::new(StreamId::from(stream_id), pseudo, fields.clone());
    frame.set_stream_priority(h2::frame::StreamDependency::new(
        StreamId::from(priority.stream_dependency),
        priority.weight_wire,
        priority.exclusive,
    ));

    // ⚠ THE ENCODER'S TABLE IS THIS CONNECTION'S, so it is created here and
    // dropped here. A shared encoder would make the second frame this function
    // produced depend on the first, and a caller comparing two frames would be
    // comparing two dynamic-table states.
    let mut encoder = Encoder::new(4096, 0);
    let mut out = BytesMut::new();

    // ⛔ THE CONTINUATION CHAIN IS DRIVEN TO THE END. `encode` returns the
    // remainder when the header block did not fit, and a caller that dropped it
    // would emit a HEADERS frame with no END_HEADERS and nothing after it,
    // which is a connection a peer closes.
    let mut buf = (&mut out).limit(max_frame_size);
    let mut rest = frame.encode(&mut encoder, &mut buf);
    while let Some(continuation) = rest {
        let mut buf = (&mut out).limit(max_frame_size);
        rest = continuation.encode(&mut buf);
    }

    Ok(out.to_vec())
}

/// The whole of what a client sends before its first request, with the frame
/// above at the end of it.
///
/// ⭐ **Assembled here so the suite can hand it to this project's own reader**,
/// which parses a CONNECTION rather than a frame: the preface, a SETTINGS frame
/// and then the request. ⛔ It is not a client and it never becomes one.
///
/// # Errors
///
/// Whatever [`headers_with_priority`] refuses.
pub fn opening_with_priority(
    stream_id: u32,
    fields: &HeaderMap,
    pseudo: Pseudo,
    priority: &StreamPriority,
    max_frame_size: usize,
) -> Result<Vec<u8>, Refusal> {
    let headers = headers_with_priority(stream_id, fields, pseudo, priority, max_frame_size)?;
    let mut out = Vec::with_capacity(headers.len() + 33);
    out.extend_from_slice(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n");
    // An empty SETTINGS frame: three length bytes, the type, no flags, stream 0.
    out.extend_from_slice(&[0, 0, 0, 0x04, 0, 0, 0, 0, 0]);
    out.extend_from_slice(&headers);
    Ok(out)
}
