//! What a parser could not read, kept beside the capture rather than thrown.
//!
//! ⚠ **A note is not an error.** The capture still happened and the raw bytes
//! are still there; the note says which derived field could not be filled in.
//! A parser that refused a message it did not recognise would have thrown away
//! a moment that cannot be retaken.

/// Something the parser could not read, kept beside the capture.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Note {
    /// The field the note is about.
    pub field: String,
    /// What could not be read.
    pub why: String,
}

impl Note {
    /// A note about `field`, saying `why`.
    #[must_use]
    pub fn new(field: impl Into<String>, why: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            why: why.into(),
        }
    }
}
