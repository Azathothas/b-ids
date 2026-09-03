//! The publishable tree, assembled once and shipped two ways.
//!
//! ⭐ **`PUB-01` and `PUB-02` publish the SAME BYTES.** A release archive and a
//! data branch built by two assemblers is two answers to what this project
//! publishes, and the day they differ nobody finds out from either. So there is
//! one assembler here, and both surfaces take what it produced.
//!
//! ⛔ **Nothing here reads a clock.** A build stamped with the time it ran
//! produces a different archive every run, and a consumer then cannot tell a
//! real change from a rebuild. The manifest's `generated_from` is a digest of
//! the corpus, which is what actually decides the content.
//!
//! ⛔ **Nothing here writes to a remote.** This assembles a directory; a
//! workflow tags it, archives it or pushes it. A module that also called an API
//! would be one component with two reasons to fail.
//!
//! ⚠ **What a consumer of the published tree never has to reason about:** the
//! source, any vendored dependency, and the reference corpus. None of it is
//! here, so none of it is in what they downloaded. `TODO/publish.md`, `PUB-02`.

use std::path::Path;

use b_ids_harness::{hex, sha256};
use b_ids_schema::Profile;

use crate::formats::{Format, SUPPORT_MATRIX_FILE, render, support_matrix, verify};
use crate::routes::{MANIFEST_FILE, indexes, manifest as route_manifest, routes};
use crate::store::Store;

/// The manifest's own schema identifier.
pub const MANIFEST_SCHEMA: &str = "corpus-publish/1";

/// The file the manifest is written to.
pub const MANIFEST: &str = "MANIFEST.json";

/// The file the checksums are written to.
///
/// ⚠ **Beside the manifest rather than instead of it.** The manifest is what a
/// program reads; this is what `sha256sum -c` reads, and a consumer with
/// neither a JSON parser nor this project's code still has a way to check what
/// they fetched.
pub const CHECKSUMS: &str = "SHA256SUMS";

/// One file in the published tree.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Artefact {
    /// Where it is, relative to the tree's root, with forward slashes.
    pub path: String,
    /// Its SHA-256, lowercase hex.
    pub sha256: String,
    /// How many bytes it holds.
    pub bytes: usize,
}

/// What one build produced.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Built {
    /// The manifest's schema identifier.
    pub schema: String,
    /// The licence every artefact here is published under.
    pub license: String,
    /// The layout version the corpus paths are under.
    pub layout: String,
    /// ⭐ The digest of the corpus this was built from.
    ///
    /// ⛔ **This is what a build is stamped with instead of a clock.** Two runs
    /// over one corpus produce the same value, and a corpus that moved produces
    /// a different one, which is the only thing a consumer needs to tell a
    /// rebuild from a change.
    pub generated_from: String,
    /// How many profiles the corpus held.
    pub profiles: usize,
    /// Every file, ordered by path.
    pub artefacts: Vec<Artefact>,
}

impl Built {
    /// The total size of everything published.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.artefacts.iter().map(|a| a.bytes).sum()
    }
}

/// Write one file and record it.
fn put(out: &Path, path: &str, body: &[u8], artefacts: &mut Vec<Artefact>) -> Result<(), String> {
    let full = out.join(path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&full, body).map_err(|e| format!("{}: {e}", full.display()))?;
    artefacts.push(Artefact {
        path: path.to_owned(),
        sha256: hex(&sha256(body)),
        bytes: body.len(),
    });
    Ok(())
}

/// Assemble the publishable tree under `out`.
///
/// ⛔ **Deterministic for a given corpus**, which is what `check-release` and
/// `check-data-branch` both assert by building twice and comparing bytes.
///
/// # Errors
///
/// The reason the corpus could not be read, or a generator's own refusal.
pub fn build(root: &str, out: &Path) -> Result<Built, String> {
    let store = Store::at(root);
    if !store.exists() {
        return Err(format!("there is no corpus under {root}"));
    }
    let published = store.profiles()?;
    let profiles: Vec<Profile> = published.iter().map(|(_, p)| p.clone()).collect();

    let mut artefacts: Vec<Artefact> = Vec::new();

    // -- the corpus and its raw sidecars, copied verbatim --------------------
    //
    // ⛔ COPIED, never regenerated. A published profile is immutable and its
    // content address is the file's own bytes; a build that re-serialised one
    // would publish something that is not what the corpus holds.
    for (path, _) in &published {
        let body = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let relative = relative_to(root, path)?;
        put(out, &relative, &body, &mut artefacts)?;
    }
    // ⚠ THE SIDECAR IS DERIVED FROM THE PROFILE'S OWN ROUTE, never walked
    // separately. A walk would publish a `.hex` with no profile beside it, and
    // `b-ids-corpus verify` is what says the two agree.
    for (path, profile) in &published {
        let relative = relative_to(root, path)?;
        let sidecar = relative
            .replacen(
                &format!("{}/", crate::route::CORPUS_DIR),
                &format!("{}/", crate::route::RAW_DIR),
                1,
            )
            .replacen(".json", ".hello.hex", 1);
        let full = Path::new(root).join(&sidecar);
        if !full.is_file() {
            return Err(format!("{} has no raw sidecar at {sidecar}", profile.id));
        }
        let body = std::fs::read(&full).map_err(|e| format!("{}: {e}", full.display()))?;
        put(out, &sidecar, &body, &mut artefacts)?;
    }
    for name in ["index.json", "latest.json"] {
        let path = Path::new(root).join("corpus").join("v1").join(name);
        if path.is_file() {
            let body = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            put(out, &format!("corpus/v1/{name}"), &body, &mut artefacts)?;
        }
    }

    // -- every generated format, plus the support matrix ---------------------
    for format in Format::all() {
        let text = render(format, &profiles)?;
        // ⛔ READ BACK BEFORE IT IS PUBLISHED, for the reason the generator's
        // own command does it: a format correct over a fixture and wrong over
        // this corpus would otherwise ship.
        verify(format, &profiles, &text)?;
        put(
            out,
            &format!("formats/{}", format.file_name()),
            text.as_bytes(),
            &mut artefacts,
        )?;
    }
    put(
        out,
        &format!("formats/{SUPPORT_MATRIX_FILE}"),
        support_matrix().as_bytes(),
        &mut artefacts,
    )?;

    // -- the flat routes -----------------------------------------------------
    let with_paths: Vec<(String, Profile)> = published
        .iter()
        .map(|(path, profile)| {
            (
                path.to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches("./")
                    .to_owned(),
                profile.clone(),
            )
        })
        .collect();
    let generated = routes(&with_paths);
    for route in &generated {
        let body = if route.multi_value {
            format!("{}\n", route.value)
        } else {
            route.value.clone()
        };
        put(
            out,
            &format!("routes/{}", route.path),
            body.as_bytes(),
            &mut artefacts,
        )?;
    }
    for (path, body) in indexes(&generated) {
        put(
            out,
            &format!("routes/{path}"),
            body.as_bytes(),
            &mut artefacts,
        )?;
    }
    let route_manifest = serde_json::to_string_pretty(&route_manifest(generated))
        .map_err(|e| format!("serialising the route manifest: {e}"))?;
    put(
        out,
        &format!("routes/{MANIFEST_FILE}"),
        format!("{route_manifest}\n").as_bytes(),
        &mut artefacts,
    )?;

    // -- every build's trust-anchor list -------------------------------------
    //
    // ⛔ A MALFORMED BODY IS A REFUSAL, never a skip, which is the rule the
    // `anchors` command already holds. A profile that does not carry the
    // extension is the ordinary case and produces no file.
    for profile in &profiles {
        match crate::trust::anchor_list(profile) {
            Ok(list) => {
                let text = serde_json::to_string_pretty(&list)
                    .map_err(|e| format!("serialising {}: {e}", list.profile_id))?;
                put(
                    out,
                    &format!(
                        "anchors/{}-{}-{}.json",
                        list.browser.to_ascii_lowercase(),
                        list.version,
                        list.platform
                    ),
                    format!("{text}\n").as_bytes(),
                    &mut artefacts,
                )?;
            }
            Err(crate::trust::NotAList::Absent) => {}
            Err(why) => return Err(format!("{}: {why}", profile.id)),
        }
    }

    // -- the licence, so the tree says what it is ----------------------------
    //
    // ⛔ COPIED FROM THE REPOSITORY'S OWN FILE. A build that wrote its own text
    // would be a second copy of a legal document. `TODO/publish.md`, `PUB-07`.
    let license = Path::new(root).join("LICENSE");
    let license_body =
        std::fs::read(&license).map_err(|e| format!("{}: {e}", license.display()))?;
    put(out, "LICENSE", &license_body, &mut artefacts)?;

    // ⛔ ORDERED BY PATH, so two builds on two filesystems produce one manifest.
    artefacts.sort_by(|a, b| a.path.cmp(&b.path));

    // ⭐ THE CORPUS'S OWN DIGEST, not a clock. Taken over the profile artefacts
    // alone, so a change to a generator moves the artefacts and not this.
    let corpus_digest = {
        let mut joined = String::new();
        for artefact in artefacts
            .iter()
            .filter(|a| a.path.starts_with("corpus/") || a.path.starts_with("raw/"))
        {
            joined.push_str(&artefact.path);
            joined.push(' ');
            joined.push_str(&artefact.sha256);
            joined.push('\n');
        }
        hex(&sha256(joined.as_bytes()))
    };

    let built = Built {
        schema: MANIFEST_SCHEMA.to_owned(),
        license: b_ids_schema::LICENSE.to_owned(),
        layout: crate::route::LAYOUT.to_owned(),
        generated_from: corpus_digest,
        profiles: profiles.len(),
        artefacts: artefacts.clone(),
    };

    // ⚠ The manifest and the checksums are NOT in the artefact list, because a
    // file cannot carry its own digest. A consumer checks everything else
    // against them.
    let manifest_text = serde_json::to_string_pretty(&built)
        .map_err(|e| format!("serialising the manifest: {e}"))?;
    std::fs::write(out.join(MANIFEST), format!("{manifest_text}\n").as_bytes())
        .map_err(|e| format!("{MANIFEST}: {e}"))?;

    let mut sums = String::new();
    for artefact in &built.artefacts {
        // ⚠ The two-space form `sha256sum -c` reads, with the digest first.
        sums.push_str(&format!("{}  {}\n", artefact.sha256, artefact.path));
    }
    std::fs::write(out.join(CHECKSUMS), sums.as_bytes())
        .map_err(|e| format!("{CHECKSUMS}: {e}"))?;

    Ok(built)
}

/// A published file's path, relative to the corpus root, with forward slashes.
fn relative_to(root: &str, path: &Path) -> Result<String, String> {
    let root = Path::new(root);
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is not under {}", path.display(), root.display()))?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

/// The release tag one build gets.
///
/// ⭐ **The schema major, the date, and a counter.** The first component says
/// which schema a consumer is getting without opening anything, the date says
/// how fresh the data is, the counter allows more than one release a day, and
/// the whole thing sorts correctly and is a valid git tag.
///
/// ⛔ **The date is the CALLER'S, never a clock read here.** A tag built from
/// this process's own clock would make the function untestable and the build
/// non-reproducible.
#[must_use]
pub fn tag(layout: &str, date: &str, counter: u32) -> String {
    format!("{layout}.{}.{counter}", date.replace('-', "."))
}

/// Why a release cannot be cut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotReleasable {
    /// The tag already exists.
    ///
    /// ⛔ **A published release is immutable.** Never re-upload an asset: cut a
    /// new release that supersedes it and say so in the body. Consumers pin
    /// releases, and a mutated asset breaks them silently.
    TagExists {
        /// The tag that would have been overwritten.
        tag: String,
    },
    /// The date is not the `YYYY-MM-DD` a tag is built from.
    BadDate {
        /// What was passed.
        date: String,
    },
    /// The build produced nothing, so there is nothing to release.
    Empty,
}

impl core::fmt::Display for NotReleasable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TagExists { tag } => write!(
                f,
                "{tag} already exists. A published release is immutable: cut a new one that \
                 supersedes it rather than re-uploading an asset"
            ),
            Self::BadDate { date } => write!(f, "{date} is not a YYYY-MM-DD date"),
            Self::Empty => {
                f.write_str("the build produced no artefact, so there is nothing to release")
            }
        }
    }
}

/// The tag a release would take, refusing one that already exists.
///
/// ⛔ **The existing tags are the CALLER'S**, read from the repository by
/// whoever asks. A function that shelled out to git would be untestable and
/// would tie a naming rule to a working directory.
///
/// # Errors
///
/// [`NotReleasable`], naming the tag, the date or the empty build.
pub fn plan_release(
    built: &Built,
    date: &str,
    counter: u32,
    existing: &[String],
) -> Result<String, NotReleasable> {
    if built.artefacts.is_empty() {
        return Err(NotReleasable::Empty);
    }
    let parts: Vec<&str> = date.split('-').collect();
    let shaped = parts.len() == 3
        && parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit()));
    if !shaped {
        return Err(NotReleasable::BadDate {
            date: date.to_owned(),
        });
    }
    let tag = tag(&built.layout, date, counter);
    if existing.contains(&tag) {
        return Err(NotReleasable::TagExists { tag });
    }
    Ok(tag)
}

/// The moving tags a script follows instead of listing releases.
///
/// ⭐ **One per schema major and one overall**, so a fetch needs no API call.
/// ⚠ These MOVE, which is exactly why the dated tag exists beside them: a
/// reproducible build pins the dated one.
#[must_use]
pub fn moving_tags(built: &Built) -> Vec<String> {
    vec![built.layout.clone(), "latest".to_owned()]
}

/// Whether a push to the data branch would rewrite history.
///
/// ⛔ **The data branch is APPEND-ONLY and never force-pushed.** A consumer
/// pinning a commit on it keeps working forever, and that property is free
/// right up until somebody rewrites the branch. `TODO/publish.md`, `PUB-02`.
///
/// ⚠ **`head` is what the remote holds and `parent` is what the new commit was
/// built on.** They agree on an append; they differ when the new commit was
/// built on something the branch has moved past, which is the case a force
/// push papers over.
#[must_use]
pub fn would_rewrite(head: Option<&str>, parent: Option<&str>) -> bool {
    match (head, parent) {
        // ⚠ A branch that does not exist yet cannot be rewritten. The first
        // push creates it and has no parent.
        (None, _) => false,
        // ⛔ A branch that exists and a commit built on nothing IS a rewrite:
        // an orphan commit pushed over a branch discards every commit on it.
        (Some(_), None) => true,
        (Some(head), Some(parent)) => head != parent,
    }
}
