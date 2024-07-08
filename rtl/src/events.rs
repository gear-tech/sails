use crate::{errors::Result, ActorId, Box, Vec};
use core::marker::PhantomData;
use futures::{Stream, StreamExt};

#[allow(async_fn_in_trait)]
pub trait Listener<E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin>;
}

pub struct RemotingListener<R, E> {
    remoting: R,
    _event: PhantomData<E>,
}

impl<R: Listener<Vec<u8>>, E: TryFrom<Vec<u8>>> RemotingListener<R, E> {
    pub fn new(remoting: R) -> Self {
        Self {
            remoting,
            _event: PhantomData,
        }
    }
}

impl<R: Listener<Vec<u8>>, E: TryFrom<Vec<u8>>> Listener<E> for RemotingListener<R, E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin> {
        let stream = self.remoting.listen().await?;
        let map = stream.filter_map(move |(actor_id, payload)| async move {
            E::try_from(payload).ok().map(|e| (actor_id, e))
        });
        Ok(Box::pin(map))
    }
}
