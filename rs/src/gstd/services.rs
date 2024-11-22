#[cfg(not(target_arch = "wasm32"))]
use crate::boxed::Box;
use crate::prelude::*;
use crate::{collections::BTreeMap, MessageId, Vec};
use core::ops::DerefMut;

#[cfg(not(target_arch = "wasm32"))]
fn get_message_id_to_service_route_map(
) -> impl DerefMut<Target = BTreeMap<MessageId, Vec<&'static [u8]>>> {
    use spin::Mutex;

    static MESSAGE_ID_TO_SERVICE_ROUTE: Mutex<BTreeMap<MessageId, Vec<&'static [u8]>>> =
        Mutex::new(BTreeMap::new());

    MESSAGE_ID_TO_SERVICE_ROUTE.lock()
}

#[cfg(target_arch = "wasm32")]
fn get_message_id_to_service_route_map(
) -> impl DerefMut<Target = BTreeMap<MessageId, Vec<&'static [u8]>>> {
    static mut MESSAGE_ID_TO_SERVICE_ROUTE: BTreeMap<MessageId, Vec<&'static [u8]>> =
        BTreeMap::new();

    // SAFETY: only for wasm32 target
    #[allow(static_mut_refs)]
    unsafe {
        &mut MESSAGE_ID_TO_SERVICE_ROUTE
    }
}

pub struct ServiceExposure<T, E> {
    message_id: MessageId,
    route: &'static [u8],
    #[cfg(not(target_arch = "wasm32"))]
    inner_ptr: *const T, // Prevent exposure being Send + Sync
    #[cfg(not(target_arch = "wasm32"))]
    pub inner: Box<T>,
    #[cfg(target_arch = "wasm32")]
    pub inner: T,
    pub extend: E,
}

impl<T, E> ServiceExposure<T, E> {
    pub fn new(message_id: MessageId, route: &'static [u8], inner: T, extend: E) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let inner = Box::new(inner);

        Self {
            message_id,
            route,
            #[cfg(not(target_arch = "wasm32"))]
            inner_ptr: inner.as_ref() as *const T,
            inner,
            extend,
        }
    }

    pub async fn handle(&mut self, input: &[u8]) -> (Vec<u8>, u128)
    where
        T: ServiceHandle + Service<Extend = E>,
        E: ServiceHandle,
    {
        if let Some(result) = self.try_handle_inner(input).await {
            result
        } else if let Some(result) = self.extend.try_handle(input).await {
            result
        } else {
            let mut __input = input;
            let input: String = Decode::decode(&mut __input).unwrap_or_else(|_| {
                if input.len() <= 8 {
                    format!("0x{}", hex::encode(input))
                } else {
                    format!(
                        "0x{}..{}",
                        hex::encode(&input[..4]),
                        hex::encode(&input[input.len() - 4..])
                    )
                }
            });
            panic!("Unknown request: {}", input)
        }
    }

    async fn try_handle_inner(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)>
    where
        T: ServiceHandle + Service<Extend = E>,
    {
        let _scope = ExposureCallScope::new2(self);
        self.inner.try_handle(input).await
    }

    // async fn call_scoped<F, P, R>(&mut self, f: F) -> P
    // where
    //     F: FnOnce(&mut T) -> R,
    //     R: IntoFuture<Output = P>,
    //     T: Service2<Extend = E>,
    // {
    //     let _scope = ExposureCallScope::new2(self);
    //     let inner = &mut self.inner;
    //     let future = f(inner).into_future();
    //     future.await
    // }

    pub fn extend(&self) -> &E {
        &self.extend
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_event_listener<'e, V>(
        &self,
        listener: impl FnMut(&V) + 'e,
    ) -> crate::gstd::events::EventListenerGuard<'e> {
        if core::mem::size_of_val(self.inner.as_ref()) == 0 {
            panic!("setting event listener on a zero-sized service is not supported for now");
        }
        let service_ptr = self.inner_ptr as usize;
        let listener: Box<dyn FnMut(&V)> = Box::new(listener);
        let listener = Box::new(listener);
        let listener_ptr = Box::into_raw(listener) as usize;
        crate::gstd::events::EventListenerGuard::new(service_ptr, listener_ptr)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T, E> Drop for ServiceExposure<T, E> {
    fn drop(&mut self) {
        let service_ptr = self.inner_ptr as usize;
        let mut event_listeners = crate::gstd::events::event_listeners().lock();
        if event_listeners.remove(&service_ptr).is_some() {
            panic!("there should be no any event listeners left by this time");
        }
    }
}

// pub trait Service {
//     type Exposure: Exposure;

//     fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure;
// }

pub trait Service {
    type Exposure: Exposure2;
    type Extend;

    fn expose(self, message_id: MessageId, route: &'static [u8]) -> Self::Exposure;
}

#[allow(async_fn_in_trait)]
pub trait ServiceHandle {
    async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)>;
}

#[allow(async_fn_in_trait)]
pub trait Exposure2 {
    type Service;

    fn message_id(&self) -> MessageId;
    fn route(&self) -> &'static [u8];
}

impl<T: Service> Exposure2 for ServiceExposure<T, T::Extend> {
    type Service = T;

    fn message_id(&self) -> MessageId {
        self.message_id
    }

    fn route(&self) -> &'static [u8] {
        self.route
    }
}

impl<T, E> ServiceHandle for ServiceExposure<T, E>
where
    T: ServiceHandle + Service<Extend = E>,
{
    async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
        self.try_handle_inner(input).await
    }
}

pub trait Exposure {
    fn message_id(&self) -> MessageId;
    fn route(&self) -> &'static [u8];
}

#[derive(Debug, Clone, Copy)]
pub struct ExposureContext {
    message_id: MessageId,
    route: &'static [u8],
}

impl ExposureContext {
    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    pub fn route(&self) -> &'static [u8] {
        self.route
    }
}

pub(crate) fn exposure_context(message_id: MessageId) -> ExposureContext {
    let map = get_message_id_to_service_route_map();
    let route = map
        .get(&message_id)
        .and_then(|routes| routes.last().copied())
        .unwrap_or_else(|| {
            panic!(
                "Exposure context is not found for message id {:?}",
                message_id
            )
        });
    ExposureContext { message_id, route }
}

pub struct ExposureCallScope {
    message_id: MessageId,
    route: &'static [u8],
}

impl ExposureCallScope {
    pub fn new(exposure: &impl Exposure) -> Self {
        let mut map = get_message_id_to_service_route_map();
        let routes = map.entry(exposure.message_id()).or_default();
        routes.push(exposure.route());
        Self {
            message_id: exposure.message_id(),
            route: exposure.route(),
        }
    }

    pub fn new2(exposure: &impl Exposure2) -> Self {
        let mut map = get_message_id_to_service_route_map();
        let routes = map.entry(exposure.message_id()).or_default();
        routes.push(exposure.route());
        Self {
            message_id: exposure.message_id(),
            route: exposure.route(),
        }
    }
}

impl Drop for ExposureCallScope {
    fn drop(&mut self) {
        let mut map = get_message_id_to_service_route_map();
        let routes = map
            .get_mut(&self.message_id)
            .unwrap_or_else(|| unreachable!("Entry for message should always exist"));
        let route = routes
            .pop()
            .unwrap_or_else(|| unreachable!("Route should always exist"));
        if route != self.route {
            unreachable!("Route should always match");
        }
        if routes.is_empty() {
            map.remove(&self.message_id);
        }
    }
}

impl ServiceHandle for () {
    async fn try_handle(&mut self, _input: &[u8]) -> Option<(Vec<u8>, u128)> {
        None
    }
}

impl<T1: ServiceHandle, T2: ServiceHandle> ServiceHandle for (T1, T2) {
    async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
        if let Some(result) = self.0.try_handle(input).await {
            Some(result)
        } else {
            self.1.try_handle(input).await
        }
    }
}

// todo: make macro_rules
impl<T1: ServiceHandle, T2: ServiceHandle, T3: ServiceHandle> ServiceHandle for (T1, T2, T3) {
    async fn try_handle(&mut self, input: &[u8]) -> Option<(Vec<u8>, u128)> {
        if let Some(result) = self.0.try_handle(input).await {
            Some(result)
        } else if let Some(result) = self.1.try_handle(input).await {
            Some(result)
        } else {
            self.2.try_handle(input).await
        }
    }
}
