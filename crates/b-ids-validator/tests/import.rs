//! VALID-02. Run it over the prior art, and publish what it finds.
//!
//! The acceptance: the report names at least the three violations the sweep
//! located, each with its file, its line and the check it failed, and re-running
//! against the same commits produces byte-identical output.
//!
//! ⛔ Every test name starts with `import`, because
//! `cargo test -p b-ids-validator import` is the entry's acceptance command.
//!
//! ⚠ **These tests read the tracked reference corpus**, which is why it is
//! tracked. A conclusion nobody can re-check is an opinion, and a test that
//! could not open the file it cites would be one.

use std::path::{Path, PathBuf};

use b_ids_validator::{Check, import_references, render_report};

/// The reference corpus, at the commits its provenance files record.
fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("references")
}

fn report() -> String {
    let exhibits = import_references(&corpus()).expect("the corpus is readable");
    render_report(&exhibits)
}

#[test]
fn import_names_every_entry_that_returns_another_version_handshake() {
    let exhibits = import_references(&corpus()).expect("the corpus is readable");
    // ⭐ The sweep located five. The reader found those five and one more, in a
    // second family, which is the argument for reading with a tool rather than
    // by eye.
    let shared: Vec<_> = exhibits
        .iter()
        .filter(|e| e.check == Some(Check::Handshake) && e.message.contains("handshake, which"))
        .collect();
    assert!(
        shared.len() >= 6,
        "expected at least six shared handshakes, got {}: {shared:#?}",
        shared.len()
    );
    for module in [
        "chrome_101",
        "chrome_104",
        "chrome_107",
        "chrome_110",
        "chrome_116",
    ] {
        assert!(
            shared.iter().any(|e| e.message.starts_with(module)),
            "{module} is not in the report"
        );
    }
    // ⛔ Every exhibit names a file and a line, because a finding nobody can
    // open is a claim.
    for exhibit in &shared {
        assert!(!exhibit.file.is_empty(), "{exhibit:?}");
        assert!(exhibit.line > 0, "{exhibit:?}");
    }
}

#[test]
fn import_names_the_cipher_table_served_to_every_version() {
    let exhibits = import_references(&corpus()).expect("the corpus is readable");
    let stale: Vec<_> = exhibits
        .iter()
        .filter(|e| e.message.contains("cipher list is commented"))
        .collect();
    assert!(
        stale.len() >= 2,
        "expected the chrome and firefox tables, got {stale:#?}"
    );
    assert!(
        stale
            .iter()
            .any(|e| e.message.contains("Chrome v92") && e.check == Some(Check::Handshake)),
        "{stale:#?}"
    );
}

#[test]
fn import_names_the_family_no_classifier_can_reach() {
    let exhibits = import_references(&corpus()).expect("the corpus is readable");
    let dead: Vec<_> = exhibits.iter().filter(|e| e.check.is_none()).collect();
    assert_eq!(dead.len(), 1, "{dead:#?}");
    assert!(dead[0].message.contains("\"edge\""), "{:?}", dead[0]);
    // ⚠ It is deliberately NOT one of the eight checks. VALID-03 is the entry
    // that makes dead data something this project can check over its own
    // corpus, and reporting it under a check it does not fail would be worse
    // than reporting it as what it is.
    assert!(dead[0].check.is_none());
}

#[test]
fn import_produces_byte_identical_output_across_runs() {
    // ⛔ The acceptance says so in as many words. A directory walk does not
    // promise an order, so a report that differed between two runs over one
    // tree would be a report nobody could diff.
    assert_eq!(report(), report());
}

#[test]
fn import_returns_its_exhibits_in_sorted_order() {
    // ⚠ THIS IS THE ASSERTION THAT CAN FAIL, and the one above is not.
    // Removing the sort was planted and every test still passed, because the
    // walk underneath happens to be stable on this host: the file list is
    // sorted, the grouping is an ordered map, and the reference list is a
    // constant. Equality between two runs cannot tell a sorted answer from an
    // incidentally stable one. This can.
    let exhibits = import_references(&corpus()).expect("the corpus is readable");
    assert!(exhibits.is_sorted(), "{exhibits:#?}");
}

#[test]
fn import_refuses_a_corpus_it_cannot_read() {
    // ⛔ A reader that finds nothing is BLIND, not clean. Without this, a
    // reference edited into a shape the reader does not know would drop its
    // shipped violations out of the report and the run would look green.
    let empty = std::env::temp_dir().join(format!("b-ids-import-{}", std::process::id()));
    std::fs::create_dir_all(&empty).expect("a scratch directory");
    let refused = import_references(&empty).expect_err("an empty directory is refused");
    assert!(refused.contains("no known reference tree"), "{refused}");
    let _ = std::fs::remove_dir_all(&empty);
}

#[test]
fn import_report_carries_the_check_name_beside_every_exhibit() {
    let text = report();
    assert!(
        text.starts_with("b-ids-validator import report/1"),
        "{text}"
    );
    assert!(text.contains("handshake"), "{text}");
    assert!(text.contains("unreachable-data"), "{text}");
    assert!(text.contains("exhibit(s)"), "{text}");
}
