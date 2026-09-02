//! The HTTP half, its variants, and the one privacy rule.
//!
//! ⛔ **A header set says which request kind produced it.** A top-level
//! navigation, a subresource fetch and a reload are three different sets from
//! one browser, and a corpus that records one without saying which cannot be
//! compared against anything.
//!
//! ⚠ **This project has no measured example of the difference.** The variant
//! model is designed rather than derived: a claim that two Chrome captures
//! differed by a `cache-control` header was checked against the capture it came
//! from and refuted. `docs/inherited-claims.md` section 10 carries the
//! refutation. The model survives it because nothing in it rested on that
//! capture: three request kinds are three kinds whether or not any two captures
//! happen to differ.
//!
//! # The privacy rule, which is a schema rule
//!
//! ⛔ **It is a rule about the DEFAULT SHAPE, which is why it lives here rather
//! than in a capture flag.** A model whose natural form carries header values
//! is a model that will one day publish a credential, whatever the capture code
//! is careful about.
//!
//! - the default shape carries header names only;
//! - values are recorded only under [`ValuePolicy::WithValues`];
//! - `cookie` and `authorization` keep their NAME and their POSITION and lose
//!   their value under either policy, marked `withheld`. `SCHEMA-14`.

use serde::{Deserialize, Serialize};

/// Header names that never reach a profile, whatever the policy.
///
/// ⛔ Matched case-insensitively. HTTP/2 lower-cases header names, but a capture
/// read from an HTTP/1.1 connection does not, and a rule that only catches one
/// spelling is a rule that catches nothing on the other wire.
pub const NEVER_RECORDED: [&str; 2] = ["cookie", "authorization"];

/// Which request kind produced a header set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    /// A top-level navigation.
    Navigate,
    /// A subresource fetch made by the page.
    Subresource,
    /// A reload of a page already loaded.
    Reload,
}

impl core::fmt::Display for Variant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Navigate => "navigate",
            Self::Subresource => "subresource",
            Self::Reload => "reload",
        })
    }
}

/// Whether a capture records header values at all.
///
/// ⭐ [`ValuePolicy::NamesOnly`] is [`Default`], and that is the whole
/// mechanism. A switch that has to be turned OFF for safety is a switch that
/// ships on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ValuePolicy {
    /// Names only. The default.
    #[default]
    NamesOnly,
    /// Names and values, minus [`NEVER_RECORDED`].
    WithValues,
}

/// One header field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderField {
    /// The header name, as it appeared.
    pub name: String,
    /// The value, present only under [`ValuePolicy::WithValues`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Whether this header carried a credential, whose value is withheld.
    ///
    /// ⭐ **Whether the header was sent, and where in the order, is a
    /// fingerprint signal in its own right, and it carries no secret.** Before
    /// 2026-09-02 a credential header was dropped entirely, name and all, so a
    /// recorded order closed over the gap and nothing downstream could tell
    /// that a header had been there. `TODO/schema.md`, `SCHEMA-14`.
    ///
    /// ⛔ **There is no mode under which the value is retained.** This field
    /// adds a way to say a header was here; it adds no way to say what was in
    /// it. [`Profile::check`] refuses an entry that is both withheld and
    /// carrying a value, and refuses the marker on a name that is not a
    /// credential.
    ///
    /// [`Profile::check`]: crate::Profile::check
    #[serde(default, skip_serializing_if = "is_false")]
    pub withheld: bool,
}

/// Whether a boolean is false, for `skip_serializing_if`.
///
/// ⚠ A profile written before the field existed has no `withheld` key, and a
/// profile whose headers carry no credential should not gain one on every
/// entry. The default and the omission are the same fact.
fn is_false(value: &bool) -> bool {
    !*value
}

/// One request kind's header set, in the order the headers were sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderSet {
    /// Which request kind produced this set.
    pub variant: Variant,
    /// The headers, in wire order, after the pseudo-headers.
    pub headers: Vec<HeaderField>,
}

impl HeaderSet {
    /// Record a header set from what the wire carried, under a policy.
    ///
    /// ⛔ This is the ONE construction path, and the rule is enforced here
    /// rather than by the caller remembering it. A second path that skipped the
    /// filter would be the "control gated on one of several paths" defect in
    /// the one place this project cannot afford it.
    #[must_use]
    pub fn record<I, N, V>(variant: Variant, headers: I, policy: ValuePolicy) -> Self
    where
        I: IntoIterator<Item = (N, V)>,
        N: Into<String>,
        V: Into<String>,
    {
        let headers = headers
            .into_iter()
            .map(|(name, value)| {
                let name: String = name.into();
                // ⛔ RECORDED AS PRESENT, NEVER AS A VALUE. The entry keeps its
                // place in the order and the value is not read at all: there is no
                // branch here that can put it into the field.
                if is_never_recorded(&name) {
                    return HeaderField {
                        name,
                        value: None,
                        withheld: true,
                    };
                }
                let value = match policy {
                    ValuePolicy::NamesOnly => None,
                    ValuePolicy::WithValues => Some(value.into()),
                };
                HeaderField {
                    name,
                    value,
                    withheld: false,
                }
            })
            .collect();
        Self { variant, headers }
    }

    /// Whether any field in this set carries a value.
    #[must_use]
    pub fn carries_values(&self) -> bool {
        self.headers.iter().any(|h| h.value.is_some())
    }

    /// The header names, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.headers.iter().map(|h| h.name.as_str())
    }

    /// Every entry whose value is withheld because it was a credential.
    ///
    /// ⭐ A reader asking "was a credential sent, and where" gets an answer
    /// here rather than inferring one from a gap that is not marked.
    pub fn withheld(&self) -> impl Iterator<Item = &HeaderField> {
        self.headers.iter().filter(|h| h.withheld)
    }
}

/// Whether a header name is one that never reaches a profile.
#[must_use]
pub fn is_never_recorded(name: &str) -> bool {
    NEVER_RECORDED
        .iter()
        .any(|banned| name.eq_ignore_ascii_case(banned))
}

/// The HTTP half of a profile: one header set per request kind captured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpHalf {
    /// The header sets, one per request kind.
    ///
    /// ⚠ At minimum a [`Variant::Navigate`] set. A profile with none records
    /// nothing about HTTP at all, which is a different thing from a profile
    /// that records a navigation and no subresource fetch.
    pub variants: Vec<HeaderSet>,
}

impl HttpHalf {
    /// The header set for one request kind, if it was captured.
    #[must_use]
    pub fn variant(&self, variant: Variant) -> Option<&HeaderSet> {
        self.variants.iter().find(|s| s.variant == variant)
    }

    /// Whether any set in this half carries a header value.
    #[must_use]
    pub fn carries_values(&self) -> bool {
        self.variants.iter().any(HeaderSet::carries_values)
    }
}
