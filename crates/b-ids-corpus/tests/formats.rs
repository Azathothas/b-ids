//! SCHEMA-08. Every generated format, from one generator, round-tripped.
//!
//! ⛔ Every test name starts with `formats`, because
//! `cargo test -p b-ids-corpus formats` is what runs this file alone.

use b_ids_corpus::formats::{FLAT_COLUMNS, Format, flat_row, read_back, read_flat, render};
use b_ids_schema::Profile;

/// Two profiles that differ in every flat column, so a row taken from the wrong
/// profile cannot pass by looking like the right one.
fn corpus() -> Vec<Profile> {
    let first = b_ids_schema::fixture::profile();
    let mut second = b_ids_schema::fixture::profile();
    second.browser.version = "152.0.7977.75".to_owned();
    second.browser.major = 152;
    second.browser.branded = false;
    second.captured.at = "2026-09-02T00:00:00Z".to_owned();
    second.id = b_ids_schema::ProfileId::derive(
        &second.browser.name,
        &second.browser.version,
        &second.platform_token(),
        second.browser.channel,
    );
    vec![first, second]
}

#[test]
fn formats_the_lossless_ones_round_trip_to_byte_identical_canonical_json() {
    // ⛔ THE ACCEPTANCE'S CORE. A format with a writer and no reader can only be
    // checked against the thing that wrote it, which is the shape this entry
    // was re-scoped to avoid.
    let profiles = corpus();
    let canonical = render(Format::Json, &profiles).expect("the canonical form renders");

    for format in Format::all().into_iter().filter(|f| f.lossless()) {
        let text = render(format, &profiles).expect("it renders");
        let back = read_back(format, &text).expect("it reads back");
        let again = render(Format::Json, &back).expect("the canonical form renders");
        assert_eq!(
            again, canonical,
            "{format} did not round trip to byte-identical canonical JSON"
        );
    }
}

#[test]
fn formats_a_lossy_one_round_trips_the_documented_subset_and_refuses_to_be_a_profile() {
    // ⚠ THE SUBSET IS ASSERTED, not the whole profile. A spreadsheet cannot
    // hold a nested extension list, and a test that pretended otherwise would
    // be asserting something the format never promised.
    let profiles = corpus();
    for format in [Format::Csv, Format::Tsv] {
        let text = render(format, &profiles).expect("it renders");
        let rows = read_flat(format, &text).expect("its rows read back");
        assert_eq!(rows.len(), profiles.len(), "{format}");
        for (row, profile) in rows.iter().zip(&profiles) {
            assert_eq!(row.as_slice(), flat_row(profile).as_slice(), "{format}");
        }
        // ⛔ And it refuses to become a profile rather than approximating one.
        let refusal = read_back(format, &text).expect_err("a lossy format is not a profile");
        assert!(refusal.contains("lossy"), "{refusal}");
    }
}

#[test]
fn formats_are_deterministic_so_two_runs_are_byte_identical() {
    // ⛔ THE OTHER HALF OF THE ACCEPTANCE. A generator that read a clock or a
    // hash seed would produce a diff on every run, and a diff on every run is a
    // published artefact nobody can tell a real change from.
    let profiles = corpus();
    for format in Format::all() {
        let once = render(format, &profiles).expect("it renders");
        let twice = render(format, &profiles).expect("it renders");
        assert_eq!(once, twice, "{format} is not deterministic");
    }
}

#[test]
fn formats_every_lossy_file_says_what_it_leaves_out_in_its_own_header() {
    // ⛔ IN THE FILE, not in a document beside it. A reader arriving at a
    // rendered table on the web has nothing else to read.
    let profiles = corpus();
    let markdown = render(Format::Markdown, &profiles).expect("it renders");
    assert!(markdown.contains("Do not edit"), "{markdown}");
    assert!(
        markdown.contains("are in the JSON"),
        "the header names where the rest of a profile is: {markdown}"
    );
    // ⚠ The delimited pair carry their subset in the header ROW rather than in
    // prose, because a comment line would break every spreadsheet that opens
    // them. `read_flat` is what asserts the row against the documented subset.
    for format in [Format::Csv, Format::Tsv] {
        let text = render(format, &profiles).expect("it renders");
        let header = text.lines().next().expect("a header row");
        for column in FLAT_COLUMNS {
            assert!(header.contains(column), "{format}: {header}");
        }
    }
}

#[test]
fn formats_an_edited_file_is_not_what_the_generator_produces() {
    // ⛔ THE RULE THE ENTRY STATES, made a test. "Never hand-edit a generated
    // format" is enforceable only if something compares the file with what the
    // generator would write, and this is that comparison in miniature.
    let profiles = corpus();
    let generated = render(Format::Csv, &profiles).expect("it renders");
    let edited = generated.replace("152.0.7977.75", "152.0.7977.99");
    assert_ne!(edited, generated, "the edit did not apply");
    let rows = read_flat(Format::Csv, &edited).expect("it still parses");
    assert_ne!(
        rows[1].as_slice(),
        flat_row(&profiles[1]).as_slice(),
        "an edited row must not compare equal to what the generator would write"
    );
}

#[test]
fn formats_a_name_the_generator_does_not_have_is_refused() {
    assert_eq!(Format::parse("json"), Some(Format::Json));
    assert_eq!(Format::parse("md"), Some(Format::Markdown));
    // ⛔ None rather than a default. YAML, TOML, SQLite, CBOR, MessagePack and
    // Protobuf are SCHEMA-12's, and answering with JSON would publish a file
    // under a name it is not.
    assert_eq!(Format::parse("yaml"), None);
    assert_eq!(Format::parse("sqlite"), None);
    assert!(Format::names().contains("ndjson"));
}

#[test]
fn formats_an_empty_corpus_generates_every_file_and_no_rows() {
    // ⚠ The edge case a publisher hits on day one. Five files, each valid, none
    // of them carrying a row, rather than a crash or a missing file.
    let empty: Vec<Profile> = Vec::new();
    for format in Format::all() {
        let text = render(format, &empty).expect("it renders");
        if format.lossless() {
            let back = read_back(format, &text).expect("it reads back");
            assert!(back.is_empty(), "{format}");
        } else if format != Format::Markdown {
            let rows = read_flat(format, &text).expect("its header still parses");
            assert!(rows.is_empty(), "{format}");
        }
    }
}
