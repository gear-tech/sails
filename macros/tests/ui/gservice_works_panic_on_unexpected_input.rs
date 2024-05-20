use fluent_asserter::prelude::*;
use parity_scale_codec::Encode;
use sails_rtl::{
    gstd::{gservice, services::Service},
    MessageId,
};
use tokio::runtime::Runtime;

struct MyService;

#[gservice]
impl MyService {
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    pub fn this(&self, p1: bool) -> bool {
        !p1
    }
}

fn main() {
    let rt = Runtime::new().unwrap();
    assert_that_code!(|| {
        rt.block_on(async {
            let input = [0xffu8; 16];
            MyService
                .expose(MessageId::from(123), &[1, 2, 3])
                .handle(&input)
                .await
        })
    })
    .panics()
    .with_message("Unknown request: 0xffffffff..ffffffff");
}
