use sails_rs::program;

struct MyProgram;

struct Svc1;

#[program(handle_signal = handle_signal_impl)]
impl MyProgram {
    pub fn new() -> Self {
        MyProgram
    }

    pub fn svc1(&self) -> Svc1 {
        Svc1
    }
}

fn main() {}
