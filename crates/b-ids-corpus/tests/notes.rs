//! PUB-08. One generator for the release body and the changelog.
//!
//! ⛔ Every test name starts with `notes`, because
//! `cargo test -p b-ids-corpus notes` is what runs this file alone.

use b_ids_corpus::notes::{Movement, changelog_entry, facts, model, release_body};
use b_ids_schema::Profile;

fn at(version: &str, major: u32) -> Profile {
    let mut profile = b_ids_schema::fixture::profile();
    profile.browser.version = version.to_owned();
    profile.browser.major = major;
    profile.id = b_ids_schema::ProfileId::derive(
        &profile.browser.name,
        &profile.browser.version,
        &profile.platform_token(),
        profile.browser.channel,
    );
    profile
}

#[test]
fn notes_the_two_outputs_agree_fact_for_fact() {
    // ⛔ THE ENTRY'S WHOLE POINT. Two documents written separately drift, and
    // the reader who trusts the wrong one is the one being careful. Both are
    // rendered from one model, and every fact the model holds appears in both.
    let before = vec![at("151.0.7922.76", 151)];
    let mut after_profile = at("152.0.7977.75", 152);
    after_profile.tls.cipher_suites = vec![0x1301, 0x1302];
    let after = vec![after_profile];

    let change = model(&before, &after);
    let body = release_body(&change);
    let entry = changelog_entry(&change);

    let facts = facts(&change);
    assert!(!facts.is_empty(), "the fixture must move something");
    for fact in &facts {
        assert!(body.contains(fact), "the release body omits {fact:?}");
        assert!(entry.contains(fact), "the changelog entry omits {fact:?}");
    }
}

#[test]
fn notes_a_no_op_change_renders_nothing_at_all() {
    // ⛔ SILENCE IS THE CORRECT OUTPUT for "the browser did not change". A bot
    // that writes on a schedule trains people to ignore it, and CI-04 states
    // the same rule for the pull request.
    let corpus = vec![at("151.0.7922.76", 151)];
    let change = model(&corpus, &corpus);
    assert!(change.is_empty());
    assert_eq!(release_body(&change), "");
    assert_eq!(changelog_entry(&change), "");
    assert!(facts(&change).is_empty());
}

#[test]
fn notes_a_version_move_is_field_level_rather_than_updated_a_browser() {
    // ⛔ "Updated a browser" is what a reader cannot act on. The fields come
    // from b_ids_validator::diff rather than from a second comparison here.
    let before = vec![at("151.0.7922.76", 151)];
    let mut moved = at("152.0.7977.75", 152);
    moved.tls.cipher_suites = vec![0x1301];
    let after = vec![moved];

    let change = model(&before, &after);
    let Some(Movement::Advanced { from, to, .. }) = change.movements.first() else {
        panic!("expected an advance: {:?}", change.movements);
    };
    assert_eq!(from, "151.0.7922.76");
    assert_eq!(to, "152.0.7977.75");
    let fields = change.movements[0].fields();
    assert!(
        fields.iter().any(|f| f.contains("cipher")),
        "the field that moved is named: {fields:?}"
    );
}

#[test]
fn notes_a_first_profile_at_a_route_is_an_addition_and_not_a_diff_against_nothing() {
    // ⚠ Listing every field of a first profile as "changed" would be a diff
    // against nothing, which is a number nobody measured.
    let change = model(&[], &[at("151.0.7922.76", 151)]);
    let Some(Movement::Added { version, .. }) = change.movements.first() else {
        panic!("expected an addition: {:?}", change.movements);
    };
    assert_eq!(version, "151.0.7922.76");
    assert!(change.movements[0].fields().is_empty());
}

#[test]
fn notes_two_runs_over_one_change_produce_identical_text() {
    // ⛔ A generator that read a clock or a map's iteration order would produce
    // a diff on every run, and a release body that diffs on every run is one
    // nobody can review.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![at("152.0.7977.75", 152)];
    let once = model(&before, &after);
    let twice = model(&before, &after);
    assert_eq!(release_body(&once), release_body(&twice));
    assert_eq!(changelog_entry(&once), changelog_entry(&twice));
}

#[test]
fn notes_two_outputs_generated_from_different_inputs_do_not_agree() {
    // ⛔ THE NEGATIVE CASE THE ACCEPTANCE NAMES. If the two were generated from
    // different queries the drift would be invisible, so this asserts that the
    // check comparing them can actually fail.
    let before = vec![at("151.0.7922.76", 151)];
    let after_a = vec![at("152.0.7977.75", 152)];
    let after_b = vec![at("152.0.7977.76", 152)];

    let body = release_body(&model(&before, &after_a));
    let entry = changelog_entry(&model(&before, &after_b));

    let facts_a = facts(&model(&before, &after_a));
    assert!(
        facts_a.iter().any(|fact| !entry.contains(fact)),
        "a changelog generated from a different input must NOT carry every fact \
         the body does: {body}"
    );
}
