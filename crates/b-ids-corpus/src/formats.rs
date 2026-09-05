//! Every published format, from one generator.
//!
//! ⛔ **JSON is one consumer, not the consumer.** A corpus reachable only by
//! writing a JSON walker is a corpus most people copy values out of by hand,
//! and a value copied by hand is a value that stops matching the day the build
//! moves. `docs/history/todo/schema.md`, `SCHEMA-08` and `SCHEMA-12`.
//!
//! ⭐ **ONE generator, canonical JSON in, every format out.** Nothing here reads
//! the corpus tree: it takes the profiles a caller already loaded, so the
//! layout stays [`crate::route`]'s and the shapes stay this module's.
//!
//! # ⚠ Four fidelities, and every file says which one it is
//!
//! | format | fidelity | for |
//! | --- | --- | --- |
//! | [`Format::Json`] | ⭐ canonical. Every other format is generated from it. | the reference form |
//! | [`Format::Ndjson`] | lossless | streaming, line-oriented tools |
//! | [`Format::Yaml`] | lossless | configuration-shaped tooling |
//! | [`Format::Sqlite`] | lossless, as a text dump | a query, with no library here |
//! | [`Format::Toml`] | ⚠ every field that is not null | configuration-shaped tooling that has no null |
//! | [`Format::Csv`] | ⚠ [`FLAT_COLUMNS`] | spreadsheets |
//! | [`Format::Tsv`] | ⚠ [`FLAT_COLUMNS`] | shell pipelines |
//! | [`Format::Markdown`] | ⚠ [`FLAT_COLUMNS`] | browsing on the web with no tooling |
//! | [`Format::Protobuf`] | a definition, not values | a typed consumer generating its own decoder |
//!
//! ⛔ **A format that is not lossless asserts its DOCUMENTED SUBSET rather than
//! equality**, and each subset is a value in this module rather than a sentence
//! in a comment: [`FLAT_COLUMNS`] for the three flat ones, [`strip_nulls`] for
//! TOML, and [`proto::Definition`] for the definition.
//!
//! # ⛔ Two formats are declined, and the reason is published
//!
//! [`Declined`] carries CBOR and MessagePack with what each would have cost.
//! They are absent from [`Format`], so nothing can generate them by accident,
//! and [`support_matrix`] names them so a consumer asking for one finds the
//! answer rather than a silence.
//!
//! ⛔ **Never hand-edit a generated format.** If one is ever edited directly the
//! generator has lost, and the round-trip test is what says so.

pub mod proto;
pub mod sql;
pub mod toml;
pub mod yaml;

use b_ids_schema::Profile;
use serde_json::Value;

/// A format the corpus is published in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Format {
    /// Pretty-printed JSON, one array of profiles. The canonical form.
    Json,
    /// One compact JSON object per line.
    Ndjson,
    /// Block YAML, over the subset [`yaml`] writes and reads.
    Yaml,
    /// TOML, every field except the ones whose value is null.
    Toml,
    /// A SQLite dump, as text, carrying the canonical JSON per row.
    Sqlite,
    /// Comma-separated, one row per profile, the flat columns only.
    Csv,
    /// Tab-separated, one row per profile, the flat columns only.
    Tsv,
    /// A Markdown table, the flat columns only.
    Markdown,
    /// A protobuf definition of the model, with no encoded bytes.
    Protobuf,
}

/// How much of a profile one format carries.
///
/// ⭐ **A value rather than a boolean, because there are four answers.** The
/// question a consumer asks is "what will I not get", and `lossless: false` is
/// the same answer for a spreadsheet with eight columns and for a TOML file
/// carrying every field but the null ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fidelity {
    /// Every field. It reads back into a profile.
    Lossless,
    /// Every field whose value is not null. TOML has no null.
    NoNulls,
    /// The eight columns of [`FLAT_COLUMNS`].
    FlatColumns,
    /// The shape of the model rather than any profile's values.
    Definition,
}

impl Fidelity {
    /// The word a report and the support matrix print.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lossless => "lossless",
            Self::NoNulls => "every field that is not null",
            Self::FlatColumns => "the eight flat columns",
            Self::Definition => "a definition, not values",
        }
    }
}

impl Format {
    /// Every format, in the order they are generated.
    #[must_use]
    pub fn all() -> [Self; 9] {
        [
            Self::Json,
            Self::Ndjson,
            Self::Yaml,
            Self::Toml,
            Self::Sqlite,
            Self::Csv,
            Self::Tsv,
            Self::Markdown,
            Self::Protobuf,
        ]
    }

    /// The name a caller writes.
    ///
    /// ⚠ **Not always the file extension.** A caller asks for `sqlite` and gets
    /// `corpus.sql`, because the artefact is a dump rather than a database and
    /// naming it `.sqlite` would invite somebody to open it as one.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Ndjson => "ndjson",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Sqlite => "sqlite",
            Self::Csv => "csv",
            Self::Tsv => "tsv",
            Self::Markdown => "md",
            Self::Protobuf => "protobuf",
        }
    }

    /// The extension the generated file carries.
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Self::Sqlite => "sql",
            Self::Protobuf => "proto",
            other => other.as_str(),
        }
    }

    /// The file this format is written to, under a caller's output directory.
    #[must_use]
    pub fn file_name(self) -> String {
        format!("corpus.{}", self.extension())
    }

    /// How much of a profile this format carries.
    #[must_use]
    pub fn fidelity(self) -> Fidelity {
        match self {
            Self::Json | Self::Ndjson | Self::Yaml | Self::Sqlite => Fidelity::Lossless,
            Self::Toml => Fidelity::NoNulls,
            Self::Csv | Self::Tsv | Self::Markdown => Fidelity::FlatColumns,
            Self::Protobuf => Fidelity::Definition,
        }
    }

    /// Whether this format carries every field a profile has.
    ///
    /// ⛔ **A lossy format is not a defect and is not hidden.** A spreadsheet
    /// cannot hold a nested extension list and TOML cannot hold a null, and
    /// pretending otherwise would publish a file that looks complete. Each says
    /// what it leaves out in its own header, and [`Fidelity`] is the value.
    #[must_use]
    pub fn lossless(self) -> bool {
        self.fidelity() == Fidelity::Lossless
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

/// A format this project was asked for and does not publish.
///
/// ⛔ **A declined format is recorded rather than forgotten.** A consumer who
/// wants one otherwise cannot tell "nobody thought of it" from "it was weighed
/// and lost", and the second is the useful answer. `docs/history/todo/schema.md`,
/// `SCHEMA-12`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Declined {
    /// Concise Binary Object Representation.
    Cbor,
    /// MessagePack.
    MessagePack,
}

impl Declined {
    /// Every declined format.
    #[must_use]
    pub fn all() -> [Self; 2] {
        [Self::Cbor, Self::MessagePack]
    }

    /// The name a consumer would have asked for.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cbor => "cbor",
            Self::MessagePack => "msgpack",
        }
    }

    /// Why it is not published.
    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            Self::Cbor => {
                "a binary codec is a dependency here and a decoder this project owns forever, \
                 and it serves the same consumer MessagePack would. Nothing consumes either \
                 today. The lossless artefact for a program is the SQLite dump, which needs no \
                 library at all."
            }
            Self::MessagePack => {
                "the same trade as CBOR, and publishing both would be two binary encodings of \
                 one model with nothing choosing between them."
            }
        }
    }
}

impl core::fmt::Display for Declined {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The file [`support_matrix`] is written to.
pub const SUPPORT_MATRIX_FILE: &str = "formats.md";

/// The columns a flat format carries, in order.
///
/// ⛔ **This IS the documented subset.** The round-trip test for a flat format
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

/// The same tree with every null removed, which is TOML's documented subset.
///
/// ⛔ **An element of an ARRAY is never removed**, only a key of an object. A
/// wire order with an element missing is a different fingerprint, and
/// [`toml::render`] refuses an array carrying a null rather than shortening it.
#[must_use]
pub fn strip_nulls(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(_, child)| !child.is_null())
                .map(|(key, child)| (key.clone(), strip_nulls(child)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(strip_nulls).collect()),
        other => other.clone(),
    }
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
/// The serialiser's own message, where a profile cannot be serialised at all,
/// or a format's own refusal where the model holds a shape it has no spelling
/// for.
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
        Format::Yaml => Ok(yaml::render(&tree(profiles)?)),
        Format::Toml => toml::render(&tree(profiles)?),
        Format::Sqlite => sql::render(profiles),
        Format::Csv => Ok(delimited_table(profiles, ',')),
        Format::Tsv => Ok(delimited_table(profiles, '\t')),
        Format::Markdown => Ok(markdown_table(profiles)),
        Format::Protobuf => proto::render(profiles),
    }
}

/// The canonical JSON tree of every profile.
///
/// ⭐ **One conversion, so the tree formats cannot disagree about what a profile
/// is.** YAML and TOML both render from this rather than each walking the
/// model.
fn tree(profiles: &[Profile]) -> Result<Value, String> {
    serde_json::to_value(profiles).map_err(|err| format!("serialising the corpus: {err}"))
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

/// Read `text` back and assert it carries what `format` promises.
///
/// ⛔ **A generator that can only pass verifies its own effect.** The
/// round-trip suite renders its own fixture, so without this a format could be
/// correct over two invented profiles and wrong over the published corpus, and
/// the file would still be written. `docs/conventions/forbidden-patterns.md`
/// carries the class: a step that exits 0 having done nothing it was asked to
/// do.
///
/// # Errors
///
/// A refusal naming the format and what did not come back.
pub fn verify(format: Format, profiles: &[Profile], text: &str) -> Result<(), String> {
    match format.fidelity() {
        Fidelity::Lossless => {
            let back = read_back(format, text)?;
            let canonical = render(Format::Json, profiles)?;
            if render(Format::Json, &back)? != canonical {
                return Err(format!(
                    "{format} did not read back to byte-identical canonical JSON"
                ));
            }
        }
        Fidelity::NoNulls => {
            let back = read_tree(format, text)?;
            if back != strip_nulls(&tree(profiles)?) {
                return Err(format!(
                    "{format} did not read back to every field that is not null"
                ));
            }
        }
        Fidelity::FlatColumns => {
            // ⚠ Markdown is a table for a person and has no reader. What is
            // asserted instead is that it says so, which is the promise it
            // makes.
            if format == Format::Markdown {
                if !text.contains("are in the JSON") {
                    return Err("the markdown table does not say what it leaves out".to_owned());
                }
                return Ok(());
            }
            let rows = read_flat(format, text)?;
            if rows.len() != profiles.len() {
                return Err(format!(
                    "{format} read back {} row(s) for {} profile(s)",
                    rows.len(),
                    profiles.len()
                ));
            }
            for (row, profile) in rows.iter().zip(profiles) {
                if row.as_slice() != flat_row(profile).as_slice() {
                    return Err(format!("{format}: a row is not what the profile carries"));
                }
            }
        }
        Fidelity::Definition => {
            if proto::parse(text)? != proto::declared(profiles)? {
                return Err(format!(
                    "{format} did not read back to the definition the corpus implies"
                ));
            }
        }
    }
    Ok(())
}

/// The support matrix, generated beside the formats it describes.
///
/// ⛔ **Generated rather than written, so it cannot drift from the generator.**
/// A support matrix maintained by hand states what somebody believed on the day
/// they wrote it, and a consumer reading it has no way to tell.
#[must_use]
pub fn support_matrix() -> String {
    let mut out = String::new();
    out.push_str("# Published formats\n\n");
    out.push_str(
        "Generated. Do not edit: this table is derived from the generator, so a format that \
         is added, changed or declined moves this file in the same change.\n\n",
    );
    out.push_str("## Published\n\n");
    out.push_str("| ask for | file | carries |\n| --- | --- | --- |\n");
    for format in Format::all() {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            format.as_str(),
            format.file_name(),
            format.fidelity().as_str()
        ));
    }
    out.push_str("\n## Declined\n\n");
    out.push_str(
        "Weighed and not published. A consumer who wants one of these can read why rather \
         than guess.\n\n",
    );
    out.push_str("| asked for | why not |\n| --- | --- |\n");
    for declined in Declined::all() {
        out.push_str(&format!(
            "| `{}` | {} |\n",
            declined.as_str(),
            declined.reason()
        ));
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
/// this cannot read back into a profile.
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
        Format::Yaml => {
            let value = yaml::parse(text)?;
            serde_json::from_value(value).map_err(|err| format!("reading the yaml tree: {err}"))
        }
        Format::Sqlite => sql::parse(text),
        // ⛔ Refused rather than approximated. A format that does not carry
        // every field cannot produce a profile, and returning one built from
        // what it does carry would be a fabricated profile wearing a
        // measurement's label.
        other => Err(format!(
            "{other} carries {} rather than every field, so it cannot be read back into a \
             profile. Its round trip is over that subset",
            other.fidelity().as_str()
        )),
    }
}

/// The tree a subset format carries, read back out of it.
///
/// ⚠ **Compare it with [`strip_nulls`] of the canonical tree**, which is the
/// subset TOML promises, rather than with the canonical tree itself.
///
/// # Errors
///
/// The reader's own message, or a refusal for a format that is not a tree.
pub fn read_tree(format: Format, text: &str) -> Result<Value, String> {
    match format {
        Format::Toml => toml::parse(text),
        Format::Yaml => yaml::parse(text),
        other => Err(format!("{other} is not a tree format")),
    }
}

/// The flat rows a delimited format carries, read back out of it.
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
    let mut records = split_delimited(text, delimiter)?.into_iter();
    let header = records.next().ok_or_else(|| "no header row".to_owned())?;
    if header != FLAT_COLUMNS {
        return Err(format!(
            "the header is {header:?} and the documented subset is {FLAT_COLUMNS:?}"
        ));
    }
    let mut rows = Vec::new();
    for row in records {
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

/// Split a delimited file into records, honouring the quoting [`delimited`]
/// writes.
///
/// ⛔ **NOT LINE BY LINE, and that was a defect rather than a style.** The
/// writer quotes a value carrying a newline, which is what a delimited format
/// says to do, and a reader that split on lines first cut such a row in two and
/// reported a row of one field. Found on 2026-09-02 by rendering a profile
/// whose values carry every character these formats escape.
/// `docs/history/todo/schema.md`, `SCHEMA-12`.
///
/// # Errors
///
/// A refusal where a quoted field is never closed, which is a truncated file.
fn split_delimited(text: &str, delimiter: char) -> Result<Vec<Vec<String>>, String> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            c if c == delimiter && !quoted => row.push(std::mem::take(&mut field)),
            '\r' if !quoted && chars.peek() == Some(&'\n') => {}
            '\n' if !quoted => {
                row.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut row));
            }
            c => field.push(c),
        }
    }
    if quoted {
        return Err("a quoted field is never closed".to_owned());
    }
    // ⚠ A file with no final newline still ends a record. The generator always
    // writes one, so this is the guard rather than the path.
    if !field.is_empty() || !row.is_empty() {
        row.push(field);
        records.push(row);
    }
    Ok(records)
}
