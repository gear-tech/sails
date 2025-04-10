use super::message_future::MessageFutureWithRedirect;
use crate::{
    calls::Remoting, collections::BTreeMap, errors::Result, futures::FutureExt, prelude::*, rc::Rc,
};
use core::{cell::RefCell, future::Future};
use gstd::{msg, prog};

#[derive(Default)]
pub struct GStdArgs {
    wait_up_to: Option<BlockCount>,
    #[cfg(not(feature = "ethexe"))]
    reply_deposit: Option<GasUnit>,
    #[cfg(not(feature = "ethexe"))]
    reply_hook: Option<Box<dyn FnOnce() + Send + 'static>>,
}

#[cfg(not(feature = "ethexe"))]
impl GStdArgs {
    pub fn with_wait_up_to(mut self, block_count: Option<BlockCount>) -> Self {
        self.wait_up_to = block_count;
        self
    }

    pub fn with_reply_deposit(mut self, reply_deposit: Option<GasUnit>) -> Self {
        self.reply_deposit = reply_deposit;
        self
    }

    pub fn with_reply_hook<F: FnOnce() + Send + 'static>(mut self, f: F) -> Self {
        self.reply_hook = Some(Box::new(f));
        self
    }

    pub fn wait_up_to(&self) -> Option<BlockCount> {
        self.wait_up_to
    }

    pub fn reply_deposit(&self) -> Option<GasUnit> {
        self.reply_deposit
    }
}

#[derive(Debug, Default, Clone)]
pub struct GStdRemoting {
    redirects: Rc<RefCell<BTreeMap<ActorId, ActorId>>>,
}

impl GStdRemoting {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn send_for_reply<T: AsRef<[u8]>>(
        target: ActorId,
        payload: T,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        #[allow(unused_variables)] args: GStdArgs,
        redirects: Rc<RefCell<BTreeMap<ActorId, ActorId>>>,
    ) -> Result<MessageFutureWithRedirect<T>> {
        let target = GStdRemoting::redirect_target(&redirects.borrow(), &target);
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
            redirects,
        ))
    }

    fn redirect_target(redirects: &BTreeMap<ActorId, ActorId>, target: &ActorId) -> ActorId {
        let mut target = target;
        while let Some(redirect) = redirects.get(&target) {
            target = redirect;
        }
        *target
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
            self.redirects,
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
            self.redirects,
        )?;
        let reply = reply_future.await?;
        Ok(reply)
    }
}
