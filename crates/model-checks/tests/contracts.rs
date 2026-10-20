//! Every checker is run here on REAL output saved from `apr` and `apr serve`
//! (`fixtures/`), so the contracts are tested with no model and no server.
//! Each must pass on the good fixture and refuse a broken one.

use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(name).to_string_lossy().into_owned()
}

fn run(bin: &str, args: &[&str]) -> Output {
    Command::new(bin).args(args).output().expect("spawn")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// Write a broken copy of a fixture into a temp file and return its path.
fn broken(name: &str, from: &str, to: &str) -> String {
    let text = std::fs::read_to_string(fixture(name)).unwrap();
    assert!(text.contains(from), "{name} does not contain {from:?}");
    let path = std::env::temp_dir().join(format!("model-checks-broken-{}-{name}", std::process::id()));
    std::fs::write(&path, text.replacen(from, to, 1)).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn anatomy_adds_up_on_the_real_model() {
    let o = run(env!("CARGO_BIN_EXE_anatomy-check"), &[&fixture("inspect.json"), &fixture("tensors.json")]);
    assert!(o.status.success(), "{}", stdout(&o));
    assert!(stdout(&o).contains("494,032,768"));
    assert!(stdout(&o).contains("contract: parameters add up"));
}

#[test]
fn anatomy_refuses_a_wrong_total() {
    let bad = broken("inspect.json", "494032768", "494032769");
    let o = run(env!("CARGO_BIN_EXE_anatomy-check"), &[&bad, &fixture("tensors.json")]);
    assert!(!o.status.success());
}

#[test]
fn chat_shape_holds_on_a_real_response() {
    let o = run(env!("CARGO_BIN_EXE_chat-check"), &[&fixture("rust-1.json")]);
    assert!(o.status.success(), "{}", stdout(&o));
    assert!(stdout(&o).contains("contract: OpenAI response shape"));
}

#[test]
fn chat_check_refuses_bad_token_accounting() {
    let bad = broken("rust-1.json", "\"total_tokens\":47", "\"total_tokens\":48");
    assert!(!run(env!("CARGO_BIN_EXE_chat-check"), &[&bad]).status.success());
}

#[test]
fn chat_check_refuses_a_non_assistant_role() {
    let bad = broken("rust-1.json", "\"role\":\"assistant\"", "\"role\":\"user\"");
    assert!(!run(env!("CARGO_BIN_EXE_chat-check"), &[&bad]).status.success());
}

#[test]
fn serving_is_deterministic_on_real_responses() {
    let o = run(
        env!("CARGO_BIN_EXE_serve-check"),
        &[&fixture("rust-1.json"), &fixture("rust-2.json"), &fixture("rust-stream.sse")],
    );
    assert!(o.status.success(), "{}", stdout(&o));
    assert!(stdout(&o).contains("contract: greedy determinism"));
}

#[test]
fn serve_check_refuses_a_different_second_answer() {
    let bad = broken("rust-2.json", "open-source", "closed-source");
    let o = run(env!("CARGO_BIN_EXE_serve-check"), &[&fixture("rust-1.json"), &bad, &fixture("rust-stream.sse")]);
    assert!(!o.status.success());
}

#[test]
fn metrics_show_requests_served() {
    let o = run(env!("CARGO_BIN_EXE_metrics-delta"), &[&fixture("metrics-before.prom"), &fixture("metrics-after.prom")]);
    assert!(o.status.success(), "{}", stdout(&o));
    assert!(stdout(&o).contains("contract: answered locally"));
}

#[test]
fn metrics_delta_refuses_when_nothing_was_served() {
    let p = fixture("metrics-before.prom");
    assert!(!run(env!("CARGO_BIN_EXE_metrics-delta"), &[&p, &p]).status.success());
}
