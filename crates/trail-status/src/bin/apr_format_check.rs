//! Lesson 3.1.2: which `.apr` files can `apr compile` turn into a program?
//!
//! An `.apr` file starts with a 4-byte magic number. `aprender::format::save`
//! (classical models such as our trail-status bundle) writes APR v1, `APRN`.
//! `apr import` (tensor models such as an LLM) writes APR v2, `APR\0`.
//! `apr compile` only accepts v2.
//!
//! Provable contract: compile takes APR v2 only. The classical bundle carries
//! the v1 magic and `apr compile` refuses it; the imported LLM carries the v2
//! magic.
//!
//! Usage: apr-format-check <v1-bundle.apr> <imported-v2.apr>

use std::process::Command;

fn magic(path: &str) -> [u8; 4] {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    bytes[..4].try_into().expect("an .apr file has at least 4 bytes")
}

fn show(m: [u8; 4]) -> String {
    m.iter().map(|&b| if b == 0 { "\\0".to_string() } else { (b as char).to_string() }).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, v1, v2] = args.as_slice() else {
        eprintln!("usage: apr-format-check <v1-bundle.apr> <imported-v2.apr>");
        std::process::exit(2);
    };
    let (m1, m2) = (magic(v1), magic(v2));
    println!("{v1:<55} magic {:<5} APR v1 (aprender::format::save)", show(m1));
    println!("{v2:<55} magic {:<5} APR v2 (apr import)", show(m2));

    let out_path = std::env::temp_dir().join(format!("apr-format-check-{}", std::process::id()));
    let refused = Command::new("apr").args(["compile", v1, "-o"]).arg(&out_path).output().expect("run apr compile");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    println!("apr compile {v1}: {}", stderr.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim());

    assert_eq!(&m1, b"APRN", "the classical bundle must be APR v1");
    assert_eq!(&m2, b"APR\0", "the imported model must be APR v2");
    assert!(!refused.status.success() && stderr.contains("APR v2"), "apr compile must refuse APR v1");
    assert!(!out_path.exists(), "a refused compile must write nothing");
    println!("contract: compile takes APR v2 only (v1 bundle refused, imported model is v2) OK");
}
