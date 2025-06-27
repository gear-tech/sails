use super::calls::GStdArgs;
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
    pub(crate) enum MessageFutureExtended<T: AsRef<[u8]>> {
        NonRedirect {
            #[pin]
            message_future: msg::MessageFuture,
        },
        Redirect {
            #[pin]
            message_future: msg::MessageFuture,
            target: ActorId,
            payload: T,
            gas_limit: Option<GasUnit>,
            value: ValueUnit,
            reply_deposit: Option<GasUnit>,
            wait_up_to: Option<BlockCount>,
            created_block: Option<BlockCount>,
        },
        Dummy,
    }
}

impl<T: AsRef<[u8]>> MessageFutureExtended<T> {
    pub(crate) fn without_redirect(message_future: msg::MessageFuture) -> Self {
        Self::NonRedirect { message_future }
    }

    pub(crate) fn with_redirect(
        message_future: msg::MessageFuture,
        target: ActorId,
        payload: T,
        #[cfg(not(feature = "ethexe"))] gas_limit: Option<GasUnit>,
        value: ValueUnit,
        #[cfg(not(feature = "ethexe"))] reply_deposit: Option<GasUnit>,
        wait_up_to: Option<BlockCount>,
    ) -> Self {
        let created_block = wait_up_to.map(|_| gstd::exec::block_height());
        Self::Redirect {
            message_future,
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
            wait_up_to,
            created_block,
        }
    }
}

impl<T: AsRef<[u8]>> futures::future::FusedFuture for MessageFutureExtended<T> {
    fn is_terminated(&self) -> bool {
        match self {
            Self::NonRedirect { message_future } => message_future.is_terminated(),
            Self::Redirect { message_future, .. } => message_future.is_terminated(),
            Self::Dummy => true,
        }
    }
}

impl<T: AsRef<[u8]>> Future for MessageFutureExtended<T> {
    type Output = Result<Vec<u8>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self;

        let output = match this.as_mut().project() {
            Projection::NonRedirect { message_future } => {
                // Return if not redirecting on exit
                let output = ready!(message_future.poll(cx));
                return Poll::Ready(output.map_err(Into::into));
            }
            Projection::Redirect { message_future, .. } => {
                ready!(message_future.poll(cx))
            }
            Projection::Dummy => {
                unreachable!("polled after completion or invalid state")
            }
        };
        match output {
            Ok(res) => Poll::Ready(Ok(res)),
            Err(err) => match err {
                gstd::errors::Error::ErrorReply(
                    error_payload,
                    ErrorReplyReason::UnavailableActor(SimpleUnavailableActorError::ProgramExited),
                ) => {
                    if let Ok(new_target) = ActorId::try_from(error_payload.0.as_ref()) {
                        // Safely extract values by replacing with Dummy
                        let Replace::Redirect {
                            target: _target,
                            payload,
                            #[cfg(not(feature = "ethexe"))]
                            gas_limit,
                            value,
                            #[cfg(not(feature = "ethexe"))]
                            reply_deposit,
                            wait_up_to,
                            created_block,
                            ..
                        } = this.as_mut().project_replace(MessageFutureExtended::Dummy)
                        else {
                            unreachable!("Invalid state during replacement")
                        };
                        gstd::debug!("Redirecting message from {_target} to {new_target}");
                        // here can insert new target into redirects

                        // Calculate updated `wait_up_to` if provided
                        // wait_up_to = wait_up_to - (current_block - created_block)
                        let wait_up_to = wait_up_to.and_then(|wait_up_to| {
                            created_block.map(|created_block| {
                                let current_block = gstd::exec::block_height();
                                wait_up_to
                                    .saturating_sub(current_block.saturating_sub(created_block))
                            })
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
                        let future_res = super::calls::send_for_reply(
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
