//! Which dimensions of a corpus a resolver can actually produce.
//!
//! ⭐ **A corpus can carry data for a browser family, a channel or a platform
//! that no code path can select.** It sits there looking authoritative, and a
//! reader who finds it believes it is used. `TODO/validator.md`, `VALID-03`.
//!
//! ⛔ **Measured by reading, at a named commit**: one library's classifier
//! returns three families and its header-order table carries a fourth key that
//! nothing can reach. Grep finds that key in exactly one file, as a key, and
//! nowhere else. `docs/reference-sweeps/usable.md` section 7.
//!
//! # ⛔ The resolver's list is INJECTED, never imported
//!
//! This crate is pure logic over the model and it does not depend on the
//! driver. ⚠ That is not a limitation to work around: the caller names what its
//! resolver can produce, so the same check answers for a driver, for a fixture,
//! and for a resolver that has not been written yet. `b-ids-driver`'s
//! `Family::all` is what the acceptance test passes.
//!
//! # ⛔ It says they disagree; it does not pick
//!
//! The data may be right and the resolver wrong. A check that "fixed" this by
//! deleting a profile would destroy a measurement to satisfy a code path, which
//! is the wrong way round in a project whose product is measurements.

use std::collections::BTreeSet;

use b_ids_schema::Profile;

/// One dimension of the corpus that no resolver branch can select.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unreachable {
    /// Which dimension: `browser`, `channel` or `platform`.
    pub dimension: &'static str,
    /// The value the corpus carries.
    pub value: String,
    /// Every profile id that carries it, in order.
    pub profiles: Vec<String>,
}

impl core::fmt::Display for Unreachable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} {} is published and no resolver branch can select it: {}",
            self.dimension,
            self.value,
            self.profiles.join(", ")
        )
    }
}

/// What a resolver can produce, as the caller's own code reports it.
///
/// ⚠ **Every list is required and none defaults to "anything".** A field that
/// defaulted to accepting everything would make this check pass over the
/// dimension the caller forgot to fill in, which is the shape of a guard that
/// has never been seen to fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reachable {
    /// The browser families the resolver has a branch for, lower-cased.
    pub browsers: BTreeSet<String>,
    /// The channels it can select.
    pub channels: BTreeSet<String>,
    /// The platform tokens it can run on.
    pub platforms: BTreeSet<String>,
}

impl Reachable {
    /// Build a list from three sets of names.
    #[must_use]
    pub fn new<I, J, K>(browsers: I, channels: J, platforms: K) -> Self
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        J: IntoIterator,
        J::Item: AsRef<str>,
        K: IntoIterator,
        K::Item: AsRef<str>,
    {
        let lower = |name: &str| name.to_ascii_lowercase();
        Self {
            browsers: browsers.into_iter().map(|n| lower(n.as_ref())).collect(),
            channels: channels.into_iter().map(|n| lower(n.as_ref())).collect(),
            platforms: platforms.into_iter().map(|n| lower(n.as_ref())).collect(),
        }
    }
}

/// Every dimension the corpus carries that the resolver cannot produce.
///
/// ⭐ **Nobody writes this check**, which is why the defect it looks for
/// survives in projects that are otherwise carefully written.
///
/// ⚠ The comparison is on the LOWER-CASED name, because that is what the corpus
/// derives its route from: `b_ids_corpus::route` lower-cases `browser.name`, so
/// a resolver reporting `Chrome` and a profile carrying `chrome` name the same
/// thing.
#[must_use]
pub fn unreachable_dimensions(profiles: &[Profile], reachable: &Reachable) -> Vec<Unreachable> {
    let mut out: Vec<Unreachable> = Vec::new();
    let mut push = |dimension: &'static str, value: String, id: &str| {
        if let Some(existing) = out
            .iter_mut()
            .find(|u| u.dimension == dimension && u.value == value)
        {
            existing.profiles.push(id.to_owned());
        } else {
            out.push(Unreachable {
                dimension,
                value,
                profiles: vec![id.to_owned()],
            });
        }
    };

    for profile in profiles {
        let browser = profile.browser.name.to_ascii_lowercase();
        if !reachable.browsers.contains(&browser) {
            push("browser", browser, profile.id.as_str());
        }
        let channel = profile.browser.channel.as_str().to_ascii_lowercase();
        if !reachable.channels.contains(&channel) {
            push("channel", channel, profile.id.as_str());
        }
        let platform = profile.platform_token().as_str().to_ascii_lowercase();
        if !reachable.platforms.contains(&platform) {
            push("platform", platform, profile.id.as_str());
        }
    }
    out.sort();
    out
}
