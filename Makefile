precommit: fmt clippy test

fmt:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo fmt --all -- --check

test:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo test --workspace --all-targets

clippy:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo clippy --workspace --all-targets -- -D warnings

build-parser:
	@echo "Building idlparser"
	@cargo build -p sails-idl-parser --target=wasm32-unknown-unknown --release
	@ls -lah ./target/wasm32-unknown-unknown/release/sails_idl_parser.wasm
	@cp ./target/wasm32-unknown-unknown/release/sails_idl_parser.wasm js/parser.wasm

build-js:
	@echo "Building sails-js"
	@cd js && yarn build
