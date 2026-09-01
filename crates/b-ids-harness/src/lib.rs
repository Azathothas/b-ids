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
//! - ⛔ **TLS is not terminated.** Completing a handshake can change what a
//!   client offers, and reaching an HTTP/2 connection over TLS needs it. The
//!   cleartext surface reaches HTTP/2 with prior knowledge and `--ca-out` is
//!   the switch that would reach it over TLS.
//!
//! # The rule this crate is built around
//!
//! ⛔ **Parse permissively, emit exactly, and keep the bytes whatever happens.**
//! A capture is a moment that cannot be retaken: the build will be gone, the
//! download will stop being served, and the machine will be reimaged. Every
//! field the parser cannot read becomes a note beside the capture rather than
//! an error that throws it away.

pub mod bytes;
pub mod h2;
pub mod hello;
pub mod hpack;
pub mod listener;
pub mod note;
pub mod rebuild;
pub mod sampling;
pub mod select;

pub use bytes::{Cursor, hex, unhex};
pub use h2::{Http2Capture, RawFrame};
pub use hello::{HelloCapture, parse_record};
pub use listener::{BindRefused, CAPTURE_SCHEMA, Capture, Config, Oracle, Protocol, parse_bind};
pub use note::Note;
pub use rebuild::{Rebuilt, differences, rebuild};
pub use sampling::{Sampling, summarise};
pub use select::{Kind, Selection, select};
