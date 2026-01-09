use ping_pong_bench_app::PingPongProgram;

fn main() {
    let idl_file_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("ping_pong.idl");

    // Generate IDL file for the `PingPongProgram` app
    sails_idl_gen::generate_idl_to_file::<PingPongProgram>(Some("PingPong"), &idl_file_path)
        .unwrap();

    // Generate client code from IDL file
    sails_client_gen::ClientGenerator::from_idl_path(&idl_file_path)
        .generate_to(
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
                .join("../app/src/ping_pong_client.rs"),
        )
        .unwrap();
}