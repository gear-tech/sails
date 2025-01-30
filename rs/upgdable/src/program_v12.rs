#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use super::*;

//#[source_program_state(MigrationImpl)]
struct MyProgramV1 {
    svc1_state: u32,
    svc2_state: String,
}

//#[program_state(MigrationImpl)]
struct MyProgramV2 {
    svc1_state: u64,
    svc2_state: Vec<u16>,
}

//#[program(state = MyProgramV2)]
impl MyProgramV2 {
    pub fn from_u32(p1: u32) -> Self {
        Self {
            svc1_state: Default::default(),
            svc2_state: Default::default(),
        }
    }

    pub fn my_service(&self) -> MyService {
        MyService //::new(self.state().svc1_state)
    }

    // fn state(&self) -> RefCell<MyProgramState> {
    //     todo!()
    // }

    // // generated
    // pub fn __from_migration(source_actor_id: ActorId, migration: &MigrationImpl) -> Self {
    //     let s = MaybeUnint<Self>::default();
    //     migration.migrate()

    // }
}

struct MyService;
