//! Where the corpus is, for code that has to go to disk for it.
//!
//! ⛔ **ONE RESOLVER, NOT SIX.** `corpus/`, `raw/` and `vectors/` left the
//! default branch in `PUB-13`, and on that day every place that walked up from
//! its own manifest looking for `corpus/v1/index.json` stopped finding one.
//! Measured rather than predicted: with the directories removed, the workspace
//! suite failed in four separate files, each with its own private copy of the
//! same walk.
//!
//! ⭐ **The order is the one `scripts/common/corpus-root.sh` answers with**, so
//! a check that resolves a root and exports [`ROOT_ENV`] gets the same answer
//! here, in `b-ids/build.rs`, and in every check that asks the shell resolver.
//!
//! ⚠ **It does NOT know about branches, and that is deliberate.** Materialising
//! a git ref is the shell resolver's job, because it needs `git` and this crate
//! has no process-spawning code at all. What this reads is an explicit root or
//! an ancestor that already holds one, which is exactly the seam the shell side
//! fills in. `AGENTS.md` section 5 is the rule that a general tool used
//! where a purpose-built one exists gives a plausible wrong answer.

use std::path::{Path, PathBuf};

/// The environment variable that names a corpus root.
///
/// ⭐ **The seam `PUB-11` created and `PUB-13` made load-bearing.** Every check
/// exports it before running `cargo`, so the suite and the crate's own build
/// script read one corpus rather than two.
pub const ROOT_ENV: &str = "B_IDS_CORPUS_ROOT";

/// The marker that says a directory holds a corpus.
const MARK: [&str; 3] = ["corpus", "v1", "index.json"];

/// Whether `dir` holds a corpus.
///
/// ⭐ **The one test for it**, so no caller answers it a second way.
#[must_use]
pub fn holds_corpus(dir: &Path) -> bool {
    let mut path = dir.to_path_buf();
    for part in MARK {
        path.push(part);
    }
    path.is_file()
}

/// Resolve the corpus root, starting the walk at `from`.
///
/// ⛔ **An explicit root is never second guessed.** If [`ROOT_ENV`] is set, that
/// is the answer whether or not it holds a corpus: a caller that named a root
/// and got a different one silently would report on a corpus it did not choose.
/// Ask [`holds_corpus`] about the result when that matters.
///
/// Returns `None` when nothing is set and no ancestor of `from` holds a corpus.
/// ⚠ That is the ORDINARY state of a fresh checkout of the default branch, not
/// an error condition: the corpus lives on the source branch and
/// `scripts/common/corpus-root.sh` is what materialises it.
#[must_use]
pub fn corpus_root_from(from: &Path) -> Option<PathBuf> {
    if let Some(named) = std::env::var_os(ROOT_ENV) {
        return Some(PathBuf::from(named));
    }
    let mut here = from;
    loop {
        if holds_corpus(here) {
            return Some(here.to_path_buf());
        }
        here = here.parent()?;
    }
}

/// Resolve the corpus root, or explain what to do about it.
///
/// ⛔ **The message names the command**, because since `PUB-13` a tree with no
/// corpus in it is the ordinary state rather than a mistake, and a panic that
/// only says "not found" sends a reader looking for a deleted directory.
///
/// # Panics
///
/// When no root is set and no ancestor of `from` holds a corpus.
#[must_use]
pub fn corpus_root_or_explain(from: &Path) -> PathBuf {
    corpus_root_from(from).unwrap_or_else(|| {
        panic!(
            "no corpus above {}. corpus/ lives on the source branch since PUB-13: \
             set {ROOT_ENV}=$(sh scripts/common/corpus-root.sh), or run the gate, \
             which does it for you.",
            from.display()
        )
    })
}
