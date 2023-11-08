use sails_macros::query_handlers;

#[query_handlers]
mod queries {
    fn this(value: u32) -> u32 {
        value
    }
}

fn main() {}
