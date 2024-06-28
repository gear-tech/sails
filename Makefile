precommit: fmt clippy test

fmt:
	@cargo fmt --all -- --check

test:
	@cargo test --workspace --all-targets

clippy:
	@cargo clippy --workspace --all-targets -- -D warnings

build-parser:
	@echo "Building idlparser"
	@cargo build -p sails-idl-parser --target=wasm32-unknown-unknown --release
	@ls -lah ./target/wasm32-unknown-unknown/release/sails_idl_parser.wasm
	@cp ./target/wasm32-unknown-unknown/release/sails_idl_parser.wasm js/parser.wasm

build-js:
	@echo "Building sails-js"
	@cd js && yarn build
