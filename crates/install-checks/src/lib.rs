//! Contract checkers for the install and first-run lessons (1.2.3, 1.2.4, 1.2.5).
//!
//! The binaries shell out to the real tools (`cargo`, `git`, `curl`, `apr`)
//! and hold their output to a contract. Everything that READS that output
//! lives here, as plain functions with unit tests, so a change in a tool's
//! output format fails a test rather than a recording.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The release every lesson pins.
pub const PINNED_VERSION: &str = "0.71.4";

/// Run a command and return its stdout, or an error naming the command.
pub fn run(program: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("could not start {program}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{program} {} exited {}: {}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(strip_ansi(&String::from_utf8_lossy(&out.stdout)))
}

/// Remove ANSI escape sequences (colors, cursor moves). Tools print them when
/// told to (`CARGO_TERM_COLOR=always` in the demo shell), and a parser must
/// read the same text either way.
pub fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // CSI: parameters and intermediates, then one final byte in @..~
                for d in chars.by_ref() {
                    if ('@'..='~').contains(&d) {
                        break;
                    }
                }
            } else {
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The `version:` line of `cargo info`.
pub fn cargo_info_version(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|l| l.trim().strip_prefix("version: "))
        .map(str::trim)
}

/// Asset names in a GitHub release JSON document.
pub fn release_asset_names(json: &str) -> Result<Vec<String>, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("release JSON: {e}"))?;
    let assets = v["assets"]
        .as_array()
        .ok_or("release JSON has no assets array")?;
    Ok(assets
        .iter()
        .filter_map(|a| a["name"].as_str().map(str::to_string))
        .collect())
}

/// The prebuilt archive `install.sh` downloads for a version, target and variant.
pub fn archive_name(version: &str, target: &str, variant: &str) -> String {
    format!("apr-v{version}-{target}-{variant}.tar.gz")
}

/// `"apr 0.71.4 (5027da624)"` -> `"0.71.4"`.
pub fn apr_version(text: &str) -> Option<&str> {
    text.lines()
        .find_map(|l| l.trim().strip_prefix("apr "))
        .and_then(|rest| rest.split_whitespace().next())
}

/// Commands from `apr --help`'s `Commands:` section.
pub fn help_commands(help: &str) -> Vec<String> {
    help.lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| !l.trim().is_empty())
        .filter(|l| l.starts_with("  ") && !l.starts_with("      "))
        .filter_map(|l| l.split_whitespace().next().map(str::to_string))
        .collect()
}

/// The generated text in `apr run` output: the lines after `Output:` up to the
/// blank line before `Completed in`.
pub fn run_answer(text: &str) -> Option<String> {
    let mut lines = text.lines().skip_while(|l| l.trim() != "Output:");
    lines.next()?;
    let answer: Vec<&str> = lines
        .take_while(|l| !l.trim().is_empty() && !l.starts_with("Completed in"))
        .collect();
    (!answer.is_empty()).then(|| answer.join("\n").trim().to_string())
}

/// The first executable named `name` on a `PATH` string, as `which` finds it.
pub fn first_on_path(name: &str, path_var: &str) -> Option<PathBuf> {
    path_var
        .split(':')
        .filter(|d| !d.is_empty())
        .map(|d| Path::new(d).join(name))
        .find(|p| {
            use std::os::unix::fs::PermissionsExt;
            p.metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_colors() {
        assert_eq!(
            strip_ansi("\u{1b}[1m\u{1b}[32mversion:\u{1b}[0m 0.71.4"),
            "version: 0.71.4"
        );
        assert_eq!(strip_ansi("plain"), "plain");
    }

    #[test]
    fn reads_a_colored_cargo_info_version_once_stripped() {
        let colored = "\u{1b}[1mversion:\u{1b}[0m 0.71.4\n";
        assert_eq!(cargo_info_version(&strip_ansi(colored)), Some("0.71.4"));
    }

    #[test]
    fn reads_the_cargo_info_version() {
        let text = "aprender #ml\nNext-generation ML\nversion: 0.71.4\nlicense: MIT\n";
        assert_eq!(cargo_info_version(text), Some("0.71.4"));
        assert_eq!(cargo_info_version("no version here"), None);
    }

    #[test]
    fn reads_release_asset_names() {
        let json =
            r#"{"tag_name":"v0.71.4","assets":[{"name":"a.tar.gz"},{"name":"a.tar.gz.sha256"}]}"#;
        assert_eq!(
            release_asset_names(json).unwrap(),
            vec!["a.tar.gz", "a.tar.gz.sha256"]
        );
        assert!(release_asset_names("{}").is_err());
        assert!(release_asset_names("not json").is_err());
    }

    #[test]
    fn names_the_archive_install_sh_downloads() {
        assert_eq!(
            archive_name("0.71.4", "x86_64-unknown-linux-gnu", "cpu"),
            "apr-v0.71.4-x86_64-unknown-linux-gnu-cpu.tar.gz"
        );
    }

    #[test]
    fn reads_the_apr_version() {
        assert_eq!(apr_version("apr 0.71.4 (5027da624)\n"), Some("0.71.4"));
        assert_eq!(apr_version("something else"), None);
    }

    #[test]
    fn lists_help_commands_but_not_wrapped_descriptions() {
        let help = "CLI tool\n\nUsage: apr [OPTIONS] <COMMAND>\n\nCommands:\n  run      Run model\n  serve    Inference server\n  \
                    compile  Compile model into standalone executable (APR-SPEC\n                           §4.16)\n\nOptions:\n  -h\n";
        assert_eq!(help_commands(help), vec!["run", "serve", "compile"]);
    }

    #[test]
    fn reads_the_answer_from_apr_run() {
        let text = "=== APR Run ===\n\nSource: m.safetensors\n\nOutput:\n4\n\nCompleted in 2.30s (cached)\n";
        assert_eq!(run_answer(text).as_deref(), Some("4"));
        let two_lines = "Output:\nline one\nline two\n\nCompleted in 1s\n";
        assert_eq!(run_answer(two_lines).as_deref(), Some("line one\nline two"));
        assert_eq!(run_answer("Output:\n\nCompleted in 1s\n"), None);
        assert_eq!(run_answer("no output marker"), None);
    }

    #[test]
    fn finds_the_first_executable_on_path() {
        assert!(first_on_path("sh", "/nonexistent:/bin:/usr/bin").is_some());
        assert_eq!(
            first_on_path("definitely-not-a-command-xyz", "/bin:/usr/bin"),
            None
        );
    }
}
