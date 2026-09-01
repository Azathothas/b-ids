//! The corpus: where a capture becomes a profile, and where a profile lives.
//!
//! ⭐ **Content-addressed, append-only, never edited in place.** Those three
//! words are the entry's title and each one is enforced by something here
//! rather than asked for in a document:
//!
//! - **content-addressed**: the index carries the SHA-256 of every published
//!   file, and [`store::Store::verify`] recomputes them. A version number that
//!   does not pin its bytes pins nothing, which was measured in somebody else's
//!   published dataset.
//! - **append-only**: [`store::Store::add`] refuses a path that already exists.
//!   A correction is a new profile carrying `supersedes`.
//! - **never edited in place**: that is a question about the git history rather
//!   than about the working tree, and `scripts/common/check-corpus.sh` is what
//!   asks it.
//!
//! # ⛔ What this crate refuses to do
//!
//! It does not repair. A capture that carries a credential, a profile whose
//! identifier disagrees with its keys, a route a profile's own keys cannot
//! produce: each is refused by name, and the operator decides. A corpus whose
//! entries look measured and are not is the one failure this project cannot
//! recover from, and every repair is a step towards one.
//!
//! `TODO/corpus.md`, `CORPUS-01`.

pub mod capture;
pub mod route;
pub mod store;

pub use capture::{Identity, Refusal, profile_from};
pub use route::{CORPUS_DIR, LAYOUT, NoRoute, RAW_DIR, Route, route};
pub use store::{Added, Index, IndexEntry, Pointers, Published, STABLE, Store};
