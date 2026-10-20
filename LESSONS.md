# The commands from each lesson

Every command typed in the lesson videos, in order. Run them from the root of
this repository, after the setup in README.md.

## 1.2.3 Options for installing Aprender

```bash
cargo info aprender@0.71.4
git ls-remote --tags https://github.com/paiml/aprender v0.71.4
cat examples/install/install-options.txt
cargo run -p install-checks --bin install-options
```

## 1.2.4 Installing and verifying

```bash
apr --version
curl -LsSf https://paiml.com/apr/install.sh | sh -s -- --version v0.71.4 --cpu --install-dir /tmp/apr-demo/bin
export PATH=/tmp/apr-demo/bin:$PATH
which apr
apr --version
apr --help | head -14
cargo run -p install-checks --bin verify-install -- /tmp/apr-demo/bin/apr
```

## 1.2.5 Running with a small model

```bash
du -hL models/qwen2.5-0.5b-instruct.*
apr run models/qwen2.5-0.5b-instruct.safetensors --chat --temperature 0 --no-gpu -n 48 -p 'What is Rust? Answer in one sentence.'
apr run models/qwen2.5-0.5b-instruct.safetensors --chat --temperature 0 --no-gpu -n 96 -p 'Give three short tips for riding a muddy trail.'
cargo run -p install-checks --bin run-check
```

## 1.3.3 Using the CLI versus the library

```bash
cargo run -p first-steps --bin save-model
apr inspect --json models/reopen-hours.safetensors
grep -n -A3 Step crates/first-steps/src/bin/inspect_lib.rs
cargo run -p first-steps --bin inspect-lib
```

## 1.3.4 Example application using the library

```bash
grep -n -A3 Step crates/first-steps/src/bin/reopen_estimate.rs
cargo build --release -p first-steps
ls -sh target/release/reopen-estimate
target/release/reopen-estimate --rain 20 --sun 4
target/release/reopen-estimate --rain 2 --sun 9
target/release/reopen-estimate --rain -3 --sun 4
target/release/reopen-estimate --rain 35 --sun 1
```

## 2.1.2 Training and testing a model (regression, with a train/test split)

```bash
ls crates/regression/src/bin data
head -6 data/reopen-hours.csv && wc -l data/reopen-hours.csv
grep -n -A3 Step crates/regression/src/bin/train_test.rs
cargo run -p regression --bin train-test
grep -n assert crates/regression/src/bin/train_test.rs
```

## 2.1.3 The Estimator pattern: fit and predict

```bash
grep -n -A5 "fn evaluate" crates/regression/src/bin/estimators.rs
grep -n -A1 Step crates/regression/src/bin/estimators.rs
grep -n new crates/regression/src/bin/estimators.rs
cargo run -p regression --bin estimators
grep -n assert crates/regression/src/bin/estimators.rs
```

## 2.2.2 Reading a model's anatomy (apr inspect and apr tensors)

```bash
apr inspect --json models/qwen2.5-0.5b-instruct.safetensors
apr tensors models/qwen2.5-0.5b-instruct.safetensors --filter layers.0.self_attn
apr tensors models/qwen2.5-0.5b-instruct.safetensors --filter layers.0.mlp
mkdir -p out
apr inspect --json models/qwen2.5-0.5b-instruct.safetensors > out/inspect.json
apr tensors --json models/qwen2.5-0.5b-instruct.safetensors > out/tensors.json
cargo run -p model-checks --bin anatomy-check -- out/inspect.json out/tensors.json
grep -n assert crates/model-checks/src/bin/anatomy_check.rs
```

## 2.2.4 Trading size for quality (quantizing with apr)

```bash
ls -shL models/qwen2.5-0.5b-instruct
mkdir -p models/quant
apr import models/qwen2.5-0.5b-instruct/model.safetensors --quantize int8 -o models/quant/qwen-int8.apr
apr import models/qwen2.5-0.5b-instruct/model.safetensors --quantize int4 -o models/quant/qwen-int4.apr
ls -sh models/quant
cargo run -p model-checks --bin quant-compare
```

## 2.3.2 Introduction to the OpenAI API

```bash
apr serve run models/qwen2.5-0.5b-instruct.apr --port 8089 > serve.log 2>&1 &
curl -s localhost:8089/health | jq
curl -s localhost:8089/v1/models | jq
cat examples/openai-api/chat-request.json
curl -s --json @examples/openai-api/chat-request.json localhost:8089/v1/chat/completions > chat.json
jq . chat.json
cargo run -p model-checks --bin chat-check -- chat.json
kill %1
```

## 2.3.3 Serving a model with apr serve and the OpenAI-style API

```bash
apr serve run models/qwen2.5-0.5b-instruct.apr --port 8089 > serve.log 2>&1 &
head -19 serve.log
grep v1 serve.log
curl -s --json @examples/openai-api/chat-request.json localhost:8089/v1/chat/completions > answer-1.json
curl -s --json @examples/openai-api/chat-request.json localhost:8089/v1/chat/completions > answer-2.json
curl -s --json @examples/openai-api/chat-request-stream.json localhost:8089/v1/chat/completions > stream.txt
head -6 stream.txt
curl -s localhost:8089/metrics | grep -v "#"
cargo run -p model-checks --bin serve-check -- answer-1.json answer-2.json stream.txt
kill %1; wait
curl localhost:8089/health
```

## 2.3.4 Using the API with other applications (curl and opencode)

```bash
apr serve run models/qwen2.5-0.5b-instruct.apr --port 8089 > serve.log 2>&1 &
curl -s --json @examples/openai-api/chat-request.json localhost:8089/v1/chat/completions | jq .choices
cat examples/opencode-local/opencode.json
curl -s localhost:8089/metrics > before.txt
opencode run --dir examples/opencode-local --agent ask "In one sentence, what is Rust?"
curl -s localhost:8089/metrics > after.txt
cargo run -p model-checks --bin metrics-delta -- before.txt after.txt
kill %1; wait
```

## 3.1.2 Introduction to APR compile

```bash
apr compile --help
apr compile --list-targets
apr compile models/compile/qwen2.5-0.5b-instruct-int8.apr -o models/compile/qwen-debug
models/compile/qwen-debug --help
cargo run -p trail-status --bin standalone-check -- models/compile/qwen-debug models/compile/qwen2.5-0.5b-instruct-int8.apr
```

## 3.1.3 Using compile with a model

```bash
ls models/compile/hf
apr import models/compile/hf/model.safetensors --quantize int8 -o models/compile/qwen2.5-0.5b-instruct-int8.apr
apr compile models/compile/qwen2.5-0.5b-instruct-int8.apr -o models/compile/qwen-chat --release
ls -sh models/compile
models/compile/qwen-chat -p "In one sentence, what is the Rust programming language?" -n 48 --no-gpu 2>/dev/null
cargo run -p trail-status --bin standalone-check -- models/compile/qwen-chat models/compile/qwen2.5-0.5b-instruct-int8.apr
```

## 3.1.4 Including bytes in a CLI

```bash
grep -n -A2 Step crates/trail-status/src/model.rs
wc -c crates/trail-status/models/trail_status_model.apr
sha256sum crates/trail-status/models/trail_status_model.apr
cargo build --release -p trail-status --bin trail-status-cli
ls -sh target/release/trail-status-cli
cp target/release/trail-status-cli /tmp
/tmp/trail-status-cli "blankets creek is open. rope mill remains closed."
cargo run --release -p trail-status --bin embed-check
```

## 3.2.2 Exploring the data

```bash
ls data/trail-status
head -3 data/trail-status/train.jsonl
head -11 crates/trail-status/src/data.rs
cargo run --release -p trail-status --bin explore
grep -n assert crates/trail-status/src/bin/explore.rs
```

## 3.2.3 Deciding on a model implementation

```bash
head -12 crates/trail-status/src/bin/compare.rs
cargo run --release -p trail-status --bin compare
grep -n assert crates/trail-status/src/bin/compare.rs
```

## 3.2.4 Training the model

```bash
grep -n Step crates/trail-status/src/bin/train.rs
cargo run --release -p trail-status --bin train
sha256sum crates/trail-status/models/trail_status_model.apr
grep -n assert crates/trail-status/src/bin/train.rs
```

## 3.3.2 Overview of an HTTP API

```bash
cargo run --release -p trail-status --bin trail-status-api > /tmp/api.log 2>&1 &
cat /tmp/api.log
curl -s localhost:8080/ | jq .routes
curl -s -X POST localhost:8080/predict -d '{"text": "blankets creek is open. rope mill remains closed."}' | jq .
curl -s -i -X POST localhost:8080/predict -d '{"words": "no text field"}'
cargo run --release -p trail-status --bin api-check
cat /tmp/api.log
kill %1
```

## 3.3.3 Building and running an HTTP API

```bash
grep -n -A18 Route crates/trail-status/src/bin/api.rs
grep -n -A2 Step crates/trail-status/src/bin/api.rs
cargo build --release -p trail-status
cargo run --release -p trail-status --bin trail-status-api > /tmp/api.log 2>&1 &
cargo run --release -p trail-status --bin api-parity
cargo test --release -p trail-status --test contracts api_keeps
kill %1
```
