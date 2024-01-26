#![no_std]

use core::fmt::Debug;
use core::marker::PhantomData;
use gstd::{errors::Error as GStdError, msg::MessageFuture, prelude::*, ActorId, MessageId};
use parity_scale_codec::Decode;

#[derive(Default)]
pub struct SendArgs {
    value: u128,
    reply_deposit: u64,
}

pub struct SendTicket<R: Decode + Debug> {
    // message we are waiting on
    f: MessageFuture,
    // silence the compiler
    _marker: PhantomData<R>,
}

impl<R: Decode + Debug> SendTicket<R> {
    pub async fn response(self) -> Result<R, GStdError> {
        let payload = self.f.await?;

        R::decode(&mut payload.as_ref()).map_err(GStdError::Decode)
    }

    pub fn message_id(&self) -> MessageId {
        self.f.waiting_reply_to
    }
}

#[derive(Default)]
pub struct GStdSender;

impl GStdSender {
    pub fn new() -> Self {
        Self
    }
}

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
    Sender(GStdError),
}

impl From<parity_scale_codec::Error> for SendError {
    fn from(e: parity_scale_codec::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<GStdError> for SendError {
    fn from(e: GStdError) -> Self {
        Self::Sender(e)
    }
}

impl<R: Decode + Debug> Call<R> {
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            payload,
            args: SendArgs::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_value(mut self, value: u128) -> Self {
        self.args.value = value;
        self
    }

    pub fn with_reply_deposit(mut self, reply_deposit: u64) -> Self {
        self.args.reply_deposit = reply_deposit;
        self
    }

    pub async fn send(self, address: ActorId) -> Result<SendTicket<R>, SendError> {
        let future = gstd::msg::send_bytes_for_reply(
            address,
            self.payload,
            self.args.value,
            self.args.reply_deposit,
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
