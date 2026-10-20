# Capstone: Introduction to Aprender

## Project Overview

Build, ship and serve your own text classifier with aprender. Take a
labelled text dataset, compare candidate models against a baseline, train
and save the winner as a single `.apr` file, embed that file into a CLI, and
serve the same model over HTTP. The project uses everything from the course:
the library's `Estimator` pattern, train/test discipline, the `.apr` model
format, `include_bytes!` embedding, and an HTTP API whose answers you can
prove match the CLI.

The reference implementation is [`crates/trail-status`](crates/trail-status).
It classifies free-text trail-status updates into whether each of two trails
is open. You can adapt it to the bundled `data/trail-status/` corpus, or bring
your own labelled text (support tickets, commit messages, product reviews,
anything with a few classes). Either way, the deliverables below apply.

## Deliverables

### 1. Explore the data

Write an `explore` program that prints the number of rows in your training
and validation sets, the class balance of each, and a few example rows. It
should assert something you rely on later, such as no empty texts, every
label in range, or train and validation sets that don't overlap.

### 2. Choose a model, and earn the choice

Write a `compare` program that scores at least three candidates on the same
features and the same held-out rows. One of them must be a majority-class
baseline. Its contract: the candidate you choose beats the baseline on your
chosen metric, and the program fails if it doesn't.

### 3. Train and save one `.apr` file

Write a `train` program that fits the chosen model, saves it as an `.apr`
file, loads it back from disk, and asserts that the reloaded model predicts
exactly what the in-memory model did on every validation row. Training twice
should write the same bytes, so record the file's `sha256sum`.

### 4. Ship it: CLI and HTTP API

- A CLI that embeds the `.apr` file with `include_bytes!` and classifies
  text passed as an argument. Copy the release binary somewhere outside the
  repository and run it there.
- An HTTP API (any framework; the reference uses `tiny_http`) with at least
  a route listing endpoint and a `POST /predict` endpoint, which returns a
  clear 4xx error for a malformed request.
- A parity check: the API and the CLI give identical answers on every
  validation row.

### 5. Contracts and tests

Every program ends by printing `contract: … OK`. A `tests/contracts.rs`
runs each program the way you demo it and fails if that line is missing. The
crate builds cleanly under `cargo clippy --all-targets -- -D warnings`, and
`make check` passes.

### 6. Write-up

A short README (half a page to a page) for your crate that includes:

- The dataset, where it came from, and its class balance.
- The comparison table from `compare`, and why you chose the model you did.
- Validation accuracy (or your chosen metric) for the saved model.
- The size of the `.apr` file and of the release CLI binary.
- The exact commands to train, run the CLI, start the API, and run the
  parity check.

## Evaluation Criteria

### Advanced

- Your own dataset (or a substantial extension of the bundled one) with its
  provenance documented.
- At least three candidates compared on held-out data. The chosen model
  beats the baseline, and the program fails if it doesn't.
- Deterministic training: two runs produce the same `.apr` sha256.
- The CLI runs from a copy outside the repository. The API returns clear
  errors for bad input, and the parity check passes on every validation row.
- Every program prints its `contract: … OK`. `cargo test` and
  `clippy -D warnings` are clean.

### Proficient

- A working explore → compare → train → CLI → API run, on the bundled data
  or your own.
- A majority-class baseline is included, and the chosen model beats it.
- The saved model reloads and predicts the same as the in-memory model.
- `cargo test` passes, and `clippy` is clean.
- An API exists, but parity with the CLI is checked on a few examples
  rather than every validation row.

### Developing

- A model is trained and saved, but there's no comparison against a
  baseline.
- The CLI or the API is missing, or the CLI reads the model from disk
  instead of embedding it.
- There are no contracts, or the tests cover only the happy path.
- Clippy warnings may be present.

## Resources

- **aprender**: <https://github.com/paiml/aprender>
- **aprender API docs**: <https://docs.rs/aprender/0.71.4/aprender/>
- **`include_bytes!`**: <https://doc.rust-lang.org/std/macro.include_bytes.html>
- **`tiny_http`** (the HTTP server used by the reference API):
  <https://docs.rs/tiny_http/0.12/tiny_http/>
- **TF-IDF** (the features the reference classifier uses):
  <https://en.wikipedia.org/wiki/Tf%E2%80%93idf>

Reference implementation:

- **The trail-status crate**: `crates/trail-status/` in this repository.
  `src/bin/explore.rs`, `compare.rs`, `train.rs`, `cli.rs`, `api.rs` and
  `api_parity.rs` match deliverables 1 to 4.
- **The lesson commands**: [`LESSONS.md`](LESSONS.md), sections 3.1 to 3.3.

## Share Your Work

Add this capstone to your LinkedIn profile as a portfolio project:

1. On your LinkedIn profile, choose **Add profile section** -> **Projects**.
2. **Title:** *Introduction to Aprender: built and served a text classifier
   in pure Rust*.
3. **Description:** Summarize the deliverables: the model comparison
   against a baseline, the reproducible `.apr` model, the single-binary CLI
   with the model embedded, and the HTTP API proven to answer exactly like
   the CLI.
4. **Link:** the URL to your public GitHub repository for the project.
