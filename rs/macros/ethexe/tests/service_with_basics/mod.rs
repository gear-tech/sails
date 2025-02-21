use sails_macros::service;

pub(super) struct MyService;

#[service]
impl MyService {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}
