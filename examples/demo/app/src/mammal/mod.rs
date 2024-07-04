use sails_rtl::{gstd::gservice, prelude::*};

#[derive(Clone)]
pub struct MammalService {
    avg_weight: u32,
}

impl MammalService {
    pub const fn new(avg_weight: u32) -> Self {
        Self { avg_weight }
    }
}

#[gservice]
impl MammalService {
    pub fn make_sound(&mut self) -> &'static str {
        panic!("Not implemented")
    }

    pub fn avg_weight(&self) -> u32 {
        self.avg_weight
    }
}
