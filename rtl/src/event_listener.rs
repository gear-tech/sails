use crate::{
    errors::{Result, RtlError},
    ActorId, Decode, Vec,
};

#[allow(async_fn_in_trait)]
pub trait EventSubscriber {
    async fn subscribe(&mut self) -> Result<impl EventListener>;
}

#[allow(async_fn_in_trait)]
pub trait EventListener {
    async fn next_event(
        &mut self,
        predicate: impl Fn(&(ActorId, Vec<u8>)) -> bool,
    ) -> Result<(ActorId, Vec<u8>)>;
}

#[allow(async_fn_in_trait)]
pub trait Subscribe<E: Decode> {
    async fn subscribe(&mut self, source: ActorId) -> Result<impl Listen<E>>;
}

#[allow(async_fn_in_trait)]
pub trait Listen<E: Decode> {
    async fn next_event(&mut self) -> Result<E>;
}

pub struct RemotingSubscribe<R: EventSubscriber> {
    remoting: R,
    route: &'static [u8],
    event_names: &'static [&'static [u8]],
}

impl<R: EventSubscriber> RemotingSubscribe<R> {
    pub fn new(remoting: R, route: &'static [u8], event_names: &'static [&'static [u8]]) -> Self {
        Self {
            remoting,
            route,
            event_names,
        }
    }
}

impl<R: EventSubscriber, E: Decode> Subscribe<E> for RemotingSubscribe<R> {
    async fn subscribe(&mut self, source: ActorId) -> Result<impl Listen<E>> {
        let listener = self.remoting.subscribe().await?;
        Ok(ClientListener::new(
            listener,
            self.route,
            self.event_names,
            source,
        ))
    }
}

pub struct ClientListener<L: EventListener> {
    listener: L,
    route: &'static [u8],
    event_names: &'static [&'static [u8]],
    source: ActorId,
}

impl<L: EventListener> ClientListener<L> {
    pub fn new(
        listener: L,
        route: &'static [u8],
        event_names: &'static [&'static [u8]],
        source: ActorId,
    ) -> Self {
        Self {
            listener,
            route,
            event_names,
            source,
        }
    }
}

impl<L: EventListener, E: Decode> Listen<E> for ClientListener<L> {
    async fn next_event(&mut self) -> Result<E> {
        let (_, payload) = self
            .listener
            .next_event(|(source, payload)| {
                *source == self.source && payload.starts_with(self.route)
            })
            .await?;
        let event_bytes = &payload[self.route.len()..];
        for (idx, name) in self.event_names.iter().enumerate() {
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