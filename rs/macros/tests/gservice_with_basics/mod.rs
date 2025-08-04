use sails_rs::prelude::*;

pub(super) struct SomeService;

#[service]
impl SomeService {
    #[export]
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    #[export]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[derive(Encode)]
pub(super) struct DoThisParams {
    pub(super) p1: u32,
    pub(super) p2: String,
}
