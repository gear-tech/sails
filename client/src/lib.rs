#![no_std]

use core::fmt::Debug;
use core::marker::PhantomData;
use gstd::{msg::MessageFuture, prelude::*, ActorId, MessageId};
use parity_scale_codec::{Decode, Encode};

#[non_exhaustive]
#[derive(Default)]
pub struct SendArgs {
    value: u128,
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

/// Sender that runs message against gtest::Program
#[derive(Default)]
pub struct NativeSender;
impl NativeSender {
    pub fn new() -> Self {
        Self
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
pub enum SendError {
    Parse(parity_scale_codec::Error),
    Sender(gstd::errors::Error),
}

impl From<parity_scale_codec::Error> for SendError {
    fn from(e: parity_scale_codec::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<gstd::errors::Error> for SendError {
    fn from(e: gstd::errors::Error) -> Self {
        Self::Sender(e)
    }
}

impl<R: Decode + Debug> Call<R> {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            args: Default::default(),
            _marker: PhantomData,
        }
    }

    pub async fn send(self, address: impl Into<[u8; 32]>) -> Result<SendTicket<R>, SendError> {
        let future = gstd::msg::send_bytes_for_reply(
            ActorId::from(address.into()),
            self.payload,
            self.args.value,
            0,
        )?;

        Ok(SendTicket {
            f: future,
            _marker: PhantomData,
        })
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
