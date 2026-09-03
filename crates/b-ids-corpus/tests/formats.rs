//! `SCHEMA-08` and `SCHEMA-12`. Every generated format, from one generator,
//! round-tripped.
//!
//! ⛔ Every test name starts with `formats`, because
//! `cargo test -p b-ids-corpus formats` is what runs this file alone.

use b_ids_corpus::formats::{
    Declined, FLAT_COLUMNS, Fidelity, Format, flat_row, proto, read_back, read_flat, read_tree,
    render, strip_nulls, support_matrix,
};
use b_ids_schema::Profile;

/// A value carrying every character one of these formats escapes differently.
///
/// ⛔ **The published corpus carries none of these**, measured on 2026-09-02:
/// no profile of the six holds an apostrophe. So a suite that rendered only
/// real-shaped values would leave the SQL quote-doubling, the TOML basic-string
/// escapes and the delimited quoting all unexercised, and each of those is a
/// place where a wrong escape reads back as a different value rather than as an
/// error.
/// ⚠ The non-ASCII character is an ESCAPE rather than the byte. The value under
/// test is still non-ASCII; the file stays ASCII, which is what
/// `scripts/common/check-markers.sh` holds over every tracked text file.
const ADVERSARIAL: &str = "it's \"quoted\", has a\ttab, a\nnewline, a \\ and \u{e9}";

/// Two profiles that differ in every flat column, so a row taken from the wrong
/// profile cannot pass by looking like the right one.
fn corpus() -> Vec<Profile> {
    let first = b_ids_schema::fixture::profile();
    let mut second = b_ids_schema::fixture::profile();
    second.browser.version = "152.0.7977.75".to_owned();
    second.browser.major = 152;
    second.browser.branded = false;
    second.captured.at = "2026-09-02T00:00:00Z".to_owned();
    second.captured.operator = ADVERSARIAL.to_owned();
    // ⚠ A FLAT column too, because the delimited writers escape and the nested
    // ones do not, so one adversarial value cannot exercise both.
    second.platform.arch = format!("x86_64 {ADVERSARIAL}");
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

    let mut checked = 0_usize;
    for format in Format::all().into_iter().filter(|f| f.lossless()) {
        let text = render(format, &profiles).expect("it renders");
        let back = read_back(format, &text).expect("it reads back");
        let again = render(Format::Json, &back).expect("the canonical form renders");
        assert_eq!(
            again, canonical,
            "{format} did not round trip to byte-identical canonical JSON"
        );
        checked += 1;
    }
    // ⚠ The count, so a format losing its `Lossless` fidelity by accident is a
    // failure here rather than one fewer silent assertion.
    assert_eq!(checked, 4, "four formats are lossless");
}

#[test]
fn formats_toml_round_trips_every_field_that_is_not_null() {
    // ⛔ THE DOCUMENTED SUBSET, ASSERTED AS A VALUE. TOML has no null, so its
    // promise is every field whose value is not one, and `strip_nulls` is that
    // promise rather than a sentence in a comment.
    let profiles = corpus();
    let text = render(Format::Toml, &profiles).expect("it renders");
    let back = read_tree(Format::Toml, &text).expect("it reads back");
    let want = strip_nulls(&serde_json::to_value(&profiles).expect("the canonical tree"));
    assert_eq!(back, want, "toml did not round trip its documented subset");

    // ⛔ And it refuses to become a profile rather than approximating one.
    let refusal = read_back(Format::Toml, &text).expect_err("toml is not a profile");
    assert!(refusal.contains("not null"), "{refusal}");
}

#[test]
fn formats_toml_actually_drops_the_nulls_it_says_it_drops() {
    // ⚠ A test that only compared with `strip_nulls` would pass over a writer
    // that emitted nulls AND a reader that ignored them. This asserts the file.
    let profiles = corpus();
    let text = render(Format::Toml, &profiles).expect("it renders");
    assert!(
        !text.contains("= null"),
        "toml wrote a null it has no spelling for"
    );
    let canonical = serde_json::to_value(&profiles).expect("the canonical tree");
    assert_ne!(
        strip_nulls(&canonical),
        canonical,
        "the fixture carries no null, so this test proves nothing"
    );
}

#[test]
fn formats_the_definition_round_trips_to_the_table_the_corpus_implies() {
    // ⛔ A DEFINITION IS STILL READ BACK. It carries no profile's values, so its
    // round trip is over the messages and fields rather than over data, and a
    // field deleted from the generated text fails here.
    let profiles = corpus();
    let text = render(Format::Protobuf, &profiles).expect("it renders");
    let want = proto::declared(&profiles).expect("the corpus implies a definition");
    let back = proto::parse(&text).expect("the definition reads back");
    assert_eq!(back, want, "the definition did not round trip");

    assert!(
        want.messages.contains_key(proto::ROOT_MESSAGE),
        "the definition declares the profile message"
    );
    // ⚠ A field deleted from the text is caught, which is what makes the
    // comparison above evidence rather than two writers agreeing.
    let mutilated = text.replace("  string id = ", "  // string id = ");
    assert_ne!(mutilated, text, "the mutation did not apply");
    assert_ne!(
        proto::parse(&mutilated).expect("it still parses"),
        want,
        "a deleted field must not compare equal to the corpus's own table"
    );
}

#[test]
fn formats_a_flat_one_round_trips_the_documented_subset_and_refuses_to_be_a_profile() {
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
        let refusal = read_back(format, &text).expect_err("a flat format is not a profile");
        assert!(refusal.contains("flat columns"), "{refusal}");
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
    assert_eq!(support_matrix(), support_matrix());
}

#[test]
fn formats_every_partial_file_says_what_it_leaves_out_in_its_own_header() {
    // ⛔ IN THE FILE, not in a document beside it. A reader arriving at a
    // rendered table on the web has nothing else to read.
    let profiles = corpus();
    let markdown = render(Format::Markdown, &profiles).expect("it renders");
    assert!(markdown.contains("Do not edit"), "{markdown}");
    assert!(
        markdown.contains("are in the JSON"),
        "the header names where the rest of a profile is: {markdown}"
    );

    let toml = render(Format::Toml, &profiles).expect("it renders");
    let header: String = toml.lines().take_while(|l| l.starts_with('#')).collect();
    assert!(header.contains("no null"), "{header}");

    let definition = render(Format::Protobuf, &profiles).expect("it renders");
    let header: String = definition
        .lines()
        .take_while(|l| l.starts_with("//"))
        .collect();
    assert!(
        header.contains("DERIVED FROM THE PUBLISHED CORPUS"),
        "{header}"
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
fn formats_the_dump_says_how_to_load_it_and_carries_the_whole_profile() {
    // ⭐ The dump is lossless BECAUSE of the canonical column, so a test that
    // only checked the flat columns would pass over the field that matters.
    let profiles = corpus();
    let text = render(Format::Sqlite, &profiles).expect("it renders");
    assert!(text.contains("sqlite3 corpus.db < corpus.sql"), "{text}");
    assert!(
        text.contains(b_ids_corpus::formats::sql::CANONICAL_COLUMN),
        "the dump declares the canonical column"
    );
    let inserts = text.lines().filter(|l| l.starts_with("INSERT ")).count();
    assert_eq!(inserts, profiles.len(), "one insert per profile");
}

#[test]
fn formats_a_declined_format_is_absent_from_the_generator_and_named_with_its_reason() {
    // ⛔ BOTH HALVES. A format that is declined and still generable is a
    // published artefact nobody ruled on, and one that is declined and unnamed
    // is a consumer guessing whether anybody thought about it.
    for declined in Declined::all() {
        assert_eq!(
            Format::parse(declined.as_str()),
            None,
            "{declined} is declined and the generator still has it"
        );
        assert!(
            !declined.reason().is_empty(),
            "{declined} is declined with no reason"
        );
        assert!(
            support_matrix().contains(declined.as_str()),
            "the support matrix does not name {declined}"
        );
        assert!(
            support_matrix().contains(declined.reason()),
            "the support matrix does not carry {declined}'s reason"
        );
    }
}

#[test]
fn formats_the_support_matrix_names_every_format_with_its_file_and_its_fidelity() {
    let matrix = support_matrix();
    assert!(matrix.contains("Do not edit"), "{matrix}");
    for format in Format::all() {
        assert!(matrix.contains(&format.file_name()), "{format}: {matrix}");
        assert!(
            matrix.contains(format.fidelity().as_str()),
            "{format}: {matrix}"
        );
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
    assert_eq!(Format::parse("yaml"), Some(Format::Yaml));
    assert_eq!(Format::parse("sqlite"), Some(Format::Sqlite));
    assert_eq!(Format::parse("protobuf"), Some(Format::Protobuf));
    // ⛔ None rather than a default. Answering with JSON would publish a file
    // under a name it is not.
    assert_eq!(Format::parse("cbor"), None);
    assert_eq!(Format::parse("sql"), None);
    assert!(Format::names().contains("ndjson"));
}

#[test]
fn formats_a_name_and_a_file_extension_are_not_the_same_question() {
    // ⚠ Measured against the two that differ, because a test over the seven
    // that agree would pass with `extension` returning `as_str`.
    assert_eq!(Format::Sqlite.file_name(), "corpus.sql");
    assert_eq!(Format::Protobuf.file_name(), "corpus.proto");
    assert_eq!(Format::Yaml.file_name(), "corpus.yaml");
    let names: Vec<String> = Format::all().iter().map(|f| f.file_name()).collect();
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(names.len(), unique.len(), "two formats share a file name");
}

#[test]
fn formats_an_empty_corpus_generates_every_file_and_no_rows() {
    // ⚠ The edge case a publisher hits on day one. Every file valid, none of
    // them carrying a row, rather than a crash or a missing file.
    let empty: Vec<Profile> = Vec::new();
    for format in Format::all() {
        let text = render(format, &empty).expect("it renders");
        match format.fidelity() {
            Fidelity::Lossless => {
                let back = read_back(format, &text).expect("it reads back");
                assert!(back.is_empty(), "{format}");
            }
            Fidelity::NoNulls => {
                let back = read_tree(format, &text).expect("it reads back");
                assert_eq!(back, serde_json::json!([]), "{format}");
            }
            Fidelity::FlatColumns if format != Format::Markdown => {
                let rows = read_flat(format, &text).expect("its header still parses");
                assert!(rows.is_empty(), "{format}");
            }
            _ => assert!(!text.is_empty(), "{format} generated nothing at all"),
        }
    }
}

#[test]
fn formats_the_yaml_reader_refuses_a_document_it_did_not_write() {
    // ⭐ THE READER IS NARROW ON PURPOSE and this is where that is stated as a
    // behaviour. A general YAML parser is what SCHEMA-12 says must not be
    // written, so a document using a style this generator never emits is
    // refused rather than half-understood.
    for document in [
        "browser: chrome\n",
        "- &anchor\n  \"a\": 1\n",
        "\"a\": !!str 1\n",
        "\"a\":\n",
    ] {
        let refusal = b_ids_corpus::formats::yaml::parse(document)
            .expect_err("a document this writer never produces");
        assert!(!refusal.is_empty(), "{document}");
    }
    // ⚠ And the one it does write reads back, so the refusals above are not a
    // reader that refuses everything.
    let text = render(Format::Yaml, &corpus()).expect("it renders");
    b_ids_corpus::formats::yaml::parse(&text).expect("its own output reads back");
}

#[test]
fn formats_a_value_carrying_a_newline_does_not_split_a_record() {
    // ⛔ THE REGRESSION. Both the delimited reader and the dump reader split on
    // LINES, and both formats allow a newline inside a quoted value, so a
    // profile carrying one was written correctly and read back as a row of one
    // field and an insert that looked unterminated. Found 2026-09-02.
    let profiles = corpus();
    assert!(
        ADVERSARIAL.contains('\n'),
        "the fixture no longer carries the character this test is about"
    );
    for format in [Format::Csv, Format::Tsv] {
        let text = render(format, &profiles).expect("it renders");
        assert!(
            text.lines().count() > profiles.len() + 1,
            "{format}: the writer did not put a newline inside a value, so this proves nothing"
        );
        let rows = read_flat(format, &text).expect("its rows read back");
        assert_eq!(rows.len(), profiles.len(), "{format}");
    }
    let dump = render(Format::Sqlite, &profiles).expect("it renders");
    assert_eq!(
        read_back(Format::Sqlite, &dump)
            .expect("the dump reads back")
            .len(),
        profiles.len()
    );
}

#[test]
fn formats_a_null_inside_an_array_is_refused_rather_than_shortened() {
    // ⛔ AN ELEMENT IS NEVER DROPPED. A wire order with one element missing is
    // a different fingerprint, so TOML refuses rather than writing a shorter
    // array than the profile carries.
    let tree = serde_json::json!([{ "alpn": ["h2", null] }]);
    let refusal = b_ids_corpus::formats::toml::render(&tree).expect_err("a null in an array");
    assert!(refusal.contains("null"), "{refusal}");
}
