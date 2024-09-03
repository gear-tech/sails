use demo_client::{
    counter::{self, events::CounterEvents},
    dog::{self, events::DogEvents},
    Counter, DemoFactory, Dog, References,
};
use sails_rs::{
    events::Listener,
    gtest::calls::*,
    gtest::{BlockRunResult, Program, System},
    prelude::*,
};

const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

pub(crate) struct Fixture {
    admin_id: u64,
    program_space: GTestRemoting,
    demo_code_id: CodeId,
}

impl Fixture {
    pub(crate) fn admin_id(&self) -> ActorId {
        self.admin_id.into()
    }

    pub(crate) fn new(admin_id: u64) -> Self {
        let system = System::new();
        system.init_logger();
        system.mint_to(admin_id, 100_000_000_000_000);
        let demo_code_id = system.submit_code_file(DEMO_WASM_PATH);

        let program_space =
            GTestRemoting::new_from_system(system, admin_id.into(), BlockRunMode::Auto);
        Self {
            admin_id,
            program_space,
            demo_code_id,
        }
    }

    pub(crate) fn demo_code_id(&self) -> CodeId {
        self.demo_code_id
    }

    pub(crate) fn demo_factory(&self) -> DemoFactory<GTestRemoting> {
        DemoFactory::new(self.program_space.clone())
    }

    pub(crate) fn demo_program(&self, program_id: ActorId) -> Program<'_> {
        self.program_space.get_program(program_id).unwrap()
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

    pub(crate) fn run_next_block(&self) -> BlockRunResult {
        self.program_space.run_next_block()
    }
}
