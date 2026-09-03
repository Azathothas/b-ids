//! Turning a profile into the form an impersonating client consumes.
//!
//! ⛔ **The emitter's model is NOT the parser's model, and that is the whole
//! design.** They are two different requirements on one table:
//!
//! - the parser accepts any codepoint with any body, because a hello it refused
//!   is a capture thrown away, and a capture is a moment that cannot be
//!   retaken;
//! - the emitter refuses anything it cannot put on the wire byte for byte,
//!   because an approximation is a `ClientHello` that exists nowhere and is
//!   more distinguishing than an honestly old one.
//!
//! ⚠ **A codebase that uses one type for both gets one of them wrong.** The
//! measured cost of getting it wrong in the parser's direction: a parser that
//! mapped a GREASE codepoint to a typed field with an empty body rejected the
//! GREASE extension that carries a byte, which is what a browser sends at the
//! end of its list. Three of the sixteen reserved values were affected, so
//! about one handshake in five. `docs/inherited-claims.md` section 8.
//!
//! # What is here, and what is not
//!
//! [`hello`] is the `ClientHello` side. The HTTP/2 and header sides arrive with
//! `EMIT-01`, and the support matrix with the holes left in is `EMIT-01` as
//! well.

pub mod hello;
pub mod matrix;

pub use hello::{
    EmittableExtension, Unreproducible, client_hello, extension, extensions, extensions_block,
    unnamed_codepoints,
};
pub use matrix::{
    Cell, Hole, MATRIX_SCHEMA, Matrix, REPRODUCE, RUNNABLE_STACK, holes, support_matrix,
};
