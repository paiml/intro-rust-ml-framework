# Introduction to Aprender: Course Outline

Three weeks and nine modules. Each lesson listed here has a terminal demo,
and every command it types is in [`LESSONS.md`](LESSONS.md).

## Week 1 · Get Started

### 1.2 Installing Aprender

- **1.2.3 Options for installing Aprender.** The same release, 0.71.4,
  through three channels: crates.io, the git tag, and the prebuilt release
  archive. (`crates/install-checks`, `install-options`)
- **1.2.4 Installing and verifying.** Install `apr` with the curl installer
  into a scratch directory, then confirm that the binary on PATH is the
  version you asked for. (`verify-install`)
- **1.2.5 Running with a small model.** Run Qwen2.5-0.5B-Instruct with
  `apr run` on the CPU, with greedy decoding so the answer is repeatable.
  (`run-check`)

### 1.3 The CLI and the Library

- **1.3.3 Using the CLI versus the library.** Save a model from Rust, read
  it with `apr inspect`, then read it again from the library and get the
  same answer. (`crates/first-steps`, `save-model`, `inspect-lib`)
- **1.3.4 Example application using the library.** `reopen-estimate`, a
  small release binary that loads the saved model and refuses inputs outside
  the range it was trained on.

## Week 2 · Models

### 2.1 Training Models with the Library

- **2.1.2 Training and testing a model.** A regression with a train/test
  split on generated rainfall data, and assertions that make "the model
  learned something" testable. (`crates/regression`, `train-test`)
- **2.1.3 The Estimator pattern: fit and predict.** Linear regression,
  ridge and lasso behind one `Estimator` interface, evaluated by one function
  that never names a concrete model. (`estimators`)

### 2.2 Inside a Model File

- **2.2.2 Reading a model's anatomy.** `apr inspect` and `apr tensors` on
  a real language model: metadata, attention and MLP layers, and tensor
  shapes. (`crates/model-checks`, `anatomy-check`)
- **2.2.4 Trading size for quality.** Quantize to int8 and int4 with
  `apr import --quantize` and compare the size each one saves.
  (`quant-compare`)

### 2.3 Serving Models with an OpenAI-style API

- **2.3.2 Introduction to the OpenAI API.** `apr serve`, its health and
  models endpoints, and a chat completion sent with `curl`.
  (`examples/openai-api`, `chat-check`)
- **2.3.3 Serving a model with apr serve.** Repeatable answers, streaming
  with server-sent events, and the Prometheus metrics endpoint.
  (`serve-check`)
- **2.3.4 Using the API with other applications.** Point opencode at the
  local server and use the metrics to show that it really answered.
  (`examples/opencode-local`, `metrics-delta`)

## Week 3 · Ship It

### 3.1 Compiling Models into Programs

- **3.1.2 Introduction to APR compile.** `apr compile`, its targets, and a
  model turned into a standalone executable. (`crates/trail-status`,
  `standalone-check`)
- **3.1.3 Using compile with a model.** Import a model as int8, compile a
  release chat program, and run it with no `apr` installed.
- **3.1.4 Including bytes in a CLI.** Embed a trained `.apr` file into your
  own binary with `include_bytes!`, then copy the binary anywhere and run it.
  (`trail-status-cli`, `embed-check`)

### 3.2 Building a Trail-Status Classifier

- **3.2.2 Exploring the data.** A real, hand-labelled corpus of trail-status
  updates: its shape, its labels and its class balance. (`explore`)
- **3.2.3 Deciding on a model implementation.** Majority class, naive
  Bayes and logistic regression on the same TF-IDF features, scored on the
  same held-out rows. The chosen model has to beat the baseline. (`compare`)
- **3.2.4 Training the model.** Train the chosen model and save it as one
  `.apr` file. Training is deterministic, so the file has the same sha256
  every time. (`train`)

### 3.3 Serving the Classifier over HTTP

- **3.3.2 Overview of an HTTP API.** The trail-status API: its routes, a
  prediction, and a clear error for a bad request. (`trail-status-api`,
  `api-check`)
- **3.3.3 Building and running an HTTP API.** How the API is built, and a
  parity check showing it answers exactly like the CLI. (`api-parity`)
