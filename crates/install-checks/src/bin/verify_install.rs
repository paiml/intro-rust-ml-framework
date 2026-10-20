//! Lesson 1.2.4: verify an installed `apr` is the release we asked for.
//!
//! Provable contract: installed and resolvable. The `apr` that PATH finds is
//! the binary just installed (no older copy shadows it), it reports the
//! pinned version, and it offers the commands this course uses.
//!
//! Usage: verify-install <path-to-installed-apr> [version]
//!   cargo run -p install-checks --bin verify-install -- /tmp/apr-demo/bin/apr

use install_checks::{apr_version, first_on_path, help_commands, run, PINNED_VERSION};

const COURSE_COMMANDS: [&str; 6] = ["run", "serve", "inspect", "tensors", "quantize", "compile"];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let installed = args.first().expect("usage: verify-install <path-to-installed-apr> [version]");
    let want = args.get(1).map_or(PINNED_VERSION, String::as_str);
    let installed = std::fs::canonicalize(installed).expect("the installed binary exists");

    let path_var = std::env::var("PATH").unwrap_or_default();
    let resolved = first_on_path("apr", &path_var).and_then(|p| std::fs::canonicalize(p).ok());
    println!("installed:  {}", installed.display());
    println!("on PATH:    {}", resolved.as_ref().map_or("(no apr on PATH)".into(), |p| p.display().to_string()));

    let exe = installed.to_str().expect("utf-8 path");
    let version_text = run(exe, &["--version"]).expect("apr --version");
    let version = apr_version(&version_text).expect("apr --version names a version");
    println!("version:    {version}");

    let commands = help_commands(&run(exe, &["--help"]).expect("apr --help"));
    let missing: Vec<&str> = COURSE_COMMANDS.iter().copied().filter(|c| !commands.iter().any(|h| h == c)).collect();
    println!("commands:   {} available; course needs {}", commands.len(), COURSE_COMMANDS.join(", "));

    assert_eq!(resolved.as_ref(), Some(&installed), "PATH must resolve `apr` to the binary just installed");
    assert_eq!(version, want, "the installed apr must report {want}");
    assert!(missing.is_empty(), "apr is missing commands: {missing:?}");
    println!("contract: installed and resolvable (PATH's apr is the new binary, version {want}, all course commands present) OK");
}
