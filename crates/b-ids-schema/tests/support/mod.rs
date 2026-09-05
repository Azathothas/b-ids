//! Test support: a fixture profile, and a JSON Schema checker with a guard.
//!
//! ⭐ **The guard is the point of writing a checker rather than taking one.**
//! `KNOWN_KEYWORDS` is the list this checker implements, and
//! [`check_schema_is_supported`] refuses a schema that uses anything outside
//! it. Without that, a keyword nobody implemented would be a constraint the
//! published schema states and nothing enforces, which is the shape of every
//! defect this repository has found in its own checks.
//!
//! ⚠ **It is a subset, and the subset is declared rather than implied.** It
//! validates what `schema/browser-profile-1.schema.json` actually uses. It is
//! not a JSON Schema implementation and nothing here should be reached for as
//! one.

// ⚠ This module is compiled into every test binary in this directory and each
// one uses a different part of it, so what is unused HERE is used next door.
#![allow(dead_code, unused_imports)]

use std::path::PathBuf;

use b_ids_schema::Profile;
use serde_json::Value;

/// Every keyword this checker implements.
///
/// ⛔ Adding a keyword to the schema without adding it here fails
/// [`check_schema_is_supported`], on purpose.
pub const KNOWN_KEYWORDS: &[&str] = &[
    // Annotations, read by a person and ignored here.
    "$schema",
    "$id",
    "title",
    "description",
    // Structure.
    "$defs",
    "$ref",
    "type",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "enum",
    "const",
    "oneOf",
    "minItems",
    "minimum",
    "maximum",
];

/// The published schema, read from the crate's own directory.
pub fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schema/browser-profile-1.schema.json")
}

/// The published schema, parsed.
pub fn schema() -> Value {
    let text = std::fs::read_to_string(schema_path()).expect("the published schema is readable");
    serde_json::from_str(&text).expect("the published schema is valid JSON")
}

/// Every keyword the schema uses that this checker does not implement.
pub fn check_schema_is_supported(schema: &Value) -> Vec<String> {
    fn walk(node: &Value, path: &str, unknown: &mut Vec<String>) {
        let Value::Object(map) = node else { return };
        for (key, value) in map {
            if KNOWN_KEYWORDS.contains(&key.as_str()) {
                match key.as_str() {
                    // These hold named subschemas, so their KEYS are names
                    // rather than keywords.
                    "properties" | "$defs" => {
                        if let Value::Object(children) = value {
                            for (name, child) in children {
                                walk(child, &format!("{path}/{key}/{name}"), unknown);
                            }
                        }
                    }
                    // These hold subschemas directly.
                    "items" | "additionalProperties" => {
                        walk(value, &format!("{path}/{key}"), unknown);
                    }
                    "oneOf" => {
                        if let Value::Array(items) = value {
                            for (i, child) in items.iter().enumerate() {
                                walk(child, &format!("{path}/oneOf/{i}"), unknown);
                            }
                        }
                    }
                    // Leaf keywords carry data, not subschemas.
                    _ => {}
                }
            } else {
                unknown.push(format!("{path}/{key}"));
            }
        }
    }
    let mut unknown = Vec::new();
    walk(schema, "#", &mut unknown);
    unknown
}

/// Validate an instance against a schema, returning every problem found.
pub fn validate(schema: &Value, instance: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    validate_at(schema, schema, instance, "$", &mut problems);
    problems
}

fn resolve<'a>(root: &'a Value, node: &'a Value) -> &'a Value {
    let Some(reference) = node.get("$ref").and_then(Value::as_str) else {
        return node;
    };
    let name = reference
        .strip_prefix("#/$defs/")
        .expect("only #/$defs/NAME references are used");
    root.get("$defs")
        .and_then(|d| d.get(name))
        .unwrap_or_else(|| panic!("the schema references $defs/{name}, which is not defined"))
}

fn type_matches(expected: &str, instance: &Value) -> bool {
    match expected {
        "object" => instance.is_object(),
        "array" => instance.is_array(),
        "string" => instance.is_string(),
        "boolean" => instance.is_boolean(),
        "null" => instance.is_null(),
        "integer" => instance.is_i64() || instance.is_u64(),
        "number" => instance.is_number(),
        other => panic!("the schema names a type this checker does not implement: {other}"),
    }
}

fn validate_at(
    root: &Value,
    node: &Value,
    instance: &Value,
    path: &str,
    problems: &mut Vec<String>,
) {
    let node = resolve(root, node);

    if let Some(expected) = node.get("const")
        && instance != expected
    {
        problems.push(format!("{path}: expected {expected}, found {instance}"));
    }

    if let Some(Value::Array(allowed)) = node.get("enum")
        && !allowed.contains(instance)
    {
        problems.push(format!("{path}: {instance} is not one of {allowed:?}"));
    }

    match node.get("type") {
        Some(Value::String(t)) => {
            if !type_matches(t, instance) {
                problems.push(format!("{path}: expected type {t}, found {instance}"));
                return;
            }
        }
        Some(Value::Array(types)) => {
            let ok = types
                .iter()
                .filter_map(Value::as_str)
                .any(|t| type_matches(t, instance));
            if !ok {
                problems.push(format!(
                    "{path}: expected one of {types:?}, found {instance}"
                ));
                return;
            }
        }
        _ => {}
    }

    // ⛔ BOUNDS ARE CHECKED, not merely allowed as a keyword. The schema said
    // nothing about width until 2026-09-02, so a profile claiming 999 in a
    // byte-wide field satisfied the contract this project PUBLISHES and failed
    // the one it implements. `docs/history/todo/schema.md`, `SCHEMA-13`.
    // ⚠ Read as f64 rather than i64, because a number outside the Rust width is
    // exactly the case being refused and `as_i64` returns None for some of them.
    if let Some(number) = instance.as_f64() {
        if let Some(minimum) = node.get("minimum").and_then(Value::as_f64)
            && number < minimum
        {
            problems.push(format!("{path}: {instance} is below the minimum {minimum}"));
        }
        if let Some(maximum) = node.get("maximum").and_then(Value::as_f64)
            && number > maximum
        {
            problems.push(format!("{path}: {instance} is above the maximum {maximum}"));
        }
    }

    if let Some(Value::Array(branches)) = node.get("oneOf") {
        let matching = branches
            .iter()
            .filter(|branch| {
                let mut scratch = Vec::new();
                validate_at(root, branch, instance, path, &mut scratch);
                scratch.is_empty()
            })
            .count();
        if matching != 1 {
            problems.push(format!(
                "{path}: matched {matching} of {} oneOf branches, expected exactly 1",
                branches.len()
            ));
        }
    }

    if let Some(object) = instance.as_object() {
        if let Some(Value::Array(required)) = node.get("required") {
            for name in required.iter().filter_map(Value::as_str) {
                if !object.contains_key(name) {
                    problems.push(format!("{path}.{name}: required and absent"));
                }
            }
        }

        let properties = node.get("properties").and_then(Value::as_object);
        for (name, value) in object {
            if let Some(subschema) = properties.and_then(|p| p.get(name)) {
                validate_at(root, subschema, value, &format!("{path}.{name}"), problems);
                continue;
            }
            match node.get("additionalProperties") {
                Some(Value::Bool(false)) => {
                    problems.push(format!("{path}.{name}: not declared by the schema"));
                }
                Some(subschema @ Value::Object(_)) => {
                    validate_at(root, subschema, value, &format!("{path}.{name}"), problems);
                }
                _ => {}
            }
        }
    }

    if let Some(array) = instance.as_array() {
        if let Some(min) = node.get("minItems").and_then(Value::as_u64)
            && (array.len() as u64) < min
        {
            problems.push(format!("{path}: {} items, minimum {min}", array.len()));
        }
        if let Some(items) = node.get("items") {
            for (i, item) in array.iter().enumerate() {
                validate_at(root, items, item, &format!("{path}[{i}]"), problems);
            }
        }
    }
}

/// The shared fixture, so this crate and `b-ids-validator` test the same one.
pub use b_ids_schema::fixture::{
    http as fixture_http, http2 as fixture_http2, profile as fixture, raw_headers,
    tls as fixture_tls,
};

/// A profile as JSON, which is what the schema is checked against.
pub fn as_json(profile: &Profile) -> Value {
    serde_json::to_value(profile).expect("a profile serialises")
}

/// Every defect's message, joined, so a test can assert on the field name.
pub fn messages(defects: &[b_ids_schema::Defect]) -> String {
    defects
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
