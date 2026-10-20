//! Every program in this crate is run here exactly as its demo runs it, and
//! must exit 0 and print its `contract: … OK` line. A demo can only be
//! recorded if this suite is green.

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(bin_path: &str) -> String {
    let out = Command::new(bin_path)
        .current_dir(workspace_root())
        .output()
        .expect("spawn");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{bin_path} failed\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout
}

#[test]
fn committed_dataset_matches_the_generator() {
    let on_disk = std::fs::read_to_string(workspace_root().join(regression::DATA_PATH)).unwrap();
    let fresh = regression::to_csv(&regression::generate(regression::N_ROWS, regression::DATA_SEED));
    assert_eq!(on_disk, fresh, "data/reopen-hours.csv drifted from the generator");
}

#[test]
fn train_test_contract_holds() {
    let out = run(env!("CARGO_BIN_EXE_train-test"));
    assert!(out.contains("160 train / 40 test"), "{out}");
    assert!(out.contains("contract: honest evaluation"), "{out}");
}

#[test]
fn estimators_contract_holds() {
    let out = run(env!("CARGO_BIN_EXE_estimators"));
    for name in ["LinearRegression", "Ridge", "Lasso"] {
        assert!(out.contains(name), "{name} missing from:\n{out}");
    }
    assert!(out.contains("contract: one interface, one behaviour"), "{out}");
}
