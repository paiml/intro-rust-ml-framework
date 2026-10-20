//! Every checker is run here the way its demo runs it, and must exit 0 and
//! print its `contract: … OK` line.
//!
//! These checkers hold REAL tools to a contract, so the tests need what the
//! demos need: network for `install-options`, an `apr` 0.71.4 for
//! `verify-install`, and the model for `run-check`. A missing prerequisite is
//! printed as NOT RUN, never passed silently; set REQUIRE_ALL=1 to make every
//! NOT RUN a failure.

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn not_run(why: &str) {
    assert!(std::env::var_os("REQUIRE_ALL").is_none(), "NOT RUN with REQUIRE_ALL set: {why}");
    eprintln!("NOT RUN: {why}");
}

fn assert_contract(mut cmd: Command, contract: &str) {
    let out = cmd.current_dir(workspace_root()).output().expect("spawn");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success() && stdout.contains(contract),
        "missing {contract:?}\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The real apr this machine has, if it is the pinned version.
fn pinned_apr() -> Option<PathBuf> {
    let out = Command::new("apr").arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    if install_checks::apr_version(&text) != Some(install_checks::PINNED_VERSION) {
        return None;
    }
    let path = std::env::var("PATH").ok()?;
    install_checks::first_on_path("apr", &path)
}

#[test]
fn install_options_contract_holds() {
    if Command::new("curl").args(["-fsS", "-o", "/dev/null", "https://api.github.com"]).status().map_or(true, |s| !s.success()) {
        return not_run("install-options needs network access to crates.io and GitHub");
    }
    assert_contract(Command::new(env!("CARGO_BIN_EXE_install-options")), "contract: one release, three channels");
}

#[test]
fn verify_install_contract_holds_for_the_installed_apr() {
    let Some(apr) = pinned_apr() else {
        return not_run("verify-install needs apr 0.71.4 on PATH");
    };
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_verify-install"));
    cmd.arg(&apr);
    assert_contract(cmd, "contract: installed and resolvable");
}

#[test]
fn verify_install_refuses_a_shadowed_binary() {
    let Some(apr) = pinned_apr() else {
        return not_run("verify-install needs apr 0.71.4 on PATH");
    };
    // A copy of apr that is NOT what PATH resolves to must be refused.
    let dir = std::env::temp_dir().join(format!("shadow-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let copy = dir.join("apr");
    std::fs::copy(&apr, &copy).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_verify-install"))
        .arg(&copy)
        .env("PATH", format!("{}:/usr/bin:/bin", apr.parent().unwrap_or(Path::new("/")).display()))
        .output()
        .unwrap();
    std::fs::remove_file(&copy).unwrap();
    std::fs::remove_dir(&dir).unwrap();
    assert!(!out.status.success(), "a shadowed install must fail the contract");
}

#[test]
fn run_check_contract_holds() {
    let model = workspace_root().join("models/qwen2.5-0.5b-instruct.safetensors");
    if !model.exists() || pinned_apr().is_none() {
        return not_run("run-check needs apr 0.71.4 and `make models`");
    }
    assert_contract(Command::new(env!("CARGO_BIN_EXE_run-check")), "contract: greedy decoding is deterministic");
}
