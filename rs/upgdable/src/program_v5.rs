use super::*;
use collections::HashMap;

struct State<T> {
    data: T,
}

struct Svc1Data {}

struct Svc2Item {}

struct Svc3Value {}

struct Svc4Data {}

struct ProgramStatePrev {
    // Document
    svc1_state: State<Svc1Data>,
    // Collection
    svc2_state: State<Vec<Svc2Item>>,
    // Map (can be treated as collection of pairs)
    svc3_state: State<HashMap<String, Svc3Value>>,
}

struct ProgramState {}

struct Program(());

impl Program {
    pub fn new() -> (Self, ProgramStatePrev) {
        (
            Self(()),
            ProgramStatePrev {
                svc1_state: State { data: Svc1Data {} },
                svc2_state: State {
                    data: vec![Svc2Item {}],
                },
                svc3_state: State {
                    data: HashMap::new(),
                },
            },
        )
    }
}

trait TempState<PrevStateType> {
    fn apply_document(document: &mut Self, prev_state: &PrevStateType);
}

#[derive(Default)]
struct MyTempState {}

impl TempState<Svc1Data> for MyTempState {
    fn apply_document(document: &mut Self, prev_state: &Svc1Data) {
        // Do something with prev_state
    }
}

impl TempState<Svc4Data> for MyTempState {
    fn apply_document(document: &mut Self, prev_state: &Svc4Data) {
        // Do something with prev_state
    }
}

trait TempCollection<Item> {
    fn apply_collection(&mut self, idx: u32, prev_state: &[Item]);
}

impl TempCollection<Svc2Item> for MyTempState {
    fn apply_collection(&mut self, idx: u32, prev_state: &[Svc2Item]) {
        // Do something with prev_state
    }
}

fn apply() {
    let mut temp_state = MyTempState::default();
    let svc1_data = Svc1Data {};
    <MyTempState as TempState<Svc1Data>>::apply_document(&mut temp_state, &svc1_data);
    //temp_state.apply_document(&svc1_data);
    let svc2_data = vec![Svc2Item {}];
    <MyTempState as TempCollection<Svc2Item>>::apply_collection(&mut temp_state, 0, &svc2_data);
}
