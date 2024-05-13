use sails_macros::gprogram;

pub struct MyProgram;

#[gprogram]
impl MyProgram {
    pub fn new() -> Self {
        Self
    }
}

#[tokio::main]
async fn main() {
    let _ = MyProgram::new();
}
