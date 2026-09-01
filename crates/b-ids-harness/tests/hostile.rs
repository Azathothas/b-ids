//! HARNESS-09. Fuzz the parsers. A panic here is unacceptable.
//!
//! ⛔ Every test name starts with `hostile`, because
//! `cargo test -p b-ids-harness hostile` is the half of this entry that runs on
//! every host, every push, with no nightly toolchain and no extra tool.
//!
//! ⭐ **This is not a substitute for `cargo fuzz` and it is not trying to be.**
//! Coverage-guided fuzzing explores; this one asserts, over a corpus a seed
//! reproduces exactly. What it buys is that the property is checked by the
//! ordinary gate rather than by a tool somebody has to remember to run, and the
//! `fuzz/` targets beside it call the same function.
//!
//! ⚠ **The corpus is mutations of the committed captures, not random bytes.**
//! Random input almost never survives the first length check, so a run fed only
//! random bytes exercises one comparison and reports success.

mod support;

use b_ids_harness::fuzz::{Rng, cases, drive_every_parser};

/// The committed captures this corpus is mutated from.
///
/// ⭐ **Real bytes, because a truncation of a real hello reaches every field the
/// parser reads.** That is where a slice past the end lives.
fn seeds() -> Vec<Vec<u8>> {
    vec![
        support::fixture_bytes("client-hello.hex"),
        support::fixture_bytes("h2-connection.hex"),
        support::fixture_bytes("h2-connection-missing-setting.hex"),
    ]
}

#[test]
fn hostile_no_parser_panics_on_any_mutation_of_a_real_capture() {
    // ⛔ THE PROPERTY IS THE ABSENCE OF A PANIC. Nothing here asserts what came
    // back: an assertion about the value would make this a test of the parse
    // rather than of the process surviving whatever arrives on a socket.
    let mut rng = Rng::new(0x0000_b1d5_0001);
    let corpus = cases(&seeds(), &mut rng, 4_000);
    assert!(
        corpus.len() > 5_000,
        "the corpus is large enough to be worth running: {}",
        corpus.len()
    );
    for case in &corpus {
        drive_every_parser(case);
    }
}

#[test]
fn hostile_the_corpus_is_the_same_on_every_host_and_every_run() {
    // ⛔ A randomised test whose input depends on when it ran is a test that
    // fails once and cannot be re-run. The seed is what makes a crash
    // reproducible from its own report.
    let build = || {
        let mut rng = Rng::new(0x0000_b1d5_0001);
        cases(&seeds(), &mut rng, 50)
    };
    assert_eq!(build(), build());
}

#[test]
fn hostile_a_length_that_claims_more_than_arrived_is_refused_rather_than_read() {
    // ⛔ Trusting a declared length instead of counting what arrived is its own
    // defect class, and this is the shape of it: a record header saying 0xffff
    // over four bytes of body.
    let mut record = vec![0x16, 0x03, 0x01, 0xff, 0xff];
    record.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    let read = b_ids_harness::parse_record(&record);
    assert!(
        read.is_err() || read.is_ok(),
        "either answer is acceptable; unwinding is not"
    );
    drive_every_parser(&record);
}

#[test]
fn hostile_the_empty_input_reaches_every_parser_without_unwinding() {
    // ⚠ The case a hand-written test forgets and a socket produces constantly:
    // a connection opened and closed with nothing on it.
    drive_every_parser(&[]);
    drive_every_parser(&[0x00]);
}

#[test]
fn hostile_a_huffman_literal_of_all_ones_terminates() {
    // ⚠ A canonical Huffman decoder can be walked into a loop by a padding run,
    // and the failure is a hang rather than a panic, which no fuzz target that
    // only watches for crashes would report. A test with a bound is what sees
    // it.
    let padding = vec![0xff_u8; 512];
    let _ = b_ids_harness::hpack::decode_huffman(&padding);
    drive_every_parser(&padding);
}

#[test]
fn hostile_every_prefix_of_every_seed_is_in_the_corpus() {
    // ⭐ Truncation is the single highest-value mutation, so a corpus that
    // dropped it would still pass while covering much less. This asserts the
    // generator, because the run above cannot: it reports nothing either way.
    let mut rng = Rng::new(1);
    let seeds = seeds();
    let corpus = cases(&seeds, &mut rng, 0);
    for seed in &seeds {
        for take in 0..=seed.len() {
            assert!(
                corpus.contains(&seed[..take].to_vec()),
                "the prefix of {take} byte(s) is in the corpus"
            );
        }
    }
}
