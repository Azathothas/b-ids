//! `CI-04`. A scheduled run that finds a change opens a pull request.
//!
//! ⛔ Every test name starts with `pull_request`, because
//! `cargo test -p b-ids-corpus --test pull_request` is what runs this file
//! alone.

use b_ids_corpus::notes::facts;
use b_ids_corpus::pull_request::{Run, batch, requests, required_lines};
use b_ids_schema::{Os, Profile, ProvenanceEntry, ProvenanceKind};

/// A profile at one build, on the platform the fixture already carries.
fn at(version: &str, major: u32) -> Profile {
    let mut profile = b_ids_schema::fixture::profile();
    profile.browser.version = version.to_owned();
    profile.browser.major = major;
    profile.captured.operator = "a laptop".to_owned();
    profile.id = b_ids_schema::ProfileId::derive(
        &profile.browser.name,
        &profile.browser.version,
        &profile.platform_token(),
        profile.browser.channel,
    );
    profile
}

/// The same build on a second platform, which is a second independent source.
///
/// ⚠ The fixture's own platform is `linux64`, so this one is Windows. A helper
/// that set the platform the fixture already has would produce ONE route and a
/// test asserting two would have been asserting nothing.
fn on_second_platform(version: &str, major: u32) -> Profile {
    let mut profile = at(version, major);
    profile.platform.os = Os::Windows;
    profile.platform.distribution = None;
    profile.captured.operator = "a hosted runner".to_owned();
    profile.id = b_ids_schema::ProfileId::derive(
        &profile.browser.name,
        &profile.browser.version,
        &profile.platform_token(),
        profile.browser.channel,
    );
    profile
}

/// A run that did everything it was asked to do.
fn clean_run() -> Run {
    Run {
        workflow: "capture.yml".to_owned(),
        run_id: "33647839757".to_owned(),
        images: vec![("linux64".to_owned(), "ubuntu24/20260823.283".to_owned())],
        harness: "b-ids-harness 0.0.0".to_owned(),
        command: "cargo run -p b-ids-corpus -- add --captures c.ndjson --identity i.json"
            .to_owned(),
        unavailable: Vec::new(),
        validator_output: "corpus=validate profiles:2 findings:0 notcheckable:0".to_owned(),
        validator_findings: 0,
        formats_round_trip: true,
    }
}

#[test]
fn pull_request_a_body_carries_every_fact_the_model_holds() {
    // ⛔ THE THIRD RENDERER. A body with its own idea of what changed would
    // disagree with the release body the first time either moved, so every fact
    // the model holds has to appear here too.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![at("152.0.7977.75", 152)];
    let run = clean_run();
    let opened = requests(&before, &after, &run);
    assert_eq!(opened.len(), 1, "one route moved");

    let body = &opened[0].body;
    let model = facts(&b_ids_corpus::notes::model(&before, &after));
    assert!(!model.is_empty(), "the fixture must move something");
    for fact in &model {
        assert!(body.contains(fact), "the body omits {fact:?}");
    }
    for line in required_lines(&before, &after, &run) {
        assert!(body.contains(&line), "the body omits {line:?}");
    }
}

#[test]
fn pull_request_a_no_op_change_opens_nothing_at_all() {
    // ⛔ SILENCE IS THE CORRECT OUTPUT for "the browser did not change". A bot
    // that opens a request on a schedule trains people to ignore it.
    let corpus = vec![at("151.0.7922.76", 151)];
    assert!(requests(&corpus, &corpus, &clean_run()).is_empty());
    assert!(requests(&corpus, &[], &clean_run()).is_empty());
}

#[test]
fn pull_request_a_body_names_what_the_run_could_not_do() {
    // ⛔ A REQUEST THAT SILENTLY OMITS A FIELD IS WORSE THAN ONE THAT SAYS IT
    // COULD NOT CAPTURE IT. Both directions are asserted: the list when there
    // is one, and the sentence saying so when there is not.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![at("152.0.7977.75", 152)];

    let mut run = clean_run();
    run.unavailable = vec![
        "the macos lane had no runner".to_owned(),
        "no JA3 was computed: nothing here implements one".to_owned(),
    ];
    let opened = requests(&before, &after, &run);
    for missing in &run.unavailable {
        assert!(
            opened[0].body.contains(missing),
            "the body omits {missing:?}"
        );
    }

    let clean = requests(&before, &after, &clean_run());
    assert!(
        clean[0]
            .body
            .contains("Nothing. Every step this run was asked for ran."),
        "an empty list is said rather than left absent"
    );
}

#[test]
fn pull_request_two_runs_over_one_change_produce_identical_text() {
    // ⛔ A BODY THAT DIFFERS BETWEEN RUNS is one nobody can compare with the
    // previous one, which is what makes updating an open request useful.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![at("152.0.7977.75", 152)];
    let run = clean_run();
    assert_eq!(
        requests(&before, &after, &run),
        requests(&before, &after, &run)
    );
}

#[test]
fn pull_request_one_branch_per_run_carries_every_route_that_moved() {
    // ⛔ MEASURED, NOT ARGUED. The generator opened one branch per route and
    // the workflow pushed the SAME merged tree to each: run 33851238648 opened
    // five branches and all five resolved to tree
    // 97248d83821e, abbreviated because this project's secret scan refuses a
    // 40-character hex run in a tracked file. A title naming one route over a
    // diff carrying five is a title a reviewer cannot act on.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![
        at("152.0.7977.75", 152),
        on_second_platform("152.0.7977.75", 152),
    ];
    let opened = requests(&before, &after, &clean_run());
    assert_eq!(opened.len(), 2, "two routes moved");

    let one = batch(&before, &after, &clean_run()).expect("two routes moved, so one request");
    // ⭐ DETERMINISTIC IN THE RUN IDENTIFIER, so a re-run of the same run
    // updates its request rather than opening a second one.
    assert!(one.branch.starts_with("capture/run-"), "{}", one.branch);
    assert!(one.branch.ends_with("/v1"), "{}", one.branch);
    assert_eq!(
        one.branch,
        batch(&before, &after, &clean_run()).unwrap().branch,
        "the branch is not deterministic"
    );

    // ⛔ THE TITLE SAYS HOW MANY ROUTES THE DIFF CARRIES AND NAMES THEM, which
    // is the whole point of the change.
    assert!(one.title.contains("2 route(s)"), "{}", one.title);
    for profile in &after {
        let token = profile.platform_token().as_str().to_owned();
        assert!(
            one.title.contains(&token),
            "the title {:?} does not name {token}",
            one.title
        );
    }

    // ⛔ AND THE BODY CARRIES BOTH ROUTES' OWN SECTIONS, so the per-route body a
    // reviewer reads is composed rather than replaced.
    for request in &opened {
        assert!(
            one.body.contains(request.body.trim_end()),
            "the batch body dropped a route's body"
        );
    }

    // ⛔ ONE CONFIDENCE LABEL, NEVER TWO. A union of the routes' labels would
    // carry both confidence:auto and confidence:review on one request.
    let confidence: Vec<&String> = one
        .labels
        .iter()
        .filter(|label| label.starts_with("confidence:"))
        .collect();
    assert_eq!(confidence.len(), 1, "{:?}", one.labels);

    // ⛔ A NO-OP OPENS NOTHING HERE TOO, and the two must agree.
    assert!(batch(&before, &before, &clean_run()).is_none());
}

#[test]
fn pull_request_a_run_identifier_that_is_not_a_branch_name_is_made_into_one() {
    // ⛔ THE FIXTURE'S OWN RUN IDENTIFIER IS A SENTENCE WITH SPACES, and git
    // refuses a branch name carrying one at push time, which is the worst place
    // to find out. check-pr-body drives exactly that value.
    let mut run = clean_run();
    run.run_id = "a fixture run".to_owned();
    let after = vec![at("152.0.7977.75", 152)];
    let one = batch(&[], &after, &run).expect("a new route moved");
    assert_eq!(one.branch, "capture/run-a-fixture-run/v1");
    assert!(
        !one.branch.contains(' '),
        "a branch name may not carry a space: {}",
        one.branch
    );

    // ⚠ AND AN IDENTIFIER WITH NOTHING USABLE IN IT still produces a branch,
    // rather than `capture/run-/v1`, which git also refuses.
    run.run_id = "///".to_owned();
    let one = batch(&[], &after, &run).expect("a new route moved");
    assert_eq!(one.branch, "capture/run-unknown/v1");
}

#[test]
fn pull_request_the_merge_conditions_can_fail_and_say_which() {
    // ⛔ A GUARD WHOSE TEST HAS NEVER BEEN SEEN TO FAIL IS THEATRE. Each of the
    // five is taken down on its own, and the body has to name the one that
    // went.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![
        at("152.0.7977.75", 152),
        on_second_platform("152.0.7977.75", 152),
    ];

    let mut findings = clean_run();
    findings.validator_findings = 3;
    assert!(!requests(&before, &after, &findings)[0].conditions.met());
    assert!(
        requests(&before, &after, &findings)[0]
            .conditions
            .failed()
            .contains(&"the validator passes with no findings")
    );

    let mut formats = clean_run();
    formats.formats_round_trip = false;
    assert!(
        requests(&before, &after, &formats)[0]
            .conditions
            .failed()
            .contains(&"every generated format round-trips")
    );

    // ⚠ One source, because one platform captured the build.
    let one_source = vec![at("152.0.7977.75", 152)];
    assert!(
        requests(&before, &one_source, &clean_run())[0]
            .conditions
            .failed()
            .contains(&"the capture agrees across two independent sources")
    );

    // ⛔ A field the change class does not predict.
    let mut moved = at("152.0.7977.75", 152);
    moved.tls.cipher_suites = vec![0x1301, 0x1302];
    let unpredicted = vec![moved.clone(), on_second_platform("152.0.7977.75", 152)];
    assert!(
        requests(&before, &unpredicted, &clean_run())[0]
            .conditions
            .failed()
            .contains(&"the diff touches only fields this change class predicts")
    );

    // ⛔ A field that became unreproducible where it was not.
    let mut regressed = at("152.0.7977.75", 152);
    regressed.provenance.insert(
        "tls",
        ProvenanceEntry {
            kind: ProvenanceKind::Unreproducible,
            reason: Some("a planted regression".to_owned()),
        },
    );
    let regression = vec![regressed, on_second_platform("152.0.7977.75", 152)];
    assert!(
        requests(&before, &regression, &clean_run())[0]
            .conditions
            .failed()
            .contains(&"no field regressed to vendor or became unreproducible")
    );
}

#[test]
fn pull_request_every_condition_holding_is_reachable_rather_than_impossible() {
    // ⚠ THE OTHER HALF OF THE TEST ABOVE. A set of conditions that can only
    // fail is a merge gate nothing ever passes, which reads as caution and is
    // a feature nobody has.
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![
        at("152.0.7977.75", 152),
        on_second_platform("152.0.7977.75", 152),
    ];
    let opened = requests(&before, &after, &clean_run());
    for request in &opened {
        assert!(
            request.conditions.met(),
            "{}: {:?}",
            request.branch,
            request.conditions.failed()
        );
        assert!(request.labels.contains(&"confidence:auto".to_owned()));
        assert!(
            request
                .body
                .contains("Every condition holds, so this may merge without a human.")
        );
    }
}

#[test]
fn pull_request_the_labels_carry_the_class_the_confidence_and_the_subject() {
    let before = vec![at("151.0.7922.76", 151)];
    let after = vec![at("152.0.7977.75", 152)];
    let opened = requests(&before, &after, &clean_run());
    let labels = &opened[0].labels;
    assert_eq!(labels.len(), 3, "{labels:?}");
    assert!(
        labels.contains(&"class:major-bump".to_owned()),
        "{labels:?}"
    );
    assert!(
        labels.contains(&"confidence:review".to_owned()),
        "{labels:?}"
    );
    assert!(labels.contains(&"subject:chrome".to_owned()), "{labels:?}");

    // ⚠ A patch bump is a different class, so the classifier is not a constant.
    let patch = vec![at("151.0.7922.174", 151)];
    let opened = requests(&before, &patch, &clean_run());
    assert!(
        opened[0].labels.contains(&"class:patch-bump".to_owned()),
        "{:?}",
        opened[0].labels
    );

    // ⚠ And a route that held nothing is a third.
    let opened = requests(&[], &after, &clean_run());
    assert!(
        opened[0].labels.contains(&"class:new-route".to_owned()),
        "{:?}",
        opened[0].labels
    );
}

#[test]
fn pull_request_a_new_route_says_it_has_nothing_to_diff_against() {
    // ⛔ RENDERING EVERY FIELD OF A FIRST PROFILE AS "CHANGED" would be a diff
    // against nothing, which reads as a huge change and is not one.
    let after = vec![at("152.0.7977.75", 152)];
    let opened = requests(&[], &after, &clean_run());
    assert!(
        opened[0]
            .body
            .contains("this route held no profile before, so there is nothing to diff"),
        "{}",
        opened[0].body
    );
}
