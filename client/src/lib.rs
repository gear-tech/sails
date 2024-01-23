#![no_std]

use async_trait::async_trait;
use core::fmt::Debug;
use core::marker::PhantomData;
use gstd::{msg::MessageFuture, prelude::*, MessageId};
use parity_scale_codec::{Decode, Encode};

#[non_exhaustive]
#[derive(Default)]
pub struct SendArgs {
    value: u128,
    program_id: [u8; 32],
    _sender_id: [u8; 32],
}

#[non_exhaustive]
pub struct SendTicket<R: Decode + Debug> {
    // message we are waiting on
    f: MessageFuture,
    // silence the compiler
    _marker: PhantomData<R>,
}

impl<R: Decode + Debug> SendTicket<R> {
    pub async fn result(self) -> Result<R, gstd::errors::Error> {
        let payload = self.f.await?;

        R::decode(&mut payload.as_ref()).map_err(gstd::errors::Error::Decode)
    }

    pub fn message_id(&self) -> MessageId {
        self.f.waiting_reply_to
    }
}

// implemented by: gclient
// by: native gr_send
// by: gtest
#[async_trait(?Send)]
pub trait Sender<R: Decode + Debug> {
    type Error;

    async fn send(&mut self, payload: &[u8], args: SendArgs) -> Result<SendTicket<R>, Self::Error>;
}

/// Sender that runs message against gtest::Program
#[derive(Default)]
pub struct NativeSender;
impl NativeSender {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait(?Send)]
impl<R: Decode + Debug> Sender<R> for NativeSender {
    type Error = gstd::errors::Error;

    async fn send(&mut self, payload: &[u8], args: SendArgs) -> Result<SendTicket<R>, Self::Error> {
        let future =
            gstd::msg::send_bytes_for_reply(args.program_id.into(), payload, args.value, 0)?;

        Ok(SendTicket {
            f: future,
            _marker: PhantomData,
        })
    }
}

#[derive(Default)]
pub struct Call<R: Decode + Debug> {
    /// request payload
    payload: Vec<u8>,
    /// optional args
    args: SendArgs,
    /// silence the compiler
    _marker: PhantomData<R>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError<T> {
    Parse(parity_scale_codec::Error),
    Sender(T),
    FailedToMap,
}

impl<R: Decode + Debug> Call<R> {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            args: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_program_id(mut self, program_id: impl Into<[u8; 32]>) -> Self {
        self.args.program_id = program_id.into();
        self
    }

    pub async fn send<E>(
        self,
        sender: &mut dyn Sender<R, Error = E>,
    ) -> Result<SendTicket<R>, SendError<E>> {
        let ticket = sender
            .send(&self.payload, self.args)
            .await
            .map_err(|e| SendError::<E>::Sender(e))?;

        Ok(ticket)
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.payload
    }
}

impl<R: Encode + Decode + Debug> Call<R> {
    /// Create call that instantly resolves into `result`. Useful for mocking responses
    pub fn ready(result: R) -> Self {
        let mut payload = Vec::new();
        result.encode_to(&mut payload);

        Self::new(payload)
    }
}
