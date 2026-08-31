//! Print the fixture profile as JSON, so the shape can be read and the
//! validator's command can be driven against a real file.
//!
//! ```text
//! cargo run -p b-ids-schema --features fixtures --example dump
//! ```
//!
//! ⛔ **What it prints is not a measurement.** It is the fixture, shaped like a
//! capture and not one, and no field of it may enter the corpus.

fn main() {
    let profile = b_ids_schema::fixture::profile_with_header_values();
    match serde_json::to_string_pretty(&profile) {
        Ok(text) => println!("{text}"),
        Err(err) => {
            eprintln!("dump: {err}");
            std::process::exit(1);
        }
    }
}
