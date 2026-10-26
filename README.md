# Introduction to Aprender

[![ci](https://github.com/paiml/intro-rust-ml-framework/actions/workflows/ci.yml/badge.svg)](https://github.com/paiml/intro-rust-ml-framework/actions/workflows/ci.yml)

**Machine learning in pure Rust, from a first install to a model served over HTTP**

Learn to train, inspect, quantize, serve and compile machine-learning models
with [aprender](https://github.com/paiml/aprender), a machine-learning
framework written in pure Rust, and its command-line tool `apr`. You start by
running a small language model on your own machine. Then you train classic
models with the library, read and shrink a real model file, and serve it
through an OpenAI-compatible API. The course ends with you building a text
classifier, embedding it in a single binary, and putting it behind an HTTP
API. Designed for learners new to Rust and new to aprender.

Everything you see run in a lesson video runs here, from the root of this
repository. Every program pins **aprender 0.71.4**, and every program checks a
**contract**: a statement about its result that it proves with assertions
before printing `contract: … OK`. If the contract does not hold, the program
stops with an error.

## Table of Contents

- [What You'll Learn](#what-youll-learn)
- [Course Outline](#course-outline)
- [Requirements](#requirements)
- [Setup](#setup)
- [Hands-On: the lessons](#hands-on-the-lessons)
- [The Data](#the-data)
- [Repository Layout](#repository-layout)
- [Capstone](#capstone)
- [Instructor](#instructor)
- [License](#license)

## What You'll Learn

By the end you can:

- **Install and verify aprender.** Choose between crates.io, the git tag
  and the prebuilt release, then confirm the `apr` on your PATH is the
  version you asked for.
- **Run a language model locally.** Run Qwen2.5-0.5B-Instruct with
  `apr run`, with no GPU and no cloud account.
- **Choose between the CLI and the library.** Do a job once with `apr`,
  then do the same job from Rust code, and build a small application on the
  library.
- **Train and test a model.** Split data into train and test sets, fit a
  regression, and use the `Estimator` pattern (`fit`, `predict`, `score`) to
  swap models without changing the code that evaluates them.
- **Read a model's anatomy.** List a model file's metadata, layers and
  tensors with `apr inspect` and `apr tensors`.
- **Trade size for quality.** Quantize a model to int8 and int4 and measure
  what each one costs.
- **Serve a model with an OpenAI-compatible API.** Run `apr serve`, call it
  with `curl`, stream responses, read its metrics, and point another
  application (opencode) at it.
- **Compile a model into a program.** Turn an `.apr` model into a
  standalone executable with `apr compile`, and embed a model's bytes into
  your own CLI with `include_bytes!`.
- **Build a real classifier from a real dataset.** Explore a hand-labelled
  corpus, compare candidate models against a baseline, train the winner, and
  save it reproducibly as one `.apr` file.
- **Serve your model over HTTP.** Build a small HTTP API around the
  classifier, and check that it gives the same answers as the CLI.

## Course Outline

Three weeks and nine modules. The lesson-by-lesson breakdown is in
[`outline.md`](outline.md).

| Week | Module | Title |
|---|---|---|
| 1 · Get Started | 1.2 | Installing Aprender |
| 1 · Get Started | 1.3 | The CLI and the Library |
| 2 · Models | 2.1 | Training Models with the Library |
| 2 · Models | 2.2 | Inside a Model File |
| 2 · Models | 2.3 | Serving Models with an OpenAI-style API |
| 3 · Ship It | 3.1 | Compiling Models into Programs |
| 3 · Ship It | 3.2 | Building a Trail-Status Classifier |
| 3 · Ship It | 3.3 | Serving the Classifier over HTTP |

Week 3 is the hands-on core. One crate,
[`crates/trail-status`](crates/trail-status), takes a real dataset all the way
to a classifier embedded in a CLI and served over HTTP.

## Requirements

- Rust and `cargo`. CI builds with Rust 1.99.1 and a weekly check builds with the
  current stable release. Install it for your system:

  **Linux/Ubuntu**

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.99.1 --component clippy
  ```

  **macOS**

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  ```
- `apr` 0.71.4, the aprender command-line tool. Pick one:
  - prebuilt (Linux x86_64 / aarch64, seconds):
    `curl -LsSf https://paiml.com/apr/install.sh | sh -s -- --version v0.71.4`
  - from crates.io (any platform, several minutes):
    `cargo install aprender --version 0.71.4 --locked`
- `curl` and `jq` (lessons 2.3 and 3.3)
- `opencode` (lesson 2.3.4 only)

## Setup

```bash
cargo build --workspace            # build every example once
cargo test --workspace             # every crate's tests, including each program's contract
sh scripts/get-model.sh            # Qwen2.5-0.5B-Instruct (~1 GB download) into models/
```

Or, with the Makefile: `make build`, `make test`, `make models`. Run `make help`
for every target.

`models/` and `out/` are created as you go and are not tracked by git. Tests
that need `apr` or the model files print `NOT RUN` or `skipping` and pass
until you have them.

## Hands-On: the lessons

Every command each lesson types is in [`LESSONS.md`](LESSONS.md), in order.

| Lesson | Where |
|---|---|
| 1.2.3 Options for installing Aprender | `crates/install-checks` |
| 1.2.4 Installing and verifying | `examples/install/install-options.txt` |
| 1.2.5 Running with a small model | `models/` |
| 1.3.3 Using the CLI versus the library | `crates/first-steps` |
| 1.3.4 Example application using the library | `crates/first-steps` |
| 2.1.2 Training and testing a model | `crates/regression`, `data/reopen-hours.csv` |
| 2.1.3 The Estimator pattern: fit and predict | `crates/regression` |
| 2.2.2 Reading a model's anatomy | `crates/model-checks` |
| 2.2.4 Trading size for quality | `crates/model-checks` |
| 2.3.2 Introduction to the OpenAI API | `examples/openai-api` |
| 2.3.3 Serving a model with apr serve | `examples/openai-api`, `crates/model-checks` |
| 2.3.4 Using the API with other applications | `examples/opencode-local` |
| 3.1.2 Introduction to APR compile | `crates/trail-status` |
| 3.1.3 Using compile with a model | `models/compile/` |
| 3.1.4 Including bytes in a CLI | `crates/trail-status/src/model.rs` |
| 3.2.2 Exploring the data | `data/trail-status`, `crates/trail-status` |
| 3.2.3 Deciding on a model implementation | `crates/trail-status` |
| 3.2.4 Training the model | `crates/trail-status` |
| 3.3.2 Overview of an HTTP API | `crates/trail-status` |
| 3.3.3 Building and running an HTTP API | `crates/trail-status` |

Week 3 in four commands:

```bash
cargo run --release -p trail-status --bin explore     # what the data looks like
cargo run --release -p trail-status --bin compare     # which model earns its place
cargo run --release -p trail-status --bin train       # train it, save one .apr file
cargo run --release -p trail-status --bin trail-status-cli -- "blankets creek is open. rope mill remains closed."
```

## The Data

- `data/reopen-hours.csv` is **generated**: hours until a trail reopens, from
  rainfall and sun hours, using a stated formula plus seeded noise
  (`crates/regression/src/lib.rs`). `cargo run -p regression --bin gen-data`
  rebuilds it byte for byte.
- `data/trail-status/` is a real, hand-labelled corpus of trail-status updates
  from the SORBA Woodstock trail-status page. See
  [`data/trail-status/PROVENANCE.md`](data/trail-status/PROVENANCE.md).

## Repository Layout

```text
intro-rust-ml-framework/
├── README.md                 # This file
├── outline.md                # Week/module/lesson outline
├── capstone.md               # Capstone brief: deliverables, rubric, share-your-work
├── LESSONS.md                # Every command each lesson types, in order
├── CLAUDE.md                 # Workflow guidance for agentic contributors
├── Cargo.toml                # The Cargo workspace (every crate pins aprender 0.71.4)
├── crates/
│   ├── install-checks/       # 1.2.x: install channels, an installed apr, a model run
│   ├── first-steps/          # 1.3.x: the CLI versus the library, reopen-estimate
│   ├── regression/           # 2.1.x: train/test split, the Estimator pattern
│   ├── model-checks/         # 2.2.x, 2.3.x: anatomy, quantization, serving (with fixtures/)
│   └── trail-status/         # Week 3: explore, compare, train, embed, compile, serve
├── data/
│   ├── reopen-hours.csv      # Generated rainfall data (rebuild with gen-data)
│   └── trail-status/         # The real, hand-labelled trail-status corpus
├── examples/
│   ├── install/              # The install commands from lesson 1.2
│   ├── openai-api/           # Request bodies for apr serve
│   └── opencode-local/       # opencode pointed at a local apr serve
├── scripts/
│   └── get-model.sh          # Downloads the model and lays out models/
├── models/  out/             # Created while following along (gitignored)
├── Makefile                  # help / build / test / lint / rust / validate / check / models
└── LICENSE                   # MIT
```

Every crate has a `tests/contracts.rs` that runs each program exactly as its
lesson does and fails if the `contract: … OK` line is missing.

## Capstone

The capstone is to **build, ship and serve your own text classifier with
aprender**: take a labelled text dataset, compare models against a baseline,
train and save the winner as an `.apr` file, embed it in a CLI, and serve it
over HTTP with the same answers. Deliverables, the evaluation rubric
(Developing / Proficient / Advanced) and how to share your work are in
[`capstone.md`](capstone.md).

## Instructor

- **Alfredo Deza**

## License

Course content and code examples are released under the [MIT License](LICENSE).
The trail-status corpus in `data/trail-status/` is described in its
[`PROVENANCE.md`](data/trail-status/PROVENANCE.md).
