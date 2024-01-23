generate-this-that-client:
	cargo run -p client-gen ./target/wasm32-unknown-unknown/debug/this_that_svc.sails.idl > ./examples/puppeteer/app/src/client.rs