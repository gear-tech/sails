use crate::{errors::Result, prelude::*};
use core::marker::PhantomData;
use futures::{Stream, StreamExt};

#[allow(async_fn_in_trait)]
pub trait Listener<E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin>;
}

pub struct RemotingListener<R, E, F> {
    remoting: R,
    f: F,
    _event: PhantomData<E>,
}

impl<R: Listener<Vec<u8>>, E: Decode, F> RemotingListener<R, E, F> {
    pub fn new(remoting: R, f: F) -> Self {
        Self {
            remoting,
            f,
            _event: PhantomData,
        }
    }
}

impl<R: Listener<Vec<u8>>, E, F: Fn(Vec<u8>) -> Result<E>> Listener<E>
    for RemotingListener<R, E, F>
{
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin> {
        let stream = self.remoting.listen().await?;
        let f = &self.f;
        let map = stream.filter_map(move |(actor_id, payload)| async move {
            (f)(payload).ok().map(|e| (actor_id, e))
        });
        Ok(Box::pin(map))
    }
}
