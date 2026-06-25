use sails_rs::prelude::*;

#[sails_type]
#[event]
pub enum MyEvents {
    Transfer { from: ActorId, amount: u128 },
    Approval(ActorId),
}

fn main() {}
