//! `b-ids-conformance` - how close is this client to the browser it claims to be?
//!
//! ⛔ **A FIELD-LEVEL DIFF, NEVER A DIGEST COMPARISON.** `docs/history/todo/validator.md`,
//! `VALID-05`.
//!
//! Usage:
//!   b-ids-conformance --claim ID --observed FILE [--root DIR] [--json]
//!   b-ids-conformance --list
//!   b-ids-conformance --fixture
//!
//! `--claim` names a profile in the corpus, which is what the client says it
//! is. `--observed` is a captured profile produced from the client under test.
//!
//! ⚠ **THE OBSERVED SIDE IS A FILE, NOT A LIVE CAPTURE, AND THAT IS A LIMIT
//! RATHER THAN A DESIGN.** Capturing a client means standing up the harness and
//! pointing the client at it, which `experiments/10-first-profile.sh` already
//! does for a browser. This command compares what that produces. A client
//! author runs the capture once and this as often as they like.
//!
//! Exit codes: 0 every field both sides carry agrees, 1 at least one differs,
//! 2 could not run.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b_ids_conformance::{Verdict, compare, fields, render_report};
use b_ids_schema::Profile;

/// The environment variable that names a corpus root explicitly.
///
/// ⭐ **The same seam every other reader of the corpus uses.**
/// `docs/history/todo/publish.md`, `PUB-11`.
const ROOT_ENV: &str = "B_IDS_CORPUS_ROOT";

fn fail(why: &str) -> ExitCode {
    eprintln!("b-ids-conformance: {why}");
    ExitCode::from(2)
}

/// The corpus this run should read.
fn corpus_root(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(named) = explicit {
        return Some(PathBuf::from(named));
    }
    // ⛔ AND THE REST IS b_ids_schema::root's, NOT A SECOND COPY. PUB-13 moved
    // corpus/ off the default branch and four files had their own walk; this one
    // keeps only the layer above it, which is the `--root` argument this command
    // takes and the library has no opinion about.
    let here = std::env::current_dir().ok()?;
    b_ids_schema::root::corpus_root_from(&here)
}

/// Every profile in the corpus, read from the tree rather than from the index.
///
/// ⚠ **The tree, because the index is derived.** A profile on disk that the
/// index does not list is still a profile a client could claim, and reading the
/// index would silently narrow what this command can be asked about.
fn profiles(root: &Path) -> Vec<(PathBuf, Profile)> {
    let mut out = Vec::new();
    let mut stack = vec![root.join("corpus").join("v1")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if !name.ends_with(".json") || name == "index.json" || name == "latest.json" {
                continue;
            }
            let Ok(body) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(profile) = serde_json::from_str::<Profile>(&body) {
                out.push((path, profile));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Build a claimed profile and one that differs in exactly one field.
///
/// ⛔ **The entry's acceptance in a command.** "Run against a client that
/// deliberately differs in one field, the report names that field and nothing
/// else." A fixture is what lets that be re-run without a browser.
fn fixture(root: Option<&str>) -> ExitCode {
    // ⛔ A REAL PROFILE FROM THE CORPUS, not the schema's test fixture. The
    // fixture module is behind a feature meant for tests, and enabling it here
    // would ship test data in a binary. ⭐ Reading the corpus is also the
    // stronger test: it proves the comparison over what this project actually
    // published rather than over a shape written to be comparable.
    let Some(root) = corpus_root(root) else {
        return fail(&format!(
            "the fixture needs a corpus. Set {ROOT_ENV} or pass --root"
        ));
    };
    let all = profiles(&root);
    let Some((_, claimed)) = all.first() else {
        return fail(&format!("{} holds no profile", root.display()));
    };
    let mut observed = claimed.clone();

    // ⚠ A REORDERED HEADER, NOT A REORDERED EXTENSION. The extension order is
    // shuffled per connection, so a difference there is a per-connection verdict
    // rather than a conformance failure and would prove nothing here. The header
    // order is stable per request kind, so a client that gets it wrong is wrong.
    // ⛔ It is also the cheapest signal a naive client gets wrong first.
    let Some(set) = observed
        .http
        .variants
        .iter_mut()
        .find(|s| s.variant == b_ids_schema::http::Variant::Navigate)
    else {
        eprintln!("b-ids-conformance: the profile carries no navigation header set");
        return ExitCode::from(2);
    };
    if set.headers.len() < 2 {
        eprintln!("b-ids-conformance: the profile carries too few headers to reorder");
        return ExitCode::from(2);
    }
    set.headers.swap(0, 1);
    observed.id = observed.derived_id();

    let report = compare(claimed, &observed);
    let differing = report.differing();

    let named: Vec<&str> = differing.iter().map(|f| f.field.as_str()).collect();
    // ⛔ THE SET IS ASSERTED, not its size. A fixture that only counted would
    // pass if the tool named a different field.
    let expected = ["http.navigate.header_order"];
    if named != expected {
        eprintln!(
            "b-ids-conformance fixture FAILED: swapping two extensions should differ on \
             {expected:?} and the report named {named:?}"
        );
        return ExitCode::from(1);
    }
    // ⚠ AND THE REST HAD TO AGREE. A tool that reported every field as
    // different would satisfy the assertion above if it were only a membership
    // test.
    if report.conforming() == 0 {
        eprintln!("b-ids-conformance fixture FAILED: no field conformed, so nothing was compared");
        return ExitCode::from(1);
    }
    println!(
        "conformance fixture ok: one swapped header pair is reported as exactly one\n\
         differing field, {}, with {} other field(s) conforming, {} varying per\n\
         connection and {} not checkable.",
        expected[0],
        report.conforming(),
        report.per_connection().len(),
        report.not_checkable().len()
    );
    ExitCode::SUCCESS
}

/// A stack this project can emit for, and what it can emit.
///
/// ⛔ **A TARGET IS CLAIMED ONLY WHERE THIS PROJECT CAN PRODUCE THE BYTES.**
/// `EMIT-04`'s rule is not to claim a target the conformance run has not
/// passed, so the list here is what the tree can drive rather than what it
/// intends to support. ⚠ Adding a row without an emitter behind it is exactly
/// the claim that rule forbids.
struct Stack {
    /// What a caller names.
    name: &'static str,
    /// What it is, in one line.
    what: &'static str,
}

/// Every stack this project emits for today.
///
/// ⭐ **One, and the Approach said to start with it**: "whichever stack this
/// project already uses". That is this tree's own emitter, built on the
/// vendored `rustls` and the vendored and patched `h2`.
///
/// ⚠ **The Go TLS library is the cheapest SECOND target** and it is not here,
/// because nothing in this tree emits for it yet and a row with no emitter
/// behind it would be a claim rather than a target.
const STACKS: [Stack; 1] = [Stack {
    name: "b-ids",
    what: "this project's own emitter: b_ids_emit::hello over the vendored rustls, \
           and b_ids_emit::priority over the vendored and patched h2",
}];

/// Run the conformance comparison against what a stack emits.
///
/// ⛔ **THE WRITER IS NOT THE READER, which is what makes this a comparison.**
/// The bytes are written by `b_ids_emit` and read back by `b_ids_harness`,
/// which is the parser every capture in the corpus went through. A run that
/// compared the emitter's own model against itself would prove nothing.
///
/// ⚠ **The RANDOM is not emitted from the profile**, because a profile does not
/// record one: it is 32 bytes a client draws per connection. A fixed value is
/// used here and nothing downstream compares it, which is why
/// `tls.session_id_len` rather than the session id itself is a field.
fn stack_run(name: &str, claim: &str, root: Option<&str>, json: bool) -> ExitCode {
    let Some(stack) = STACKS.iter().find(|s| s.name == name) else {
        let known: Vec<&str> = STACKS.iter().map(|s| s.name).collect();
        return fail(&format!(
            "no emitter for stack {name:?}. This tree emits for: {}. \
             EMIT-04's rule is not to claim a target the conformance run has not passed.",
            known.join(", ")
        ));
    };
    let Some(root) = corpus_root(root) else {
        return fail(&format!(
            "no corpus found above the working directory. Set {ROOT_ENV} or pass --root"
        ));
    };
    let all = profiles(&root);
    let Some((_, claimed)) = all.iter().find(|(_, p)| p.id.to_string() == claim) else {
        return fail(&format!("{claim} is not a profile in {}", root.display()));
    };

    // ⛔ THE HOLES ARE DERIVED FROM THE EMITTER'S OWN REFUSAL, never declared in
    // a table beside it. `extensions` returns what it cannot reproduce and why,
    // so a hole here is a thing the code says it cannot do rather than a thing
    // somebody remembered to write down.
    let random = [0u8; 32];
    let bytes = match b_ids_emit::hello::client_hello(&claimed.tls, &random) {
        Ok(bytes) => bytes,
        Err(refusals) => {
            if json {
                // ⛔ SERIALISED, NEVER FORMATTED, which is this tree's own rule
                // and which check-placeholders enforced from the other side: an
                // escaped brace in a format string is the shape a template
                // placeholder takes, and that check cannot tell one from the
                // other. A value carrying a character that has to be escaped
                // would also emit JSON that does not parse.
                println!(
                    "{}",
                    serde_json::json!({
                        "schema": "conformance-stack/1",
                        "stack": stack.name,
                        "claim": claim,
                        "emitted": false,
                        "refusals": refusals.len(),
                    })
                );
            } else {
                println!("{} cannot emit {claim}:", stack.name);
                for refusal in &refusals {
                    println!("  {refusal}");
                }
                println!(
                    "\n⛔ Reported rather than approximated. An emitter that wrote a plausible \
                     value here would produce a profile wrong in a way nothing notices."
                );
            }
            return ExitCode::from(1);
        }
    };

    // ⭐ BACK THROUGH THE PARSER EVERY REAL CAPTURE WENT THROUGH.
    let raw = b_ids_schema::Raw {
        client_hello_hex: Some(b_ids_harness::hex(&bytes)),
        ..claimed.raw.clone()
    };
    let rebuilt = b_ids_harness::rebuild::rebuild(&raw, b_ids_schema::http::ValuePolicy::NamesOnly);
    let Some(tls) = rebuilt.tls else {
        return fail(&format!(
            "{} emitted {} byte(s) that this project's own parser could not read back",
            stack.name,
            bytes.len()
        ));
    };

    let mut observed = claimed.clone();
    observed.tls = tls;
    let report = compare(claimed, &observed);
    let differing = report.differing();

    if json {
        println!(
            "{}",
            serde_json::json!({
                "schema": "conformance-stack/1",
                "stack": stack.name,
                "claim": claim,
                "emitted": true,
                "bytes": bytes.len(),
                "differing": differing.len(),
                "conforming": report.conforming(),
            })
        );
        return if differing.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
    }

    println!("stack {} against {claim}", stack.name);
    println!("  {}", stack.what);
    println!(
        "  emitted {} byte(s), read back by b_ids_harness",
        bytes.len()
    );
    if differing.is_empty() {
        println!(
            "\n⭐ no differing field. {} conforming, {} varying per connection, {} not checkable.",
            report.conforming(),
            report.per_connection().len(),
            report.not_checkable().len()
        );
        println!(
            "⛔ The writer is b_ids_emit and the reader is b_ids_harness, so this is a \
             comparison rather than the emitter checking its own arithmetic."
        );
        return ExitCode::SUCCESS;
    }
    println!("\n⛔ {} field(s) differ:", differing.len());
    for field in &differing {
        println!("  {}: {:?}", field.field, field.verdict);
    }
    println!(
        "\n⚠ EMIT-01's support matrix records what a stack cannot reproduce. A field here \
         that the matrix does not record as a hole is a defect in the emitter."
    );
    ExitCode::from(1)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut claim: Option<String> = None;
    let mut observed_path: Option<String> = None;
    let mut root: Option<String> = None;
    let mut json = false;
    let mut list = false;
    let mut run_fixture = false;
    let mut stack: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--claim" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return fail("--claim needs a profile id");
                };
                claim = Some(v.clone());
            }
            "--observed" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return fail("--observed needs a path to a captured profile");
                };
                observed_path = Some(v.clone());
            }
            "--root" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return fail("--root needs a directory");
                };
                root = Some(v.clone());
            }
            // ⭐ EMIT-04. Compare a claimed profile against what this project's
            // own emitter produces for a named stack, with no capture and no
            // network: the bytes are written by b_ids_emit and read back by
            // b_ids_harness.
            "--stack" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    return fail("--stack needs the name of a stack this project emits for");
                };
                stack = Some(v.clone());
            }
            "--json" => json = true,
            "--list" => list = true,
            "--fixture" => run_fixture = true,
            "-h" | "--help" => {
                println!("{}", include_str!("usage.txt"));
                return ExitCode::SUCCESS;
            }
            other => return fail(&format!("unknown argument: {other}")),
        }
        i += 1;
    }

    if run_fixture {
        return fixture(root.as_deref());
    }

    if list {
        for field in fields() {
            println!("{field}");
        }
        return ExitCode::SUCCESS;
    }

    let Some(claim) = claim else {
        return fail("a run needs --claim, the profile this client says it is");
    };

    // ⚠ BEFORE --observed IS REQUIRED, because --stack replaces it: the
    // observed side comes from this project's own emitter rather than from a
    // file somebody captured.
    if let Some(name) = stack {
        return stack_run(&name, &claim, root.as_deref(), json);
    }
    let Some(root) = corpus_root(root.as_deref()) else {
        return fail(&format!(
            "no corpus found above the working directory. Set {ROOT_ENV} or pass --root"
        ));
    };

    let all = profiles(&root);
    if all.is_empty() {
        return fail(&format!("{} holds no profile", root.display()));
    }
    let Some((_, claimed)) = all.iter().find(|(_, p)| p.id.as_str() == claim) else {
        // ⛔ NAMED, NOT GUESSED. A claim this corpus cannot answer is a "could
        // not run", and answering with the nearest profile would report a
        // client as conforming to something it never claimed.
        eprintln!("b-ids-conformance: no profile in this corpus has the id {claim}. It holds:");
        for (_, p) in &all {
            eprintln!("  {}", p.id);
        }
        return ExitCode::from(2);
    };

    let Some(observed_path) = observed_path else {
        return fail(
            "a run needs --observed, a captured profile produced from the client under test",
        );
    };
    let body = match std::fs::read_to_string(&observed_path) {
        Ok(b) => b,
        Err(e) => return fail(&format!("{observed_path}: {e}")),
    };
    let observed: Profile = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(e) => return fail(&format!("{observed_path} is not a profile: {e}")),
    };

    let report = compare(claimed, &observed);
    let differing = report.differing();

    if json {
        let rows: Vec<serde_json::Value> = report
            .fields
            .iter()
            .map(|f| match &f.verdict {
                Verdict::Conforms(v) => serde_json::json!({
                    "field": f.field, "verdict": "conforms", "value": v
                }),
                Verdict::Differs { claimed, observed } => serde_json::json!({
                    "field": f.field, "verdict": "differs",
                    "claimed": claimed, "observed": observed
                }),
                Verdict::PerConnection {
                    claimed,
                    observed,
                    why,
                } => serde_json::json!({
                    "field": f.field, "verdict": "per-connection",
                    "claimed": claimed, "observed": observed, "why": why
                }),
                Verdict::NotCheckable { claimed, observed } => serde_json::json!({
                    "field": f.field, "verdict": "not-checkable",
                    "claimed_carries": claimed, "observed_carries": observed
                }),
            })
            .collect();
        let doc = serde_json::json!({
            "schema": "conformance/1",
            "claimed": report.claimed,
            "observed": report.observed,
            "compared": report.fields.len(),
            "conforming": report.conforming(),
            "differing": differing.len(),
            "not_checkable": report.not_checkable().len(),
            "fields": rows,
        });
        println!("{doc}");
    } else {
        print!("{}", render_report(&report));
    }

    if differing.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
