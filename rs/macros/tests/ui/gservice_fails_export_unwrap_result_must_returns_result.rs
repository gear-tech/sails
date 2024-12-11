use sails_macros::service;

struct MyService;

#[service]
impl MyService {
    #[export(route = "this", unwrap_result)]
    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[tokio::main]
async fn main() {}
