//! SQLite, as a text dump rather than a binary database.
//!
//! ⭐ **This is `SCHEMA-12`'s recommendation and the reason it wins.** A binary
//! `.sqlite` needs a library to write and the same library to read, and this
//! project would then own a storage engine. A dump is text: it loads with the
//! `sqlite3` any host already has, it diffs, it round-trips, and this tree
//! ships no database code at all.
//!
//! ```text
//! sqlite3 corpus.db < corpus.sql
//! ```
//!
//! ⭐ **It is LOSSLESS, and the canonical JSON column is why.** The eight flat
//! columns are there so a consumer can index, join and filter without a JSON
//! function; `canonical_json` carries the whole profile beside them, so nothing
//! is lost and the flat columns are a convenience rather than a subset.
//!
//! ⚠ **The two are generated from the same profile in one pass**, so a flat
//! column cannot disagree with the JSON beside it.
//!
//! `TODO/schema.md`, `SCHEMA-12`.

use b_ids_schema::Profile;

use super::{FLAT_COLUMNS, flat_row};

/// The table the dump creates.
pub const TABLE: &str = "profile";

/// The column carrying the whole profile.
///
/// ⛔ **Named once and read by the reader**, so a rename cannot leave the two
/// halves looking at different columns.
pub const CANONICAL_COLUMN: &str = "canonical_json";

/// Render the profiles as a SQLite dump.
///
/// # Errors
///
/// The serialiser's own message, where a profile cannot be serialised.
pub fn render(profiles: &[Profile]) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("-- The corpus, as a SQLite dump. Generated: do not edit.\n");
    out.push_str("-- Load it with: sqlite3 corpus.db < corpus.sql\n");
    out.push_str(&format!(
        "-- Lossless: {CANONICAL_COLUMN} carries the whole profile, and the columns beside\n\
         -- it are the same values lifted out so a query does not need a JSON function.\n"
    ));
    out.push_str("BEGIN TRANSACTION;\n");
    out.push_str(&format!("CREATE TABLE {TABLE} (\n"));
    for (index, column) in FLAT_COLUMNS.iter().enumerate() {
        let kind = if *column == "branded" {
            // ⚠ SQLite has no boolean type. INTEGER is what it stores one as,
            // and the reader turns 0 and 1 back rather than reading `true`.
            "INTEGER NOT NULL"
        } else if index == 0 {
            "TEXT PRIMARY KEY"
        } else {
            "TEXT NOT NULL"
        };
        out.push_str(&format!("  {column} {kind},\n"));
    }
    out.push_str(&format!("  {CANONICAL_COLUMN} TEXT NOT NULL\n"));
    out.push_str(");\n");
    for profile in profiles {
        let row = flat_row(profile);
        let canonical = serde_json::to_string(profile)
            .map_err(|err| format!("serialising a profile for the dump: {err}"))?;
        let mut values: Vec<String> = Vec::with_capacity(row.len() + 1);
        for (column, value) in FLAT_COLUMNS.iter().zip(row.iter()) {
            values.push(if *column == "branded" {
                boolean(value)?
            } else {
                literal(value)
            });
        }
        values.push(literal(&canonical));
        out.push_str(&format!(
            "INSERT INTO {TABLE} VALUES({});\n",
            values.join(",")
        ));
    }
    out.push_str("COMMIT;\n");
    Ok(out)
}

/// One value as a SQL string literal.
///
/// ⛔ **A quote is doubled, which is SQL's own escape and the only one SQLite
/// accepts in a string literal.** A backslash is NOT an escape there, so a
/// generator that reached for one would write a value that reads back with an
/// extra character.
fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// A boolean column, as the integer SQLite stores.
fn boolean(value: &str) -> Result<String, String> {
    match value {
        "true" => Ok("1".to_owned()),
        "false" => Ok("0".to_owned()),
        other => Err(format!("{other} is not a boolean")),
    }
}

/// Read the canonical JSON back out of a dump.
///
/// ⛔ **It reads the LAST column of every insert**, which is the whole profile,
/// and never reconstructs one from the flat columns beside it. Those exist for
/// a query; a profile assembled from eight of them would be a profile this
/// project fabricated.
///
/// # Errors
///
/// A refusal naming the line, or the parser's own message where the recovered
/// JSON does not parse.
pub fn parse(text: &str) -> Result<Vec<Profile>, String> {
    let prefix = format!("INSERT INTO {TABLE} VALUES(");
    let mut profiles = Vec::new();
    let mut rest = text;
    let mut ordinal = 0_usize;
    while let Some(at) = rest.find(&prefix) {
        ordinal += 1;
        // ⛔ NOT LINE BY LINE, and that was a defect rather than a style. SQL
        // allows a newline inside a string literal, and a flat column carrying
        // one is written that way. A reader that split on lines reported the
        // insert as unterminated. Found on 2026-09-02 by rendering a profile
        // whose values carry every character these formats escape.
        let (values, tail) = take_values(&rest[at + prefix.len()..], ordinal)?;
        // ⛔ The count is checked before the last value is taken. A dump with a
        // column added and no reader change would otherwise read the wrong
        // column and parse it as a profile.
        if values.len() != FLAT_COLUMNS.len() + 1 {
            return Err(format!(
                "insert {ordinal}: {} value(s) where {} were expected",
                values.len(),
                FLAT_COLUMNS.len() + 1
            ));
        }
        let canonical = values
            .last()
            .ok_or_else(|| format!("insert {ordinal}: no {CANONICAL_COLUMN} value"))?;
        profiles.push(
            serde_json::from_str(canonical)
                .map_err(|err| format!("insert {ordinal}: {CANONICAL_COLUMN}: {err}"))?,
        );
        rest = tail;
    }
    Ok(profiles)
}

/// Read one insert's values up to its closing `);`, and what follows it.
///
/// ⚠ **The close is found at quote depth zero.** A `)` inside a string literal,
/// which any JSON object with one in a value carries, is not the end of the
/// insert.
fn take_values(after: &str, ordinal: usize) -> Result<(Vec<String>, &str), String> {
    let mut values = Vec::new();
    let mut chars = after.char_indices().peekable();
    loop {
        // ⚠ Skip whatever separates values, so a dump written with a space
        // after the comma reads the same as one written without.
        while let Some((_, c)) = chars.peek() {
            if *c == ',' || c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
        match chars.peek() {
            None => return Err(format!("insert {ordinal}: it is never terminated")),
            Some((index, ')')) => {
                let end = index + 1;
                let tail = after[end..]
                    .strip_prefix(';')
                    .ok_or_else(|| format!("insert {ordinal}: no semicolon after its values"))?;
                return Ok((values, tail));
            }
            Some(_) => {}
        }
        let Some((_, first)) = chars.next() else {
            return Err(format!("insert {ordinal}: it is never terminated"));
        };
        if first != '\'' {
            // An unquoted value, which is how the integer column is written.
            let mut text = String::from(first);
            while let Some((_, c)) = chars.peek() {
                if *c == ',' || *c == ')' {
                    break;
                }
                text.push(*c);
                chars.next();
            }
            values.push(text.trim().to_owned());
            continue;
        }
        let mut text = String::new();
        loop {
            match chars.next() {
                Some((_, '\'')) => {
                    if chars.peek().map(|(_, c)| *c) == Some('\'') {
                        text.push('\'');
                        chars.next();
                    } else {
                        break;
                    }
                }
                Some((_, c)) => text.push(c),
                None => {
                    return Err(format!("insert {ordinal}: an unterminated string literal"));
                }
            }
        }
        values.push(text);
    }
}
