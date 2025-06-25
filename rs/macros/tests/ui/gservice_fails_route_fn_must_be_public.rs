use sails_macros::{export, service};

struct MyService;

#[service]
impl MyService {
    pub fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    #[export(route = "this")]
    fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[tokio::main]
async fn main() {}
