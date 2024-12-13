use sails_macros::{export, service};

struct MyService;

#[service]
impl MyService {
    #[export(route = "this")]
    #[route("this")]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[tokio::main]
async fn main() {}
