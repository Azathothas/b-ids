//! Did measuring change what was measured?
//!
//! ⭐ **An instrument that has to relax something in order to see anything may
//! have changed what it is watching.** This project has two capture surfaces
//! over the same wire: one that reads the `ClientHello` and closes without ever
//! completing a handshake, and one that completes it so a browser's HTTP/2
//! becomes reachable. The second is the only one that can see the HTTP/2 half
//! at all, so if it also changes the TLS half, everything published from it is
//! a reading of the harness rather than of the browser.
//!
//! # ⛔ The trap this module exists to avoid, which is the whole difficulty
//!
//! **A browser draws GREASE per connection and shuffles its extension order per
//! connection.** So two captures of one build differ in several fields with no
//! mode change involved at all, and a naive diff between two runs reports those
//! draws as findings. A comparison that cannot tell a draw from a mode effect
//! is worse than no comparison, because it produces a list of differences that
//! reads like evidence.
//!
//! ⭐ **So stability is MEASURED inside each run before anything is compared
//! across runs.** A field that is stable across every connection of both runs
//! and differs between them is a mode effect. A field that varies inside either
//! run is reported as not comparable on this sample, by name, rather than being
//! dropped or counted as agreement.
//!
//! ⚠ **This compares the TLS half only, and that is not a shortcut.** It is the
//! only half both surfaces can produce: the raw surface never completes a
//! handshake, so it never sees an HTTP/2 frame. That asymmetry is the reason
//! the terminating surface exists and it is stated by
//! [`Comparison::only_terminated_sees`] rather than left for a reader to infer.
//!
//! # ⛔ The caller selects, and the first driven run is why
//!
//! [`compare`] takes two slices and compares exactly what it is given. ⚠ **It
//! must be given connections of one KIND.** The first run of
//! `experiments/20-compare-capture-modes.sh` handed it every connection of each
//! run and reported the extension SET as not comparable, because the
//! terminating run had two distinct sets in it. That was not a mode effect on a
//! hello: it was resumption. A completed handshake leaves a session to resume,
//! so later connections of the terminating run offer a pre-shared key and a
//! different extension set, and the raw surface completes nothing so it can
//! never produce one.
//!
//! ⭐ **So the caller runs [`crate::select`] on each side and compares the cold
//! connections.** That is the same rule the corpus already follows: a resumed
//! connection is its own set with its own label and is never averaged with the
//! cold one. [`resumption_split`] is what reports the counts beside the
//! comparison, because "only one of these surfaces can produce a resumption at
//! all" is a finding rather than noise.
//!
//! `TODO/harness.md`, `HARNESS-10`.

use std::collections::BTreeSet;

use b_ids_schema::tls::{TlsHalf, is_grease_value};

use crate::listener::Capture;

/// How one named field behaved across the connections of a single run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stability {
    /// Every connection of the run agreed on it.
    Stable(String),
    /// The connections disagreed, so it is drawn or shuffled per connection.
    ///
    /// ⛔ Not a defect. It is what a browser does, and recording how many
    /// distinct values were seen is what makes the fact usable.
    Varies {
        /// How many distinct values the run produced.
        distinct: usize,
    },
    /// No connection of the run carried a readable `ClientHello`.
    Absent,
}

/// What the comparison concluded about one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Stable in both runs and equal. ⭐ The mode did not change it.
    Agrees(String),
    /// Stable in both runs and different. ⛔ The mode changed it.
    Differs {
        /// What the raw surface saw.
        raw: String,
        /// What the terminating surface saw.
        terminated: String,
    },
    /// It varies inside at least one run, so these two runs cannot be compared
    /// on it.
    ///
    /// ⚠ **A statement about the sample, not about the subject.** More
    /// connections do not fix it: a field drawn per connection has no single
    /// value for a mode to have changed.
    NotComparable {
        /// How the field behaved in the raw run.
        raw: Stability,
        /// How the field behaved in the terminating run.
        terminated: Stability,
    },
}

/// One field's name and what the comparison concluded about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldComparison {
    /// The field's path in a profile.
    pub field: String,
    /// What the comparison concluded.
    pub verdict: Verdict,
}

/// What two runs of the same browser, in two capture modes, showed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// How many connections of the raw run carried a readable hello.
    pub raw_hellos: usize,
    /// How many connections of the terminating run carried one.
    pub terminated_hellos: usize,
    /// Every field, in a fixed order.
    pub fields: Vec<FieldComparison>,
    /// What only the terminating surface can see at all.
    ///
    /// ⭐ Stated rather than inferred: the raw surface closes before a
    /// handshake completes, so no cleartext ever crosses it.
    pub only_terminated_sees: Vec<String>,
}

impl Comparison {
    /// Every field the mode changed.
    ///
    /// ⭐ **This is the answer the entry asks for.** An empty result means the
    /// terminating surface did not perturb any field both surfaces could see.
    #[must_use]
    pub fn differing(&self) -> Vec<&FieldComparison> {
        self.fields
            .iter()
            .filter(|f| matches!(f.verdict, Verdict::Differs { .. }))
            .collect()
    }

    /// Every field these two runs could not be compared on.
    ///
    /// ⚠ **Reported, never dropped.** A comparison that silently skipped the
    /// per-connection draws would report "no differences" over a set it never
    /// looked at.
    #[must_use]
    pub fn not_comparable(&self) -> Vec<&FieldComparison> {
        self.fields
            .iter()
            .filter(|f| matches!(f.verdict, Verdict::NotComparable { .. }))
            .collect()
    }

    /// Whether the comparison rests on enough connections to mean anything.
    ///
    /// ⛔ **One connection per mode cannot establish stability at all**, so a
    /// comparison over one is a comparison that calls every draw stable. The
    /// caller is told rather than the result being quietly weaker than it
    /// looks.
    #[must_use]
    pub fn thin(&self) -> bool {
        self.raw_hellos < 2 || self.terminated_hellos < 2
    }
}

/// Render one field of a TLS half as a comparable string.
///
/// ⚠ **Strings rather than typed values, deliberately.** The comparison is over
/// a heterogeneous field list, and the alternative is an enum with one variant
/// per field type that nothing else would use. What matters is that the same
/// function renders both sides.
fn render(field: &str, tls: &TlsHalf) -> Option<String> {
    let list = |v: &[u16]| {
        v.iter()
            .map(|x| format!("{x:#06x}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let no_grease = |v: &[u16]| {
        list(
            &v.iter()
                .copied()
                .filter(|x| !is_grease_value(*x))
                .collect::<Vec<_>>(),
        )
    };
    Some(match field {
        "tls.record_version" => format!("{:#06x}", tls.record_version),
        "tls.legacy_version" => format!("{:#06x}", tls.legacy_version),
        "tls.session_id_len" => tls.session_id_len.to_string(),
        "tls.cipher_suites" => list(&tls.cipher_suites),
        // ⭐ The same list with GREASE removed. The raw list carries a value
        // drawn per connection, so it can never be stable; this one can, and it
        // is what lets the comparison say anything about the cipher list.
        "tls.cipher_suites.no_grease" => no_grease(&tls.cipher_suites),
        "tls.compression_methods" => tls
            .compression_methods
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        "tls.extensions.order" => list(
            &tls.extensions
                .iter()
                .map(|e| e.codepoint)
                .collect::<Vec<_>>(),
        ),
        // ⚠ The SET rather than the order. A browser that shuffles has no
        // stable order, and which extensions it offers is a different question
        // from what order it offers them in.
        "tls.extensions.set.no_grease" => {
            let set: BTreeSet<u16> = tls
                .extensions
                .iter()
                .map(|e| e.codepoint)
                .filter(|c| !is_grease_value(*c))
                .collect();
            list(&set.into_iter().collect::<Vec<_>>())
        }
        "tls.extensions.count" => tls.extensions.len().to_string(),
        "tls.key_exchange_groups.no_grease" => no_grease(&tls.key_exchange_groups),
        "tls.key_shares.groups.no_grease" => {
            no_grease(&tls.key_shares.iter().map(|k| k.group).collect::<Vec<_>>())
        }
        // ⚠ The LENGTHS, not the key material. A key share's bytes are new every
        // connection and its length is a property of the group.
        "tls.key_shares.lengths" => tls
            .key_shares
            .iter()
            .filter(|k| !is_grease_value(k.group))
            .map(|k| k.entry_len.to_string())
            .collect::<Vec<_>>()
            .join(","),
        "tls.signature_algorithms" => list(&tls.signature_algorithms),
        "tls.signature_algorithms_cert" => tls
            .signature_algorithms_cert
            .as_ref()
            .map_or_else(|| "absent".to_owned(), |v| list(v)),
        "tls.alpn" => tls.alpn.join(","),
        "tls.ech" => tls.ech.as_ref().map_or_else(
            || "absent".to_owned(),
            |e| format!("{}/{:#06x}", e.mode, e.kem_id),
        ),
        "tls.padding_len" => tls
            .padding_len
            .map_or_else(|| "absent".to_owned(), |p| p.to_string()),
        "tls.grease.count" => tls.grease.values.len().to_string(),
        "tls.grease.positions" => tls
            .grease
            .extension_positions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        _ => return None,
    })
}

/// Every field this comparison covers, in the order it reports them.
///
/// ⛔ **A fixed list rather than a walk of the struct.** A field added to the
/// model and not added here is a field nobody compares, which is a real gap;
/// discovering it by reading this list is the point of the list being here.
pub const FIELDS: &[&str] = &[
    "tls.record_version",
    "tls.legacy_version",
    "tls.session_id_len",
    "tls.cipher_suites",
    "tls.cipher_suites.no_grease",
    "tls.compression_methods",
    "tls.extensions.order",
    "tls.extensions.set.no_grease",
    "tls.extensions.count",
    "tls.key_exchange_groups.no_grease",
    "tls.key_shares.groups.no_grease",
    "tls.key_shares.lengths",
    "tls.signature_algorithms",
    "tls.signature_algorithms_cert",
    "tls.alpn",
    "tls.ech",
    "tls.padding_len",
    "tls.grease.count",
    "tls.grease.positions",
];

/// How one field behaved across a run's connections.
fn stability(field: &str, captures: &[Capture]) -> Stability {
    let values: BTreeSet<String> = captures
        .iter()
        .filter_map(|c| c.tls.as_ref())
        .filter_map(|tls| render(field, tls))
        .collect();
    match values.len() {
        0 => Stability::Absent,
        1 => Stability::Stable(values.into_iter().next().unwrap_or_default()),
        distinct => Stability::Varies { distinct },
    }
}

/// What one run's connections were, split by kind.
///
/// ⭐ **Reported beside a comparison rather than folded into it.** Only a
/// surface that completes a handshake leaves a session to resume, so a
/// difference in these counts is a mode effect on the RUN even when every field
/// of the cold hello agrees. A comparison that did not report them would answer
/// a narrower question than a reader assumes it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    /// Connections that reached HTTP/2 without resuming.
    pub cold: usize,
    /// Connections that reached HTTP/2 and offered a pre-shared key.
    pub resumed: usize,
    /// Connections that never reached HTTP/2.
    ///
    /// ⚠ On the raw surface this is every connection, by construction: nothing
    /// can reach HTTP/2 through a handshake that never completes.
    pub abandoned: usize,
}

/// Split connections by kind.
///
/// ⛔ **Counted PER CAPTURE, never through [`crate::select`].** That function
/// answers a question about one navigation: which connection is the cold one to
/// build a profile from. A caller here concatenates several runs, so connection
/// numbers repeat and "the first cold one" means nothing across the join.
/// [`crate::select::kind`] classifies one capture on its own and has no such
/// assumption, which is why it is what this reads.
#[must_use]
pub fn resumption_split(captures: &[Capture]) -> Split {
    let mut split = Split {
        cold: 0,
        resumed: 0,
        abandoned: 0,
    };
    for capture in captures {
        match crate::select::kind(capture) {
            crate::Kind::NoHttp2 => split.abandoned += 1,
            crate::Kind::Cold => split.cold += 1,
            crate::Kind::Resumed => split.resumed += 1,
        }
    }
    split
}

/// The connections that carried a hello and did not offer a pre-shared key.
///
/// ⛔ **What [`compare`] should be given**, and it asks each capture directly
/// rather than routing through a navigation-shaped selection. A run that
/// concatenates several launches has repeating connection numbers, so excluding
/// the resumed ones by number would drop a cold connection from one launch
/// because a resumed one in another launch shared its number.
///
/// ⚠ **The raw surface reaches HTTP/2 on nothing**, so its hellos are kept here
/// on the strength of not offering a pre-shared key alone. That asymmetry is the
/// point: a surface that completes no handshake cannot produce a resumption, so
/// every hello it sees is a cold one whatever its HTTP/2 says.
#[must_use]
pub fn comparable(captures: &[Capture]) -> Vec<&Capture> {
    captures
        .iter()
        .filter(|c| {
            c.tls
                .as_ref()
                .is_some_and(|tls| !crate::select::offers_pre_shared_key(tls))
        })
        .collect()
}

/// Compare a raw-surface run against a terminating-surface run.
///
/// ⛔ **Both runs must be of the same browser and build**, and nothing here can
/// check that: a comparison across two builds would report a version bump as a
/// mode effect. The caller establishes it, and
/// `experiments/20-compare-capture-modes.sh` does so by driving one resolved
/// browser twice in one run.
#[must_use]
pub fn compare(raw: &[Capture], terminated: &[Capture]) -> Comparison {
    let count = |cs: &[Capture]| cs.iter().filter(|c| c.tls.is_some()).count();
    let fields = FIELDS
        .iter()
        .map(|field| {
            let r = stability(field, raw);
            let t = stability(field, terminated);
            let verdict = match (&r, &t) {
                (Stability::Stable(a), Stability::Stable(b)) if a == b => {
                    Verdict::Agrees(a.clone())
                }
                (Stability::Stable(a), Stability::Stable(b)) => Verdict::Differs {
                    raw: a.clone(),
                    terminated: b.clone(),
                },
                _ => Verdict::NotComparable {
                    raw: r.clone(),
                    terminated: t.clone(),
                },
            };
            FieldComparison {
                field: (*field).to_owned(),
                verdict,
            }
        })
        .collect();

    Comparison {
        raw_hellos: count(raw),
        terminated_hellos: count(terminated),
        fields,
        only_terminated_sees: vec![
            "http2".to_owned(),
            "http".to_owned(),
            "raw.http2_frames_hex".to_owned(),
            "raw.connection_hex".to_owned(),
        ],
    }
}
