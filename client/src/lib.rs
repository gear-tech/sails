use async_trait::async_trait;
use core::fmt::Debug;
use gtest::RunResult;
use parity_scale_codec::Decode;
use std::marker::PhantomData;

#[non_exhaustive]
#[derive(Default)]
pub struct SendArgs {
    value: u128,
    program_id: [u8; 32],
    sender_id: [u8; 32],
}

#[non_exhaustive]
#[derive(Default, Debug)]
pub struct SendResult {
    payload: Vec<u8>,
}

// implemented by: gclient
// by: native gr_send
// by: gtest
#[async_trait]
pub trait Sender {
    type Error;

    async fn send(&mut self, payload: &[u8], args: SendArgs) -> Result<SendResult, Self::Error>;
}

/// Sender that runs message against gtest::Program
pub struct GTestSender<'a> {
    program: &'a gtest::Program<'a>,
}

unsafe impl Send for GTestSender<'_> {}

impl<'a> GTestSender<'a> {
    pub fn new(program: &'a gtest::Program<'a>) -> Self {
        Self { program }
    }
}

#[async_trait]
impl<'a> Sender for GTestSender<'a> {
    type Error = RunResult;

    async fn send(&mut self, payload: &[u8], args: SendArgs) -> Result<SendResult, Self::Error> {
        let result = self
            .program
            .send_bytes_with_value(args.sender_id, payload, args.value);

        if result.main_failed() || result.others_failed() {
            return Err(result);
        }

        // find response in result logs
        let predicate = |l: &&gtest::CoreLog| {
            l.source() == self.program.id() && l.destination() == args.sender_id.into()
        };

        let resp_msg = result
            .log()
            .iter()
            .find(predicate)
            .ok_or_else(|| result.clone())?;

        Ok(SendResult {
            payload: resp_msg.payload().to_owned(),
        })
    }
}

/// Sender that runs message against gtest::Program
pub struct NativeSender;

impl NativeSender {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Sender for NativeSender {
    type Error = gstd::errors::Error;

    async fn send(&mut self, payload: &[u8], args: SendArgs) -> Result<SendResult, Self::Error> {
        let payload =
            gstd::msg::send_bytes_for_reply(args.program_id.into(), payload, args.value, 0)?
                .await?;

        Ok(SendResult { payload })
    }
}

#[derive(Default)]
pub struct Call<R: Decode + Debug, F: FnOnce(R) -> Option<T>, T> {
    /// request payload
    payload: Vec<u8>,
    /// optional args
    args: SendArgs,
    /// function to map result of computation
    map: F,
    /// silence the compiler
    _marker: PhantomData<R>,
}

#[non_exhaustive]
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SendError<T> {
    Parse(parity_scale_codec::Error),
    Sender(T),
    FailedToMap,
}

impl<R: Decode + Debug, F: FnOnce(R) -> Option<T>, T> Call<R, F, T> {
    pub fn new(payload: Vec<u8>, f: F) -> Self {
        Self {
            payload,
            args: Default::default(),
            map: f,
            _marker: PhantomData,
        }
    }

    pub async fn send<E>(self, sender: &mut dyn Sender<Error = E>) -> Result<T, SendError<E>> {
        let result = sender
            .send(&self.payload, self.args)
            .await
            .map_err(|e| SendError::<E>::Sender(e))?;

        let parsed = R::decode(&mut result.payload.as_ref()).map_err(|e| SendError::Parse(e))?;
        let mapped = (self.map)(parsed).ok_or(SendError::FailedToMap)?;

        Ok(mapped)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.payload
    }
}
