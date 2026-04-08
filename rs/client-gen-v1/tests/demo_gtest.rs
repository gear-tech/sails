#[allow(dead_code, unexpected_cfgs)]
mod demo_client {
    include!("generated/demo_client_v1.rs");
}

use demo_client::*;
use sails_rs::{client::*, futures::StreamExt as _, gtest::System, prelude::*};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

const ACTOR_ID: u64 = 42;
const DEMO_WASM_URL: &str =
    "https://github.com/gear-tech/sails/releases/download/rs%2Fv0.10.3/demo.wasm";
const DEMO_WASM_FILE: &str = "demo-v0.10.3.wasm";

fn demo_wasm_path() -> &'static Path {
    static DEMO_WASM_PATH: OnceLock<PathBuf> = OnceLock::new();

    DEMO_WASM_PATH.get_or_init(|| {
        let dir = std::env::temp_dir().join("sails-client-gen-v1-tests");
        fs::create_dir_all(&dir).expect("create demo wasm cache dir");

        let path = dir.join(DEMO_WASM_FILE);
        if !path.exists() {
            download_demo_wasm(&path);
        }
        path
    })
}

fn download_demo_wasm(path: &Path) {
    let curl = ["curl.exe", "curl"]
        .into_iter()
        .find(|bin| Command::new(bin).arg("--version").output().is_ok())
        .expect("curl is required to download the demo wasm fixture");

    let status = Command::new(curl)
        .args(["-L", "--fail", "--silent", "--show-error", "-o"])
        .arg(path)
        .arg(DEMO_WASM_URL)
        .status()
        .expect("spawn curl");

    assert!(
        status.success(),
        "curl failed to download demo wasm from {DEMO_WASM_URL}"
    );
}

fn create_env() -> (GtestEnv, CodeId) {
    let system = System::new();
    system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
    system.mint_to(ACTOR_ID, 100_000_000_000_000);

    let code_id = system.submit_code_file(demo_wasm_path());
    let env = GtestEnv::new(system, ACTOR_ID.into());
    (env, code_id)
}

#[tokio::test]
async fn counter_add_works() {
    use demo_client::counter::{Counter as _, events::CounterEvents};

    let (env, code_id) = create_env();

    let demo_program = env
        .deploy::<DemoProgram>(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let mut counter_client = demo_program.counter();
    let listener = counter_client.listener();
    let mut events = listener.listen().await.unwrap();

    let result = counter_client.add(10).await.unwrap();

    assert_eq!(result, 52);
    assert_eq!(
        (demo_program.id(), CounterEvents::Added(10)),
        events.next().await.unwrap()
    );
}

#[tokio::test]
async fn counter_query_works() {
    use demo_client::counter::Counter as _;

    let (env, code_id) = create_env();

    let demo_program = env
        .deploy::<DemoProgram>(code_id, vec![])
        .new(Some(42), None)
        .await
        .unwrap();

    let counter_client = demo_program.counter();

    let result = counter_client.value().await.unwrap();

    assert_eq!(result, 42);
}

#[tokio::test]
async fn dog_barks() {
    use demo_client::dog::{Dog as _, events::DogEvents};

    let (env, code_id) = create_env();

    let demo_program = env
        .deploy::<DemoProgram>(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut dog_client = demo_program.dog();
    let listener = dog_client.listener();
    let mut events = listener.listen().await.unwrap();

    let result = dog_client.make_sound().await.unwrap();

    assert_eq!(result, "Woof! Woof!");
    assert_eq!(
        (demo_program.id(), DogEvents::Barked),
        events.next().await.unwrap()
    );
}

#[tokio::test]
async fn dog_walks() {
    use demo_client::dog::{Dog as _, events::DogEvents};

    let (env, code_id) = create_env();

    let demo_program = env
        .deploy::<DemoProgram>(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut dog_client = demo_program.dog();
    let listener = dog_client.listener();
    let mut events = listener.listen().await.unwrap();

    dog_client.walk(10, 20).await.unwrap();

    let position = dog_client.position().await.unwrap();

    assert_eq!(position, (11, 19));
    assert_eq!(
        (
            demo_program.id(),
            DogEvents::Walked {
                from: (1, -1),
                to: (11, 19),
            },
        ),
        events.next().await.unwrap()
    );
}

#[tokio::test]
async fn references_guess_num_works() {
    use demo_client::references::References as _;

    let (env, code_id) = create_env();

    let demo_program = env
        .deploy::<DemoProgram>(code_id, vec![])
        .new(None, Some((1, -1)))
        .await
        .unwrap();

    let mut references_client = demo_program.references();

    let res1 = references_client.guess_num(42).await.unwrap();
    let res2 = references_client.guess_num(89).await.unwrap();
    let message = references_client.message().await.unwrap();
    let set_num = references_client.set_num(14).await.unwrap();
    let res3 = references_client.guess_num(14).await.unwrap();

    assert_eq!(res1, Ok("demo".to_owned()));
    assert_eq!(res2, Err("Number is too large".to_owned()));
    assert_eq!(message, Some("demo".to_owned()));
    assert_eq!(set_num, Ok(()));
    assert_eq!(res3, Ok("demo".to_owned()));
}
