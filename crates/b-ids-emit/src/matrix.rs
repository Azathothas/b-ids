//! The support matrix, with the holes left in.
//!
//! ⭐ **A profile is only useful if some stack can emit it, and no published
//! table says which stack can emit which profile.** A client author currently
//! finds out by building it. `TODO/emitters.md`, `EMIT-01`.
//!
//! ⛔ **A CELL IS PRODUCED BY A RUN AND A HOLE IS PRODUCED BY A READING, and
//! the two are different kinds in this model rather than two colours of one.**
//! A table somebody maintains goes stale the day a hole closes and nobody
//! notices; a table generated from a run notices. But this tree can only RUN
//! the stack it has, so every other stack gets a hole row carrying the file and
//! the line it was read at, and no cell at all.
//!
//! ⛔ **Let it have holes.** A cell that says "cannot" is more useful than one
//! that says "approximately", and an emitter that approximates silently is the
//! defect the whole project is about.

use b_ids_schema::Profile;

use crate::hello::{client_hello, unnamed_codepoints};

/// The matrix's own schema identifier.
pub const MATRIX_SCHEMA: &str = "emit-support-matrix/1";

/// A stack this tree can actually run, so its cells come from a run.
pub const RUNNABLE_STACK: &str = "b-ids-emit";

/// The command that reproduces every cell below.
///
/// ⛔ **Named here so the check can assert it, and so a reader can run it.** A
/// cell whose reproduction is a paragraph is a cell nobody re-runs.
pub const REPRODUCE: &str = "cargo test -p b-ids-emit escape_hatch";

/// What one stack did with one profile.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Cell {
    /// The stack.
    pub stack: String,
    /// The profile.
    pub profile: String,
    /// Whether the whole `ClientHello` came out.
    pub emits: bool,
    /// How many bytes it came to, where it did.
    pub bytes: Option<usize>,
    /// How many of the profile's extensions carry a codepoint the model gives
    /// no field to, and which the escape hatch is therefore the only way to
    /// write.
    pub unnamed_codepoints: usize,
    /// Every reason it did not, where it did not.
    pub refusals: Vec<String>,
    /// ⛔ Always `run` for a cell. A cell filled any other way is a hole.
    pub evidence: String,
    /// The command that reproduces it.
    pub reproduce: String,
}

/// What a stack this tree cannot run is known not to be able to emit.
///
/// ⛔ **Read at a file and a line, in a tree this repository holds at a named
/// commit.** A hole filled from a project's own documentation, or from memory,
/// is a claim nobody can re-check. `TODO/RULES.md` section 3.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Hole {
    /// The stack.
    pub stack: String,
    /// What it cannot emit.
    pub cannot: String,
    /// Where that was read, as a path under `references/` and a line.
    pub file: String,
    /// The line.
    pub line: u32,
    /// Whether this project could patch it in its own tree.
    pub patchable_here: bool,
    /// ⛔ Always `read` for a hole. It is not a run and must not read as one.
    pub evidence: String,
}

/// The whole matrix.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Matrix {
    /// The schema this matrix is written against.
    pub schema: String,
    /// Every cell, produced by a run.
    pub cells: Vec<Cell>,
    /// Every hole, produced by a reading.
    pub holes: Vec<Hole>,
}

/// The holes read out of the reference corpus.
///
/// ⚠ **Every one of these is a READING and says so.** The rows are here rather
/// than in a document because the check has to resolve each path and line, and
/// a citation nobody resolves is the defect `TOOL-10` exists for.
#[must_use]
pub fn holes() -> Vec<Hole> {
    let hole = |stack: &str, cannot: &str, file: &str, line: u32, patchable: bool| Hole {
        stack: stack.to_owned(),
        cannot: cannot.to_owned(),
        file: file.to_owned(),
        line,
        patchable_here: patchable,
        evidence: "read".to_owned(),
    };
    vec![
        hole(
            "rustls",
            "an extension whose codepoint was learned at run time: the parser's own doc comment \
             says unknown extensions are dropped, and the extension struct is crate-private",
            "references/apify__rustls/tree/rustls/src/msgs/client_hello.rs",
            147,
            // ⭐ This tree already vendors rustls, so this one IS patchable here.
            true,
        ),
        hole(
            "rustls",
            "an arbitrary captured extension order: the order is drawn from a sixteen-bit seed, \
             so at most 65,536 orders are reachable out of the factorial of the extension count",
            "references/apify__rustls/tree/rustls/src/msgs/client_hello.rs",
            337,
            true,
        ),
        hole(
            "h2",
            "the priority block inside a headers frame: both send-path constructors hardcode no \
             dependency, and the closure that would carry it is passed empty",
            "references/hyperium__h2/tree/src/frame/headers.rs",
            123,
            // ⭐ PATCHED HERE SINCE 2026-09-04, so this flipped from false.
            // vendor/h2 is the tree and patches/h2/ is what changed:
            // StreamDependency::encode, the half `load` never had, and
            // Headers::set_stream_priority, which sets the payload AND the flag
            // in one call. ⚠ The hole is still real about UPSTREAM, which is
            // what this matrix describes; what moved is whether this project
            // can close it, and it can. TODO/emitters.md, EMIT-03.
            true,
        ),
        hole(
            "impit",
            "any unenumerated codepoint at all: its extension set is a boolean per extension \
             beside a closed enum",
            "references/apify__impit/tree/impit/src/fingerprint/types.rs",
            87,
            false,
        ),
        hole(
            "utls",
            "⭐ no known hole for the extension model. It carries an ordered list of \
             codepoint-and-body pairs and refuses an unknown codepoint by default rather than \
             dropping it",
            "references/refraction-networking__utls/tree/u_common.go",
            184,
            false,
        ),
    ]
}

/// Run this project's own emitter over every profile and record what it did.
///
/// ⛔ **Every cell here is a run.** The random is the caller's, and it is the
/// one part of a hello this project does not record; a fixed one is used so two
/// runs of this produce the same byte count.
#[must_use]
pub fn support_matrix(profiles: &[Profile]) -> Matrix {
    // ⚠ A FIXED random, and only because a byte COUNT is what a cell records. A
    // client that sent this would be sending a constant, which is why
    // `b_ids_cli::random` exists and is not this.
    let random = [0_u8; 32];
    let cells = profiles
        .iter()
        .map(|profile| match client_hello(&profile.tls, &random) {
            Ok(bytes) => Cell {
                stack: RUNNABLE_STACK.to_owned(),
                profile: profile.id.to_string(),
                emits: true,
                bytes: Some(bytes.len()),
                unnamed_codepoints: unnamed_codepoints(&profile.tls).len(),
                refusals: Vec::new(),
                evidence: "run".to_owned(),
                reproduce: REPRODUCE.to_owned(),
            },
            Err(why) => Cell {
                stack: RUNNABLE_STACK.to_owned(),
                profile: profile.id.to_string(),
                emits: false,
                bytes: None,
                unnamed_codepoints: unnamed_codepoints(&profile.tls).len(),
                refusals: why.iter().map(ToString::to_string).collect(),
                evidence: "run".to_owned(),
                reproduce: REPRODUCE.to_owned(),
            },
        })
        .collect();
    Matrix {
        schema: MATRIX_SCHEMA.to_owned(),
        cells,
        holes: holes(),
    }
}
