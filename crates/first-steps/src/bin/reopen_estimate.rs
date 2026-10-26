//! Lesson 1.3.4: a small application built on the library.
//!
//! `reopen-estimate --rain <mm> --sun <hours>` answers one question: after
//! this storm, how long until the trail reopens? It loads the model saved in
//! lesson 1.3.3 and refuses inputs outside the range the model was trained on.
//!
//! Provable contract: a trustworthy answer. The estimate is exactly the saved
//! model's formula, and more rain never means an earlier reopen (one more
//! millimetre always adds time).
//!
//! Run from the workspace root (after `save-model`):
//!   cargo run -p first-steps --bin reopen-estimate -- --rain 20 --sun 4

use first_steps::{by_formula, check_inputs, estimate, load, MODEL_PATH};
use std::process::ExitCode;

/// Read `--rain <mm> --sun <hours>`.
fn parse_args(args: &[String]) -> Result<(f32, f32), String> {
    let mut rain = None;
    let mut sun = None;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next().ok_or(format!("{flag} needs a value"))?;
        let number: f32 = value
            .parse()
            .map_err(|_| format!("{flag}: {value:?} is not a number"))?;
        match flag.as_str() {
            "--rain" => rain = Some(number),
            "--sun" => sun = Some(number),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    Ok((
        rain.ok_or("missing --rain <mm>")?,
        sun.ok_or("missing --sun <hours>")?,
    ))
}

/// Keep a parsed `(rain, sun)` only if the model was trained on inputs like it.
fn checked((rain, sun): (f32, f32)) -> Result<(f32, f32), String> {
    check_inputs(rain, sun).map(|()| (rain, sun))
}

/// Say why an input was refused, and exit 2 without an estimate.
fn refuse(why: &str) -> ExitCode {
    eprintln!("refused: {why}");
    eprintln!("usage: reopen-estimate --rain <0-40 mm> --sun <0-10 hours>");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Step 1: read the flags, and refuse what the training data never covered.
    let (rain, sun) = match parse_args(&args).and_then(checked) {
        Ok(inputs) => inputs,
        Err(why) => return refuse(&why),
    };

    // Step 2: load the model saved in lesson 1.3.3, and ask it.
    let model = load(MODEL_PATH).expect("load the model (run `save-model` first)");
    let hours = estimate(&model, rain, sun);
    let days = hours / 24.0;

    // Step 3: answer in hours and in days.
    let unit = if sun == 1.0 { "hour" } else { "hours" };
    println!("{rain} mm of rain, {sun} {unit} of sun");
    println!("estimate: the trail reopens in about {hours:.0} hours ({days:.1} days)");

    // Step 4: the contract. The app checks its answer before you see it.
    let formula = by_formula(&model, rain, sun);
    assert!(
        (hours - formula).abs() < 1e-3,
        "estimate must be the model's formula"
    );
    assert!(
        estimate(&model, rain + 1.0, sun) > hours,
        "more rain must reopen later"
    );
    println!(
        "contract: trustworthy answer (the saved model's formula; more rain, never sooner) OK"
    );
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reads_both_flags_in_any_order() {
        assert_eq!(
            parse_args(&args(&["--sun", "4", "--rain", "20"])),
            Ok((20.0, 4.0))
        );
    }

    #[test]
    fn refuses_missing_and_bad_values() {
        assert!(parse_args(&args(&["--rain", "20"])).is_err());
        assert!(parse_args(&args(&["--rain", "lots", "--sun", "4"])).is_err());
        assert!(parse_args(&args(&["--rain"])).is_err());
        assert!(parse_args(&args(&["--snow", "1", "--sun", "4"])).is_err());
    }
}
