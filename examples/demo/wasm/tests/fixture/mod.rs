use core::cell::OnceCell;
use demo_client::{counter, Counter, DemoFactory, PingPong};
use sails_rtl::{event_listener::*, gtest::calls::*, prelude::*};

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

    #[allow(dead_code)]
    pub(crate) fn cloned_program_space(&self) -> GTestRemoting {
        self.program_space.clone()
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
        let demo_code_id = self
            .demo_program_code_id
            .get_or_init(|| self.program_space.system().submit_code_file(DEMO_WASM_PATH));
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

    #[allow(dead_code)]
    pub(crate) fn counter_listener(&self) -> impl Subscribe<counter::events::CounterEvents> {
        counter::events::listener(self.program_space.clone())
    }
}
