# CLAUDE.md

Guidance for Claude Code (and any contributor) when working in this repository.

This repo is a Cargo workspace (`crates/install-checks`, `crates/first-steps`,
`crates/regression`, `crates/model-checks`, `crates/trail-status`) plus course
materials (`outline.md`, `capstone.md`, `LESSONS.md`). Every crate pins
**aprender 0.71.4**. Don't upgrade it: the lesson videos show that version's
output.

## The lesson videos depend on this code

The lessons are recorded from this repository, and they show source files on
screen with commands like `grep -n -A3 Step …` and `grep -n assert …`
(see `LESSONS.md`). So:

- **Do not run `cargo fmt` over the workspace.** The code is hand-formatted
  to about 110 columns, and reformatting moves the line numbers the videos
  show. Format only the new code you write.
- Keep the `// Step N` comments, the `assert!` lines and the
  `contract: … OK` output that a lesson greps for or waits for.
- Don't rename binaries, crates or paths that appear in `LESSONS.md`.
- `LESSONS.md` is generated from the course's demo scripts. Don't edit it by
  hand.

## Core workflow: Test-Driven Development

Every program states a **Provable contract** in its doc comment, checks it
with `assert!`, and prints `contract: … OK` as its last line. Each crate's
`tests/contracts.rs` runs the programs exactly as their lessons do. For every
fix or improvement:

1. **Write a failing test first.** Add or change a test in `tests/` (or a
   `#[cfg(test)]` module) that captures the bug or the new behavior. Run
   `cargo test` and confirm it fails for the expected reason.
2. **Make the minimal change** to `src/` needed to make that test pass.
3. **Run `cargo test` again** and confirm the full suite passes.

### Never add a feature without a test

Every new program, flag or code path ships with a test that exercises it, and
every new program states and prints its contract. Bug fixes need a
regression test that fails without the fix and passes with it.

Tests that need `apr` on PATH or the model files under `models/` must skip
(print `NOT RUN` or `skipping`) when those are missing, so that `cargo test`
passes on a fresh clone.

## Required checks before considering any change complete

```sh
cargo clippy --workspace --all-targets -- -D warnings   # lint, warnings denied
cargo build --workspace                                  # confirm it compiles
cargo test --workspace                                   # confirm the full suite passes
```

Or use the Makefile target that bundles them:

```sh
make check   # lint (markdown) + rust (clippy + test) + validate (course structure)
```

## Repository shape

- `crates/install-checks`: contract checkers for 1.2.x (install channels,
  an installed `apr`, a greedy model run).
- `crates/first-steps`: 1.3.x. `save-model`, `inspect-lib` and the
  `reopen-estimate` application. It depends on `regression`.
- `crates/regression`: 2.1.x. `gen-data` (rebuilds `data/reopen-hours.csv`
  byte for byte), `train-test` and `estimators`.
- `crates/model-checks`: checkers for 2.2.x and 2.3.x. Its tests run every
  checker on real `apr` output saved in `fixtures/`.
- `crates/trail-status`: week 3. `explore`, `compare`, `train`, the
  embedded-model CLI and HTTP API, and their checkers. The trained model in
  `crates/trail-status/models/trail_status_model.apr` is committed because
  the CLI and the API embed it with `include_bytes!`.

## Summary of rules

- TDD only: failing test -> implementation -> passing test. No exceptions.
- No feature or fix lands without a test that would fail without it.
- No workspace-wide `cargo fmt`. Always run `cargo clippy -- -D warnings`
  before finishing.
- Always confirm `cargo build` and `cargo test` succeed before finishing.
- Prefer `make check` as the single pre-commit gate.
- Data files are inputs. Rebuild `data/reopen-hours.csv` with `gen-data`
  instead of editing it, and never rewrite `data/trail-status/`.
- Retrain `trail_status_model.apr` only with `cargo run --release -p
  trail-status --bin train`. The lessons show its sha256.
