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
    /// It never reached HTTP/2.
    ///
    /// ⛔ **Renamed from `Abandoned` on 2026-09-02, and the rename is the
    /// entry.** The old name said the connection was useless; what is actually
    /// true of it is that it carried no HTTP/2, which is a fact about ONE HALF
    /// rather than a verdict on the connection. A connection classified this
    /// way can carry the only cold `ClientHello` of a navigation, and on
    /// `ubuntu-latest` that is exactly what happened: every connection that
    /// reached HTTP/2 had resumed, so discarding these published nothing.
    /// `docs/history/todo/harness.md`, `HARNESS-15`.
    NoHttp2,
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
        return Kind::NoHttp2;
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
    /// ⭐ **The connection the TLS half is taken from: the first whose hello
    /// offers no pre-shared key, whether or not it reached HTTP/2.**
    ///
    /// ⛔ **The `whether or not` is the whole entry.** Requiring one connection
    /// to carry both halves threw away a perfectly good cold hello whenever the
    /// connection carrying it was a preconnect, and on `ubuntu-latest` that was
    /// every cold hello there was. `docs/history/todo/harness.md`, `HARNESS-15`.
    pub tls_from: Option<&'a Capture>,
    /// ⭐ **The connection the HTTP/2 half is taken from: the first that
    /// reached HTTP/2, resumed or not.**
    ///
    /// ⚠ Resumption is a property of the TLS handshake and says nothing about
    /// the frames, so a resumed connection's HTTP/2 half is as good as a cold
    /// one's. What a profile owes the reader is which connection it came from,
    /// and [`Selection::one_connection`] is how that is said.
    pub http2_from: Option<&'a Capture>,
    /// ⭐ The first connection that reached HTTP/2 without resuming.
    ///
    /// ⚠ **Kept for the report and no longer the selection.** It is the
    /// connection that carries BOTH halves where one exists, which is the
    /// ordinary case and the one a reader should not have to think about.
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
    /// Every connection that reached no HTTP/2, in order.
    ///
    /// ⚠ **Renamed from `abandoned` with [`Kind::NoHttp2`].** One of these can
    /// be the connection [`Selection::tls_from`] points at.
    pub no_http2: Vec<&'a Capture>,
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
        self.no_http2.len() + self.resumed.len() + self.additional_cold.len() + self.cold_count()
    }

    /// Whether both halves came from one connection.
    ///
    /// ⛔ **A profile says this, because two halves from two sockets of one
    /// navigation is a condition of the measurement rather than a detail.** A
    /// reader who cannot tell cannot reason about anything that spans the two,
    /// and the ordinary case and the interesting case look identical without
    /// it. `docs/history/todo/harness.md`, `HARNESS-15`.
    #[must_use]
    pub fn one_connection(&self) -> Option<bool> {
        match (self.tls_from, self.http2_from) {
            (Some(tls), Some(http2)) => Some(tls.connection == http2.connection),
            _ => None,
        }
    }

    /// One line naming the connection each half was taken from.
    ///
    /// ⚠ Separate from [`Selection::report`] so a caller can print the counts
    /// without committing to a navigation that has both halves.
    #[must_use]
    pub fn halves(&self) -> String {
        let name =
            |c: Option<&Capture>| c.map_or_else(|| "none".to_owned(), |c| c.connection.to_string());
        format!(
            "tls from connection {}, http2 from connection {}",
            name(self.tls_from),
            name(self.http2_from)
        )
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
            "{} connection(s): {} cold, {} resumed, {} further cold, {} with no http2",
            self.connections(),
            self.cold_count(),
            self.resumed.len(),
            self.additional_cold.len(),
            self.no_http2.len()
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
        tls_from: None,
        http2_from: None,
        cold: None,
        resumed: Vec::new(),
        additional_cold: Vec::new(),
        no_http2: Vec::new(),
        resumption_observable: captures.iter().any(resumption_observable),
    };
    for capture in captures {
        // ⭐ THE TWO HALVES ARE SELECTED INDEPENDENTLY, and neither asks
        // anything about the other. This is the whole of HARNESS-15: the rule
        // it replaced required ONE connection to carry both, and a navigation
        // in which no connection did published nothing at all even when the
        // cold hello and the frames were both sitting in the capture file.
        //
        // ⛔ A COLD HELLO IS A HELLO THAT OFFERS NO PRE-SHARED KEY. Whether its
        // connection went on to speak HTTP/2 is a fact about the other half.
        if selection.tls_from.is_none()
            && capture
                .tls
                .as_ref()
                .is_some_and(|tls| !offers_pre_shared_key(tls))
        {
            selection.tls_from = Some(capture);
        }
        // ⚠ RESUMED OR NOT. Resumption is a property of the handshake and says
        // nothing about the frames, so the first connection to reach HTTP/2 is
        // the one to read them from.
        if selection.http2_from.is_none() && capture.reached_h2() {
            selection.http2_from = Some(capture);
        }

        match kind(capture) {
            Kind::NoHttp2 => selection.no_http2.push(capture),
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
