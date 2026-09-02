//! What changed between two profiles, at field level.
//!
//! ⭐ **"What changed between these two versions" is the single most useful
//! artefact for anybody maintaining a client**, and it is free once two profiles
//! exist. `TODO/validator.md`, `VALID-06`.
//!
//! ⛔ **It names the change, not the digest.** "This header moved from position
//! twelve to position five" is something a reader can act on; "the digest
//! changed" is something they have to go and find out about.
//!
//! ⛔ **AND IT REFUSES TO PRETEND TWO CAPTURES ARE COMPARABLE WHEN THEY ARE
//! NOT.** Two profiles differing in version AND platform AND trust configuration
//! cannot isolate anything, and a diff rendered without saying so invites a
//! reader to attribute every line to the version.

use b_ids_schema::Profile;

/// One field that differs between two profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The field, in the profile's own naming.
    pub field: String,
    /// What the first profile has.
    pub before: String,
    /// What the second has.
    pub after: String,
}

impl core::fmt::Display for Change {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {} -> {}", self.field, self.before, self.after)
    }
}

/// A condition the two captures did not share.
///
/// ⚠ **Each one is a reason a change cannot be attributed to the version.** They
/// are listed rather than counted, because which condition moved decides what
/// the diff can mean.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uncontrolled {
    /// The condition that differs.
    pub condition: &'static str,
    /// What the first capture was taken under.
    pub before: String,
    /// What the second was.
    pub after: String,
}

impl core::fmt::Display for Uncontrolled {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}: {} against {}",
            self.condition, self.before, self.after
        )
    }
}

/// What two profiles differ in, and what they did not hold still.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// The fields that changed, in a fixed order.
    pub changes: Vec<Change>,
    /// The conditions that were not held still, besides the version.
    pub uncontrolled: Vec<Uncontrolled>,
}

impl Diff {
    /// Whether anything changed at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Whether this diff can attribute its changes to the version.
    ///
    /// ⛔ **False when any other condition moved**, and a renderer says so above
    /// the changes rather than leaving a reader to notice.
    #[must_use]
    pub fn isolates_the_version(&self) -> bool {
        self.uncontrolled.is_empty()
    }
}

fn header_positions(profile: &Profile) -> Vec<(String, usize)> {
    profile
        .http
        .variants
        .first()
        .map(|set| {
            set.headers
                .iter()
                .enumerate()
                .map(|(i, h)| (h.name.clone(), i))
                .collect()
        })
        .unwrap_or_default()
}

/// Every field-level difference between two profiles.
///
/// ⛔ **The order is fixed** so two runs over the same pair produce the same
/// text, which is what lets a pull-request body be compared against a previous
/// one.
#[must_use]
pub fn diff(before: &Profile, after: &Profile) -> Diff {
    let mut changes = Vec::new();
    let mut push = |field: &str, a: String, b: String| {
        if a != b {
            changes.push(Change {
                field: field.to_owned(),
                before: a,
                after: b,
            });
        }
    };

    push(
        "browser.version",
        before.browser.version.clone(),
        after.browser.version.clone(),
    );

    // -- the TLS half ---------------------------------------------------------
    //
    // ⚠ GREASE is excluded from every list compared here, because it is drawn
    // per connection: a diff that reported it would report a draw as a change
    // on every pair, which is a diff nobody reads twice.
    let without_grease = |values: &[u16]| {
        values
            .iter()
            .filter(|v| !b_ids_schema::tls::is_grease_value(**v))
            .map(|v| format!("0x{v:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    push(
        "tls.cipher_suites",
        without_grease(&before.tls.cipher_suites),
        without_grease(&after.tls.cipher_suites),
    );
    let codepoints = |p: &Profile| {
        without_grease(
            &p.tls
                .extensions
                .iter()
                .map(|e| e.codepoint)
                .collect::<Vec<_>>(),
        )
    };
    // ⭐ The SET, sorted, so an order shuffle is not reported as an appearance.
    let sorted_set = |p: &Profile| {
        let mut all: Vec<u16> = p
            .tls
            .extensions
            .iter()
            .map(|e| e.codepoint)
            .filter(|c| !b_ids_schema::tls::is_grease_value(*c))
            .collect();
        all.sort_unstable();
        all.iter()
            .map(|v| format!("0x{v:04x}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    push("tls.extensions.set", sorted_set(before), sorted_set(after));
    let _ = codepoints;
    push(
        "tls.key_exchange_groups",
        without_grease(&before.tls.key_exchange_groups),
        without_grease(&after.tls.key_exchange_groups),
    );
    push(
        "tls.alpn",
        before.tls.alpn.join(","),
        after.tls.alpn.join(","),
    );

    // -- the HTTP half, position by position ---------------------------------
    //
    // ⭐ THE CHANGE THIS ENTRY EXISTS FOR. A header that moved is the kind of
    // change only a capture finds, and reporting it as "the header list changed"
    // would be reporting the digest again.
    let a_positions = header_positions(before);
    let b_positions = header_positions(after);
    for (name, index) in &a_positions {
        match b_positions.iter().find(|(n, _)| n == name) {
            Some((_, other)) if other != index => changes.push(Change {
                field: format!("http.headers.{name}"),
                before: format!("position {index}"),
                after: format!("position {other}"),
            }),
            Some(_) => {}
            None => changes.push(Change {
                field: format!("http.headers.{name}"),
                before: format!("position {index}"),
                after: "absent".to_owned(),
            }),
        }
    }
    for (name, index) in &b_positions {
        if !a_positions.iter().any(|(n, _)| n == name) {
            changes.push(Change {
                field: format!("http.headers.{name}"),
                before: "absent".to_owned(),
                after: format!("position {index}"),
            });
        }
    }

    // -- what was not held still ---------------------------------------------
    let mut uncontrolled = Vec::new();
    let mut condition = |name: &'static str, a: String, b: String| {
        if a != b {
            uncontrolled.push(Uncontrolled {
                condition: name,
                before: a,
                after: b,
            });
        }
    };
    condition(
        "platform",
        before.platform_token().as_str().to_owned(),
        after.platform_token().as_str().to_owned(),
    );
    condition(
        "browser.name",
        before.browser.name.clone(),
        after.browser.name.clone(),
    );
    condition(
        "browser.channel",
        before.browser.channel.as_str().to_owned(),
        after.browser.channel.as_str().to_owned(),
    );
    condition(
        "captured.trust",
        before.captured.trust.as_str().to_owned(),
        after.captured.trust.as_str().to_owned(),
    );
    condition(
        "captured.resumption",
        before
            .captured
            .resumption
            .map_or_else(|| "not recorded".to_owned(), |r| r.as_str().to_owned()),
        after
            .captured
            .resumption
            .map_or_else(|| "not recorded".to_owned(), |r| r.as_str().to_owned()),
    );

    Diff {
        changes,
        uncontrolled,
    }
}

/// The diff as a person reads it.
///
/// ⛔ **The uncontrolled conditions come FIRST**, above the changes, because a
/// reader who sees them after the list has already attributed the list.
#[must_use]
pub fn render(before: &Profile, after: &Profile, diff: &Diff) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{} -> {}\n",
        before.id.as_str(),
        after.id.as_str()
    ));
    if !diff.isolates_the_version() {
        out.push_str(
            "\n\u{26D4} these two captures do not differ only in version, so nothing below can \
             be\n   attributed to the version alone:\n",
        );
        for condition in &diff.uncontrolled {
            out.push_str(&format!("     {condition}\n"));
        }
    }
    if diff.is_empty() {
        out.push_str("\nno field differs\n");
        return out;
    }
    out.push_str(&format!("\n{} field(s) differ:\n", diff.changes.len()));
    for change in &diff.changes {
        out.push_str(&format!("  {change}\n"));
    }
    out
}
