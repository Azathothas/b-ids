//! Every parser the harness exposes to the network, over one input.
//!
//! ⭐ **One target, not four, and the reason is that the list of parsers lives
//! in the library.** A target per parser is four lists to keep in step with a
//! crate that grows one, and the day a fifth parser lands the four targets keep
//! passing while covering less. `b_ids_harness::fuzz::drive_every_parser` is
//! the list, and `crates/b-ids-harness/tests/hostile.rs` calls the same
//! function on every host.
//!
//! ⚠ **The coverage feedback is not lost by combining them.** libFuzzer
//! measures coverage over the whole call, so an input that reaches deep into
//! the HPACK decoder is kept whether or not it also reached the hello parser.
//!
//! ```text
//! cargo +nightly fuzz run parsers -- -runs=1000000
//! ```
//!
//! `TODO/harness.md`, `HARNESS-09`.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // ⛔ The property is the absence of a panic. Nothing is asserted about what
    // came back: an assertion about the value would make this a test of the
    // parse rather than of the process surviving what arrives on a socket.
    b_ids_harness::fuzz::drive_every_parser(data);
});
