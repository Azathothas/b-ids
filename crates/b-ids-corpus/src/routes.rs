//! Routes a program with nothing but `curl` can read.
//!
//! ⛔ **A single-value file contains the value and nothing else.** No trailing
//! newline, no leading whitespace, no quotes. A consumer must never need to
//! strip anything, and `scripts/common/check-routes.sh` is what says so.
//! `TODO/publish.md`, `PUB-03`.
//!
//! ⭐ **Measured on the reference the requirement came from**, which publishes
//! single-value files that DO end with a newline. That defect is what this
//! entry exists to remove, and it is why the rule is a check rather than a
//! preference. `docs/reference-sweeps/usable.md` section 9.
//!
//! # The layout
//!
//! ```text
//! user-agent/chrome/stable/152.0.7977.76/win64/navigate.txt
//! user-agent/chrome/stable/latest/win64/navigate.txt
//! client-hello-hex/chrome/stable/latest/win64.txt
//! header-order/chrome/stable/latest/win64/navigate.list.txt
//! ```
//!
//! ⚠ **The platform is the corpus's own token**, `win64` rather than
//! `windows`, so a consumer that knows the corpus route knows this one. A
//! second spelling would be a value in two places with nothing checking that
//! they agree.
//!
//! ⛔ **A route is generated only where the corpus HOLDS the value.** A route
//! that resolves to a plausible-looking wrong value is worse than one that
//! 404s, which is the same rule the corpus itself has. So `ja3` and `ja4` have
//! no routes at all today: nothing here computes one, and `VALID-04` is the
//! entry that will.
//!
//! ⛔ **Nothing falls back to a neighbouring platform.** A missing route is a
//! fact; a substituted value is a lie.

use b_ids_schema::{Channel, Profile};

/// A value a route publishes.
///
/// ⭐ **A table rather than a case statement in a generator.** Adding a property
/// is a variant and a reader here, and `scripts/common/check-routes.sh` reads
/// the same names out of the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Property {
    /// The `user-agent` header of one request variant.
    UserAgent,
    /// The `sec-ch-ua` header of one request variant.
    SecChUa,
    /// The `accept-language` header of one request variant.
    AcceptLanguage,
    /// The header names in wire order, one per line.
    HeaderOrder,
    /// The ALPN protocols offered, in order, one per line.
    Alpn,
    /// The `ClientHello` as this project read it off the wire.
    ClientHelloHex,
}

impl Property {
    /// Every property, in the order the routes are generated.
    #[must_use]
    pub fn all() -> [Self; 6] {
        [
            Self::UserAgent,
            Self::SecChUa,
            Self::AcceptLanguage,
            Self::HeaderOrder,
            Self::Alpn,
            Self::ClientHelloHex,
        ]
    }

    /// The directory a route for this property sits under.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserAgent => "user-agent",
            Self::SecChUa => "sec-ch-ua",
            Self::AcceptLanguage => "accept-language",
            Self::HeaderOrder => "header-order",
            Self::Alpn => "alpn",
            Self::ClientHelloHex => "client-hello-hex",
        }
    }

    /// The header this property reads, where it reads one.
    ///
    /// ⚠ `None` for a property that is not a single header, which is what
    /// decides whether the route is per REQUEST VARIANT or per platform.
    #[must_use]
    pub fn header(self) -> Option<&'static str> {
        match self {
            Self::UserAgent => Some("user-agent"),
            Self::SecChUa => Some("sec-ch-ua"),
            Self::AcceptLanguage => Some("accept-language"),
            _ => None,
        }
    }

    /// Whether the file holds more than one value.
    ///
    /// ⛔ **A multi-value file says so by its extension**, so the two are
    /// distinguishable without fetching one.
    #[must_use]
    pub fn multi_value(self) -> bool {
        matches!(self, Self::HeaderOrder | Self::Alpn)
    }

    /// Whether a route for this property carries a request variant.
    #[must_use]
    pub fn per_variant(self) -> bool {
        self.header().is_some() || self == Self::HeaderOrder
    }

    /// The extension a file for this property carries.
    #[must_use]
    pub fn extension(self) -> &'static str {
        if self.multi_value() {
            "list.txt"
        } else {
            "txt"
        }
    }

    /// Read a property from the name a caller wrote.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|p| p.as_str() == name)
    }
}

impl core::fmt::Display for Property {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One generated route.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Route {
    /// Where the file is written, relative to the route root.
    pub path: String,
    /// What the file contains, with ⛔ no trailing newline for a single value.
    pub value: String,
    /// The published profile the value was read from.
    pub profile: String,
    /// Which property it is, as the manifest spells it.
    pub property: String,
    /// The request variant, where the property has one.
    pub variant: Option<String>,
    /// Whether the file carries more than one value.
    pub multi_value: bool,
}

/// The manifest a checker reads to verify a route against the corpus.
///
/// ⭐ **Without it a check can only ask whether the generator agrees with
/// itself.** The manifest names the profile and the property behind every
/// route, so an independent reader can go and look.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Manifest {
    /// The manifest's own schema identifier.
    pub schema: &'static str,
    /// Every route, in generation order.
    pub routes: Vec<Route>,
}

/// The manifest's schema identifier.
pub const MANIFEST_SCHEMA: &str = "corpus-routes/1";

/// The file the manifest is written to.
pub const MANIFEST_FILE: &str = "routes.json";

/// The per-directory listing, for a consumer that discovers rather than
/// constructs.
pub const INDEX_FILE: &str = "index.txt";

/// The name a route uses where a version would be, for the current stable build.
///
/// ⛔ **A real file, not a redirect**, so one fetch is one round trip.
pub const LATEST: &str = "latest";

/// The value one property has in one profile, where the profile carries it.
///
/// ⚠ `None` where the corpus does not hold it, which is what stops a route
/// being generated at all.
#[must_use]
fn value_of(profile: &Profile, property: Property, variant: usize) -> Option<String> {
    let set = profile.http.variants.get(variant)?;
    match property {
        Property::UserAgent | Property::SecChUa | Property::AcceptLanguage => {
            let name = property.header()?;
            set.headers
                .iter()
                .find(|h| h.name == name)
                .and_then(|h| h.value.clone())
        }
        Property::HeaderOrder => {
            let names: Vec<&str> = set.headers.iter().map(|h| h.name.as_str()).collect();
            if names.is_empty() {
                None
            } else {
                Some(names.join("\n"))
            }
        }
        Property::Alpn => {
            if profile.tls.alpn.is_empty() {
                None
            } else {
                Some(profile.tls.alpn.join("\n"))
            }
        }
        Property::ClientHelloHex => profile.raw.client_hello_hex.clone(),
    }
}

/// The route key a profile publishes under, as `browser/channel/platform`.
fn keys(profile: &Profile) -> (String, &'static str, String) {
    (
        profile.browser.name.to_ascii_lowercase(),
        profile.browser.channel.as_str(),
        profile.platform_token().as_str().to_owned(),
    )
}

/// Version components, so `7922.9` sorts below `7922.76`.
fn version_key(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

/// Every route the corpus supports, plus the `latest` copies.
///
/// ⛔ **Only where the value exists.** A property a profile does not carry gets
/// no file, so a consumer that 404s has learned something true.
///
/// ⚠ **`latest` is the highest STABLE build at a route**, duplicated as a real
/// file rather than published as a redirect. `CORPUS-03` is the rule that
/// `latest` means stable and nothing else.
#[must_use]
pub fn routes(published: &[(String, Profile)]) -> Vec<Route> {
    let mut out = Vec::new();
    for (path, profile) in published {
        let (browser, channel, platform) = keys(profile);
        for property in Property::all() {
            let variants: Vec<usize> = if property.per_variant() {
                (0..profile.http.variants.len()).collect()
            } else {
                vec![0]
            };
            for variant in variants {
                let Some(value) = value_of(profile, property, variant) else {
                    continue;
                };
                let variant_name = profile
                    .http
                    .variants
                    .get(variant)
                    .map(|set| set.variant.to_string());
                let tail = if property.per_variant() {
                    match &variant_name {
                        Some(name) => format!("{platform}/{name}.{}", property.extension()),
                        None => continue,
                    }
                } else {
                    format!("{platform}.{}", property.extension())
                };
                out.push(Route {
                    path: format!(
                        "{}/{browser}/{channel}/{}/{tail}",
                        property.as_str(),
                        profile.browser.version
                    ),
                    value,
                    profile: path.clone(),
                    property: property.as_str().to_owned(),
                    variant: if property.per_variant() {
                        variant_name
                    } else {
                        None
                    },
                    multi_value: property.multi_value(),
                });
            }
        }
    }

    // ⭐ THE LATEST COPIES, derived from the routes above rather than from a
    // second walk of the corpus. A second walk would be a second answer to
    // which build is current.
    let mut latest: Vec<Route> = Vec::new();
    for route in &out {
        let Some(profile) = published
            .iter()
            .find(|(path, _)| *path == route.profile)
            .map(|(_, profile)| profile)
        else {
            continue;
        };
        // ⛔ STABLE ONLY. A consumer following `latest` must never be handed a
        // pre-release build. `TODO/corpus.md`, `CORPUS-03`.
        if profile.browser.channel != Channel::Stable {
            continue;
        }
        let version = &profile.browser.version;
        let newer = published.iter().any(|(_, other)| {
            other.browser.channel == Channel::Stable
                && keys(other) == keys(profile)
                && version_key(&other.browser.version) > version_key(version)
        });
        if newer {
            continue;
        }
        latest.push(Route {
            path: route
                .path
                .replacen(&format!("/{version}/"), &format!("/{LATEST}/"), 1),
            ..route.clone()
        });
    }
    out.extend(latest);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The manifest for a set of routes.
#[must_use]
pub fn manifest(routes: Vec<Route>) -> Manifest {
    Manifest {
        schema: MANIFEST_SCHEMA,
        routes,
    }
}

/// The per-directory listings a discovering consumer reads.
///
/// ⭐ **Every directory gets one, including the intermediate ones**, so a
/// consumer can walk down from the root without knowing the axes.
///
/// ⚠ **A listing ends with a newline and that is not the single-value rule.**
/// It is a multi-value file by construction, and a consumer reading a list
/// splits on newlines rather than taking the whole body as a value.
#[must_use]
pub fn indexes(routes: &[Route]) -> Vec<(String, String)> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut children: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    children.insert(String::new(), BTreeSet::new());
    for route in routes {
        let parts: Vec<&str> = route.path.split('/').collect();
        for depth in 0..parts.len() {
            let parent = parts[..depth].join("/");
            let child = parts[depth].to_owned();
            let entry = if depth + 1 == parts.len() {
                child
            } else {
                format!("{child}/")
            };
            children.entry(parent).or_default().insert(entry);
        }
    }
    children
        .into_iter()
        .map(|(dir, names)| {
            let path = if dir.is_empty() {
                INDEX_FILE.to_owned()
            } else {
                format!("{dir}/{INDEX_FILE}")
            };
            let mut body: String = names.into_iter().collect::<Vec<_>>().join("\n");
            body.push('\n');
            (path, body)
        })
        .collect()
}
