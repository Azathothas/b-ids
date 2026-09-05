//! The command form of the validator.
//!
//! ⛔ **Exit 0 clean, 1 a check refused, 2 nothing could be checked.** Those are
//! three different facts, and a command that returned 1 for the last two would
//! hide the difference between "this profile is wrong" and "this run verified
//! nothing".
//!
//! ⭐ **`import` reads the prior art's own tables and reports what the checks
//! say about them.** It is the one result this project can publish without a
//! capture. ⛔ It reports a defect, a file, a line and the check it fails, and
//! nothing about the project it read.

use std::collections::BTreeSet;
use std::process::ExitCode;

use b_ids_schema::Profile;
use b_ids_validator::{Options, Outcome, validate};

fn usage() -> &'static str {
    "usage: b-ids-validator [--publishing] [--decodes LIST] PROFILE.json...\n\
            b-ids-validator diff BEFORE.json AFTER.json\n\
            b-ids-validator import DIR --report\n\
     \n\
       --publishing   refuse a vendor-provenance field, which a draft may carry\n\
       --decodes LIST comma-separated content encodings the consumer can decode\n\
       diff A B       what changed between two profiles, field by field.\n\
                      ⛔ It says so when the two differ in more than the\n\
                      version, because nothing can then be attributed to it.\n\
       import DIR     read the reference corpus at DIR and report what the\n\
                      coherence checks say about the tables in it\n\
       --report       print that report. Required by import, because a run\n\
                      that read the corpus and printed nothing is not a result\n\
     \n\
     exit 0 clean, 1 a check refused, 2 nothing could be checked."
}

/// Read the reference corpus and print what the checks say about it.
///
/// ⛔ **Exit 1 when it finds something**, the same as a refused profile. A
/// command that reported violations and exited 0 would be a command nothing
/// downstream could act on.
fn import(dir: &str, report: bool) -> ExitCode {
    if !report {
        eprintln!("b-ids-validator: import needs --report\n{}", usage());
        return ExitCode::from(2);
    }
    let exhibits = match b_ids_validator::import_references(std::path::Path::new(dir)) {
        Ok(exhibits) => exhibits,
        Err(why) => {
            // ⛔ 2, not 1. A reader that went blind verified nothing, which is a
            // different fact from a corpus with nothing wrong in it.
            eprintln!("b-ids-validator: {why}");
            return ExitCode::from(2);
        }
    };
    print!("{}", b_ids_validator::render_report(&exhibits));
    if exhibits.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn main() -> ExitCode {
    let mut options = Options::default();
    let mut paths: Vec<String> = Vec::new();
    let mut import_dir: Option<String> = None;
    let mut report = false;
    let mut diffing = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            // ⭐ "What changed between these two versions" is the most useful
            // artefact for anybody maintaining a client, and it is free once two
            // profiles exist. docs/history/todo/validator.md, VALID-06.
            "diff" => diffing = true,
            "import" => {
                let Some(dir) = args.next() else {
                    eprintln!("b-ids-validator: import needs a directory\n{}", usage());
                    return ExitCode::from(2);
                };
                import_dir = Some(dir);
            }
            "--report" => report = true,
            "--publishing" => options.publishing = true,
            "--decodes" => {
                let Some(list) = args.next() else {
                    eprintln!("b-ids-validator: --decodes needs a value\n{}", usage());
                    return ExitCode::from(2);
                };
                options.decodes = list
                    .split(',')
                    .map(|t| t.trim().to_lowercase())
                    .filter(|t| !t.is_empty())
                    .collect::<BTreeSet<String>>();
            }
            "-h" | "--help" => {
                println!("{}", usage());
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("b-ids-validator: unknown argument: {other}\n{}", usage());
                return ExitCode::from(2);
            }
            other => paths.push(other.to_owned()),
        }
    }

    if let Some(dir) = import_dir {
        return import(&dir, report);
    }

    if diffing {
        // ⛔ EXACTLY TWO. A diff of three profiles is three diffs, and a
        // command that guessed which pair was meant would be guessing about
        // the only thing the caller had to say.
        let [before, after] = paths.as_slice() else {
            eprintln!(
                "b-ids-validator: diff needs exactly two profiles, given {}\n{}",
                paths.len(),
                usage()
            );
            return ExitCode::from(2);
        };
        let mut loaded = Vec::new();
        for path in [before, after] {
            let text = match std::fs::read_to_string(path) {
                Ok(text) => text,
                Err(err) => {
                    eprintln!("{path}: {err}");
                    return ExitCode::from(2);
                }
            };
            match serde_json::from_str::<Profile>(&text) {
                Ok(profile) => loaded.push(profile),
                Err(err) => {
                    eprintln!("{path}: not a profile: {err}");
                    return ExitCode::from(2);
                }
            }
        }
        let report = b_ids_validator::diff(&loaded[0], &loaded[1]);
        print!(
            "{}",
            b_ids_validator::render_diff(&loaded[0], &loaded[1], &report)
        );
        // ⚠ 0 WHATEVER IT FOUND. A diff is a report rather than a verdict:
        // two versions differing is what versions do, and a command that
        // exited 1 for it would make every pipeline treat a normal release
        // as a failure.
        return ExitCode::SUCCESS;
    }

    if paths.is_empty() {
        eprintln!("b-ids-validator: no profile named\n{}", usage());
        return ExitCode::from(2);
    }

    let mut worst = 0_u8;
    for path in &paths {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) => {
                eprintln!("{path}: {err}");
                worst = worst.max(2);
                continue;
            }
        };
        let profile: Profile = match serde_json::from_str(&text) {
            Ok(profile) => profile,
            Err(err) => {
                // ⛔ Malformed is a REFUSAL, not "could not run". The bytes were
                // read and they do not describe a profile.
                println!("{path}: not a profile: {err}");
                worst = worst.max(1);
                continue;
            }
        };

        // ⚠ Well-formedness first. A coherence check over a profile whose id
        // disagrees with its keys is a check over something nobody can name.
        let defects = profile.check();
        if !defects.is_empty() {
            for defect in &defects {
                println!("{path}: malformed: {defect}");
            }
            worst = worst.max(1);
            continue;
        }

        let report = validate(&profile, &options);
        for (check, outcome) in &report.results {
            match outcome {
                Outcome::Passed => println!("{path}: ok    {check}"),
                Outcome::Failed(findings) => {
                    for f in findings {
                        println!("{path}: FAIL  {f}");
                    }
                }
                Outcome::NotCheckable(why) => println!("{path}: SKIP  {check} -- {why}"),
            }
        }
        let code = report.exit_code();
        worst = worst.max(u8::try_from(code).unwrap_or(1));
    }

    ExitCode::from(worst)
}
