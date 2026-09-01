//! The one ISO 8601 UTC instant, produced here and checked here.
//!
//! ⛔ **`captured.at` is never optional**, so something has to produce it, and
//! that something lives beside the check that refuses a badly shaped one. Two
//! modules, one writing the format and one validating it, is two places for the
//! format to be defined and one of them to be wrong.
//!
//! ⚠ **Civil-date arithmetic rather than a date library.** The whole
//! requirement is one direction, seconds since the epoch to
//! `2026-08-30T03:53:11Z`, in the one field a capture always fills in from a
//! clock. A dependency for that would be carried by every consumer that links
//! the model.
//!
//! ⛔ **Leap seconds are not represented and the format cannot represent one.**
//! Unix time does not count them, so an instant produced here during one names
//! the second before it. That is a property of the input, written down rather
//! than hidden.

/// Days in each month of a non-leap year.
const MONTH_LENGTHS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Whether `year` is a leap year in the proleptic Gregorian calendar.
fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Format seconds since the Unix epoch as an ISO 8601 UTC instant.
///
/// ⚠ **Whole seconds, and no fractional part.** A capture is ordered against a
/// build release, and a build is not released to the millisecond. A field with
/// a precision nobody needs is a field two writers will format differently.
///
/// ⚠ Instants before the epoch are represented, because refusing them here
/// would put a range check in the one function a caller cannot work around; a
/// capture dated before 1970 is a defect the record's own reader should name,
/// not something this formatter should silently repair.
#[must_use]
pub fn from_unix(seconds: i64) -> String {
    let mut days = seconds.div_euclid(86_400);
    let rem = seconds.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let mut year = 1970_i64;
    loop {
        let len = if is_leap(year) { 366 } else { 365 };
        if days >= len {
            days -= len;
            year += 1;
        } else if days < 0 {
            year -= 1;
            days += if is_leap(year) { 366 } else { 365 };
        } else {
            break;
        }
    }

    let mut month = 0_usize;
    let mut day = u32::try_from(days).unwrap_or(0);
    while month < 12 {
        let mut len = MONTH_LENGTHS[month];
        if month == 1 && is_leap(year) {
            len += 1;
        }
        if day < len {
            break;
        }
        day -= len;
        month += 1;
    }

    format!(
        "{year:04}-{:02}-{:02}T{hour:02}:{minute:02}:{second:02}Z",
        month + 1,
        day + 1
    )
}

/// This machine's clock, as an ISO 8601 UTC instant.
///
/// ⚠ **A clock the caller cannot check.** The instant a capture carries is only
/// as good as the host's clock, which is why the capture records it once, at
/// the moment the bytes arrived, rather than being stamped later by whatever
/// read the capture back.
#[must_use]
pub fn now() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0));
    from_unix(seconds)
}
