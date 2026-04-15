use sails_rs::prelude::*;

struct SomeService;

#[sails_rs::service]
impl SomeService {
    #[export(ethabi)]
    pub fn abi_method(
        &self,
        _addr: sails_rs::alloy_primitives::Address,
    ) -> sails_rs::alloy_primitives::B256 {
        sails_rs::alloy_primitives::B256::ZERO
    }
}

fn main() {}
