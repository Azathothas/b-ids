//! TOML, written and read here, over the one shape this generator produces.
//!
//! ⛔ **TOML has no null, and that is what makes this format lossy.** The model
//! carries absent values as `null`: `digests.ja3` is null in every published
//! profile because nothing here computes one yet, and `captured.acquisition` is
//! null on every build nobody chose. TOML can say a key is not set only by not
//! writing it, so a reader cannot tell "this field is null" from "this field
//! does not exist in this version of the schema".
//!
//! ⭐ **So the documented subset is every field whose value is not null**, and
//! the file says that in its own header rather than in a document beside it.
//! The round trip asserts exactly that subset, which is
//! [`super::strip_nulls`].
//!
//! ⚠ **Why no dependency.** A TOML crate would bring a full parser and a
//! serialiser for a format this project writes in one shape and reads in one
//! shape. What it would NOT do is decide the null question above, which is the
//! only hard part. `SCHEMA-12` asks for the cost to be recorded: the cost of
//! the dependency is a parser this project does not need, and the cost of this
//! module is the reader below, which is narrow on purpose.
//!
//! # The subset
//!
//! | written | as |
//! | --- | --- |
//! | the corpus | one `[[profile]]` array-of-tables entry per profile |
//! | a nested object | its own `[path]` table, after its parent's scalar keys |
//! | an array of objects | one `[[path]]` entry each |
//! | an array of scalars | an inline array on one line |
//! | a null | ⛔ nothing at all. See above. |
//!
//! `TODO/schema.md`, `SCHEMA-12`.

use serde_json::{Map, Value};

/// The top-level array-of-tables key every profile is written under.
pub const ROOT_KEY: &str = "profile";

/// Render the profiles as the canonical TOML this module reads back.
///
/// # Errors
///
/// A refusal naming the path, where the value at it is a shape TOML has no
/// spelling for. ⛔ Never an approximation: a nested array of arrays of tables
/// is refused rather than flattened.
pub fn render(profiles: &Value) -> Result<String, String> {
    let items = profiles
        .as_array()
        .ok_or_else(|| "the corpus is an array of profiles".to_owned())?;
    let mut out = String::new();
    out.push_str("# The corpus, as TOML. Generated: do not edit.\n");
    out.push_str(
        "# ⛔ TOML has no null, so a field whose value is null is NOT WRITTEN HERE. Every\n\
         # other field is present. The canonical JSON is what carries the difference\n\
         # between a field that is null and a field that does not exist.\n",
    );
    if items.is_empty() {
        // ⚠ An EMPTY corpus, which is what a publisher has on day one. TOML has
        // no way to write an array of tables with no entries in it, so the key
        // is written as an empty inline array. ⛔ Writing nothing instead would
        // make an empty corpus and a truncated file the same document.
        out.push_str(&format!("\n{ROOT_KEY} = []\n"));
        return Ok(out);
    }
    for item in items {
        let map = item
            .as_object()
            .ok_or_else(|| "a profile is an object".to_owned())?;
        out.push_str(&format!("\n[[{ROOT_KEY}]]\n"));
        table(map, &[ROOT_KEY.to_owned()], &mut out)?;
    }
    Ok(out)
}

/// Write one table: its scalar keys, then its sub-tables, then its arrays of
/// tables.
///
/// ⛔ **The order is TOML's rule rather than a preference.** A `key = value`
/// written after a `[sub.table]` header belongs to that sub-table, so a
/// generator that emitted in map order would silently move fields.
fn table(map: &Map<String, Value>, path: &[String], out: &mut String) -> Result<(), String> {
    for (key, value) in map {
        match value {
            Value::Null => {}
            Value::Object(child) if !child.is_empty() => {}
            Value::Array(items) if items.iter().any(Value::is_object) => {}
            _ => {
                out.push_str(&key_text(key));
                out.push_str(" = ");
                out.push_str(&inline(value, &joined(path, key))?);
                out.push('\n');
            }
        }
    }
    for (key, value) in map {
        let Value::Object(child) = value else {
            continue;
        };
        if child.is_empty() {
            continue;
        }
        let mut child_path = path.to_vec();
        child_path.push(key.clone());
        out.push_str(&format!("\n[{}]\n", header(&child_path)));
        table(child, &child_path, out)?;
    }
    for (key, value) in map {
        let Value::Array(items) = value else {
            continue;
        };
        if !items.iter().any(Value::is_object) {
            continue;
        }
        let mut child_path = path.to_vec();
        child_path.push(key.clone());
        for item in items {
            let child = item.as_object().ok_or_else(|| {
                format!(
                    "{}: an array mixes objects with other values, which TOML has no spelling for",
                    joined(path, key)
                )
            })?;
            out.push_str(&format!("\n[[{}]]\n", header(&child_path)));
            table(child, &child_path, out)?;
        }
    }
    Ok(())
}

/// The one-line spelling of a scalar or an array of scalars.
fn inline(value: &Value, path: &str) -> Result<String, String> {
    match value {
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(Value::String(s.clone()).to_string()),
        Value::Object(map) if map.is_empty() => Ok("{}".to_owned()),
        Value::Array(items) => {
            let mut parts = Vec::with_capacity(items.len());
            for item in items {
                if item.is_null() {
                    // ⛔ Refused rather than dropped. Dropping one element of
                    // an array changes its LENGTH, and a wire order with an
                    // element missing is a different fingerprint.
                    return Err(format!(
                        "{path}: an array carries a null, and TOML has no spelling for one"
                    ));
                }
                parts.push(inline(item, path)?);
            }
            Ok(format!("[{}]", parts.join(", ")))
        }
        // ⛔ Unreachable from `table`, which routes objects and arrays of
        // objects to their own headers before calling this.
        other => Err(format!("{path}: {other} has no inline TOML spelling")),
    }
}

/// One key, bare where TOML allows it and quoted where it does not.
///
/// ⚠ **`provenance` is why the quoted branch exists.** Its keys are field
/// paths such as the one for the switch list, and a dot in a bare TOML key is a
/// nesting operator rather than a character.
fn key_text(key: &str) -> String {
    if !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        key.to_owned()
    } else {
        Value::String(key.to_owned()).to_string()
    }
}

fn header(path: &[String]) -> String {
    path.iter()
        .map(|part| key_text(part))
        .collect::<Vec<_>>()
        .join(".")
}

fn joined(path: &[String], key: &str) -> String {
    let mut parts = path.to_vec();
    parts.push(key.to_owned());
    parts.join(".")
}

/// Read back the TOML [`render`] writes, as the profiles array with no nulls.
///
/// # Errors
///
/// A refusal naming the line and what about it this reader does not accept.
pub fn parse(text: &str) -> Result<Value, String> {
    let mut root = Map::new();
    // ⚠ The path of the table every following assignment belongs to, which is
    // what a header line changes and nothing else does.
    let mut current: Vec<String> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let number = index + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("[[") {
            let name = rest
                .strip_suffix("]]")
                .ok_or_else(|| format!("line {number}: an array-of-tables header is not closed"))?;
            current = split_path(name, number)?;
            append_table(&mut root, &current, number)?;
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            let name = rest
                .strip_suffix(']')
                .ok_or_else(|| format!("line {number}: a table header is not closed"))?;
            current = split_path(name, number)?;
            open_table(&mut root, &current, number)?;
            continue;
        }
        let (key, value) = split_assignment(line, number)?;
        let table = table_at(&mut root, &current, number)?;
        if table.insert(key.clone(), value).is_some() {
            return Err(format!("line {number}: the key {key} is assigned twice"));
        }
    }
    root.remove(ROOT_KEY)
        .ok_or_else(|| format!("the document carries no [[{ROOT_KEY}]] entry"))
}

/// The table one path names, following array-of-tables entries to their last
/// element, which is the one a header opened.
fn table_at<'a>(
    root: &'a mut Map<String, Value>,
    path: &[String],
    number: usize,
) -> Result<&'a mut Map<String, Value>, String> {
    let mut cursor = root;
    for part in path {
        let entry = cursor
            .get_mut(part)
            .ok_or_else(|| format!("line {number}: no table named {part} has been opened"))?;
        cursor = match entry {
            Value::Object(map) => map,
            Value::Array(items) => items
                .last_mut()
                .and_then(Value::as_object_mut)
                .ok_or_else(|| format!("line {number}: {part} is an empty array of tables"))?,
            other => return Err(format!("line {number}: {part} is already {other}")),
        };
    }
    Ok(cursor)
}

/// Open a `[path]` table, creating it and every parent it needs.
fn open_table(root: &mut Map<String, Value>, path: &[String], number: usize) -> Result<(), String> {
    let (last, parents) = path
        .split_last()
        .ok_or_else(|| format!("line {number}: an empty table header"))?;
    let parent = table_at(root, parents, number)?;
    if parent.contains_key(last) {
        return Err(format!("line {number}: the table {last} is opened twice"));
    }
    parent.insert(last.clone(), Value::Object(Map::new()));
    Ok(())
}

/// Append a `[[path]]` entry, creating the array and every parent it needs.
fn append_table(
    root: &mut Map<String, Value>,
    path: &[String],
    number: usize,
) -> Result<(), String> {
    let (last, parents) = path
        .split_last()
        .ok_or_else(|| format!("line {number}: an empty array-of-tables header"))?;
    let parent = table_at(root, parents, number)?;
    let entry = parent
        .entry(last.clone())
        .or_insert_with(|| Value::Array(Vec::new()));
    let items = entry
        .as_array_mut()
        .ok_or_else(|| format!("line {number}: {last} is not an array of tables"))?;
    items.push(Value::Object(Map::new()));
    Ok(())
}

/// Split a dotted header path, honouring quoted segments.
fn split_path(text: &str, number: usize) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut rest = text.trim();
    loop {
        let (part, tail) = read_key(rest, number)?;
        parts.push(part);
        let tail = tail.trim_start();
        match tail.strip_prefix('.') {
            Some(more) => rest = more.trim_start(),
            None if tail.is_empty() => return Ok(parts),
            None => return Err(format!("line {number}: {tail} follows a header path")),
        }
    }
}

/// Split `key = value`, honouring a quoted key.
fn split_assignment(line: &str, number: usize) -> Result<(String, Value), String> {
    let (key, tail) = read_key(line, number)?;
    let rest = tail
        .trim_start()
        .strip_prefix('=')
        .ok_or_else(|| format!("line {number}: no `=` after the key"))?
        .trim();
    Ok((key, read_value(rest, number)?))
}

/// Read one key, bare or quoted, and return what follows it.
fn read_key(text: &str, number: usize) -> Result<(String, &str), String> {
    let text = text.trim_start();
    if let Some(rest) = text.strip_prefix('"') {
        let mut escaped = false;
        for (index, c) in rest.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            match c {
                '\\' => escaped = true,
                '"' => {
                    let key: String = serde_json::from_str(&text[..=index + 1])
                        .map_err(|err| format!("line {number}: a quoted key: {err}"))?;
                    return Ok((key, &rest[index + 1..]));
                }
                _ => {}
            }
        }
        return Err(format!("line {number}: a quoted key is not closed"));
    }
    let end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .unwrap_or(text.len());
    if end == 0 {
        return Err(format!("line {number}: no key here"));
    }
    Ok((text[..end].to_owned(), &text[end..]))
}

/// Read one value: a scalar, or an inline array of them.
fn read_value(text: &str, number: usize) -> Result<Value, String> {
    if text == "{}" {
        return Ok(Value::Object(Map::new()));
    }
    if let Some(inner) = text.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        let inner = inner.trim();
        if inner.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let mut items = Vec::new();
        for part in split_array(inner, number)? {
            items.push(read_value(part.trim(), number)?);
        }
        return Ok(Value::Array(items));
    }
    match text {
        "true" => Ok(Value::Bool(true)),
        "false" => Ok(Value::Bool(false)),
        // ⛔ There is no `null` branch, and its absence is the format's whole
        // documented limit. A file carrying one was not written by this
        // generator.
        _ => serde_json::from_str(text).map_err(|err| {
            format!("line {number}: {text} is not a value this reader accepts: {err}")
        }),
    }
}

/// Split an inline array's elements at its own level.
///
/// ⚠ **A comma inside a quoted string or inside a nested array is not a
/// separator.** Nothing in this model nests arrays today, and a splitter that
/// only handled the quoting would turn the first one that does into two wrong
/// elements rather than into a refusal.
fn split_array(inner: &str, number: usize) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut start = 0_usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut depth = 0_i32;
    for (index, c) in inner.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '[' if !quoted => depth += 1,
            ']' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                parts.push(&inner[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if quoted {
        return Err(format!("line {number}: an array has an unclosed string"));
    }
    if depth != 0 {
        return Err(format!("line {number}: an array's brackets do not balance"));
    }
    parts.push(&inner[start..]);
    Ok(parts)
}
