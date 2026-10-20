//! Lesson 3.1.4: prove the CLI carries its model inside it.
//!
//! Provable contract: the bytes travel with the binary. The model embedded by
//! `include_bytes!` has the same SHA-256 as the `.apr` file it was built from,
//! and a copy of the CLI, run from an empty directory with no model file
//! anywhere near it, still answers.
//!
//! Usage (from the workspace root, after building the CLI):
//!   cargo run --release -p trail-status --bin embed-check

use std::process::Command;
use trail_status::model::MODEL_BYTES;
use trail_status::{sha256_hex, MODEL_PATH};

fn main() {
    let embedded = sha256_hex(MODEL_BYTES);
    let on_disk = sha256_hex(&std::fs::read(MODEL_PATH).expect("the model file"));
    println!("model embedded in this program: {} bytes", MODEL_BYTES.len());
    println!("embedded   sha256 {embedded}");
    println!("model file sha256 {on_disk}");

    // Copy the CLI into an empty directory and run it there.
    let cli = std::env::current_exe().expect("own path").with_file_name("trail-status-cli");
    let empty = std::env::temp_dir().join(format!("embed-check-{}", std::process::id()));
    std::fs::create_dir_all(&empty).expect("make empty dir");
    let copy = empty.join("trail-status-cli");
    std::fs::copy(&cli, &copy).expect("copy the CLI");
    let entries = std::fs::read_dir(&empty).expect("list").count();
    let out = Command::new(&copy)
        .arg("blankets creek is open. rope mill remains closed.")
        .current_dir(&empty)
        .output()
        .expect("run the copy");
    std::fs::remove_dir_all(&empty).expect("clean up");
    let stdout = String::from_utf8_lossy(&out.stdout);
    println!("copied the CLI alone into an empty directory ({entries} file) and ran it:");
    print!("{stdout}");

    assert_eq!(embedded, on_disk, "the embedded model must be the model file's exact bytes");
    assert_eq!(entries, 1, "the directory must hold nothing but the binary");
    assert!(out.status.success() && stdout.contains("Blankets Creek:"), "the copy must answer on its own");
    println!("contract: the bytes travel with the binary (sha256 embedded == file, answers with no model file present) OK");
}
