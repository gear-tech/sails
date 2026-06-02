precommit: fmt clippy test

precommit-js:
	@yarn install
	@yarn build
	@yarn format
	@yarn lint

fmt:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo fmt --all -- --check

test:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo test --workspace --all-targets

clippy:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo clippy --workspace --all-targets -- -D warnings

bench:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo test --release --manifest-path=benchmarks/Cargo.toml

build-bench-analyzer:
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo build --bin bench-analyzer

build-parser:
	@echo "Building idlparser"
	@cargo build -p sails-idl-parser --target=wasm32v1-none --release
	@ls -lah ./target/wasm32v1-none/release/sails_idl_parser.wasm
	@cp ./target/wasm32v1-none/release/sails_idl_parser.wasm js/parser/parser.wasm

build-proxy:
# Just a regular build using the `wasm32v1-none` target.
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo build -p proxy --target=wasm32v1-none
# Optinal optimization using `binaryen` tools.
	@wasm-opt target/wasm32v1-none/debug/proxy.wasm -O4 -o target/wasm32v1-none/debug/proxy.opt.wasm -mvp --enable-sign-ext --zero-filled-memory --dae --vacuum -g

build-proxy-idl:
# This command has to be run every time there are changes in your contract.
# Essentially, it has to be a part of your build pipeline.
	@__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1 cargo run -p proxy -F="idl-gen" --bin proxy-idl-gen

build-js:
	@echo "Building sails-js"
	yarn build
