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
//! - ⛔ **HTTP/2 is not here.** Reaching it needs the handshake terminated, and
//!   terminating one can change what the client offers. `HARNESS-03` is that
//!   entry and `--ca-out` is the switch it lands behind.
//!
//! # The rule this crate is built around
//!
//! ⛔ **Parse permissively, emit exactly, and keep the bytes whatever happens.**
//! A capture is a moment that cannot be retaken: the build will be gone, the
//! download will stop being served, and the machine will be reimaged. Every
//! field the parser cannot read becomes a note beside the capture rather than
//! an error that throws it away.

pub mod hello;
pub mod listener;

pub use hello::{HelloCapture, Note, hex, parse_record, unhex};
pub use listener::{BindRefused, CAPTURE_SCHEMA, Capture, Config, Oracle, Protocol, parse_bind};
