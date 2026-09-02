//! Every published format, from one generator.
//!
//! ⛔ **JSON is one consumer, not the consumer.** A corpus reachable only by
//! writing a JSON walker is a corpus most people copy values out of by hand,
//! and a value copied by hand is a value that stops matching the day the build
//! moves. `TODO/schema.md`, `SCHEMA-08`.
//!
//! ⭐ **ONE generator, canonical JSON in, every format out.** Nothing here reads
//! the corpus tree: it takes the profiles a caller already loaded, so the
//! layout stays [`crate::route`]'s and the shapes stay this module's.
//!
//! # ⚠ Two of the five are lossy, and each says so in its own header
//!
//! | format | lossless | for |
//! | --- | --- | --- |
//! | [`Format::Json`] | ⭐ canonical. Every other format is generated from it. | the reference form |
//! | [`Format::Ndjson`] | yes | streaming, line-oriented tools |
//! | [`Format::Csv`] | ⚠ no | spreadsheets |
//! | [`Format::Tsv`] | ⚠ no | shell pipelines |
//! | [`Format::Markdown`] | ⚠ no | browsing on the web with no tooling |
//!
//! ⛔ **A lossy format's round trip asserts the DOCUMENTED SUBSET rather than
//! equality**, and the subset is [`FLAT_COLUMNS`] rather than a sentence in a
//! comment, so a column added without a reader is a compile error rather than a
//! silent widening.
//!
//! ⛔ **Never hand-edit a generated format.** If one is ever edited directly the
//! generator has lost, and the round-trip test is what says so.

use b_ids_schema::Profile;

/// A format the corpus is published in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    /// Pretty-printed JSON, one array of profiles. The canonical form.
    Json,
    /// One compact JSON object per line.
    Ndjson,
    /// Comma-separated, one row per profile, the flat columns only.
    Csv,
    /// Tab-separated, one row per profile, the flat columns only.
    Tsv,
    /// A Markdown table, the flat columns only.
    Markdown,
}

impl Format {
    /// Every format, in the order they are generated.
    #[must_use]
    pub fn all() -> [Self; 5] {
        [
            Self::Json,
            Self::Ndjson,
            Self::Csv,
            Self::Tsv,
            Self::Markdown,
        ]
    }

    /// The name a caller writes and a file carries.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Ndjson => "ndjson",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Markdown => "md",
        }
    }

    /// The file this format is written to, under a caller's output directory.
    #[must_use]
    pub fn file_name(self) -> String {
        format!("corpus.{}", self.as_str())
    }

    /// Whether this format carries every field a profile has.
    ///
    /// ⛔ **A lossy format is not a defect and is not hidden.** A spreadsheet
    /// cannot hold a nested extension list, and pretending otherwise would
    /// publish a file that looks complete. The three lossy ones carry
    /// [`FLAT_COLUMNS`] and say so in their own header.
    #[must_use]
    pub fn lossless(self) -> bool {
        matches!(self, Self::Json | Self::Ndjson)
    }

    /// Read a format from the name a caller wrote.
    ///
    /// ⛔ An unknown name is `None` rather than a default: a caller asking for a
    /// format this generator does not have is asking for something that cannot
    /// be produced.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|f| f.as_str() == name)
    }

    /// Every format's name, for a message that has to say what is available.
    #[must_use]
    pub fn names() -> String {
        Self::all()
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl core::fmt::Display for Format {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The columns a lossy format carries, in order.
///
/// ⛔ **This IS the documented subset.** The round-trip test for a lossy format
/// reads these back and compares them, so a column added here without a reader
/// beside it fails to compile rather than widening the promise quietly.
pub const FLAT_COLUMNS: [&str; 8] = [
    "id",
    "browser",
    "version",
    "channel",
    "branded",
    "os",
    "arch",
    "captured_at",
];

/// One profile's flat row, in [`FLAT_COLUMNS`] order.
///
/// ⚠ **Every value is already free of the delimiters**, because each is either
/// derived from a path component, a version, an enum or an ISO instant. The
/// escape below is a guard rather than a formatter: if a value ever does carry
/// one, the file stays parseable and the round trip still holds.
#[must_use]
pub fn flat_row(profile: &Profile) -> [String; 8] {
    [
        profile.id.to_string(),
        profile.browser.name.clone(),
        profile.browser.version.clone(),
        profile.browser.channel.as_str().to_owned(),
        profile.browser.branded.to_string(),
        profile.platform.os.as_str().to_owned(),
        profile.platform.arch.clone(),
        profile.captured.at.clone(),
    ]
}

/// Escape one value for a delimited format.
///
/// ⛔ **Quoted only when it has to be**, so a file two runs apart is
/// byte-identical for the same input rather than depending on which branch ran.
fn delimited(value: &str, delimiter: char) -> String {
    if value.contains(delimiter) || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

/// Render every profile in one format.
///
/// ⛔ **Deterministic for a given input**, which the acceptance asserts by
/// generating twice and comparing bytes. Nothing here reads a clock, a
/// filesystem or an environment variable.
///
/// # Errors
///
/// The serialiser's own message, where a profile cannot be serialised at all.
pub fn render(format: Format, profiles: &[Profile]) -> Result<String, String> {
    match format {
        Format::Json => {
            let text = serde_json::to_string_pretty(profiles)
                .map_err(|err| format!("serialising the canonical form: {err}"))?;
            Ok(format!("{text}\n"))
        }
        Format::Ndjson => {
            let mut out = String::new();
            for profile in profiles {
                let line = serde_json::to_string(profile)
                    .map_err(|err| format!("serialising a profile: {err}"))?;
                out.push_str(&line);
                out.push('\n');
            }
            Ok(out)
        }
        Format::Csv => Ok(delimited_table(profiles, ',')),
        Format::Tsv => Ok(delimited_table(profiles, '\t')),
        Format::Markdown => Ok(markdown_table(profiles)),
    }
}

fn delimited_table(profiles: &[Profile], delimiter: char) -> String {
    let mut out = String::new();
    out.push_str(
        &FLAT_COLUMNS
            .iter()
            .map(|c| (*c).to_owned())
            .collect::<Vec<_>>()
            .join(&delimiter.to_string()),
    );
    out.push('\n');
    for profile in profiles {
        let row = flat_row(profile);
        out.push_str(
            &row.iter()
                .map(|v| delimited(v, delimiter))
                .collect::<Vec<_>>()
                .join(&delimiter.to_string()),
        );
        out.push('\n');
    }
    out
}

/// A Markdown table, with the header that says what it leaves out.
///
/// ⚠ **The header is part of the file rather than a note somewhere else.** A
/// reader arriving at a rendered table on the web has nothing else to read.
fn markdown_table(profiles: &[Profile]) -> String {
    let mut out = String::new();
    out.push_str("# The corpus\n\n");
    out.push_str(
        "Generated. Do not edit: every value here comes from the canonical JSON, and an edit \
         here is lost the next time it is generated.\n\n",
    );
    out.push_str(
        "This table carries the keys and the capture instant only. The handshake, the frames, \
         the headers and the raw bytes are in the JSON.\n\n",
    );
    out.push_str(&format!("| {} |\n", FLAT_COLUMNS.join(" | ")));
    out.push_str(&format!(
        "| {} |\n",
        FLAT_COLUMNS
            .iter()
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    for profile in profiles {
        // ⚠ A pipe in a value would end the cell, so it is escaped. None of the
        // eight columns can carry one today and the guard costs nothing.
        let row = flat_row(profile)
            .iter()
            .map(|v| v.replace('|', "\\|"))
            .collect::<Vec<_>>()
            .join(" | ");
        out.push_str(&format!("| {row} |\n"));
    }
    out
}

/// Read a lossless format back into profiles.
///
/// ⛔ **This is what makes the acceptance a ROUND TRIP rather than a
/// comparison of two writers.** A format with no reader can only be checked
/// against the thing that wrote it, which is the shape `SCHEMA-08` was
/// re-scoped to avoid.
///
/// # Errors
///
/// The parser's own message, or a refusal naming the format where it is one
/// this cannot read back.
pub fn read_back(format: Format, text: &str) -> Result<Vec<Profile>, String> {
    match format {
        Format::Json => {
            serde_json::from_str(text).map_err(|err| format!("reading the canonical form: {err}"))
        }
        Format::Ndjson => text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line).map_err(|err| format!("reading an ndjson line: {err}"))
            })
            .collect(),
        // ⛔ Refused rather than approximated. A lossy format cannot produce a
        // profile, and returning one built from eight columns would be a
        // fabricated profile wearing a measurement's label.
        other => Err(format!(
            "{other} is lossy and carries {} of a profile's fields, so it cannot be read back \
             into one. Its round trip is over those columns and b_ids_corpus::formats::flat_row \
             is what reads them",
            FLAT_COLUMNS.len()
        )),
    }
}

/// The flat rows a lossy format carries, read back out of it.
///
/// ⚠ **The header row is skipped and its names are checked**, so a column added
/// to the writer and not to `FLAT_COLUMNS` is caught here rather than silently
/// widening the file.
///
/// # Errors
///
/// A refusal naming what disagreed with [`FLAT_COLUMNS`].
pub fn read_flat(format: Format, text: &str) -> Result<Vec<Vec<String>>, String> {
    let delimiter = match format {
        Format::Csv => ',',
        Format::Tsv => '\t',
        other => return Err(format!("{other} is not a delimited format")),
    };
    let mut lines = text.lines();
    let header = lines.next().ok_or_else(|| "no header row".to_owned())?;
    let names: Vec<&str> = header.split(delimiter).collect();
    if names != FLAT_COLUMNS {
        return Err(format!(
            "the header is {names:?} and the documented subset is {FLAT_COLUMNS:?}"
        ));
    }
    let mut rows = Vec::new();
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let row: Vec<String> = split_delimited(line, delimiter);
        if row.len() != FLAT_COLUMNS.len() {
            return Err(format!(
                "a row has {} values and the subset has {}",
                row.len(),
                FLAT_COLUMNS.len()
            ));
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Split one delimited line, honouring the quoting [`delimited`] writes.
fn split_delimited(line: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            c if c == delimiter && !quoted => out.push(std::mem::take(&mut field)),
            c => field.push(c),
        }
    }
    out.push(field);
    out
}
