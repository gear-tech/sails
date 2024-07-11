use core::marker::PhantomData;
use demo_client::traits::ThisThat;
use sails::{calls::Query, gstd::gservice, prelude::*};

#[derive(Clone)]
pub struct ThisThatCaller<ThisThatClient, CallArgs> {
    this_that: ThisThatClient,
    _args: PhantomData<CallArgs>,
}

#[gservice]
impl<ThisThatClient, Args> ThisThatCaller<ThisThatClient, Args>
where
    ThisThatClient: ThisThat<Args>,
    Args: Default,
{
    pub const fn new(this_that: ThisThatClient) -> Self {
        Self {
            this_that,
            _args: PhantomData,
        }
    }

    pub async fn call_this(&mut self, this_that_addr: ActorId) -> u32 {
        self.this_that.this().recv(this_that_addr).await.unwrap()
    }
}
