//! Lesson 1.2.5: check a greedy model run is repeatable and right.
//!
//! Provable contract: greedy decoding is deterministic. With temperature 0,
//! asking the model the same question twice gives the identical answer, and
//! that answer to "What is 2+2?" contains 4.
//!
//! Usage: run-check [model]
//!   cargo run -p install-checks --bin run-check -- models/qwen2.5-0.5b-instruct.safetensors

use install_checks::{run, run_answer};

const DEFAULT_MODEL: &str = "models/qwen2.5-0.5b-instruct.safetensors";
const PROMPT: &str = "What is 2+2? Answer with just the number.";

fn ask(model: &str) -> String {
    let out = run(
        "apr",
        &["run", model, "-p", PROMPT, "-n", "8", "--chat", "--temperature", "0", "--no-gpu"],
    )
    .expect("apr run");
    run_answer(&out).expect("apr run printed an answer")
}

fn main() {
    let model = std::env::args().nth(1).unwrap_or_else(|| DEFAULT_MODEL.to_string());
    println!("prompt:  {PROMPT:?}  (temperature 0, run twice)");
    let first = ask(&model);
    let second = ask(&model);
    println!("run 1:   {first:?}");
    println!("run 2:   {second:?}");

    assert_eq!(first, second, "greedy decoding must repeat itself exactly");
    assert!(first.contains('4'), "2+2 must come back as 4, got {first:?}");
    println!("contract: greedy decoding is deterministic (same prompt, identical answer, and it is 4) OK");
}
