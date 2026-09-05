// b-ids: browser network fingerprints, measured and embedded.
//
// ⛔ THE CORPUS IS EMBEDDED AND NOTHING IS FETCHED. A package that needed the
// network to answer would fail in the environment its consumers care most
// about. docs/history/todo/publish.md, PUB-05.
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
/// never be handed a pre-release build. docs/history/todo/corpus.md, CORPUS-03.
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
