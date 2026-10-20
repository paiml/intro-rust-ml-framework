//! Lesson 1.2.3: check the three ways to get aprender all offer the pinned release.
//!
//! 1. crates.io  — `cargo install aprender` (compiles from source on your machine)
//! 2. git source — clone the v0.71.4 tag and `cargo install --path .`
//! 3. prebuilt   — `curl … install.sh | sh` downloads a release archive
//!
//! Provable contract: one release, three channels. crates.io publishes
//! 0.71.4, the git repository has the v0.71.4 tag, and the GitHub release has
//! a prebuilt archive for this machine together with the .sha256 file the
//! installer checks it against.
//!
//! Run from the workspace root:
//!   cargo run -p install-checks --bin install-options

use install_checks::{archive_name, cargo_info_version, release_asset_names, run, PINNED_VERSION};

const REPO: &str = "https://github.com/paiml/aprender";
const TARGET: &str = "x86_64-unknown-linux-gnu";

fn main() {
    let v = PINNED_VERSION;

    // 1. crates.io
    let info = run("cargo", &["info", &format!("aprender@{v}")]).expect("cargo info");
    let crates_version = cargo_info_version(&info).expect("cargo info prints a version line");
    println!("crates.io   aprender {crates_version}");

    // 2. git source: ask the remote for the tag without cloning anything.
    let tag_ref = format!("refs/tags/v{v}");
    let tags = run("git", &["ls-remote", "--tags", REPO, &tag_ref]).expect("git ls-remote");
    let tag_commit = tags.split_whitespace().next().unwrap_or("");
    println!("git source  {REPO} tag v{v} -> {}", tag_commit.get(..12).unwrap_or("(missing)"));

    // 3. prebuilt archives on the GitHub release.
    let api = format!("https://api.github.com/repos/paiml/aprender/releases/tags/v{v}");
    let assets = release_asset_names(&run("curl", &["-fsSL", &api]).expect("GitHub API")).expect("release JSON");
    let prefix = format!("apr-v{v}-");
    let archives: Vec<&String> = assets.iter().filter(|a| a.starts_with(&prefix) && a.ends_with(".tar.gz")).collect();
    println!("prebuilt    {} apr archives on the GitHub release:", archives.len());
    for a in &archives {
        println!("              {a}");
    }
    let archive = archive_name(v, TARGET, "cpu");
    let has_archive = assets.contains(&archive);
    let has_checksum = assets.contains(&format!("{archive}.sha256"));
    let found = |yes: bool| if yes { "found" } else { "MISSING" };
    println!("this machine  {:<56}{}", archive, found(has_archive));
    println!("              {:<56}{}", format!("{archive}.sha256"), found(has_checksum));

    assert_eq!(crates_version, v, "crates.io must publish {v}");
    assert!(tag_commit.len() == 40, "the git remote must have tag v{v}");
    assert!(has_archive && has_checksum, "the release must carry {archive} and its .sha256");
    println!("contract: one release, three channels (crates.io, git tag, prebuilt archive: all {v}) OK");
}
