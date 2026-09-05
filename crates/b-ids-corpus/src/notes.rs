//! The release body and the changelog entry, from one model.
//!
//! ⛔ **Release notes and a changelog written separately drift, and the reader
//! who trusts the wrong one is the one who was doing something careful.** So
//! they cannot disagree by construction rather than by discipline: [`model`]
//! computes what changed, and the two renderers below are the only things that
//! turn it into text. `docs/history/todo/publish.md`, `PUB-08`.
//!
//! ⭐ **`CI-04`'s pull-request body is the third renderer of the same model**,
//! for the same reason.
//!
//! ⚠ **The field-level diff is `b_ids_validator::diff`'s**, not a second
//! comparison written here. A generator with its own idea of what changed would
//! disagree with the validator the first time either moved.

use b_ids_schema::Profile;
use b_ids_validator::{Diff, diff};

/// One route's movement between two corpus states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Movement {
    /// A profile at a route that held none before.
    Added {
        /// The profile that arrived.
        id: String,
        /// The build it describes.
        version: String,
    },
    /// A newer build at a route that already held one.
    ///
    /// ⛔ **Never an edit.** The corpus is append-only, so this is a new file
    /// beside the old one and both remain published.
    Advanced {
        /// The route, as `browser/channel/platform`.
        route: String,
        /// The build that was there.
        from: String,
        /// The build that arrived.
        to: String,
        /// What the two differ in, field by field.
        diff: Diff,
    },
}

impl Movement {
    /// The line both renderers use to name this movement.
    ///
    /// ⛔ **One sentence, one place.** The release body and the changelog print
    /// the same words for the same movement because they call this, which is
    /// the whole of what makes them agree.
    #[must_use]
    pub fn headline(&self) -> String {
        match self {
            Self::Added { id, version } => format!("new: {id} at {version}"),
            Self::Advanced {
                route, from, to, ..
            } => format!("advanced: {route} from {from} to {to}"),
        }
    }

    /// The fields that moved, in the diff's own fixed order.
    ///
    /// ⚠ Empty for an addition: there is nothing to compare it against, and
    /// listing every field of a first profile as "changed" would be a diff
    /// against nothing.
    #[must_use]
    pub fn fields(&self) -> Vec<String> {
        match self {
            Self::Added { .. } => Vec::new(),
            Self::Advanced { diff, .. } => diff.changes.iter().map(|c| c.field.clone()).collect(),
        }
    }
}

/// What changed between two corpus states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Every movement, in route order.
    pub movements: Vec<Movement>,
    /// How many profiles the corpus held afterwards.
    pub profiles_after: usize,
}

impl Change {
    /// Whether anything moved at all.
    ///
    /// ⛔ **A no-op change renders nothing**, in both renderers. Silence is the
    /// correct output for "the browser did not change", and a bot that writes
    /// on a schedule trains people to ignore it. `CI-04` states the same rule.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.movements.is_empty()
    }
}

/// The route a profile publishes at, as the key two states are compared on.
fn route_key(profile: &Profile) -> String {
    format!(
        "{}/{}/{}",
        profile.browser.name.to_ascii_lowercase(),
        profile.browser.channel.as_str(),
        profile.platform_token().as_str()
    )
}

/// Compare two corpus states.
///
/// ⛔ **The ONE place that decides what changed.** Both renderers take this and
/// neither computes anything of its own.
///
/// ⚠ **Ordered by route**, so two runs over one pair produce identical text.
#[must_use]
pub fn model(before: &[Profile], after: &[Profile]) -> Change {
    let mut movements = Vec::new();
    let mut routes: Vec<String> = after.iter().map(route_key).collect();
    routes.sort();
    routes.dedup();

    for route in routes {
        let latest = |set: &[Profile]| -> Option<Profile> {
            let mut here: Vec<&Profile> = set.iter().filter(|p| route_key(p) == route).collect();
            // ⚠ Highest build wins, compared as parsed components rather than
            // as text, so `7922.9` sorts below `7922.76`.
            here.sort_by_key(|p| {
                p.browser
                    .version
                    .split('.')
                    .map(|part| part.parse::<u64>().unwrap_or(0))
                    .collect::<Vec<u64>>()
            });
            here.last().map(|p| (*p).clone())
        };
        let Some(now) = latest(after) else { continue };
        match latest(before) {
            None => movements.push(Movement::Added {
                id: now.id.to_string(),
                version: now.browser.version.clone(),
            }),
            Some(was) if was.browser.version == now.browser.version => {}
            Some(was) => movements.push(Movement::Advanced {
                route: route.clone(),
                from: was.browser.version.clone(),
                to: now.browser.version.clone(),
                diff: diff(&was, &now),
            }),
        }
    }

    Change {
        movements,
        profiles_after: after.len(),
    }
}

/// The release body.
///
/// ⚠ **Longer than the changelog entry and made of the same facts.** A reader
/// arriving at a release has nothing else open; a reader of the changelog is
/// already in the tree.
#[must_use]
pub fn release_body(change: &Change) -> String {
    if change.is_empty() {
        return String::new();
    }
    let mut out = String::from("## What changed\n\n");
    for movement in &change.movements {
        out.push_str(&format!("- {}\n", movement.headline()));
        for field in movement.fields() {
            out.push_str(&format!("  - {field}\n"));
        }
    }
    out.push_str(&format!(
        "\nThe corpus holds {} profile(s) after this change.\n",
        change.profiles_after
    ));
    // ⛔ EVERY RELEASE BODY STATES THE LICENCE, read from the one home rather
    // than typed. A release asset that travels alone still has to say what it
    // is. `docs/history/todo/publish.md`, `PUB-07`.
    out.push_str(&format!(
        "\nPublished under {}. Every profile and the index carry the same identifier.\n",
        b_ids_schema::LICENSE
    ));
    out
}

/// The changelog entry.
#[must_use]
pub fn changelog_entry(change: &Change) -> String {
    if change.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for movement in &change.movements {
        out.push_str(&format!("- {}\n", movement.headline()));
        for field in movement.fields() {
            out.push_str(&format!("  - {field}\n"));
        }
    }
    out.push_str(&format!(
        "- the corpus holds {} profile(s) after this change\n",
        change.profiles_after
    ));
    out
}

/// Every fact both renderers must carry, in order.
///
/// ⛔ **This is what "agree field for field" means**, and it is a function
/// rather than a sentence so the check can assert it. Both renderers are
/// required to contain every line this returns.
#[must_use]
pub fn facts(change: &Change) -> Vec<String> {
    let mut out = Vec::new();
    for movement in &change.movements {
        out.push(movement.headline());
        out.extend(movement.fields());
    }
    if !change.is_empty() {
        out.push(format!(
            "{} profile(s) after this change",
            change.profiles_after
        ));
    }
    out
}
