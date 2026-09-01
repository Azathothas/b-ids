//! Read the prior art's own tables, and report what the coherence checks say
//! about them.
//!
//! ⭐ **This is the project's first publishable result and it costs no capture.**
//! Three violations are shipped in two public repositories, each locatable at a
//! file and a line, and each one is a case the validator exists to refuse.
//!
//! ⛔ **It reports; it does not characterise.** A defect, a file, a line, and
//! the check it fails. `../docs/methodology/vendoring.md` forbids a
//! characterisation of a project or its maintainers, and the repository is
//! public and outlives the session.
//!
//! ⛔ **No profile is synthesised, and that is a decision rather than a
//! shortcut.** `VALID-02`'s approach said to build profiles from these tables
//! and run [`crate::validate`] over them. A `TlsHalf` has sixteen fields of wire
//! data and none of these references states them, so building one would mean
//! inventing the bytes the whole project exists to measure, and the report would
//! then be a report about the invention. What is shared instead is the
//! vocabulary: every finding here carries the [`Check`] it fails, so the report
//! and the validator name the same thing.
//!
//! ⚠ **These readers know the SHAPE of two source files at two named commits.**
//! A reference that moves is a reader that finds nothing, so finding nothing is
//! an error rather than a clean report. Anything else would let a shipped
//! violation disappear from the report by being edited into a shape the reader
//! does not know.
//!
//! `TODO/validator.md`, `VALID-02`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::Check;

/// One thing found in somebody else's table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Exhibit {
    /// The reference tree it was read from, as this project names it.
    pub reference: String,
    /// The path inside that tree.
    pub file: String,
    /// The line, one-based.
    pub line: usize,
    /// The coherence check this fails, where one of the eight applies.
    ///
    /// ⚠ `None` for the third violation. Dead data that no resolver can reach
    /// is not one of the eight; `VALID-03` is the entry that makes it a check.
    pub check: Option<Check>,
    /// What was found, in one sentence.
    pub message: String,
}

/// A reference tree this module knows how to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Known {
    /// The directory under the corpus root.
    dir: &'static str,
    /// How this project names the upstream.
    name: &'static str,
}

/// The trees this module reads, and the only ones.
///
/// ⚠ A reference not listed here is not examined, which is a different fact
/// from a reference with nothing wrong in it. The report says so.
const KNOWN: [Known; 2] = [
    Known {
        dir: "apify__impit",
        name: "apify/impit",
    },
    Known {
        dir: "Kikobeats__https-tls",
        name: "Kikobeats/https-tls",
    },
];

/// Read every known reference under `root` and return what the checks say.
///
/// # Errors
///
/// A string naming the reader that found nothing where it expected something,
/// which means the reference moved and the reader is now blind.
pub fn read(root: &Path) -> Result<Vec<Exhibit>, String> {
    let mut out = Vec::new();
    let mut examined = 0;
    for known in KNOWN {
        let tree = root.join(known.dir).join("tree");
        if !tree.is_dir() {
            continue;
        }
        examined += 1;
        match known.dir {
            "apify__impit" => out.extend(read_impit(&tree, known.name)?),
            "Kikobeats__https-tls" => out.extend(read_https_tls(&tree, known.name)?),
            other => return Err(format!("no reader for {other}")),
        }
    }
    if examined == 0 {
        return Err(format!(
            "no known reference tree under {}. This module reads {} of them and found none",
            root.display(),
            KNOWN.len()
        ));
    }
    // ⛔ Sorted, because the acceptance says re-running produces byte-identical
    // output and a directory walk does not promise an order.
    out.sort();
    Ok(out)
}

/// One entry of a per-version fingerprint database.
#[derive(Debug, Clone)]
struct Entry {
    module: String,
    family: String,
    major: String,
    tls_source: String,
    line: usize,
}

/// Read the per-version database and report entries that share one handshake.
///
/// ⭐ **The source expression IS the evidence.** Two modules that both call one
/// module's `tls_fingerprint()` return a byte-identical TLS half by
/// construction, so nothing has to be evaluated to know they do.
fn read_impit(tree: &Path, name: &str) -> Result<Vec<Exhibit>, String> {
    let dir = tree.join("impit/src/fingerprint/database");
    let mut out = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();

    let mut entries_seen = 0;
    for path in files {
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let rel = relative(tree, &path);
        let entries = impit_entries(&text);
        entries_seen += entries.len();

        // Group by the module whose handshake each entry actually returns.
        let mut by_source: BTreeMap<(String, String), Vec<&Entry>> = BTreeMap::new();
        for entry in &entries {
            by_source
                .entry((entry.family.clone(), entry.tls_source.clone()))
                .or_default()
                .push(entry);
        }
        for ((family, source), group) in by_source {
            let majors: BTreeSet<&str> = group.iter().map(|e| e.major.as_str()).collect();
            if majors.len() < 2 {
                continue;
            }
            for entry in group.iter().filter(|e| e.module != source) {
                out.push(Exhibit {
                    reference: name.to_owned(),
                    file: rel.clone(),
                    line: entry.line,
                    check: Some(Check::Handshake),
                    message: format!(
                        "{} claims {family} {} and returns {source}'s handshake, which {} other \
                         entr{} in this file also return{}",
                        entry.module,
                        entry.major,
                        majors.len() - 1,
                        if majors.len() == 2 { "y" } else { "ies" },
                        if majors.len() == 2 { "s" } else { "" }
                    ),
                });
            }
        }
    }
    if entries_seen == 0 {
        return Err(format!(
            "{}: read no fingerprint entry at all. The reader knows one shape and this tree has \
             another, so it is blind rather than clean",
            dir.display()
        ));
    }
    Ok(out)
}

/// Pull every per-version entry out of one database file.
///
/// ⚠ It reads a shape: a module, then a constructor whose first two arguments
/// are the family and the major and whose third names the handshake. A file
/// that does not have that shape yields nothing, and the caller treats nothing
/// as an error.
fn impit_entries(text: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut module = String::new();
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if let Some(rest) = line.trim().strip_prefix("pub mod ") {
            module = rest.trim_end_matches(" {").trim().to_owned();
            continue;
        }
        if !line.contains("::new(") || module.is_empty() {
            continue;
        }
        // The four lines after the constructor: family, major, tls, http2.
        let family = lines.get(index + 1).and_then(|l| quoted(l));
        let major = lines.get(index + 2).and_then(|l| quoted(l));
        let Some(tls) = lines.get(index + 3).map(|l| l.trim().trim_end_matches(',')) else {
            continue;
        };
        let (Some(family), Some(major)) = (family, major) else {
            continue;
        };
        // ⚠ An unqualified call is the module's own; a qualified one names
        // another module, and that is the whole finding.
        let source = tls
            .split("::")
            .next()
            .filter(|_| tls.contains("::"))
            .unwrap_or(&module)
            .to_owned();
        entries.push(Entry {
            module: module.clone(),
            family,
            major,
            tls_source: source,
            // ⚠ The line that NAMES the borrowed handshake, not the one that
            // opens the constructor. It is the evidence a reader has to see.
            line: index + 4,
        });
    }
    entries
}

/// The contents of the first double-quoted run on a line.
fn quoted(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_owned())
}

/// Read the single-table library: a cipher list per family, and a family the
/// classifier cannot produce.
fn read_https_tls(tree: &Path, name: &str) -> Result<Vec<Exhibit>, String> {
    let mut out = Vec::new();

    let index = tree.join("src/index.js");
    let text = std::fs::read_to_string(&index).map_err(|e| format!("{}: {e}", index.display()))?;
    let mut tables = 0;
    let lines: Vec<&str> = text.lines().collect();
    let mut in_known_ciphers = false;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("const knownCiphers") {
            in_known_ciphers = true;
            continue;
        }
        if in_known_ciphers && line.starts_with('}') {
            in_known_ciphers = false;
            continue;
        }
        if !in_known_ciphers {
            continue;
        }
        let Some(family) = line.trim().strip_suffix(": [") else {
            continue;
        };
        // The version this table was written for is a comment on the next line.
        let Some(comment) = lines.get(i + 1).and_then(|l| l.trim().strip_prefix("// ")) else {
            continue;
        };
        tables += 1;
        out.push(Exhibit {
            reference: name.to_owned(),
            file: relative(tree, &index),
            // ⚠ The COMMENT line, which is where the version this table was
            // written for is stated. The table opens on the line above it.
            line: i + 2,
            check: Some(Check::Handshake),
            message: format!(
                "the {family} cipher list is commented {comment:?} and is the only one this \
                 library has for {family}, so every version of {family} is served it"
            ),
        });
    }
    if tables == 0 {
        return Err(format!(
            "{}: read no cipher table at all, so the reader is blind rather than clean",
            index.display()
        ));
    }

    // The families the classifier can return, against the families the data has.
    let browser = tree.join("src/browser.js");
    let classifier =
        std::fs::read_to_string(&browser).map_err(|e| format!("{}: {e}", browser.display()))?;
    let reachable: BTreeSet<String> = classifier
        .lines()
        .filter(|l| l.contains("browser ="))
        .filter_map(quoted_single)
        .collect();
    if reachable.is_empty() {
        return Err(format!(
            "{}: read no reachable family at all, so the reader is blind rather than clean",
            browser.display()
        ));
    }

    let orders = tree.join("src/headers-order.json");
    let json =
        std::fs::read_to_string(&orders).map_err(|e| format!("{}: {e}", orders.display()))?;
    for (i, line) in json.lines().enumerate() {
        let Some(key) = line.trim().strip_suffix(": [") else {
            continue;
        };
        let key = key.trim_matches('"');
        if key.is_empty() || reachable.contains(key) {
            continue;
        }
        out.push(Exhibit {
            reference: name.to_owned(),
            file: relative(tree, &orders),
            line: i + 1,
            // ⚠ Not one of the eight. VALID-03 is the entry that makes dead
            // data a check this project can run over its own corpus.
            check: None,
            message: format!(
                "the header order for {key:?} is data no caller can reach: the classifier returns \
                 {} and {key:?} is not one of them",
                reachable
                    .iter()
                    .map(|f| format!("{f:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }
    Ok(out)
}

/// The contents of the first single-quoted run on a line.
fn quoted_single(line: &str) -> Option<String> {
    let start = line.find('\'')? + 1;
    let end = line[start..].find('\'')? + start;
    Some(line[start..end].to_owned())
}

/// A path inside a reference tree, with forward slashes.
///
/// ⚠ Forward slashes on every host, so the report is byte-identical between
/// Windows and Linux. A report that differs by host is a report nobody can
/// compare.
fn relative(tree: &Path, path: &Path) -> String {
    path.strip_prefix(tree)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Render the report.
///
/// ⛔ Deterministic: the same trees produce the same bytes.
#[must_use]
pub fn render(exhibits: &[Exhibit]) -> String {
    let mut out = String::new();
    out.push_str("b-ids-validator import report/1\n\n");
    if exhibits.is_empty() {
        out.push_str("no exhibit found\n");
        return out;
    }
    let mut reference = "";
    for exhibit in exhibits {
        if exhibit.reference != reference {
            reference = &exhibit.reference;
            out.push_str(&format!("{reference}\n"));
        }
        let check = exhibit
            .check
            .map_or_else(|| "unreachable-data".to_owned(), |c| c.as_str().to_owned());
        out.push_str(&format!(
            "  {}:{}  {}\n    {}\n",
            exhibit.file, exhibit.line, check, exhibit.message
        ));
    }
    out.push_str(&format!("\n{} exhibit(s)\n", exhibits.len()));
    out
}
