use sails_macros::program;

#[allow(dead_code)]
pub(super) struct MyProgram {
    counter: i32,
}

#[program]
impl MyProgram {
    pub fn new(counter: i32) -> Self {
        Self { counter }
    }
}
