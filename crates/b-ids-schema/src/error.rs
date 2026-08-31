//! The one error type this crate returns.
//!
//! ⛔ Every variant names the FIELD it is about. An error that says a profile is
//! malformed without saying where is an error whose reader has to re-derive the
//! answer, and the acceptance for three entries in `TODO/schema.md` is a
//! message naming the field.

use core::fmt;

/// A profile that is malformed on its own terms.
///
/// ⚠ This is not the validator's question. A [`Defect`] means the bytes do not
/// describe a profile at all; whether a well-formed profile could have come
/// from a real browser is `b-ids-validator`'s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Defect {
    /// A field the model requires is absent or empty.
    FieldMissing {
        /// The dotted path of the field, as a provenance key would spell it.
        field: String,
    },
    /// A field is present and its content cannot be read as what it claims.
    FieldMalformed {
        /// The dotted path of the field.
        field: String,
        /// What was wrong with it, in terms of what was found.
        why: String,
    },
    /// The declared identifier disagrees with the four keys it is derived from.
    ///
    /// ⛔ The identifier is derived so a path can be constructed without an
    /// index. A stored identifier that has drifted from its keys points a
    /// consumer at a file that describes something else.
    IdMismatch {
        /// What the profile declared.
        declared: String,
        /// What the four keys derive to.
        derived: String,
    },
    /// A provenance kind outside the four the vocabulary allows.
    ProvenanceKindUnknown {
        /// The field the provenance entry is about.
        field: String,
        /// The kind that was found.
        found: String,
    },
    /// A digest used as identity.
    ///
    /// ⛔ A digest is DERIVED from a profile. Keying on one lets a consumer
    /// round-trip a profile through a value that cannot reconstruct it, and it
    /// makes the identity move whenever a browser reshuffles.
    DigestUsedAsIdentity {
        /// The digest field the identifier was taken from.
        field: String,
    },
    /// Connection state promoted into identity.
    ///
    /// ⛔ A session ticket, a pre-shared key or a server-echoed setting is
    /// something the browser LEARNED from the network. It is not identity, and
    /// a profile carrying it changes for reasons nothing in the corpus can
    /// explain.
    ///
    /// ⚠ The bytes still live in `raw.client_hello_hex`, because a capture is
    /// not edited. What is refused is promoting them into a parsed field.
    ConnectionStateInIdentity {
        /// The field carrying the state.
        field: String,
        /// What it is.
        what: String,
    },
    /// A `substituted` or `unreproducible` entry carrying no reason.
    ///
    /// ⚠ The reason is the whole content of those two kinds. "This came from
    /// somewhere else" without saying where is not provenance.
    ProvenanceReasonMissing {
        /// The field the provenance entry is about.
        field: String,
        /// The kind that requires a reason.
        kind: crate::ProvenanceKind,
    },
}

impl fmt::Display for Defect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldMissing { field } => {
                write!(f, "{field}: required and absent")
            }
            Self::FieldMalformed { field, why } => {
                write!(f, "{field}: {why}")
            }
            Self::IdMismatch { declared, derived } => write!(
                f,
                "id: declared {declared}, but the four keys derive {derived}"
            ),
            Self::ProvenanceKindUnknown { field, found } => write!(
                f,
                "provenance.{field}: {found} is not one of wire, substituted, vendor, unreproducible"
            ),
            Self::DigestUsedAsIdentity { field } => write!(
                f,
                "id: is the {field} digest. A digest is derived from a profile, and a profile is never derived from a digest"
            ),
            Self::ConnectionStateInIdentity { field, what } => write!(
                f,
                "{field}: carries {what}, which the browser learned from the network. It stays in raw.client_hello_hex and out of the identity"
            ),
            Self::ProvenanceReasonMissing { field, kind } => write!(
                f,
                "provenance.{field}: {kind} carries no reason, and the reason is what it is for"
            ),
        }
    }
}

impl core::error::Error for Defect {}

/// The field a defect is about, whatever its variant.
///
/// ⭐ Reporting is the reason this exists: a report that groups defects by field
/// needs the field without matching on every variant.
impl Defect {
    /// The dotted field path this defect is about.
    #[must_use]
    pub fn field(&self) -> &str {
        match self {
            Self::FieldMissing { field }
            | Self::FieldMalformed { field, .. }
            | Self::ProvenanceKindUnknown { field, .. }
            | Self::ConnectionStateInIdentity { field, .. }
            | Self::ProvenanceReasonMissing { field, .. } => field,
            Self::IdMismatch { .. } | Self::DigestUsedAsIdentity { .. } => "id",
        }
    }
}
