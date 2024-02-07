cargo build --manifest-path=../idlparser/Cargo.toml --target=wasm32-unknown-unknown --release

wasm-opt -O4 -o ./parser.wasm ../target/wasm32-unknown-unknown/release/sails_idlparser.wasm

ls -lah ./parser.wasm

yarn build
