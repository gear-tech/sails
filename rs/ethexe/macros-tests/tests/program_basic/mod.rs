use sails_rs::{export, program};

#[allow(dead_code)]
pub(super) struct MyProgram {
    counter: i32,
}

#[program]
impl MyProgram {
    #[export(route = "create")]
    pub fn new(counter: i32) -> Self {
        Self { counter }
    }

    #[export]
    pub fn new_forty_two() -> Result<Self, String> {
        Ok(Self { counter: 42 })
    }
}
