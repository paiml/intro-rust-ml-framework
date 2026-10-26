//! Lesson 2.2.2: read a model's anatomy from `apr inspect` and `apr tensors`.
//!
//! Provable contract: the parameters add up. Multiplying out the shape of
//! every tensor `apr tensors` lists, and adding them, gives exactly the total
//! `apr inspect` reports; the tensor counts agree; and every tensor's size in
//! bytes is its parameter count times the width of its dtype.
//!
//! Usage: anatomy-check <inspect.json> <tensors.json>
//!   (from `apr inspect --json <model>` and `apr tensors --json <model>`)

use model_checks::{bytes_per_value, component, layer_count, parse_inspect, parse_tensors, Tensor};
use std::collections::BTreeMap;

fn grouped(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, inspect_path, tensors_path] = args.as_slice() else {
        eprintln!("usage: anatomy-check <inspect.json> <tensors.json>");
        std::process::exit(2);
    };
    let read = |p: &String| std::fs::read_to_string(p).unwrap_or_else(|e| panic!("read {p}: {e}"));
    let summary = parse_inspect(&read(inspect_path)).expect("inspect JSON");
    let tensors = parse_tensors(&read(tensors_path)).expect("tensors JSON");

    let total: u64 = tensors.iter().map(Tensor::params).sum();
    let mut by_part: BTreeMap<&str, u64> = BTreeMap::new();
    for t in &tensors {
        *by_part.entry(component(&t.name)).or_default() += t.params();
    }

    println!(
        "architecture {}: {} tensors, {} layers",
        summary.architecture,
        tensors.len(),
        layer_count(&tensors)
    );
    println!("where the parameters live:");
    for (part, n) in &by_part {
        println!(
            "  {part:<10} {:>13}  {:5.1}%",
            grouped(*n),
            100.0 * *n as f64 / total as f64
        );
    }
    println!("sum of tensor shapes  {:>13}", grouped(total));
    println!(
        "apr inspect says      {:>13}",
        grouped(summary.total_params)
    );

    assert_eq!(
        total, summary.total_params,
        "tensor shapes must add up to the reported total"
    );
    assert_eq!(
        tensors.len() as u64,
        summary.tensor_count,
        "tensor counts must agree"
    );
    for t in &tensors {
        let width = bytes_per_value(&t.dtype)
            .unwrap_or_else(|| panic!("{}: unknown dtype {}", t.name, t.dtype));
        assert_eq!(
            t.size_bytes,
            t.params() * width,
            "{}: bytes != params x dtype width",
            t.name
        );
    }
    println!(
        "contract: parameters add up ({} tensor shapes sum to {}, bytes = params x dtype width) OK",
        tensors.len(),
        grouped(total)
    );
}
