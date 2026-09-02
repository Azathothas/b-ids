//! HARNESS-04. Decode HPACK Huffman, because header order is behind it.
//!
//! The acceptance: every case in the fetched vector corpus decodes to its
//! expected output, and the count of cases run is asserted, so a table that
//! stopped early cannot report green over a smaller suite.
//!
//! ⛔ Every test name starts with `hpack`, because
//! `cargo test -p b-ids-harness hpack` is the entry's acceptance command.
//!
//! ⚠ **The corpus is fetched, not written here.** A Huffman decoder that is
//! subtly wrong produces plausible header names, so a decoder checked only
//! against cases its own author invented is a decoder checked against its own
//! misreading. `references/http2jp__hpack-test-case/PROVENANCE.md` names the
//! commit.

use std::path::{Path, PathBuf};

use b_ids_harness::hpack::{Decoder, Indexing, decode_huffman};
use b_ids_harness::unhex;
use b_ids_schema::http::ValuePolicy;
use serde_json::Value;

/// Where the fetched vector corpus lives.
fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../references/http2jp__hpack-test-case/tree")
}

/// Every story file that carries wire bytes, sorted so a run is reproducible.
///
/// ⚠ One directory of the corpus holds the header lists with no wire at all.
/// It is the encoders' input rather than a decoder's, and skipping it is
/// recorded here rather than left to be inferred from a count.
fn story_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let root = corpus();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("the fetched corpus is at {}: {e}", root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .expect("a corpus directory is readable")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .collect();
        files.sort();
        out.extend(files);
    }
    out
}

struct Run {
    cases: usize,
    files: usize,
    failures: Vec<String>,
}

fn run_corpus() -> Run {
    let mut run = Run {
        cases: 0,
        files: 0,
        failures: Vec::new(),
    };
    for path in story_files() {
        let text = std::fs::read_to_string(&path).expect("a story file is readable");
        let story: Value = serde_json::from_str(&text).expect("a story file is JSON");
        let Some(cases) = story.get("cases").and_then(Value::as_array) else {
            continue;
        };
        if !cases.iter().any(|c| c.get("wire").is_some()) {
            continue;
        }
        run.files += 1;

        // ⛔ ONE decoder per story file, not one per case. The dynamic table is
        // connection state, and a decoder reset between blocks resolves every
        // later index against the wrong table.
        let mut decoder = Decoder::default();
        for case in cases {
            let Some(wire) = case.get("wire").and_then(Value::as_str) else {
                continue;
            };
            if let Some(size) = case.get("header_table_size").and_then(Value::as_u64) {
                decoder.set_settings_max(usize::try_from(size).unwrap_or(usize::MAX));
            }
            run.cases += 1;

            let block = unhex(wire).expect("a case's wire is hex");
            let expected: Vec<(String, String)> = case
                .get("headers")
                .and_then(Value::as_array)
                .expect("a case carries headers")
                .iter()
                .filter_map(|entry| {
                    let object = entry.as_object()?;
                    let (name, value) = object.iter().next()?;
                    Some((name.clone(), value.as_str()?.to_owned()))
                })
                .collect();

            match decoder.decode(&block) {
                Ok(decoded) => {
                    let produced: Vec<(String, String)> = decoded
                        .fields
                        .iter()
                        .map(|f| (f.name.clone(), f.value.clone().unwrap_or_default()))
                        .collect();
                    if produced != expected && run.failures.len() < 8 {
                        run.failures.push(format!(
                            "{}#{}: decoded {produced:?}, expected {expected:?}",
                            path.display(),
                            case.get("seqno").and_then(Value::as_u64).unwrap_or(0)
                        ));
                    }
                }
                Err(why) => {
                    if run.failures.len() < 8 {
                        run.failures.push(format!(
                            "{}#{}: {why}",
                            path.display(),
                            case.get("seqno").and_then(Value::as_u64).unwrap_or(0)
                        ));
                    }
                }
            }
        }
    }
    run
}

/// How many cases the fetched corpus carries.
///
/// ⛔ **Asserted, not reported.** A decoder that stopped early, or a corpus
/// that lost a directory, would otherwise report green over a smaller suite,
/// and a green count nobody pinned is the shape of every suite that quietly
/// shrank.
const EXPECTED_CASES: usize = 47_142;

/// How many story files carry wire bytes.
///
/// ⚠ 478 files are in the corpus and 32 of them are the encoders' input rather
/// than a decoder's, so the difference is the directory this run skips.
const EXPECTED_FILES: usize = 446;

#[test]
fn hpack_decodes_every_case_in_the_fetched_vector_corpus() {
    let run = run_corpus();
    assert!(
        run.failures.is_empty(),
        "{} case(s) ran, and these failed:\n{}",
        run.cases,
        run.failures.join("\n")
    );
    assert_eq!(
        run.cases, EXPECTED_CASES,
        "the corpus changed size. Re-read what moved before editing this number"
    );
    assert_eq!(run.files, EXPECTED_FILES);
    println!("hpack: {} case(s) across {} file(s)", run.cases, run.files);
}

#[test]
fn hpack_refuses_padding_that_is_not_the_end_of_string_code() {
    // ⛔ The specification names three decoding errors and each is a client
    // doing something a capture should not paper over. Padding that is not all
    // ones is not padding.
    let why = decode_huffman(&[0xff, 0xff, 0xff, 0xf0]).expect_err("padding of zeroes is refused");
    assert!(why.contains("padding"), "{why}");
}

#[test]
fn hpack_refuses_padding_longer_than_seven_bits() {
    // `0x1f` is one 5-bit symbol and three bits of padding; adding a whole byte
    // of ones puts the padding over the limit.
    let why = decode_huffman(&[0x07, 0xff]).expect_err("over-long padding is refused");
    assert!(why.contains("padding"), "{why}");
}

#[test]
fn hpack_records_whether_each_field_was_huffman_coded() {
    // ⛔ It is a choice the encoder made, it differs between clients, and it
    // cannot be added later because the capture is gone.
    //
    // Two literal fields with an indexed name (`:authority`, index 1): the
    // first value Huffman-coded, the second plain.
    let mut decoder = Decoder::default();
    // ⚠ The value bytes are written as a byte array rather than as a hex
    // string. A long hex run in a source file is a shape the secret sweep
    // refuses on purpose, and narrowing that rule to let a test literal
    // through would be widening it.
    let mut block = vec![0x41, 0x8c];
    block.extend_from_slice(&[
        0xf1, 0xe3, 0xc2, 0xe5, 0xf2, 0x3a, 0x6b, 0xa0, 0xab, 0x90, 0xf4, 0xff,
    ]);
    block.push(0x41);
    block.push(0x03);
    block.extend_from_slice(b"abc");
    let decoded = decoder.decode(&block).expect("both fields decode");
    assert_eq!(decoded.fields.len(), 2);

    assert_eq!(decoded.fields[0].name, ":authority");
    assert_eq!(decoded.fields[0].value.as_deref(), Some("www.example.com"));
    assert_eq!(decoded.fields[0].value_huffman, Some(true));
    assert_eq!(decoded.fields[1].value.as_deref(), Some("abc"));
    assert_eq!(decoded.fields[1].value_huffman, Some(false));

    // ⚠ A name taken from an index was not CODED at all, so the flag is absent
    // rather than false. False would say the encoder chose plain text.
    assert_eq!(decoded.fields[0].name_huffman, None);
    assert_eq!(decoded.fields[0].indexing, Indexing::Incremental);
}

#[test]
fn hpack_records_which_indexing_form_the_encoder_chose() {
    // The four forms, in one block: indexed, incremental, without indexing and
    // never indexed.
    let mut decoder = Decoder::default();
    let mut block = vec![0x82]; // indexed: :method GET
    block.extend_from_slice(&[0x40, 0x01, b'a', 0x01, b'b']); // incremental
    block.extend_from_slice(&[0x00, 0x01, b'c', 0x01, b'd']); // without indexing
    block.extend_from_slice(&[0x10, 0x01, b'e', 0x01, b'f']); // never indexed
    let decoded = decoder.decode(&block).expect("all four decode");
    let forms: Vec<Indexing> = decoded.fields.iter().map(|f| f.indexing).collect();
    assert_eq!(
        forms,
        vec![
            Indexing::Indexed,
            Indexing::Incremental,
            Indexing::WithoutIndexing,
            Indexing::NeverIndexed,
        ]
    );
    // ⛔ Only the incremental one reached the dynamic table.
    assert_eq!(decoder.entries(), 1);
}

#[test]
fn hpack_records_a_dynamic_table_size_update() {
    // ⭐ A choice the encoder made about a table it owns, and one a settings
    // value does not predict.
    let mut decoder = Decoder::default();
    let decoded = decoder.decode(&[0x20, 0x3f, 0xe1, 0x1f]).expect("decodes");
    assert_eq!(decoded.table_size_updates, vec![0, 4096]);
    assert!(decoded.fields.is_empty());
}

#[test]
fn hpack_refuses_a_size_update_above_what_the_peer_was_allowed() {
    // ⛔ An error rather than a clamp. A peer asking for a table it was not
    // allowed is a peer this decoder cannot follow, and clamping would decode
    // every later index against a table the encoder does not have.
    let mut decoder = Decoder::with_settings_max(4096);
    let why = decoder
        .decode(&[0x3f, 0xe1, 0x3f])
        .expect_err("an over-large update is refused");
    assert!(why.contains("allowed"), "{why}");
}

#[test]
fn hpack_refuses_an_index_that_names_nothing() {
    let mut decoder = Decoder::default();
    let why = decoder
        .decode(&[0xff, 0x00])
        .expect_err("an index past the table is refused");
    assert!(why.contains("dynamic table"), "{why}");

    let why = decoder.decode(&[0x80]).expect_err("index 0 is refused");
    assert!(why.contains("index 0"), "{why}");
}

#[test]
fn hpack_evicts_from_the_dynamic_table_by_size_and_not_by_count() {
    // ⚠ The entry size the specification counts includes 32 octets of
    // overhead, so a table sized by name and value lengths alone holds too
    // much and resolves later indices against entries a real decoder evicted.
    // Two octets of name and value plus 32 of overhead is 34, so a 68-octet
    // table holds exactly two of these and no more.
    let mut decoder = Decoder::with_settings_max(68);
    decoder
        .decode(&[0x40, 0x01, b'a', 0x01, b'b'])
        .expect("the first entry fits");
    assert_eq!(decoder.entries(), 1);
    assert_eq!(decoder.size(), 1 + 1 + 32);

    decoder
        .decode(&[0x40, 0x01, b'c', 0x01, b'd'])
        .expect("the second fits too");
    assert_eq!(decoder.entries(), 2);
    assert_eq!(decoder.size(), 68);

    decoder
        .decode(&[0x40, 0x01, b'e', 0x01, b'f'])
        .expect("the third evicts");
    assert_eq!(decoder.entries(), 2, "eviction is by size, not by count");
    assert_eq!(decoder.size(), 68);
}

// ---------------------------------------------------------------------------
// HARNESS-04, the half that reaches a capture rather than a decoder.
// ---------------------------------------------------------------------------

/// Read the committed HTTP/2 fixture as a capture would.
fn fixture_capture(policy: ValuePolicy) -> (b_ids_harness::Http2Capture, Vec<b_ids_harness::Note>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/h2-connection.hex");
    let text = std::fs::read_to_string(&path).expect("the committed fixture is readable");
    let bytes = unhex(&text).expect("the committed fixture is hex");
    let mut notes = Vec::new();
    let capture = b_ids_harness::h2::parse_connection(&bytes, policy, &mut notes)
        .expect("the committed fixture is an HTTP/2 connection");
    (capture, notes)
}

#[test]
fn hpack_reads_the_header_order_off_a_captured_connection() {
    // ⛔ This is the whole reason the decoder exists. Header order is a
    // first-class part of the fingerprint and it is behind HPACK.
    let (capture, _) = fixture_capture(ValuePolicy::NamesOnly);
    let names: Vec<&str> = capture.headers.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            ":method",
            ":authority",
            ":scheme",
            ":path",
            "x-fixture-marker",
            "user-agent",
            // ⭐ TWICE, AND IN PLACE. `SCHEMA-14`: a credential keeps its name
            // and its position and loses its value, because a gap nothing marks
            // is a sequence a consumer believes is whole and is not.
            "cookie",
            "cookie",
            "x-fixture-marker",
        ]
    );
}

#[test]
fn hpack_fills_the_pseudo_header_order_from_the_recorded_fields() {
    // ⚠ Derived from the header list rather than counted separately. Two
    // places holding one quantity is how the copy a reader trusts becomes the
    // wrong one.
    let (capture, _) = fixture_capture(ValuePolicy::NamesOnly);
    assert_eq!(
        capture.half.pseudo_header_order,
        vec![":method", ":authority", ":scheme", ":path"]
    );
    // ⭐ Which is the order the Akamai string renders as `m,a,s,p`.
    assert!(capture.half.akamai_text().ends_with("|m,a,s,p"));
}

#[test]
fn hpack_drops_a_credential_the_dynamic_table_had_to_see() {
    // ⛔ The two requirements pull in opposite directions and both hold. The
    // decoder MUST store `cookie` or every later index resolves against the
    // wrong table; the capture MUST NOT record its VALUE.
    //
    // ⭐ THE TITLE STAYS AND WHAT IS DROPPED CHANGED. `SCHEMA-14` made the
    // NAME and the POSITION recordable, because whether a credential was sent
    // and where is a fingerprint signal that carries no secret. The value is
    // what is dropped, and this test asserts exactly that.
    let (capture, _) = fixture_capture(ValuePolicy::WithValues);
    let credentials: Vec<&b_ids_harness::hpack::HeaderRecord> = capture
        .headers
        .iter()
        .filter(|h| h.name.eq_ignore_ascii_case("cookie"))
        .collect();
    assert_eq!(credentials.len(), 2, "{credentials:?}");
    assert!(
        credentials.iter().all(|h| h.value.is_none()),
        "no credential carries a value even with the switch on: {credentials:?}"
    );

    let serialised = serde_json::to_string(&capture).expect("serialises");
    assert!(
        !serialised.contains("not-a-real-value"),
        "a credential value reached the capture"
    );

    // ⭐ And the proof that the table DID see it: the last field is an index
    // one slot further down than the marker was inserted at. It names the
    // marker only because the credential took the slot in front of it.
    assert_eq!(
        capture.headers.last().expect("a last field").name,
        "x-fixture-marker"
    );
    assert_eq!(
        capture
            .headers
            .last()
            .expect("a last field")
            .value
            .as_deref(),
        Some("kept")
    );
}

#[test]
fn hpack_records_no_header_value_by_default() {
    // ⛔ Names only is the default, because a switch that has to be turned OFF
    // for safety is a switch that ships on.
    let (capture, _) = fixture_capture(ValuePolicy::NamesOnly);
    assert!(capture.headers.iter().all(|h| h.value.is_none()));
    let serialised = serde_json::to_string(&capture).expect("serialises");
    assert!(!serialised.contains("a-fixture-value"), "{serialised}");

    let (with_values, _) = fixture_capture(ValuePolicy::WithValues);
    assert_eq!(
        with_values.headers[1].value.as_deref(),
        Some("fixture.invalid")
    );
}

#[test]
fn hpack_records_the_indexing_form_of_every_captured_field() {
    // ⛔ A choice the encoder made, and one that cannot be added later because
    // the capture is gone.
    let (capture, _) = fixture_capture(ValuePolicy::NamesOnly);
    let forms: Vec<Indexing> = capture.headers.iter().map(|h| h.indexing).collect();
    assert_eq!(
        forms,
        vec![
            Indexing::Indexed,
            Indexing::Incremental,
            Indexing::Indexed,
            Indexing::Indexed,
            Indexing::Incremental,
            Indexing::WithoutIndexing,
            // ⭐ The two credential fields keep their indexing form too. It is a
            // choice the encoder made about a header it sent, and it carries no
            // part of the value.
            Indexing::Incremental,
            Indexing::Indexed,
            Indexing::Indexed,
        ]
    );
}

#[test]
fn hpack_the_transcribed_table_is_a_canonical_huffman_code() {
    // ⛔ The decoder counts forward from the first code at each bit length,
    // which is correct only where the codes at each length are consecutive and
    // ascending in symbol order. Nothing else in the tree states that, and a
    // table that stopped being canonical would decode plausible nonsense.
    b_ids_harness::hpack::check_table_is_canonical().expect("the table is canonical");
}
