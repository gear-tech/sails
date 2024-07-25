use sails_macros::service;

struct MyService;

#[service]
#[service]
impl MyService {
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[tokio::main]
async fn main() {}
