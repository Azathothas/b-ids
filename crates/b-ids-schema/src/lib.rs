//! The profile model: one browser, one build, one platform, one channel, one
//! instant.
//!
//! Every other crate in this workspace reads or writes a [`Profile`], which is
//! why this crate is the one that had to exist first.
//!
//! # The shape, and why it has this shape
//!
//! - **Three measured halves**, [`tls`], [`http2`] and [`http`], each defined
//!   by its own entry in `TODO/schema.md`.
//! - **`digests` and `raw` are siblings of the measured halves, never inside
//!   them.** A digest is derived from a profile; a profile is never derived
//!   from a digest, and putting a derived value inside a measured block is how
//!   the two stop being distinguishable.
//! - **[`provenance`] is a sibling map keyed by field path**, so a consumer can
//!   ask "was this measured or copied" per field without every scalar being
//!   wrapped in a struct nobody reads.
//!
//! # What this crate does not do
//!
//! It does not capture, validate coherence, or emit. It defines what a capture
//! writes, and it refuses a profile that is malformed on its own terms:
//! a missing capture instant, an identifier that disagrees with the four keys
//! it is derived from, a provenance kind outside the four. Asking whether a
//! well-formed profile could have come from a real browser is
//! `b-ids-validator`'s question.

mod error;
mod id;
mod profile;
mod provenance;

#[cfg(feature = "fixtures")]
pub mod fixture;
pub mod http;
pub mod http2;
pub mod instant;
pub mod tls;

pub use error::Defect;
pub use id::{PlatformToken, ProfileId, version_order};
pub use profile::{
    Acquisition, Browser, Captured, Channel, Digests, Os, Platform, Profile, Raw, RecordLayer,
    Resumption, Trust,
};
pub use provenance::{Provenance, ProvenanceEntry, ProvenanceKind};

/// The schema identifier every profile carries, and the only value this version
/// of the crate accepts.
///
/// ⛔ A version is part of the data rather than implied by the reader. Anything
/// persisted carries a version and enough structure to be evolved, so old data
/// still reads and new code knows which version it is looking at.
pub const SCHEMA_ID: &str = "browser-profile/1";
