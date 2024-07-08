use crate::{errors::Result, ActorId, Box, Decode, Vec};
use core::marker::PhantomData;
use futures::{Stream, StreamExt};

#[allow(async_fn_in_trait)]
pub trait EventListener<E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin>;
}

pub struct RemotingListener<R, E> {
    remoting: R,
    route: &'static [u8],
    event_names: &'static [&'static [u8]],
    _event: PhantomData<E>,
}

impl<R: EventListener<Vec<u8>>, E: Decode> RemotingListener<R, E> {
    pub fn new(remoting: R, route: &'static [u8], event_names: &'static [&'static [u8]]) -> Self {
        Self {
            remoting,
            route,
            event_names,
            _event: PhantomData,
        }
    }
}

impl<R: EventListener<Vec<u8>>, E: Decode> EventListener<E> for RemotingListener<R, E> {
    async fn listen(&mut self) -> Result<impl Stream<Item = (ActorId, E)> + Unpin> {
        let stream = self.remoting.listen().await?;
        let route: &'static [u8] = self.route;
        let event_names: &'static [&'static [u8]] = self.event_names;
        let map = stream.filter_map(move |(actor_id, payload)| async move {
            if !payload.starts_with(route) {
                return None;
            }
            let event_bytes = &payload[route.len()..];
            for (idx, name) in event_names.iter().enumerate() {
                if event_bytes.starts_with(name) {
                    let idx = idx as u8;
                    let bytes = [&[idx], &event_bytes[name.len()..]].concat();
                    let mut event_bytes = &bytes[..];
                    return E::decode(&mut event_bytes).ok().map(|e| (actor_id, e));
                }
            }
            None
        });
        Ok(Box::pin(map))
    }
}
