//! Reading an HTTP/2 connection off the wire: the preface and the frames
//! behind it.
//!
//! ⛔ **A frame sequence, not a settings map.** Order is part of the
//! fingerprint, and a map cannot say which settings were ABSENT. One browser
//! sends no `SETTINGS_MAX_FRAME_SIZE` where a general-purpose stack sends the
//! protocol default, and those are two visibly different connections.
//!
//! ⛔ **The PRIORITY block is read as bytes, never as a rendered string.** An
//! Akamai string cannot distinguish a block that was not sent from a block that
//! was not read, and that ambiguity is why three published readings of the
//! field disagree. `docs/inherited-claims.md` section 5 has the table.
//!
//! ⛔ **Parse permissively, emit exactly.** Everything unreadable inside a
//! well-framed connection becomes a [`Note`]; only bytes that are not an
//! HTTP/2 connection at all are an error. Every frame is kept, including a
//! frame type this project has no name for.

use b_ids_schema::http::ValuePolicy;
use b_ids_schema::http2::{Frame, Http2Half, PriorityFrame, SettingEntry, StreamPriority};
use serde::{Deserialize, Serialize};

use crate::bytes::{Cursor, hex, unhex};
use crate::hpack::{Decoder, HeaderRecord};
use crate::note::Note;

/// The connection preface a client sends before its first frame.
pub const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// The fixed frame head: a 24-bit length, a type, a flags byte, and a reserved
/// bit beside a 31-bit stream identifier.
pub const FRAME_HEAD_LEN: usize = 9;

/// The PRIORITY block: an exclusive bit, a 31-bit dependency, and one weight
/// byte.
pub const PRIORITY_BLOCK_LEN: usize = 5;

/// HEADERS.
pub const FRAME_HEADERS: u8 = 0x1;
/// The standalone PRIORITY frame, which is a different seam from the block
/// inside a HEADERS frame.
pub const FRAME_PRIORITY: u8 = 0x2;
/// SETTINGS.
pub const FRAME_SETTINGS: u8 = 0x4;
/// WINDOW_UPDATE.
pub const FRAME_WINDOW_UPDATE: u8 = 0x8;
/// CONTINUATION, which carries the rest of a header block.
pub const FRAME_CONTINUATION: u8 = 0x9;

/// SETTINGS acknowledgement. ⚠ The same bit is END_STREAM on a HEADERS frame,
/// which is why the flag is read against the frame type rather than alone.
pub const FLAG_ACK: u8 = 0x01;
/// The header block ends in this frame.
pub const FLAG_END_HEADERS: u8 = 0x04;
/// A pad length byte leads the payload.
pub const FLAG_PADDED: u8 = 0x08;
/// A PRIORITY block follows the pad length byte, where there is one.
pub const FLAG_PRIORITY: u8 = 0x20;

/// One frame exactly as it arrived, before anything was made of it.
///
/// ⛔ **The flags byte of every frame is kept**, not only of the frames this
/// project currently reads a meaning from. If the wire carried it, the capture
/// records it: a capture is a moment that cannot be retaken, and a field nobody
/// wanted this year is a field nobody can add next year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawFrame {
    /// The payload length the frame head declared.
    pub declared_length: u32,
    /// ⛔ How many payload bytes actually arrived. Counting what arrived rather
    /// than trusting what was declared is what stops a truncated frame being
    /// recorded as a complete one.
    pub bytes_arrived: usize,
    /// The frame type byte.
    pub frame_type: u8,
    /// The flags byte.
    pub flags: u8,
    /// The stream identifier, with the reserved bit masked off.
    pub stream_id: u32,
    /// ⚠ The reserved bit beside the stream identifier. The specification says
    /// a receiver ignores it, so a sender that sets it is a sender that stands
    /// out.
    pub reserved_bit: bool,
    /// The payload that arrived, hex-encoded.
    pub payload_hex: String,
}

impl RawFrame {
    /// This frame as it was on the wire: the nine-byte head, then the payload
    /// that arrived.
    ///
    /// ⭐ **The one place the frame head layout is written in the emitting
    /// direction**, and it is here rather than in the corpus writer because the
    /// reading direction is here. Two modules that each know where the flags
    /// byte sits is two places for that to be wrong.
    ///
    /// ⛔ **The DECLARED length is written and the ARRIVED payload follows,
    /// even where they disagree.** A frame that declared more than it delivered
    /// is what the wire carried, and re-encoding it with the arrived length
    /// would produce bytes no client sent while looking tidier. The
    /// disagreement is the measurement.
    #[must_use]
    pub fn wire_hex(&self) -> String {
        let mut head = Vec::with_capacity(FRAME_HEAD_LEN);
        let length = self.declared_length & 0x00ff_ffff;
        head.push(u8::try_from((length >> 16) & 0xff).unwrap_or(0));
        head.push(u8::try_from((length >> 8) & 0xff).unwrap_or(0));
        head.push(u8::try_from(length & 0xff).unwrap_or(0));
        head.push(self.frame_type);
        head.push(self.flags);
        // ⚠ The reserved bit goes back where it was read from. The
        // specification says a receiver ignores it, so a sender that set it is
        // a sender that stands out, and a re-encoding that dropped it would
        // erase exactly that.
        let stream = (self.stream_id & 0x7fff_ffff) | (u32::from(self.reserved_bit) << 31);
        head.extend_from_slice(&stream.to_be_bytes());
        format!("{}{}", hex(&head), self.payload_hex)
    }
}

/// What one HTTP/2 connection produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Http2Capture {
    /// The half of a profile this connection filled in.
    pub half: Http2Half,
    /// Every frame that arrived, in order.
    pub frames: Vec<RawFrame>,
    /// The first request's header fields, in wire order, with what HPACK did
    /// to each.
    ///
    /// ⛔ `cookie` and `authorization` keep their NAME and their POSITION
    /// here and lose their value, whatever the policy. The decoder still stores
    /// them, because the dynamic table has to see every field the encoder
    /// inserted or every later index is wrong. That is the third door into the
    /// credential rule and it is gated like the other two. `SCHEMA-14`.
    pub headers: Vec<HeaderRecord>,
    /// Every dynamic table size update the encoder sent, in order.
    ///
    /// ⭐ A choice the encoder made about a table it owns, and one that a
    /// settings value does not predict.
    pub table_size_updates: Vec<usize>,
}

impl Http2Capture {
    /// The five raw PRIORITY-block bytes from the first HEADERS frame.
    ///
    /// ⭐ **Derived rather than stored.** The bytes are already inside the
    /// frame's payload, and a second copy is a second thing to keep in step.
    #[must_use]
    pub fn priority_block_hex(&self) -> Option<String> {
        let frame = self.frames.iter().find(|f| f.frame_type == FRAME_HEADERS)?;
        if frame.flags & FLAG_PRIORITY == 0 {
            return None;
        }
        let payload = unhex(&frame.payload_hex).ok()?;
        let skip = usize::from(frame.flags & FLAG_PADDED != 0);
        payload
            .get(skip..skip.checked_add(PRIORITY_BLOCK_LEN)?)
            .map(hex)
    }

    /// Whether a HEADERS frame arrived, which is what "this connection reached
    /// HTTP/2" means for a capture.
    #[must_use]
    pub fn opened_a_stream(&self) -> bool {
        self.frames.iter().any(|f| f.frame_type == FRAME_HEADERS)
    }
}

/// Whether these bytes are the start of an HTTP/2 connection preface.
///
/// ⚠ **True for a PREFIX as well as for the whole thing**, because the
/// completeness rule has to answer this before all 24 bytes have arrived. The
/// preface contains a blank line at byte 16, so a cleartext reader that stopped
/// at the first blank line would cut an HTTP/2 connection in half and read the
/// remainder as nothing.
#[must_use]
pub fn starts_like_preface(bytes: &[u8]) -> bool {
    if bytes.len() >= PREFACE.len() {
        bytes.starts_with(PREFACE)
    } else {
        PREFACE.starts_with(bytes)
    }
}

/// Whether the first header block has been fully received.
///
/// ⭐ This is where a cleartext read of an HTTP/2 connection stops: the frames
/// that carry the fingerprint are the ones a client sends before its first
/// request is complete, and waiting for a response nobody is going to send
/// costs the whole read timeout.
#[must_use]
pub fn first_header_block_complete(bytes: &[u8]) -> bool {
    let Some(rest) = bytes.strip_prefix(PREFACE) else {
        return false;
    };
    let mut cursor = Cursor::new(rest);
    let mut in_header_block = false;
    while cursor.remaining() >= FRAME_HEAD_LEN {
        let Some(length) = cursor.u24() else { break };
        let Some(frame_type) = cursor.u8() else { break };
        let Some(flags) = cursor.u8() else { break };
        let Some(_identifier) = cursor.u32() else {
            break;
        };
        if cursor
            .take(usize::try_from(length).unwrap_or(usize::MAX))
            .is_none()
        {
            return false;
        }
        match frame_type {
            FRAME_HEADERS | FRAME_CONTINUATION => {
                if flags & FLAG_END_HEADERS != 0 {
                    return true;
                }
                in_header_block = true;
            }
            // ⚠ A header block may not be interleaved with any other frame, so
            // anything else here means the block ended in a way this reader
            // cannot follow. Stop rather than wait.
            _ if in_header_block => return true,
            _ => {}
        }
    }
    false
}

/// Read an HTTP/2 connection: the preface, then every frame behind it.
///
/// ⚠ Notes accumulate into `notes` rather than into the returned value, because
/// a note belongs to the capture as a whole and a field that is always empty
/// once stored is a field a reader has to ask about.
///
/// ⛔ `policy` decides whether header VALUES are recorded, and
/// [`ValuePolicy::NamesOnly`] is the default everywhere. `cookie` and
/// `authorization` lose their value under either, and keep their name and
/// their place in the order.
///
/// # Errors
///
/// Only where the bytes are not an HTTP/2 connection at all. Everything
/// unreadable INSIDE a well-framed connection becomes a note.
pub fn parse_connection(
    bytes: &[u8],
    policy: ValuePolicy,
    notes: &mut Vec<Note>,
) -> Result<Http2Capture, String> {
    let mut cursor = Cursor::new(bytes);
    let opening = cursor.take(PREFACE.len()).ok_or_else(|| {
        format!(
            "{} byte(s) arrived and the connection preface is {}",
            bytes.len(),
            PREFACE.len()
        )
    })?;
    if opening != PREFACE {
        return Err(format!(
            "the first {} bytes are not the HTTP/2 connection preface",
            PREFACE.len()
        ));
    }

    let mut raw_frames = Vec::new();
    let mut model_frames = Vec::new();
    let mut priority_frames = Vec::new();
    let mut stream_priority = None;
    let mut first_headers_read = false;

    // ⛔ ONE decoder for the whole connection. The dynamic table is connection
    // state, and a decoder reset between header blocks resolves every later
    // index against the wrong table and produces plausible nonsense.
    //
    // ⚠ It starts at the protocol's own table size because this harness sends
    // no SETTINGS, so a peer has been told nothing else.
    let mut decoder = Decoder::default();
    let mut fragment: Vec<u8> = Vec::new();
    let mut in_header_block = false;
    let mut headers = Vec::new();
    let mut table_size_updates = Vec::new();
    let mut blocks_read = 0_usize;

    while cursor.remaining() > 0 {
        if cursor.remaining() < FRAME_HEAD_LEN {
            notes.push(Note::new(
                "http2.frames",
                format!(
                    "{} trailing byte(s) is shorter than the {FRAME_HEAD_LEN}-byte frame head",
                    cursor.remaining()
                ),
            ));
            break;
        }
        let (Some(declared_length), Some(frame_type), Some(flags), Some(identifier)) =
            (cursor.u24(), cursor.u8(), cursor.u8(), cursor.u32())
        else {
            break;
        };
        let reserved_bit = identifier >> 31 == 1;
        let stream_id = identifier & 0x7fff_ffff;

        let want = usize::try_from(declared_length).unwrap_or(usize::MAX);
        // ⛔ Count what arrived; do not trust what was declared. Padding to
        // match a declared length would record bytes nobody sent.
        let payload = match cursor.take(want) {
            Some(payload) => payload,
            None => {
                let arrived = cursor.remaining();
                notes.push(Note::new(
                    format!("http2.frame.{frame_type:#04x}"),
                    format!("declares {declared_length} payload byte(s) and {arrived} arrived"),
                ));
                cursor.take(arrived).unwrap_or(&[])
            }
        };

        raw_frames.push(RawFrame {
            declared_length,
            bytes_arrived: payload.len(),
            frame_type,
            flags,
            stream_id,
            reserved_bit,
            payload_hex: hex(payload),
        });

        match frame_type {
            // ⚠ An acknowledgement carries no entries, and recording it as a
            // SETTINGS frame with an empty list would read as a client that
            // sent no settings. It is kept as the frame it is.
            FRAME_SETTINGS if flags & FLAG_ACK == 0 => model_frames.push(Frame::Settings {
                entries: settings_entries(payload, notes),
            }),
            // ⚠ Only the CONNECTION-level update is the fingerprint's window
            // increment. A stream-level one is kept as the frame it is, with
            // its payload, rather than answering for the connection.
            FRAME_WINDOW_UPDATE if stream_id == 0 => match Cursor::new(payload).u32() {
                Some(value) => model_frames.push(Frame::WindowUpdate {
                    window_size_increment: value & 0x7fff_ffff,
                }),
                None => {
                    notes.push(Note::new(
                        "http2.window_update",
                        format!(
                            "carries {} payload byte(s) and an increment is four",
                            payload.len()
                        ),
                    ));
                    model_frames.push(other_frame(frame_type, payload));
                }
            },
            FRAME_HEADERS => {
                model_frames.push(Frame::Headers {
                    stream_id,
                    has_priority_block: flags & FLAG_PRIORITY != 0,
                });
                if !first_headers_read {
                    first_headers_read = true;
                    stream_priority = headers_priority(payload, flags, notes);
                }
                fragment.clear();
                fragment.extend_from_slice(&header_block_fragment(payload, flags, notes));
                in_header_block = true;
            }
            // ⚠ A header block may be split across frames, and a decoder fed
            // half of one produces nothing rather than half the fields. The
            // fragments are joined before anything is decoded.
            FRAME_CONTINUATION if in_header_block => {
                fragment.extend_from_slice(payload);
                model_frames.push(other_frame(frame_type, payload));
            }
            FRAME_PRIORITY => {
                match read_priority_block(payload) {
                    Some(priority) => priority_frames.push(PriorityFrame {
                        stream_id,
                        priority,
                    }),
                    None => notes.push(Note::new(
                        "http2.priority_frame",
                        format!(
                            "carries {} payload byte(s) and a priority block is \
                             {PRIORITY_BLOCK_LEN}",
                            payload.len()
                        ),
                    )),
                }
                model_frames.push(other_frame(frame_type, payload));
            }
            _ => model_frames.push(other_frame(frame_type, payload)),
        }

        if in_header_block
            && matches!(frame_type, FRAME_HEADERS | FRAME_CONTINUATION)
            && flags & FLAG_END_HEADERS != 0
        {
            in_header_block = false;
            blocks_read += 1;
            match decoder.decode(&fragment) {
                Ok(decoded) => {
                    table_size_updates.extend(decoded.table_size_updates);
                    for position in &decoded.not_utf8 {
                        notes.push(Note::new(
                            format!("http2.headers.{position}"),
                            "the name or the value is not valid UTF-8, and the raw bytes are the \
                             capture's own record of it",
                        ));
                    }
                    // ⛔ The FIRST block is the request the fingerprint
                    // describes. A later one is still decoded, because the
                    // dynamic table has to stay in step, and it is not
                    // recorded as though it were the same request.
                    if blocks_read == 1 {
                        headers = record_fields(&decoded.fields, policy);
                    }
                }
                Err(why) => notes.push(Note::new("http2.headers", why)),
            }
        }
    }
    if blocks_read > 1 {
        notes.push(Note::new(
            "http2.headers",
            format!(
                "{blocks_read} header block(s) arrived and the first one is recorded. A \
                 connection carries more than one request and SCHEMA-04's variants are how they \
                 are told apart"
            ),
        ));
    }

    if in_header_block {
        notes.push(Note::new(
            "http2.headers",
            "a header block began and no frame closed it, so its fields are not decoded",
        ));
    }

    // ⛔ Read from the recorded fields rather than counted separately. A
    // pseudo-header order that disagreed with the header list would be one
    // quantity in two places with nothing checking them.
    let pseudo_header_order = headers
        .iter()
        .map(|h| h.name.clone())
        .filter(|name| name.starts_with(':'))
        .collect();

    Ok(Http2Capture {
        half: Http2Half {
            frames: model_frames,
            stream_priority,
            priority_frames,
            pseudo_header_order,
            // ⛔ The capture records the INCREMENT, because the increment is
            // what the wire carried. The window is the same quantity in the
            // human unit and it is derived, never captured.
            connection_window: None,
        },
        frames: raw_frames,
        headers,
        table_size_updates,
    })
}

/// The header block fragment inside a HEADERS frame's payload.
///
/// ⚠ **Three things sit in front of it or behind it**, and each is optional:
/// the pad length byte, the five-byte priority block, and the padding itself at
/// the end. A reader that took the whole payload would hand the padding to the
/// decoder as though it were coded fields.
fn header_block_fragment(payload: &[u8], flags: u8, notes: &mut Vec<Note>) -> Vec<u8> {
    let mut cursor = Cursor::new(payload);
    let pad_len = if flags & FLAG_PADDED == 0 {
        0
    } else {
        usize::from(cursor.u8().unwrap_or(0))
    };
    if flags & FLAG_PRIORITY != 0 {
        let _ = cursor.take(PRIORITY_BLOCK_LEN);
    }
    let remaining = cursor.remaining();
    if pad_len > remaining {
        // ⛔ A note, never a repair. Padding longer than what is left is a
        // connection error by the specification, and trimming to fit would
        // record a header block nobody sent.
        notes.push(Note::new(
            "http2.headers",
            format!("{pad_len} byte(s) of padding declared and {remaining} remain"),
        ));
        return Vec::new();
    }
    cursor
        .take(remaining - pad_len)
        .unwrap_or_default()
        .to_vec()
}

/// Apply the capture's own rules to what the decoder read.
///
/// ⛔ **This is the third door into the credential rule**, and it is gated like
/// the model's and the HTTP/1.1 reader's. The decoder deliberately does not
/// filter, because the dynamic table has to see every field the encoder
/// inserted.
///
/// ⭐ **A credential keeps its NAME and its POSITION and loses its value.**
/// Before 2026-09-02 it was dropped entirely, so the recorded order closed over
/// the gap and a consumer reading the sequence believed it had the whole of it.
/// `docs/history/todo/schema.md`, `SCHEMA-14`. ⛔ There is no branch here that can put the
/// value into the field: the match is on the policy for an ordinary header and
/// `None` unconditionally for a credential.
fn record_fields(fields: &[HeaderRecord], policy: ValuePolicy) -> Vec<HeaderRecord> {
    fields
        .iter()
        .map(|field| HeaderRecord {
            value: if b_ids_schema::http::is_never_recorded(&field.name) {
                None
            } else {
                match policy {
                    ValuePolicy::NamesOnly => None,
                    ValuePolicy::WithValues => field.value.clone(),
                }
            },
            ..field.clone()
        })
        .collect()
}

fn other_frame(frame_type: u8, payload: &[u8]) -> Frame {
    Frame::Other {
        frame_type,
        payload_hex: hex(payload),
    }
}

fn settings_entries(payload: &[u8], notes: &mut Vec<Note>) -> Vec<SettingEntry> {
    let mut cursor = Cursor::new(payload);
    let mut entries = Vec::new();
    while cursor.remaining() >= 6 {
        let (Some(id), Some(value)) = (cursor.u16(), cursor.u32()) else {
            break;
        };
        entries.push(SettingEntry { id, value });
    }
    if cursor.remaining() != 0 {
        notes.push(Note::new(
            "http2.settings",
            format!(
                "{} byte(s) after the last entry, and an entry is six",
                cursor.remaining()
            ),
        ));
    }
    entries
}

/// The PRIORITY block inside a HEADERS frame, if the flags say one is there.
///
/// ⚠ **The pad length byte comes FIRST.** A reader that took the five bytes
/// straight after the frame head would be right on every unpadded frame and
/// silently wrong on a padded one, reporting a dependency built from the pad
/// length and four bytes of the real block.
fn headers_priority(payload: &[u8], flags: u8, notes: &mut Vec<Note>) -> Option<StreamPriority> {
    if flags & FLAG_PRIORITY == 0 {
        return None;
    }
    let mut cursor = Cursor::new(payload);
    if flags & FLAG_PADDED != 0 && cursor.u8().is_none() {
        notes.push(Note::new(
            "http2.stream_priority",
            "the padded flag is set and the payload is empty",
        ));
        return None;
    }
    let block = cursor.take(PRIORITY_BLOCK_LEN);
    match block.and_then(read_priority_block) {
        Some(priority) => Some(priority),
        None => {
            notes.push(Note::new(
                "http2.stream_priority",
                format!(
                    "the priority flag is set and {} payload byte(s) remain, which is fewer \
                     than the {PRIORITY_BLOCK_LEN} a block needs",
                    cursor.remaining()
                ),
            ));
            None
        }
    }
}

fn read_priority_block(bytes: &[u8]) -> Option<StreamPriority> {
    let mut cursor = Cursor::new(bytes);
    let dependency = cursor.u32()?;
    let weight_wire = cursor.u8()?;
    Some(StreamPriority {
        exclusive: dependency >> 31 == 1,
        stream_dependency: dependency & 0x7fff_ffff,
        // ⛔ AS ENCODED. HTTP/2 writes the weight minus one, so a client that
        // means 256 puts 255 here and they are one quantity in two units.
        weight_wire,
    })
}
