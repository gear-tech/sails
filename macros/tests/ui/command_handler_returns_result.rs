use sails_macros::command_handlers;

#[command_handlers]
mod commands {
    fn do_this(value: u32) -> u32 {
        value
    }
}

fn main() {}
