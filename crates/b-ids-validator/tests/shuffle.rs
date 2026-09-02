//! SCHEMA-10. Record the shuffle as a property, and consider recording its
//! seed. The validator half.
//!
//! ⚠ **The other half is `crates/b-ids-schema/tests/shuffle.rs`**, and it is
//! in another crate because `b-ids-schema` cannot depend on the validator that
//! depends on it. `cargo test --workspace shuffle` runs both.
//!
//! ⛔ Every test name starts with `shuffle`.

use b_ids_schema::tls::Shuffle;
use b_ids_validator::{Options, Outcome, check_grease};

#[test]
fn shuffle_eight_draws_of_one_order_is_a_finding_for_a_family_that_shuffles() {
    // ⛔ THE SECOND HALF OF THE ACCEPTANCE. Eight captures of one binary that
    // produced a single order is a reason to doubt the capture, and
    // docs/inherited-claims.md section 2 is why: reproducing a recorded order
    // exactly is what a browser that shuffles does not do.
    let mut profile = b_ids_schema::fixture::profile();
    profile.tls.shuffled = Shuffle::Fixed { draws: 8 };

    let options = Options {
        expects_shuffle: Some(true),
        ..Options::default()
    };
    let outcome = check_grease(&profile, &options);
    let message = format!("{outcome:?}");
    assert!(
        message.contains("produced one") && message.contains("doubt the capture"),
        "{message}"
    );
}

#[test]
fn shuffle_the_same_profile_is_clean_when_nobody_stated_the_family_shuffles() {
    // ⛔ THE FACT IS THE CALLER'S, NOT THE PROFILE'S. Whether a FAMILY shuffles
    // is a fact about a browser rather than about one connection, and a check
    // that assumed it would report every non-shuffling browser as broken.
    let mut profile = b_ids_schema::fixture::profile();
    profile.tls.shuffled = Shuffle::Fixed { draws: 8 };
    let outcome = check_grease(&profile, &Options::default());
    assert_eq!(outcome, Outcome::Passed, "{outcome:?}");

    // ⚠ And stating it as false is not the same as not stating it: both are
    // clean here, and only the third state is a finding.
    let stated_false = Options {
        expects_shuffle: Some(false),
        ..Options::default()
    };
    assert_eq!(check_grease(&profile, &stated_false), Outcome::Passed);
}

#[test]
fn shuffle_one_draw_says_nothing_whatever_the_state_claims() {
    // ⚠ Already held before this entry and asserted here so it stays held: a
    // shuffle state read off a single handshake is a state nobody sampled.
    for state in [
        Shuffle::Fixed { draws: 1 },
        Shuffle::Observed {
            draws: 1,
            distinct_orders: 2,
        },
    ] {
        let mut profile = b_ids_schema::fixture::profile();
        profile.tls.shuffled = state.clone();
        let outcome = check_grease(&profile, &Options::default());
        let message = format!("{outcome:?}");
        assert!(
            message.contains("one draw is not a sample"),
            "{state:?}: {message}"
        );
    }
}
