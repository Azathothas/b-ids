//! VALID-05. A conformance suite for impersonating clients.
//!
//! ⛔ Every test name starts with `conformance`, because
//! `cargo test -p b-ids-conformance conformance` is the entry's acceptance
//! command.
//!
//! ⚠ **The fixture profile, not the corpus.** These assert the COMPARISON, and
//! a comparison asserted against live data changes meaning every time a profile
//! lands. The binary's `--fixture` is what runs it over the real corpus.

use b_ids_conformance::{HTTP_FIELDS, Verdict, compare, fields};
use b_ids_schema::http::Variant;
use b_ids_schema::http2::{Frame, SettingEntry};

fn verdict<'a>(report: &'a b_ids_conformance::Report, field: &str) -> &'a Verdict {
    &report
        .fields
        .iter()
        .find(|f| f.field == field)
        .unwrap_or_else(|| panic!("{field} is not a field this comparison knows"))
        .verdict
}

#[test]
fn conformance_a_profile_against_itself_differs_on_nothing() {
    let profile = b_ids_schema::fixture::profile();
    let report = compare(&profile, &profile);
    assert!(
        report.differing().is_empty(),
        "{:?}",
        report
            .differing()
            .iter()
            .map(|f| &f.field)
            .collect::<Vec<_>>()
    );
    // ⛔ AND SOMETHING WAS ACTUALLY COMPARED. A comparison that rendered
    // nothing on either side would report every field as not-checkable and
    // satisfy the assertion above.
    assert!(report.conforming() > 0, "nothing was compared");
}

#[test]
fn conformance_a_swapped_extension_pair_is_reported_as_per_connection_not_wrong() {
    // ⛔ THE SHUFFLE IS WHY THIS IS NOT A CONFORMANCE FAILURE. Chrome reorders
    // its extensions per connection, so two captures of one browser differ here
    // every time and a report calling that wrong would name a field on every
    // run. ⚠ The verdict still carries both values and the reason, because a
    // client whose order NEVER changes is distinguishable for that reason and
    // one capture cannot see it.
    let claimed = b_ids_schema::fixture::profile();
    let mut observed = claimed.clone();
    observed.tls.extensions.swap(0, 1);
    observed.id = observed.derived_id();

    let report = compare(&claimed, &observed);
    assert!(
        report.differing().is_empty(),
        "a reshuffled order is not a difference a single capture can conclude from: {:?}",
        report
            .differing()
            .iter()
            .map(|f| &f.field)
            .collect::<Vec<_>>()
    );
    let v = verdict(&report, "tls.extensions.order");
    let Verdict::PerConnection { why, .. } = v else {
        panic!("the shuffle is a per-connection verdict, not {v:?}");
    };
    assert!(why.contains("shuffled per connection"), "{why}");
}

#[test]
fn conformance_a_grease_draw_is_not_a_difference() {
    // ⛔ GREASE IS DRAWN PER CONNECTION, so two captures of ONE browser differ
    // on it every time. A tool that called that a failure would name a field on
    // every run and stop being read.
    let claimed = b_ids_schema::fixture::profile();
    let mut observed = claimed.clone();

    let Some(index) = observed
        .tls
        .cipher_suites
        .iter()
        .position(|c| b_ids_schema::tls::is_grease_value(*c))
    else {
        panic!("the fixture carries no GREASE cipher suite, so this cannot be tested");
    };
    // ⚠ A DIFFERENT GREASE VALUE IN THE SAME POSITION. 0x0a0a and 0x1a1a are
    // both RFC 8701 reserved, so this is what a second connection of the same
    // browser looks like.
    observed.tls.cipher_suites[index] = if claimed.tls.cipher_suites[index] == 0x0a0a {
        0x1a1a
    } else {
        0x0a0a
    };
    observed.id = observed.derived_id();

    let report = compare(&claimed, &observed);
    assert!(
        matches!(
            verdict(&report, "tls.cipher_suites"),
            Verdict::PerConnection { .. }
        ),
        "a redrawn GREASE value is not a conformance failure: {:?}",
        verdict(&report, "tls.cipher_suites")
    );
    assert!(
        report.differing().is_empty(),
        "{:?}",
        report
            .differing()
            .iter()
            .map(|f| &f.field)
            .collect::<Vec<_>>()
    );
}

#[test]
fn conformance_a_grease_value_moved_to_another_position_is_still_caught() {
    // ⚠ THE OTHER HALF OF THE RULE ABOVE, and without it the forgiveness would
    // be a hole. Only the drawn VALUE is forgiven; where it sits is a separate
    // field and it is still compared.
    let claimed = b_ids_schema::fixture::profile();
    let mut observed = claimed.clone();
    let Some(index) = observed
        .tls
        .cipher_suites
        .iter()
        .position(|c| b_ids_schema::tls::is_grease_value(*c))
    else {
        panic!("the fixture carries no GREASE cipher suite");
    };
    let grease = observed.tls.cipher_suites.remove(index);
    observed.tls.cipher_suites.push(grease);
    observed.id = observed.derived_id();

    let report = compare(&claimed, &observed);
    let named: Vec<&str> = report
        .differing()
        .iter()
        .map(|f| f.field.as_str())
        .collect();
    assert!(
        named.contains(&"tls.cipher_suites.no_grease") || named.contains(&"tls.cipher_suites"),
        "moving GREASE reorders the list around it and that is a real difference: {named:?}"
    );
}

#[test]
fn conformance_a_reordered_header_is_named() {
    // ⛔ THE ENTRY ASKS FOR "WHICH HEADER CHANGED POSITION", and a comparison
    // over the TLS vocabulary alone cannot answer it.
    let claimed = b_ids_schema::fixture::profile();
    let mut observed = claimed.clone();
    let set = observed
        .http
        .variants
        .iter_mut()
        .find(|s| s.variant == Variant::Navigate)
        .expect("the fixture carries a navigation set");
    assert!(set.headers.len() >= 2, "too few headers to reorder");
    set.headers.swap(0, 1);
    observed.id = observed.derived_id();

    let report = compare(&claimed, &observed);
    let named: Vec<&str> = report
        .differing()
        .iter()
        .map(|f| f.field.as_str())
        .collect();
    assert_eq!(named, ["http.navigate.header_order"], "{named:?}");
    // ⚠ THE COUNT DID NOT CHANGE, which is what makes this a REORDER rather
    // than an added header. A tool that reported both would be describing one
    // change twice.
    assert!(matches!(
        verdict(&report, "http.navigate.header_count"),
        Verdict::Conforms(_)
    ));
}

#[test]
fn conformance_a_changed_setting_value_is_named_and_the_order_is_not() {
    // ⛔ THE ENTRY ASKS FOR "WHICH SETTING IS ABSENT". The order and the values
    // are two fields, so a client that sends every setting with the right value
    // in the wrong order is distinguishable from one that changed a value.
    let claimed = b_ids_schema::fixture::profile();
    let mut observed = claimed.clone();
    let mut touched = false;
    for frame in &mut observed.http2.frames {
        if let Frame::Settings { entries } = frame
            && let Some(first) = entries.first_mut()
        {
            *first = SettingEntry {
                id: first.id,
                value: first.value.wrapping_add(1),
            };
            touched = true;
        }
    }
    assert!(touched, "the fixture carries no SETTINGS frame");
    observed.id = observed.derived_id();

    let report = compare(&claimed, &observed);
    let named: Vec<&str> = report
        .differing()
        .iter()
        .map(|f| f.field.as_str())
        .collect();
    assert_eq!(named, ["http2.settings.values"], "{named:?}");
    assert!(matches!(
        verdict(&report, "http2.settings.order"),
        Verdict::Conforms(_)
    ));
}

#[test]
fn conformance_a_field_one_side_does_not_carry_is_not_checkable_rather_than_agreed() {
    // ⛔ REPORTING AN ABSENCE AS AGREEMENT IS HOW A CLIENT PASSES ON A FIELD
    // NOBODY LOOKED AT. It is the single most useful thing this type does.
    let claimed = b_ids_schema::fixture::profile();
    let mut observed = claimed.clone();
    observed.http.variants.clear();
    observed.id = observed.derived_id();

    let report = compare(&claimed, &observed);
    assert!(matches!(
        verdict(&report, "http.navigate.header_order"),
        Verdict::NotCheckable {
            claimed: true,
            observed: false
        }
    ));
    // ⚠ And it is NOT counted as a difference, because nothing was compared.
    let named: Vec<&str> = report
        .differing()
        .iter()
        .map(|f| f.field.as_str())
        .collect();
    assert!(
        !named.contains(&"http.navigate.header_order"),
        "an uncomparable field is not a differing one: {named:?}"
    );
}

#[test]
fn conformance_every_field_it_lists_is_a_field_it_compares() {
    // ⚠ A vocabulary that names a field the comparison never renders is a field
    // that reads as not-checkable forever, which looks like a measurement and
    // is a typo.
    let profile = b_ids_schema::fixture::profile();
    let report = compare(&profile, &profile);
    let listed = fields();
    assert_eq!(report.fields.len(), listed.len());
    for field in HTTP_FIELDS {
        let v = verdict(&report, field);
        assert!(
            !matches!(v, Verdict::NotCheckable { .. }),
            "{field} is listed and the fixture profile renders nothing for it: {v:?}"
        );
    }
}
