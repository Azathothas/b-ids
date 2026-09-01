//! The one ISO 8601 UTC instant, produced and checked in one place.
//!
//! ⛔ Every test name starts with `instant`.
//!
//! ⚠ **This file exists because the guard-mutation pass found the formatter had
//! no test at all.** It fills `captured.at`, which a profile can never be
//! ordered against a build release without, and civil-date arithmetic is the
//! classic place to be wrong at a month boundary or a leap year while looking
//! right every other day of the year.

mod support;

use b_ids_schema::instant::{from_unix, now};

#[test]
fn instant_the_epoch_itself() {
    assert_eq!(from_unix(0), "1970-01-01T00:00:00Z");
}

#[test]
fn instant_the_last_second_of_a_day_does_not_roll_the_date() {
    // ⚠ The off-by-one that turns 23:59:59 into the next day, or midnight into
    // the day before.
    assert_eq!(from_unix(86_399), "1970-01-01T23:59:59Z");
    assert_eq!(from_unix(86_400), "1970-01-02T00:00:00Z");
}

#[test]
fn instant_a_leap_day_is_a_real_day() {
    // ⛔ 2024 is a leap year, so 29 February exists and 1 March is a day later
    // than a naive calendar would put it.
    assert_eq!(from_unix(1_709_164_800), "2024-02-29T00:00:00Z");
    assert_eq!(from_unix(1_709_251_200), "2024-03-01T00:00:00Z");
}

#[test]
fn instant_a_century_that_is_not_a_leap_year() {
    // ⚠ 1900 is divisible by 4 and is NOT a leap year, because it is divisible
    // by 100 and not by 400. A formatter that only tested the modulo-4 rule
    // passes every year in living memory and fails here.
    // ⛔ Before the epoch, which this deliberately represents rather than
    // refusing: a range check inside the formatter would be a repair the
    // record's own reader should make instead.
    assert_eq!(from_unix(-2_203_891_200), "1900-03-01T00:00:00Z");
}

#[test]
fn instant_a_year_divisible_by_four_hundred_is_a_leap_year() {
    // ⚠ The other half of the same rule, and the half that is wrong in the
    // opposite direction: 2000 IS a leap year.
    assert_eq!(from_unix(951_782_400), "2000-02-29T00:00:00Z");
}

#[test]
fn instant_the_end_of_a_year_and_the_start_of_the_next() {
    assert_eq!(from_unix(1_767_225_599), "2025-12-31T23:59:59Z");
    assert_eq!(from_unix(1_767_225_600), "2026-01-01T00:00:00Z");
}

#[test]
fn instant_every_month_boundary_of_one_year_lands_on_the_first() {
    // ⛔ The whole month table, walked, rather than one month spot-checked. A
    // wrong length for a single month is invisible until a capture happens to
    // fall in it.
    let starts = [
        (1_767_225_600_i64, "2026-01-01"),
        (1_769_904_000, "2026-02-01"),
        (1_772_323_200, "2026-03-01"),
        (1_775_001_600, "2026-04-01"),
        (1_777_593_600, "2026-05-01"),
        (1_780_272_000, "2026-06-01"),
        (1_782_864_000, "2026-07-01"),
        (1_785_542_400, "2026-08-01"),
        (1_788_220_800, "2026-09-01"),
        (1_790_812_800, "2026-10-01"),
        (1_793_491_200, "2026-11-01"),
        (1_796_083_200, "2026-12-01"),
    ];
    for (seconds, day) in starts {
        assert_eq!(from_unix(seconds), format!("{day}T00:00:00Z"), "{seconds}");
    }
}

#[test]
fn instant_what_it_produces_is_what_the_profile_check_accepts() {
    // ⭐ THE POINT OF THE TWO LIVING IN ONE MODULE. A formatter whose output the
    // validator refuses is two definitions of one format, and the profile would
    // be refused by the very check meant to protect it.
    let mut profile = support::fixture();
    for seconds in [0_i64, 951_782_400, 1_788_220_800] {
        profile.captured.at = from_unix(seconds);
        let messages = support::messages(&profile.check());
        assert!(
            !messages.contains("captured.at"),
            "{} is accepted: {messages}",
            profile.captured.at
        );
    }
    profile.captured.at = now();
    let messages = support::messages(&profile.check());
    assert!(
        !messages.contains("captured.at"),
        "this machine's clock produces an accepted instant: {messages}"
    );
}

#[test]
fn instant_a_shape_the_check_refuses_is_not_one_this_formatter_can_produce() {
    // ⚠ The negative half. `2026-08-30 03:53:11` is what a database's own "now"
    // hands back, and no consumer can sort it against an ISO column.
    let mut profile = support::fixture();
    profile.captured.at = "2026-08-30 03:53:11".to_owned();
    assert!(support::messages(&profile.check()).contains("captured.at"));
}
