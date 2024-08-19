use core::cell::OnceCell;
use demo_client::{
    counter::{self, events::CounterEvents},
    dog::{self, events::DogEvents},
    Counter, DemoFactory, Dog, References,
};
use gtest::Program;
use sails_rs::{events::Listener, gtest::calls::*, prelude::*};

const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.opt.wasm";

pub(crate) const ADMIN_ID: u64 = 10;

pub(crate) struct Fixture {
    admin_id: u64,
    program_space: GTestRemoting,
    demo_program_code_id: OnceCell<CodeId>,
}

impl Fixture {
    pub(crate) fn admin_id(&self) -> ActorId {
        self.admin_id.into()
    }

    pub(crate) fn new(admin_id: u64) -> Self {
        let program_space = GTestRemoting::new(admin_id.into());
        program_space.system().init_logger();
        Self {
            admin_id,
            program_space,
            demo_program_code_id: OnceCell::new(),
        }
    }

    pub(crate) fn demo_code_id(&self) -> CodeId {
        let demo_code_id = self
            .demo_program_code_id
            .get_or_init(|| self.program_space.system().submit_code_file(DEMO_WASM_PATH));
        *demo_code_id
    }

    pub(crate) fn demo_factory(&self) -> DemoFactory<GTestRemoting> {
        DemoFactory::new(self.program_space.clone())
    }

    pub(crate) fn demo_program(&self, program_id: ActorId) -> Program<'_> {
        self.program_space
            .system()
            .get_program(program_id.as_ref())
            .unwrap()
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

    pub(crate) fn run_next_block(&self) -> gtest::BlockRunResult {
        self.program_space.system().run_next_block()
    }
}
