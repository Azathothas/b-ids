//! HARNESS-06. Parse permissively, emit exactly.
//!
//! The acceptance: a fixture per GREASE value, each with a one-byte body, all
//! parse; and the emitter refuses a profile whose extension body it cannot
//! reproduce, rather than emitting an approximation.
//!
//! ⛔ Every test name starts with `grease_bodies`, because
//! `cargo test -p b-ids-harness grease_bodies` is the entry's acceptance
//! command.
//!
//! ⛔ **The two models are two types and this file is where that is asserted.**
//! `b_ids_schema::tls::Extension` is what a parser records: any codepoint, any
//! body, and a declared length beside the body so a disagreement between them
//! is kept rather than repaired. `b_ids_emit::EmittableExtension` is what an
//! emitter can put on the wire: bytes, with the length derived from them. A
//! codebase that used one type for both would get one of the two wrong.

mod support;

use b_ids_emit::{Unreproducible, extension, extensions};
use b_ids_harness::parse_record;
use b_ids_schema::tls::{Extension, is_grease_value};

use support::{client_hello, grease_values};

/// A hello carrying GREASE at both ends: empty first, one zero byte last.
///
/// ⚠ That is the shape a browser sends, and it is the shape a parser with a
/// typed GREASE field gets wrong.
fn hello_with(first: u16, last: u16) -> Vec<u8> {
    client_hello(&[
        (first, Vec::new()),
        (0x0000, vec![0x00, 0x00]),
        (0x002b, vec![0x02, 0x03, 0x04]),
        (last, vec![0x00]),
    ])
}

#[test]
fn grease_bodies_every_reserved_value_parses_with_a_one_byte_body() {
    // ⛔ The acceptance's first half. Measured elsewhere: three of the sixteen
    // reserved values were rejected by a parser that mapped them to a typed
    // field with an empty body, so about one handshake in five. A test over one
    // value would have passed four times in five.
    let values = grease_values();
    assert_eq!(values.len(), 16);
    assert!(values.iter().all(|v| is_grease_value(*v)));

    for (index, first) in values.iter().enumerate() {
        // ⚠ Distinct at the two ends, which is what a browser draws. The same
        // value at both ends is a constant a server can key on.
        let last = values[(index + 1) % values.len()];
        let bytes = hello_with(*first, last);
        let capture = parse_record(&bytes)
            .unwrap_or_else(|why| panic!("GREASE 0x{first:04x}/0x{last:04x} did not parse: {why}"));

        assert!(
            capture.notes.is_empty(),
            "0x{first:04x}: {:?}",
            capture.notes
        );
        assert_eq!(capture.tls.grease.values, vec![*first, last]);
        assert!(capture.tls.grease.distinct);
        assert_eq!(
            capture.tls.grease.bodies_hex,
            vec![String::new(), "00".to_owned()],
            "0x{first:04x}: the trailing GREASE body was not kept"
        );
        assert_eq!(capture.tls.grease.extension_positions, vec![0, 3]);
    }
}

#[test]
fn grease_bodies_the_emitter_reproduces_every_body_byte_for_byte() {
    // ⛔ Emit EXACTLY. An approximation is a ClientHello that exists nowhere,
    // and a client announcing one version over a hello nobody sends is more
    // distinguishing than an honestly old one.
    for (index, first) in grease_values().iter().enumerate() {
        let last = grease_values()[(index + 1) % 16];
        let capture = parse_record(&hello_with(*first, last)).expect("parses");
        let emittable = extensions(&capture.tls).expect("every extension is reproducible");
        assert_eq!(emittable.len(), capture.tls.extensions.len());

        for (out, captured) in emittable.iter().zip(&capture.tls.extensions) {
            let mut expected = Vec::new();
            expected.extend_from_slice(&captured.codepoint.to_be_bytes());
            expected.extend_from_slice(&captured.length.to_be_bytes());
            expected.extend_from_slice(&b_ids_harness::unhex(&captured.body_hex).expect("hex"));
            assert_eq!(out.encode(), expected);
        }

        // ⭐ And the trailing GREASE keeps its one byte rather than being
        // flattened to empty.
        let trailing = emittable.last().expect("a trailing extension");
        assert_eq!(trailing.codepoint, last);
        assert_eq!(trailing.body, vec![0x00]);
    }
}

#[test]
fn grease_bodies_the_emitter_refuses_a_body_it_cannot_reproduce() {
    // ⛔ The acceptance's second half. A refusal, never an approximation: an
    // emitter that wrote its best guess would put a hello on the wire that no
    // browser sends.
    let disagreeing = Extension {
        codepoint: 0x4a4a,
        length: 4,
        body_hex: "00".to_owned(),
    };
    let refusal = extension(&disagreeing).expect_err("a length that disagrees is refused");
    assert_eq!(
        refusal,
        Unreproducible::LengthDisagrees {
            codepoint: 0x4a4a,
            declared: 4,
            actual: 1,
        }
    );
    assert!(refusal.to_string().contains("believe one of them"));

    let not_hex = Extension {
        codepoint: 0x0015,
        length: 1,
        body_hex: "zz".to_owned(),
    };
    assert!(matches!(
        extension(&not_hex).expect_err("a body that is not hex is refused"),
        Unreproducible::BodyNotHex {
            codepoint: 0x0015,
            ..
        }
    ));
}

#[test]
fn grease_bodies_the_emitter_reports_every_refusal_and_not_only_the_first() {
    // ⚠ An emitter that stopped at the first refusal sends its author back for
    // one more run per defect, and a capture cannot be retaken.
    let mut tls = parse_record(&hello_with(0x0a0a, 0x1a1a))
        .expect("parses")
        .tls;
    tls.extensions[0].length = 9;
    tls.extensions[3].body_hex = "0".to_owned();

    let refusals = extensions(&tls).expect_err("two extensions are unreproducible");
    assert_eq!(refusals.len(), 2, "{refusals:?}");
}

#[test]
fn grease_bodies_the_parser_keeps_what_the_emitter_refuses() {
    // ⭐ This is the whole entry in one assertion. The two requirements pull in
    // opposite directions, and a single shared type could satisfy only one of
    // them: the parser must accept a capture whose declared length disagrees
    // with its body, because that disagreement is a finding worth keeping, and
    // the emitter must refuse the same record, because it cannot put both
    // numbers on the wire.
    let recorded = Extension {
        codepoint: 0xca34,
        length: 206,
        body_hex: "00".to_owned(),
    };
    // The parser's model represents it without complaint.
    assert_eq!(recorded.length, 206);
    assert_eq!(recorded.body_hex, "00");
    // The emitter's model cannot represent it at all.
    assert!(extension(&recorded).is_err());
}
