//! Every program in this crate is run here the way its demo runs it, and must
//! exit 0 and print its `contract: … OK` line. A demo is only recorded from a
//! green suite.

use std::path::PathBuf;
use std::process::{Child, Command, Output};
use std::time::Duration;
use trail_status::http::wait_for_server;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(bin: &str, args: &[&str]) -> String {
    let out: Output = Command::new(bin)
        .args(args)
        .current_dir(workspace_root())
        .output()
        .expect("spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{bin} failed\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

#[test]
fn explore_contract_holds() {
    let out = run(env!("CARGO_BIN_EXE_explore"), &[]);
    assert!(out.contains("rows: train 401   val 70"), "{out}");
    assert!(
        out.contains("contract: the data is fit to learn from"),
        "{out}"
    );
}

#[test]
fn compare_contract_holds_and_picks_balanced_logistic() {
    let out = run(env!("CARGO_BIN_EXE_compare"), &[]);
    assert!(out.contains("chosen: logistic, balanced"), "{out}");
    assert!(out.contains("contract: the choice is earned"), "{out}");
}

#[test]
fn train_contract_holds_without_touching_the_committed_model() {
    let out_path =
        std::env::temp_dir().join(format!("trail-train-test-{}.apr", std::process::id()));
    let out = run(env!("CARGO_BIN_EXE_train"), &[out_path.to_str().unwrap()]);
    let fresh = std::fs::read(&out_path).expect("the trained file");
    let _ = std::fs::remove_file(&out_path);
    assert!(
        fresh == trail_status::model::MODEL_BYTES,
        "training again must write the committed model's exact bytes"
    );
    assert!(
        out.contains("70/70 validation predictions identical"),
        "{out}"
    );
    assert!(
        out.contains("contract: the saved file is the model"),
        "{out}"
    );
}

#[test]
fn committed_model_predicts_what_a_fresh_training_predicts() {
    // A fresh training is the committed model, so their predictions must be identical.
    let committed =
        trail_status::bundle::TrailStatusBundle::from_bytes(trail_status::model::MODEL_BYTES)
            .unwrap();
    std::env::set_current_dir(workspace_root()).unwrap();
    let train = trail_status::data::load(trail_status::data::TRAIN_PATH).unwrap();
    let val = trail_status::data::load(trail_status::data::VAL_PATH).unwrap();
    let fresh = trail_status::bundle::TrailStatusBundle::train(&train).unwrap();
    let texts: Vec<&str> = val.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(committed.predict_many(&texts), fresh.predict_many(&texts));
}

#[test]
fn cli_answers_both_trails() {
    let out = run(
        env!("CARGO_BIN_EXE_trail-status-cli"),
        &["blankets creek is open. rope mill remains closed."],
    );
    assert!(out.contains("Blankets Creek: OPEN"), "{out}");
    assert!(out.contains("Rope Mill:      CLOSED"), "{out}");
}

#[test]
fn embed_check_contract_holds() {
    let out = run(env!("CARGO_BIN_EXE_embed-check"), &[]);
    assert!(
        out.contains("contract: the bytes travel with the binary"),
        "{out}"
    );
}

struct Server(Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn api_keeps_its_documented_contract_and_matches_the_cli() {
    let port = free_port().to_string();
    let addr = format!("127.0.0.1:{port}");
    let _server = Server(
        Command::new(env!("CARGO_BIN_EXE_trail-status-api"))
            .args(["--port", &port])
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("start the API"),
    );
    wait_for_server(&addr, Duration::from_secs(10)).unwrap();

    let out = run(env!("CARGO_BIN_EXE_api-check"), &[&addr]);
    assert!(
        out.contains("contract: the API keeps its word (7/7"),
        "{out}"
    );
    let out = run(env!("CARGO_BIN_EXE_api-parity"), &[&addr]);
    assert!(
        out.contains("contract: one model, two front ends (API == CLI on 4/4"),
        "{out}"
    );
}

/// The two `apr compile` lessons need apr and the prepared model files
/// (lesson 3.1.3: the int8 import and the `apr compile` into models/compile/).
/// Skip, saying so, where they are absent.
fn compile_fixtures() -> Option<(PathBuf, PathBuf)> {
    let root = workspace_root();
    let model = root.join("models/compile/qwen2.5-0.5b-instruct-int8.apr");
    let program = root.join("models/compile/qwen-chat");
    let have_apr = Command::new("apr")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if have_apr && model.exists() && program.exists() {
        Some((model, program))
    } else {
        eprintln!("NOTE: skipping apr compile contracts: follow lesson 3.1.3 to create models/compile/qwen-chat first");
        None
    }
}

/// One test, not two: standalone-check moves the model file aside while it
/// runs, so the format check must not read it at the same moment.
#[test]
fn apr_compile_contracts_hold() {
    let Some((model, program)) = compile_fixtures() else {
        return;
    };
    let out = run(
        env!("CARGO_BIN_EXE_apr-format-check"),
        &[
            "crates/trail-status/models/trail_status_model.apr",
            model.to_str().unwrap(),
        ],
    );
    assert!(out.contains("contract: compile takes APR v2 only"), "{out}");
    let out = run(
        env!("CARGO_BIN_EXE_standalone-check"),
        &[program.to_str().unwrap(), model.to_str().unwrap()],
    );
    assert!(
        out.contains("contract: the model travels inside the program"),
        "{out}"
    );
}
