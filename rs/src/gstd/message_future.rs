use super::calls::{GStdArgs, GStdRemoting};
use crate::{collections::BTreeMap, errors::Result, prelude::*, rc::Rc};
use core::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, ready},
};
use gstd::msg;
use pin_project_lite::pin_project;

pin_project! {
    #[project = Projection]
    #[project_replace = Replace]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub(crate) enum MessageFutureWithRedirect<T: AsRef<[u8]>> {
        Incomplete {
            #[pin]
            future: msg::MessageFuture,
            target: ActorId,
            payload: T,
            gas_limit: Option<GasUnit>,
            value: ValueUnit,
            reply_deposit: Option<GasUnit>,
            redirects: Rc<RefCell<BTreeMap<ActorId, ActorId>>>,
        },
        Dummy,
    }
}

impl<T: AsRef<[u8]>> MessageFutureWithRedirect<T> {
    #[cfg(not(feature = "ethexe"))]
    pub(crate) fn new(
        future: msg::MessageFuture,
        target: ActorId,
        payload: T,
        gas_limit: Option<GasUnit>,
        value: ValueUnit,
        reply_deposit: Option<GasUnit>,
        redirects: Rc<RefCell<BTreeMap<ActorId, ActorId>>>,
    ) -> Self {
        Self::Incomplete {
            future,
            target,
            payload,
            gas_limit,
            value,
            reply_deposit,
            redirects,
        }
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn new(
        future: msg::MessageFuture,
        target: ActorId,
        payload: T,
        value: ValueUnit,
        redirects: Rc<RefCell<BTreeMap<ActorId, ActorId>>>,
    ) -> Self {
        Self::Incomplete {
            future,
            target,
            payload,
            gas_limit: None,
            value,
            reply_deposit: None,
            redirects,
        }
    }
}

impl<T: AsRef<[u8]>> futures::future::FusedFuture for MessageFutureWithRedirect<T> {
    fn is_terminated(&self) -> bool {
        match self {
            Self::Incomplete { future, .. } => future.is_terminated(),
            Self::Dummy => true,
        }
    }
}

impl<T: AsRef<[u8]>> Future for MessageFutureWithRedirect<T> {
    type Output = Result<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self;

        let output = match this.as_mut().project() {
            Projection::Incomplete { future, .. } => {
                ready!(future.poll(cx))
            }
            Projection::Dummy => {
                unreachable!("polled after completion or invalid state")
            }
        };
        match output {
            Ok(_) => Poll::Ready(output.map_err(Into::into)),
            Err(err) => match err {
                gstd::errors::Error::ErrorReply(error_payload, ErrorReplyReason::InactiveActor) => {
                    if let Ok(new_target) = ActorId::try_from(error_payload.0.as_ref()) {
                        // Safely extract values by replacing with Dummy
                        let Replace::Incomplete {
                            target,
                            payload,
                            gas_limit,
                            value,
                            reply_deposit,
                            redirects,
                            ..
                        } = this
                            .as_mut()
                            .project_replace(MessageFutureWithRedirect::Dummy)
                        else {
                            unreachable!("Invalid state during replacement")
                        };
                        // Insert new target into redirects
                        redirects.borrow_mut().insert(target, new_target);
                        // Get new future
                        #[cfg(not(feature = "ethexe"))]
                        let args = GStdArgs::default().with_reply_deposit(reply_deposit);
                        #[cfg(feature = "ethexe")]
                        let args = GStdArgs::default();
                        let future = GStdRemoting::send_for_reply(
                            new_target,
                            payload,
                            #[cfg(not(feature = "ethexe"))]
                            gas_limit,
                            value,
                            args,
                            redirects,
                        )
                        .unwrap();
                        // Replace the future with a new one
                        this.set(future);
                        // Immediately poll the new future instead of returning `Poll::Pending`
                        match this.as_mut().project() {
                            Projection::Incomplete { future, .. } => {
                                let output = ready!(future.poll(cx));
                                return Poll::Ready(output.map_err(Into::into));
                            }
                            Projection::Dummy => {
                                unreachable!("Invalid state during replacement")
                            }
                        }
                    } else {
                        Poll::Ready(Err(gstd::errors::Error::ErrorReply(
                            error_payload,
                            ErrorReplyReason::InactiveActor,
                        )
                        .into()))
                    }
                }
                _ => Poll::Ready(Err(err.into())),
            },
        }
    }
}
