use counter_client::inc_dec_events::IncDecEvents;
use counter_client::traits::{CounterFactory, IncDec, IncDecListener, Query};
use sails_rtl::{
    calls::{Action, Activation, Call},
    event_listener::{EventListener, EventSubscriber, Listen, Subscribe},
    gtest::calls::{GTestArgs, GTestRemoting},
};

mod counter_client;

const PROGRAM_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/counter.opt.wasm";
const ADMIN_ID: u64 = 10;

#[tokio::test]
async fn counter_succeed() {
    let remoting = GTestRemoting::new();

    let mut rem = remoting.clone();
    // Low level remoting listener
    let mut remoting_listener = rem.subscribe().await.unwrap();

    let code_id = remoting.system().submit_code_file(PROGRAM_WASM_PATH);

    let program_id = counter_client::CounterFactory::new(&remoting)
        .with_initial_value(10)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(code_id, vec![])
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    let mut incdec_listener = counter_client::inc_dec_events::Listener::new(&remoting).listener();
    // Typed service event listener
    let mut listener = incdec_listener.subscribe(program_id).await.unwrap();

    let mut client = counter_client::IncDec::new(remoting.clone());
    let reply = client
        .inc(32)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program_id)
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(10, reply);

    let query_client = counter_client::Query::new(remoting.clone());
    let reply = query_client
        .current_value()
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program_id)
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(42, reply);

    let reply = client
        .dec(1)
        .with_args(GTestArgs::default().with_actor_id(ADMIN_ID.into()))
        .publish(program_id)
        .await
        .unwrap()
        .reply()
        .await
        .unwrap();

    assert_eq!(42, reply);

    let event = remoting_listener.next_event(|_| true).await.unwrap();
    println!("{:?}", event);
    let event = remoting_listener.next_event(|_| true).await.unwrap();
    println!("{:?}", event);

    let event = listener.next_event().await.unwrap();
    assert_eq!(IncDecEvents::Inc(32), event);
    let event = listener.next_event().await.unwrap();
    assert_eq!(IncDecEvents::Dec(1), event);
}
