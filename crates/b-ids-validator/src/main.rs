//! The command form of the validator.
//!
//! ⛔ **Exit 0 clean, 1 a check refused, 2 nothing could be checked.** Those are
//! three different facts, and a command that returned 1 for the last two would
//! hide the difference between "this profile is wrong" and "this run verified
//! nothing".
//!
//! ⚠ The `import` subcommand `VALID-02` names is not here yet. That entry is
//! open, and a subcommand that printed an empty report would be worse than an
//! absent one.

use std::collections::BTreeSet;
use std::process::ExitCode;

use b_ids_schema::Profile;
use b_ids_validator::{Options, Outcome, validate};

fn usage() -> &'static str {
    "usage: b-ids-validator [--publishing] [--decodes LIST] PROFILE.json...\n\
     \n\
       --publishing   refuse a vendor-provenance field, which a draft may carry\n\
       --decodes LIST comma-separated content encodings the consumer can decode\n\
     \n\
     exit 0 clean, 1 a check refused, 2 nothing could be checked."
}

fn main() -> ExitCode {
    let mut options = Options::default();
    let mut paths: Vec<String> = Vec::new();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
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
