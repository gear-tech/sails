use sails_rs::prelude::*;

// Service extension requires the service to implement `Clone`
#[derive(Clone)]
pub struct MammalService {
    avg_weight: u32,
}

impl MammalService {
    pub const fn new(avg_weight: u32) -> Self {
        Self { avg_weight }
    }
}

#[service]
impl MammalService {
    #[export]
    pub fn make_sound(&mut self) -> &'static str {
        panic!("Not implemented")
    }

    #[export]
    pub fn avg_weight(&self) -> u32 {
        self.avg_weight
    }
}
