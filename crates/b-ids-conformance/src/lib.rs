//! Asking an impersonating client what it claims, and comparing that to a
//! captured profile.
//!
//! ⭐ **A client author who wants to know how close their client is to a real
//! browser has to build a capture server and a comparison by hand.** Every
//! client examined in the reference sweep would use one if it existed.
//! `TODO/validator.md`, `VALID-05`.
//!
//! ⛔ **A FIELD-LEVEL DIFF, NEVER A DIGEST COMPARISON.** A digest says two
//! things differ without saying what, and that is the tool everybody already
//! has. This reports which extension moved, which setting is absent, and which
//! header changed position.
//!
//! ⭐ **One artefact, both directions.** The same report is a conformance
//! result for a client author and a detection reference for a server author,
//! and neither has one today.
//!
//! ⚠ **THE RENDERER IS THE HARNESS'S, NOT A SECOND ONE.** The TLS half is
//! rendered by [`b_ids_harness::modes::render`] over
//! [`b_ids_harness::modes::FIELDS`], which is the vocabulary the capture-mode
//! comparison already uses. A second renderer would be two spellings of one
//! value with nothing checking that they agree.

use b_ids_harness::modes::{FIELDS, render};
use b_ids_schema::Profile;
use b_ids_schema::http::Variant;
use b_ids_schema::http2::Frame;
use b_ids_schema::tls::is_grease_value;

/// What the comparison concluded about one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Both sides carry the field and they are equal.
    Conforms(String),
    /// Both sides carry it and they differ.
    ///
    /// ⛔ **Both values are kept.** A report saying only that a field differs
    /// sends the reader back to the two files to find out how.
    Differs {
        /// What the profile the client CLAIMS to be carries.
        claimed: String,
        /// What the client actually sent.
        observed: String,
    },
    /// Both sides carry it, and the only difference is which GREASE value was
    /// drawn.
    ///
    /// ⛔ **NOT A CONFORMANCE FAILURE, and reporting it as one is what makes a
    /// tool like this useless.** RFC 8701 GREASE is drawn per connection, so
    /// two captures of the SAME browser differ here every time. A report that
    /// called that a difference would name three or four fields on every run
    /// and a reader would stop reading it.
    ///
    /// ⚠ **Where the values SIT is a different question and it is still
    /// asked.** `tls.grease.positions` compares the indices, so a client that
    /// puts GREASE in the wrong place is still caught; only the drawn value is
    /// forgiven.
    PerConnection {
        /// What the profile the client claims to be carries.
        claimed: String,
        /// What the client actually sent.
        observed: String,
        /// Why a single capture cannot conclude from this field.
        why: String,
    },
    /// At least one side carries nothing for it.
    ///
    /// ⚠ **Not a pass and not a failure.** A field absent from the claim cannot
    /// be conformed to, and a field absent from the capture was not measured.
    /// Reporting it as agreement is how a client passes on a field nobody
    /// looked at.
    NotCheckable {
        /// Whether the claimed profile carries it.
        claimed: bool,
        /// Whether the observed capture carries it.
        observed: bool,
    },
}

/// One field and what the comparison concluded about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldReport {
    /// The field's path in a profile.
    pub field: String,
    /// What the comparison concluded.
    pub verdict: Verdict,
}

/// What one client's capture showed against the profile it claims to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The profile the client claims to be.
    pub claimed: String,
    /// The capture the client actually produced.
    pub observed: String,
    /// Every field, in a fixed order.
    pub fields: Vec<FieldReport>,
}

impl Report {
    /// Every field the client got wrong.
    ///
    /// ⭐ **This is the answer the entry asks for.** An empty result means every
    /// field both sides carry agreed.
    #[must_use]
    pub fn differing(&self) -> Vec<&FieldReport> {
        self.fields
            .iter()
            .filter(|f| matches!(f.verdict, Verdict::Differs { .. }))
            .collect()
    }

    /// Every field whose only difference is the GREASE value drawn.
    ///
    /// ⚠ **Reported rather than hidden.** It is not a failure, and a reader
    /// still wants to know the tool looked.
    #[must_use]
    pub fn per_connection(&self) -> Vec<&FieldReport> {
        self.fields
            .iter()
            .filter(|f| matches!(f.verdict, Verdict::PerConnection { .. }))
            .collect()
    }

    /// Every field neither side could be compared on.
    #[must_use]
    pub fn not_checkable(&self) -> Vec<&FieldReport> {
        self.fields
            .iter()
            .filter(|f| matches!(f.verdict, Verdict::NotCheckable { .. }))
            .collect()
    }

    /// How many fields agreed.
    #[must_use]
    pub fn conforming(&self) -> usize {
        self.fields
            .iter()
            .filter(|f| matches!(f.verdict, Verdict::Conforms(_)))
            .count()
    }
}

/// The HTTP and HTTP/2 fields, which the TLS vocabulary does not cover.
///
/// ⛔ **The entry names three things a report must be able to say**, and one of
/// them is about a header and one is about a setting. A comparison over
/// [`FIELDS`] alone would answer only the first.
pub const HTTP_FIELDS: &[&str] = &[
    "http.navigate.header_order",
    "http.navigate.header_count",
    "http2.settings.order",
    "http2.settings.values",
    "http2.window_update.increment",
    "http2.pseudo_header_order",
    "http2.stream_priority",
    "http2.priority_frames.count",
    "http2.frames.order",
];

/// Render one HTTP or HTTP/2 field as a comparable string.
///
/// ⚠ `None` means the profile carries nothing for that field, which is a
/// different fact from carrying an empty list.
#[must_use]
fn render_http(field: &str, profile: &Profile) -> Option<String> {
    let navigate = profile.http.variant(Variant::Navigate);
    let settings = profile.http2.frames.iter().find_map(|f| match f {
        Frame::Settings { entries } => Some(entries),
        _ => None,
    });
    match field {
        "http.navigate.header_order" => navigate.map(|set| {
            set.headers
                .iter()
                .map(|h| h.name.as_str())
                .collect::<Vec<_>>()
                .join(",")
        }),
        "http.navigate.header_count" => navigate.map(|set| set.headers.len().to_string()),
        // ⛔ THE ORDER AND THE VALUES ARE TWO FIELDS. A client that sends every
        // setting with the right value in the wrong order is wrong in a way a
        // combined field would report as one difference and a reader would
        // misread as two.
        "http2.settings.order" => settings.map(|e| {
            e.iter()
                .map(|s| format!("{:#06x}", s.id))
                .collect::<Vec<_>>()
                .join(",")
        }),
        "http2.settings.values" => settings.map(|e| {
            e.iter()
                .map(|s| format!("{:#06x}={}", s.id, s.value))
                .collect::<Vec<_>>()
                .join(",")
        }),
        "http2.window_update.increment" => profile.http2.frames.iter().find_map(|f| match f {
            Frame::WindowUpdate {
                window_size_increment,
            } => Some(window_size_increment.to_string()),
            _ => None,
        }),
        "http2.pseudo_header_order" => {
            if profile.http2.pseudo_header_order.is_empty() {
                None
            } else {
                Some(profile.http2.pseudo_header_order.join(","))
            }
        }
        // ⚠ `None` here means no block was sent, which the model records as a
        // measurement rather than as an absence. It renders as the string
        // `none` so the two sides can be compared on it: a client that sends a
        // block where the browser sends none is a difference, not an
        // unanswerable question.
        "http2.stream_priority" => Some(profile.http2.stream_priority.map_or_else(
            || "none".to_owned(),
            |p| {
                // ⛔ NAMED FOR THE WIRE, like the field it reads. HTTP/2
                // encodes the weight as the weight minus one, so a report
                // saying `weight=255` where the model holds `weight_wire=255`
                // would be off by one against every tool a reader compares it
                // with.
                format!(
                    "dep={} weight_wire={} exclusive={}",
                    p.stream_dependency, p.weight_wire, p.exclusive
                )
            },
        )),
        "http2.priority_frames.count" => Some(profile.http2.priority_frames.len().to_string()),
        "http2.frames.order" => {
            if profile.http2.frames.is_empty() {
                None
            } else {
                Some(
                    profile
                        .http2
                        .frames
                        .iter()
                        .map(|f| match f {
                            Frame::Settings { .. } => "settings".to_owned(),
                            Frame::WindowUpdate { .. } => "window_update".to_owned(),
                            Frame::Headers { .. } => "headers".to_owned(),
                            Frame::Other { frame_type, .. } => format!("other:{frame_type:#04x}"),
                        })
                        .collect::<Vec<_>>()
                        .join(","),
                )
            }
        }
        _ => None,
    }
}

/// Why a difference on one GREASE-masked field is not evidence.
const GREASE_WHY: &str =
    "only the drawn GREASE value differs, and RFC 8701 GREASE is drawn per connection";

/// Fields a real browser varies per connection, with the reason each does.
///
/// ⛔ **A DIFFERENCE HERE BETWEEN TWO SINGLE CAPTURES IS NOT EVIDENCE, and
/// reporting one as a conformance failure is what would make this tool
/// unreadable.** Chrome has shuffled its extension order since 110, so two
/// captures of one browser differ on it every time. Measured in this corpus:
/// `chrome-151.0.7922.174-win64-stable` against
/// `chrome-151.0.7922.173-linux64-stable` conforms on 26 of 28 fields, and the
/// one remaining difference is the shuffle.
///
/// ⚠ **IT IS NOT A PASS EITHER.** `docs/glossary.md` records that a client
/// whose order never changes is MORE distinguishable, not less, because a fixed
/// sequence is itself a signal. One capture cannot see that, so the verdict says
/// so rather than concluding.
const PER_CONNECTION: &[(&str, &str)] = &[
    (
        "tls.extensions.order",
        "the extension order is shuffled per connection, and has been since Chrome 110, \
         so two captures of one browser differ here every time. The comparable field is \
         tls.extensions.set.no_grease, and a client whose order never changes across \
         several captures is distinguishable for that reason",
    ),
    (
        "tls.grease.positions",
        "GREASE sits inside the shuffled extension list, so its positions move with the \
         shuffle. tls.grease.count is the stable half and is compared separately",
    ),
];

/// The reason a field varies per connection, where it does.
#[must_use]
fn per_connection_reason(field: &str) -> Option<&'static str> {
    PER_CONNECTION
        .iter()
        .find(|(name, _)| *name == field)
        .map(|(_, why)| *why)
}

/// A rendered list with every GREASE value replaced by a placeholder.
///
/// ⛔ **MASKED IN PLACE, NEVER REMOVED, and the difference is the whole
/// correctness of this.** Removing them makes `[GREASE, a, b]` and
/// `[a, GREASE, b]` compare equal, so a client that put its GREASE in the wrong
/// position would be forgiven as a redraw. ⚠ Measured: the first version of
/// this function stripped, and
/// `conformance_a_grease_value_moved_to_another_position_is_still_caught` went
/// red on exactly that. Masking keeps the position and forgives only the value.
///
/// ⛔ **The project's own predicate**, so a value this comparison forgives is
/// exactly one [`b_ids_schema::tls::is_grease_value`] recognises. A second
/// definition of GREASE would be a second answer to what RFC 8701 reserved.
///
/// ⚠ **It parses each element rather than pattern-matching the text.** A field
/// rendered as `0xcaca=2` in a settings list must not lose its value half to a
/// string match on `caca`.
fn grease_masked(rendered: &str) -> Vec<String> {
    rendered
        .split(',')
        .map(|element| {
            let head = element.split('=').next().unwrap_or(element);
            let Some(hex) = head.strip_prefix("0x") else {
                return element.to_owned();
            };
            match u16::from_str_radix(hex, 16) {
                Ok(value) if is_grease_value(value) => {
                    // ⚠ The tail is kept, so `0xcaca=2` and `0x1a1a=3` still
                    // differ. Only the reserved codepoint is masked.
                    element.replacen(head, "GREASE", 1)
                }
                _ => element.to_owned(),
            }
        })
        .collect()
}

/// Every field this comparison knows, TLS first and then HTTP.
#[must_use]
pub fn fields() -> Vec<&'static str> {
    FIELDS
        .iter()
        .copied()
        .chain(HTTP_FIELDS.iter().copied())
        .collect()
}

/// Compare a captured client against the profile it claims to be.
///
/// ⛔ **Nothing here checks that the claim is plausible.** A client claiming a
/// build it is nothing like produces a report full of differences, which is the
/// correct answer rather than an error.
///
/// ⚠ **The two sides are both `Profile`s and that is deliberate.** The observed
/// side is what this project's own capture path produces from a client, so the
/// comparison is over one model rather than over a capture type on one side and
/// a profile on the other.
#[must_use]
pub fn compare(claimed: &Profile, observed: &Profile) -> Report {
    let fields = fields()
        .into_iter()
        .map(|field| {
            let (c, o) = if field.starts_with("http") {
                (render_http(field, claimed), render_http(field, observed))
            } else {
                (render(field, &claimed.tls), render(field, &observed.tls))
            };
            let verdict = match (c, o) {
                (Some(a), Some(b)) if a == b => Verdict::Conforms(a),
                (Some(a), Some(b)) if grease_masked(&a) == grease_masked(&b) => {
                    Verdict::PerConnection {
                        claimed: a,
                        observed: b,
                        why: GREASE_WHY.to_owned(),
                    }
                }
                (Some(a), Some(b)) if per_connection_reason(field).is_some() => {
                    Verdict::PerConnection {
                        claimed: a,
                        observed: b,
                        why: per_connection_reason(field).unwrap_or_default().to_owned(),
                    }
                }
                (Some(a), Some(b)) => Verdict::Differs {
                    claimed: a,
                    observed: b,
                },
                (a, b) => Verdict::NotCheckable {
                    claimed: a.is_some(),
                    observed: b.is_some(),
                },
            };
            FieldReport {
                field: field.to_owned(),
                verdict,
            }
        })
        .collect();
    Report {
        claimed: claimed.id.as_str().to_owned(),
        observed: observed.id.as_str().to_owned(),
        fields,
    }
}

/// Render a report the way the command prints it.
#[must_use]
pub fn render_report(report: &Report) -> String {
    let differing = report.differing();
    let mut out = String::new();
    out.push_str(&format!(
        "conformance: {} field(s) compared, {} conform, {} differ, {} vary per \
         connection, {} not checkable\n",
        report.fields.len(),
        report.conforming(),
        differing.len(),
        report.per_connection().len(),
        report.not_checkable().len(),
    ));
    out.push_str(&format!(
        "  claimed   {}\n  observed  {}\n",
        report.claimed, report.observed
    ));
    if differing.is_empty() {
        out.push_str(
            "\n⭐ Every field both sides carry agrees.\n\
             ⚠ A not-checkable field is not a passing one: it is a field one side\n\
             does not carry, and nothing was compared on it.\n",
        );
        return out;
    }
    out.push_str("\n⛔ The fields this client got wrong:\n\n");
    for field in differing {
        if let Verdict::Differs { claimed, observed } = &field.verdict {
            out.push_str(&format!("  {}\n", field.field));
            out.push_str(&format!("    claimed   {claimed}\n"));
            out.push_str(&format!("    observed  {observed}\n"));
        }
    }
    out
}
