//! What the TCP layer of an accepted connection shows, and what it does not.
//!
//! ⭐ **A whole second fingerprint sits below TLS and every project examined
//! throws it away.** The source port, the maximum segment size, the window size
//! and scale, the time to live and the order of the TCP options are
//! operating-system-level signal, and they cost almost nothing when the
//! listener already belongs to this project. `TODO/harness.md`, `HARNESS-11`.
//!
//! ⛔ **FOUR OF THE FIVE ARE ABSENT ON EVERY HOST THIS WORKSPACE CAN BUILD
//! FOR, and that is a measurement rather than a limitation of this module.**
//! The entry asked for the capability to be established first, because it
//! decides whether this is a lane in the capture matrix or a local-only extra.
//! It was, and the answer is below.
//!
//! ⚠ **Measured 2026-09-04 on one Windows 11 host**, with a real loopback
//! connection, by setting a distinctive hop limit on the client and reading the
//! accepted socket on the server:
//!
//! ```text
//! peer_addr                127.0.0.1:51268
//!   source port            51268  <- available
//! server socket ttl()      Ok(128)  <- the SERVER's own outgoing hop limit
//! client set its ttl to    37
//!   so ttl() reads the peer's TTL: false
//! ```
//!
//! ⛔ **`TcpStream::ttl` is the LOCAL outgoing hop limit and not the peer's.**
//! That is the trap this module exists to stop: it is named `ttl`, it returns a
//! plausible number, and recording it as the peer's would put this host's own
//! configuration into a profile as though it were the browser's.
//!
//! ⭐ **What would change the answer**, and none of it is free:
//!
//! | route | what it costs |
//! | --- | --- |
//! | a raw socket, read directly | `unsafe_code = "deny"` at the workspace root, so it needs a dependency that wraps the syscalls |
//! | a packet capture beside the listener | a capture library, elevated privileges, and a second artefact to correlate |
//! | a platform socket option, `TCP_INFO` on Linux | not portable, and it carries the negotiated window rather than the peer's advertised options |
//!
//! ⛔ **So this is a local-only extra rather than a matrix lane**, until one of
//! those three is ruled on. `HARNESS-11` records the finding and the entry that
//! would take it further.

use std::net::TcpStream;

use serde::Serialize;

/// Why a TCP-layer field could not be recorded.
///
/// ⛔ **An unavailable field is ABSENT WITH A REASON, never zero.** A zero
/// window size is a real value a real stack can send, so a model that used it
/// for "not measured" would publish a measurement nobody took. `TODO/RULES.md`
/// rule 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Absent {
    /// The field.
    pub field: String,
    /// Why this host could not read it.
    pub why: String,
}

/// The TCP layer of one accepted connection.
///
/// ⚠ **Every field is an `Option` and the absences are listed beside them.** A
/// reader that only looked at the options would know a value was missing and
/// not why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TcpObservation {
    /// The port the peer connected from.
    ///
    /// ⭐ **The one field safe std gives.** Ephemeral port ranges differ
    /// between operating systems, so it is real signal on its own, and it is
    /// the weakest of the five.
    pub source_port: Option<u16>,
    /// The maximum segment size the peer advertised.
    pub maximum_segment_size: Option<u16>,
    /// The window size the peer advertised, before scaling.
    pub window_size: Option<u16>,
    /// The window scale shift the peer advertised.
    pub window_scale: Option<u8>,
    /// The time to live on the packets the peer sent.
    ///
    /// ⛔ **Not `TcpStream::ttl`, which is this host's own.** See the module
    /// header for the measurement.
    pub time_to_live: Option<u8>,
    /// The TCP option kinds the peer sent, in order.
    pub option_order: Option<Vec<u8>>,
    /// Every field this host could not read, with the reason.
    pub absent: Vec<Absent>,
}

impl TcpObservation {
    /// How many of the six fields carry a value.
    #[must_use]
    pub fn observed(&self) -> usize {
        usize::from(self.source_port.is_some())
            + usize::from(self.maximum_segment_size.is_some())
            + usize::from(self.window_size.is_some())
            + usize::from(self.window_scale.is_some())
            + usize::from(self.time_to_live.is_some())
            + usize::from(self.option_order.is_some())
    }

    /// Whether every absence carries a reason.
    ///
    /// ⛔ **The invariant this type exists for.** A field that is `None` with no
    /// entry in [`Self::absent`] is a value nobody explained, which reads as
    /// "not applicable" and means "nobody looked".
    #[must_use]
    pub fn every_absence_explained(&self) -> bool {
        let named: Vec<&str> = self.absent.iter().map(|a| a.field.as_str()).collect();
        let missing = [
            ("source_port", self.source_port.is_none()),
            ("maximum_segment_size", self.maximum_segment_size.is_none()),
            ("window_size", self.window_size.is_none()),
            ("window_scale", self.window_scale.is_none()),
            ("time_to_live", self.time_to_live.is_none()),
            ("option_order", self.option_order.is_none()),
        ];
        missing
            .iter()
            .filter(|(_, absent)| *absent)
            .all(|(field, _)| named.contains(field))
    }
}

/// The reason each field the platform does not expose is absent.
///
/// ⛔ **One reason per field, written once.** A reason composed at each call
/// site is a reason that drifts between two of them.
const WHY_NO_RAW_SOCKET: &str = "safe std exposes no TCP option data on an accepted connection, and reading it needs a raw \
     socket or a packet capture. The workspace denies unsafe_code, so it needs a dependency \
     this project has not taken. TODO/harness.md, HARNESS-11";

const WHY_TTL_IS_LOCAL: &str = "TcpStream::ttl is this host's own outgoing hop limit rather than the peer's. Measured: a \
     client that set 37 was read as 128 on the server, which is the server's default. Reading \
     the peer's needs a raw socket or a packet capture. TODO/harness.md, HARNESS-11";

/// Observe the TCP layer of an accepted connection.
///
/// ⛔ **It records what this platform gives and names what it does not.**
/// Nothing here infers, defaults or fabricates: the entry's own rule is that an
/// unavailable field is absent with a reason.
#[must_use]
pub fn observe(stream: &TcpStream) -> TcpObservation {
    let source_port = stream.peer_addr().ok().map(|a| a.port());

    let mut absent = Vec::new();
    if source_port.is_none() {
        absent.push(Absent {
            field: "source_port".to_owned(),
            why: "the accepted socket reported no peer address".to_owned(),
        });
    }
    for field in [
        "maximum_segment_size",
        "window_size",
        "window_scale",
        "option_order",
    ] {
        absent.push(Absent {
            field: field.to_owned(),
            why: WHY_NO_RAW_SOCKET.to_owned(),
        });
    }
    absent.push(Absent {
        field: "time_to_live".to_owned(),
        why: WHY_TTL_IS_LOCAL.to_owned(),
    });

    TcpObservation {
        source_port,
        maximum_segment_size: None,
        window_size: None,
        window_scale: None,
        time_to_live: None,
        option_order: None,
        absent,
    }
}

/// What this host can read of the TCP layer, as one line.
///
/// ⭐ **The capability answer, printable.** The entry asked for it to be
/// established and recorded, so it is something a session can run rather than
/// a paragraph somebody has to trust.
#[must_use]
pub fn capability() -> String {
    "tcp layer: 1 of 6 field(s) readable from safe std on an accepted connection \
         (source_port). The other 5 need a raw socket or a packet capture, which the \
         workspace's unsafe_code=deny makes a dependency question rather than a code one."
        .to_string()
}
