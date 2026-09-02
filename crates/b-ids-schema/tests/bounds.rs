//! SCHEMA-13. The published schema accepts 999 for a byte.
//!
//! The acceptance: every integer field in the published schema carries a
//! `minimum` and a `maximum` that match its Rust width; a profile with 999 in a
//! byte-wide field is refused by the published schema as well as by the type;
//! and a field added without a bound fails.
//!
//! ⛔ Every test name starts with `bounds`, because
//! `cargo test -p b-ids-schema bounds` is the entry's acceptance command.
//!
//! ⭐ **The expected bounds are DERIVED from the Rust widths, never typed.** A
//! table of numbers beside a table of types is a value in two places with no
//! check that they agree, and the drift it produces is silent: a field widened
//! in Rust and left alone in the schema publishes a contract that refuses values
//! the implementation now accepts.

mod support;

use serde_json::Value;

use support::{schema, validate};

/// The largest integer a JSON number carries exactly.
///
/// ⛔ **This is a property of the wire format, not of Rust.** `usize` is wider
/// than this on every host that runs the harness, and a consumer reading a
/// larger value out of JSON reads a different number than was written. The two
/// `usize`-backed fields are bounded here rather than at `usize::MAX`, and the
/// schema says so in its own description.
const JSON_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// Every integer field the schema carries, with the bound its Rust type gives.
///
/// ⛔ **The pointer list is the checked half.** `bounds_every_integer_field_is_in_this_table`
/// walks the schema and fails on an integer field this list does not name, so a
/// field added without a bound cannot pass by being forgotten here.
fn expected() -> Vec<(&'static str, f64, f64)> {
    vec![
        ("#/u8", 0.0, f64::from(u8::MAX)),
        ("#/u16", 0.0, f64::from(u16::MAX)),
        ("#/u32", 0.0, f64::from(u32::MAX)),
        // Option<u16> and Option<u32>: the null is the absence, and the bound
        // is the width of the value when it is there.
        ("#/tls/padding_len", 0.0, f64::from(u16::MAX)),
        ("#/http2/connection_window", 0.0, f64::from(u32::MAX)),
        // usize on the Rust side, bounded by what JSON can carry exactly.
        ("#/captured/acquisition/bytes", 0.0, JSON_SAFE_INTEGER),
        ("#/record_layer/bytes_arrived", 0.0, JSON_SAFE_INTEGER),
        // ⛔ The minimum is 1 rather than 0. A zero-length random part records a
        // CONSTANT, which is what the field exists to avoid.
        ("#/multipart_boundary/random_len", 1.0, JSON_SAFE_INTEGER),
    ]
}

/// Every integer-typed node in the schema, by the path this test names it with.
fn integer_fields(node: &Value, path: String, out: &mut Vec<(String, Value)>) {
    let Some(object) = node.as_object() else {
        return;
    };
    let is_integer = match object.get("type") {
        Some(Value::String(t)) => t == "integer",
        Some(Value::Array(types)) => types.iter().any(|t| t == "integer"),
        _ => false,
    };
    if is_integer {
        out.push((path.clone(), node.clone()));
    }
    for (key, value) in object {
        match key.as_str() {
            "properties" | "$defs" => {
                if let Some(named) = value.as_object() {
                    for (name, sub) in named {
                        integer_fields(sub, format!("{path}/{name}"), out);
                    }
                }
            }
            "items" => integer_fields(value, format!("{path}/items"), out),
            "oneOf" => {
                if let Some(branches) = value.as_array() {
                    for (i, sub) in branches.iter().enumerate() {
                        integer_fields(sub, format!("{path}/oneOf[{i}]"), out);
                    }
                }
            }
            _ => {}
        }
    }
}

#[test]
fn bounds_every_integer_field_is_in_this_table() {
    // ⛔ THE GUARD AGAINST A FIELD ADDED WITHOUT A BOUND. A schema that gained
    // an integer nobody bounded would otherwise pass every other test here,
    // because they all iterate the table rather than the file.
    let schema = schema();
    let mut found = Vec::new();
    integer_fields(&schema, "#".to_owned(), &mut found);
    let named: Vec<&str> = expected().iter().map(|(p, _, _)| *p).collect();
    for (path, _) in &found {
        assert!(
            named.contains(&path.as_str()),
            "{path} is an integer field with no row in this test's table. Add it with the bound \
             its Rust type gives, or bound it in the schema and add it here"
        );
    }
    assert_eq!(
        found.len(),
        named.len(),
        "the table names {} field(s) and the schema has {}: {found:?}",
        named.len(),
        found.len()
    );
}

#[test]
fn bounds_every_integer_field_carries_the_bound_its_rust_type_gives() {
    let schema = schema();
    let mut found = Vec::new();
    integer_fields(&schema, "#".to_owned(), &mut found);
    for (path, minimum, maximum) in expected() {
        let (_, node) = found
            .iter()
            .find(|(p, _)| p == path)
            .unwrap_or_else(|| panic!("{path} is named in the table and not in the schema"));
        let carried_min = node
            .get("minimum")
            .and_then(Value::as_f64)
            .unwrap_or_else(|| panic!("{path} carries no minimum"));
        let carried_max = node
            .get("maximum")
            .and_then(Value::as_f64)
            .unwrap_or_else(|| panic!("{path} carries no maximum"));
        assert!(
            (carried_min - minimum).abs() < f64::EPSILON,
            "{path}: schema says minimum {carried_min}, the Rust width gives {minimum}"
        );
        assert!(
            (carried_max - maximum).abs() < f64::EPSILON,
            "{path}: schema says maximum {carried_max}, the Rust width gives {maximum}"
        );
    }
}

#[test]
fn bounds_the_published_schema_refuses_999_for_a_byte() {
    // ⭐ THE ENTRY'S OWN EXAMPLE, run against the published contract rather than
    // against the Rust type. Before this change the schema accepted it, so a
    // consumer validating against the file this project publishes accepted a
    // profile the implementation cannot hold.
    let schema = schema();
    let mut profile =
        serde_json::to_value(b_ids_schema::fixture::profile()).expect("the fixture serialises");
    assert!(
        validate(&schema, &profile).is_empty(),
        "the fixture conforms before it is broken: {:?}",
        validate(&schema, &profile)
    );

    // ⚠ `session_id_len` is a `u8` on both sides, which is why 999 is the entry
    // title's number: it is representable in JSON, it is not representable in the
    // Rust type, and until this change the published schema accepted it.
    profile["tls"]["session_id_len"] = Value::from(999);
    let problems = validate(&schema, &profile);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("999") && p.contains("above the maximum")),
        "999 in a byte-wide field is refused by the published schema: {problems:?}"
    );
}

#[test]
fn bounds_a_negative_value_is_refused_where_the_rust_type_is_unsigned() {
    // ⛔ THE OTHER END, and it is not symmetry for its own sake. Every integer
    // this schema carries is read into an UNSIGNED Rust type, so a negative
    // value is exactly as unrepresentable as an oversized one, and a schema that
    // bounded only the top would publish half a contract.
    let schema = schema();
    let mut profile =
        serde_json::to_value(b_ids_schema::fixture::profile()).expect("the fixture serialises");
    profile["tls"]["record_version"] = Value::from(-1);
    let problems = validate(&schema, &profile);
    assert!(
        problems.iter().any(|p| p.contains("below the minimum")),
        "a negative record version is refused: {problems:?}"
    );
}
