mod gen;

use gclient::{EventProcessor, GearApi, Result};

const WASM_PATH: &str = "../target/wasm32-unknown-unknown/debug/this_that_svc.opt.wasm";

#[tokio::main]
async fn main() -> Result<()> {
    // Create API instance
    let api = GearApi::dev().await?;

    // Subscribe to events
    let mut listener = api.subscribe().await?;

    // Check that blocks are still running
    assert!(listener.blocks_running().await?);

    println!("Uploading program...");

    // Calculate gas amount needed for initialization
    let gas_info = api
        .calculate_upload_gas(None, gclient::code_from_os(WASM_PATH)?, vec![], 0, true)
        .await?;

    // Upload and init the program
    let (message_id, program_id, _hash) = api
        .upload_program_bytes_by_path(
            WASM_PATH,
            gclient::now_micros().to_le_bytes(),
            vec![],
            gas_info.min_limit,
            0,
        )
        .await?;

    assert!(listener.message_processed(message_id).await?.succeed());
    println!("Program {program_id} uploaded");

    // construct program client
    let mut sender = sails_client::GClientSender::new(api).await?;

    let client = gen::Client::new().with_program_id(program_id);

    let response = client
        .that()
        .send(&mut sender)
        .await
        .expect("expected call to succeed");

    dbg!(&response);

    Ok(())
}
