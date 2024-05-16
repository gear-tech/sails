use sails_macros::gservice;

struct MyService;

#[gservice]
impl MyService {}

#[tokio::main]
async fn main() {}
