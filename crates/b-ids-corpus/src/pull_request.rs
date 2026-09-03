//! A scheduled run that finds a change opens a pull request, not an issue.
//!
//! ⛔ **An issue is a request for somebody else to do work. A pull request with
//! the work already in it is the deliverable.** `TODO/ci.md`, `CI-04`.
//!
//! ⭐ **This is the THIRD renderer of [`crate::notes`]'s model**, beside the
//! release body and the changelog entry. Nothing here recomputes what changed:
//! a body with its own idea of the change would disagree with the release the
//! first time either moved.
//!
//! # ⭐ What decides whether a request may merge without a human
//!
//! Five conditions, each a value rather than a sentence, so
//! `scripts/common/check-pr-body.sh` can assert them and a reviewer can read
//! which one failed. [`Conditions`] carries all five and [`Conditions::met`] is
//! the only thing that answers.
//!
//! ⛔ **Three of the five are computed from the corpus here, and two are facts
//! about the RUN that only the run knows.** The split matters: a caller cannot
//! claim agreement across two sources, because that is a question about the
//! published profiles and this module reads them; a caller must state whether
//! the formats round-tripped, because that happened in a step this module did
//! not watch.
//!
//! # ⚠ What is deliberately not decided here
//!
//! Opening, updating and closing the request is `capture.yml`'s. This module
//! produces the branch, the title, the body and the labels, and a workflow that
//! calls it can be read against
//! [`../../../.github/workflows/capture.yml`](../../../.github/workflows/capture.yml).
//! ⛔ A module that also called an API would be one component with two reasons
//! to fail, and the interesting half is the text.

use std::collections::BTreeSet;

use b_ids_schema::Profile;
use b_ids_validator::{Diff, render_diff};

use crate::notes::{Change, Movement, facts, model};

/// What kind of change this is, which is a label and a merge condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeClass {
    /// A route that held no profile before.
    NewRoute,
    /// A newer build of the same major.
    PatchBump,
    /// A newer major.
    MajorBump,
}

impl ChangeClass {
    /// The word a label carries.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NewRoute => "new-route",
            Self::PatchBump => "patch-bump",
            Self::MajorBump => "major-bump",
        }
    }

    /// The fields a change of this class is allowed to move.
    ///
    /// ⛔ **A TABLE, and it is short because of what
    /// [`b_ids_validator::diff`] actually compares.** `CI-04` anticipated that a
    /// version bump moves the User-Agent and the brand list; the diff compares
    /// header POSITIONS and not header VALUES, so neither is a field it can
    /// report. What a bump is predicted to move, in the fields that exist, is
    /// the version.
    ///
    /// ⚠ **So anything else is a human's decision**, which is the strict
    /// reading and the safe one. When the diff learns to compare a header
    /// value, this table is where the prediction grows.
    #[must_use]
    pub fn predicted_fields(self) -> &'static [&'static str] {
        match self {
            // ⚠ A new route has nothing to diff against, so it constrains
            // nothing and the other four conditions carry the decision.
            Self::NewRoute => &[],
            Self::PatchBump | Self::MajorBump => &["browser.version"],
        }
    }
}

impl core::fmt::Display for ChangeClass {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What the run knows and the corpus cannot say.
///
/// ⛔ **Every field here is a fact the caller measured**, and none of it is
/// defaulted. A body that filled in a run identifier it did not have would be a
/// fabricated provenance block in a project whose product is provenance.
///
/// ⚠ **Deserialised rather than assembled on a command line.** The workflow
/// writes this file from values the platform gives it, and a run identifier
/// typed into an argument list is a value somebody could type wrongly.
/// ⛔ There is no `Default`: every field is required, so a file missing one is
/// a refusal rather than a body with a blank in it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Run {
    /// The workflow that produced the capture.
    pub workflow: String,
    /// The run identifier, so a reader can open the artefacts.
    pub run_id: String,
    /// The runner image each platform's lane ran on, in the order the lanes
    /// are reported.
    pub images: Vec<(String, String)>,
    /// The harness that took the capture.
    pub harness: String,
    /// ⭐ The exact command that produced this, runnable locally.
    ///
    /// ⚠ It is also `CI-08`'s manual fallback, which is why the body carries it
    /// rather than pointing at a document.
    pub command: String,
    /// ⛔ Everything the run could not do, named.
    ///
    /// **A pull request that silently omits a field is worse than one that says
    /// it could not capture it.** An empty list is written as "nothing" rather
    /// than as an absent section.
    pub unavailable: Vec<String>,
    /// The validator's output, in full.
    pub validator_output: String,
    /// How many findings the validator reported.
    pub validator_findings: usize,
    /// Whether every generated format round-tripped in this run.
    pub formats_round_trip: bool,
}

/// One merge condition and whether it held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    /// What it asserts, as a reviewer reads it.
    pub name: &'static str,
    /// Whether it held.
    pub met: bool,
    /// What was measured, whichever way it went.
    pub detail: String,
}

/// The five conditions, in `CI-04`'s own order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditions(pub Vec<Condition>);

impl Conditions {
    /// Whether every condition held.
    ///
    /// ⛔ **All five, and the list is the only thing that answers.** A body that
    /// said "auto" from a field somebody set would be a merge decision nobody
    /// derived.
    #[must_use]
    pub fn met(&self) -> bool {
        self.0.iter().all(|c| c.met)
    }

    /// The conditions that did not hold, by name.
    #[must_use]
    pub fn failed(&self) -> Vec<&'static str> {
        self.0.iter().filter(|c| !c.met).map(|c| c.name).collect()
    }
}

/// One pull request, ready for a workflow to open or update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// ⭐ Deterministic, so a re-run updates the open request rather than
    /// opening a second one.
    pub branch: String,
    /// The title.
    pub title: String,
    /// The body, which is the whole of what a reviewer reads.
    pub body: String,
    /// The labels, for triage at a glance.
    pub labels: Vec<String>,
    /// What decided whether this may merge without a human.
    pub conditions: Conditions,
}

/// The schema major a branch name carries.
///
/// ⚠ **Derived from `b_ids_schema::SCHEMA_ID` rather than typed**, so a schema
/// major bump moves every branch name without anybody editing this.
fn schema_major() -> &'static str {
    b_ids_schema::SCHEMA_ID
        .rsplit_once('/')
        .map_or("1", |(_, major)| major)
}

/// The route a movement is about, as `browser/channel/platform`.
fn movement_route(movement: &Movement, after: &[Profile]) -> Option<String> {
    match movement {
        Movement::Advanced { route, .. } => Some(route.clone()),
        Movement::Added { id, .. } => after.iter().find(|p| p.id.as_str() == id).map(|p| {
            format!(
                "{}/{}/{}",
                p.browser.name.to_ascii_lowercase(),
                p.browser.channel.as_str(),
                p.platform_token().as_str()
            )
        }),
    }
}

/// The profile a movement ends at.
fn arrived<'a>(movement: &Movement, after: &'a [Profile]) -> Option<&'a Profile> {
    let route = movement_route(movement, after)?;
    let mut here: Vec<&Profile> = after
        .iter()
        .filter(|p| {
            format!(
                "{}/{}/{}",
                p.browser.name.to_ascii_lowercase(),
                p.browser.channel.as_str(),
                p.platform_token().as_str()
            ) == route
        })
        .collect();
    here.sort_by_key(|p| {
        p.browser
            .version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<u64>>()
    });
    here.last().copied()
}

/// The profile a movement started from, where there was one.
fn departed<'a>(movement: &Movement, before: &'a [Profile]) -> Option<&'a Profile> {
    let Movement::Advanced { route, from, .. } = movement else {
        return None;
    };
    before.iter().find(|p| {
        &p.browser.version == from
            && format!(
                "{}/{}/{}",
                p.browser.name.to_ascii_lowercase(),
                p.browser.channel.as_str(),
                p.platform_token().as_str()
            ) == *route
    })
}

/// Which class of change this movement is.
fn classify(movement: &Movement) -> ChangeClass {
    let Movement::Advanced { from, to, .. } = movement else {
        return ChangeClass::NewRoute;
    };
    let major = |v: &str| {
        v.split('.')
            .next()
            .and_then(|m| m.parse::<u64>().ok())
            .unwrap_or_default()
    };
    if major(from) == major(to) {
        ChangeClass::PatchBump
    } else {
        ChangeClass::MajorBump
    }
}

/// How many independent sources captured this build.
///
/// ⛔ **Computed from the published profiles rather than claimed.** `CI-04`
/// wants agreement across two independent sources, and a caller that asserted
/// it would be asserting something about the corpus that only the corpus knows.
///
/// ⚠ **A source is a platform and an operator together.** Two profiles of one
/// build on one platform taken by one operator are one source measured twice.
fn sources(profile: &Profile, after: &[Profile]) -> BTreeSet<String> {
    after
        .iter()
        .filter(|p| {
            p.browser.name == profile.browser.name && p.browser.version == profile.browser.version
        })
        .map(|p| {
            format!(
                "{} via {}",
                p.platform_token().as_str(),
                if p.captured.operator.is_empty() {
                    "an operator nobody recorded"
                } else {
                    p.captured.operator.as_str()
                }
            )
        })
        .collect()
}

/// Every field whose provenance got worse between two profiles.
///
/// ⛔ **`vendor` is the one this project cannot publish at all**, and
/// `unreproducible` appearing where it was not is a field that stopped being
/// measurable. Both are a human's decision.
fn provenance_regressions(before: &Profile, after: &Profile) -> Vec<String> {
    use b_ids_schema::ProvenanceKind::{Unreproducible, Vendor};
    let mut out = Vec::new();
    for (field, entry) in after.provenance.entries() {
        let was = before.provenance.get(field).map(|e| e.kind);
        match entry.kind {
            Vendor if was != Some(Vendor) => {
                out.push(format!("{field} became vendor"));
            }
            Unreproducible if was != Some(Unreproducible) => {
                out.push(format!("{field} became unreproducible"));
            }
            _ => {}
        }
    }
    out
}

/// Work out the five conditions for one movement.
fn conditions(
    movement: &Movement,
    class: ChangeClass,
    before: &[Profile],
    after: &[Profile],
    run: &Run,
) -> Conditions {
    let mut out = Vec::new();

    out.push(Condition {
        name: "the validator passes with no findings",
        met: run.validator_findings == 0,
        detail: format!("{} finding(s)", run.validator_findings),
    });

    let found = arrived(movement, after);
    let source_set = found.map(|p| sources(p, after)).unwrap_or_default();
    out.push(Condition {
        name: "the capture agrees across two independent sources",
        met: source_set.len() >= 2,
        detail: if source_set.is_empty() {
            "no source recorded".to_owned()
        } else {
            source_set.iter().cloned().collect::<Vec<_>>().join("; ")
        },
    });

    let regressions = match (departed(movement, before), found) {
        (Some(was), Some(now)) => provenance_regressions(was, now),
        _ => Vec::new(),
    };
    out.push(Condition {
        name: "no field regressed to vendor or became unreproducible",
        met: regressions.is_empty(),
        detail: if regressions.is_empty() {
            "none".to_owned()
        } else {
            regressions.join("; ")
        },
    });

    let predicted = class.predicted_fields();
    let unpredicted: Vec<String> = movement
        .fields()
        .into_iter()
        .filter(|f| !predicted.contains(&f.as_str()))
        .collect();
    out.push(Condition {
        name: "the diff touches only fields this change class predicts",
        met: unpredicted.is_empty(),
        detail: if unpredicted.is_empty() {
            format!("class {class}, predicting {}", predicted.join(", "))
        } else {
            format!("class {class}, unpredicted: {}", unpredicted.join(", "))
        },
    });

    out.push(Condition {
        name: "every generated format round-trips",
        met: run.formats_round_trip,
        detail: if run.formats_round_trip {
            "every format read back".to_owned()
        } else {
            "at least one format did not read back".to_owned()
        },
    });

    Conditions(out)
}

/// The body a reviewer reads without checking anything out.
fn body(
    movement: &Movement,
    class: ChangeClass,
    change: &Change,
    before: &[Profile],
    after: &[Profile],
    run: &Run,
    conditions: &Conditions,
) -> String {
    let mut out = String::new();

    out.push_str("## What changed\n\n");
    out.push_str(&format!("- {}\n", movement.headline()));
    for field in movement.fields() {
        out.push_str(&format!("  - {field}\n"));
    }
    out.push_str(&format!(
        "\nThe corpus holds {} profile(s) after this change.\n",
        change.profiles_after
    ));

    out.push_str("\n## The fields that differ\n\n```text\n");
    match (departed(movement, before), arrived(movement, after)) {
        (Some(was), Some(now)) => {
            let diff: Diff = b_ids_validator::diff(was, now);
            out.push_str(&render_diff(was, now, &diff));
        }
        // ⚠ A new route has nothing to compare against, and rendering every
        // field of a first profile as "changed" would be a diff against
        // nothing.
        _ => out.push_str("this route held no profile before, so there is nothing to diff\n"),
    }
    out.push_str("```\n");

    out.push_str("\n## Where this capture came from\n\n");
    if let Some(now) = arrived(movement, after) {
        out.push_str(&format!("- build: {}\n", now.browser.version));
        out.push_str(&format!("- channel: {}\n", now.browser.channel.as_str()));
        out.push_str(&format!("- platform: {}\n", now.platform_token().as_str()));
        out.push_str(&format!("- captured at: {}\n", now.captured.at));
        out.push_str(&format!("- trust: {}\n", now.captured.trust.as_str()));
        match &now.captured.acquisition {
            Some(acquisition) => out.push_str(&format!(
                "- acquisition: {} from {}, sha256 {}\n",
                acquisition.route,
                acquisition.url.as_deref().unwrap_or("no url recorded"),
                acquisition.sha256
            )),
            // ⛔ Named rather than omitted. A build nobody chose is a fact about
            // the capture and it belongs in front of the reviewer.
            None => out.push_str("- acquisition: nobody chose this build\n"),
        }
    }
    out.push_str(&format!("- harness: {}\n", run.harness));
    out.push_str(&format!("- workflow: {}\n", run.workflow));
    out.push_str(&format!("- run: {}\n", run.run_id));
    for (platform, image) in &run.images {
        out.push_str(&format!("- image on {platform}: {image}\n"));
    }

    out.push_str("\n## The validator\n\n```text\n");
    out.push_str(run.validator_output.trim_end());
    out.push_str("\n```\n");

    out.push_str("\n## What this run could not do\n\n");
    if run.unavailable.is_empty() {
        out.push_str("Nothing. Every step this run was asked for ran.\n");
    } else {
        for missing in &run.unavailable {
            out.push_str(&format!("- {missing}\n"));
        }
    }

    out.push_str("\n## Reproducing this\n\n```bash\n");
    out.push_str(run.command.trim_end());
    out.push_str("\n```\n");

    out.push_str("\n## Merging\n\n");
    out.push_str(&format!("Change class: {class}.\n\n"));
    for condition in &conditions.0 {
        out.push_str(&format!(
            "- {} {}: {}\n",
            if condition.met {
                "\u{2705}"
            } else {
                "\u{274c}"
            },
            condition.name,
            condition.detail
        ));
    }
    out.push_str(if conditions.met() {
        "\nEvery condition holds, so this may merge without a human.\n"
    } else {
        "\nAt least one condition does not hold, so this needs review.\n"
    });

    out
}

/// One pull request per route that moved.
///
/// ⛔ **A no-op change produces NOTHING.** Silence is the correct output for a
/// browser that did not change, and a bot that writes on a schedule trains
/// people to ignore it. `CI-04` states the same rule.
///
/// ⚠ **The early return below is the rule stated, not the rule enforced**, and
/// the mutation pass is what established the difference: removing it changes
/// nothing, because a change with no movement has nothing for the loop to
/// iterate. It is kept as the explicit statement of an invariant a reader
/// should not have to derive from a loop, and
/// `pull_request_a_no_op_change_opens_nothing_at_all` is what actually holds
/// the behaviour.
///
/// ⚠ **One request per route rather than one per run**, because the branch name
/// is per browser, channel, platform and schema major, and a re-run has to
/// update the open request rather than open a second one.
#[must_use]
pub fn requests(before: &[Profile], after: &[Profile], run: &Run) -> Vec<Request> {
    let change = model(before, after);
    if change.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for movement in &change.movements {
        let Some(route) = movement_route(movement, after) else {
            continue;
        };
        let class = classify(movement);
        let conditions = conditions(movement, class, before, after, run);
        let subject = route.split('/').next().unwrap_or("unknown").to_owned();
        out.push(Request {
            branch: format!("capture/{route}/v{}", schema_major()),
            title: format!("corpus: {}", movement.headline()),
            body: body(movement, class, &change, before, after, run, &conditions),
            labels: vec![
                format!("class:{class}"),
                format!(
                    "confidence:{}",
                    if conditions.met() { "auto" } else { "review" }
                ),
                format!("subject:{subject}"),
            ],
            conditions,
        });
    }
    out
}

/// Every fact a request's body is required to carry, in order.
///
/// ⛔ **This is what makes the body checkable rather than readable.** It is the
/// same shape as [`crate::notes::facts`], and the suite asserts that a body
/// contains every line of both: the model's facts, so the third renderer cannot
/// drift from the other two, and this module's own, so a section cannot go
/// missing quietly.
#[must_use]
pub fn required_lines(before: &[Profile], after: &[Profile], run: &Run) -> Vec<String> {
    let mut out = vec![
        "## What changed".to_owned(),
        "## The fields that differ".to_owned(),
        "## Where this capture came from".to_owned(),
        "## The validator".to_owned(),
        "## What this run could not do".to_owned(),
        "## Reproducing this".to_owned(),
        "## Merging".to_owned(),
        format!("- run: {}", run.run_id),
        format!("- harness: {}", run.harness),
    ];
    out.extend(facts(&model(before, after)));
    out.extend(run.unavailable.clone());
    out
}
