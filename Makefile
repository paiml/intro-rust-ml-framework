SHELL := /bin/bash
.DEFAULT_GOAL := help

.PHONY: help build test lint lint-md rust validate check models clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
	awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-16s\033[0m %s\n", $$1, $$2}'

build: ## Build every example in the workspace
	cargo build --workspace

test: ## Run every crate's tests, including each program's contract
	cargo test --workspace

lint: lint-md ## Markdown lint (no tabs)
	@echo "=== Markdown lint passed ==="

lint-md: ## Reject literal tabs in markdown
	@echo "=== Markdown lint (no tabs) ==="
	@find . -name '*.md' -not -path './.git/*' -not -path '*/target/*' -not -path './models/*' -not -path './out/*' | \
		xargs -I{} sh -c 'grep -Pn "\t" "$$1" && echo "FAIL: tab in $$1" && exit 1 || true' _ {}
	@echo "  no tabs found"

# No `cargo fmt --check`: the code is hand-formatted, and the lesson videos
# show its line numbers (see CLAUDE.md).
rust: ## clippy + test the Rust workspace
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

validate: ## Structural checks (outline, capstone, lessons, data, README)
	@echo "=== Course structure ==="
	@for f in outline.md capstone.md LESSONS.md LICENSE; do \
		test -f $$f || { echo "FAIL: $$f missing"; exit 1; }; done
	@for s in "## Deliverables" "## Evaluation Criteria" "## Share Your Work"; do \
		grep -qF "$$s" capstone.md || { echo "FAIL: capstone.md missing $$s"; exit 1; }; done
	@grep -q "Introduction to Aprender" README.md || { echo "FAIL: README missing course title"; exit 1; }
	@for f in data/reopen-hours.csv data/trail-status/train.jsonl data/trail-status/val.jsonl \
		crates/trail-status/models/trail_status_model.apr; do \
		test -f $$f || { echo "FAIL: $$f missing"; exit 1; }; done
	@echo "  outline + capstone (3 sections) + lessons + data + embedded model + README present"

check: lint rust validate ## Full gate
	@echo "=== All quality gates passed ==="

models: ## Download the model and lay out models/ (needs apr 0.71.4, ~3 GB disk)
	sh scripts/get-model.sh

clean: ## Remove build artifacts
	cargo clean
