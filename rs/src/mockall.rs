use crate::{calls::*, errors::Result, prelude::*};

pub use mockall::*;

mock! {
    pub Activation<A> {}

    impl<A> Action for Activation<A> {
        type Args = A;

        #[cfg(not(feature = "ethexe"))]
        fn with_gas_limit(self, gas_limit: GasUnit) -> Self;
        fn with_value(self, value: ValueUnit) -> Self;
        #[mockall::concretize]
        fn with_args<F: FnOnce(A) -> A>(self, args_fn: F) -> Self;

        #[cfg(not(feature = "ethexe"))]
        fn gas_limit(&self) -> Option<GasUnit>;
        fn value(&self) -> ValueUnit;
        fn args(&self) -> &A;
    }

    impl<A> Activation for Activation<A>
    {
        #[allow(refining_impl_trait)]
        #[mockall::concretize]
        async fn send<S: AsRef<[u8]>>(self, code_id: CodeId, salt: S) -> Result<MockReply<ActorId>>;
        #[mockall::concretize]
        async fn send_recv<S: AsRef<[u8]>>(self, d: CodeId, salt: S) -> Result<ActorId>;
    }
}

mock! {
    pub Call<R: Remoting, O> {}

    impl<R: Remoting, O> Action for Call<R, O> {
        type Args = R::Args;

        #[cfg(not(feature = "ethexe"))]
        fn with_gas_limit(self, gas_limit: GasUnit) -> Self;
        fn with_value(self, value: ValueUnit) -> Self;
        #[mockall::concretize]
        fn with_args<F: FnOnce(R::Args) -> R::Args>(self, args_fn: F) -> Self;

        #[cfg(not(feature = "ethexe"))]
        fn gas_limit(&self) -> Option<GasUnit>;
        fn value(&self) -> ValueUnit;
        fn args(&self) -> &R::Args;
    }

    impl<R: Remoting, O> Call for Call<R, O>
    {
        type Remoting = R;
        type Output = O;

        #[allow(refining_impl_trait)]
        async fn send(self, target: ActorId) -> Result<MockReply<O>>;
        #[allow(refining_impl_trait)]
        async fn send_recv(self, target: ActorId) -> Result<O>;
        fn send_one_way(self, target: ActorId) -> Result<MessageId>;
    }
}

mock! {
    pub Query<R: Remoting, O> {}

    impl<R: Remoting, O> Action for Query<R, O> {
        type Args = R::Args;

        #[cfg(not(feature = "ethexe"))]
        fn with_gas_limit(self, gas_limit: GasUnit) -> Self;
        fn with_value(self, value: ValueUnit) -> Self;
        #[mockall::concretize]
        fn with_args<F: FnOnce(R::Args) -> R::Args>(self, args_fn: F) -> Self;

        #[cfg(not(feature = "ethexe"))]
        fn gas_limit(&self) -> Option<GasUnit>;
        fn value(&self) -> ValueUnit;
        fn args(&self) -> &R::Args;
    }

    impl<R: Remoting, O> Query for Query<R, O> {
        type Remoting = R;
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
