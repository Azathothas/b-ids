//! Compare two capture files taken in two modes off one browser.
//!
//! ⭐ **A thin driver over `b_ids_harness::modes`, which is where the logic and
//! its tests live.** An example that carried the comparison itself would be a
//! second implementation nothing tests.
//!
//! ⛔ **It concludes nothing about a browser on its own.** It answers one
//! question: does the terminating capture surface change any TLS field the raw
//! surface can also see. `experiments/20-compare-capture-modes.sh` is what
//! produces the two files, off one resolved browser, in one run.
//!
//! ```text
//! cargo run -p b-ids-harness --example compare-modes -- RAW.jsonl TERMINATED.jsonl
//! ```
//!
//! Exit 0 nothing differed, 1 a field differed, 2 it could not run.

use std::process::ExitCode;

use b_ids_harness::modes::{Stability, Verdict};
use b_ids_harness::{Capture, compare};

/// Read the harness's `--json` output, skipping the base URL line it opens
/// with.
///
/// ⛔ **A line that is not a capture is reported, never dropped**, for the same
/// reason the corpus writer reports one: a reader that silently skipped it
/// would compare a smaller sample than the caller believes.
fn read(path: &str) -> Result<Vec<Capture>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !line.starts_with('{') {
            eprintln!("{path}: skipped a line that is not a capture: {line}");
            continue;
        }
        out.push(serde_json::from_str::<Capture>(line).map_err(|e| format!("{path}: {e}"))?);
    }
    Ok(out)
}

fn describe(stability: &Stability) -> String {
    match stability {
        Stability::Stable(value) => format!("stable {value}"),
        Stability::Varies { distinct } => format!("{distinct} distinct value(s) within the run"),
        Stability::Absent => "no connection carried it".to_owned(),
    }
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // ⭐ THE LABELS ARE THE CALLER'S, because this driver is not only about the raw
    // and terminating surfaces. `experiments/30-resumption-control.sh` compares two
    // TERMINATING runs whose ticket policy differs, and a report calling one of them
    // `raw` would be the "display that lies" row of
    // docs/conventions/forbidden-patterns.md.
    let mut labels = ("raw".to_owned(), "terminated".to_owned());
    if let Some(i) = args.iter().position(|a| a == "--labels") {
        let Some(value) = args.get(i + 1).cloned() else {
            eprintln!("compare-modes: --labels needs A,B");
            return ExitCode::from(2);
        };
        let Some((a, b)) = value.split_once(',') else {
            eprintln!("compare-modes: --labels takes two names separated by a comma");
            return ExitCode::from(2);
        };
        if a.is_empty() || b.is_empty() {
            eprintln!("compare-modes: --labels needs a name on both sides of the comma");
            return ExitCode::from(2);
        }
        labels = (a.to_owned(), b.to_owned());
        args.drain(i..=i + 1);
    }
    let (label_a, label_b) = labels;
    let [raw_path, terminated_path] = args.as_slice() else {
        eprintln!(
            "usage: compare-modes [--labels A,B] RAW.jsonl TERMINATED.jsonl\n\
             \n\
             Both files are what `b-ids-harness --json` printed, the first from a run with\n\
             --raw and the second from a run with --ca-out, off the SAME browser and build."
        );
        return ExitCode::from(2);
    };

    let (raw, terminated) = match (read(raw_path), read(terminated_path)) {
        (Ok(r), Ok(t)) => (r, t),
        (Err(why), _) | (_, Err(why)) => {
            eprintln!("compare-modes: {why}");
            return ExitCode::from(2);
        }
    };

    // ⛔ SELECTED, not handed over whole. A completed handshake leaves a session
    // to resume, so later connections of the terminating run offer a pre-shared
    // key and a different extension set; the raw surface completes nothing and
    // can never produce one. Comparing those two sets answers a question about
    // resumption while looking like a question about the capture mode.
    let raw_split = b_ids_harness::resumption_split(&raw);
    let terminated_split = b_ids_harness::resumption_split(&terminated);
    println!(
        "{label_a}: {} cold, {} resumed, {} with no http2",
        raw_split.cold, raw_split.resumed, raw_split.no_http2
    );
    println!(
        "{label_b}: {} cold, {} resumed, {} with no http2",
        terminated_split.cold, terminated_split.resumed, terminated_split.no_http2
    );
    // ⭐ A finding in its own right, and it belongs above the field list rather
    // than inside it: only a surface that completes a handshake can produce a
    // resumption at all.
    if raw_split.resumed != terminated_split.resumed {
        println!(
            "⚠ {label_a} and {label_b} produced different numbers of resumed connections, \
             which is an effect on the RUN even where every field of the cold hello agrees"
        );
    }

    let raw: Vec<Capture> = b_ids_harness::comparable(&raw)
        .into_iter()
        .cloned()
        .collect();
    let terminated: Vec<Capture> = b_ids_harness::comparable(&terminated)
        .into_iter()
        .cloned()
        .collect();
    let comparison = compare(&raw, &terminated);
    println!(
        "\ncomparing {} {label_a} hello(s) against {} {label_b}, resumed connections excluded",
        comparison.raw_hellos, comparison.terminated_hellos
    );
    if comparison.raw_hellos == 0 || comparison.terminated_hellos == 0 {
        // ⛔ 2, not 0. A comparison with nothing on one side compared nothing,
        // which is a different fact from a comparison that found no difference.
        eprintln!("compare-modes: one side carried no ClientHello at all, so nothing was compared");
        return ExitCode::from(2);
    }
    if comparison.thin() {
        // ⚠ Printed rather than fatal. One connection per mode cannot establish
        // stability, so every field reads as stable and the comparison is
        // weaker than it looks. Saying so is the whole handling.
        println!(
            "⚠ thin: stability needs at least two connections per mode, and one side has fewer"
        );
    }

    for field in &comparison.fields {
        match &field.verdict {
            Verdict::Agrees(value) => println!("  agrees        {}  {value}", field.field),
            Verdict::Differs { raw, terminated } => {
                println!("  DIFFERS       {}", field.field);
                println!("      {label_a}  {raw}");
                println!("      {label_b}  {terminated}");
            }
            Verdict::NotComparable { raw, terminated } => {
                println!("  not comparable {}", field.field);
                println!("      {label_a}  {}", describe(raw));
                println!("      {label_b}  {}", describe(terminated));
            }
        }
    }

    // ⛔ A statement about the COMPARISON rather than about either file. The
    // TLS half is the only half both sides can be assumed to carry, so these
    // fields are never compared here whatever the two runs were.
    println!(
        "\nnot compared, because only a terminating surface carries them: {}",
        comparison.only_terminated_sees.join(", ")
    );
    let differing = comparison.differing().len();
    let not_comparable = comparison.not_comparable().len();
    println!(
        "modes={} differing:{differing} not_comparable:{not_comparable} fields:{}",
        if differing == 0 { "agree" } else { "differ" },
        comparison.fields.len()
    );
    if differing == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
