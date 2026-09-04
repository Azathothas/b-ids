//! `LIB-01`. The crate that hands a program a profile.
//!
//! ⛔ **Every assertion here runs with no network**, because the crate has no
//! network code at all. That is the property, not a test condition.

use std::path::{Path, PathBuf};

use b_ids::schema::Channel;
use b_ids::schema::http::Variant;
use b_ids::{Version, at, client_hints, header_order, latest_stable, profiles, release, select};

/// ⭐ **The same seam [`b-ids/build.rs`] reads**, and it has to be, because a
/// suite that read a different corpus than the crate embedded would report on
/// one and ship the other.
/// The corpus this build embedded, so the suite can recompute its identifier.
///
/// ⛔ **Resolved, never assumed.** `corpus/` and `raw/` LEFT the default branch
/// in `PUB-13`, and this suite reached them by walking up from its own manifest.
/// Measured the day they left:
/// `the_embedded_release_identifier_is_the_corpus_this_build_was_cut_from`
/// panicked with `NotFound` on the index while every other case here passed,
/// because the others read what the BUILD embedded and only this one goes back
/// to disk. ⭐ That asymmetry is the point of the case and it is why it is the
/// one that broke. TODO/publish.md, `PUB-11` and `PUB-13`.
fn corpus_root() -> PathBuf {
    b_ids_schema::root::corpus_root_or_explain(Path::new(env!("CARGO_MANIFEST_DIR")))
}

#[test]
fn the_embedded_release_identifier_is_the_corpus_this_build_was_cut_from() {
    // ⛔ THE BUILD SCRIPT DOES NOT GRADE ITSELF. The identifier is recomputed
    // here from the index on disk, with the same digest the rest of the tree
    // uses, and compared with the one the build embedded.
    let index = corpus_root().join("corpus").join("v1").join("index.json");
    let bytes = std::fs::read(&index).expect("the corpus index");
    let want = b_ids_harness::hex(&b_ids_harness::sha256(&bytes));
    assert_eq!(
        release().identifier,
        want,
        "the embedded identifier is not this corpus"
    );

    // ⚠ And the identifier is a pin rather than a label: it moves when the
    // corpus moves.
    let mut moved = bytes.clone();
    moved.push(b'\n');
    let other = b_ids_harness::hex(&b_ids_harness::sha256(&moved));
    assert_ne!(want, other, "the identifier does not move with the corpus");
}

#[test]
fn the_release_says_how_old_the_data_is_without_leaving_the_language() {
    let released = release();
    assert_eq!(released.profiles, profiles().len());
    assert!(released.profiles > 0, "the crate embedded no profile");
    assert_eq!(released.layout, "v1");
    let newest = released
        .newest_capture
        .as_deref()
        .expect("a corpus with profiles has a newest capture");
    // ⚠ The instant is the profiles' own spelling, not one reformatted here.
    assert!(
        profiles().iter().any(|p| p.captured.at == newest),
        "the newest capture is not one any profile records"
    );
}

#[test]
fn every_embedded_profile_parses_and_carries_its_provenance() {
    // ⛔ PROVENANCE IS PART OF THE PUBLIC SHAPE. A consumer has to be able to
    // ask whether a field was measured, and a crate that hid the map would make
    // that unanswerable in the language they are already in.
    for profile in profiles() {
        assert!(!profile.id.to_string().is_empty());
        assert!(
            !profile.provenance.is_empty(),
            "{} carries no provenance at all",
            profile.id
        );
    }
}

#[test]
fn selecting_an_uncaptured_platform_returns_nothing_rather_than_a_substitute() {
    // ⛔ THE RULE THIS CRATE EXISTS TO HOLD. A neighbouring platform's profile
    // behind a measured interface is an unmeasured value a consumer cannot
    // tell apart from a measured one.
    assert!(
        select("chrome", Channel::Stable, "solaris-sparc", Version::Latest).is_none(),
        "a platform this project never captured produced a profile"
    );
    assert!(
        select("netscape", Channel::Stable, "linux64", Version::Latest).is_none(),
        "a browser this project never captured produced a profile"
    );
    // ⚠ And an exact version nobody captured is absent too, on a platform that
    // IS captured, which is the case a fallback would have hidden.
    assert!(
        select(
            "chrome",
            Channel::Stable,
            "linux64",
            Version::Exact("1.2.3.4")
        )
        .is_none()
    );
}

#[test]
fn latest_is_read_from_the_corpus_pointer_rather_than_derived_here() {
    let Some(newest) = latest_stable("chrome", "linux64") else {
        panic!("the corpus publishes a stable chrome/linux64 pointer and the crate lost it");
    };
    assert_eq!(newest.browser.channel, Channel::Stable);

    // ⭐ The same profile through the channel-keyed route, so the two ways of
    // asking agree.
    let by_channel = select("chrome", Channel::Stable, "linux64", Version::Latest)
        .expect("the same pointer, keyed by channel");
    assert_eq!(newest.id, by_channel.id);

    // ⚠ And asking for it by its exact version is the same profile again.
    let exact = select(
        "chrome",
        Channel::Stable,
        "linux64",
        Version::Exact(&newest.browser.version),
    )
    .expect("the build the pointer names");
    assert_eq!(exact.id, newest.id);
}

#[test]
fn a_profile_is_reachable_by_the_published_path_the_index_gave() {
    // ⚠ NOTHING HERE PARSES A ROUTE. The layout belongs to the corpus, and this
    // crate keys what it embedded by the path the index stated.
    let first = b_ids::paths().first().expect("at least one profile");
    let profile = at(first).expect("the path the index gave");
    assert!(first.contains(&profile.browser.version));
    assert!(at("corpus/v1/nothing/here.json").is_none());
}

#[test]
fn the_parts_a_consumer_actually_wants_come_out_on_their_own() {
    let profile = latest_stable("chrome", "linux64").expect("a stable chrome profile");

    // ⭐ THE ORDER IS THE FINGERPRINT, so it comes back in wire order and a
    // caller that sorts it has thrown away what it asked for.
    let order = header_order(profile, Variant::Navigate);
    assert!(order.len() > 3, "{order:?}");
    let mut sorted = order.clone();
    sorted.sort_unstable();
    assert_ne!(
        order, sorted,
        "the navigate header order is already sorted, so this assertion proves nothing"
    );

    // ⚠ A profile that records names only has no values, and that is the
    // default rather than a defect. Both answers are honest; neither is a zero.
    let ua = b_ids::user_agent(profile);
    let hints = client_hints(profile);
    if profile.http.carries_values() {
        assert!(
            ua.is_some(),
            "values are recorded and the User-Agent is not"
        );
    } else {
        assert!(ua.is_none());
        assert!(hints.is_empty());
    }

    // ⭐ The raw bytes are part of the profile, because every digest strips
    // GREASE and this is the one artefact that does not.
    assert!(
        b_ids::client_hello_hex(profile).is_some(),
        "the profile publishes no ClientHello"
    );
}
