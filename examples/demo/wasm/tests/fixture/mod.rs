use core::cell::OnceCell;
use demo_client::{Counter, DemoFactory, PingPong};
use sails_rtl::{gtest::calls::*, prelude::*};

const DEMO_WASM_PATH: &str = "../../../target/wasm32-unknown-unknown/debug/demo.wasm";

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
        let program_space = GTestRemoting::new();
        program_space.system().init_logger();
        Self {
            admin_id,
            program_space,
            demo_program_code_id: OnceCell::new(),
        }
    }

    pub(crate) fn demo_code_id(&self) -> CodeId {
        let demo_code_id = self.demo_program_code_id.get_or_init(|| {
            let code_id = self.program_space.system().submit_code_file(DEMO_WASM_PATH);
            code_id.as_ref().into()
        });
        *demo_code_id
    }

    pub(crate) fn demo_factory(&self) -> DemoFactory<GTestRemoting, GTestArgs> {
        DemoFactory::new(self.program_space.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn ping_pong_client(&self) -> PingPong<GTestRemoting, GTestArgs> {
        PingPong::new(self.program_space.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn counter_client(&self) -> Counter<GTestRemoting, GTestArgs> {
        Counter::new(self.program_space.clone())
    }
}
