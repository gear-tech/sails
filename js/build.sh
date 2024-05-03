cargo build --manifest-path=../idl-parser/Cargo.toml --target=wasm32-unknown-unknown --release

wasm-opt -O4 -o ./parser.wasm ../target/wasm32-unknown-unknown/release/sails_idl_parser.wasm

ls -lah ./parser.wasm

yarn build
