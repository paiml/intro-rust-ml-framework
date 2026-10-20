//! Lesson 2.2.4: trade size for quality, and measure both sides of the trade.
//!
//! Runs the SAME greedy prompt (temperature 0, so the answer is repeatable)
//! through the baseline model and each quantized copy, with `apr run`, and
//! prints one line per model as it finishes (all models run at once): size,
//! share of the baseline, the answer, and how long the run took.
//!
//! Provable contract: smaller and still right. Every quantized file is smaller
//! than the baseline AND gives the baseline's answer to the same prompt.
//!
//! Usage: quant-compare [<baseline> <quantized.apr>...]
//!   With no arguments it compares the lesson's files: the original
//!   `models/qwen2.5-0.5b-instruct/model.safetensors` against
//!   `models/quant/qwen-int8.apr` and `models/quant/qwen-int4.apr`
//!   (made with `apr import ... --quantize int8|int4`).

use model_checks::{check_quantized, parse_run_output, QuantRow};
use std::process::Command;
use std::time::Instant;

const PROMPT: &str = "What is the capital of France? Answer in one word.";
const LESSON_MODELS: [&str; 3] = [
    "models/qwen2.5-0.5b-instruct/model.safetensors",
    "models/quant/qwen-int8.apr",
    "models/quant/qwen-int4.apr",
];

fn run(model: &str) -> (QuantRow, f64) {
    let size_bytes = std::fs::metadata(model).unwrap_or_else(|e| panic!("{model}: {e}")).len();
    let start = Instant::now();
    let out = Command::new("apr")
        .args(["run", model, "-p", PROMPT, "-n", "12", "--chat", "--temperature", "0", "--no-gpu"])
        .output()
        .expect("spawn apr");
    let secs = start.elapsed().as_secs_f64();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "apr run {model} failed:\n{}", String::from_utf8_lossy(&out.stderr));
    let answer = parse_run_output(&stdout).unwrap_or_else(|| panic!("no answer from {model}:\n{stdout}"));
    let name = model.rsplit('/').next().unwrap_or(model).to_string();
    (QuantRow { name, size_bytes, answer }, secs)
}

fn main() {
    let mut models: Vec<String> = std::env::args().skip(1).collect();
    if models.is_empty() {
        models = LESSON_MODELS.iter().map(|m| m.to_string()).collect();
    }
    let (baseline_path, quantized_paths) = models.split_first().expect("at least the baseline");
    if quantized_paths.is_empty() {
        eprintln!("usage: quant-compare [<baseline> <quantized.apr>...]");
        std::process::exit(2);
    }
    println!("prompt: {PROMPT:?} (greedy)\n");
    println!("{:<34} {:>8} {:>6}  {:<8} {:>6}", "model", "MiB", "size", "answer", "time");

    // All models load at once; each row prints the moment its model answers.
    let (tx, rx) = std::sync::mpsc::channel();
    for (i, path) in models.iter().enumerate() {
        let (tx, path) = (tx.clone(), path.clone());
        std::thread::spawn(move || tx.send((i, run(&path))).expect("main thread is listening"));
    }
    drop(tx);

    let mib = |b: u64| b as f64 / (1024.0 * 1024.0);
    let baseline_size = std::fs::metadata(baseline_path).expect("baseline").len();
    let mut results: Vec<Option<QuantRow>> = vec![None; models.len()];
    for (i, (row, secs)) in rx {
        let share = 100.0 * row.size_bytes as f64 / baseline_size as f64;
        println!("{:<34} {:>8.0} {:>5.0}%  {:<8} {:>5.1}s", row.name, mib(row.size_bytes), share, row.answer, secs);
        results[i] = Some(row);
    }
    println!();
    let mut results = results.into_iter().map(|r| r.expect("every model answered"));
    let baseline = results.next().expect("baseline");
    let rows: Vec<QuantRow> = results.collect();
    assert_eq!(rows.len(), quantized_paths.len());

    let verdict = check_quantized(&baseline, &rows);
    assert!(verdict.is_ok(), "{}", verdict.unwrap_err());
    println!(
        "contract: smaller and still right ({} quantized models smaller than the baseline, same answer {:?}) OK",
        rows.len(),
        baseline.answer
    );
}
