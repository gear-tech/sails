#[tokio::test]
async fn do_something_works() {
    println!("WASM_BINARY length: {}", {{ program-name-snake }}::WASM_BINARY.len());
}
