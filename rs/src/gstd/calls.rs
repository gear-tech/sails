use super::message_future::MessageFutureExtended;
use crate::{
    calls::{Action, CallOneWay, Remoting, RemotingMessage},
    errors::Result,
    futures::FutureExt,
    prelude::*,
};
use core::future::Future;
use gstd::{msg, prog};

#[derive(Default)]
pub struct GStdArgs {
    wait_up_to: Option<BlockCount>,
    #[cfg(not(feature = "ethexe"))]
    reply_deposit: Option<GasUnit>,
    #[cfg(not(feature = "ethexe"))]
    reply_hook: Option<Box<dyn FnOnce() + Send + 'static>>,
    redirect_on_exit: bool,
}

impl GStdArgs {
    pub fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self {
        Self { wait_up_to, ..self }
    }

    pub fn with_redirect_on_exit(self, redirect_on_exit: bool) -> Self {
        Self {
            redirect_on_exit,
            ..self
        }
    }

    pub fn wait_up_to(&self) -> Option<BlockCount> {
        self.wait_up_to
    }

    pub fn redirect_on_exit(&self) -> bool {
        self.redirect_on_exit
    }
}

#[cfg(not(feature = "ethexe"))]
impl GStdArgs {
    pub fn with_reply_deposit(self, reply_deposit: Option<GasUnit>) -> Self {
        Self {
            reply_deposit,
            ..self
        }
    }

    pub fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self {
        Self {
            reply_hook: Some(Box::new(f)),
            ..self
        }
    }

    pub fn reply_deposit(&self) -> Option<GasUnit> {
        self.reply_deposit
    }
}

#[derive(Debug, Default, Clone)]
pub struct GStdRemoting;

impl GStdRemoting {
    pub fn new() -> Self {
        Self
    }
}

impl Remoting for GStdRemoting {
    type Args = GStdArgs;

    async fn activate(
        self,
        code_id: CodeId,
        salt: impl AsRef<[u8]>,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        #[allow(unused_variables)] args: GStdArgs,
    ) -> Result<impl Future<Output = Result<(ActorId, Vec<u8>)>>> {
        #[cfg(not(feature = "ethexe"))]
        let mut reply_future = if let Some(gas_limit) = gas_limit {
            prog::create_program_bytes_with_gas_for_reply(
                code_id,
                salt,
                payload,
                gas_limit,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        } else {
            prog::create_program_bytes_for_reply(
                code_id,
                salt,
                payload,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        };
        #[cfg(feature = "ethexe")]
        let mut reply_future = prog::create_program_bytes_for_reply(code_id, salt, payload, value)?;

        reply_future = reply_future.up_to(args.wait_up_to)?;

        #[cfg(not(feature = "ethexe"))]
        if let Some(reply_hook) = args.reply_hook {
            reply_future = reply_future.handle_reply(reply_hook)?;
        }

        let reply_future = reply_future.map(|result| result.map_err(Into::into));
        Ok(reply_future)
    }

    async fn message(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GStdArgs,
    ) -> Result<impl Future<Output = Result<Vec<u8>>>> {
        let reply_future = send_for_reply(
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            value,
            args,
        )?;
        Ok(reply_future)
    }

    async fn query(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: GStdArgs,
    ) -> Result<Vec<u8>> {
        let reply_future = send_for_reply(
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            value,
            args,
        )?;
        let reply = reply_future.await?;
        Ok(reply)
    }
}

#[cfg(not(feature = "ethexe"))]
pub(crate) fn send_for_reply_future(
    target: ActorId,
    payload: &[u8],
    gas_limit: Option<GasUnit>,
    value: ValueUnit,
    args: GStdArgs,
) -> Result<msg::MessageFuture> {
    // here can be a redirect target
    let mut message_future = if let Some(gas_limit) = gas_limit {
        msg::send_bytes_with_gas_for_reply(
            target,
            payload,
            gas_limit,
            value,
            args.reply_deposit.unwrap_or_default(),
        )?
    } else {
        msg::send_bytes_for_reply(
            target,
            payload,
            value,
            args.reply_deposit.unwrap_or_default(),
        )?
    };

    message_future = message_future.up_to(args.wait_up_to)?;

    if let Some(reply_hook) = args.reply_hook {
        message_future = message_future.handle_reply(reply_hook)?;
    }
    Ok(message_future)
}

#[cfg(feature = "ethexe")]
pub(crate) fn send_for_reply_future(
    target: ActorId,
    payload: &[u8],
    value: ValueUnit,
    args: GStdArgs,
) -> Result<msg::MessageFuture> {
    // here can be a redirect target
    let mut message_future = msg::send_bytes_for_reply(target, payload, value)?;

    message_future = message_future.up_to(args.wait_up_to)?;

    Ok(message_future)
}

pub(crate) fn send_for_reply<T: AsRef<[u8]>>(
    target: ActorId,
    payload: T,
    #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
    value: ValueUnit,
    args: GStdArgs,
) -> Result<MessageFutureExtended<T>> {
    #[cfg(not(feature = "ethexe"))]
    let reply_deposit = args.reply_deposit;
    let wait_up_to = args.wait_up_to;
    let redirect_on_exit = args.redirect_on_exit;
    let message_future = send_for_reply_future(
        target,
        payload.as_ref(),
        #[cfg(not(feature = "ethexe"))]
        gas_limit,
        value,
        args,
    )?;

    if redirect_on_exit {
        Ok(MessageFutureExtended::with_redirect(
            message_future,
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            value,
            #[cfg(not(feature = "ethexe"))]
            reply_deposit,
            wait_up_to,
        ))
    } else {
        Ok(MessageFutureExtended::without_redirect(message_future))
    }
}

pub trait WithArgs {
    fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self;

    #[cfg(not(feature = "ethexe"))]
    fn with_reply_deposit(self, reply_deposit: Option<GasUnit>) -> Self;

    #[cfg(not(feature = "ethexe"))]
    fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self;

    fn with_redirect_on_exit(self, redirect_on_exit: bool) -> Self;
}

impl<T> WithArgs for T
where
    T: Action<Args = GStdArgs>,
{
    fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self {
        self.with_args(|args| args.with_wait_up_to(wait_up_to))
    }

    #[cfg(not(feature = "ethexe"))]
    fn with_reply_deposit(self, reply_deposit: Option<GasUnit>) -> Self {
        self.with_args(|args| args.with_reply_deposit(reply_deposit))
    }

    #[cfg(not(feature = "ethexe"))]
    fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self {
        self.with_args(|args| args.with_reply_hook(f))
    }

    /// Set `redirect_on_exit` flag to `true``
    ///
    /// This flag is used to redirect a message to a new program when the target program exits
    /// with an inheritor.
    ///
    /// WARNING: When this flag is set, the message future captures the payload and other arguments,
    /// potentially resulting in multiple messages being sent. This can lead to increased gas consumption.
    ///
    /// This flag is set to `false`` by default.
    fn with_redirect_on_exit(self, redirect_on_exit: bool) -> Self {
        self.with_args(|args| args.with_redirect_on_exit(redirect_on_exit))
    }
}

impl CallOneWay for GStdRemoting {
    type Args = GStdArgs;

    fn send_one_way(
        self,
        target: ActorId,
        payload: impl AsRef<[u8]>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        _args: Self::Args,
    ) -> Result<MessageId> {
        let payload = payload.as_ref();
        #[cfg(not(feature = "ethexe"))]
        if let Some(gas_limit) = gas_limit {
            gcore::msg::send_with_gas(target, payload, gas_limit, value).map_err(Into::into)
        } else {
            gcore::msg::send(target, payload, value).map_err(Into::into)
        }

        #[cfg(feature = "ethexe")]
        gcore::msg::send(target, payload, value).map_err(Into::into)
    }
}

impl crate::calls::MessageFuture for MessageFutureExtended<Vec<u8>> {
    type Error = crate::errors::Error;

    fn message_id(&self) -> MessageId {
        match self {
            Self::NonRedirect { message_future } => message_future.waiting_reply_to,
            Self::Redirect { message_future, .. } => message_future.waiting_reply_to,
            Self::Dummy => MessageId::zero(),
        }
    }
}

impl RemotingMessage for GStdRemoting {
    type Args = GStdArgs;
    type MessageFuture = MessageFutureExtended<Vec<u8>>;

    fn send_message(
        self,
        target: ActorId,
        payload: Vec<u8>,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        args: Self::Args,
    ) -> Result<Self::MessageFuture> {
        let message_future = send_for_reply(target, payload, gas_limit, value, args)?;
        Ok(message_future)
    }
}
