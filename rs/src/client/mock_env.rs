use super::*;

#[derive(Default, Clone)]
pub struct MockEnv;

#[derive(Default)]
pub struct MockParams;

impl GearEnv for MockEnv {
    type Error = ::gstd::errors::Error;
    type Params = MockParams;
    type MessageState = core::future::Ready<Result<Vec<u8>, Self::Error>>;
}

impl<O: Encode + Decode> PendingCall<MockEnv, O> {
    pub fn from_output(output: O) -> Self {
        Self::from_result(Ok(output))
    }

    pub fn from_error(err: <MockEnv as GearEnv>::Error) -> Self {
        Self::from_result(Err(err))
    }

    pub fn from_result(res: Result<O, <MockEnv as GearEnv>::Error>) -> Self {
        PendingCall {
            env: mock_env::MockEnv,
            destination: ActorId::zero(),
            params: None,
            payload: None,
            _output: PhantomData,
            state: Some(future::ready(res.map(|v| v.encode()))),
        }
    }
}

impl<O: Encode + Decode> From<O> for PendingCall<MockEnv, O> {
    fn from(value: O) -> Self {
        PendingCall::from_output(value)
    }
}

impl<O: Decode> Future for PendingCall<MockEnv, O> {
    type Output = Result<O, <MockEnv as GearEnv>::Error>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state.take() {
            Some(ready) => {
                let res = ready.into_inner();
                Poll::Ready(res.map(|v| O::decode(&mut v.as_ref()).unwrap()))
            }
            None => panic!("PendingCall polled after completion or invalid state"),
        }
    }
}
