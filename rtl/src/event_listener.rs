use scale_info::StaticTypeInfo;

use crate::{
    errors::{Result, RtlError},
    gstd::events::encoded_event_names,
    ActorId, Decode, MessageId, Vec,
};

#[allow(async_fn_in_trait)]
pub trait EventSubscriber {
    async fn subscribe(&mut self) -> Result<impl EventListener>;
}

#[allow(async_fn_in_trait)]
pub trait EventListener {
    async fn next_event<T>(
        &mut self,
        predicate: impl Fn((ActorId, MessageId, Vec<u8>)) -> Option<T>,
    ) -> Result<T>;
}

#[allow(async_fn_in_trait)]
pub trait Subscribe<E: StaticTypeInfo + Decode> {
    async fn subscribe(&mut self, target: ActorId) -> Result<impl Listen<E>>;
}

#[allow(async_fn_in_trait)]
pub trait Listen<E: StaticTypeInfo + Decode> {
    async fn next_event(&mut self) -> Result<E>;
}

pub struct RemotingSubscribe<R: EventSubscriber> {
    remoting: R,
    route: &'static [u8],
}

impl<R: EventSubscriber> RemotingSubscribe<R> {
    pub fn new(remoting: R, route: &'static [u8]) -> Self {
        Self { remoting, route }
    }
}

impl<R: EventSubscriber, E: StaticTypeInfo + Decode> Subscribe<E> for RemotingSubscribe<R> {
    async fn subscribe(&mut self, target: ActorId) -> Result<impl Listen<E>> {
        let listener = self.remoting.subscribe().await?;
        Ok(ClientListener::new(listener, self.route, target))
    }
}

pub struct ClientListener<L: EventListener> {
    listener: L,
    route: &'static [u8],
    target: ActorId,
}

impl<L: EventListener> ClientListener<L> {
    pub fn new(listener: L, route: &'static [u8], target: ActorId) -> Self {
        Self {
            listener,
            route,
            target,
        }
    }
}

impl<L: EventListener, E: StaticTypeInfo + Decode> Listen<E> for ClientListener<L> {
    async fn next_event(&mut self) -> Result<E> {
        let vec = self
            .listener
            .next_event(|e| {
                if e.0 != self.target || !e.2.starts_with(self.route) {
                    return None;
                }
                Some(e.2)
            })
            .await?;
        let event_bytes = &vec[self.route.len()..];
        let event_names = encoded_event_names::<E>()?;
        for (idx, name) in event_names.iter().enumerate() {
            if event_bytes.starts_with(name) {
                let idx = idx as u8;
                let bytes = [&[idx], &event_bytes[name.len()..]].concat();
                let mut event_bytes = &bytes[..];
                return Ok(E::decode(&mut event_bytes)?);
            }
        }
        Err(RtlError::EventNameIsNotFound)?
    }
}
