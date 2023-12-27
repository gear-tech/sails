use parity_scale_codec::{Decode, Encode};
use sails_macros::gservice;
use scale_info::TypeInfo;

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

#[derive(Encode)]
struct MyDoThisParams {
    p1: u32,
    p2: String,
}

#[tokio::main]
async fn main() {
    const DO_THIS: &str = "DoThis";

    let mut my_service = MyService;

    let input = [
        DO_THIS.encode(),
        MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode(),
    ]
    .concat();
    let output = requests::process(&mut my_service, &input).await;
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, "42: correct");

    assert_eq!(output.len(), 0);
}
