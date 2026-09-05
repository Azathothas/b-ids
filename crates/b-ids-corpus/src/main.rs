//! The corpus command.
//!
//! ⛔ **Exit 0 clean, 1 refused, 2 nothing could be run.** Those are three
//! different facts. A corpus with something wrong in it and a corpus nobody
//! could read mean opposite things about whether you can publish.
//!
//! ⭐ **`add` takes a capture file and an identity file, and neither is typed
//! at a prompt.** The captures come from `b-ids-harness --json` and the
//! identity from a file an operator writes once per subject; a version typed on
//! a command line is a number nobody measured, in the field that decides which
//! build a profile describes.
//!
//! `docs/history/todo/corpus.md`, `CORPUS-01`.

use std::process::ExitCode;

use b_ids_corpus::{
    Built, Format, INITIAL_RELEASE_TAG, Identity, Run, SUPPORT_MATRIX_FILE, Store, batch, build,
    indexes, manifest, model, parse_tag, plan_initial_release, plan_release, profile_from,
    release_body, render, requests, routes, support_matrix, verify as verify_format, would_rewrite,
};
use b_ids_harness::Capture;
use b_ids_schema::Profile;

const USAGE: &str = "\
usage: b-ids-corpus add --captures FILE --identity FILE [--root DIR]
       b-ids-corpus verify [--root DIR]
       b-ids-corpus validate [--root DIR]
       b-ids-corpus latest --assert-stable [--root DIR]
       b-ids-corpus index --write [--root DIR]
       b-ids-corpus formats --out DIR [--root DIR]
       b-ids-corpus anchors --out DIR [--root DIR]
       b-ids-corpus routes --out DIR [--root DIR]
       b-ids-corpus publish --out DIR [--root DIR]
       b-ids-corpus data-branch --head COMMIT|none --parent COMMIT|none
       b-ids-corpus release --tree DIR --tag TAG [--existing FILE] [--notes FILE]
                            [--before DIR] [--root DIR]
       b-ids-corpus pull-request --before DIR --after DIR --run FILE --out DIR

  add              turn the cold connection of a navigation into a profile and
                   publish it, with its ClientHello beside it and the index
                   rewritten. Refuses a path that already holds a profile: a
                   correction is a NEW profile naming the one it replaces.
  validate         run the coherence checks over every PUBLISHED profile and
                   across the whole set, which is the question a push has to
                   settle and the one the validator's own command cannot ask:
                   it takes the paths a caller names. Its LAST line is a fixed
                   `corpus=validate profiles:N findings:N notcheckable:N`.
  verify           every profile validates, sits at the route its own keys
                   derive, publishes the bytes it says it does, and the index
                   is what the tree derives to. Its LAST line is a fixed
                   `corpus=profiles:N problems:N`, which is what
                   scripts/common/check-corpus reads. Parse that, never the
                   prose above it.
  latest           every `latest` pointer resolves to a STABLE profile. A
                   consumer following one must never be handed a pre-release
                   build. --assert-stable is required, because a command that
                   read the pointer file and asserted nothing looks like it
                   did the job.
  index --write    rewrite the index and the pointer file from the tree. The
                   only writer of either.
  formats          generate every published format from the canonical JSON,
                   with the support matrix beside them saying what each one
                   carries and which two were declined. ⚠ Four are LOSSLESS,
                   TOML carries every field that is not null, three are the flat
                   columns and the protobuf one is a DEFINITION rather than
                   values; each says so in its own header. ⛔ Every file is read
                   back before it is written, so a format the generator cannot
                   parse is a refusal rather than an artefact. ⛔ Never
                   hand-edit a generated file: the generator has lost if you do,
                   and scripts/common/check-formats is what says so. Its LAST
                   line is a fixed `corpus=formats files:N profiles:N`.
  anchors          publish every build's trust-anchor list as its own artefact,
                   with the capture date and the identifiers in the browser's
                   own order. ⚠ Most profiles do not carry the extension and
                   that is a fact about the builds; a body that IS there and
                   does not decode is a refusal. Its LAST line is a fixed
                   `corpus=anchors lists:N profiles:N`.
  routes           generate the flat route tree a program with nothing but curl
                   can read: one file per value, at every permutation the corpus
                   HOLDS a value for. ⛔ A single-value file carries the value
                   and nothing else, with no trailing newline, so a consumer
                   never strips anything; a multi-value one says so by its
                   extension. ⛔ Nothing falls back to a neighbouring platform: a
                   missing route is a fact and a substituted value is a lie. Its
                   LAST line is a fixed
                   `corpus=routes files:N single:N profiles:N`.
  publish          assemble the publishable tree: the corpus and its raw
                   sidecars copied verbatim, every generated format, the flat
                   routes, every build's trust-anchor list, the licence, a
                   manifest and a SHA256SUMS file. ⭐ ONE assembler, so a
                   release archive and the data branch cannot publish different
                   bytes. ⛔ Nothing here reads a clock: a build is stamped with
                   the corpus's own digest. ⛔ It writes a directory and pushes
                   nothing. Its LAST line is a fixed
                   `corpus=publish files:N bytes:N profiles:N from:DIGEST`.
  data-branch      would pushing this commit REWRITE the data branch? --head is
                   what the remote holds and --parent is what the new commit was
                   built on; they agree on an append and differ when the commit
                   was built on something the branch has moved past. ⛔ The case
                   that matters is a commit with no parent: an orphan pushed
                   over an existing branch discards every commit on it. Exit 1
                   is a refusal. Its LAST line is a fixed
                   `corpus=data-branch head:X parent:Y verdict:append|rewrite`.
  release          the tag an assembled tree would be released under, and the
                   body that goes with it. --tag is the tag somebody PUSHED and
                   it is parsed, planned against the tags that already carry a
                   release, and rebuilt from its own parts, so a tag this
                   project's rule would not have produced is refused. ⛔ It
                   publishes nothing: no tag, no asset, no remote. Its LAST line
                   is a fixed
                   `corpus=release tag:T files:N bytes:N profiles:N notes:N from:D`.
  pull-request     what a scheduled run that found a change should open: ⭐ ONE
                   request for the RUN, carrying every route it captured, with a
                   branch deterministic in the run identifier, a body a reviewer
                   can read without checking anything out, labels, and the five
                   merge conditions aggregated over every route with the ones
                   that failed named. ⛔ One branch per route was measured and
                   withdrawn: the workflow pushed the same merged tree to each,
                   five branches over one tree on run 33851238648. ⛔ A NO-OP
                   CHANGE PRODUCES NOTHING: silence is the correct output for a
                   browser that did not change. It opens nothing itself; the
                   workflow does. Its LAST line is a fixed
                   `corpus=pull-request requests:N auto:N routes:N`, where
                   requests is 0 or 1 and routes is how many moved.
  --before DIR     the corpus root as it is published today.
  --after DIR      the corpus root with this run's captures merged in.
  --run FILE       what the run knows and the corpus cannot say, as JSON: the
                   workflow, the run identifier, the images, the harness, the
                   command that reproduces this, what the run could not do, and
                   the validator's output. ⛔ Every field is required.
  --head COMMIT    what the data branch holds on the remote, or none.
  --parent COMMIT  what the commit being pushed was built on, or none.
  --tree DIR       a tree `publish` assembled, read through its MANIFEST.json.
  --tag TAG        v0.0.1 for the initial release, then LAYOUT.YYYY.MM.DD.COUNTER.
  --existing FILE  the tags that already carry a release, one per line.
  --notes FILE     where the release body is written. ⚠ Empty is the correct
                   body for a release nothing moved in.
  --out DIR        where the generated formats or lists are written.
  --captures FILE  what `b-ids-harness --json` printed. Its first line is the
                   base URL and is not a capture; every other line is one.
  --identity FILE  what the subject was and under what conditions it was
                   measured, as JSON.
  --root DIR       the corpus root. The working directory by default.

exit 0 clean, 1 refused, 2 nothing could be run.";

fn fail(message: &str) -> ExitCode {
    eprintln!("b-ids-corpus: {message}");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

/// Read the capture file, keeping every line that is one.
///
/// ⛔ **A line that is not a capture is REPORTED, never dropped.** The harness
/// prints the base URL first, so exactly one such line is expected; a second
/// one means the file is not what the caller thinks it is, and a reader that
/// silently skipped it would build a profile from a smaller sample than the
/// operator believes.
fn read_captures(path: &str) -> Result<(Vec<Capture>, Vec<String>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let mut captures = Vec::new();
    let mut skipped = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('{') {
            skipped.push(line.to_owned());
            continue;
        }
        let capture: Capture =
            serde_json::from_str(line).map_err(|e| format!("{path}: not a capture: {e}"))?;
        captures.push(capture);
    }
    Ok((captures, skipped))
}

#[allow(clippy::too_many_lines)]
fn add(root: &str, captures_path: &str, identity_path: &str) -> ExitCode {
    let (captures, skipped) = match read_captures(captures_path) {
        Ok(pair) => pair,
        Err(why) => return fail(&why),
    };
    for line in &skipped {
        println!("skipped a line that is not a capture: {line}");
    }
    if captures.is_empty() {
        return fail(&format!("{captures_path} holds no captures"));
    }

    let identity_text = match std::fs::read_to_string(identity_path) {
        Ok(text) => text,
        Err(err) => return fail(&format!("{identity_path}: {err}")),
    };
    let identity: Identity = match serde_json::from_str(&identity_text) {
        Ok(identity) => identity,
        Err(err) => return fail(&format!("{identity_path}: not an identity: {err}")),
    };

    // ⭐ THE SELECTION IS HERE, and it is the one place a profile's connection
    // is chosen. A browser opens sockets it abandons and it resumes; the cold
    // connection is the one a fresh client sends, and a resumed one produces a
    // different digest. Nothing is averaged and nothing is deduplicated.
    let selection = b_ids_harness::select(&captures);
    println!("{}", selection.report());
    println!("{}", selection.halves());

    // ⛔ TWO ABSENCES, NAMED SEPARATELY. "No connection sent a cold hello" and
    // "no connection reached HTTP/2" send a reader to two different places, and
    // a message covering both would send them to neither. docs/history/todo/harness.md,
    // HARNESS-15.
    let Some(tls_from) = selection.tls_from else {
        eprintln!(
            "b-ids-corpus: every hello in this navigation offered a pre-shared key, so the \
             session resumed on all {} connection(s) and there is no cold handshake to publish. \
             ⛔ A resumed hello is NOT published in a cold one's place.",
            selection.connections()
        );
        return ExitCode::from(1);
    };
    let Some(http2_from) = selection.http2_from else {
        eprintln!(
            "b-ids-corpus: no connection of this navigation reached HTTP/2, so there is no \
             HTTP/2 half to publish. {} connection(s) carried none",
            selection.no_http2.len()
        );
        return ExitCode::from(1);
    };

    let profile = match profile_from(tls_from, http2_from, &identity) {
        Ok(profile) => profile,
        Err(refusals) => {
            for refusal in &refusals {
                eprintln!("b-ids-corpus: {refusal}");
            }
            return ExitCode::from(1);
        }
    };

    let store = Store::at(root);
    match store.add(&profile) {
        Ok(added) => {
            println!("wrote {}", added.profile.display());
            println!("wrote {}", added.hello.display());
        }
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    }
    match store.write_index() {
        Ok((index, pointers)) => {
            println!("wrote {}", index.display());
            println!("wrote {}", pointers.display());
        }
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    }
    println!("{}", profile.id);
    ExitCode::SUCCESS
}

/// The prefix of the one line a caller parses.
///
/// ⛔ **A FIXED LAST LINE, and it is the whole machine contract.** Both halves
/// of `scripts/common/check-corpus` read this and nothing else: parsing the
/// prose above it would make every wording change a silent behaviour change.
/// `scripts/common/check-powershell.ps1` carries the same discipline, and its
/// header says why.
const STATUS: &str = "corpus=";

fn verify(root: &str) -> ExitCode {
    let store = Store::at(root);
    // ⛔ 2, not 0. A tree with no corpus has verified nothing, which is a
    // different fact from a corpus with nothing wrong in it.
    if !store.exists() {
        eprintln!(
            "b-ids-corpus: there is no corpus at {}/{}. Nothing was verified",
            root,
            b_ids_corpus::CORPUS_DIR
        );
        println!("{STATUS}absent");
        return ExitCode::from(2);
    }
    let problems = match store.verify() {
        Ok(problems) => problems,
        Err(why) => return fail(&why),
    };
    let profiles = store.profile_paths().map_or(0, |p| p.len());
    for problem in &problems {
        println!("{problem}");
    }
    // ⛔ Last, after everything a person reads, and always printed.
    println!("{STATUS}profiles:{profiles} problems:{}", problems.len());
    if problems.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Assert that every `latest` pointer resolves to a stable profile.
///
/// ⛔ **It reads the pointer file on disk rather than the derivation.** A
/// consumer follows the file, so the file is what has to be right; the
/// derivation cannot produce a bad entry and this is what catches a hand-edited
/// one. `CORPUS-03`.
fn latest(root: &str) -> ExitCode {
    let store = Store::at(root);
    if !store.exists() {
        eprintln!(
            "b-ids-corpus: there is no corpus at {}/{}. Nothing was checked",
            root,
            b_ids_corpus::CORPUS_DIR
        );
        println!("{STATUS}absent");
        return ExitCode::from(2);
    }
    let problems = match store.latest_that_is_not_stable() {
        Ok(problems) => problems,
        Err(why) => return fail(&why),
    };
    for problem in &problems {
        println!("{problem}");
    }
    println!("{STATUS}latest problems:{}", problems.len());
    if problems.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Run the coherence checks over every published profile, and across the set.
///
/// ⭐ **The corpus-scale form, which is the one nothing had.** The validator's
/// own command takes paths a caller names, so it answers about whatever
/// somebody remembered to list. This answers about what is PUBLISHED, which is
/// the question a push has to settle, and it is answerable here because this
/// crate owns the layout. `docs/history/todo/ci.md`, `CI-01`.
///
/// ⛔ **`publishing` is on, and it is not a caller's choice.** Every profile
/// this reads is already published, so a `vendor`-provenance field in one of
/// them is a defect rather than a draft.
///
/// ⚠ **`NotCheckable` is counted and reported, never folded into the pass.** A
/// corpus of one profile cannot answer the handshake check at all, and a run
/// that printed nothing about that would say the corpus is coherent when what
/// it means is that three of eight checks had nothing to read.
///
/// ⭐ **The cross-profile leg is why this reads the whole corpus at once.**
/// [`b_ids_validator::shared_handshakes`] is the form of check 4 that runs
/// across a set: two profiles claiming different majors and carrying a
/// byte-identical TLS half, of which at most one was measured. ⚠ It is
/// structurally silent on a corpus of one, and `CORPUS-02` is what ends that.
fn validate_corpus(root: &str) -> ExitCode {
    let store = Store::at(root);
    if !store.exists() {
        eprintln!(
            "b-ids-corpus: there is no corpus at {}/{}. Nothing was validated",
            root,
            b_ids_corpus::CORPUS_DIR
        );
        println!("{STATUS}absent");
        return ExitCode::from(2);
    }
    let entries = match store.profiles() {
        Ok(entries) => entries,
        Err(why) => return fail(&why),
    };
    // ⛔ 2, not 0. A corpus directory holding no profile has validated nothing,
    // which is the "step that exits 0 having done nothing it was asked to do"
    // row in docs/conventions/forbidden-patterns.md.
    if entries.is_empty() {
        eprintln!("b-ids-corpus: the corpus holds no profile, so nothing was validated");
        println!("{STATUS}absent");
        return ExitCode::from(2);
    }

    let options = b_ids_validator::Options {
        publishing: true,
        ..b_ids_validator::Options::default()
    };
    let mut findings = 0_usize;
    let mut notcheckable = 0_usize;
    for (path, profile) in &entries {
        // ⚠ The route, never the absolute path, for the reason Store::verify
        // gives: a message naming a path on the machine that ran the check is a
        // message nobody else can act on.
        let at = b_ids_corpus::route::as_route(path.strip_prefix(store.root()).unwrap_or(path));
        let report = b_ids_validator::validate(profile, &options);
        for (check, outcome) in &report.results {
            match outcome {
                b_ids_validator::Outcome::Passed => {}
                b_ids_validator::Outcome::Failed(found) => {
                    for finding in found {
                        println!("{at}: FAIL  {finding}");
                        findings += 1;
                    }
                }
                b_ids_validator::Outcome::NotCheckable(why) => {
                    println!("{at}: SKIP  {check} -- {why}");
                    notcheckable += 1;
                }
            }
        }
    }

    let profiles: Vec<b_ids_schema::Profile> =
        entries.iter().map(|(_, profile)| profile.clone()).collect();
    for finding in b_ids_validator::shared_handshakes(&profiles) {
        println!("across the corpus: FAIL  {finding}");
        findings += 1;
    }

    // ⛔ Last, after everything a person reads, and always printed.
    println!(
        "{STATUS}validate profiles:{} findings:{findings} notcheckable:{notcheckable}",
        entries.len()
    );
    if findings == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn index(root: &str) -> ExitCode {
    let store = Store::at(root);
    if !store.exists() {
        eprintln!(
            "b-ids-corpus: there is no corpus at {}/{}. Nothing to index",
            root,
            b_ids_corpus::CORPUS_DIR
        );
        return ExitCode::from(2);
    }
    match store.write_index() {
        Ok((index, pointers)) => {
            println!("wrote {}", index.display());
            println!("wrote {}", pointers.display());
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            ExitCode::from(1)
        }
    }
}

/// Generate every published format from the canonical corpus.
///
/// ⛔ **ONE GENERATOR AND ONE READ OF THE TREE.** Every format comes out of the
/// same `Vec<Profile>`, in the same route order the index uses, so two formats
/// cannot disagree about what the corpus holds. `docs/history/todo/schema.md`, `SCHEMA-08`.
///
/// ⛔ **The last line is a fixed `corpus=formats files:N profiles:N`**, which is
/// what `scripts/common/check-formats` reads. Parse that, never the prose above
/// it.
fn formats(root: &str, out_dir: &str) -> ExitCode {
    let store = Store::at(root);
    if !store.exists() {
        // ⛔ 2, not 0. A tree with no corpus has generated nothing, which is a
        // different fact from a corpus that generated cleanly.
        eprintln!("b-ids-corpus: there is no corpus under {root}, so there is nothing to generate");
        return ExitCode::from(2);
    }
    let profiles: Vec<Profile> = match store.profiles() {
        Ok(found) => found.into_iter().map(|(_, profile)| profile).collect(),
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    };

    if let Err(why) = std::fs::create_dir_all(out_dir) {
        eprintln!("b-ids-corpus: cannot create {out_dir}: {why}");
        return ExitCode::from(2);
    }

    let mut written = 0_usize;
    for format in Format::all() {
        let text = match render(format, &profiles) {
            Ok(text) => text,
            Err(why) => {
                eprintln!("b-ids-corpus: {format}: {why}");
                return ExitCode::from(1);
            }
        };
        // ⛔ READ BACK BEFORE IT IS WRITTEN. The round-trip suite renders its
        // own fixture, so a format could be correct over two invented profiles
        // and wrong over the published corpus, and the file would still be on
        // disk. `docs/history/todo/schema.md`, `SCHEMA-12`.
        if let Err(why) = verify_format(format, &profiles, &text) {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
        let path = std::path::Path::new(out_dir).join(format.file_name());
        if let Err(why) = std::fs::write(&path, text.as_bytes()) {
            eprintln!("b-ids-corpus: {}: {why}", path.display());
            return ExitCode::from(1);
        }
        println!("wrote {} ({} bytes)", path.display(), text.len());
        written += 1;
    }

    // ⛔ GENERATED BESIDE THE FORMATS IT DESCRIBES. A support matrix kept by
    // hand states what somebody believed on the day they wrote it, and a
    // consumer reading it has no way to tell. `docs/history/todo/schema.md`, `SCHEMA-12`.
    let matrix = support_matrix();
    let path = std::path::Path::new(out_dir).join(SUPPORT_MATRIX_FILE);
    if let Err(why) = std::fs::write(&path, matrix.as_bytes()) {
        eprintln!("b-ids-corpus: {}: {why}", path.display());
        return ExitCode::from(1);
    }
    println!("wrote {} ({} bytes)", path.display(), matrix.len());
    written += 1;

    println!("corpus=formats files:{written} profiles:{}", profiles.len());
    ExitCode::SUCCESS
}

/// Generate the flat route tree a program with nothing but `curl` can read.
///
/// ⛔ **The last line is a fixed `corpus=routes files:N single:N profiles:N`**,
/// which is what `scripts/common/check-routes` reads.
///
/// ⛔ **A single-value file is written with NO trailing newline**, which is the
/// whole of this entry. `docs/history/todo/publish.md`, `PUB-03`.
fn routes_command(root: &str, out_dir: &str) -> ExitCode {
    let store = Store::at(root);
    if !store.exists() {
        eprintln!("b-ids-corpus: there is no corpus under {root}, so there is nothing to publish");
        return ExitCode::from(2);
    }
    let published = match store.profiles() {
        Ok(found) => found
            .into_iter()
            .map(|(path, profile)| {
                (
                    // ⚠ Written with forward slashes whatever the host uses and
                    // with no leading `./`, so a manifest generated on Windows
                    // names the same path a reader on Linux opens, and one
                    // generated with `--root .` names the same path as one
                    // generated with an absolute root.
                    path.to_string_lossy()
                        .replace('\\', "/")
                        .trim_start_matches("./")
                        .to_owned(),
                    profile,
                )
            })
            .collect::<Vec<_>>(),
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    };

    let generated = routes(&published);
    let mut single = 0_usize;
    let mut written = 0_usize;
    for route in &generated {
        let path = std::path::Path::new(out_dir).join(&route.path);
        if let Some(parent) = path.parent()
            && let Err(why) = std::fs::create_dir_all(parent)
        {
            eprintln!("b-ids-corpus: cannot create {}: {why}", parent.display());
            return ExitCode::from(1);
        }
        // ⛔ THE BYTES ARE THE VALUE. A single-value file gets no newline at
        // all; a list gets one after every entry including the last, so a
        // consumer reading lines sees exactly its entries.
        let body = if route.multi_value {
            format!("{}\n", route.value)
        } else {
            route.value.clone()
        };
        if let Err(why) = std::fs::write(&path, body.as_bytes()) {
            eprintln!("b-ids-corpus: {}: {why}", path.display());
            return ExitCode::from(1);
        }
        if !route.multi_value {
            single += 1;
        }
        written += 1;
    }

    for (path, body) in indexes(&generated) {
        let path = std::path::Path::new(out_dir).join(path);
        if let Some(parent) = path.parent()
            && let Err(why) = std::fs::create_dir_all(parent)
        {
            eprintln!("b-ids-corpus: cannot create {}: {why}", parent.display());
            return ExitCode::from(1);
        }
        if let Err(why) = std::fs::write(&path, body.as_bytes()) {
            eprintln!("b-ids-corpus: {}: {why}", path.display());
            return ExitCode::from(1);
        }
        written += 1;
    }

    let manifest = manifest(generated);
    let path = std::path::Path::new(out_dir).join(b_ids_corpus::routes::MANIFEST_FILE);
    let text = match serde_json::to_string_pretty(&manifest) {
        Ok(text) => format!("{text}\n"),
        Err(why) => {
            eprintln!("b-ids-corpus: serialising the manifest: {why}");
            return ExitCode::from(1);
        }
    };
    if let Err(why) = std::fs::write(&path, text.as_bytes()) {
        eprintln!("b-ids-corpus: {}: {why}", path.display());
        return ExitCode::from(1);
    }
    written += 1;

    println!(
        "corpus=routes files:{written} single:{single} profiles:{}",
        published.len()
    );
    ExitCode::SUCCESS
}

/// Assemble the publishable tree.
///
/// ⛔ **The last line is a fixed `corpus=publish files:N bytes:N profiles:N
/// from:DIGEST`**, which is what `scripts/common/check-release` and
/// `scripts/common/check-data-branch` both read.
///
/// ⛔ **It writes a directory and pushes nothing.** A release workflow archives
/// what this produced and a data-branch workflow commits it; a command that did
/// either would be one thing with two jobs. `docs/history/todo/publish.md`, `PUB-01` and
/// `PUB-02`.
fn publish_command(root: &str, out_dir: &str) -> ExitCode {
    let out = std::path::Path::new(out_dir);
    if let Err(why) = std::fs::create_dir_all(out) {
        eprintln!("b-ids-corpus: cannot create {out_dir}: {why}");
        return ExitCode::from(2);
    }
    match build(root, out) {
        Ok(built) => {
            println!(
                "corpus=publish files:{} bytes:{} profiles:{} from:{}",
                built.artefacts.len(),
                built.bytes(),
                built.profiles,
                built.generated_from
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            // ⛔ 2 for a tree with no corpus, 1 for a corpus that would not
            // build. Those are different facts about whether you can publish.
            if why.starts_with("there is no corpus") {
                ExitCode::from(2)
            } else {
                ExitCode::from(1)
            }
        }
    }
}

/// Whether the commit a workflow built may be pushed to the data branch.
///
/// ⛔ **The rule lives in the crate and this is how a workflow reaches it.** A
/// workflow that decided this in shell would be a second statement of the rule,
/// and the day the two disagreed the branch would be the thing that lost.
/// `b_ids_corpus::publish::would_rewrite` is the rule and its four cases are in
/// the suite. `docs/history/todo/publish.md`, `PUB-02` and `PUB-10`.
///
/// ⚠ **`none` is how absence is spelled on a command line.** An empty argument
/// is what a shell produces from an unset variable, so both mean absent and
/// neither is a parse error: the caller reading `head=` from a branch that does
/// not exist gets the same answer as one that wrote `none`.
fn data_branch_command(head: &str, parent: &str) -> ExitCode {
    fn present(value: &str) -> Option<&str> {
        let value = value.trim();
        if value.is_empty() || value == "none" {
            None
        } else {
            Some(value)
        }
    }
    let (head, parent) = (present(head), present(parent));
    let rewrite = would_rewrite(head, parent);
    println!(
        "corpus=data-branch head:{} parent:{} verdict:{}",
        head.unwrap_or("none"),
        parent.unwrap_or("none"),
        if rewrite { "rewrite" } else { "append" }
    );
    if !rewrite {
        return ExitCode::SUCCESS;
    }
    eprintln!("b-ids-corpus: this push would REWRITE the data branch, and it is append-only.");
    if parent.is_none() {
        // ⛔ THE CASE THAT MATTERS. A branch built with `--orphan` destroys
        // itself on its second run, and the push that does it looks like every
        // other push.
        eprintln!(
            "b-ids-corpus: the commit has no parent, so pushing it over an existing branch \
             would discard every commit on it."
        );
    }
    ExitCode::from(1)
}

/// The tag a release would take, and the body that goes with it.
///
/// ⛔ **It publishes nothing.** No tag is created, no asset uploaded and no
/// remote written to: this reads an assembled tree, plans the release against
/// the tags that already carry one, and writes the body. The workflow holds the
/// token. `docs/history/todo/publish.md`, `PUB-01` and `PUB-10`.
///
/// ⚠ **The tag is the CALLER'S**, because a release job is handed one somebody
/// pushed rather than asked to invent one. The explicit `v0.0.1` bootstrap tag
/// is planned once; a later tag is parsed, planned, and rebuilt from its own
/// parts, so a tag the dated rule would not have produced fails the round trip.
fn release_command(
    root: &str,
    tree: &str,
    wanted: &str,
    existing_file: Option<&str>,
    notes_file: Option<&str>,
    before: Option<&str>,
) -> ExitCode {
    let manifest_path = std::path::Path::new(tree).join(b_ids_corpus::publish::MANIFEST);
    let manifest_text = match std::fs::read_to_string(&manifest_path) {
        Ok(text) => text,
        // ⛔ 2, not 1. A tree with no manifest is one this never read, which is
        // a different fact from a tree it read and refused.
        Err(err) => return fail(&format!("{}: {err}", manifest_path.display())),
    };
    let built: Built = match serde_json::from_str(&manifest_text) {
        Ok(built) => built,
        Err(err) => {
            return fail(&format!(
                "{}: not a manifest: {err}",
                manifest_path.display()
            ));
        }
    };

    // ⛔ THE TAGS THAT ALREADY CARRY A RELEASE, read by the caller from the
    // repository. A published release is immutable, so the question is whether
    // this one would overwrite an asset somebody has pinned.
    let existing: Vec<String> = match existing_file {
        None => Vec::new(),
        Some(path) => match std::fs::read_to_string(path) {
            Ok(text) => text
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect(),
            Err(err) => return fail(&format!("{path}: {err}")),
        },
    };

    let planned = if wanted == INITIAL_RELEASE_TAG {
        match plan_initial_release(&built, &existing) {
            Ok(planned) => planned,
            Err(why) => {
                eprintln!("b-ids-corpus: {why}");
                return ExitCode::from(1);
            }
        }
    } else {
        let Some((layout, date, counter)) = parse_tag(wanted) else {
            eprintln!(
                "b-ids-corpus: {wanted} is not a release tag. Use {INITIAL_RELEASE_TAG} for the \
                 initial release or LAYOUT.YYYY.MM.DD.COUNTER, for example v1.2026.09.03.1"
            );
            return ExitCode::from(1);
        };
        if layout != built.layout {
            eprintln!(
                "b-ids-corpus: {wanted} names layout {layout} and this build is {}",
                built.layout
            );
            return ExitCode::from(1);
        }
        let planned = match plan_release(&built, &date, counter, &existing) {
            Ok(planned) => planned,
            Err(why) => {
                eprintln!("b-ids-corpus: {why}");
                return ExitCode::from(1);
            }
        };
        if planned != wanted {
            // ⚠ THE ROUND TRIP IS THE CHECK. `v1.2026.09.03.01` parses, plans,
            // and rebuilds as `v1.2026.09.03.1`, which is a different tag.
            eprintln!(
                "b-ids-corpus: {wanted} is not the tag this rule produces, which is {planned}"
            );
            return ExitCode::from(1);
        }
        planned
    };

    let load = |root: &str| -> Result<Vec<Profile>, String> {
        let store = Store::at(root);
        if !store.exists() {
            // ⚠ ABSENT IS EMPTY HERE, as it is for `pull-request`: the first
            // release has no published state behind it and every route is then
            // new. A refusal would make the first release the one with no body.
            return Ok(Vec::new());
        }
        store
            .profiles()
            .map(|found| found.into_iter().map(|(_, profile)| profile).collect())
    };
    // ⛔ NO `--before` IS AN EMPTY STATE, never the working directory. Passing
    // "" to the loader resolved to the corpus this build was made from, so the
    // before and the after were the same set and the body of the FIRST release
    // came out empty. Found by running it.
    let was = before.map_or_else(|| Ok(Vec::new()), load);
    let (was, now) = match (was, load(root)) {
        (Ok(was), Ok(now)) => (was, now),
        (Err(why), _) | (_, Err(why)) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    };
    // ⛔ THE BODY IS `PUB-08`'s RENDERER, never one written here. A release body
    // and a changelog entry that disagree is the defect that entry exists to
    // make impossible.
    let body = release_body(&model(&was, &now));
    if let Some(path) = notes_file
        && let Err(why) = std::fs::write(path, body.as_bytes())
    {
        eprintln!("b-ids-corpus: {path}: {why}");
        return ExitCode::from(1);
    }

    println!(
        "corpus=release tag:{planned} files:{} bytes:{} profiles:{} notes:{} from:{}",
        built.artefacts.len(),
        built.bytes(),
        built.profiles,
        body.len(),
        built.generated_from
    );
    ExitCode::SUCCESS
}

/// What a scheduled run that found a change should open.
///
/// ⛔ **It opens nothing.** The workflow holds the token and this holds the
/// text, so a generator that could not reach a network is testable and a step
/// that calls an API is one thing with one job. `docs/history/todo/ci.md`, `CI-04`.
///
/// ⛔ **The last line is a fixed `corpus=pull-request requests:N auto:N routes:N`**,
/// which is what `scripts/common/check-pr-body` and the workflow both read.
fn pull_request_command(before: &str, after: &str, run_file: &str, out_dir: &str) -> ExitCode {
    let load = |root: &str| -> Result<Vec<Profile>, String> {
        let store = Store::at(root);
        if !store.exists() {
            // ⚠ An ABSENT corpus is an empty one here rather than a refusal,
            // and that is deliberate: the first run of this on a fresh tree has
            // no published state to compare against, and every route is then
            // new. A refusal would make the first change the one nobody sees.
            return Ok(Vec::new());
        }
        store
            .profiles()
            .map(|found| found.into_iter().map(|(_, profile)| profile).collect())
    };
    let (before, after) = match (load(before), load(after)) {
        (Ok(before), Ok(after)) => (before, after),
        (Err(why), _) | (_, Err(why)) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    };
    let run_text = match std::fs::read_to_string(run_file) {
        Ok(text) => text,
        Err(err) => return fail(&format!("{run_file}: {err}")),
    };
    let run: Run = match serde_json::from_str(&run_text) {
        Ok(run) => run,
        // ⛔ 2, not 1. A run file this reader cannot parse means nothing was
        // asked about the corpus, which is a different fact from a corpus with
        // something wrong in it.
        Err(err) => return fail(&format!("{run_file}: not a run: {err}")),
    };

    if let Err(why) = std::fs::create_dir_all(out_dir) {
        eprintln!("b-ids-corpus: cannot create {out_dir}: {why}");
        return ExitCode::from(2);
    }

    // ⛔ ONE REQUEST PER RUN, AND THE ROUTE COUNT IS REPORTED BESIDE IT. The
    // generator opened one request per route until 2026-09-04 and the workflow
    // pushed the same merged tree to each branch: five branches, one tree,
    // `97248d83821e`, abbreviated for the secret scan, on run 33851238648.
    // ⚠ `requests` is still the model: `batch` composes it, so the per-route
    // body a reviewer reads is unchanged and there is now one place to read it.
    let routes = requests(&before, &after, &run).len();
    let opened: Vec<b_ids_corpus::Request> = batch(&before, &after, &run).into_iter().collect();
    let mut auto = 0_usize;
    for request in &opened {
        // ⚠ The branch is a path, so the directory it is written under replaces
        // the separators.
        let dir = std::path::Path::new(out_dir).join(request.branch.replace('/', "_"));
        if let Err(why) = std::fs::create_dir_all(&dir) {
            eprintln!("b-ids-corpus: cannot create {}: {why}", dir.display());
            return ExitCode::from(1);
        }
        let files = [
            ("branch", request.branch.clone()),
            ("title", request.title.clone()),
            ("body.md", request.body.clone()),
            ("labels", request.labels.join("\n")),
            (
                "mergeable",
                if request.conditions.met() {
                    "auto".to_owned()
                } else {
                    request.conditions.failed().join("\n")
                },
            ),
        ];
        for (name, text) in files {
            let path = dir.join(name);
            if let Err(why) = std::fs::write(&path, text.as_bytes()) {
                eprintln!("b-ids-corpus: {}: {why}", path.display());
                return ExitCode::from(1);
            }
        }
        if request.conditions.met() {
            auto += 1;
        }
        println!("{} -> {}", request.branch, dir.display());
    }

    println!(
        "corpus=pull-request requests:{} auto:{auto} routes:{routes}",
        opened.len()
    );
    ExitCode::SUCCESS
}

/// Publish every build's trust-anchor list as its own artefact.
///
/// ⛔ **Beside the corpus rather than inside a profile.** The list is a snapshot
/// of the browser's own root store and it changes on a different schedule from
/// everything else a profile carries, so a consumer that wants it should not
/// have to fetch a whole profile, and a consumer that does not want it should
/// not be handed it. `docs/history/todo/corpus.md`, `CORPUS-04`.
///
/// ⛔ **The last line is a fixed `corpus=anchors lists:N profiles:N`**, which is
/// what `scripts/common/check-trust-anchors` reads.
fn anchors(root: &str, out_dir: &str) -> ExitCode {
    let store = Store::at(root);
    if !store.exists() {
        eprintln!("b-ids-corpus: there is no corpus under {root}, so there is nothing to publish");
        return ExitCode::from(2);
    }
    let profiles: Vec<Profile> = match store.profiles() {
        Ok(found) => found.into_iter().map(|(_, profile)| profile).collect(),
        Err(why) => {
            eprintln!("b-ids-corpus: {why}");
            return ExitCode::from(1);
        }
    };

    // ⛔ A MALFORMED BODY IS A REFUSAL, never a skip. `anchor_lists` skips the
    // profiles that do not carry the extension, which is the ordinary case;
    // this loop is what separates that from a body that is there and does not
    // decode, because the second is a defect and the first is not.
    let mut lists = Vec::new();
    for profile in &profiles {
        match b_ids_corpus::anchor_list(profile) {
            Ok(list) => lists.push(list),
            Err(b_ids_corpus::NotAList::Absent) => {}
            Err(why) => {
                eprintln!("b-ids-corpus: {}: {why}", profile.id);
                return ExitCode::from(1);
            }
        }
    }

    if let Err(why) = std::fs::create_dir_all(out_dir) {
        eprintln!("b-ids-corpus: cannot create {out_dir}: {why}");
        return ExitCode::from(2);
    }

    for list in &lists {
        let path = std::path::Path::new(out_dir).join(format!(
            "{}-{}-{}.json",
            list.browser.to_ascii_lowercase(),
            list.version,
            list.platform
        ));
        let text = match serde_json::to_string_pretty(list) {
            Ok(text) => format!("{text}\n"),
            Err(err) => {
                eprintln!("b-ids-corpus: serialising {}: {err}", list.profile_id);
                return ExitCode::from(1);
            }
        };
        if let Err(why) = std::fs::write(&path, text.as_bytes()) {
            eprintln!("b-ids-corpus: {}: {why}", path.display());
            return ExitCode::from(1);
        }
        println!(
            "wrote {} ({} identifier(s), captured {})",
            path.display(),
            list.identifiers.len(),
            list.captured_at
        );
    }

    println!(
        "corpus=anchors lists:{} profiles:{}",
        lists.len(),
        profiles.len()
    );
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let mut argv = std::env::args().skip(1);
    let Some(command) = argv.next() else {
        return fail("no command");
    };

    let mut root = ".".to_owned();
    let mut captures = None;
    let mut identity = None;
    let mut write = false;
    let mut assert_stable = false;
    let mut out_dir: Option<String> = None;
    let mut before: Option<String> = None;
    let mut after: Option<String> = None;
    let mut run_file: Option<String> = None;
    let mut head: Option<String> = None;
    let mut parent: Option<String> = None;
    let mut tree: Option<String> = None;
    let mut wanted_tag: Option<String> = None;
    let mut existing: Option<String> = None;
    let mut notes: Option<String> = None;
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--write" => write = true,
            "--assert-stable" => assert_stable = true,
            "--root" => match argv.next() {
                Some(value) => root = value,
                None => return fail("--root needs a directory"),
            },
            "--out" => match argv.next() {
                Some(value) => out_dir = Some(value),
                None => return fail("--out needs a directory"),
            },
            "--captures" => match argv.next() {
                Some(value) => captures = Some(value),
                None => return fail("--captures needs a path"),
            },
            "--identity" => match argv.next() {
                Some(value) => identity = Some(value),
                None => return fail("--identity needs a path"),
            },
            "--before" => match argv.next() {
                Some(value) => before = Some(value),
                None => return fail("--before needs a directory"),
            },
            "--after" => match argv.next() {
                Some(value) => after = Some(value),
                None => return fail("--after needs a directory"),
            },
            "--run" => match argv.next() {
                Some(value) => run_file = Some(value),
                None => return fail("--run needs a path"),
            },
            // ⚠ EMPTY IS ALLOWED and means absent, because a shell reading a
            // branch that does not exist produces an empty string rather than
            // the word. `data-branch` treats the two the same.
            "--head" => match argv.next() {
                Some(value) => head = Some(value),
                None => return fail("--head needs a commit, or none"),
            },
            "--parent" => match argv.next() {
                Some(value) => parent = Some(value),
                None => return fail("--parent needs a commit, or none"),
            },
            "--tree" => match argv.next() {
                Some(value) => tree = Some(value),
                None => return fail("--tree needs a directory"),
            },
            "--tag" => match argv.next() {
                Some(value) => wanted_tag = Some(value),
                None => return fail("--tag needs a tag"),
            },
            "--existing" => match argv.next() {
                Some(value) => existing = Some(value),
                None => return fail("--existing needs a path"),
            },
            "--notes" => match argv.next() {
                Some(value) => notes = Some(value),
                None => return fail("--notes needs a path"),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => return fail(&format!("unknown argument: {other}")),
        }
    }

    match command.as_str() {
        "add" => {
            let (Some(captures), Some(identity)) = (captures, identity) else {
                return fail("add needs --captures and --identity");
            };
            add(&root, &captures, &identity)
        }
        "formats" => {
            let Some(out_dir) = out_dir else {
                return fail("formats needs --out, the directory to generate into");
            };
            formats(&root, &out_dir)
        }
        "publish" => {
            let Some(out_dir) = out_dir else {
                return fail("publish needs --out, the directory to assemble into");
            };
            publish_command(&root, &out_dir)
        }
        "routes" => {
            let Some(out_dir) = out_dir else {
                return fail("routes needs --out, the directory to generate into");
            };
            routes_command(&root, &out_dir)
        }
        // ⛔ BOTH ARGUMENTS ARE REQUIRED rather than defaulted to absent. A
        // caller that forgot one would otherwise be told the push appends,
        // which is the answer that lets it through.
        "data-branch" => {
            let (Some(head), Some(parent)) = (head, parent) else {
                return fail("data-branch needs --head and --parent, each a commit or none");
            };
            data_branch_command(&head, &parent)
        }
        "release" => {
            let (Some(tree), Some(wanted_tag)) = (tree, wanted_tag) else {
                return fail("release needs --tree, an assembled tree, and --tag");
            };
            release_command(
                &root,
                &tree,
                &wanted_tag,
                existing.as_deref(),
                notes.as_deref(),
                before.as_deref(),
            )
        }
        "anchors" => {
            let Some(out_dir) = out_dir else {
                return fail("anchors needs --out, the directory to publish into");
            };
            anchors(&root, &out_dir)
        }
        "pull-request" => {
            let (Some(before), Some(after), Some(run_file), Some(out_dir)) =
                (before, after, run_file, out_dir)
            else {
                return fail("pull-request needs --before, --after, --run and --out");
            };
            pull_request_command(&before, &after, &run_file, &out_dir)
        }
        "verify" => verify(&root),
        "validate" => validate_corpus(&root),
        // ⛔ The flag is required for the same reason `index` requires
        // --write and the validator's `import` requires --report.
        "latest" if assert_stable => latest(&root),
        "latest" => fail("latest needs --assert-stable"),
        // ⛔ --write is required rather than defaulted, for the reason the
        // validator's `import` requires --report: a command that read the tree
        // and wrote nothing looks like it did the job.
        "index" if write => index(&root),
        "index" => fail("index needs --write"),
        other => fail(&format!("unknown command: {other}")),
    }
}
