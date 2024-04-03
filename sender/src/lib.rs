#![no_std]

use core::fmt::Debug;
use core::marker::PhantomData;
use gstd::{
    errors::Error as GStdError,
    msg::{CreateProgramFuture, MessageFuture},
    prelude::*,
    prog::{create_program_bytes_for_reply, ProgramGenerator},
    ActorId, CodeId, MessageId,
};
use parity_scale_codec::{Decode, Error as ParseError};

#[derive(Default, Clone, Copy)]
pub struct GStdSender;

impl GStdSender {
    pub fn new() -> Self {
        Self
    }
}

impl GStdSender {
    pub fn send(
        &self,
        address: ActorId,
        payload: Vec<u8>,
        value: u128,
        reply_deposit: u64,
    ) -> Result<MessageFuture, GStdError> {
        let future = gstd::msg::send_bytes_for_reply(address, payload, value, reply_deposit)?;

        Ok(future)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError {
    Parser(ParseError),
    Sender(GStdError),
}

impl From<ParseError> for SendError {
    fn from(e: ParseError) -> Self {
        Self::Parser(e)
    }
}

impl From<GStdError> for SendError {
    fn from(e: GStdError) -> Self {
        Self::Sender(e)
    }
}

#[derive(Default)]
struct SendArgs {
    value: u128,
    reply_deposit: u64,
}

pub struct Call<'a, R: Decode + Debug> {
    /// serialized method and args
    payload: Vec<u8>,
    /// method to verify we received the correct response
    method: String,
    /// optional args
    send_args: SendArgs,
    /// the client to send the message
    sender: &'a GStdSender,
    /// silence the compiler
    _marker: PhantomData<R>,
}

impl<'a, R: Decode + Debug> Call<'a, R> {
    pub fn new<T: Encode + Debug>(sender: &'a GStdSender, method: &str, args: T) -> Self {
        let capacity = method.encoded_size() + args.encoded_size();
        let mut payload = Vec::with_capacity(capacity);

        method.encode_to(&mut payload);
        args.encode_to(&mut payload);

        Self {
            payload,
            method: method.to_string(),
            send_args: SendArgs::default(),
            sender,
            _marker: PhantomData,
        }
    }

    pub fn with_value(mut self, value: u128) -> Self {
        self.send_args.value = value;
        self
    }

    pub fn with_reply_deposit(mut self, reply_deposit: u64) -> Self {
        self.send_args.reply_deposit = reply_deposit;
        self
    }

    pub async fn send(self, address: ActorId) -> Result<CallTicket<R>, SendError> {
        let future = self.sender.send(
            address,
            self.payload,
            self.send_args.value,
            self.send_args.reply_deposit,
        )?;

        Ok(CallTicket {
            f: future,
            method: self.method,
            _marker: PhantomData,
        })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.payload
    }
}

pub struct CallTicket<R: Decode + Debug> {
    /// message we are waiting on
    f: MessageFuture,
    /// method
    method: String,
    /// silence the compiler
    _marker: PhantomData<R>,
}

impl<R: Decode + Debug> CallTicket<R> {
    pub async fn response(self) -> Result<R, GStdError> {
        let payload = self.f.await?;
        Self::decode_response(self.method, payload)
    }

    fn decode_response(method: String, payload: Vec<u8>) -> Result<R, GStdError> {
        let encoded = method.encode();

        if payload.len() < encoded.len() {
            return Err(GStdError::Decode(ParseError::from("response too short")));
        }

        if payload[..encoded.len()] != encoded {
            return Err(GStdError::Decode(ParseError::from(
                "unexpected response method",
            )));
        }

        let mut args = &payload[encoded.len()..];

        R::decode(&mut args).map_err(GStdError::Decode)
    }

    pub fn message_id(&self) -> MessageId {
        self.f.waiting_reply_to
    }
}

#[derive(Default)]
struct CreateArgs {
    value: u128,
    reply_deposit: u64,
    salt: Option<Vec<u8>>,
}

pub struct CreateProgramCall {
    /// serialized method and args
    payload: Vec<u8>,
    /// optional args
    create_args: CreateArgs,
}

impl CreateProgramCall {
    pub fn new<T: Encode + Debug>(method: &str, args: T) -> Self {
        let capacity = method.encoded_size() + args.encoded_size();
        let mut payload = Vec::with_capacity(capacity);

        method.encode_to(&mut payload);
        args.encode_to(&mut payload);

        Self {
            payload,
            create_args: CreateArgs::default(),
        }
    }

    pub fn with_value(mut self, value: u128) -> Self {
        self.create_args.value = value;
        self
    }

    pub fn with_reply_deposit(mut self, reply_deposit: u64) -> Self {
        self.create_args.reply_deposit = reply_deposit;
        self
    }

    /// Specify a salt to use for the program creation
    /// Random salt is generated if salt is not set
    pub fn with_salt(mut self, salt: Vec<u8>) -> Self {
        self.create_args.salt = Some(salt);
        self
    }

    pub async fn create(self, code_id: CodeId) -> Result<CreateProgramTicket, SendError> {
        let future = create_program_bytes_for_reply(
            code_id,
            self.create_args
                .salt
                .unwrap_or(ProgramGenerator::get_salt()),
            &self.payload,
            self.create_args.value,
            self.create_args.reply_deposit,
        )?;

        Ok(CreateProgramTicket { f: future })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.payload
    }
}

pub struct CreateProgramTicket {
    /// message we are waiting on
    f: CreateProgramFuture,
}

pub trait ProgramClient {
    fn new(id: ActorId) -> Self;
}

impl CreateProgramTicket {
    pub async fn response(self) -> Result<ActorId, GStdError> {
        let (prog_id, reply) = self.f.await?;

        assert_eq!(reply, vec![123]);

        Ok(prog_id)
    }

    pub fn message_id(&self) -> MessageId {
        self.f.waiting_reply_to
    }

    pub fn program_id(&self) -> ActorId {
        self.f.program_id
    }
}
mod tests {
    use super::*;

    #[test]
    fn test_request_encoding() {
        let sender = GStdSender::new();
        let call: Call<()> = Call::new(&sender, "AAAA", "BBBB");
        let bytes = call.into_bytes();
        assert_eq!(bytes, vec![16, 65, 65, 65, 65, 16, 66, 66, 66, 66]);

        assert_eq!(
            CallTicket::decode_response("AAAA".to_string(), bytes),
            Ok("BBBB".to_string())
        );
    }

    #[test]
    fn test_wrong_response_method() {
        let bytes = vec![16, 66, 66, 66, 66, 16, 66, 66, 66, 66];

        assert_eq!(
            CallTicket::<String>::decode_response("AAAA".to_string(), bytes),
            Err(GStdError::Decode(ParseError::from(
                "unexpected response method"
            )))
        );
    }

    #[test]
    fn test_short_response() {
        let bytes = vec![];

        assert_eq!(
            CallTicket::<String>::decode_response("AAAA".to_string(), bytes),
            Err(GStdError::Decode(ParseError::from("response too short")))
        );
    }
}
