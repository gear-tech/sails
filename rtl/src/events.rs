use crate::{errors::Result, ActorId, Box, Vec};
use core::marker::PhantomData;
use futures::{Stream, StreamExt};

#[allow(async_fn_in_trait)]
pub trait Listener<E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin>;
}

pub trait DecodeEvent: Sized {
    fn decode_event(payload: impl AsRef<[u8]>) -> Result<Self>;
}

pub struct RemotingListener<R, E> {
    remoting: R,
    _event: PhantomData<E>,
}

impl<R: Listener<Vec<u8>>, E: DecodeEvent> RemotingListener<R, E> {
    pub fn new(remoting: R) -> Self {
        Self {
            remoting,
            _event: PhantomData,
        }
    }
}

impl<R: Listener<Vec<u8>>, E: DecodeEvent> Listener<E> for RemotingListener<R, E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin> {
        let stream = self.remoting.listen().await?;
        let map = stream.filter_map(move |(actor_id, payload)| async move {
            E::decode_event(&payload).ok().map(|e| (actor_id, e))
        });
        Ok(Box::pin(map))
    }
}
