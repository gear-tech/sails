use super::calls::{GStdArgs, GStdRemoting};
use crate::{errors::Result, prelude::*};
use core::{
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
            wait_up_to_and_created: Option<(BlockCount, BlockCount)>,
            redirect_on_exit: bool,
        },
        Dummy,
    }
}

impl<T: AsRef<[u8]>> MessageFutureWithRedirect<T> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        future: msg::MessageFuture,
        target: ActorId,
        payload: T,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        #[cfg(not(feature = "ethexe"))] reply_deposit: Option<GasUnit>,
        wait_up_to: Option<BlockCount>,
        redirect_on_exit: bool,
    ) -> Self {
        let wait_up_to_and_created = wait_up_to.map(|wait_up_to| {
            let current_block = gstd::exec::block_height();
            (wait_up_to, current_block)
        });
        Self::Incomplete {
            future,
            target,
            payload,
            #[cfg(not(feature = "ethexe"))]
            gas_limit,
            #[cfg(feature = "ethexe")]
            gas_limit: None,
            value,
            #[cfg(not(feature = "ethexe"))]
            reply_deposit,
            #[cfg(feature = "ethexe")]
            reply_deposit: None,
            wait_up_to_and_created,
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
                            wait_up_to_and_created,
                            ..
                        } = this
                            .as_mut()
                            .project_replace(MessageFutureWithRedirect::Dummy)
                        else {
                            unreachable!("Invalid state during replacement")
                        };
                        gstd::debug!("Redirecting message from {} to {}", target, new_target);
                        // here can insert new target into redirects

                        // Calculate updated `wait_up_to` if provided
                        // wait_up_to = wait_up_to - (current_block - created_block)
                        let wait_up_to =
                            wait_up_to_and_created.map(|(wait_up_to, created_block)| {
                                let current_block = gstd::exec::block_height();
                                wait_up_to
                                    .saturating_sub(current_block.saturating_sub(created_block))
                            });
                        #[cfg(not(feature = "ethexe"))]
                        let args = GStdArgs::default()
                            .with_reply_deposit(reply_deposit)
                            .with_wait_up_to(wait_up_to)
                            .with_redirect_on_exit(true);
                        #[cfg(feature = "ethexe")]
                        let args = GStdArgs::default()
                            .with_wait_up_to(wait_up_to)
                            .with_redirect_on_exit(true);
                        // Get new future
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
