use core::marker::PhantomData;
use parity_scale_codec::{Decode, Encode};
use sails_macros::gservice;
use sails_rtl::gstd::services::Service;
use scale_info::TypeInfo;

struct MyService<'a, T> {
    _a: PhantomData<&'a T>,
}

#[gservice]
impl<'a, T> MyService<'a, T>
where
    T: Clone,
{
    pub fn do_this(&mut self) -> u32 {
        42
    }
}

#[tokio::main]
async fn main() {
    const DO_THIS: &str = "DoThis";

    let my_service = MyService::<'_, String> { _a: PhantomData };

    let output = my_service
        .expose(&[1, 2, 3])
        .handle(&DO_THIS.encode())
        .await;
    let mut output = output.as_slice();

    let func_name = String::decode(&mut output).unwrap();
    assert_eq!(func_name, DO_THIS);

    let result = u32::decode(&mut output).unwrap();
    assert_eq!(result, 42);

    assert_eq!(output.len(), 0);
}
