//! SCHEMA-09. Name every field for the wire, because three quantities have two
//! units.
//!
//! The acceptance: a profile whose named increment and named window do not
//! differ by exactly 65,535 is rejected, and a profile whose weight field is
//! 256 is rejected with a message naming the encoding.
//!
//! ⛔ Every test name starts with `units`, because
//! `cargo test -p b-ids-schema units` is the entry's acceptance command.

mod support;

use b_ids_schema::Profile;
use b_ids_schema::http2::PROTOCOL_DEFAULT_WINDOW;
use support::{as_json, fixture, messages};

#[test]
fn units_the_fixture_carries_both_and_they_agree() {
    let half = fixture().http2;
    assert_eq!(half.connection_window, Some(15_728_640));
    assert_eq!(half.window_size_increment(), Some(15_663_105));
    assert_eq!(
        half.connection_window.expect("a window")
            - half.window_size_increment().expect("an increment"),
        PROTOCOL_DEFAULT_WINDOW
    );
    assert!(half.check_units().is_empty());
}

#[test]
fn units_a_window_and_increment_that_do_not_differ_by_the_default_is_rejected() {
    // ⛔ The exact defect the sweep found shipped: one field holding the window
    // in one entry and the increment in seven others, the seven emitting a
    // value 65,535 short.
    let mut profile = fixture();
    profile.http2.connection_window = Some(15_663_105);
    let defects = profile.check();
    let text = messages(&defects);
    assert!(text.contains("http2.connection_window"), "{text}");
    assert!(text.contains("65535"), "{text}");
    assert!(text.contains("one quantity in two units"), "{text}");
}

#[test]
fn units_the_check_is_arithmetic_rather_than_a_comment() {
    // ⚠ "Must not: rely on a comment to carry a unit. The comment in the
    // reference database was correct and seven entries beside it were still
    // wrong." So the check computes rather than asserting a documented value.
    let mut profile = fixture();
    for offset in [1_i64, -1, 65_534, 65_536] {
        let window = i64::from(15_663_105_u32) + offset;
        profile.http2.connection_window = Some(u32::try_from(window).expect("in range"));
        assert!(
            !profile.http2.check_units().is_empty(),
            "an offset of {offset} was accepted"
        );
    }
    profile.http2.connection_window = Some(15_663_105 + PROTOCOL_DEFAULT_WINDOW);
    assert!(profile.http2.check_units().is_empty());
}

#[test]
fn units_a_profile_that_records_only_the_wire_number_is_complete() {
    // ⭐ The increment is the measurement and the window is a convenience. A
    // capture that recorded only what the wire carried is not missing anything.
    let mut profile = fixture();
    profile.http2.connection_window = None;
    assert!(profile.http2.check_units().is_empty());
    assert!(profile.check().is_empty(), "{}", messages(&profile.check()));
}

#[test]
fn units_a_weight_of_256_is_rejected_with_a_message_naming_the_encoding() {
    // ⛔ 256 is the specification's unit and 255 is the wire's. A tool that
    // takes 256 puts 255 on the wire, and they are one quantity.
    let mut json = as_json(&fixture());
    json.pointer_mut("/http2/stream_priority/weight_wire")
        .map(|w| *w = serde_json::Value::from(256))
        .expect("the fixture carries a priority block");

    let err = serde_json::from_value::<Profile>(json).expect_err("256 is refused");
    let text = err.to_string();
    assert!(text.contains("weight_wire"), "{text}");
    assert!(text.contains("weight minus one"), "{text}");
    assert!(text.contains("0 to 255"), "{text}");
    // ⭐ The message names BOTH units, so a reader is not left to work out which
    // one they wrote.
    assert!(text.contains("256") && text.contains("255"), "{text}");
}

#[test]
fn units_a_weight_of_255_is_accepted() {
    // ⚠ Asserted separately, or the test above proves only that something was
    // refused rather than that the boundary is in the right place.
    let mut json = as_json(&fixture());
    json.pointer_mut("/http2/stream_priority/weight_wire")
        .map(|w| *w = serde_json::Value::from(255))
        .expect("the fixture carries a priority block");
    let profile = serde_json::from_value::<Profile>(json).expect("255 is accepted");
    let priority = profile.http2.stream_priority.expect("a priority block");
    assert_eq!(priority.weight_wire, 255);
    assert_eq!(priority.weight_spec(), 256);
}

#[test]
fn units_the_third_quantity_is_absence_and_it_is_a_different_shape() {
    // ⚠ "The settings a stack does not override" is the same class and is not a
    // naming problem: an absent setting produces the stack's own default on the
    // wire. The model distinguishes absent from defaulted, which is SCHEMA-03,
    // and this asserts the two entries agree about it.
    const MAX_FRAME_SIZE: u16 = 5;
    let profile = fixture();
    assert!(!profile.http2.sends_setting(MAX_FRAME_SIZE));

    let mut defaulted = fixture();
    for frame in &mut defaulted.http2.frames {
        if let b_ids_schema::http2::Frame::Settings { entries } = frame {
            entries.push(b_ids_schema::http2::SettingEntry {
                id: MAX_FRAME_SIZE,
                value: 16_384,
            });
        }
    }
    assert!(defaulted.http2.sends_setting(MAX_FRAME_SIZE));
    assert_ne!(profile.http2, defaulted.http2);
}
