# Quality gate. `make` runs every check;
# `make build` refuses to build until they all pass.

.PHONY: check fmt-check lint test build fmt audit clean

check: fmt-check lint test

fmt-check:
	cargo fmt --check

lint:
	cargo clippy --all-targets --locked -- -D warnings

test:
	cargo test --locked
	python3 tests/cli_test.py

build: check
	cargo build --release --locked

fmt:
	cargo fmt

audit:
	cargo audit

clean:
	cargo clean
