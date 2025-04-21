use super::calls::{GStdArgs, GStdRemoting};
use crate::{collections::BTreeMap, errors::Result, prelude::*};
use core::ops::DerefMut;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, ready},
};
use gstd::msg;
use pin_project_lite::pin_project;

type RedirectMap = BTreeMap<ActorId, ActorId>;

#[cfg(not(target_arch = "wasm32"))]
fn redirect_map() -> impl DerefMut<Target = RedirectMap> {
    use spin::Mutex;

    static MAP: Mutex<RedirectMap> = Mutex::new(RedirectMap::new());
    MAP.lock()
}

// This code relies on the fact contracts are executed in a single-threaded environment
#[cfg(target_arch = "wasm32")]
fn redirect_map() -> impl DerefMut<Target = RedirectMap> {
    static mut MAP: RedirectMap = RedirectMap::new();
    #[allow(static_mut_refs)]
    unsafe {
        &mut MAP
    }
}

pub(crate) fn redirect_target(target: &ActorId) -> ActorId {
    let redirect_map = redirect_map();
    let mut target = target;
    while let Some(redirect) = redirect_map.get(target) {
        target = redirect;
    }
    *target
}

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
            redirect_on_exit: bool,
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
        redirect_on_exit: bool,
    ) -> Self {
        Self::Incomplete {
            future,
            target,
            payload,
            gas_limit,
            value,
            reply_deposit,
            redirect_on_exit,
        }
    }

    #[cfg(feature = "ethexe")]
    pub(crate) fn new(
        future: msg::MessageFuture,
        target: ActorId,
        payload: T,
        value: ValueUnit,
        redirect_on_exit: bool,
    ) -> Self {
        Self::Incomplete {
            future,
            target,
            payload,
            gas_limit: None,
            value,
            reply_deposit: None,
            redirect_on_exit,
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

        let (output, redirect_on_exit) = match this.as_mut().project() {
            Projection::Incomplete {
                future,
                redirect_on_exit,
                ..
            } => (ready!(future.poll(cx)), *redirect_on_exit),
            Projection::Dummy => {
                unreachable!("polled after completion or invalid state")
            }
        };
        // Return if not redirecting on exit
        if !redirect_on_exit {
            return Poll::Ready(output.map_err(Into::into));
        }
        match output {
            Ok(_) => Poll::Ready(output.map_err(Into::into)),
            Err(err) => match err {
                gstd::errors::Error::ErrorReply(
                    error_payload,
                    ErrorReplyReason::UnavailableActor(SimpleUnavailableActorError::ProgramExited),
                ) => {
                    if let Ok(new_target) = ActorId::try_from(error_payload.0.as_ref()) {
                        // Safely extract values by replacing with Dummy
                        let Replace::Incomplete {
                            target,
                            payload,
                            #[cfg(not(feature = "ethexe"))]
                            gas_limit,
                            value,
                            #[cfg(not(feature = "ethexe"))]
                            reply_deposit,
                            ..
                        } = this
                            .as_mut()
                            .project_replace(MessageFutureWithRedirect::Dummy)
                        else {
                            unreachable!("Invalid state during replacement")
                        };
                        gstd::debug!("Redirecting message from {} to {}", target, new_target);
                        // Insert new target into redirects
                        redirect_map().insert(target, new_target);
                        // Get new future
                        #[cfg(not(feature = "ethexe"))]
                        let args = GStdArgs::default()
                            .with_reply_deposit(reply_deposit)
                            .with_redirect_on_exit(true);
                        #[cfg(feature = "ethexe")]
                        let args = GStdArgs::default().with_redirect_on_exit(true);
                        let future_res = GStdRemoting::send_for_reply(
                            new_target,
                            payload,
                            #[cfg(not(feature = "ethexe"))]
                            gas_limit,
                            value,
                            args,
                        );
                        match future_res {
                            Ok(future) => {
                                // Replace the future with a new one
                                _ = this.as_mut().project_replace(future);
                                // Return Pending to allow the new future to be polled
                                Poll::Pending
                            }
                            Err(err) => Poll::Ready(Err(err)),
                        }
                    } else {
                        Poll::Ready(Err(gstd::errors::Error::ErrorReply(
                            error_payload,
                            ErrorReplyReason::UnavailableActor(
                                SimpleUnavailableActorError::ProgramExited,
                            ),
                        )
                        .into()))
                    }
                }
                _ => Poll::Ready(Err(err.into())),
            },
        }
    }
}
