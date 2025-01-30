use ptr::addr_of;

use super::*;

//#[migration(MigrationImpl, State)
//#[prev_state(StateBuilderImpl, State)]
struct PrevState {
    svc1_state: u32,
    svc2_state: u32,
    svc3_state: Vec<u32>,
}

trait DocumentMigration<TDocument> {
    fn extend(&mut self, document: &TDocument);
}

trait Migration: Default {
    type StateType;

    fn build(self) -> Self::StateType;
}

#[derive(Default)]
struct MigrationImpl {
    svc1_state: Option<String>,
    svc2_state: Option<u32>,
    svc3_state: Option<Vec<u64>>,
}

impl DocumentMigration<u32> for MigrationImpl {
    fn extend(&mut self, document: &u32) {
        // Do something with document
    }
}

impl DocumentMigration<u16> for MigrationImpl {
    fn extend(&mut self, document: &u16) {
        // Do something with document
    }
}

// Generated based by the `migration` attribute. It knows about all
// properties comprising the state and their types. It knows about
// current state type via parameter `State`.
impl Migration for MigrationImpl {
    type StateType = State;

    fn build(mut self) -> Self::StateType {
        let doc = 42u32;
        <MigrationImpl as DocumentMigration<u32>>::extend(&mut self, &doc);
        let doc = 42u16;
        <MigrationImpl as DocumentMigration<u16>>::extend(&mut self, &doc);
        State {}
    }
}

// This needs be attributed so it can be read by chunks
//#[state]
struct State {}

struct Program {}

//#[program(migration = MigrationImpl)]
//#[program(state = State)]
impl Program {
    pub fn new(p1: u32) -> (Self, State) {
        (Self {}, MigrationImpl::default().build())
    }

    pub fn default() -> Self {
        Self {}
    }
}

fn handle() {
    let mut mugration = MigrationImpl::default();
    // We have to make sure that the state type used by migration matches state type used by the program
    let new_state = <MigrationImpl as Migration>::build(mugration);
}

// fn handle() {
//     let mut migration = Migration::default();
//     let field_name = "svc1_state".to_string();
//     let field_value = 42u32; // read from prev impl
//     migration.accumulate(field_name, field_value);
// }

// #[derive(Default)]
// struct Migration;

// impl Migration {
//     pub fn accumulate(&mut self, field: String, value: u32) {
//         // Do something with field and value
//     }

//     pub fn accumulate_vec(&mut self, field: String, value: Vec<u32>) {
//         // Do something with field and value
//     }

//     pub fn state(self) -> State {
//         State {}
//     }
// }
