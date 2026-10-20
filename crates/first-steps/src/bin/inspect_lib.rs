//! Lesson 1.3.3, step 2: read the same model file with the library, and
//! compare what the library sees with what the `apr` CLI reports.
//!
//! Provable contract: one file, one model. `apr tensors --json` and the
//! library agree on the tensors in the file (names and shapes), on the
//! parameter count, and the library's loaded model predicts exactly what its
//! own formula says.
//!
//! Run from the workspace root (after `save-model`):
//!   cargo run -p first-steps --bin inspect-lib

use first_steps::{by_formula, estimate, load, MODEL_PATH};
use std::process::Command;

/// `(name, shape)` for every tensor `apr tensors --json` lists.
fn cli_tensors(path: &str) -> Vec<(String, Vec<usize>)> {
    let out = Command::new("apr")
        .args(["tensors", "--json", path])
        .output()
        .expect("run `apr tensors`");
    assert!(out.status.success(), "apr tensors failed: {}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("apr tensors --json is JSON");
    json["tensors"]
        .as_array()
        .expect("a tensors array")
        .iter()
        .map(|t| {
            let name = t["name"].as_str().expect("name").to_string();
            let shape = t["shape"].as_array().expect("shape").iter().map(|d| d.as_u64().unwrap() as usize).collect();
            (name, shape)
        })
        .collect()
}

fn main() {
    // Step 1: the library loads the file as a real LinearRegression.
    let model = load(MODEL_PATH).expect("load (run `save-model` first)");
    let coefficients = model.coefficients().as_slice();
    println!("library: loaded a LinearRegression from {MODEL_PATH}");
    println!("         coefficients {coefficients:?}, intercept {:.4}", model.intercept());
    let library_tensors = vec![
        ("coefficients".to_string(), vec![coefficients.len()]),
        ("intercept".to_string(), vec![1]),
    ];

    // Step 2: ask the apr CLI what it sees in the same file.
    let cli = cli_tensors(MODEL_PATH);
    let params: usize = cli.iter().map(|(_, s)| s.iter().product::<usize>()).sum();
    println!("apr CLI: {} tensors: {cli:?}", cli.len());

    // Step 3: the library can do what the CLI cannot: run this model.
    let hours = estimate(&model, 20.0, 4.0);
    let formula = by_formula(&model, 20.0, 4.0);
    println!("predict: 20 mm of rain, 4 sun hours -> {hours:.1} hours until reopen");

    // Step 4: the contract. Both views must agree.
    assert_eq!(cli, library_tensors, "CLI and library must see the same tensors");
    assert_eq!(params, coefficients.len() + 1, "parameter counts must agree");
    assert!((hours - formula).abs() < 1e-3, "predict must follow the formula");
    println!("contract: one file, one model (CLI and library agree on {params} parameters in {} tensors) OK", cli.len());
}
