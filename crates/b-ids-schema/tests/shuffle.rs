//! SCHEMA-10. Record the shuffle as a property, and consider recording its
//! seed.
//!
//! The acceptance: a profile marked `shuffled` with exactly one observed order
//! fails validation, and a fixture of eight captures of one binary that shows a
//! single order fails with a message saying so.
//!
//! ⛔ Every test name starts with `shuffle`, because
//! `cargo test --workspace shuffle` is the entry's acceptance command.
//!
//! ⚠ **The tests are in TWO crates and the acceptance runs the workspace.**
//! Half of this entry is a refusal in the model and half is a finding in the
//! validator, and `b-ids-schema` cannot depend on `b-ids-validator` because
//! the validator depends on it. `crates/b-ids-validator/tests/shuffle.rs` is
//! the other half.
//!
//! ⛔ **A profile records ONE connection.** What is carried here is a COUNT of
//! the distinct orders the sample produced, never the orders themselves: a
//! profile holding the other connections' orders would fold a set of captures
//! into one, which `docs/inherited-claims.md` section 8 says never to do.

mod support;

use b_ids_schema::tls::Shuffle;

use support::{schema, validate};

#[test]
fn shuffle_observed_with_one_order_is_refused() {
    // ⭐ THE ENTRY'S OWN CASE. `Observed` says the order differed between draws.
    // Reporting one distinct order beside it is a claim its own field denies,
    // and a consumer reading the state alone would take one draw for the shape.
    for distinct in [0_u32, 1] {
        let mut profile = b_ids_schema::fixture::profile();
        profile.tls.shuffled = Shuffle::Observed {
            draws: 8,
            distinct_orders: distinct,
        };
        let defects = profile.check();
        assert!(
            defects
                .iter()
                .any(|d| d.to_string().contains("shuffle nobody saw twice")),
            "distinct_orders={distinct}: {defects:?}"
        );
    }
}

#[test]
fn shuffle_observed_with_two_orders_is_accepted() {
    // ⚠ The positive control. A refusal that fired on everything would report
    // nothing, and this is the shape the harness actually produces.
    let mut profile = b_ids_schema::fixture::profile();
    profile.tls.shuffled = Shuffle::Observed {
        draws: 8,
        distinct_orders: 2,
    };
    assert!(profile.check().is_empty(), "{:?}", profile.check());
}

#[test]
fn shuffle_the_published_schema_carries_the_count() {
    let schema = schema();
    let mut profile =
        serde_json::to_value(b_ids_schema::fixture::profile()).expect("the fixture serialises");
    assert!(validate(&schema, &profile).is_empty());

    // ⛔ It is bounded like every other integer in the contract, and a negative
    // is refused at the schema as well as by the Rust width.
    profile["tls"]["shuffled"]["distinct_orders"] = serde_json::json!(-1);
    let problems = validate(&schema, &profile);
    assert!(!problems.is_empty(), "{problems:?}");
}

#[test]
fn shuffle_a_profile_written_before_the_field_existed_still_reads() {
    // ⚠ Defaulted on the way in, and 0 then means "not recorded". ⛔ A profile
    // that omitted the field and claimed `observed` is still refused, because
    // 0 is fewer than 2: an absent count cannot support the claim either.
    let json = serde_json::json!({ "state": "observed", "draws": 8 });
    let state: Shuffle = serde_json::from_value(json).expect("it deserialises");
    assert_eq!(
        state,
        Shuffle::Observed {
            draws: 8,
            distinct_orders: 0
        }
    );
}
