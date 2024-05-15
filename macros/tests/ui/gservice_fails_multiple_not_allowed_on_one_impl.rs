use parity_scale_codec::{Decode, Encode};
use sails_macros::gservice;
use scale_info::TypeInfo;

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
