//! Lesson 3.3.3: the HTTP API and the CLI give the same answer.
//!
//! Both binaries embed the same model through the same library code, so for
//! any text they must agree exactly. This runs the real CLI binary (built next
//! to this one) and posts the same text to the running API.
//!
//! Provable contract: one model, two front ends. For every sentence, the API's
//! JSON answer equals the CLI's, field for field.
//!
//! Usage: api-parity [127.0.0.1:8080]   (the API must already be running)

use std::process::Command;
use trail_status::bundle::Prediction;
use trail_status::http::request;

const SENTENCES: [&str; 4] = [
    "all trails are open",
    "blankets creek is open. rope mill remains closed.",
    "blankets creek is closed due to wet conditions. rope mill is open.",
    "all trails are closed due to freeze/thaw conditions",
];

fn main() {
    let addr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let cli = std::env::current_exe().expect("own path").with_file_name("trail-status-cli");

    let verdict = |open: bool| if open { "open" } else { "closed" };
    let mut agree = 0;
    for text in SENTENCES {
        let out = Command::new(&cli).args(["--json", text]).output().expect("run trail-status-cli");
        assert!(out.status.success(), "the CLI failed on {text:?}");
        let from_cli: Prediction = serde_json::from_slice(&out.stdout).expect("CLI JSON");

        let body = serde_json::json!({ "text": text }).to_string();
        let r = request(&addr, "POST", "/predict", Some(&body)).unwrap_or_else(|e| panic!("{e} (is the API running?)"));
        assert_eq!(r.status, 200, "the API refused {text:?}");
        let from_api: Prediction = serde_json::from_str(&r.body).expect("API JSON");

        let same = from_cli == from_api;
        agree += usize::from(same);
        println!("{text:?}");
        println!(
            "  {}  Blankets {:<6}  Rope {}",
            if same { "same" } else { "DIFF" },
            verdict(from_api.blankets_creek_open),
            verdict(from_api.rope_mill_open)
        );
    }

    assert_eq!(agree, SENTENCES.len(), "the API and the CLI must agree on every sentence");
    println!("contract: one model, two front ends (API == CLI on {agree}/{} sentences) OK", SENTENCES.len());
}
