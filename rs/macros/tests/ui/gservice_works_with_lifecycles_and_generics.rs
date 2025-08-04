use core::marker::PhantomData;
use sails_rs::{
    gstd::{service, services::Service},
    MessageId,
};

struct MyService<'a, T> {
    _a: PhantomData<&'a T>,
}

#[service]
impl<'a, T> MyService<'a, T>
where
    T: Clone,
{
    #[export]
    pub fn do_this(&mut self) -> u32 {
        42
    }
}

#[tokio::main]
async fn main() {
    const DO_THIS: &str = "DoThis";

    let my_service = MyService::<'_, String> { _a: PhantomData };

    let output = my_service
        .expose(MessageId::from(123), &[1, 2, 3])
        .handle(&DO_THIS.encode())
        .await;
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = u32::decode(&mut output).unwrap();
    assert_eq!(result, 42);

    assert_eq!(output.len(), 0);
}
