use sails_macros::service;

struct MyService;

#[service]
impl MyService {}

#[tokio::main]
async fn main() {}
