use demo_client::{TupleStruct, traits::ThisThat};
use sails_rename::{calls::*, prelude::*};
#[derive(Clone)]
pub struct ThisThatCaller<ThisThatClient> {
    this_that: ThisThatClient,
}

#[service(crate = sails_rename)]
impl<ThisThatClient> ThisThatCaller<ThisThatClient>
where
    ThisThatClient: ThisThat,
{
    pub const fn new(this_that: ThisThatClient) -> Self {
        Self { this_that }
    }

    pub async fn call_do_this(
        &mut self,
        p1: u32,
        p2: String,
        p3: (Option<H160>, NonZeroU8),
        p4: TupleStruct,
        this_that_addr: ActorId,
    ) -> (String, u32) {
        self.this_that
            .do_this(p1, p2, p3, p4)
            .send_recv(this_that_addr)
            .await
            .unwrap()
    }

    pub async fn query_this(&self, this_that_addr: ActorId) -> u32 {
        self.this_that.this().recv(this_that_addr).await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use demo_client::mockall::MockThisThat;
    use sails_rename::mockall::*;

    #[tokio::test]
    async fn this_that_caller_query_this() {
        // arrange
        const ACTOR_ID: u64 = 11;
        let mut mock_this_that = MockThisThat::<()>::new();
        mock_this_that.expect_this().returning(|| {
            let mut mock_query_this = MockQuery::new();
            mock_query_this
                .expect_recv()
                .with(predicate::eq(ActorId::from(ACTOR_ID)))
                .times(1)
                .returning(move |_| Ok(42));
            mock_query_this
        });

        // act
        let this_that_caller = ThisThatCaller::new(mock_this_that);
        let resp = this_that_caller.query_this(ACTOR_ID.into()).await;

        // assert
        assert_eq!(42, resp);
    }

    #[tokio::test]
    async fn this_that_caller_call_do_this() {
        // arrange
        const ACTOR_ID: u64 = 11;
        let mut mock_this_that = MockThisThat::<()>::new();
        mock_this_that
            .expect_do_this()
            .returning(move |p1, p2, _p3, _p4| {
                let mut mock_call_do_this = MockCall::new();
                mock_call_do_this
                    .expect_send_recv()
                    .with(predicate::eq(ActorId::from(ACTOR_ID)))
                    .times(1)
                    .returning(move |_| Ok((p2.clone(), p1)));
                mock_call_do_this
            });

        // act
        let mut this_that_caller = ThisThatCaller::new(mock_this_that);
        let resp = this_that_caller
            .call_do_this(
                42,
                "test".to_owned(),
                (None, NonZeroU8::MAX),
                TupleStruct(true),
                ACTOR_ID.into(),
            )
            .await;

        // assert
        assert_eq!(("test".to_owned(), 42), resp);
    }
}
