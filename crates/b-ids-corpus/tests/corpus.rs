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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity())
        .expect("the cold capture converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity())
        .expect("the cold capture converts");
    assert_eq!(profile.browser.major, 999);
    assert_eq!(profile.id, profile.derived_id());
}

#[test]
fn corpus_the_capture_instant_reaches_the_profile_rather_than_the_clock_at_conversion() {
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity())
        .expect("the cold capture converts");
    assert_eq!(profile.captured.at, "2026-09-01T04:05:06Z");
}

#[test]
fn corpus_the_trust_configuration_is_carried_into_the_profile() {
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity())
        .expect("the cold capture converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity())
        .expect("the cold capture converts");
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
    let refusals =
        profile_from(&capture, &capture, &identity()).expect_err("an abandoned socket is refused");
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
    let refusals = profile_from(&capture, &capture, &identity()).expect_err("it is refused");
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
    let refusals = profile_from(&capture, &capture, &identity()).expect_err("it is refused");
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

    let profile = profile_from(&capture, &capture, &identity()).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");

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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
    let added = store.add(&profile).expect("the write lands");
    let text = std::fs::read_to_string(&added.hello).expect("the sidecar is readable");
    assert!(!text.ends_with('\n'), "it carries exactly one value");
    assert_eq!(text, profile.raw.client_hello_hex.unwrap_or_default());
}

#[test]
fn corpus_a_supersedes_naming_nothing_is_refused() {
    let throwaway = Throwaway::new("dangling");
    let store = Store::at(&throwaway.root);
    let mut profile =
        profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
        let profile =
            profile_from(&cold_capture(), &cold_capture(), &identity).expect("it converts");
        store.add(&profile).expect("the write lands");
    }
    let pointers = store.pointers().expect("the pointers derive");
    assert_eq!(
        pointers.latest.get("chrome/win64").map(String::as_str),
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
    let profile = profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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
fn corpus_latest_names_the_newest_stable_and_never_a_pre_release() {
    // ⛔ CORPUS-03. A consumer following a pointer called `latest` must never be
    // handed a pre-release build, and the beta here is a HIGHER build than the
    // stable, so a pointer that took the newest of everything would take it.
    let throwaway = Throwaway::new("latest-stable");
    let store = Store::at(&throwaway.root);
    for (version, channel) in [
        ("152.0.7977.64", b_ids_schema::Channel::Stable),
        ("153.0.8010.12", b_ids_schema::Channel::Beta),
    ] {
        let mut identity = identity();
        identity.version = version.to_owned();
        identity.channel = channel;
        let profile =
            profile_from(&cold_capture(), &cold_capture(), &identity).expect("it converts");
        store.add(&profile).expect("the write lands");
    }
    store.write_index().expect("the index is written");

    let pointers = store.pointers().expect("the pointers derive");
    assert_eq!(
        pointers.latest.get("chrome/win64").map(String::as_str),
        Some("corpus/v1/chrome/stable/win64/152.0.7977.64.json"),
        "latest is the newest STABLE, not the newest build"
    );
    // ⭐ And the pre-release is published beside it, clearly labelled, because
    // capturing beta is how this project gets ahead of a release.
    assert_eq!(
        pointers
            .per_channel
            .get("chrome/beta/win64")
            .map(String::as_str),
        Some("corpus/v1/chrome/beta/win64/153.0.8010.12.json")
    );
    assert!(store.verify().expect("readable").is_empty());
    assert!(
        store
            .latest_that_is_not_stable()
            .expect("readable")
            .is_empty()
    );
}

#[test]
fn corpus_a_hand_edited_latest_pointing_at_a_pre_release_is_refused_by_name() {
    // ⛔ THE MUTATION. The derivation cannot produce this entry, so the only way
    // it exists is somebody editing the published file. That is exactly what a
    // consumer would then follow.
    let throwaway = Throwaway::new("latest-edited");
    let store = Store::at(&throwaway.root);
    for (version, channel) in [
        ("152.0.7977.64", b_ids_schema::Channel::Stable),
        ("153.0.8010.12", b_ids_schema::Channel::Beta),
    ] {
        let mut identity = identity();
        identity.version = version.to_owned();
        identity.channel = channel;
        let profile =
            profile_from(&cold_capture(), &cold_capture(), &identity).expect("it converts");
        store.add(&profile).expect("the write lands");
    }
    store.write_index().expect("the index is written");

    // ⚠ Edited through the type rather than by string replacement, so the test
    // does not silently stop editing anything the day the serialisation changes
    // its whitespace.
    let path = store.corpus_dir().join(b_ids_corpus::store::POINTER_FILE);
    let text = std::fs::read_to_string(&path).expect("readable");
    let mut pointers: b_ids_corpus::Pointers = serde_json::from_str(&text).expect("it parses");
    let previous = pointers.latest.insert(
        "chrome/win64".to_owned(),
        "corpus/v1/chrome/beta/win64/153.0.8010.12.json".to_owned(),
    );
    assert_eq!(
        previous.as_deref(),
        Some("corpus/v1/chrome/stable/win64/152.0.7977.64.json"),
        "the edit landed on the entry the derivation produced"
    );
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&pointers).expect("serialises"),
    )
    .expect("writable");

    let problems = store.latest_that_is_not_stable().expect("readable");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        problems[0].contains("chrome/win64") && problems[0].contains("beta"),
        "the message names the route and the channel: {}",
        problems[0]
    );
    // ⛔ And the ordinary corpus check refuses it too, because the written file
    // no longer matches what the tree derives to.
    assert!(
        store
            .verify()
            .expect("readable")
            .iter()
            .any(|p| p.contains("latest.json")),
        "a hand-edited pointer file is refused by the derivation comparison as well"
    );
}

#[test]
fn corpus_a_browser_name_that_would_escape_the_root_has_no_route() {
    let mut profile =
        profile_from(&cold_capture(), &cold_capture(), &identity()).expect("it converts");
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

// -- HARNESS-15: two halves, two connections, and the profile says so --------

/// ⭐ The profile records which connection each half came from.
#[test]
fn corpus_records_the_connection_each_half_came_from() {
    // ⛔ THE RUNNER'S SHAPE. The cold hello arrived on a connection that
    // reached no HTTP/2, and the frames came from a later one. Before
    // HARNESS-15 this combination could not produce a profile at all.
    let mut tls_from = cold_capture();
    tls_from.connection = 1;
    tls_from.http2 = None;
    tls_from.termination = None;

    let mut http2_from = cold_capture();
    http2_from.connection = 3;

    let profile = profile_from(&tls_from, &http2_from, &identity()).expect("it converts");
    let connections = profile
        .captured
        .connections
        .expect("a profile written today records both connections");
    assert_eq!(connections.tls, 1);
    assert_eq!(connections.http2, 3);
    assert!(
        !connections.one_connection(),
        "two sockets of one navigation is a condition of the measurement"
    );
}

/// ⚠ The ordinary case still records one connection twice rather than nothing.
///
/// ⛔ **Absent means "written before the field existed", not "one
/// connection".** A reader has to be able to tell those apart, so the ordinary
/// case says so explicitly.
#[test]
fn corpus_records_one_connection_twice_when_one_carried_both_halves() {
    let capture = cold_capture();
    let profile = profile_from(&capture, &capture, &identity()).expect("it converts");
    let connections = profile.captured.connections.expect("recorded");
    assert_eq!(connections.tls, capture.connection);
    assert_eq!(connections.http2, capture.connection);
    assert!(connections.one_connection());
}

/// ⛔ The TLS half and the raw hello come from the TLS connection, and the
/// frames from the HTTP/2 one.
///
/// ⚠ **This is the assertion that would catch a half taken from the wrong
/// capture**, which a test that passed one capture twice cannot see at all.
#[test]
fn corpus_takes_each_half_from_its_own_connection() {
    let mut tls_from = cold_capture();
    tls_from.connection = 1;
    tls_from.at = "2026-09-01T00:00:01Z".to_owned();
    tls_from.raw_hex = tls_from.raw_hex.to_uppercase();

    let mut http2_from = cold_capture();
    http2_from.connection = 7;
    http2_from.at = "2026-09-01T00:00:09Z".to_owned();

    let profile = profile_from(&tls_from, &http2_from, &identity()).expect("it converts");

    // ⛔ The bytes came from the TLS connection, not from the HTTP/2 one.
    assert_eq!(
        profile.raw.client_hello_hex.as_deref(),
        Some(tls_from.raw_hex.as_str()),
        "the ClientHello bytes belong to the connection that sent the hello"
    );
    // ⚠ The instant is the TLS connection's, which is stated in profile_from's
    // own documentation rather than left for a reader to discover.
    assert_eq!(profile.captured.at, "2026-09-01T00:00:01Z");
    // ⛔ And the frame bytes came from the HTTP/2 connection.
    assert_eq!(
        profile.raw.connection_hex,
        http2_from
            .termination
            .as_ref()
            .map(|t| t.plaintext_hex.clone()),
        "the decrypted first message belongs to the connection that carried it"
    );
}

// -- CORPUS-06. The headless normalisation, wired where DRIVER-03 said -------
//
// ⛔ Every test name starts with `headless`, because
// `cargo test -p b-ids-corpus headless` is this entry's acceptance.

/// A capture whose header set carries values, with `user-agent` set to `value`.
///
/// ⚠ **The committed fixture is parsed names-only**, which is the default and
/// what the published corpus mostly holds. A capture that turned values on is
/// what the normalisation has anything to act on, so this builds one rather
/// than pretending the default case exercises it.
fn capture_announcing(value: &str) -> b_ids_harness::Capture {
    let mut capture = cold_capture();
    let http2 = capture.http2.as_mut().expect("the fixture reached HTTP/2");
    http2.headers.retain(|h| h.name != "user-agent");
    http2.headers.push(b_ids_harness::hpack::HeaderRecord {
        name: "user-agent".to_owned(),
        value: Some(value.to_owned()),
        name_huffman: Some(false),
        value_huffman: Some(false),
        indexing: b_ids_harness::hpack::Indexing::Incremental,
    });
    capture
}

/// The identity above, launched without a window.
fn headless_identity() -> b_ids_corpus::Identity {
    let mut identity = identity();
    identity.switches.push("--headless=new".to_owned());
    identity
}

/// What the profile publishes for one header, if anything.
fn header_value(profile: &b_ids_schema::Profile, name: &str) -> Option<String> {
    profile
        .http
        .variants
        .first()?
        .headers
        .iter()
        .find(|h| h.name == name)
        .and_then(|h| h.value.clone())
}

const HEADLESS_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     HeadlessChrome/999.0.0.0 Safari/537.36";

const WINDOWED_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/999.0.0.0 Safari/537.36";

#[test]
fn headless_a_capture_taken_without_a_window_publishes_the_windowed_product_token() {
    let capture = capture_announcing(HEADLESS_UA);
    let profile = profile_from(&capture, &capture, &headless_identity())
        .expect("the headless capture converts");
    assert_eq!(
        header_value(&profile, "user-agent").as_deref(),
        Some(WINDOWED_UA),
        "the product token is the one a windowed build announces"
    );
}

#[test]
fn headless_the_substitution_is_recorded_with_its_reason() {
    // ⛔ THE MUTATION THIS TEST EXISTS FOR. A normalisation with no provenance
    // entry beside it is a rewritten capture, which is the one thing this
    // project cannot ship. Deleting the `insert` in b_ids_driver::headless
    // leaves the assertion above green and this one red.
    let capture = capture_announcing(HEADLESS_UA);
    let profile = profile_from(&capture, &capture, &headless_identity())
        .expect("the headless capture converts");
    let entry = profile
        .provenance
        .get("http.headers.user-agent")
        .expect("the rewritten field carries a provenance entry");
    assert_eq!(entry.kind, b_ids_schema::ProvenanceKind::Substituted);
    assert_eq!(
        entry.reason.as_deref(),
        Some(b_ids_driver::headless::REASON)
    );
}

#[test]
fn headless_a_windowed_launch_is_left_alone_even_where_the_value_carries_the_token() {
    // ⛔ THE GATE IS THE LAUNCH, NOT THE VALUE. A build that announced a
    // headless token with a window on screen is announcing its own value, and
    // rewriting one this project did not cause is the failure it is about.
    let capture = capture_announcing(HEADLESS_UA);
    let profile =
        profile_from(&capture, &capture, &identity()).expect("the windowed capture converts");
    assert_eq!(
        header_value(&profile, "user-agent").as_deref(),
        Some(HEADLESS_UA),
        "a windowed capture is published exactly as it arrived"
    );
    assert!(
        profile.provenance.get("http.headers.user-agent").is_none(),
        "and nothing claims a substitution that did not happen"
    );
}

#[test]
fn headless_a_windowed_value_from_a_headless_launch_is_not_marked_substituted() {
    // ⚠ The capture-with-values path exists on a lane that turned values on;
    // a headless launch whose User-Agent carries no headless token is a build
    // that did not rewrite it, and marking that field substituted would be a
    // claim about a value nobody changed.
    let capture = capture_announcing(WINDOWED_UA);
    let profile = profile_from(&capture, &capture, &headless_identity())
        .expect("the headless capture converts");
    assert_eq!(
        header_value(&profile, "user-agent").as_deref(),
        Some(WINDOWED_UA)
    );
    assert!(profile.provenance.get("http.headers.user-agent").is_none());
}

#[test]
fn headless_the_switch_that_says_so_is_published_beside_the_substitution() {
    // ⭐ A consumer can see the condition the substitution was made under,
    // because the launch mode reaches `captured.switches` unredacted.
    let capture = capture_announcing(HEADLESS_UA);
    let profile = profile_from(&capture, &capture, &headless_identity())
        .expect("the headless capture converts");
    assert!(
        profile
            .captured
            .switches
            .iter()
            .any(|s| s == "--headless=new"),
        "the switches are {:?}",
        profile.captured.switches
    );
}
