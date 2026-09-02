//! Which connection of a navigation is the one to keep.
//!
//! ⛔ **One navigation is not one connection, and neither the first nor the last
//! is the one to keep.** Driving one browser at a probe produced thirteen
//! connections: the first carried no HTTP/2 at all, a preconnect the browser
//! abandoned, and every one after the second offered a pre-shared key instead
//! of a session ticket, because the session resumed.
//! `docs/inherited-claims.md` section 8.
//!
//! ⭐ **Keep the first connection that reached HTTP/2.** That is the cold
//! handshake and it is what a fresh client sends.
//!
//! ⛔ **A resumed connection is recorded SEPARATELY, never averaged with the
//! cold one and never deduplicated against it.** Two connections that differ
//! are the data: they produce different digests, and a corpus that folded them
//! together would publish a handshake neither of them sent.

use b_ids_schema::tls::TlsHalf;

use crate::listener::Capture;

/// The `session_ticket` extension, which a client offers on a cold handshake.
pub const SESSION_TICKET: u16 = 0x0023;

/// The `pre_shared_key` extension, which a client offers instead when the
/// session resumed.
pub const PRE_SHARED_KEY: u16 = 0x0029;

/// What one connection of a navigation turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// It never reached HTTP/2. ⚠ A browser opens sockets it abandons, and a
    /// run that dropped them would under-report what a navigation does.
    Abandoned,
    /// It reached HTTP/2 and nothing says the session resumed.
    Cold,
    /// It reached HTTP/2 and offered a pre-shared key.
    Resumed,
}

/// Whether resumption is observable on this capture at all.
///
/// ⛔ **A capture with no TLS half cannot be asked whether it resumed**, and
/// saying `Cold` about one is a claim the bytes do not support. A cleartext
/// capture is exactly that case. An unavailable field is absent with a reason,
/// never assumed.
#[must_use]
pub fn resumption_observable(capture: &Capture) -> bool {
    capture.tls.is_some()
}

/// Whether a hello offers a pre-shared key, which means the session resumed.
#[must_use]
pub fn offers_pre_shared_key(tls: &TlsHalf) -> bool {
    tls.extensions.iter().any(|e| e.codepoint == PRE_SHARED_KEY)
}

/// Classify one connection.
#[must_use]
pub fn kind(capture: &Capture) -> Kind {
    if !capture.reached_h2() {
        return Kind::Abandoned;
    }
    match &capture.tls {
        Some(tls) if offers_pre_shared_key(tls) => Kind::Resumed,
        _ => Kind::Cold,
    }
}

/// What a navigation's connections were, split by kind.
///
/// ⚠ Borrows rather than clones. The captures are the record and there is one
/// copy of each; a selection that copied them would be a second place for a
/// capture to live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<'a> {
    /// ⭐ The first connection that reached HTTP/2, which is the cold
    /// handshake.
    pub cold: Option<&'a Capture>,
    /// Every connection that reached HTTP/2 and resumed, in order.
    ///
    /// ⛔ Its own set with its own label. It becomes its own profile with its
    /// own provenance and it is never averaged with the cold one.
    pub resumed: Vec<&'a Capture>,
    /// Every LATER connection that reached HTTP/2 without resuming, in order.
    ///
    /// ⚠ Its own set rather than being folded into either of the others. A
    /// second cold handshake is not a resumed one and calling it that would be
    /// a label the bytes contradict; it is also not the one the profile is
    /// built from, which is why it is not `cold`.
    pub additional_cold: Vec<&'a Capture>,
    /// Every connection that never reached HTTP/2, in order.
    pub abandoned: Vec<&'a Capture>,
    /// ⚠ Whether the cold-against-resumed split means anything on this run.
    ///
    /// False where no connection carried a `ClientHello`, which is every
    /// cleartext capture. The split is then a statement about HTTP/2 alone and
    /// says nothing about resumption.
    pub resumption_observable: bool,
}

impl Selection<'_> {
    /// How many connections the navigation opened.
    #[must_use]
    pub fn connections(&self) -> usize {
        self.abandoned.len() + self.resumed.len() + self.additional_cold.len() + self.cold_count()
    }

    /// How many cold connections there are, which is zero or one.
    ///
    /// ⛔ **Read from the field rather than assumed.** A navigation in which
    /// every connection that reached HTTP/2 resumed has NO cold connection, and
    /// a report that said otherwise would be a number nobody measured.
    #[must_use]
    pub fn cold_count(&self) -> usize {
        usize::from(self.cold.is_some())
    }

    /// One line saying what this navigation's connections were.
    ///
    /// ⛔ **It lives here rather than in a caller's format string, and that is
    /// the defect this method exists because of.** `b-ids-corpus add` printed
    /// the word `cold` behind a hardcoded `1`, so a navigation with no cold
    /// connection was reported as having one, on the line above the refusal
    /// saying it had none. A number in a format string is a number no test can
    /// reach; a method is one every test can.
    #[must_use]
    pub fn report(&self) -> String {
        format!(
            "{} connection(s): {} cold, {} resumed, {} further cold, {} abandoned",
            self.connections(),
            self.cold_count(),
            self.resumed.len(),
            self.additional_cold.len(),
            self.abandoned.len()
        )
    }
}

/// Split a navigation's connections into the cold one, the resumed ones and the
/// abandoned ones.
///
/// ⛔ **Nothing is deduplicated and nothing is averaged.** Two connections that
/// differ are the data.
#[must_use]
pub fn select(captures: &[Capture]) -> Selection<'_> {
    let mut selection = Selection {
        cold: None,
        resumed: Vec::new(),
        additional_cold: Vec::new(),
        abandoned: Vec::new(),
        resumption_observable: captures.iter().any(resumption_observable),
    };
    for capture in captures {
        match kind(capture) {
            Kind::Abandoned => selection.abandoned.push(capture),
            Kind::Resumed => selection.resumed.push(capture),
            // ⭐ The FIRST cold one. A later cold connection is kept in its
            // own set: it is not resumed, and it is not the one the profile is
            // built from.
            Kind::Cold => {
                if selection.cold.is_none() {
                    selection.cold = Some(capture);
                } else {
                    selection.additional_cold.push(capture);
                }
            }
        }
    }
    selection
}
