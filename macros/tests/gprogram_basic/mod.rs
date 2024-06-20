use sails_macros::gprogram;

#[allow(dead_code)]
pub(super) struct MyProgram {
    counter: i32,
}

#[gprogram]
impl MyProgram {
    pub fn new(counter: i32) -> Self {
        Self { counter }
    }
}
