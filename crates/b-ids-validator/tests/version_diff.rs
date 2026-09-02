//! VALID-06. Diffs between adjacent versions.
//!
//! The acceptance: the diff of two profiles differing in exactly one header
//! position names that header and its two positions, and reports no other
//! change.
//!
//! ⛔ Every test name starts with `version_diff`.

use b_ids_schema::http::{HeaderSet, ValuePolicy, Variant};
use b_ids_validator::{diff, render_diff};

/// Two profiles of one build whose header order differs by one move.
fn moved_header() -> (b_ids_schema::Profile, b_ids_schema::Profile) {
    let names = ["sec-ch-ua", "user-agent", "accept", "accept-language"];
    let moved = ["sec-ch-ua", "accept", "user-agent", "accept-language"];
    let set = |order: &[&str]| {
        HeaderSet::record(
            Variant::Navigate,
            order.iter().map(|n| ((*n).to_owned(), String::new())),
            ValuePolicy::NamesOnly,
        )
    };
    let mut before = b_ids_schema::fixture::profile();
    before.http.variants = vec![set(&names)];
    let mut after = before.clone();
    after.http.variants = vec![set(&moved)];
    (before, after)
}

#[test]
fn version_diff_names_the_header_and_its_two_positions() {
    // ⭐ THE ENTRY'S OWN CASE, and the reason it is worth having: a header that
    // moved is the kind of change only a capture finds, and "the digest changed"
    // is not something a reader can act on.
    let (before, after) = moved_header();
    let report = diff(&before, &after);

    let moved: Vec<String> = report.changes.iter().map(ToString::to_string).collect();
    assert!(
        moved
            .iter()
            .any(|c| c == "http.headers.user-agent: position 1 -> position 2"),
        "{moved:?}"
    );
    assert!(
        moved
            .iter()
            .any(|c| c == "http.headers.accept: position 2 -> position 1"),
        "{moved:?}"
    );
    // ⛔ AND NOTHING ELSE. A diff that also reported the extension order or a
    // GREASE draw would bury the one change a reader is looking for.
    assert_eq!(report.changes.len(), 2, "{moved:?}");
}

#[test]
fn version_diff_of_a_profile_with_itself_is_empty() {
    // ⚠ The positive control. A diff that reported something here would report
    // something on every pair, which is a diff nobody reads twice.
    let profile = b_ids_schema::fixture::profile();
    let report = diff(&profile, &profile);
    assert!(report.is_empty(), "{:?}", report.changes);
    assert!(report.isolates_the_version());
    assert!(render_diff(&profile, &profile, &report).contains("no field differs"));
}

#[test]
fn version_diff_says_when_more_than_the_version_moved() {
    // ⛔ THE MUST-NOT, HELD BY A TEST. Two captures that differ in version AND
    // platform cannot isolate anything, and a diff rendered without saying so
    // invites a reader to attribute every line to the version.
    let before = b_ids_schema::fixture::profile();
    let mut after = before.clone();
    after.platform.os = b_ids_schema::Os::Windows;
    after.id = after.derived_id();

    let report = diff(&before, &after);
    assert!(!report.isolates_the_version());
    assert!(
        report
            .uncontrolled
            .iter()
            .any(|u| u.condition == "platform"),
        "{:?}",
        report.uncontrolled
    );

    // ⛔ And the warning is rendered ABOVE the changes: a reader who sees it
    // after the list has already attributed the list.
    let rendered = render_diff(&before, &after, &report);
    let warning = rendered.find("do not differ only in version");
    let changes = rendered.find("field(s) differ");
    match (warning, changes) {
        (Some(w), Some(c)) => assert!(w < c, "{rendered}"),
        (Some(_), None) => {}
        _ => panic!("the warning is missing: {rendered}"),
    }
}

#[test]
fn version_diff_reports_a_header_that_appeared_or_left() {
    let (before, _) = moved_header();
    let mut after = before.clone();
    after.http.variants = vec![HeaderSet::record(
        Variant::Navigate,
        [
            ("sec-ch-ua".to_owned(), String::new()),
            ("user-agent".to_owned(), String::new()),
            ("accept".to_owned(), String::new()),
            ("accept-language".to_owned(), String::new()),
            ("priority".to_owned(), String::new()),
        ],
        ValuePolicy::NamesOnly,
    )];
    let report = diff(&before, &after);
    let text: Vec<String> = report.changes.iter().map(ToString::to_string).collect();
    assert!(
        text.iter()
            .any(|c| c == "http.headers.priority: absent -> position 4"),
        "{text:?}"
    );

    // ⚠ And the other direction, which is the one a client author cares about:
    // a header that stopped being sent.
    let back = diff(&after, &before);
    let text: Vec<String> = back.changes.iter().map(ToString::to_string).collect();
    assert!(
        text.iter()
            .any(|c| c == "http.headers.priority: position 4 -> absent"),
        "{text:?}"
    );
}

#[test]
fn version_diff_ignores_a_grease_draw() {
    // ⛔ GREASE IS DRAWN PER CONNECTION, so a diff that reported it would report
    // a draw as a change on every pair of captures ever taken.
    let before = b_ids_schema::fixture::profile();
    let mut after = before.clone();
    for extension in &mut after.tls.extensions {
        if b_ids_schema::tls::is_grease_value(extension.codepoint) {
            extension.codepoint = 0x8a8a;
        }
    }
    after.tls.grease.values = after
        .tls
        .extensions
        .iter()
        .filter(|e| b_ids_schema::tls::is_grease_value(e.codepoint))
        .map(|e| e.codepoint)
        .collect();
    let report = diff(&before, &after);
    assert!(report.is_empty(), "{:?}", report.changes);
}
