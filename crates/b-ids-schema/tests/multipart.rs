//! SCHEMA-11. The multipart boundary, which is a per-browser surface nobody
//! listed.
//!
//! The acceptance: a fixture of sixteen boundaries from one browser all match
//! the recorded pattern, and a boundary from another browser does not.
//!
//! ⛔ Every test name starts with `multipart`, because
//! `cargo test -p b-ids-schema multipart` is the entry's acceptance command.
//!
//! ⛔ **THE FIXTURES ARE CONSTRUCTED AND THEY SAY SO.** This project has not
//! captured a form submission from any browser. What is measured here is the
//! MATCHER, against boundaries generated to the shape
//! `docs/reference-sweeps/usable.md` section 8 records from reading somebody
//! else's client at a named commit. ⚠ No profile carries this field until this
//! project measures one itself, and `http.multipart_boundary` is `None`
//! everywhere in the corpus today.

mod support;

use b_ids_schema::http::{BoundaryAlphabet, MultipartBoundary};

use support::{schema, validate};

/// The pattern one client records for one browser family.
///
/// ⚠ `----WebKitFormBoundary` plus sixteen alphanumerics.
/// `docs/reference-sweeps/usable.md` section 8. ⛔ Inherited by reading, not
/// measured here.
fn webkit() -> MultipartBoundary {
    MultipartBoundary {
        prefix: "----WebKitFormBoundary".to_owned(),
        random_len: 16,
        alphabet: BoundaryAlphabet::Alphanumeric,
    }
}

/// The pattern the same client records for another family.
///
/// ⚠ `----geckoformboundary` plus thirty-two hexadecimal characters.
fn gecko() -> MultipartBoundary {
    MultipartBoundary {
        prefix: "----geckoformboundary".to_owned(),
        random_len: 32,
        alphabet: BoundaryAlphabet::LowerHex,
    }
}

/// Sixteen boundaries of one shape, drawn without a random number generator.
///
/// ⚠ **Deterministic on purpose.** A test that drew randomly would pass or fail
/// on a seed, and a matcher checked against one lucky draw is a matcher nobody
/// has checked. These walk the alphabet instead, so every character of it is
/// exercised across the set.
fn sixteen(pattern: &MultipartBoundary, alphabet: &[char]) -> Vec<String> {
    (0..16)
        .map(|n| {
            let body: String = (0..pattern.random_len)
                .map(|i| alphabet[(n * 7 + i * 3) % alphabet.len()])
                .collect();
            format!("{}{body}", pattern.prefix)
        })
        .collect()
}

const ALPHANUMERIC: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'a',
    'b', 'c', 'x', 'y', 'z',
];
const LOWER_HEX: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[test]
fn multipart_sixteen_boundaries_of_one_browser_all_match_its_pattern() {
    let pattern = webkit();
    let drawn = sixteen(&pattern, ALPHANUMERIC);
    assert_eq!(drawn.len(), 16);
    for boundary in &drawn {
        assert!(pattern.matches(boundary), "{boundary}");
    }
    // ⛔ And they are not all the same string, which is what makes the set a
    // sample rather than one value repeated sixteen times.
    let distinct: std::collections::BTreeSet<&String> = drawn.iter().collect();
    assert_eq!(distinct.len(), 16, "{drawn:?}");
}

#[test]
fn multipart_a_boundary_from_another_browser_does_not_match() {
    // ⭐ THE HALF THAT MAKES THE PATTERN WORTH RECORDING. A field that matched
    // everything would say nothing about which browser sent a body.
    let pattern = webkit();
    for boundary in sixteen(&gecko(), LOWER_HEX) {
        assert!(!pattern.matches(&boundary), "{boundary}");
    }
    for boundary in sixteen(&webkit(), ALPHANUMERIC) {
        assert!(!gecko().matches(&boundary), "{boundary}");
    }
}

#[test]
fn multipart_the_length_is_checked_as_well_as_the_prefix() {
    // ⛔ A pattern that only asserted the prefix would match a boundary from any
    // browser whose prefix happens to agree, and one that only counted would
    // match a prefix that does not. Both halves, on the same value.
    let pattern = webkit();
    assert!(!pattern.matches("----WebKitFormBoundaryabc"), "too short");
    assert!(
        !pattern.matches("----WebKitFormBoundary0123456789abcdefgh"),
        "too long"
    );
    assert!(!pattern.matches("----WebKitFormBoundary"), "no random part");
    assert!(!pattern.matches(""), "empty");
}

#[test]
fn multipart_a_character_outside_the_alphabet_does_not_match() {
    // ⚠ Sixteen characters of the right COUNT is not the right SHAPE. A hyphen
    // is the case that matters: it appears in the prefix, so a matcher reading
    // the whole string rather than the tail would accept it.
    let pattern = webkit();
    assert!(!pattern.matches("----WebKitFormBoundary0123456789abcde-"));
    assert!(!pattern.matches("----WebKitFormBoundary0123456789abcde "));
    assert!(gecko().matches("----geckoformboundary0123456789abcdef0123456789abcdef"));
    // ⛔ Lower hex refuses an upper-case digit, because the two are different
    // generators and a matcher that accepted both would report the wrong one.
    assert!(!gecko().matches("----geckoformboundaryABCDEF0123456789abcdef0123456789"));
}

#[test]
fn multipart_a_pattern_with_no_random_part_is_a_constant_and_is_refused() {
    // ⛔ THE ONE THING THIS FIELD MUST NOT BECOME. The boundary is drawn per
    // request, like GREASE, and a pattern with a zero-length random part is one
    // captured value wearing the shape of a rule.
    let constant = MultipartBoundary {
        prefix: "----WebKitFormBoundaryAbCdEfGhIjKlMnOp".to_owned(),
        random_len: 0,
        alphabet: BoundaryAlphabet::Alphanumeric,
    };
    let problems = constant.problems();
    assert!(
        problems.iter().any(|p| p.contains("records a constant")),
        "{problems:?}"
    );
    assert!(webkit().problems().is_empty());
    assert!(gecko().problems().is_empty());
}

#[test]
fn multipart_no_profile_in_this_tree_claims_a_boundary() {
    // ⛔ THE PROVENANCE RULE, held by a test rather than by a sentence. The
    // pattern above is INHERITED by reading somebody else's client, and nothing
    // inherited is published as data. A profile gains this field when this
    // project measures a form submission itself.
    assert!(
        b_ids_schema::fixture::profile()
            .http
            .multipart_boundary
            .is_none()
    );
    assert!(
        b_ids_schema::fixture::profile_with_header_values()
            .http
            .multipart_boundary
            .is_none()
    );
}

#[test]
fn multipart_the_published_schema_carries_the_pattern() {
    let schema = schema();
    let mut profile =
        serde_json::to_value(b_ids_schema::fixture::profile()).expect("the fixture serialises");
    profile["http"]["multipart_boundary"] = serde_json::json!({
        "prefix": "----WebKitFormBoundary",
        "random_len": 16,
        "alphabet": "alphanumeric",
    });
    assert!(
        validate(&schema, &profile).is_empty(),
        "{:?}",
        validate(&schema, &profile)
    );

    // ⛔ And the schema refuses the constant too, at the contract rather than
    // only in the type.
    profile["http"]["multipart_boundary"]["random_len"] = serde_json::json!(0);
    let problems = validate(&schema, &profile);
    assert!(
        problems.iter().any(|p| p.contains("below the minimum")),
        "{problems:?}"
    );
}

#[test]
fn multipart_every_alphabet_names_itself_and_is_checkable() {
    // ⚠ An enum rather than a literal character set, and the vocabulary is
    // compared across profiles: a free string would fail silently on an
    // ordering or a spelling.
    for alphabet in BoundaryAlphabet::all() {
        assert!(!alphabet.as_str().is_empty());
        assert_eq!(alphabet.to_string(), alphabet.as_str());
    }
    assert!(BoundaryAlphabet::LowerHex.contains('f'));
    assert!(!BoundaryAlphabet::LowerHex.contains('g'));
    assert!(!BoundaryAlphabet::LowerHex.contains('F'));
    assert!(BoundaryAlphabet::UpperHex.contains('F'));
    assert!(!BoundaryAlphabet::UpperHex.contains('f'));
    assert!(BoundaryAlphabet::Alphanumeric.contains('z'));
    assert!(!BoundaryAlphabet::Alphanumeric.contains('-'));
}
