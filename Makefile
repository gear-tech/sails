precommit: fmt clippy test

fmt:
	cargo fmt --all -- --check

test:
	cargo test --workspace --all-targets

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

generate-this-that-client:
	cargo run -p client-gen ./target/wasm32-unknown-unknown/debug/this_that_svc.sails.idl > ./examples/puppeteer/app/src/puppet.rs
