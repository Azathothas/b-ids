//! DRIVER-02. Read the version that is serving, not the one that is published.
//!
//! ⛔ Every test name starts with `versions`, because
//! `cargo test -p b-ids-driver versions` is the suite half of this entry.
//!
//! ⛔ **NOTHING HERE TOUCHES THE NETWORK.** The whole decision is pure: which
//! build a list of releases means, and what two sources disagreeing means. A
//! suite that fetched would be a suite that fails during somebody else's outage
//! and passes when the answer happens to be right, which is the opposite of
//! what a test is for. `b-ids-driver versions` is the driven half.
//!
//! ⚠ **The fixture is the inherited measurement**, from
//! `docs/inherited-claims.md` section 7: the highest known build at a rollout
//! fraction of 0.005, the build at fraction 1 two majors behind it, and the
//! automation index disagreeing by one patch component. A test written against
//! invented numbers could not have caught the defect this entry is about.

use b_ids_driver::versions::{
    Release, Source, choose, parse_for_testing, parse_releases, releases_url, report,
};

/// The inherited measurement, as the endpoint would have returned it.
fn inherited() -> Vec<Release> {
    vec![
        Release {
            version: "153.0.8010.12".to_owned(),
            fraction: Some(0.005),
        },
        Release {
            version: "152.0.7977.65".to_owned(),
            fraction: Some(1.0),
        },
        Release {
            version: "152.0.7977.9".to_owned(),
            fraction: Some(1.0),
        },
    ]
}

#[test]
fn versions_the_highest_at_full_rollout_wins_not_the_highest_known() {
    // ⛔ THE DEFECT THIS ENTRY EXISTS FOR. The naive query answers with the
    // first row; capturing it produces a correct fingerprint of a browser one
    // user in two hundred has.
    let chosen = choose(&inherited()).expect("a list with releases produces a choice");
    assert_eq!(chosen.version, "152.0.7977.65");
    assert_eq!(chosen.fraction, Some(1.0));
    assert_eq!(chosen.highest_known, "153.0.8010.12");
    assert_eq!(chosen.highest_fraction, Some(0.005));
}

#[test]
fn versions_full_rollout_builds_are_ordered_numerically_rather_than_as_text() {
    // ⚠ `152.0.7977.9` sorts after `152.0.7977.65` as text, so a chooser built
    // on string order picks an older build while looking correct. Both rows are
    // at fraction 1 in the fixture, so only the ordering can separate them.
    let chosen = choose(&inherited()).expect("a choice");
    assert_eq!(chosen.version, "152.0.7977.65");
}

#[test]
fn versions_with_nothing_at_full_rollout_the_highest_fraction_wins() {
    let staged = vec![
        Release {
            version: "153.0.8010.12".to_owned(),
            fraction: Some(0.005),
        },
        Release {
            version: "152.0.7977.65".to_owned(),
            fraction: Some(0.5),
        },
    ];
    let chosen = choose(&staged).expect("a choice");
    assert_eq!(chosen.version, "152.0.7977.65");
    assert_eq!(chosen.fraction, Some(0.5));
}

#[test]
fn versions_a_release_with_no_stated_fraction_is_absent_rather_than_zero() {
    // ⚠ Absent and zero are different facts: a release the endpoint said
    // nothing about is not a release it said nobody has.
    let payload = r#"{"releases":[{"version":"152.0.7977.65"},
                                  {"version":"151.0.7922.76","fraction":1.0}]}"#;
    let releases = parse_releases(payload).expect("it parses");
    assert_eq!(releases[0].fraction, None);
    assert_eq!(releases[1].fraction, Some(1.0));
    // And the one that can be shown to be at full rollout is the one chosen,
    // even though it is the lower build.
    let chosen = choose(&releases).expect("a choice");
    assert_eq!(chosen.version, "151.0.7922.76");
    assert_eq!(chosen.highest_known, "152.0.7977.65");
}

#[test]
fn versions_a_payload_that_is_not_releases_is_refused_with_a_reason() {
    assert!(parse_releases("not json").is_err());
    let why = parse_releases(r#"{"something":"else"}"#).expect_err("it is refused");
    assert!(why.contains("no releases array"), "{why}");
    let why = parse_releases(r#"{"releases":[{"no":"version"}]}"#).expect_err("it is refused");
    assert!(why.contains("no release with a version"), "{why}");
}

#[test]
fn versions_the_automation_index_channel_is_matched_case_insensitively() {
    // ⚠ The index capitalises its channel keys and this project's vocabulary
    // does not. Matched rather than rewritten: the caller's spelling is this
    // project's and the index's is the index's.
    let payload = r#"{"channels":{"Stable":{"version":"152.0.7977.64"},
                                  "Beta":{"version":"153.0.8010.12"}}}"#;
    assert_eq!(
        parse_for_testing(payload, "stable").expect("it parses"),
        "152.0.7977.64"
    );
    assert_eq!(
        parse_for_testing(payload, "beta").expect("it parses"),
        "153.0.8010.12"
    );
    let why = parse_for_testing(payload, "canary").expect_err("it is refused");
    assert!(why.contains("canary"), "{why}");
}

#[test]
fn versions_two_sources_that_disagree_are_a_finding_and_neither_is_preferred() {
    // ⚠ The inherited disagreement: two first-party sources, one patch
    // component apart. It is how the defect above was found in the first place.
    let report = report(Ok(inherited()), Ok("152.0.7977.64".to_owned()));
    assert!(report.disagreement);
    assert!(report.answered());
    assert_eq!(
        report.chosen.as_ref().map(|c| c.version.as_str()),
        Some("152.0.7977.65")
    );
    // ⛔ Both answers are kept. A report that dropped the loser would be a
    // report nobody could check the choice against.
    let named: Vec<&str> = report
        .answers
        .iter()
        .filter_map(|a| a.version.as_deref())
        .collect();
    assert_eq!(named, vec!["152.0.7977.65", "152.0.7977.64"]);
}

#[test]
fn versions_one_source_down_is_a_degraded_run_and_not_a_disagreement() {
    // ⛔ A check that reports a problem during somebody else's outage is a
    // check people switch off.
    let report = report(Ok(inherited()), Err("curl exited 6".to_owned()));
    assert!(!report.disagreement, "one silent source is not a dispute");
    assert!(report.answered(), "the other source still answered");
    assert_eq!(
        report.answers[1].error.as_deref(),
        Some("curl exited 6"),
        "why it did not answer is kept rather than collapsed into an absent version"
    );
}

#[test]
fn versions_no_source_answering_is_reported_as_having_discovered_nothing() {
    let report = report(Err("down".to_owned()), Err("down".to_owned()));
    assert!(!report.answered());
    assert!(report.chosen.is_none());
    assert!(!report.disagreement, "silence is not disagreement");
}

#[test]
fn versions_the_releases_url_carries_the_channel_and_the_still_serving_filter() {
    // ⚠ Without the filter the answer includes superseded builds, and the
    // highest of those is one nobody runs at all.
    let url = releases_url("beta");
    assert!(url.contains("/channels/beta/"), "{url}");
    assert!(url.contains("filter=endtime%3Dnone"), "{url}");
    assert_eq!(Source::Releases.url("beta"), url);
    assert!(
        Source::ChromeForTesting
            .url("beta")
            .contains("last-known-good-versions.json"),
        "the index carries every channel in one document"
    );
}
