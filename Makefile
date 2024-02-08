build-parser:
	@echo "Building idlparser"
	@cargo build --manifest-path=idlparser/Cargo.toml --target=wasm32-unknown-unknown --release
	@ls -lah ./target/wasm32-unknown-unknown/release/sails_idlparser.wasm
	@cp ./target/wasm32-unknown-unknown/release/sails_idlparser.wasm js/parser.wasm

build-js:
	@echo "Building sails-js"
	@cd js && yarn build
