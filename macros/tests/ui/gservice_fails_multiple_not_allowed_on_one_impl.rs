use sails_macros::gservice;

struct MyService;

#[gservice]
#[gservice]
impl MyService {
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[tokio::main]
async fn main() {}
