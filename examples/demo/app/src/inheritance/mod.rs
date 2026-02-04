use crate::dog::DogService;
use sails_rs::prelude::*;

pub struct ChainService {
    dog: DogService,
}

impl ChainService {
    pub fn new(dog: DogService) -> Self {
        Self { dog }
    }
}

impl From<ChainService> for DogService {
    fn from(value: ChainService) -> Self {
        value.dog
    }
}

#[service(extends = DogService)]
impl ChainService {
    // This method has the same name as in DogService, but it's NOT an override.
    // In IDL v2, this is perfectly fine: they will have different INTERFACE_IDs.
    #[export]
    pub fn make_sound(&mut self) -> &'static str {
        "Chain Woof!"
    }
}
