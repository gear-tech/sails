use sails::prelude::*;

struct SomeService;

#[sails::service]
impl SomeService {
    #[export(ethabi)]
    pub fn abi_method(
        &self,
        _addr: sails::alloy_primitives::Address,
    ) -> sails::alloy_primitives::B256 {
        sails::alloy_primitives::B256::ZERO
    }
}

fn main() {}
