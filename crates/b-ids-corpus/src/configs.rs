//! Generated client configuration and detection rules, gated on the support
//! matrix.
//!
//! ⭐ **Most of the day-to-day value of a corpus is the artefact somebody pastes
//! into their own tool, and none of the generated DATA formats is that.** A
//! consumer reading `formats/corpus.json` still has to work out what to do with
//! it. `TODO/publish.md`, `PUB-04`.
//!
//! ⛔ **A SNIPPET IS GENERATED ONLY WHERE THE SUPPORT MATRIX SAYS THE PAIR IS
//! EMITTABLE, and a pair the matrix records as a hole gets a comment naming the
//! hole instead.** That is the whole design constraint, and it is the one this
//! project exists to hold: a snippet that silently approximates is worse than
//! no snippet, because it produces a client that is almost right, and
//! `TODO/RULES.md` rule 2 says an almost-right fingerprint is more
//! distinguishing than an honestly old one.
//!
//! ⚠ **A cell comes from a run and a hole comes from a reading**, and this
//! module never converts one into the other. `b_ids_emit::matrix` is where that
//! distinction is made and this module only reads it.
//!
//! ⛔ **No digest artefact is generated here**, and the absence is deliberate
//! rather than pending. The corpus holds no digest, and the operator's ruling
//! that a route resolves only to a value the corpus HOLDS is what declined
//! digest routes under `PUB-03`. A digest allowlist is the same value on a
//! different surface, so it needs its own ruling rather than this module's
//! judgement. `TODO/PROGRESS.md` carries the question.

use b_ids_emit::{Hole, Matrix, RUNNABLE_STACK, client_hello, unnamed_codepoints};
use b_ids_schema::{Profile, http::Variant};

/// Where generated configuration is published, under the build root.
pub const CONFIGS_DIR: &str = "configs";

/// The file every generated tree carries, explaining what the tree is.
pub const CONFIGS_README: &str = "README.md";

/// Whether a generated file is a usable snippet or a refusal that names a hole.
///
/// ⛔ **Two kinds, never one with a flag**, because the check that reads this
/// tree has to be able to fail on a snippet that should have been a hole, and a
/// boolean on one type is a field somebody forgets to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The support matrix carries a cell for this pair and the cell emits.
    Snippet,
    /// The support matrix carries a hole for this stack.
    Hole,
}

impl Kind {
    /// The kind's name, as the manifest and the check spell it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Snippet => "snippet",
            Self::Hole => "hole",
        }
    }
}

/// One generated file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    /// Its path under [`CONFIGS_DIR`].
    pub path: String,
    /// Its bytes.
    pub body: String,
    /// The stack it is for.
    pub stack: String,
    /// The profile it is for, or empty where it is not per profile.
    pub profile: String,
    /// Whether it is usable or a named refusal.
    pub kind: Kind,
}

/// The route directory a profile's configuration is published under.
///
/// ⚠ **The same four keys the flat routes use, in the same order**, so a
/// consumer that can find a route can find the configuration beside it.
fn profile_dir(profile: &Profile) -> String {
    format!(
        "{}/{}/{}/{}",
        profile.browser.name.to_lowercase(),
        profile.browser.channel.as_str(),
        profile.platform_token().as_str(),
        profile.browser.version
    )
}

/// Render the snippet for the one stack this tree can run.
///
/// ⛔ **The bytes are produced by running the emitter over this profile**, not
/// described. A snippet carrying a hand-written byte count would be a claim
/// about a run nobody made.
fn runnable_snippet(profile: &Profile) -> Result<String, String> {
    // ⚠ A FIXED RANDOM, and it is not a shortcut. The 32 random bytes are the
    // one part of a ClientHello that is different on every connection by
    // design, so a snippet that pinned a captured one would be teaching a
    // reader to send a replayed random. Zeroes are visibly not a real draw.
    let random = [0_u8; 32];
    let bytes = client_hello(&profile.tls, &random)
        .map_err(|why| format!("the emitter refused {}: {why:?}", profile.id))?;
    let unnamed = unnamed_codepoints(&profile.tls);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    Ok(format!(
        "// {stack}, for {id}\n\
         //\n\
         // Generated from the corpus. The support matrix records this pair as\n\
         // emittable, and the byte count below came from the run that produced\n\
         // this file rather than from a table.\n\
         //\n\
         //   profile        {id}\n\
         //   bytes          {len}\n\
         //   extensions     {ext} carrying a codepoint the model gives no field to,\n\
         //                  which the escape hatch is the only way to write\n\
         //\n\
         // The 32 random bytes are zeroed here on purpose: they are the one part\n\
         // of a ClientHello that differs on every connection, so a snippet that\n\
         // pinned a captured draw would teach a replay.\n\
         \n\
         use b_ids_emit::client_hello;\n\
         \n\
         let profile: b_ids_schema::Profile = serde_json::from_str(PROFILE_JSON)?;\n\
         let random: [u8; 32] = rand::random();\n\
         let hello = client_hello(&profile.tls, &random)?;\n\
         assert_eq!(hello.len(), {len});\n\
         \n\
         // The same call with a zeroed random produces exactly these bytes:\n\
         // {hex}\n",
        stack = RUNNABLE_STACK,
        id = profile.id,
        len = bytes.len(),
        ext = unnamed.len(),
    ))
}

/// Render the refusal for a stack the matrix records a hole against.
///
/// ⛔ **It names the hole, the file and the line**, because a refusal a reader
/// cannot check is a refusal they will assume is out of date.
fn hole_refusal(profile: &Profile, holes: &[&Hole]) -> String {
    let mut out = format!(
        "# {stack}, for {id}\n\
         #\n\
         # ⛔ NO SNIPPET IS GENERATED FOR THIS PAIR, and that is the answer\n\
         # rather than a gap. The support matrix records this stack as unable to\n\
         # emit what this profile carries, so a snippet here would produce a\n\
         # client that is almost right, which is more distinguishing than an\n\
         # honestly old one.\n\
         #\n\
         # What it cannot do, each read at a file and a line in a tree this\n\
         # project holds at a named commit:\n#\n",
        stack = holes.first().map_or("unknown", |h| h.stack.as_str()),
        id = profile.id,
    );
    for hole in holes {
        out.push_str(&format!("#   {}\n", hole.cannot));
        out.push_str(&format!("#     read at {}:{}\n", hole.file, hole.line));
        out.push_str(&format!(
            "#     patchable in this tree: {}\n#\n",
            if hole.patchable_here { "yes" } else { "no" }
        ));
    }
    out.push_str(
        "# A hole that closes upstream is a hole this project re-reads. It is not\n\
         # a permanent verdict about the stack.\n",
    );
    out
}

/// The detection-side rule for one profile.
///
/// ⭐ **The same data, the other direction**, and it is half the reason this
/// project is acceptable to everybody: a server deciding whether a caller is a
/// real browser needs exactly what a client trying to look like one needs.
///
/// ⛔ **Built only from values the corpus HOLDS.** The header names and their
/// order, the ALPN list and the version strings are measured; nothing here is
/// computed at generation time, which is the same rule that declined digest
/// routes under `PUB-03`.
fn detection_rule(profile: &Profile) -> String {
    // ⚠ THE NAVIGATION VARIANT, NAMED. A profile carries a header set per
    // request kind and they are not the same order, so a rule built from
    // "the headers" would be built from whichever one happened to be first.
    let order: Vec<&str> = profile
        .http
        .variant(Variant::Navigate)
        .map(|set| set.headers.iter().map(|h| h.name.as_str()).collect())
        .unwrap_or_default();
    let alpn = profile.tls.alpn.join(", ");
    format!(
        "# detection, for {id}\n\
         #\n\
         # ⚠ THIS MATCHES A BUILD, NOT A BROWSER. It is the header order and the\n\
         # ALPN list this project measured for one build on one platform. A\n\
         # different build of the same browser sends a different set, so a rule\n\
         # pinned this tightly refuses real traffic as the fleet updates.\n\
         #\n\
         # ⛔ Every value below is one the corpus HOLDS. Nothing here is computed\n\
         # at generation time.\n\
         #\n\
         #   browser        {name} {version}\n\
         #   platform       {platform}\n\
         #   captured       {captured}\n\
         #\n\
         # The header order, which is the cheapest signal on this list and the\n\
         # one a naive client gets wrong first:\n\
         #\n\
         {order}\n\
         #\n\
         # The ALPN list, in the order it was offered:\n\
         #\n\
         #   {alpn}\n",
        id = profile.id,
        name = profile.browser.name,
        version = profile.browser.version,
        platform = profile.platform_token().as_str(),
        captured = profile.captured.at,
        order = order
            .iter()
            .enumerate()
            .map(|(i, h)| format!("#   {:>2}. {h}", i + 1))
            .collect::<Vec<_>>()
            .join("\n"),
        alpn = alpn,
    )
}

/// The tree's own explanation.
fn readme(snippets: usize, holes: usize, stacks: &[String]) -> String {
    format!(
        "# configs\n\n\
         Generated client configuration and detection rules, one directory per\n\
         profile, produced from the corpus by `b-ids-corpus publish`.\n\n\
         ⛔ **A snippet exists only where the support matrix records the pair as\n\
         emittable.** Where it records a hole, this tree carries a file naming\n\
         the hole, the stack, and the file and line it was read at, and no\n\
         snippet at all. A snippet that silently approximated would produce a\n\
         client that is almost right, which is more distinguishing than an\n\
         honestly old one.\n\n\
         ⚠ **A cell comes from a run and a hole comes from a reading.**\n\
         `formats/formats.md` carries the matrix itself.\n\n\
         This build generated **{snippets}** snippet(s) and **{holes}** hole\n\
         file(s), over these stacks:\n\n\
         {stacks}\n\n\
         ⛔ **No digest artefact is here.** The corpus holds no digest, and a\n\
         value computed at generation time is the one thing this project's\n\
         routes were ruled not to carry.\n",
        snippets = snippets,
        holes = holes,
        stacks = stacks
            .iter()
            .map(|s| format!("- `{s}`"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Generate every configuration file for a corpus, gated on `matrix`.
///
/// # Errors
///
/// A string naming the profile the emitter refused, where the matrix said it
/// would emit. ⛔ That disagreement is an error rather than a hole: the matrix
/// and the emitter are the same code in this tree, so the two disagreeing means
/// one of them is wrong rather than that the stack cannot do it.
pub fn configs(profiles: &[Profile], matrix: &Matrix) -> Result<Vec<Generated>, String> {
    let mut out = Vec::new();
    let mut stacks: Vec<String> = Vec::new();
    let mut snippets = 0_usize;
    let mut holes = 0_usize;

    for profile in profiles {
        let dir = profile_dir(profile);
        let id = profile.id.as_str();

        // -- the stack this tree runs, where its cell says it emits ----------
        //
        // ⛔ THE CELL IS LOOKED UP BY PROFILE, never assumed to cover every
        // one. A matrix with a cell for five of six profiles must generate five
        // snippets and nothing for the sixth.
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.profile == id && c.stack == RUNNABLE_STACK);
        if let Some(cell) = cell
            && cell.emits
        {
            out.push(Generated {
                path: format!("{dir}/{RUNNABLE_STACK}.rs"),
                body: runnable_snippet(profile)?,
                stack: RUNNABLE_STACK.to_owned(),
                profile: id.to_owned(),
                kind: Kind::Snippet,
            });
            snippets += 1;
            if !stacks.iter().any(|s| s == RUNNABLE_STACK) {
                stacks.push(RUNNABLE_STACK.to_owned());
            }
        }

        // -- every stack the matrix records a hole against -------------------
        //
        // ⚠ GROUPED BY STACK. A stack with two holes gets one file naming both,
        // because two files for one stack would read as two stacks.
        let mut seen: Vec<&str> = Vec::new();
        for hole in &matrix.holes {
            if seen.contains(&hole.stack.as_str()) {
                continue;
            }
            seen.push(hole.stack.as_str());
            let group: Vec<&Hole> = matrix
                .holes
                .iter()
                .filter(|h| h.stack == hole.stack)
                .collect();
            out.push(Generated {
                path: format!("{dir}/{}.txt", hole.stack),
                body: hole_refusal(profile, &group),
                stack: hole.stack.clone(),
                profile: id.to_owned(),
                kind: Kind::Hole,
            });
            holes += 1;
            if !stacks.contains(&hole.stack) {
                stacks.push(hole.stack.clone());
            }
        }

        // -- the detection side ----------------------------------------------
        out.push(Generated {
            path: format!("{dir}/detect.conf"),
            body: detection_rule(profile),
            stack: "detection".to_owned(),
            profile: id.to_owned(),
            kind: Kind::Snippet,
        });
        snippets += 1;
        if !stacks.iter().any(|s| s == "detection") {
            stacks.push("detection".to_owned());
        }
    }

    out.push(Generated {
        path: CONFIGS_README.to_owned(),
        body: readme(snippets, holes, &stacks),
        stack: String::new(),
        profile: String::new(),
        kind: Kind::Snippet,
    });

    Ok(out)
}
