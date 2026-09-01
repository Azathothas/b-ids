//! CORPUS-01. Content-addressed, append-only, never edited in place.
//!
//! ⛔ Every test name starts with `corpus`, because
//! `cargo test -p b-ids-corpus corpus` is the suite half of this entry's
//! acceptance. The other half is `scripts/common/check-corpus.sh`, which asks
//! the question this suite structurally cannot: whether a published profile was
//! ever modified after its first commit.

mod support;

use b_ids_corpus::{Store, profile_from, route};
use b_ids_schema::Trust;

use support::{Throwaway, cold_capture, identity};

#[test]
fn corpus_a_capture_becomes_a_profile_at_the_route_its_own_keys_derive() {
    let profile = profile_from(&cold_capture(), &identity()).expect("the cold capture converts");
    let route = route(&profile).expect("its keys produce a route");
    assert_eq!(
        b_ids_corpus::route::as_route(&route.profile),
        "corpus/v1/chrome/stable/win64/999.0.0.1.json"
    );
    assert_eq!(
        b_ids_corpus::route::as_route(&route.hello),
        "raw/v1/chrome/stable/win64/999.0.0.1.hello.hex"
    );
    assert!(
        profile.check().is_empty(),
        "a converted profile is well formed: {:?}",
        profile.check()
    );
}

#[test]
fn corpus_the_conversion_derives_the_major_rather_than_taking_one() {
    let profile = profile_from(&cold_capture(), &identity()).expect("the cold capture converts");
    assert_eq!(profile.browser.major, 999);
    assert_eq!(profile.id, profile.derived_id());
}

#[test]
fn corpus_the_capture_instant_reaches_the_profile_rather_than_the_clock_at_conversion() {
    let profile = profile_from(&cold_capture(), &identity()).expect("the cold capture converts");
    assert_eq!(profile.captured.at, "2026-09-01T04:05:06Z");
}

#[test]
fn corpus_the_trust_configuration_is_carried_into_the_profile() {
    let profile = profile_from(&cold_capture(), &identity()).expect("the cold capture converts");
    assert_eq!(profile.captured.trust, Trust::SpkiPin);
}

#[test]
fn corpus_a_terminated_capture_may_not_claim_no_handshake_completed() {
    // ⛔ THE MUTATION THIS CHECK EXISTS FOR. `captured.trust` defaults on the
    // way in, so a profile written before the field existed reads back as
    // not-applicable. On a capture that carried both a hello and HTTP/2 frames
    // that is a condition nobody recorded reading as one somebody did.
    let mut identity = identity();
    identity.trust = Trust::NotApplicable;
    let profile = profile_from(&cold_capture(), &identity).expect("it converts");
    let messages = profile
        .check()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        messages.contains("captured.trust"),
        "the profile is refused and the message names the field: {messages}"
    );
}

#[test]
fn corpus_the_throwaway_profile_path_is_redacted_and_the_substitution_is_recorded() {
    let profile = profile_from(&cold_capture(), &identity()).expect("the cold capture converts");
    assert!(
        profile
            .captured
            .switches
            .iter()
            .all(|s| !s.contains("throwaway-1234")),
        "no published switch names a directory on the machine that took the capture: {:?}",
        profile.captured.switches
    );
    assert_eq!(
        profile.captured.switches[0], "--user-data-dir=(throwaway)",
        "the switch is still recorded as having been passed"
    );
    let entry = profile
        .provenance
        .get("captured.switches")
        .expect("the substitution is recorded rather than hidden");
    assert_eq!(
        entry.reason.as_deref(),
        Some(b_ids_corpus::capture::SWITCH_REASON)
    );
}

#[test]
fn corpus_a_capture_that_never_reached_http2_is_refused_by_name() {
    let mut capture = cold_capture();
    capture.http2 = None;
    let refusals = profile_from(&capture, &identity()).expect_err("an abandoned socket is refused");
    let listed = refusals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        listed.contains("never reached HTTP/2"),
        "the refusal says what was missing: {listed}"
    );
}

#[test]
fn corpus_a_capture_with_no_hello_is_refused_by_name() {
    let mut capture = cold_capture();
    capture.tls = None;
    let refusals = profile_from(&capture, &identity()).expect_err("it is refused");
    let listed = refusals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(listed.contains("no ClientHello"), "{listed}");
}

#[test]
fn corpus_a_capture_with_no_instant_is_refused_rather_than_stamped_here() {
    let mut capture = cold_capture();
    capture.at = String::new();
    let refusals = profile_from(&capture, &identity()).expect_err("it is refused");
    let listed = refusals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(listed.contains("no instant"), "{listed}");
}

#[test]
fn corpus_a_credential_in_the_decrypted_first_message_refuses_the_profile() {
    // ⭐ THE THIRD DOOR INTO THE CREDENTIAL RULE, and the reason this entry
    // owns open question 1. A terminated capture holds the decrypted first
    // message, which is where a real browser's credentials actually appear.
    // The parsed header set drops them; these bytes would still spell one out.
    let mut capture = cold_capture();
    let mut plaintext = b_ids_harness::h2::PREFACE.to_vec();
    plaintext.extend_from_slice(b"\r\ncookie: session=not-a-real-value\r\n");
    let termination = capture.termination.as_mut().expect("it terminated");
    termination.plaintext_hex = b_ids_harness::hex(&plaintext);

    let profile = profile_from(&capture, &identity()).expect("it converts");
    let messages = profile
        .check()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        messages.contains("raw.connection_hex") && messages.contains("cookie"),
        "the refusal names the field and the header: {messages}"
    );

    // ⛔ And the store refuses it too, because a malformed profile is never
    // published. This is the assertion that would fail if the refusal were
    // only in a check nothing calls.
    let throwaway = Throwaway::new("credential");
    let store = Store::at(&throwaway.root);
    let why = store.add(&profile).expect_err("the store refuses it");
    assert!(why.contains("malformed"), "{why}");
}

#[test]
fn corpus_a_published_profile_is_never_overwritten() {
    let throwaway = Throwaway::new("append-only");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");

    let added = store.add(&profile).expect("the first write lands");
    assert!(added.profile.is_file());
    assert!(added.hello.is_file());

    let why = store
        .add(&profile)
        .expect_err("the second write at the same route is refused");
    assert!(
        why.contains("already published") && why.contains("supersedes"),
        "the refusal says what to do instead: {why}"
    );
}

#[test]
fn corpus_the_hello_sidecar_carries_no_trailing_newline() {
    // ⚠ Measured in somebody else's published dataset: a single-value route
    // ending in a newline makes every consumer strip one.
    // docs/reference-sweeps/usable.md section 9.
    let throwaway = Throwaway::new("newline");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    let added = store.add(&profile).expect("the write lands");
    let text = std::fs::read_to_string(&added.hello).expect("the sidecar is readable");
    assert!(!text.ends_with('\n'), "it carries exactly one value");
    assert_eq!(text, profile.raw.client_hello_hex.unwrap_or_default());
}

#[test]
fn corpus_a_supersedes_naming_nothing_is_refused() {
    let throwaway = Throwaway::new("dangling");
    let store = Store::at(&throwaway.root);
    let mut profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    profile.supersedes = Some("chrome/998.0.0.1/win64/stable".to_owned());
    let why = store
        .add(&profile)
        .expect_err("a dangling correction is refused");
    assert!(why.contains("not in this corpus"), "{why}");
}

#[test]
fn corpus_verify_is_clean_over_a_corpus_the_writer_produced() {
    let throwaway = Throwaway::new("verify-clean");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    store.add(&profile).expect("the write lands");
    store.write_index().expect("the index is written");
    let problems = store.verify().expect("the corpus is readable");
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn corpus_verify_catches_a_sidecar_that_no_longer_matches_its_profile() {
    // ⛔ A value in two places needs a check that they agree, or the copy a
    // reader trusts is the wrong one.
    let throwaway = Throwaway::new("sidecar-drift");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    let added = store.add(&profile).expect("the write lands");
    store.write_index().expect("the index is written");

    std::fs::write(&added.hello, "00").expect("the sidecar is writable");
    let problems = store.verify().expect("the corpus is readable");
    assert!(
        problems.iter().any(|p| p.contains("raw.client_hello_hex")),
        "{problems:?}"
    );
}

#[test]
fn corpus_verify_catches_an_index_that_does_not_match_the_tree() {
    let throwaway = Throwaway::new("index-drift");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    store.add(&profile).expect("the write lands");
    store.write_index().expect("the index is written");

    let index = store.corpus_dir().join(b_ids_corpus::store::INDEX_FILE);
    std::fs::write(
        &index,
        "{\"schema\":\"corpus-index/1\",\"layout\":\"v1\",\"profiles\":[]}\n",
    )
    .expect("the index is writable");
    let problems = store.verify().expect("the corpus is readable");
    assert!(
        problems
            .iter()
            .any(|p| p.contains("does not match what the corpus derives to")),
        "{problems:?}"
    );
}

#[test]
fn corpus_verify_catches_a_profile_published_under_a_name_that_is_not_its_name() {
    let throwaway = Throwaway::new("wrong-route");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    let added = store.add(&profile).expect("the write lands");
    store.write_index().expect("the index is written");

    let moved = added
        .profile
        .parent()
        .expect("it has a parent")
        .join("111.0.0.1.json");
    std::fs::rename(&added.profile, &moved).expect("it is movable");
    let problems = store.verify().expect("the corpus is readable");
    assert!(
        problems.iter().any(|p| p.contains("not its name")),
        "{problems:?}"
    );
}

#[test]
fn corpus_the_index_carries_a_content_address_for_every_published_file() {
    // ⚠ A version number that does not pin its bytes pins nothing: two copies
    // of one published dataset at one version held a different number of
    // entries. docs/reference-sweeps/usable.md section 9.
    let throwaway = Throwaway::new("content-address");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    let added = store.add(&profile).expect("the write lands");

    let index = store.index().expect("the index derives");
    assert_eq!(index.profiles.len(), 1);
    let entry = &index.profiles[0];
    let bytes = std::fs::read(&added.profile).expect("the profile is readable");
    let expected = b_ids_harness::hex(&b_ids_harness::sha256(&bytes));
    assert_eq!(entry.profile.sha256, expected);
    assert_eq!(entry.profile.bytes, bytes.len());
    assert_eq!(entry.trust, "spki-pin");
}

#[test]
fn corpus_the_latest_pointer_orders_builds_numerically_rather_than_as_text() {
    // ⚠ `152.0.7977.9` sorts after `152.0.7977.64` as text, and a pointer built
    // that way hands a consumer an older build while looking correct.
    let throwaway = Throwaway::new("latest");
    let store = Store::at(&throwaway.root);

    for version in ["152.0.7977.9", "152.0.7977.64"] {
        let mut identity = identity();
        identity.version = version.to_owned();
        let profile = profile_from(&cold_capture(), &identity).expect("it converts");
        store.add(&profile).expect("the write lands");
    }
    let pointers = store.pointers().expect("the pointers derive");
    assert_eq!(
        pointers
            .latest
            .get("chrome/stable/win64")
            .map(String::as_str),
        Some("corpus/v1/chrome/stable/win64/152.0.7977.64.json")
    );
}

#[test]
fn corpus_verify_asserts_every_half_is_reproducible_from_the_bytes_beside_it() {
    // ⭐ This is what makes the raw block a backstop rather than a gesture. A
    // raw block nobody has re-parsed is a claim, and the mutation below is what
    // proves the assertion can fail: the recorded halves are left alone and the
    // bytes they were read from are replaced.
    let throwaway = Throwaway::new("rebuild");
    let store = Store::at(&throwaway.root);
    let profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    store.add(&profile).expect("the write lands");
    store.write_index().expect("the index is written");
    assert!(
        store.verify().expect("readable").is_empty(),
        "the writer's own output rebuilds"
    );

    let path = store
        .corpus_dir()
        .join("chrome/stable/win64/999.0.0.1.json");
    let text = std::fs::read_to_string(&path).expect("the profile is readable");
    let mut written: b_ids_schema::Profile = serde_json::from_str(&text).expect("it parses");
    written.raw.http2_frames_hex = vec!["00".to_owned()];
    written.raw.settings_frame_hex = Some("00".to_owned());
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&written).expect("serialises"),
    )
    .expect("the profile is writable");
    // ⚠ The sidecar and the index now disagree too, which is the point: one
    // edit to a published file breaks several assertions at once. This asserts
    // the rebuild one specifically.
    let problems = store.verify().expect("the corpus is readable");
    assert!(
        problems
            .iter()
            .any(|p| p.contains("raw.http2_frames_hex re-parses to an HTTP/2 half")),
        "{problems:?}"
    );
    // ⚠ And the message names the ROUTE rather than a path on this machine. A
    // finding nobody else can locate is a finding nobody acts on.
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("corpus/v1/chrome/stable/win64/999.0.0.1.json:")),
        "{problems:?}"
    );
}

#[test]
fn corpus_a_browser_name_that_would_escape_the_root_has_no_route() {
    let mut profile = profile_from(&cold_capture(), &identity()).expect("it converts");
    profile.browser.name = "..".to_owned();
    let why = route(&profile).expect_err("it is refused rather than sanitised");
    assert_eq!(why.field, "browser.name");
}

#[test]
fn corpus_an_absent_corpus_is_not_an_empty_one() {
    let throwaway = Throwaway::new("absent");
    let store = Store::at(&throwaway.root);
    assert!(!store.exists(), "nothing has been written here");
    assert!(
        store
            .profile_paths()
            .expect("a missing tree reads as none")
            .is_empty(),
        "and reading it is not an error"
    );
}
