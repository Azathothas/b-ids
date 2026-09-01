//! The corpus on disk: append-only, never edited in place, with a content
//! address beside every published file.
//!
//! ⛔ **A profile that exists is never overwritten.** A correction is a NEW
//! profile naming the one it replaces, so a consumer who pinned a value can
//! always tell whether it changed and a reader can always see what it used to
//! say. A store that let a write land on an existing path would make the
//! history a matter of whether anybody happened to commit in between.
//!
//! ⭐ **The index is DERIVED from the tree, never appended to.** An index that
//! is written incrementally is a second copy of what the tree already says, and
//! the copy a reader trusts is the wrong one. [`index`] walks the corpus and
//! produces the index; [`write_index`] writes what it produced; [`verify`]
//! re-derives it and compares. Nothing else may write it.
//!
//! ⚠ **The content address is what makes a version number a pin.** Two
//! published copies of one dataset, both carrying the same version and both
//! naming the same upstream, were measured holding a different number of
//! entries. `docs/reference-sweeps/usable.md` section 9 has it. A version that
//! does not pin its bytes pins nothing.
//!
//! `TODO/corpus.md`, `CORPUS-01`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use b_ids_harness::{hex, sha256};
use b_ids_schema::Profile;
use serde::{Deserialize, Serialize};

use crate::route::{CORPUS_DIR, LAYOUT, RAW_DIR, Route, as_route, route, version_order};

/// The schema identifier the index carries.
pub const INDEX_SCHEMA: &str = "corpus-index/1";

/// The schema identifier the pointer file carries.
///
/// ⚠ Version 2 splits `latest`, which is stable only, from `per_channel`.
/// Version 1 had one map keyed by channel and left what `latest` means
/// undecided. A version is part of the data rather than implied by the reader.
pub const POINTER_SCHEMA: &str = "corpus-latest/2";

/// The index file's name, under the layout directory.
pub const INDEX_FILE: &str = "index.json";

/// The pointer file's name, under the layout directory.
pub const POINTER_FILE: &str = "latest.json";

/// The channel a `latest` pointer is allowed to name.
///
/// ⛔ **One value, and it is not configurable.** A ceiling anybody can raise
/// from a flag is a ceiling that gets raised instead of met. `CORPUS-03`.
pub const STABLE: &str = "stable";

/// How deep a profile sits under the layout directory: browser, channel,
/// platform, then the file.
///
/// ⛔ **Checked rather than assumed.** A file at any other depth is a layout
/// violation reported by name, which is what stops the walk quietly treating a
/// stray file as a profile or a profile as a stray file.
const PROFILE_DEPTH: usize = 4;

/// One published file and the digest that pins its bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Published {
    /// The route, with forward slashes on every host.
    pub path: String,
    /// The SHA-256 of the file's bytes, hex-encoded.
    pub sha256: String,
    /// How many bytes the file holds.
    pub bytes: usize,
}

/// One profile, as the index lists it.
///
/// ⚠ **Every field here is derived from the profile itself.** The index states
/// nothing the profiles do not already say, so re-deriving it is always
/// possible and a disagreement is always the index's defect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    /// The profile identifier.
    pub id: String,
    /// The browser's name, as its vendor spells it.
    pub browser: String,
    /// The exact build.
    pub version: String,
    /// The release channel.
    pub channel: String,
    /// The platform token.
    pub platform: String,
    /// When the capture was taken.
    pub captured_at: String,
    /// How the subject came to trust the harness that measured it.
    ///
    /// ⭐ In the index rather than only in the profile, because it is the field
    /// a consumer filters on before fetching anything: a profile taken under a
    /// configuration they do not accept is one they should not have to download
    /// to reject.
    pub trust: String,
    /// The profile this one replaces, where it replaces one.
    pub supersedes: Option<String>,
    /// Whether any field of it was copied from somebody else's table.
    ///
    /// ⛔ A profile with any is a draft, whatever else is true of it.
    pub draft: bool,
    /// The profile file.
    pub profile: Published,
    /// The raw `ClientHello` beside it.
    pub hello: Published,
}

/// Every profile in the corpus, with its content address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    /// The schema this index is written against.
    pub schema: String,
    /// The layout version every path in it is under.
    pub layout: String,
    /// The profiles, ordered by route.
    ///
    /// ⛔ **Ordered, and by the route rather than by insertion.** An index whose
    /// order depends on the order files were walked is an index that produces a
    /// diff on a machine with a different filesystem, and a diff nobody can
    /// read is a diff nobody reviews.
    pub profiles: Vec<IndexEntry>,
}

/// The pointers a consumer follows instead of listing the corpus.
///
/// ⛔ **`latest` MEANS STABLE AND NOTHING ELSE**, and it is a separate map from
/// the per-channel one for exactly that reason. A consumer following a pointer
/// called `latest` must never be handed a pre-release build; that is the same
/// failure as shipping a version nobody runs yet.
///
/// ⭐ **The rule is enforced by CONSTRUCTION rather than by a check.**
/// [`Store::pointers`] builds `latest` from stable profiles alone, so a
/// non-stable route cannot be in it, and [`Store::verify`] compares the written
/// file against the derived one so a hand-edited pointer file is refused. A
/// class of defect that cannot be represented is stronger than one that is
/// tested for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pointers {
    /// The schema this file is written against.
    pub schema: String,
    /// The layout version every path in it is under.
    pub layout: String,
    /// `browser/platform` to the route of its newest STABLE build.
    ///
    /// ⛔ Stable only. `CORPUS-03`.
    pub latest: BTreeMap<String, String>,
    /// `browser/channel/platform` to the route of that channel's newest build.
    ///
    /// ⭐ **Beta and canary are published beside `latest`, in their own paths,
    /// clearly labelled**, because capturing them is how this project gets
    /// ahead of a release rather than staying perpetually behind it: the
    /// profile for the next stable is ready the day it ships, having been
    /// captured weeks earlier under another name.
    pub per_channel: BTreeMap<String, String>,
}

/// The corpus, rooted at a directory.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

/// What one accepted write placed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Added {
    /// The profile file.
    pub profile: PathBuf,
    /// The raw `ClientHello` beside it.
    pub hello: PathBuf,
}

impl Store {
    /// A store rooted at `root`, which is the repository root in this project.
    #[must_use]
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The directory this store is rooted at.
    ///
    /// ⚠ **For turning an absolute path back into a route**, which is what a
    /// message a person on another machine can act on needs.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The directory the profiles live under.
    #[must_use]
    pub fn corpus_dir(&self) -> PathBuf {
        self.root.join(CORPUS_DIR).join(LAYOUT)
    }

    /// The directory the raw bytes live under.
    #[must_use]
    pub fn raw_dir(&self) -> PathBuf {
        self.root.join(RAW_DIR).join(LAYOUT)
    }

    /// Whether this store has been created at all.
    ///
    /// ⚠ **An absent corpus is a different fact from an empty one**, and a
    /// caller that could not tell them apart would report a tree with no corpus
    /// as a corpus with nothing wrong in it.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.corpus_dir().is_dir()
    }

    /// Add a profile, refusing to overwrite one that is already published.
    ///
    /// # Errors
    ///
    /// A sentence naming what was refused: a malformed profile, a route its
    /// keys cannot produce, a path already taken, a `supersedes` naming nothing
    /// in the store, or whatever the filesystem said.
    pub fn add(&self, profile: &Profile) -> Result<Added, String> {
        let defects = crate::capture::defects(profile);
        if !defects.is_empty() {
            let listed = defects
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "the profile is malformed and a malformed profile is never published: {listed}"
            ));
        }

        let Route {
            profile: rel,
            hello,
        } = route(profile).map_err(|e| e.to_string())?;
        let profile_path = self.root.join(&rel);
        let hello_path = self.root.join(&hello);

        // ⛔ APPEND-ONLY, and this is where that is enforced rather than
        // remembered. A correction is a new profile carrying `supersedes`.
        if profile_path.exists() {
            return Err(format!(
                "{} is already published, and a published profile is never edited. A correction \
                 is a NEW profile at a new version naming this one in `supersedes`",
                as_route(&rel)
            ));
        }

        // ⚠ A `supersedes` naming nothing is a dangling correction: the reader
        // is told this profile replaces something and cannot find out what.
        if let Some(replaced) = &profile.supersedes {
            let known = self
                .index()?
                .profiles
                .iter()
                .any(|entry| &entry.id == replaced);
            if !known {
                return Err(format!(
                    "supersedes names {replaced}, which is not in this corpus. A correction names \
                     the profile it replaces, and one nobody can find is a correction to nothing"
                ));
            }
        }

        let hello_hex = profile.raw.client_hello_hex.as_deref().ok_or_else(|| {
            "the profile carries no raw.client_hello_hex, so there are no bytes to publish beside \
             it. The raw block is the backstop against this project's own parser being wrong"
                .to_owned()
        })?;

        for path in [&profile_path, &hello_path] {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: {e}", parent.display()))?;
            }
        }

        let text = serde_json::to_string_pretty(profile)
            .map_err(|e| format!("serialising the profile: {e}"))?;
        std::fs::write(&profile_path, format!("{text}\n"))
            .map_err(|e| format!("{}: {e}", profile_path.display()))?;
        // ⛔ NO TRAILING NEWLINE. This file carries exactly one value, and a
        // measured defect in a published dataset is that a consumer of a
        // single-value route has to strip one.
        // `docs/reference-sweeps/usable.md` section 9.
        std::fs::write(&hello_path, hello_hex)
            .map_err(|e| format!("{}: {e}", hello_path.display()))?;

        Ok(Added {
            profile: profile_path,
            hello: hello_path,
        })
    }

    /// Every profile file in the corpus, in route order.
    ///
    /// # Errors
    ///
    /// Whatever the filesystem said, or a file at a depth the layout does not
    /// have.
    pub fn profile_paths(&self) -> Result<Vec<PathBuf>, String> {
        let dir = self.corpus_dir();
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut found = Vec::new();
        walk(&dir, &mut found)?;
        found.sort();

        let mut profiles = Vec::new();
        for path in found {
            let rel = path
                .strip_prefix(&dir)
                .map_err(|e| format!("{}: {e}", path.display()))?;
            let depth = rel.components().count();
            let name = rel
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_owned();
            if depth == 1 && (name == INDEX_FILE || name == POINTER_FILE) {
                continue;
            }
            if depth != PROFILE_DEPTH {
                return Err(format!(
                    "{}: a profile sits at {CORPUS_DIR}/{LAYOUT}/BROWSER/CHANNEL/PLATFORM/\
                     VERSION.json, which is {PROFILE_DEPTH} components under the layout \
                     directory, and this file is at {depth}",
                    as_route(&path)
                ));
            }
            profiles.push(path);
        }
        Ok(profiles)
    }

    /// Read every profile in the corpus, in route order.
    ///
    /// # Errors
    ///
    /// The first file that could not be read or does not parse as a profile.
    pub fn profiles(&self) -> Result<Vec<(PathBuf, Profile)>, String> {
        let mut out = Vec::new();
        for path in self.profile_paths()? {
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let profile: Profile = serde_json::from_str(&text)
                .map_err(|e| format!("{}: not a profile: {e}", as_route(&path)))?;
            out.push((path, profile));
        }
        Ok(out)
    }

    /// The index this corpus derives to.
    ///
    /// # Errors
    ///
    /// Whatever reading the corpus said, or a profile whose keys produce no
    /// route.
    pub fn index(&self) -> Result<Index, String> {
        let mut profiles = Vec::new();
        for (path, profile) in self.profiles()? {
            let Route {
                profile: rel,
                hello,
            } = route(&profile).map_err(|e| e.to_string())?;
            let hello_path = self.root.join(&hello);
            profiles.push(IndexEntry {
                id: profile.id.to_string(),
                browser: profile.browser.name.clone(),
                version: profile.browser.version.clone(),
                channel: profile.browser.channel.to_string(),
                platform: profile.platform_token().to_string(),
                captured_at: profile.captured.at.clone(),
                trust: profile.captured.trust.to_string(),
                supersedes: profile.supersedes.clone(),
                draft: profile.is_draft(),
                profile: published(&path, &rel)?,
                hello: published(&hello_path, &hello)?,
            });
        }
        profiles.sort_by(|a, b| a.profile.path.cmp(&b.profile.path));
        Ok(Index {
            schema: INDEX_SCHEMA.to_owned(),
            layout: LAYOUT.to_owned(),
            profiles,
        })
    }

    /// The newest profile for each browser, channel and platform.
    ///
    /// # Errors
    ///
    /// Whatever deriving the index said.
    pub fn pointers(&self) -> Result<Pointers, String> {
        // ⚠ Component-wise numeric, never lexicographic: `152.0.7977.9` sorts
        // after `152.0.7977.64` as text, and a pointer built that way hands a
        // consumer an older build while looking correct.
        let keep = |map: &mut BTreeMap<String, (Vec<u64>, String, String)>,
                    key: String,
                    version: &str,
                    path: &str| {
            let order = version_order(version);
            let candidate = (order.0, order.1, path.to_owned());
            match map.get(&key) {
                Some(current) if (&current.0, &current.1) >= (&candidate.0, &candidate.1) => {}
                _ => {
                    map.insert(key, candidate);
                }
            }
        };

        let mut per_channel: BTreeMap<String, (Vec<u64>, String, String)> = BTreeMap::new();
        let mut latest: BTreeMap<String, (Vec<u64>, String, String)> = BTreeMap::new();
        for entry in self.index()?.profiles {
            let browser = entry.browser.to_ascii_lowercase();
            keep(
                &mut per_channel,
                format!("{browser}/{}/{}", entry.channel, entry.platform),
                &entry.version,
                &entry.profile.path,
            );
            // ⛔ STABLE ONLY, and this is the whole of `CORPUS-03`. A pointer
            // called `latest` that could hold a beta build would hand a
            // consumer a pre-release, and no check downstream can undo that
            // because the consumer has already fetched it.
            if entry.channel == STABLE {
                keep(
                    &mut latest,
                    format!("{browser}/{}", entry.platform),
                    &entry.version,
                    &entry.profile.path,
                );
            }
        }
        let flatten = |m: BTreeMap<String, (Vec<u64>, String, String)>| {
            m.into_iter().map(|(k, v)| (k, v.2)).collect()
        };
        Ok(Pointers {
            schema: POINTER_SCHEMA.to_owned(),
            layout: LAYOUT.to_owned(),
            latest: flatten(latest),
            per_channel: flatten(per_channel),
        })
    }

    /// Every `latest` pointer that does not resolve to a stable profile.
    ///
    /// ⭐ **Empty by construction from [`Store::pointers`], and read back
    /// anyway.** The derivation cannot produce a non-stable entry; this reads
    /// the pointer file that is actually on disk, which is what a consumer
    /// follows, and a hand-edited one is exactly what this refuses.
    ///
    /// # Errors
    ///
    /// Whatever reading the corpus or the pointer file said.
    pub fn latest_that_is_not_stable(&self) -> Result<Vec<String>, String> {
        let path = self.corpus_dir().join(POINTER_FILE);
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{POINTER_FILE}: {e}"))?;
        let pointers: Pointers = serde_json::from_str(&text)
            .map_err(|e| format!("{POINTER_FILE}: not a pointer file: {e}"))?;
        let by_route: BTreeMap<String, String> = self
            .profiles()?
            .into_iter()
            .filter_map(|(_, profile)| {
                let found = route(&profile).ok()?;
                Some((
                    as_route(&found.profile),
                    profile.browser.channel.to_string(),
                ))
            })
            .collect();

        let mut problems = Vec::new();
        for (key, at) in &pointers.latest {
            match by_route.get(at) {
                Some(channel) if channel == STABLE => {}
                Some(channel) => problems.push(format!(
                    "latest/{key} resolves to {at}, whose channel is {channel}. A pointer called \
                     latest means stable and nothing else"
                )),
                None => problems.push(format!(
                    "latest/{key} resolves to {at}, which is not a profile in this corpus"
                )),
            }
        }
        Ok(problems)
    }

    /// Write the index and the pointer file from what the tree says.
    ///
    /// ⛔ **The only writer of either.** Anything else that wrote one would be a
    /// second answer to what the corpus contains.
    ///
    /// # Errors
    ///
    /// Whatever deriving them or the filesystem said.
    pub fn write_index(&self) -> Result<(PathBuf, PathBuf), String> {
        let dir = self.corpus_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let index_path = dir.join(INDEX_FILE);
        let pointer_path = dir.join(POINTER_FILE);
        write_json(&index_path, &self.index()?)?;
        write_json(&pointer_path, &self.pointers()?)?;
        Ok((index_path, pointer_path))
    }

    /// Every way this corpus disagrees with itself.
    ///
    /// ⭐ **This is the acceptance in one function.** An empty result means
    /// every profile validates, every one sits at the route its own keys
    /// derive, every `supersedes` names a profile that exists, every raw
    /// sidecar holds the bytes its profile says it holds, and the index and the
    /// pointer file are what the tree derives to.
    ///
    /// ⚠ It says nothing about the HISTORY. Whether a published profile was
    /// ever modified after its first commit is a question for git, and
    /// `scripts/common/check-corpus.sh` is what asks it.
    ///
    /// # Errors
    ///
    /// A reading that could not happen at all, as distinct from a corpus with
    /// something wrong in it.
    pub fn verify(&self) -> Result<Vec<String>, String> {
        let mut problems = Vec::new();
        let entries = self.profiles()?;
        let known: Vec<String> = entries.iter().map(|(_, p)| p.id.to_string()).collect();

        for (path, profile) in &entries {
            // ⚠ The route this file is published at, never the absolute path.
            // A message naming a path on the machine that ran the check is a
            // message nobody else can act on, and on Windows an absolute path
            // carries a drive prefix and a root component that no route has.
            let at = as_route(path.strip_prefix(&self.root).unwrap_or(path));
            for defect in crate::capture::defects(profile) {
                problems.push(format!("{at}: malformed: {defect}"));
            }
            match route(profile) {
                Ok(expected) => {
                    let want = self.root.join(&expected.profile);
                    if &want != path {
                        problems.push(format!(
                            "{at}: its own keys derive the route {}, so it is published under a \
                             name that is not its name",
                            as_route(&expected.profile)
                        ));
                    }
                    let hello_path = self.root.join(&expected.hello);
                    match std::fs::read_to_string(&hello_path) {
                        Ok(text) => {
                            let recorded = profile.raw.client_hello_hex.as_deref().unwrap_or("");
                            if text != recorded {
                                problems.push(format!(
                                    "{}: does not hold what {at} says raw.client_hello_hex is. A \
                                     value in two places needs a check that they agree",
                                    as_route(&expected.hello)
                                ));
                            }
                            // ⚠ THE TRAILING-NEWLINE RULE IS NOT CHECKED HERE,
                            // and that is deliberate. `scripts/common/
                            // check-routes` owns it, over every published route
                            // file rather than only over a sidecar that has a
                            // profile beside it, and two checks holding one
                            // rule is two places for it to be wrong. What this
                            // owns is the question that needs the profile:
                            // whether the file and the field agree.
                        }
                        Err(err) => problems.push(format!(
                            "{}: {err}. Every profile publishes its bytes beside it",
                            as_route(&expected.hello)
                        )),
                    }
                }
                Err(why) => problems.push(format!("{at}: no route: {why}")),
            }
            if let Some(replaced) = &profile.supersedes
                && !known.contains(replaced)
            {
                problems.push(format!(
                    "{at}: supersedes names {replaced}, which is not in this corpus"
                ));
            }
            problems.extend(
                rebuildable(profile)
                    .into_iter()
                    .map(|why| format!("{at}: {why}")),
            );
        }

        // ⛔ The index is compared against what the tree derives to, never
        // trusted. An index nobody re-derives is prose.
        let dir = self.corpus_dir();
        problems.extend(compare_json(
            &dir.join(INDEX_FILE),
            &self.index()?,
            INDEX_FILE,
        )?);
        problems.extend(compare_json(
            &dir.join(POINTER_FILE),
            &self.pointers()?,
            POINTER_FILE,
        )?);
        Ok(problems)
    }
}

/// Every measured half a profile's own raw block does not reproduce.
///
/// ⭐ **This is what makes the raw block a backstop rather than a gesture.** A
/// capture is a moment that cannot be retaken, and the reason to keep the bytes
/// is that this project's own parser will one day turn out to be wrong. A raw
/// block nobody has re-parsed is a claim.
///
/// ⚠ **A second ENTRY into the parser, not a second parser.** What this answers
/// is the one question the raw block exists for: are the stored bytes enough to
/// produce the model again.
///
/// ⚠ **Only the halves the raw block carries.** A terminated HTTP/2 capture has
/// no cleartext request, so an HTTP half rebuilt from `connection_hex` is
/// absent rather than wrong, and reporting that as a failure would refuse every
/// profile a browser can produce.
fn rebuildable(profile: &b_ids_schema::Profile) -> Vec<String> {
    // ⚠ The policy is names-only because no half this compares carries a header
    // VALUE: the HTTP/2 half holds frames, the pseudo-header order and the
    // window, and the values live in the HTTP half, which is not compared here.
    let rebuilt = b_ids_harness::rebuild(&profile.raw, b_ids_schema::http::ValuePolicy::NamesOnly);
    let mut out = Vec::new();
    match &rebuilt.tls {
        Some(tls) if *tls == profile.tls => {}
        Some(_) => out.push(
            "raw.client_hello_hex re-parses to a TLS half that is not the recorded one. The bytes \
             are the backstop and they disagree with what was published from them"
                .to_owned(),
        ),
        None => out.push(
            "raw.client_hello_hex carries no readable ClientHello, so nothing could check the TLS \
             half against the bytes it came from"
                .to_owned(),
        ),
    }
    match &rebuilt.http2 {
        Some(http2) if *http2 == profile.http2 => {}
        Some(_) => out.push(
            "raw.http2_frames_hex re-parses to an HTTP/2 half that is not the recorded one"
                .to_owned(),
        ),
        None => out.push("raw.http2_frames_hex carries no readable frames".to_owned()),
    }
    out
}

/// One published file's route, digest and size.
fn published(path: &Path, rel: &Path) -> Result<Published, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Published {
        path: as_route(rel),
        sha256: hex(&sha256(&bytes)),
        bytes: bytes.len(),
    })
}

/// Write a value as pretty JSON with a trailing newline.
fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| format!("serialising: {e}"))?;
    std::fs::write(path, format!("{text}\n")).map_err(|e| format!("{}: {e}", path.display()))
}

/// Compare a written JSON file against what the tree derives to.
fn compare_json<T: Serialize>(path: &Path, derived: &T, name: &str) -> Result<Vec<String>, String> {
    let want = serde_json::to_string_pretty(derived).map_err(|e| format!("serialising: {e}"))?;
    match std::fs::read_to_string(path) {
        Ok(text) if text.trim_end() == want.trim_end() => Ok(Vec::new()),
        Ok(_) => Ok(vec![format!(
            "{name}: does not match what the corpus derives to. Rewrite it with `b-ids-corpus \
             index --write` rather than by hand"
        )]),
        Err(err) => Ok(vec![format!(
            "{name}: {err}. The corpus publishes an index and a latest-per-key pointer"
        )]),
    }
}

/// Every file under `dir`, recursively.
///
/// ⚠ **Sorted by the caller, not by the filesystem.** Directory order is not
/// guaranteed and differs between hosts, so an index built in walk order would
/// produce a diff on a different machine with nothing having changed.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}
