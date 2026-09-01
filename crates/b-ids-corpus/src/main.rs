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
//! `TODO/corpus.md`, `CORPUS-01`.

use std::process::ExitCode;

use b_ids_corpus::{Identity, Store, profile_from};
use b_ids_harness::Capture;

const USAGE: &str = "\
usage: b-ids-corpus add --captures FILE --identity FILE [--root DIR]
       b-ids-corpus verify [--root DIR]
       b-ids-corpus latest --assert-stable [--root DIR]
       b-ids-corpus index --write [--root DIR]

  add              turn the cold connection of a navigation into a profile and
                   publish it, with its ClientHello beside it and the index
                   rewritten. Refuses a path that already holds a profile: a
                   correction is a NEW profile naming the one it replaces.
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
    println!(
        "{} connection(s): 1 cold, {} resumed, {} further cold, {} abandoned",
        selection.connections(),
        selection.resumed.len(),
        selection.additional_cold.len(),
        selection.abandoned.len()
    );
    let Some(cold) = selection.cold else {
        eprintln!(
            "b-ids-corpus: no connection of this navigation reached HTTP/2, so there is no cold \
             handshake to publish. {} connection(s) were abandoned",
            selection.abandoned.len()
        );
        return ExitCode::from(1);
    };

    let profile = match profile_from(cold, &identity) {
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
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--write" => write = true,
            "--assert-stable" => assert_stable = true,
            "--root" => match argv.next() {
                Some(value) => root = value,
                None => return fail("--root needs a directory"),
            },
            "--captures" => match argv.next() {
                Some(value) => captures = Some(value),
                None => return fail("--captures needs a path"),
            },
            "--identity" => match argv.next() {
                Some(value) => identity = Some(value),
                None => return fail("--identity needs a path"),
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
        "verify" => verify(&root),
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
