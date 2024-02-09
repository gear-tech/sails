precommit: fmt clippy test

fmt:
	cargo fmt --all -- --check

test:
	cargo test --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings
