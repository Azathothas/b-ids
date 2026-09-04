//! A package per ecosystem, generated from the corpus rather than maintained.
//!
//! ⛔ **Fetching and parsing a corpus is work. A dependency line is not.**
//! `TODO/publish.md`, `PUB-05`.
//!
//! ⭐ **GENERATED, NEVER HAND-MAINTAINED.** A package somebody edits is a
//! second copy of the corpus that drifts from the first, and the drift is
//! invisible because both sides look like data. Everything here is written by
//! the assembler from the same corpus every other published surface is built
//! from.
//!
//! ⛔ **IT EMBEDS AND IT DOES NOT FETCH.** The entry's Must-not: a package that
//! needs the network to answer is a package that fails in the environment its
//! consumers care most about. Nothing generated here opens a socket.
//!
//! ⭐ **The Rust package is the reference and this follows its shape.**
//! [`crates/b-ids`](../../b-ids) is `LIB-01`: `profiles()`, `paths()`,
//! `release()`, `at()`, `latestStable()` and `select()`, with `release()`
//! carrying the SHA-256 of the corpus index so a consumer can tell how old
//! their data is without leaving their own language.
//!
//! ⚠ **The identifier is the pin.** The index carries a digest of every
//! published file, so an identifier over the index pins the corpus
//! transitively: two builds reporting the same identifier embedded the same
//! bytes. ⛔ `scripts/common/check-packages` recomputes it with `sha256sum`,
//! which is not this project's code.

use b_ids_schema::Profile;

/// One generated file in one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// Where it goes, relative to the published tree.
    pub path: String,
    /// What it holds.
    pub text: String,
}

/// The ecosystems this project generates a package for.
///
/// ⛔ **Named rather than counted.** The entry says one package per major
/// ecosystem; what exists is what is listed here, and a row with no generator
/// behind it would be a claim rather than a package.
pub const ECOSYSTEMS: [&str; 1] = ["js"];

/// Build every package the assembler publishes.
///
/// `index_json` is the corpus index verbatim, and `release` is its digest.
///
/// # Errors
///
/// Where a profile does not serialise, which is a corpus problem rather than a
/// packaging one.
pub fn packages(
    profiles: &[(String, Profile)],
    index_json: &str,
    pointers_json: &str,
    release: &str,
) -> Result<Vec<File>, String> {
    let mut out = Vec::new();
    out.extend(javascript(profiles, index_json, pointers_json, release)?);
    Ok(out)
}

/// The JavaScript package.
///
/// ⚠ **ECMAScript modules and no build step.** A package that needed a bundler
/// to be read would put a toolchain between a consumer and the data, which is
/// the work this entry exists to remove.
fn javascript(
    profiles: &[(String, Profile)],
    index_json: &str,
    pointers_json: &str,
    release: &str,
) -> Result<Vec<File>, String> {
    let mut entries = Vec::with_capacity(profiles.len());
    for (path, profile) in profiles {
        let value = serde_json::to_value(profile)
            .map_err(|e| format!("serialising {} for the js package: {e}", profile.id))?;
        // ⛔ THE PLATFORM TOKEN IS DERIVED AND IT IS DERIVED ONCE, HERE. A
        // profile stores an operating system and an architecture, and the
        // token a route is built from is a function of the two. ⚠ Deriving it
        // again in JavaScript would be a second implementation of a rule this
        // project already owns, which is exactly what LIB-03's Must-not
        // forbids read from the other side.
        entries.push(serde_json::json!({
            "path": path,
            "platform": profile.platform_token().as_str(),
            "profile": value,
        }));
    }
    let newest = profiles
        .iter()
        .map(|(_, p)| p.captured.at.clone())
        .max()
        .map_or(serde_json::Value::Null, serde_json::Value::String);
    let layout: serde_json::Value = serde_json::from_str(index_json)
        .ok()
        .and_then(|v: serde_json::Value| v.get("layout").cloned())
        .unwrap_or(serde_json::Value::Null);

    // ⛔ THE POINTER FILE IS CARRIED, NEVER RECOMPUTED IN THE OTHER LANGUAGE.
    // `latest` means the newest STABLE build and the corpus answers that in
    // corpus/v1/latest.json; the Rust crate reads that file and so does this
    // package. ⚠ A JavaScript half that compared version numbers itself would
    // be a second implementation of one selection rule, which is exactly what
    // LIB-03's Must-not forbids, and the two would agree until the day a
    // pre-release build made them disagree.
    let pointers: serde_json::Value =
        serde_json::from_str(pointers_json).unwrap_or(serde_json::Value::Null);

    let data = serde_json::json!({
        "schema": "b-ids-package/2",
        "release": {
            "identifier": release,
            "layout": layout,
            "profiles": entries.len(),
            "newestCapture": newest,
        },
        "pointers": pointers,
        "entries": entries,
    });
    let data_text = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("serialising the js package data: {e}"))?;

    // ⛔ THE VERSION IS THE LAYOUT AND THE IDENTIFIER'S FIRST TWELVE, never a
    // number somebody increments. A hand-set version is a value that can
    // disagree with the bytes it names, and this one cannot: two builds over
    // one corpus produce one version.
    let version = format!("1.0.0-{}", &release[..12.min(release.len())]);
    let manifest = serde_json::json!({
        "name": "b-ids",
        "version": version,
        "description": "Browser network fingerprints, measured and embedded. \
                        The corpus is embedded at build time and nothing is fetched at runtime.",
        "license": "0BSD",
        "type": "module",
        "main": "./index.mjs",
        "exports": { ".": "./index.mjs" },
        "files": ["index.mjs", "corpus.json", "LICENSE"],
        "sideEffects": false,
    });
    let manifest_text = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("serialising the js package manifest: {e}"))?;

    Ok(vec![
        File {
            path: "packages/js/package.json".to_owned(),
            text: format!("{manifest_text}\n"),
        },
        File {
            path: "packages/js/corpus.json".to_owned(),
            text: format!("{data_text}\n"),
        },
        File {
            path: "packages/js/index.mjs".to_owned(),
            text: JS_INDEX.to_owned(),
        },
    ])
}

/// The JavaScript package's whole interface.
///
/// ⛔ **It follows `LIB-01`'s shape rather than inventing one**, because the
/// entry's Must-not for `LIB-03` is the same rule read from the other side: a
/// binding that exposed a shape the Rust crate does not have is a second
/// interface to keep correct.
const JS_INDEX: &str = r#"// b-ids: browser network fingerprints, measured and embedded.
//
// ⛔ THE CORPUS IS EMBEDDED AND NOTHING IS FETCHED. A package that needed the
// network to answer would fail in the environment its consumers care most
// about. TODO/publish.md, PUB-05.
//
// ⭐ GENERATED FROM THE CORPUS by b_ids_corpus::packages. Do not edit: an edit
// here is a second copy of the corpus that drifts from the first, and the drift
// is invisible because both sides look like data.
//
// ⚠ THE SHAPE IS THE RUST CRATE'S, deliberately. crates/b-ids is the reference
// implementation and this follows it, so a consumer moving between the two is
// not learning a second interface.

import data from './corpus.json' with { type: 'json' };

/// What corpus this build embedded.
///
/// ⭐ `identifier` is the SHA-256 of the corpus index. The index carries a
/// digest of every published file, so it pins the corpus transitively: two
/// builds reporting the same identifier embedded the same bytes.
export function release() {
  return data.release;
}

/// Every embedded profile, in the index's own order.
export function profiles() {
  return data.entries.map((e) => e.profile);
}

/// The published path of each embedded profile, in the same order as profiles().
export function paths() {
  return data.entries.map((e) => e.path);
}

/// The profile at one published path, or undefined.
export function at(path) {
  const found = data.entries.find((e) => e.path === path);
  return found ? found.profile : undefined;
}

/// Every profile matching a browser, a channel and a platform.
///
/// ⚠ An argument left out matches everything, and an argument that matches
/// nothing returns an empty array rather than throwing. ⛔ Nothing here falls
/// back to a neighbouring platform: a missing route is a fact and a substituted
/// value is a lie.
export function select({ browser, channel, platform } = {}) {
  return data.entries
    .filter(
      (e) =>
        (browser === undefined ||
          e.profile.browser.name.toLowerCase() === String(browser).toLowerCase()) &&
        (channel === undefined || e.profile.browser.channel === channel) &&
        // ⛔ THE TOKEN IS READ, NEVER DERIVED HERE. A profile stores an
        // operating system and an architecture; the token a route is built
        // from is a function of the two and this project derives it once, in
        // Rust, at generation time.
        (platform === undefined || e.platform === platform),
    )
    .map((e) => e.profile);
}

/// The newest STABLE build for a browser on a platform, or undefined.
///
/// ⛔ `latest` means stable and nothing else. A consumer following one must
/// never be handed a pre-release build. TODO/corpus.md, CORPUS-03.
export function latestStable(browser, platform) {
  // ⛔ THE POINTER FILE ANSWERS THIS, and nothing here compares version numbers.
  // `latest` means the newest STABLE build, which is the corpus's rule rather
  // than this package's, and the Rust crate reads the same map. ⚠ Recomputing
  // it here would be a second implementation of one selection rule, and the two
  // would agree until the day a pre-release build made them disagree.
  const key = `${String(browser).toLowerCase()}/${String(platform).toLowerCase()}`;
  const path = data.pointers?.latest?.[key];
  return path === undefined ? undefined : at(path);
}

/// The build published at one browser, channel and platform, by the corpus's
/// own pointer rather than by a comparison here.
export function latestForChannel(browser, channel, platform) {
  const key = `${String(browser).toLowerCase()}/${channel}/${String(platform).toLowerCase()}`;
  const path = data.pointers?.per_channel?.[key];
  return path === undefined ? undefined : at(path);
}

/// The raw ClientHello a profile was read from, as lower-case hex, or undefined.
///
/// ⭐ The one artefact that survives every hashing scheme and every parser
/// defect.
export function clientHelloHex(profile) {
  return profile?.raw?.client_hello_hex ?? undefined;
}
"#;
