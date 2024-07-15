use crate::{
    errors::{Error, Result, RtlError},
    prelude::*,
};
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

impl<R: Listener<Vec<u8>>, E> RemotingListener<R, E> {
    pub fn new(remoting: R) -> Self {
        Self {
            remoting,
            _event: PhantomData,
        }
    }
}

impl<R: Listener<Vec<u8>>, E: EventIo> Listener<E> for RemotingListener<R, E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin> {
        let stream = self.remoting.listen().await?;
        let map = stream.filter_map(move |(actor_id, payload)| async move {
            E::decode_event(payload).ok().map(|e| (actor_id, e))
        });
        Ok(Box::pin(map))
    }
}

pub trait EventIo: Decode {
    const ROUTE: &'static [u8];
    const EVENT_NAMES: &'static [&'static [u8]];

    fn decode_event(payload: impl AsRef<[u8]>) -> Result<Self> {
        let payload = payload.as_ref();
        if !payload.starts_with(Self::ROUTE) {
            Err(RtlError::EventPrefixMismatches)?;
        }
        let event_bytes = &payload[Self::ROUTE.len()..];
        for (idx, name) in Self::EVENT_NAMES.iter().enumerate() {
            if event_bytes.starts_with(name) {
                let idx = idx as u8;
                let bytes = [&[idx], &event_bytes[name.len()..]].concat();
                let mut event_bytes = &bytes[..];
                return Decode::decode(&mut event_bytes).map_err(Error::Codec);
            }
        }
        Err(RtlError::EventNameIsNotFound)?
    }
}
