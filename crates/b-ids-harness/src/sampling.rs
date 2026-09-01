//! One handshake is not a sample.
//!
//! ⛔ **Anything drawn per connection means a single handshake tests a single
//! draw.** A defect that fires on three values in sixteen reaches a
//! one-handshake check four times in five, and passes. Measured elsewhere: one
//! CI run failed on it, the next passed over the same defect, and 64
//! handshakes completed 64 after the fix. `docs/inherited-claims.md` section 8.
//!
//! ⛔ **A run where six of eight completed is a run that reports six**, not a
//! run that reports success. That is the whole of this module: a count beside
//! the count that was asked for, and a non-zero exit when they differ.

use crate::listener::{Capture, Protocol};

/// How many connections a run accepts unless told otherwise.
///
/// ⛔ **Eight, and never one.** The number is inherited with its reason:
/// eleven captures of one binary produced eight offering a session ticket and
/// three offering a pre-shared key, so a one-draw check sees one of two
/// behaviours and calls it the behaviour.
pub const DEFAULT_HANDSHAKES: u32 = 8;

/// Whether one connection produced a usable reading of its surface.
///
/// ⚠ **Accepted is not completed.** A browser opens sockets it abandons, and
/// those are accepted, recorded, and useless as a draw.
#[must_use]
pub fn completed(capture: &Capture) -> bool {
    match capture.protocol {
        Protocol::TlsRaw => capture.tls.is_some(),
        Protocol::Cleartext => capture.http2.is_some() || capture.request_line.is_some(),
    }
}

/// What a run of several handshakes actually drew.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sampling {
    /// How many connections the run was configured to accept.
    pub requested: u32,
    /// How many it accepted.
    ///
    /// ⚠ Fewer than requested where the run ran out of time. That is a fact
    /// about the run rather than about the subject, and it is reported as one.
    pub accepted: usize,
    /// How many produced a usable reading.
    pub completed: usize,
    /// The GREASE values each completed connection drew, in order.
    ///
    /// ⭐ This is the per-draw variation. Two consecutive captures of one
    /// binary must produce two different draws, or the capture is wrong.
    pub grease_draws: Vec<Vec<u16>>,
    /// How many distinct GREASE draws were seen.
    pub distinct_grease_draws: usize,
    /// How many distinct extension orders were seen.
    ///
    /// ⚠ One distinct order across eight handshakes is not proof of a fixed
    /// order; it is what a shuffle whose input list is exhaustive also
    /// produces. `SCHEMA-10` is where the property is recorded.
    pub distinct_extension_orders: usize,
}

impl Sampling {
    /// Whether every requested handshake completed.
    #[must_use]
    pub fn every_one_completed(&self) -> bool {
        self.completed == self.requested as usize
    }

    /// The sentence a run prints when it did not get what it asked for.
    ///
    /// ⛔ It names BOTH numbers. "Some handshakes failed" is a sentence nobody
    /// can act on.
    #[must_use]
    pub fn shortfall(&self) -> Option<String> {
        if self.every_one_completed() {
            return None;
        }
        Some(format!(
            "{} of {} handshake(s) completed, from {} accepted connection(s)",
            self.completed, self.requested, self.accepted
        ))
    }
}

/// Summarise what a run drew.
#[must_use]
pub fn summarise(requested: u32, captures: &[Capture]) -> Sampling {
    let usable: Vec<&Capture> = captures.iter().filter(|c| completed(c)).collect();

    let grease_draws: Vec<Vec<u16>> = usable
        .iter()
        .filter_map(|c| c.tls.as_ref())
        .map(|tls| tls.grease.values.clone())
        .collect();
    let orders: Vec<Vec<u16>> = usable
        .iter()
        .filter_map(|c| c.tls.as_ref())
        .map(|tls| tls.extensions.iter().map(|e| e.codepoint).collect())
        .collect();

    Sampling {
        requested,
        accepted: captures.len(),
        completed: usable.len(),
        distinct_grease_draws: distinct(&grease_draws),
        distinct_extension_orders: distinct(&orders),
        grease_draws,
    }
}

fn distinct(rows: &[Vec<u16>]) -> usize {
    let mut seen: Vec<&Vec<u16>> = Vec::new();
    for row in rows {
        if !seen.contains(&row) {
            seen.push(row);
        }
    }
    seen.len()
}
