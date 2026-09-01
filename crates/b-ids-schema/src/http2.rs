//! The HTTP/2 half, as an ordered frame sequence.
//!
//! ⛔ **A frame list, not a settings map.** A map loses order, and order is part
//! of the fingerprint. A map also cannot say which settings were ABSENT, and
//! absence is load-bearing: one browser sends no `SETTINGS_MAX_FRAME_SIZE`
//! where a general-purpose stack sends the protocol default, and those are two
//! visibly different connections.
//!
//! ⛔ **An absent setting is never recorded as a default value.** That is the
//! one substitution this module exists to make impossible: it is representable
//! only by leaving the entry out of the list, and a list that omits an entry
//! and a list that carries it at the default compare unequal.
//!
//! ⚠ **Every field is named for the wire.** The connection window and the
//! WINDOW_UPDATE increment differ by the protocol's own 65,535 default, and the
//! stream weight on the wire is one less than the weight the specification
//! talks in. `docs/reference-sweeps/usable.md` section 2 has the audit: the
//! same field held both meanings in one shipped database, and seven of its
//! entries were 65,535 short.

use serde::{Deserialize, Serialize};

/// One SETTINGS entry, as an identifier and a value.
///
/// ⚠ The identifier is a number rather than an enum, for the same reason an
/// extension codepoint is: a setting this project has never seen is still a
/// setting a browser sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingEntry {
    /// The setting identifier.
    pub id: u16,
    /// The value it carried.
    pub value: u32,
}

/// A frame in the sequence a connection opened with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "frame")]
pub enum Frame {
    /// The SETTINGS frame, with its entries in the order they were sent.
    Settings {
        /// The entries, in wire order. ⛔ Absence is expressed by omission.
        entries: Vec<SettingEntry>,
    },
    /// The connection-level WINDOW_UPDATE.
    WindowUpdate {
        /// ⛔ The INCREMENT the frame carried, which is the window the client
        /// wants minus the protocol's own 65,535 default. Named for the wire
        /// because the wire is what a capture reads.
        window_size_increment: u32,
    },
    /// The HEADERS frame that opened the first request.
    Headers {
        /// The stream the frame was sent on.
        stream_id: u32,
        /// Whether the flags byte set `0x20`, so a PRIORITY block follows.
        has_priority_block: bool,
    },
    /// A frame this project has no name for, kept rather than dropped.
    ///
    /// ⛔ The same rule as an unknown TLS extension. A sequence that silently
    /// omits a frame is a sequence nobody can compare.
    Other {
        /// The frame type byte.
        frame_type: u8,
        /// The frame payload, hex-encoded.
        payload_hex: String,
    },
}

/// The PRIORITY block carried inside a HEADERS frame.
///
/// ⚠ Distinct from the standalone PRIORITY FRAME, which RFC 9113 deprecates and
/// which this model records separately as [`PriorityFrame`]. Both exist and
/// they are different seams.
///
/// ⛔ Recorded as the parsed five bytes, never only as a rendered Akamai
/// string. That string cannot distinguish "no block sent" from "block not
/// read", and two of the three sources that report a zero for this field were
/// reading a tool that could not write the block rather than a browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StreamPriority {
    /// The exclusive bit.
    pub exclusive: bool,
    /// The 31-bit stream dependency.
    pub stream_dependency: u32,
    /// ⛔ The weight AS ENCODED, which HTTP/2 defines as the weight minus one.
    /// A tool that takes `256` puts `255` here, and they are one quantity.
    pub weight_wire: u8,
}

/// ⛔ Read as a `u16` and then refused above 255, so the message can NAME THE
/// ENCODING.
///
/// ⚠ The type alone already makes 256 unrepresentable, and that is not enough:
/// the error a plain `u8` produces says "expected u8", which sends a reader
/// looking for a bounds bug rather than telling them they wrote the
/// specification's unit into the wire's field. That confusion is the whole of
/// `SCHEMA-09`.
impl<'de> Deserialize<'de> for StreamPriority {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        #[derive(Deserialize)]
        struct Raw {
            exclusive: bool,
            stream_dependency: u32,
            weight_wire: u16,
        }

        let raw = Raw::deserialize(deserializer)?;
        let weight_wire = u8::try_from(raw.weight_wire).map_err(|_| {
            D::Error::custom(format!(
                "http2.stream_priority.weight_wire is {}, and the wire encoding is weight minus \
                 one, so it holds 0 to 255. {} is the specification's unit and {} is the wire's",
                raw.weight_wire,
                raw.weight_wire,
                raw.weight_wire.saturating_sub(1)
            ))
        })?;
        Ok(Self {
            exclusive: raw.exclusive,
            stream_dependency: raw.stream_dependency,
            weight_wire,
        })
    }
}

impl StreamPriority {
    /// The weight in the units the specification talks in, which is one more
    /// than the wire carries.
    ///
    /// ⚠ Derived on request rather than stored. Storing both is how a field
    /// ends up holding whichever unit its last writer believed in.
    #[must_use]
    pub fn weight_spec(&self) -> u16 {
        u16::from(self.weight_wire) + 1
    }
}

/// A standalone PRIORITY frame sent before HEADERS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriorityFrame {
    /// The stream the frame is about.
    pub stream_id: u32,
    /// The priority it declared.
    pub priority: StreamPriority,
}

/// The HTTP/2 half of a profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Http2Half {
    /// The frames the connection opened with, in order.
    pub frames: Vec<Frame>,
    /// The PRIORITY block on the first HEADERS frame, or an explicit `null`.
    ///
    /// ⛔ `None` means no block was sent, and it is a measurement. A profile
    /// that cannot tell that from "not read" is why this is not a rendered
    /// string.
    pub stream_priority: Option<StreamPriority>,
    /// Standalone PRIORITY frames sent before HEADERS.
    pub priority_frames: Vec<PriorityFrame>,
    /// The pseudo-headers in the order they were sent.
    pub pseudo_header_order: Vec<String>,
    /// The connection window the client wants, in the HUMAN unit.
    ///
    /// ⭐ Carried as a SECOND, separately named field rather than instead of the
    /// increment, and [`Http2Half::check_units`] asserts the arithmetic between
    /// the two. ⛔ A comment cannot carry a unit: in one shipped database the
    /// comment was right and seven entries beside it were still wrong.
    ///
    /// ⚠ `None` where the capture recorded only what the wire carried, which is
    /// the ordinary case. The increment is the measurement; this is a
    /// convenience derived from it and checked against it.
    pub connection_window: Option<u32>,
}

/// The protocol's own default connection window, which is the difference
/// between the window a client wants and the increment it sends.
pub const PROTOCOL_DEFAULT_WINDOW: u32 = 65_535;

impl Http2Half {
    /// The SETTINGS entries, in order, if a SETTINGS frame was sent.
    #[must_use]
    pub fn settings(&self) -> Option<&[SettingEntry]> {
        self.frames.iter().find_map(|f| match f {
            Frame::Settings { entries } => Some(entries.as_slice()),
            _ => None,
        })
    }

    /// Whether a settings identifier was sent at all.
    ///
    /// ⭐ This is the question a map cannot answer once it has been filled in
    /// with defaults.
    #[must_use]
    pub fn sends_setting(&self, id: u16) -> bool {
        self.settings()
            .is_some_and(|entries| entries.iter().any(|e| e.id == id))
    }

    /// The connection WINDOW_UPDATE increment, if one was sent.
    #[must_use]
    pub fn window_size_increment(&self) -> Option<u32> {
        self.frames.iter().find_map(|f| match f {
            Frame::WindowUpdate {
                window_size_increment,
            } => Some(*window_size_increment),
            _ => None,
        })
    }

    /// Every place the two units of one quantity disagree.
    ///
    /// ⛔ **Checked, never commented.** `SCHEMA-09` exists because three
    /// quantities in this domain each have a human number and a wire number,
    /// and every reference project examined has confused at least one.
    #[must_use]
    pub fn check_units(&self) -> Vec<crate::Defect> {
        let mut defects = Vec::new();
        if let (Some(window), Some(increment)) =
            (self.connection_window, self.window_size_increment())
        {
            let gap = i64::from(window) - i64::from(increment);
            if gap != i64::from(PROTOCOL_DEFAULT_WINDOW) {
                defects.push(crate::Defect::FieldMalformed {
                    field: "http2.connection_window".to_owned(),
                    why: format!(
                        "is {window} and the WINDOW_UPDATE increment is {increment}, a difference \
                         of {gap}. They are one quantity in two units and must differ by exactly \
                         {PROTOCOL_DEFAULT_WINDOW}, the protocol's own default"
                    ),
                });
            }
        }
        defects
    }

    /// The rendered Akamai fingerprint: settings, increment, priority block and
    /// pseudo-header order, joined by `|`.
    ///
    /// ⛔ DERIVED, and derived here so that nothing has to store it. A digest is
    /// derived from a profile and a profile is never derived from a digest.
    ///
    /// ⚠ The rendering loses what the model keeps. An absent block renders as
    /// `0`, and `0` is also what a source that could not read a block reports
    /// and what a stack that cannot write one emits. Three published readings
    /// of this field disagree for exactly that reason, so the string is a
    /// rendering of the model and never a substitute for it.
    #[must_use]
    pub fn akamai_text(&self) -> String {
        let settings = self.settings().map_or_else(String::new, |entries| {
            entries
                .iter()
                .map(|e| format!("{}:{}", e.id, e.value))
                .collect::<Vec<_>>()
                .join(";")
        });
        let window = self
            .window_size_increment()
            .map_or_else(|| "0".to_owned(), |w| w.to_string());
        let priority = self.stream_priority.map_or_else(
            || "0".to_owned(),
            |p| {
                format!(
                    "1:{}:{}:{}",
                    u8::from(p.exclusive),
                    p.stream_dependency,
                    p.weight_wire
                )
            },
        );
        let pseudo = self
            .pseudo_header_order
            .iter()
            .filter_map(|h| h.strip_prefix(':').and_then(|s| s.chars().next()))
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!("{settings}|{window}|{priority}|{pseudo}")
    }
}
