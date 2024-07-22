use sails_macros::gservice;
use sails_rs::Encode;

pub(super) struct MyService;

#[gservice]
impl MyService {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[derive(Encode)]
pub(super) struct MyDoThisParams {
    pub(super) p1: u32,
    pub(super) p2: String,
}
