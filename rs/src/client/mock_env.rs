use super::*;

#[derive(Default, Clone)]
pub struct MockEnv;

crate::params_struct_impl!(MockEnv, MockParams {});

impl GearEnv for MockEnv {
    type Error = ::gstd::errors::Error;
    type Params = MockParams;
    type MessageState = core::future::Ready<Result<Vec<u8>, Self::Error>>;
}

impl<T: CallEncodeDecode> PendingCall<MockEnv, T>
where
    T::Reply: Encode + Decode,
{
    pub fn from_output(output: T::Reply) -> Self {
        Self::from_result(Ok(output))
    }

    pub fn from_error(err: <MockEnv as GearEnv>::Error) -> Self {
        Self::from_result(Err(err))
    }

    pub fn from_result(res: Result<T::Reply, <MockEnv as GearEnv>::Error>) -> Self {
        PendingCall {
            env: mock_env::MockEnv,
            destination: ActorId::zero(),
            route: None,
            params: None,
            args: None,
            state: Some(future::ready(res.map(|v| v.encode()))),
        }
    }
}

impl<T: CallEncodeDecode<Reply = O>, O> From<O> for PendingCall<MockEnv, T>
where
    O: Encode + Decode,
{
    fn from(value: O) -> Self {
        PendingCall::from_output(value)
    }
}

impl<T: CallEncodeDecode> Future for PendingCall<MockEnv, T> {
    type Output = Result<T::Reply, <MockEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state.take() {
            Some(ready) => {
                let res = ready.into_inner();
                Poll::Ready(res.map(|v| T::Reply::decode(&mut v.as_slice()).unwrap()))
            }
            None => panic!("{PENDING_CALL_INVALID_STATE}"),
        }
    }
}

impl<A, T: CallEncodeDecode> Future for PendingCtor<MockEnv, A, T> {
    type Output = Result<Actor<A, MockEnv>, <MockEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state.take() {
            Some(_ready) => {
                let program_id = self
                    .program_id
                    .take()
                    .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
                let env = self.env.clone();
                Poll::Ready(Ok(Actor::new(env, program_id)))
            }
            None => panic!("{PENDING_CTOR_INVALID_STATE}"),
        }
    }
}
