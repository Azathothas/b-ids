//! Print what this crate answers, as JSON, so another language's package can be
//! compared against it.
//!
//! ⛔ **`LIB-03`'s Prove compares ANSWERS rather than interfaces**, and this is
//! the Rust side of that comparison. `scripts/common/check-bindings` runs this
//! and the JavaScript package's equivalent over one corpus and requires the two
//! documents to be identical.
//!
//! ⭐ **An example rather than a binary**, deliberately: `LIB-01` is a library
//! that hands a program a profile, and adding a command to it would change what
//! that entry shipped. An example is built by `cargo` on request and is not part
//! of the crate's own interface.
//!
//! ⚠ **Every answer here is one a consumer actually asks for**, including the
//! one that must come back empty: a route the corpus does not hold. `LIB-03`
//! names that case specifically, because two implementations agree easily on
//! what exists and disagree on what does not.

use b_ids::schema::Channel;
use b_ids::{Version, at, client_hello_hex, latest_stable, paths, profiles, release, select};

fn id(profile: Option<&b_ids::schema::Profile>) -> serde_json::Value {
    profile.map_or(serde_json::Value::Null, |p| {
        serde_json::Value::String(p.id.to_string())
    })
}

fn main() {
    let r = release();
    let answers = serde_json::json!({
        "release": {
            "identifier": r.identifier,
            "layout": r.layout,
            "profiles": r.profiles,
            "newestCapture": r.newest_capture,
        },
        // ⚠ THE PATHS IN THE INDEX'S OWN ORDER, because the order is an answer
        // too: `paths()` and `profiles()` are index-aligned in both packages or
        // one of them is wrong.
        "paths": paths(),
        "ids": profiles().iter().map(|p| p.id.to_string()).collect::<Vec<_>>(),
        "at_first": id(paths().first().and_then(|p| at(p))),
        // ⛔ THE ABSENT CASES, which LIB-03 names. Two implementations agree
        // easily on what exists.
        "at_missing": id(at("corpus/v1/nothing/here/at/all.json")),
        "latest_chrome_linux64": id(latest_stable("chrome", "linux64")),
        "latest_chrome_win64": id(latest_stable("chrome", "win64")),
        "latest_firefox_linux64": id(latest_stable("firefox", "linux64")),
        "latest_chromium_linux64": id(latest_stable("chromium", "linux64")),
        "latest_chrome_macos": id(latest_stable("chrome", "macos-arm64")),
        "latest_safari_linux64": id(latest_stable("safari", "linux64")),
        // ⚠ CASE IS FOLDED ON BOTH SIDES, and asking proves it rather than
        // assuming it.
        "latest_upper_case": id(latest_stable("CHROME", "LINUX64")),
        "select_for_testing_linux64": id(select(
            "chrome",
            Channel::ForTesting,
            "linux64",
            Version::Latest,
        )),
        "hello_bytes": latest_stable("chrome", "linux64")
            .and_then(client_hello_hex)
            .map_or(0, |hex| hex.len() / 2),
    });
    match serde_json::to_string_pretty(&answers) {
        Ok(text) => println!("{text}"),
        Err(why) => {
            eprintln!("b-ids answers: {why}");
            std::process::exit(1);
        }
    }
}
