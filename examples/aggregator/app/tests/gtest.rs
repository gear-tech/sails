#[cfg(feature = "poll")]
mod tests {
    use aggregator_client::{
        AggregatorClient, AggregatorClientCtors, AggregatorClientProgram,
        aggregator::Aggregator as _,
    };
    use demo_client::{DemoClientCtors, DemoClientProgram};
    use redirect_client::{
        RedirectClient, RedirectClientCtors, RedirectClientProgram, redirect::Redirect as _,
    };
    use sails_rs::{ActorId, client::*};

    const ACTOR_ID: u64 = 42;

    #[cfg(debug_assertions)]
    const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
    #[cfg(not(debug_assertions))]
    const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

    async fn setup() -> (GtestEnv, ActorId, ActorId) {
        let system = sails_rs::gtest::System::new();
        system.init_logger();
        system.mint_to(ACTOR_ID, 100_000_000_000_000);

        let demo_code = std::fs::read(DEMO_WASM_PATH).unwrap();
        let demo_code_id = system.submit_code(demo_code);
        let aggregator_code_id = system.submit_code(aggregator_app::WASM_BINARY);

        let env = GtestEnv::new(system, ACTOR_ID.into());

        let demo_factory = env.deploy::<DemoClientProgram>(demo_code_id, vec![1]);
        let demo_program = demo_factory.default().await.unwrap();

        let aggregator_factory = env.deploy::<AggregatorClientProgram>(aggregator_code_id, vec![2]);
        let aggregator_program = aggregator_factory.new(demo_program.id()).await.unwrap();

        (env, demo_program.id(), aggregator_program.id())
    }

    #[tokio::test]
    async fn fetch_value_works() {
        let (env, _, aggregator_id) = setup().await;
        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let val: u32 = aggregator.fetch_value().await.unwrap().unwrap();
        assert_eq!(val, 0);
    }

    #[tokio::test]
    async fn fetch_summary_works() {
        let (env, _, aggregator_id) = setup().await;
        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let summary: (u32, u32) = aggregator.fetch_summary().await.unwrap().unwrap();
        assert_eq!(summary, (0, 0));
    }

    #[tokio::test]
    async fn fetch_with_fallback_works() {
        let (env, _, aggregator_id) = setup().await;
        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let val: u32 = aggregator.fetch_with_fallback(true).await.unwrap().unwrap();
        assert_eq!(val, 999);

        let val: u32 = aggregator
            .fetch_with_fallback(false)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(val, 0);
    }

    #[tokio::test]
    async fn fetch_fastest_works() {
        let (env, demo_id, aggregator_id) = setup().await;
        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let winner: u32 = aggregator.fetch_fastest(demo_id).await.unwrap().unwrap();
        assert_eq!(winner, 1);
    }

    #[tokio::test]
    async fn redirect_on_exit_works() {
        let (env, _, aggregator_id) = setup().await;
        let system = env.system();

        let redirect_code_id = system.submit_code(redirect_app::WASM_BINARY);

        let factory1 = env.deploy::<RedirectClientProgram>(redirect_code_id, vec![1]);
        let prog1 = factory1.new().await.unwrap();

        let factory2 = env.deploy::<RedirectClientProgram>(redirect_code_id, vec![2]);
        let prog2 = factory2.new().await.unwrap();

        let env_next = env.clone().with_block_run_mode(BlockRunMode::Next);
        let prog1_next = RedirectClientProgram::client(prog1.id()).with_env(&env_next);
        let _ = prog1_next.redirect().exit(prog2.id()).await.unwrap();

        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let result: ActorId = aggregator
            .fetch_redirect_id(prog1.id())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(result, prog2.id());
    }

    #[tokio::test]
    async fn poll_after_completion_panics() {
        let (env, _, aggregator_id) = setup().await;
        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let res = aggregator.test_poll_after_completion().await;

        match res {
            Err(GtestError::ReplyHasError(reason, payload)) => {
                let reason_str = format!("{:?}", reason);
                assert!(reason_str.contains("UserspacePanic"));

                let payload_str = String::from_utf8_lossy(&payload);
                assert!(payload_str.contains("PendingCall polled after completion"));
            }
            _ => panic!("Expected UserspacePanic error, got {:?}", res),
        }
    }

    #[tokio::test]
    async fn fetch_from_address_handles_timeout() {
        let (env, _, aggregator_id) = setup().await;
        let aggregator = AggregatorClientProgram::client(aggregator_id)
            .with_env(&env)
            .aggregator();

        let res = aggregator
            .fetch_from_address(ActorId::zero())
            .await
            .unwrap();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Timeout"));
    }
}
