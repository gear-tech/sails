use super::message_future::MessageFutureWithRedirect;
use crate::{
    calls::{Action, Remoting},
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
    #[allow(clippy::needless_update)]
    pub fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self {
        Self { wait_up_to, ..self }
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn with_reply_deposit(self, reply_deposit: Option<GasUnit>) -> Self {
        Self {
            reply_deposit,
            ..self
        }
    }

    #[cfg(not(feature = "ethexe"))]
    pub fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self {
        Self {
            reply_hook: Some(Box::new(f)),
            ..self
        }
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

    #[cfg(not(feature = "ethexe"))]
    pub fn reply_deposit(&self) -> Option<GasUnit> {
        self.reply_deposit
    }

    pub fn redirect_on_exit(&self) -> bool {
        self.redirect_on_exit
    }
}

#[derive(Debug, Default, Clone)]
pub struct GStdRemoting;

impl GStdRemoting {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn send_for_reply<T: AsRef<[u8]>>(
        target: ActorId,
        payload: T,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        #[allow(unused_variables)] args: GStdArgs,
    ) -> Result<MessageFutureWithRedirect<T>> {
        // here can be a redirect target
        #[cfg(not(feature = "ethexe"))]
        let mut message_future = if let Some(gas_limit) = gas_limit {
            msg::send_bytes_with_gas_for_reply(
                target,
                payload.as_ref(),
                gas_limit,
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        } else {
            msg::send_bytes_for_reply(
                target,
                payload.as_ref(),
                value,
                args.reply_deposit.unwrap_or_default(),
            )?
        };
        #[cfg(feature = "ethexe")]
        let mut message_future = msg::send_bytes_for_reply(target, payload.as_ref(), value)?;

        message_future = message_future.up_to(args.wait_up_to)?;

        #[cfg(not(feature = "ethexe"))]
        if let Some(reply_hook) = args.reply_hook {
            message_future = message_future.handle_reply(reply_hook)?;
        }

        Ok(MessageFutureWithRedirect::new(
            message_future,
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            value,
            #[cfg(not(feature = "ethexe"))]
            args.reply_deposit,
            args.redirect_on_exit,
        ))
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
        let reply_future = GStdRemoting::send_for_reply(
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
        let reply_future = GStdRemoting::send_for_reply(
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

pub trait WithArgs {
    fn with_wait_up_to(self, wait_up_to: Option<BlockCount>) -> Self;

    #[cfg(not(feature = "ethexe"))]
    fn with_reply_deposit(self, reply_deposit: Option<GasUnit>) -> Self;

    #[cfg(not(feature = "ethexe"))]
    fn with_reply_hook<F: FnOnce() + Send + 'static>(self, f: F) -> Self;
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
}
