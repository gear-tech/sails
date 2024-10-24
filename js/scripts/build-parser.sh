echo "[*] Building parser wasm"
cargo build -p sails-idl-parser --target=wasm32-unknown-unknown --release
echo "[*] Optimizing parser wasm"
wasm-opt -O4 -o ./js/parser/parser.wasm ./target/wasm32-unknown-unknown/release/sails_idl_parser.wasm
ls -l ./js/parser/parser.wasm
