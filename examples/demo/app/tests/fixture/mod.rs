use demo_client::{
    Counter, DemoClientFactory, Dog, References, ValueFee,
    counter::{self, events::CounterEvents},
    dog::{self, events::DogEvents},
};
use sails_rs::{
    events::Listener,
    gtest::calls::*,
    gtest::{Program, System},
    prelude::*,
};

#[cfg(debug_assertions)]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/debug/demo.opt.wasm";
#[cfg(not(debug_assertions))]
pub(crate) const DEMO_WASM_PATH: &str = "../../../target/wasm32-gear/release/demo.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

pub(crate) struct Fixture {
    program_space: GTestRemoting,
    demo_code_id: CodeId,
}

impl Fixture {
    pub(crate) fn new() -> Self {
        let system = System::new();
        system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
        system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
        let demo_code_id = system.submit_code_file(DEMO_WASM_PATH);

        let program_space = GTestRemoting::new(system, ADMIN_ID.into());
        Self {
            program_space,
            demo_code_id,
        }
    }

    pub(crate) fn demo_code_id(&self) -> CodeId {
        self.demo_code_id
    }

    pub(crate) fn demo_factory(&self) -> DemoClientFactory<GTestRemoting> {
        DemoClientFactory::new(self.program_space.clone())
    }

    pub(crate) fn counter_client(&self) -> Counter<GTestRemoting> {
        Counter::new(self.program_space.clone())
    }

    pub(crate) fn counter_listener(&self) -> impl Listener<CounterEvents> {
        counter::events::listener(self.program_space.clone())
    }

    pub(crate) fn dog_client(&self) -> Dog<GTestRemoting> {
        Dog::new(self.program_space.clone())
    }

    pub(crate) fn dog_listener(&self) -> impl Listener<DogEvents> {
        dog::events::listener(self.program_space.clone())
    }

    pub(crate) fn references_client(&self) -> References<GTestRemoting> {
        References::new(self.program_space.clone())
    }

    pub(crate) fn value_fee_client(&self) -> ValueFee<GTestRemoting> {
        ValueFee::new(self.program_space.clone())
    }

    pub(crate) fn balance_of(&self, id: ActorId) -> ValueUnit {
        self.program_space.system().balance_of(id)
    }

    pub(crate) fn get_program(&self, id: ActorId) -> Option<Program> {
        self.program_space.system().get_program(id)
    }

    pub(crate) fn remoting(&self) -> &GTestRemoting {
        &self.program_space
    }
}
