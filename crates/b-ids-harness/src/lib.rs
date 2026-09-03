//! The capture oracle: a listener a browser is pointed at.
//!
//! ⭐ **The reading is taken from OUTSIDE the client.** A client's own account
//! of what it sent is the account it intended, and the two differ often enough
//! that a corpus built from the first is a corpus of nothing.
//!
//! # What is here, and what is not
//!
//! - [`listener`] binds a socket, accepts connections and records each one.
//! - [`hello`] reads a `ClientHello` off the bytes, permissively.
//! - [`h2`] reads the HTTP/2 connection preface and the frames behind it.
//! - [`bytes`] is the bounds-checked cursor both parsers read through, and the
//!   hex both directions.
//! - [`tls`] mints the authority `--ca-out` writes and completes the handshake
//!   behind it, over the vendored rustls at `vendor/rustls`.
//! - ⛔ **TLS is not terminated by DEFAULT.** Completing a handshake can
//!   change what a client offers, so the raw surface stays the default and
//!   `--ca-out` is how a caller opts into the only surface that reaches a
//!   browser's HTTP/2. ⚠ **On one browser it changed nothing**, measured by
//!   [`modes`] on 2026-09-01; the default stays, because the reason for it is
//!   that the answer is a measurement per build rather than a property.
//!
//! # The rule this crate is built around
//!
//! ⛔ **Parse permissively, emit exactly, and keep the bytes whatever happens.**
//! A capture is a moment that cannot be retaken: the build will be gone, the
//! download will stop being served, and the machine will be reimaged. Every
//! field the parser cannot read becomes a note beside the capture rather than
//! an error that throws it away.

pub mod bytes;
pub mod digest;
pub mod fuzz;
pub mod h2;
pub mod hello;
pub mod hpack;
pub mod listener;
pub mod modes;
pub mod note;
pub mod rebuild;
pub mod sampling;
pub mod select;
pub mod tls;

pub use bytes::{Cursor, hex, sha256, unhex};
pub use digest::{ja4, ja4_o};
pub use fuzz::drive_every_parser;
pub use h2::{Http2Capture, RawFrame};
pub use hello::{HelloCapture, parse_record};
pub use listener::{
    BindRefused, CAPTURE_SCHEMA, Capture, Config, Oracle, Protocol, Termination, parse_bind,
};
pub use modes::{
    Comparison, FieldComparison, Split, Stability, Verdict, comparable, compare, resumption_split,
};
pub use note::Note;
pub use rebuild::{Rebuilt, differences, rebuild};
pub use sampling::{Sampling, summarise};
pub use select::{Kind, Selection, select};
pub use tls::{Authority, Terminated, mint};
