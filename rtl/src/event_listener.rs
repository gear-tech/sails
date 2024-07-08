use crate::{errors::Result, ActorId, Box, Decode, Vec};
use core::marker::PhantomData;
use futures::{Stream, StreamExt};

#[allow(async_fn_in_trait)]
pub trait EventSubscriber {
    async fn subscribe(&mut self) -> Result<impl Stream<Item = (ActorId, Vec<u8>)> + Unpin>;
}

#[allow(async_fn_in_trait)]
pub trait Subscribe<E: Decode> {
    async fn subscribe(&mut self, source: ActorId) -> Result<impl Stream<Item = E> + Unpin>;
}

pub struct RemotingSubscribe<R, E> {
    remoting: R,
    route: &'static [u8],
    event_names: &'static [&'static [u8]],
    _event: PhantomData<E>,
}

impl<R: EventSubscriber, E: Decode> RemotingSubscribe<R, E> {
    pub fn new(remoting: R, route: &'static [u8], event_names: &'static [&'static [u8]]) -> Self {
        Self {
            remoting,
            route,
            event_names,
            _event: PhantomData,
        }
    }
}

impl<R: EventSubscriber, E: Decode> Subscribe<E> for RemotingSubscribe<R, E> {
    async fn subscribe(&mut self, source: ActorId) -> Result<impl Stream<Item = E> + Unpin> {
        let stream = self.remoting.subscribe().await?;
        let route: &'static [u8] = self.route;
        let event_names: &'static [&'static [u8]] = self.event_names;
        let map = stream.filter_map(move |(actor_id, payload)| async move {
            if actor_id != source || !payload.starts_with(route) {
                return None;
            }
            let event_bytes = &payload[route.len()..];
            for (idx, name) in event_names.iter().enumerate() {
                if event_bytes.starts_with(name) {
                    let idx = idx as u8;
                    let bytes = [&[idx], &event_bytes[name.len()..]].concat();
                    let mut event_bytes = &bytes[..];
                    return E::decode(&mut event_bytes).ok();
                }
            }
            None
        });
        Ok(Box::pin(map))
    }
}
