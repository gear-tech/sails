use parity_scale_codec::{Decode, Encode};
use sails_macros::{gservice, groute};
use scale_info::TypeInfo;

struct MyService;

#[gservice]
impl MyService {
    pub fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    #[groute("this")]
    fn this(&self, p1: bool) -> bool {
        !p1
    }
}

#[tokio::main]
async fn main() {}
