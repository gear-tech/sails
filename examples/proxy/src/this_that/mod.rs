use demo_client::traits::ThisThat;
use sails::{calls::Query, prelude::*};

#[derive(Clone)]
pub struct ThisThatCaller<ThisThatClient> {
    this_that: ThisThatClient,
}

#[gservice]
impl<ThisThatClient> ThisThatCaller<ThisThatClient>
where
    ThisThatClient: ThisThat,
{
    pub const fn new(this_that: ThisThatClient) -> Self {
        Self { this_that }
    }

    pub async fn call_this(&mut self, this_that_addr: ActorId) -> u32 {
        self.this_that.this().recv(this_that_addr).await.unwrap()
    }
}
