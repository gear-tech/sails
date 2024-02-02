build:
	@echo "Building idlparser"
	@cargo build --manifest-path=idlparser/Cargo.toml --target=wasm32-unknown-unknown --release
	@ls -lah ./target/wasm32-unknown-unknown/release/sails_idlparser.wasm

build-js:
	@echo "Building sails-js"
	@cd js && yarn build