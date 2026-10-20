//! Every program in this crate is run here exactly as its demo runs it, and
//! must exit 0 and print its `contract: … OK` line. The steps run in order in
//! one test, because `inspect-lib` and `reopen-estimate` read the file
//! `save-model` writes.

use std::path::PathBuf;
use std::process::{Command, Output};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(bin: &str, args: &[&str]) -> Output {
    Command::new(bin).args(args).current_dir(workspace_root()).output().expect("spawn")
}

fn ok_stdout(bin: &str, args: &[&str]) -> String {
    let out = run(bin, args);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{bin} {args:?} failed\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

fn apr_on_path() -> bool {
    Command::new("apr").arg("--version").output().is_ok_and(|o| o.status.success())
}

#[test]
fn save_inspect_and_estimate_contracts_hold() {
    let saved = ok_stdout(env!("CARGO_BIN_EXE_save-model"), &[]);
    assert!(saved.contains("contract: lossless save"), "{saved}");

    if apr_on_path() {
        let inspected = ok_stdout(env!("CARGO_BIN_EXE_inspect-lib"), &[]);
        assert!(inspected.contains("contract: one file, one model"), "{inspected}");
    } else {
        // Refuse to pass silently: say what was not run.
        eprintln!("NOT RUN: inspect-lib needs `apr` on PATH");
    }

    let estimate = ok_stdout(env!("CARGO_BIN_EXE_reopen-estimate"), &["--rain", "20", "--sun", "4"]);
    assert!(estimate.contains("contract: trustworthy answer"), "{estimate}");
}

#[test]
fn estimate_refuses_out_of_range_input_with_exit_2() {
    let out = run(env!("CARGO_BIN_EXE_reopen-estimate"), &["--rain", "-3", "--sun", "4"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("refused:"));
    assert!(out.stdout.is_empty(), "a refused input must not print an estimate");
}
