//! YAML, written and read here, over the one shape this generator produces.
//!
//! ⛔ **This is not a YAML parser and it does not pretend to be one.** It reads
//! back exactly what [`render`] writes and refuses everything else by name. A
//! general YAML reader is the thing `SCHEMA-12` says must not be written: YAML
//! 1.2 has anchors, aliases, tags, flow collections, five scalar styles and a
//! resolution table, and a partial implementation of that is a reader that is
//! wrong quietly.
//!
//! ⭐ **What is written is ordinary YAML, and every consumer reads it.** The
//! narrow half is the reader, which exists so the acceptance is a ROUND TRIP
//! rather than a comparison of this writer with itself.
//!
//! ⚠ **Why no dependency.** `serde_yaml` was withdrawn by its author in 2024
//! and what remains are forks. Taking one would put an unmaintained parser
//! between this project's product and its consumers, and the cost `SCHEMA-12`
//! asks to be recorded is exactly that one.
//!
//! # The subset
//!
//! | written | as |
//! | --- | --- |
//! | a mapping key | a double-quoted scalar, always, so no key can collide with a YAML keyword |
//! | a string | a double-quoted scalar, escaped as JSON escapes it, which is a subset of YAML's double-quoted style |
//! | a number, a boolean, `null` | its JSON spelling |
//! | an empty mapping or sequence | `{}` or `[]`, on the key's own line |
//! | a non-empty mapping or sequence | a block, indented two spaces from its key |
//!
//! `TODO/schema.md`, `SCHEMA-12`.

use serde_json::{Map, Value};

/// How far one level of nesting is indented.
///
/// ⚠ **The reader derives structure from this**, so the writer and the reader
/// read the same constant rather than two copies of the number two.
const INDENT: usize = 2;

/// Render one value as the canonical block YAML this module reads back.
#[must_use]
pub fn render(value: &Value) -> String {
    let mut out = String::new();
    match inline(value) {
        // ⚠ A whole document that is one scalar still needs a line of its own.
        Some(text) => {
            out.push_str(&text);
            out.push('\n');
        }
        None => block(value, 0, &mut out),
    }
    out
}

/// The one-line spelling of a value, where it has one.
///
/// ⛔ **`None` means the value needs a block**, which is the only thing the
/// caller has to know. An empty container has a one-line spelling and a
/// non-empty one does not.
fn inline(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_owned()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => Some(quote(s)),
        Value::Array(items) if items.is_empty() => Some("[]".to_owned()),
        Value::Object(map) if map.is_empty() => Some("{}".to_owned()),
        _ => None,
    }
}

/// A string as a double-quoted YAML scalar.
///
/// ⭐ **Escaped by the JSON serialiser rather than by hand.** Every escape JSON
/// emits is one YAML's double-quoted style accepts, so the one correct escaper
/// already in this workspace is the one used.
fn quote(text: &str) -> String {
    Value::String(text.to_owned()).to_string()
}

/// Write a non-empty container as an indented block.
fn block(value: &Value, indent: usize, out: &mut String) {
    let pad = " ".repeat(indent);
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                out.push_str(&pad);
                out.push_str(&quote(key));
                out.push(':');
                push_child(child, indent, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                out.push_str(&pad);
                out.push('-');
                push_child(item, indent, out);
            }
        }
        // ⛔ Unreachable by construction: `render` and `push_child` both call
        // `inline` first and only reach here when it answered `None`.
        other => {
            out.push_str(&pad);
            out.push_str(&inline(other).unwrap_or_default());
            out.push('\n');
        }
    }
}

/// Write whatever follows a `key:` or a `-`, inline or as a block below it.
fn push_child(child: &Value, indent: usize, out: &mut String) {
    match inline(child) {
        Some(text) => {
            out.push(' ');
            out.push_str(&text);
            out.push('\n');
        }
        None => {
            out.push('\n');
            block(child, indent + INDENT, out);
        }
    }
}

/// One significant line, with the depth its indentation puts it at.
#[derive(Debug, Clone, Copy)]
struct Line<'a> {
    indent: usize,
    text: &'a str,
}

/// Read back the block YAML [`render`] writes.
///
/// # Errors
///
/// A refusal naming the line and what about it this reader does not accept.
/// ⛔ Never a best guess: a document this writer did not produce is refused
/// rather than approximated.
pub fn parse(text: &str) -> Result<Value, String> {
    let lines: Vec<Line<'_>> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| Line {
            indent: line.len() - line.trim_start_matches(' ').len(),
            text: line.trim_start_matches(' '),
        })
        .collect();
    if lines.is_empty() {
        return Err("the document is empty".to_owned());
    }
    // ⚠ A whole document that is one scalar, which is what an EMPTY CORPUS
    // renders as. Tried before the block reader because `[]` is not a mapping
    // and the block reader would refuse it as a key that is not quoted.
    if lines.len() == 1
        && let Ok(value) = scalar(lines[0].text, 1)
    {
        return Ok(value);
    }
    let mut cursor = 0_usize;
    let value = read_block(&lines, &mut cursor, lines[0].indent)?;
    if cursor != lines.len() {
        return Err(format!(
            "line {} is indented less than the document it is inside",
            cursor + 1
        ));
    }
    Ok(value)
}

/// Read every line at `indent` as one mapping or one sequence.
fn read_block(lines: &[Line<'_>], cursor: &mut usize, indent: usize) -> Result<Value, String> {
    let first = lines
        .get(*cursor)
        .ok_or_else(|| "a block with no lines in it".to_owned())?;
    if first.indent != indent {
        return Err(format!(
            "line {} is indented {} where {indent} was expected",
            *cursor + 1,
            first.indent
        ));
    }
    if first.text == "-" || first.text.starts_with("- ") {
        return read_sequence(lines, cursor, indent);
    }
    read_mapping(lines, cursor, indent)
}

fn read_sequence(lines: &[Line<'_>], cursor: &mut usize, indent: usize) -> Result<Value, String> {
    let mut items = Vec::new();
    while let Some(line) = lines.get(*cursor) {
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(format!(
                "line {} is indented past its sequence",
                *cursor + 1
            ));
        }
        let rest = if line.text == "-" {
            ""
        } else {
            line.text
                .strip_prefix("- ")
                .ok_or_else(|| format!("line {} is not a sequence item", *cursor + 1))?
        };
        *cursor += 1;
        items.push(read_child(lines, cursor, indent, rest)?);
    }
    Ok(Value::Array(items))
}

fn read_mapping(lines: &[Line<'_>], cursor: &mut usize, indent: usize) -> Result<Value, String> {
    let mut map = Map::new();
    while let Some(line) = lines.get(*cursor) {
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(format!("line {} is indented past its mapping", *cursor + 1));
        }
        if line.text.starts_with('-') {
            break;
        }
        let (key, rest) = split_key(line.text, *cursor + 1)?;
        *cursor += 1;
        let child = read_child(lines, cursor, indent, rest)?;
        if map.insert(key.clone(), child).is_some() {
            // ⛔ A duplicate key is refused rather than resolved. YAML's own
            // answer is "last wins", and silently dropping half a profile is
            // the class of quiet wrongness this whole project is about.
            return Err(format!("the key {key} appears twice in one mapping"));
        }
    }
    Ok(Value::Object(map))
}

/// The value after a `key:` or a `-`, whether it was on the line or below it.
fn read_child(
    lines: &[Line<'_>],
    cursor: &mut usize,
    indent: usize,
    rest: &str,
) -> Result<Value, String> {
    if !rest.is_empty() {
        return scalar(rest, *cursor);
    }
    match lines.get(*cursor) {
        Some(next) if next.indent > indent => read_block(lines, cursor, next.indent),
        // ⛔ A key with nothing after it and nothing under it is refused. This
        // writer never produces one, and YAML would read it as null, which
        // would turn a truncated file into a profile full of absent fields.
        _ => Err(format!(
            "line {} has neither a value nor a block under it",
            *cursor
        )),
    }
}

/// Split `"key": rest` into the key and whatever follows the colon.
fn split_key(text: &str, line_number: usize) -> Result<(String, &str), String> {
    let mut chars = text.char_indices();
    if chars.next().map(|(_, c)| c) != Some('"') {
        return Err(format!(
            "line {line_number} does not start with a quoted key"
        ));
    }
    let mut escaped = false;
    for (index, c) in chars {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '"' => {
                let key = serde_json::from_str::<String>(&text[..=index])
                    .map_err(|err| format!("line {line_number}: the key does not parse: {err}"))?;
                let rest = text[index + 1..]
                    .strip_prefix(':')
                    .ok_or_else(|| format!("line {line_number}: no colon after the key"))?;
                return Ok((key, rest.strip_prefix(' ').unwrap_or(rest)));
            }
            _ => {}
        }
    }
    Err(format!("line {line_number}: the key is not closed"))
}

/// Read one inline value.
fn scalar(text: &str, line_number: usize) -> Result<Value, String> {
    match text {
        "null" => Ok(Value::Null),
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        "[]" => Ok(Value::Array(Vec::new())),
        "{}" => Ok(Value::Object(Map::new())),
        _ => serde_json::from_str(text).map_err(|err| {
            format!("line {line_number}: {text} is not a scalar this reader accepts: {err}")
        }),
    }
}
