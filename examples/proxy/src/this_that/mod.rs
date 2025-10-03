use demo_client::{this_that::ThisThat, *};
use sails_rename::{client::*, gstd::Syscall, prelude::*};
#[derive(Clone)]
pub struct ThisThatCaller<ThisThatClient> {
    this_that: ThisThatClient,
}
impl<ThisThatClient> ThisThatCaller<ThisThatClient>
where
    ThisThatClient: ThisThat<Env = GstdEnv>,
{
    pub const fn new(this_that: ThisThatClient) -> Self {
        Self { this_that }
    }
}

#[service(crate = sails_rename)]
impl<ThisThatClient> ThisThatCaller<ThisThatClient>
where
    ThisThatClient: ThisThat<Env = GstdEnv>,
{
    #[export]
    pub async fn call_do_this(
        &mut self,
        p1: u32,
        p2: String,
        p3: (Option<H160>, NonZeroU8),
        p4: TupleStruct,
        this_that_addr: ActorId,
    ) -> (String, u32) {
        if Syscall::program_id() == this_that_addr {
            panic!("ThisThatCaller cannot call itself");
        }

        self.this_that.do_this(p1, p2, p3, p4).await.unwrap()
    }

    #[export]
    pub async fn query_this(&self, this_that_addr: ActorId) -> u32 {
        if Syscall::program_id() == this_that_addr {
            panic!("ThisThatCaller cannot call itself");
        }

        self.this_that.this().await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use demo_client::mockall::MockThisThat;
    use sails_rename::{client::PendingCall, gstd::services::Service};

    #[tokio::test]
    async fn this_that_caller_query_this() {
        // arrange
        const ACTOR_ID: u64 = 11;
        Syscall::with_program_id(ActorId::from(1));

        let mut mock_this_that = MockThisThat::new();
        mock_this_that
            .expect_this()
            .returning(|| PendingCall::from_output(42));

        // act
        let this_that_caller = ThisThatCaller::new(mock_this_that).expose(&[]);
        let resp = this_that_caller.query_this(ACTOR_ID.into()).await;

        // assert
        assert_eq!(42, resp);
    }

    #[tokio::test]
    async fn this_that_caller_call_do_this() {
        // arrange
        const ACTOR_ID: u64 = 11;
        Syscall::with_program_id(ActorId::from(1));

        let mut mock_this_that = MockThisThat::new();
        mock_this_that
            .expect_do_this()
            .returning(move |p1, p2, _p3, _p4| PendingCall::from_output((p2.clone(), p1)));

        // act
        let mut this_that_caller = ThisThatCaller::new(mock_this_that).expose(&[]);
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

    #[tokio::test]
    #[should_panic(expected = "ThisThatCaller cannot call itself")]
    async fn this_that_caller_should_panic() {
        // arrange
        const ACTOR_ID: u64 = 11;
        Syscall::with_program_id(ActorId::from(ACTOR_ID));

        let mock_this_that = MockThisThat::new();

        // act
        let mut this_that_caller = ThisThatCaller::new(mock_this_that).expose(&[]);
        _ = this_that_caller
            .call_do_this(
                42,
                "test".to_owned(),
                (None, NonZeroU8::MAX),
                TupleStruct(true),
                ACTOR_ID.into(),
            )
            .await;
    }
}
