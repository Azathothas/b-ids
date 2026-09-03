//! Protobuf, as a definition with no binaries.
//!
//! ⭐ **This is `SCHEMA-12`'s recommendation.** A typed consumer generates its
//! own decoder from the definition, so the corpus owes nothing: no codec
//! dependency here, no generated Rust in this tree, and no encoded artefact
//! that has to be regenerated whenever a field moves.
//!
//! ⛔ **The definition is DERIVED FROM THE PUBLISHED CORPUS, and it says so in
//! its own header.** A field absent from every published profile is absent
//! here, and a field that is null in every published profile is written as an
//! `unmeasured` comment rather than as a field with a type nobody measured.
//! ⚠ That is the honest limit of a definition generated from data, and the
//! alternative loses on a worse axis: a hand-written definition drifts from the
//! model the first time a field moves, and nothing notices.
//!
//! ⭐ **What makes it checkable is that it is read back.** [`parse`] reads the
//! generated text into the same [`Definition`] that [`declared`] derives from
//! the profiles, so a field or a comment deleted from the file fails the round
//! trip.
//!
//! `TODO/schema.md`, `SCHEMA-12`.

use std::collections::{BTreeMap, BTreeSet};

use b_ids_schema::Profile;
use serde_json::Value;

/// The proto package the definition declares.
pub const PACKAGE: &str = "b_ids.corpus.v1";

/// The message one profile is.
pub const ROOT_MESSAGE: &str = "Profile";

/// The message wrapping the whole corpus, which is written by hand rather than
/// derived, so it carries no corpus-derived field.
pub const WRAPPER_MESSAGE: &str = "Corpus";

/// The comment marking a field the corpus has never carried a value for.
pub const UNMEASURED: &str = "// unmeasured: ";

/// Object paths whose keys are not proto identifiers, so the object is a `map`.
///
/// ⛔ **DATA, and a refusal rather than a guess for anything not in it.** The
/// provenance map is keyed by field path, and a dot is not a character a proto
/// field name may carry. A generator that decided map-or-message by looking at
/// whether the keys happened to be identifier-shaped would change the published
/// shape the day a profile's keys changed, which is a contract moving under a
/// consumer for a reason nobody chose. `TODO/schema.md`, `SCHEMA-12`.
const MAP_PATHS: [&str; 1] = ["provenance"];

/// One field of one message, as the definition declares it.
pub type Field = (String, String);

/// Every message the definition declares, each with its fields in order.
pub type Messages = BTreeMap<String, Vec<Field>>;

/// What the definition says, independently of how it is written down.
///
/// ⭐ **Comparable, which is the point.** The acceptance compares what the
/// corpus implies with what the generated file says, and a type with no
/// equality could only be compared as text.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Definition {
    /// The messages, each with its fields in declaration order.
    pub messages: Messages,
    /// Per message, the keys the corpus carries and has never given a value.
    pub unmeasured: BTreeMap<String, Vec<String>>,
}

/// What one path in the corpus was seen to hold.
#[derive(Debug, Default)]
struct Shape {
    /// How many objects at this path were visited.
    occurrences: usize,
    /// How many of those carried this key with a value that is not null.
    present: usize,
    /// Whether the value was ever null.
    nulls: bool,
    /// The scalar proto types seen at this path.
    scalars: BTreeSet<&'static str>,
    /// The keys seen, where the value was an object.
    object: BTreeMap<String, Shape>,
    /// What the elements were, where the value was an array.
    array: Option<Box<Shape>>,
}

impl Shape {
    /// Merge one value into this path's shape.
    fn merge(&mut self, value: &Value, path: &str) -> Result<(), String> {
        match value {
            Value::Null => self.nulls = true,
            Value::Bool(_) => {
                self.scalars.insert("bool");
            }
            Value::Number(n) => {
                self.scalars
                    .insert(if n.is_f64() { "double" } else { "int64" });
            }
            Value::String(_) => {
                self.scalars.insert("string");
            }
            Value::Object(map) => {
                // ⚠ Counted HERE and nowhere else, so an object reached
                // through an array is counted once rather than twice.
                self.occurrences += 1;
                for (key, child) in map {
                    let entry = self.object.entry(key.clone()).or_default();
                    entry.present += usize::from(!child.is_null());
                    entry.merge(child, &join(path, key))?;
                }
            }
            Value::Array(items) => {
                let element = self.array.get_or_insert_with(Box::default);
                for item in items {
                    element.merge(item, &format!("{path}[]"))?;
                }
            }
        }
        Ok(())
    }

    /// Whether this path is a message rather than a scalar or an array.
    fn is_message(&self) -> bool {
        !self.object.is_empty()
    }

    /// The scalar type at this path, where exactly one was seen.
    fn scalar(&self, path: &str) -> Result<Option<&'static str>, String> {
        match self.scalars.len() {
            0 => Ok(None),
            1 => Ok(self.scalars.iter().next().copied()),
            // ⛔ Refused rather than widened to `string`. Two types at one path
            // is a model defect, and a definition that hid it would publish a
            // contract nothing in the corpus satisfies.
            _ => Err(format!(
                "{path} holds more than one type across the corpus: {}",
                self.scalars.iter().copied().collect::<Vec<_>>().join(", ")
            )),
        }
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

/// Whether a name may be a proto identifier.
fn identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b) if b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// The message name for one path, derived rather than typed.
fn message_name(path: &str) -> String {
    let mut out = String::from(ROOT_MESSAGE);
    for segment in path.split('.').filter(|s| !s.is_empty()) {
        for word in segment.split(['_', '-']).filter(|w| !w.is_empty()) {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }
    out
}

/// The definition the published corpus implies.
///
/// # Errors
///
/// A refusal naming the path, where one holds two types, or where an object's
/// keys are not proto identifiers and the path is not in [`MAP_PATHS`].
pub fn declared(profiles: &[Profile]) -> Result<Definition, String> {
    let mut root = Shape::default();
    for profile in profiles {
        let value = serde_json::to_value(profile)
            .map_err(|err| format!("serialising a profile for the definition: {err}"))?;
        root.merge(&value, "")?;
    }
    let mut definition = Definition::default();
    emit(&root, "", &mut definition)?;
    Ok(definition)
}

/// Walk one message's shape, adding it and every message under it.
fn emit(shape: &Shape, path: &str, definition: &mut Definition) -> Result<(), String> {
    let mut fields: Vec<Field> = Vec::new();
    let mut unmeasured: Vec<String> = Vec::new();
    for (key, child) in &shape.object {
        if !identifier(key) {
            return Err(format!(
                "{}: the key is not a proto identifier, and {} is not in the map table",
                join(path, key),
                if path.is_empty() { "the root" } else { path }
            ));
        }
        let child_path = join(path, key);
        let Some(type_text) = field_type(child, &child_path, definition)? else {
            unmeasured.push(key.clone());
            continue;
        };
        // ⚠ `optional` gives a proto3 consumer field presence, which is what
        // distinguishes a value measured as zero from one never recorded.
        // Repeated and map fields carry presence already.
        let presence = if type_text.starts_with("repeated ") || type_text.starts_with("map<") {
            ""
        } else if child.nulls || child.present < shape.occurrences {
            "optional "
        } else {
            ""
        };
        fields.push((key.clone(), format!("{presence}{type_text}")));
    }
    let name = message_name(path);
    if !unmeasured.is_empty() {
        definition.unmeasured.insert(name.clone(), unmeasured);
    }
    definition.messages.insert(name, fields);
    Ok(())
}

/// The type text for one field, adding any message it needs.
///
/// ⚠ `None` where the corpus has never carried a value at this path, which the
/// caller writes as an `unmeasured` comment rather than as a field.
fn field_type(
    shape: &Shape,
    path: &str,
    definition: &mut Definition,
) -> Result<Option<String>, String> {
    if MAP_PATHS.contains(&path) {
        let mut value_types = BTreeSet::new();
        for child in shape.object.values() {
            if let Some(scalar) = child.scalar(path)? {
                value_types.insert(scalar);
            }
        }
        return match value_types.len() {
            0 => Ok(None),
            1 => Ok(value_types
                .iter()
                .next()
                .map(|value| format!("map<string, {value}>"))),
            _ => Err(format!("{path}: a map whose values are not all one type")),
        };
    }
    if let Some(element) = &shape.array {
        // ⚠ An array that has only ever been EMPTY has no element type, and
        // `repeated string` would be a type nobody measured. It becomes an
        // `unmeasured` comment for the same reason a null-everywhere scalar
        // does.
        let inner = field_type(element, &format!("{path}[]"), definition)?;
        return Ok(inner.map(|text| format!("repeated {text}")));
    }
    if shape.is_message() {
        emit(shape, path, definition)?;
        return Ok(Some(message_name(path)));
    }
    Ok(shape.scalar(path)?.map(str::to_owned))
}

/// Render the definition.
///
/// # Errors
///
/// Whatever [`declared`] refuses.
pub fn render(profiles: &[Profile]) -> Result<String, String> {
    let definition = declared(profiles)?;
    let mut out = String::new();
    out.push_str("// The corpus, as a protobuf definition. Generated: do not edit.\n");
    out.push_str(
        "// It is DERIVED FROM THE PUBLISHED CORPUS. A field absent from every published\n\
         // profile is absent here, and a field that is null in every published profile is\n\
         // an unmeasured comment rather than a field, because its type has not been\n\
         // measured. Nothing publishes encoded protobuf: a consumer generates its own\n\
         // decoder, and this project then owes no codec.\n",
    );
    out.push_str("\nsyntax = \"proto3\";\n");
    out.push_str(&format!("\npackage {PACKAGE};\n"));
    // ⚠ Pieces rather than one format string. A doubled brace is how a format
    // string spells a literal one, and `scripts/common/check-placeholders`
    // reads a doubled brace pair on a line as a template nobody filled in.
    out.push_str("\nmessage ");
    out.push_str(WRAPPER_MESSAGE);
    out.push_str(" {\n  repeated ");
    out.push_str(ROOT_MESSAGE);
    out.push_str(" profiles = 1;\n}\n");
    for (name, fields) in &definition.messages {
        out.push_str("\nmessage ");
        out.push_str(name);
        out.push_str(" {\n");
        for (index, (field, type_text)) in fields.iter().enumerate() {
            out.push_str(&format!("  {type_text} {field} = {};\n", index + 1));
        }
        for key in definition.unmeasured.get(name).into_iter().flatten() {
            out.push_str(&format!("  {UNMEASURED}{key}\n"));
        }
        out.push_str("}\n");
    }
    Ok(out)
}

/// Read the definition back into the same [`Definition`] [`declared`] produces.
///
/// ⛔ **This is what makes the definition's round trip real.** A field deleted
/// from the generated text produces a definition that does not equal the one
/// the corpus implies, and the acceptance compares them.
///
/// # Errors
///
/// A refusal naming the line this reader does not accept.
pub fn parse(text: &str) -> Result<Definition, String> {
    let mut definition = Definition::default();
    let mut current: Option<(String, Vec<Field>, Vec<String>)> = None;
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let number = index + 1;
        if line.is_empty() {
            continue;
        }
        if let Some(key) = line.strip_prefix(UNMEASURED) {
            let (_, _, unmeasured) = current
                .as_mut()
                .ok_or_else(|| format!("line {number}: an unmeasured key outside every message"))?;
            unmeasured.push(key.trim().to_owned());
            continue;
        }
        if line.starts_with("//") {
            continue;
        }
        if let Some(rest) = line.strip_prefix("message ") {
            if let Some((previous, _, _)) = &current {
                return Err(format!("line {number}: {previous} was never closed"));
            }
            let name = rest
                .trim()
                .strip_suffix('{')
                .ok_or_else(|| format!("line {number}: a message that does not open"))?
                .trim();
            current = Some((name.to_owned(), Vec::new(), Vec::new()));
            continue;
        }
        if line == "}" {
            let (name, fields, unmeasured) = current
                .take()
                .ok_or_else(|| format!("line {number}: a closing brace with no message open"))?;
            if name == WRAPPER_MESSAGE {
                continue;
            }
            if !unmeasured.is_empty() {
                definition.unmeasured.insert(name.clone(), unmeasured);
            }
            if definition.messages.insert(name.clone(), fields).is_some() {
                return Err(format!("line {number}: {name} is declared twice"));
            }
            continue;
        }
        let Some((_, fields, _)) = current.as_mut() else {
            if line.starts_with("syntax ") || line.starts_with("package ") {
                continue;
            }
            return Err(format!("line {number}: {line} is outside every message"));
        };
        let body = line
            .strip_suffix(';')
            .ok_or_else(|| format!("line {number}: a field with no semicolon"))?;
        let (declaration, _) = body
            .rsplit_once('=')
            .ok_or_else(|| format!("line {number}: a field with no number"))?;
        let (type_text, field) = declaration
            .trim()
            .rsplit_once(' ')
            .ok_or_else(|| format!("line {number}: a field with no name"))?;
        fields.push((field.trim().to_owned(), type_text.trim().to_owned()));
    }
    if let Some((name, _, _)) = current {
        return Err(format!("{name} was never closed"));
    }
    Ok(definition)
}
