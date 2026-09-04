//! Resolving a browser build, and driving it at a URL.
//!
//! ⭐ **Two jobs, kept separate, and the separation is the entry.** Resolving
//! answers "which build is on this machine, and how do I know"; driving answers
//! "launch it at this URL and do not leave anything behind". A component that
//! did both would have two reasons to fail and one message for them.
//!
//! ⭐ **[`acquire`] is the third job and it is deliberately not the
//! resolver's.** The resolver reads what is on this machine; acquisition says
//! where a build can be got and records which route answered and the digest of
//! what arrived. ⛔ A resolver that downloaded a browser would change the
//! machine it was asked to describe, so the two stay apart.
//!
//! ⚠ **[`versions`] answers a third question and it is not the resolver's.**
//! The resolver reads what is INSTALLED on this machine; version discovery
//! reads what the vendor is SERVING, which during a staged rollout is a
//! different build. Capturing the one nobody has produces a correct fingerprint
//! of a browser that does not exist. `DRIVER-02`.
//!
//! ⚠ **Every switch a launch passes is recorded on its result**, because each
//! one is a condition of whatever was captured through it. The certificate pin
//! is the one that matters most: it trusts one key for one launch rather than
//! switching verification off.
//!
//! `TODO/driver.md`, `DRIVER-01`.

pub mod acquire;
pub mod drive;
pub mod headless;
pub mod nssdb;
pub mod resolve;
pub mod versions;

pub use acquire::{
    Acquired, Candidate, IndexRefusal, Platform, Refusal, Route, acquire_with, download_url, plan,
};
pub use drive::{Driven, Launch, TrustRoute, drive, trust_route};
pub use headless::{Normalisation, normalise, normalise_user_agent};
pub use nssdb::{Seeded, seed};
pub use resolve::{Family, NotResolved, Resolved, Source, resolve, sources_for};
pub use versions::{Chosen, Release, Report, discover};
