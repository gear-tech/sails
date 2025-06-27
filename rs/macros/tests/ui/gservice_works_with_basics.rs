use parity_scale_codec::Encode;
use sails_rs::{
    MessageId,
    gstd::{service, services::Service},
};

struct MyService;

#[service]
impl MyService {
    #[export]
    pub async fn do_this(&mut self, p1: u32, p2: String) -> String {
        format!("{p1}: ") + &p2
    }

    #[export]
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

    let input = [
        DO_THIS.encode(),
        MyDoThisParams {
            p1: 42,
            p2: "correct".into(),
        }
        .encode(),
    ]
    .concat();
    let output = MyService
        .expose(MessageId::from(123), &[1, 2, 3])
        .handle(&input)
        .await;
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = String::decode(&mut output).unwrap();
    assert_eq!(result, "42: correct");

    assert_eq!(output.len(), 0);
}
