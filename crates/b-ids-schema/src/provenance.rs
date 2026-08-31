//! Provenance, per field, with four kinds and no more.
//!
//! ⛔ **Per field, not per profile.** A profile that is nine tenths measured and
//! one tenth copied is otherwise indistinguishable from one that is entirely
//! measured, and the copied tenth is the part that is wrong.
//!
//! ⭐ **A sibling map rather than a wrapper around every scalar.** Wrapping
//! makes every consumer pay for a field most of them never read, and it puts a
//! derived concern inside the measured data.
//!
//! ```jsonc
//! "provenance": {
//!   "tls.cipher_suites":               "wire",
//!   "http.headers.sec-ch-ua-platform": "substituted:platform-token",
//!   "tls.extensions.0xca34":           "unreproducible:root-store-snapshot"
//! }
//! ```
//!
//! ⛔ **Four kinds, and a fifth is how a provenance model stops meaning
//! anything.** The vocabulary is closed in the type, so adding one is an edit
//! somebody has to make deliberately rather than a string somebody can write.

use core::fmt;
use core::str::FromStr;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Defect;

/// How a field's value was arrived at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProvenanceKind {
    /// Read off a socket by this project's own harness.
    Wire,
    /// Taken from a capture of the same build on another platform.
    ///
    /// ⚠ Requires a reason.
    Substituted,
    /// Copied from somebody else's table, unverified here.
    ///
    /// ⛔ A profile carrying any `vendor` field is a draft and is never
    /// published. `b-ids-validator` is what refuses it.
    Vendor,
    /// Measured, and deliberately not shipped.
    ///
    /// ⚠ Requires a reason.
    Unreproducible,
}

impl ProvenanceKind {
    /// Whether this kind is meaningless without a reason.
    #[must_use]
    pub fn requires_reason(self) -> bool {
        matches!(self, Self::Substituted | Self::Unreproducible)
    }

    /// The word as it is written in a profile.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wire => "wire",
            Self::Substituted => "substituted",
            Self::Vendor => "vendor",
            Self::Unreproducible => "unreproducible",
        }
    }

    /// Every kind, in the order the vocabulary is written down.
    ///
    /// ⭐ This is what the published JSON Schema's enum is checked against, so
    /// a fifth kind added to the type and not to the schema fails a test.
    #[must_use]
    pub fn all() -> [Self; 4] {
        [
            Self::Wire,
            Self::Substituted,
            Self::Vendor,
            Self::Unreproducible,
        ]
    }
}

impl fmt::Display for ProvenanceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProvenanceKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "wire" => Ok(Self::Wire),
            "substituted" => Ok(Self::Substituted),
            "vendor" => Ok(Self::Vendor),
            "unreproducible" => Ok(Self::Unreproducible),
            _ => Err(()),
        }
    }
}

/// One field's provenance: a kind, and a reason where the kind needs one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceEntry {
    /// How the value was arrived at.
    pub kind: ProvenanceKind,
    /// Why, for the two kinds that are meaningless without it.
    pub reason: Option<String>,
}

impl ProvenanceEntry {
    /// Parse the `kind` or `kind:reason` form.
    ///
    /// # Errors
    ///
    /// [`Defect::ProvenanceKindUnknown`] when the kind is outside the four.
    /// ⚠ A missing reason is NOT refused here: it is refused by
    /// [`Provenance::check`], because an entry read in isolation cannot name
    /// the field it belongs to and the message has to.
    pub fn parse(field: &str, raw: &str) -> Result<Self, Defect> {
        let (kind_text, reason) = match raw.split_once(':') {
            Some((k, r)) if !r.is_empty() => (k, Some(r.to_owned())),
            Some((k, _)) => (k, None),
            None => (raw, None),
        };
        let kind =
            kind_text
                .parse::<ProvenanceKind>()
                .map_err(|()| Defect::ProvenanceKindUnknown {
                    field: field.to_owned(),
                    found: kind_text.to_owned(),
                })?;
        Ok(Self { kind, reason })
    }

    /// The `kind` or `kind:reason` form, as it is written in a profile.
    #[must_use]
    pub fn to_wire(&self) -> String {
        match &self.reason {
            Some(reason) => format!("{}:{reason}", self.kind),
            None => self.kind.to_string(),
        }
    }
}

/// The provenance map: field path to entry.
///
/// ⚠ Ordered, so two serialisations of one profile are byte-identical. An
/// unordered map turns a no-op re-emit into a diff, and a diff nobody can
/// explain is a diff nobody reviews.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provenance(BTreeMap<String, ProvenanceEntry>);

impl Provenance {
    /// An empty map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one field's provenance.
    pub fn insert(&mut self, field: impl Into<String>, entry: ProvenanceEntry) {
        self.0.insert(field.into(), entry);
    }

    /// One field's provenance, if it is recorded.
    #[must_use]
    pub fn get(&self, field: &str) -> Option<&ProvenanceEntry> {
        self.0.get(field)
    }

    /// Every field path this map covers, in order.
    pub fn fields(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }

    /// Every entry, in field order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &ProvenanceEntry)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// How many fields carry provenance.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether nothing carries provenance.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every defect in the map.
    ///
    /// ⛔ Every one, not the first. A caller fixing one field at a time makes
    /// one pass per field, and a report that names one defect at a time is a
    /// report nobody can plan from.
    #[must_use]
    pub fn check(&self) -> Vec<Defect> {
        let mut defects = Vec::new();
        for (field, entry) in &self.0 {
            if entry.kind.requires_reason() && entry.reason.is_none() {
                defects.push(Defect::ProvenanceReasonMissing {
                    field: field.clone(),
                    kind: entry.kind,
                });
            }
        }
        defects
    }

    /// Every field whose value was copied from somebody else's table.
    ///
    /// ⭐ A profile with any of these is a draft. This is the list a publisher
    /// has to be able to print, which is why it returns the fields rather than
    /// a boolean.
    #[must_use]
    pub fn vendor_fields(&self) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(_, e)| e.kind == ProvenanceKind::Vendor)
            .map(|(f, _)| f.as_str())
            .collect()
    }
}

impl Serialize for Provenance {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (field, entry) in &self.0 {
            map.serialize_entry(field, &entry.to_wire())?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Provenance {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let raw = BTreeMap::<String, String>::deserialize(deserializer)?;
        let mut out = BTreeMap::new();
        for (field, value) in raw {
            let entry = ProvenanceEntry::parse(&field, &value).map_err(D::Error::custom)?;
            out.insert(field, entry);
        }
        Ok(Self(out))
    }
}
