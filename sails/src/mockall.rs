use crate::{calls::*, errors::Result, prelude::*};

pub use mockall::*;

mock! {
    pub Activation<A> {}

    impl<A> Action for Activation<A> {
        type Args = A;

        fn with_value(self, value: ValueUnit) -> Self;
        fn with_args(self, args: A) -> Self;
        fn value(&self) -> ValueUnit;
        fn args(&self) -> &A;
    }

    impl<A> Activation for Activation<A>
    {
        #[allow(refining_impl_trait)]
        #[mockall::concretize]
        async fn send<S: AsRef<[u8]>>(self, code_id: CodeId, salt: S) -> Result<MockReply<ActorId>>;
    }
}

mock! {
    pub Call<A, O> {}

    impl<A, O> Action for Call<A, O> {
        type Args = A;

        fn with_value(self, value: ValueUnit) -> Self;
        fn with_args(self, args: A) -> Self;
        fn value(&self) -> ValueUnit;
        fn args(&self) -> &A;
    }

    impl<A, O> Call for Call<A, O>
    {
        type Output = O;

        #[allow(refining_impl_trait)]
        async fn send(self, target: ActorId) -> Result<MockReply<O>>;
    }
}

mock! {
    pub Query<A, O> {}

    impl<A, O> Action for Query<A, O> {
        type Args = A;

        fn with_value(self, value: ValueUnit) -> Self;
        fn with_args(self, args: A) -> Self;
        fn value(&self) -> ValueUnit;
        fn args(&self) -> &A;
    }

    impl<A, O> Query for Query<A, O> {
        type Output = O;

        async fn recv(self, target: ActorId) -> Result<O>;
    }
}

mock! {
    pub Reply<O> {}

    impl<O> Reply for Reply<O>
    {
        type Output = O;

        async fn recv(self) -> Result<O>;
    }
}