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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [raw_path, terminated_path] = args.as_slice() else {
        eprintln!(
            "usage: compare-modes RAW.jsonl TERMINATED.jsonl\n\
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
        "raw:        {} cold, {} resumed, {} abandoned",
        raw_split.cold, raw_split.resumed, raw_split.abandoned
    );
    println!(
        "terminated: {} cold, {} resumed, {} abandoned",
        terminated_split.cold, terminated_split.resumed, terminated_split.abandoned
    );
    // ⭐ A finding in its own right, and it belongs above the field list rather
    // than inside it: only a surface that completes a handshake can produce a
    // resumption at all.
    if raw_split.resumed != terminated_split.resumed {
        println!(
            "⚠ the two surfaces produced different numbers of resumed connections, which is a \
             mode effect on the RUN even where every field of the cold hello agrees"
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
        "\ncomparing {} raw hello(s) against {} terminated, resumed connections excluded",
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
                println!("      raw         {raw}");
                println!("      terminated  {terminated}");
            }
            Verdict::NotComparable { raw, terminated } => {
                println!("  not comparable {}", field.field);
                println!("      raw         {}", describe(raw));
                println!("      terminated  {}", describe(terminated));
            }
        }
    }

    println!(
        "\nonly the terminating surface can see: {}",
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
