//! Resolving a browser build, and driving it at a URL.
//!
//! ⭐ **Two jobs, kept separate, and the separation is the entry.** Resolving
//! answers "which build is on this machine, and how do I know"; driving answers
//! "launch it at this URL and do not leave anything behind". A component that
//! did both would have two reasons to fail and one message for them.
//!
//! ⛔ **Nothing here acquires a browser.** `DRIVER-05` is acquisition, and a
//! resolver that downloaded one would change the machine it was asked to
//! describe.
//!
//! ⚠ **Every switch a launch passes is recorded on its result**, because each
//! one is a condition of whatever was captured through it. The certificate pin
//! is the one that matters most: it trusts one key for one launch rather than
//! switching verification off.
//!
//! `TODO/driver.md`, `DRIVER-01`.

pub mod drive;
pub mod headless;
pub mod resolve;

pub use drive::{Driven, Launch, drive};
pub use headless::{Normalisation, normalise, normalise_user_agent};
pub use resolve::{Family, NotResolved, Resolved, Source, resolve};
