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
    /// that a header had been there. `docs/history/todo/schema.md`, `SCHEMA-14`.
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

/// The alphabet a boundary's random part is drawn from.
///
/// ⛔ **An enum rather than a literal character set.** The set is a property of
/// the browser's generator and it is compared across profiles; a free string
/// would fail silently on an ordering or a spelling, which is the reason
/// [`crate::Trust`] is an enum for the same job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BoundaryAlphabet {
    /// `0123456789abcdef`.
    LowerHex,
    /// `0123456789ABCDEF`.
    UpperHex,
    /// `0-9A-Za-z`.
    Alphanumeric,
}

impl BoundaryAlphabet {
    /// Every alphabet, in the order the vocabulary is written down.
    #[must_use]
    pub fn all() -> [Self; 3] {
        [Self::LowerHex, Self::UpperHex, Self::Alphanumeric]
    }

    /// The word as it is written in a profile.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LowerHex => "lower-hex",
            Self::UpperHex => "upper-hex",
            Self::Alphanumeric => "alphanumeric",
        }
    }

    /// Whether one character is in this alphabet.
    #[must_use]
    pub fn contains(self, c: char) -> bool {
        match self {
            Self::LowerHex => c.is_ascii_digit() || ('a'..='f').contains(&c),
            Self::UpperHex => c.is_ascii_digit() || ('A'..='F').contains(&c),
            Self::Alphanumeric => c.is_ascii_alphanumeric(),
        }
    }
}

impl core::fmt::Display for BoundaryAlphabet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The shape of the boundary string a browser generates for a multipart body.
///
/// ⛔ **THE PATTERN, NEVER A DRAWN VALUE.** The boundary is generated per
/// request, like a GREASE codepoint, so one captured boundary is one draw and
/// recording it as though it were the value would publish a constant no browser
/// has. `docs/history/todo/schema.md`, `SCHEMA-11`.
///
/// ⚠ **Measured by reading somebody else's client, at a named commit, and not
/// by this project.** One client generates `----WebKitFormBoundary` plus
/// sixteen alphanumerics for one browser and `----geckoformboundary` plus
/// thirty-two hexadecimal characters for another;
/// `docs/reference-sweeps/usable.md` section 8 is the source. ⛔ No profile
/// carries this field until this project measures a form submission itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultipartBoundary {
    /// The literal text every boundary from this browser starts with.
    pub prefix: String,
    /// How many characters are drawn after the prefix.
    pub random_len: usize,
    /// Which alphabet those characters come from.
    pub alphabet: BoundaryAlphabet,
}

impl MultipartBoundary {
    /// Whether one observed boundary matches this pattern.
    ///
    /// ⛔ **Length is counted in CHARACTERS that are in the alphabet**, and the
    /// total is checked as well: a pattern that only asserted the prefix would
    /// match a boundary from any browser whose prefix happens to agree, and one
    /// that only counted would match a prefix that does not.
    #[must_use]
    pub fn matches(&self, observed: &str) -> bool {
        let Some(rest) = observed.strip_prefix(&self.prefix) else {
            return false;
        };
        rest.chars().count() == self.random_len && rest.chars().all(|c| self.alphabet.contains(c))
    }

    /// Every way this pattern is malformed on its own terms.
    ///
    /// ⛔ **A pattern with no random part is a constant**, and a constant is
    /// exactly what this field exists to avoid recording.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.prefix.is_empty() {
            out.push("multipart_boundary.prefix is empty".to_owned());
        }
        if self.random_len == 0 {
            out.push(
                "multipart_boundary.random_len is 0, which records a constant rather than a \
                 pattern. The boundary is drawn per request"
                    .to_owned(),
            );
        }
        out
    }
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
    /// The shape of the boundary this browser generates for a multipart body.
    ///
    /// ⛔ **Absent until this project measures a form submission itself.** The
    /// pattern is known by reading somebody else's client and
    /// `docs/inherited-claims.md` is where a value this project did not measure
    /// lives; nothing from there is published as data. ⚠ `None` is "not
    /// measured", which is a different fact from a browser that sends no
    /// boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multipart_boundary: Option<MultipartBoundary>,
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
