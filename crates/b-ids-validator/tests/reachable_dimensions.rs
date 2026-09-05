//! VALID-03. A family the resolver cannot produce is data nobody can reach.
//!
//! The acceptance: a fixture corpus carrying a family the resolver has no
//! branch for fails with a message naming the family and both files.
//!
//! ⛔ Every test name starts with `reachable_dimensions`, because
//! `cargo test -p b-ids-validator reachable_dimensions` is the entry's
//! acceptance command.
//!
//! ⭐ **This test is what makes the check about the REAL resolver.** The library
//! takes the reachable list as an argument so that the validator stays pure
//! logic over the model; the coupling to `b_ids_driver::Family` lives here, in
//! one place, where a family added to the driver and not to the corpus, or the
//! reverse, shows up.

use b_ids_driver::Family;
use b_ids_validator::{Reachable, unreachable_dimensions};

/// What this project's own resolver and corpus can actually produce.
///
/// ⛔ **Read from the driver rather than typed.** A list of family names here
/// would be a second copy of `Family::all` with nothing checking that the two
/// agree, which is the defect this entry is about wearing a different hat.
fn this_project() -> Reachable {
    Reachable::new(
        Family::all().iter().map(|f| f.as_str().to_owned()),
        // ⚠ The channels and platforms are the schema's own vocabularies, and
        // they are read from the schema for the same reason.
        b_ids_schema::Channel::all()
            .iter()
            .map(|c| c.as_str().to_owned()),
        ["win64", "linux64", "macos-arm64", "macos-x86_64"],
    )
}

#[test]
fn reachable_dimensions_the_published_corpus_is_wholly_reachable() {
    // ⭐ THE POSITIVE CONTROL. A check that only ever reports a problem on a
    // fixture has not been shown to pass over anything real, and this reads the
    // corpus this repository actually publishes.
    // ⛔ RESOLVED, NEVER ASSUMED. This walked two directories up until PUB-13
    // moved corpus/ onto the source branch. ⚠ It would not have FAILED: the
    // read below prints "SKIPPED, no corpus" and carries on, so a positive
    // control would have quietly stopped controlling anything.
    let root = b_ids_schema::root::corpus_root_or_explain(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )));
    let dir = root.join("corpus").join("v1");
    let mut profiles = Vec::new();
    let mut walk = vec![dir.clone()];
    while let Some(next) = walk.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else {
            println!(
                "reachable_dimensions: SKIPPED, no corpus at {}",
                dir.display()
            );
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk.push(path);
            } else if path.extension().is_some_and(|e| e == "json")
                && path
                    .file_name()
                    .is_some_and(|n| n != "index.json" && n != "latest.json")
            {
                let text = std::fs::read_to_string(&path).expect("a published profile is readable");
                profiles.push(
                    serde_json::from_str::<b_ids_schema::Profile>(&text)
                        .unwrap_or_else(|e| panic!("{}: {e}", path.display())),
                );
            }
        }
    }
    assert!(
        !profiles.is_empty(),
        "the corpus holds at least one profile"
    );
    let unreachable = unreachable_dimensions(&profiles, &this_project());
    assert!(
        unreachable.is_empty(),
        "the published corpus carries {} dimension(s) no resolver branch can select: {}",
        unreachable.len(),
        unreachable
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    );
    println!(
        "reachable_dimensions: {} published profile(s), every dimension reachable",
        profiles.len()
    );
}

#[test]
fn reachable_dimensions_a_family_the_resolver_cannot_produce_is_reported() {
    // ⛔ THE GUARD, planted rather than argued. `safari` is a family this
    // resolver has no branch for, and the corpus could carry one tomorrow.
    // ⚠ THE STAND-IN WAS `firefox` UNTIL 2026-09-04, when the resolver learned
    // that family. An example chosen because it was impossible stops testing
    // anything on the day it becomes possible, and this one went red on exactly
    // the change that fixed the gap. docs/history/todo/corpus.md, CORPUS-02.
    let mut profile = b_ids_schema::fixture::profile();
    profile.browser.name = "Safari".to_owned();
    profile.id = profile.derived_id();

    let unreachable = unreachable_dimensions(std::slice::from_ref(&profile), &this_project());
    assert_eq!(unreachable.len(), 1, "{unreachable:?}");
    assert_eq!(unreachable[0].dimension, "browser");
    assert_eq!(unreachable[0].value, "safari");
    // ⛔ The message names the family AND the profile that carries it, because a
    // report saying only that something is unreachable sends a reader looking.
    let message = unreachable[0].to_string();
    assert!(message.contains("safari"), "{message}");
    assert!(message.contains(profile.id.as_str()), "{message}");
}

#[test]
fn reachable_dimensions_a_platform_with_no_route_is_reported_too() {
    // ⚠ THE POINT OF WALKING EVERY DIMENSION. A check that only looked at the
    // browser would pass over a corpus carrying a platform nothing runs on,
    // which is the same defect one field along.
    let profile = b_ids_schema::fixture::profile();
    let narrowed = Reachable::new(
        Family::all().iter().map(|f| f.as_str().to_owned()),
        b_ids_schema::Channel::all()
            .iter()
            .map(|c| c.as_str().to_owned()),
        // ⚠ The fixture is a `linux64` profile, so a resolver that could only
        // run on `win64` cannot produce it.
        ["win64"],
    );
    let unreachable = unreachable_dimensions(std::slice::from_ref(&profile), &narrowed);
    assert_eq!(unreachable.len(), 1, "{unreachable:?}");
    assert_eq!(unreachable[0].dimension, "platform");
    assert_eq!(unreachable[0].value, "linux64");
}

#[test]
fn reachable_dimensions_every_profile_carrying_it_is_named() {
    // ⚠ Grouped rather than repeated. A corpus with forty profiles of one
    // unreachable family should report one problem naming forty, not forty
    // problems naming one each.
    let mut first = b_ids_schema::fixture::profile();
    first.browser.name = "Safari".to_owned();
    first.id = first.derived_id();
    let mut second = first.clone();
    second.browser.version = "999.0.0.2".to_owned();
    second.id = second.derived_id();

    let unreachable = unreachable_dimensions(&[first.clone(), second.clone()], &this_project());
    assert_eq!(unreachable.len(), 1, "{unreachable:?}");
    assert_eq!(
        unreachable[0].profiles,
        vec![first.id.as_str().to_owned(), second.id.as_str().to_owned()]
    );
}

#[test]
fn reachable_dimensions_the_comparison_is_on_the_route_spelling() {
    // ⚠ `b_ids_corpus::route` lower-cases `browser.name`, and the driver reports
    // `Chrome` while a route reads `chrome`. A check comparing the two verbatim
    // would report every profile in the corpus as unreachable.
    let profile = b_ids_schema::fixture::profile();
    assert_eq!(profile.browser.name, "Chrome");
    let unreachable = unreachable_dimensions(std::slice::from_ref(&profile), &this_project());
    assert!(unreachable.is_empty(), "{unreachable:?}");
}
