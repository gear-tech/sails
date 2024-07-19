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

#[cfg(test)]
mod tests {
    use super::*;
    use demo_client::mockall::MockThisThat;
    use sails::mockall::*;

    #[tokio::test]
    async fn this_that_caller_works() {
        // arrange
        const ACTOR_ID: u64 = 11;
        let mut mock_this_that = MockThisThat::<()>::new();
        mock_this_that.expect_this().returning(|| {
            let mut mock_this_query = MockQuery::new();
            mock_this_query
                .expect_recv()
                .with(predicate::eq(ActorId::from(ACTOR_ID)))
                .times(1)
                .returning(move |_| Ok(42));
            mock_this_query
        });

        // act
        let mut this_that_caller = ThisThatCaller::new(mock_this_that);
        let resp = this_that_caller.call_this(ACTOR_ID.into()).await;

        // assert
        assert_eq!(42, resp);
    }
}
