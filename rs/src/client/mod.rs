use crate::prelude::*;
use core::{
    any::TypeId,
    error::Error,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;

#[cfg(feature = "gtest")]
#[cfg(not(target_arch = "wasm32"))]
mod gtest_env;
#[cfg(feature = "gtest")]
#[cfg(not(target_arch = "wasm32"))]
pub use gtest_env::{BlockRunMode, GtestEnv, GtestError, GtestParams};

#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
mod gclient_env;
#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
pub use gclient_env::{GclientEnv, GclientError, GclientParams};

mod gstd_env;
pub use gstd_env::{GstdEnv, GstdParams};

pub(crate) const PENDING_CALL_INVALID_STATE: &str =
    "PendingCall polled after completion or invalid state";
pub(crate) const PENDING_CTOR_INVALID_STATE: &str =
    "PendingCtor polled after completion or invalid state";

pub trait GearEnv: Clone {
    type Params: Default;
    type Error: Error;
    type MessageState;

    fn deploy<P: Program>(&self, code_id: CodeId, salt: Vec<u8>) -> Deployment<P, Self> {
        Deployment::new(self.clone(), code_id, salt)
    }
}

pub trait Program: Sized {
    fn deploy(code_id: CodeId, salt: Vec<u8>) -> Deployment<Self, GstdEnv> {
        Deployment::new(GstdEnv, code_id, salt)
    }

    fn client(program_id: ActorId) -> Actor<Self, GstdEnv> {
        Actor::new(GstdEnv, program_id)
    }
}

pub type Route = &'static str;

#[derive(Debug, Clone)]
pub struct Deployment<A, E: GearEnv = GstdEnv> {
    env: E,
    code_id: CodeId,
    salt: Vec<u8>,
    _phantom: PhantomData<A>,
}

impl<A, E: GearEnv> Deployment<A, E> {
    pub fn new(env: E, code_id: CodeId, salt: Vec<u8>) -> Self {
        Deployment {
            env,
            code_id,
            salt,
            _phantom: PhantomData,
        }
    }

    pub fn with_env<N: GearEnv>(self, env: &N) -> Deployment<A, N> {
        let Self {
            env: _,
            code_id,
            salt,
            _phantom: _,
        } = self;
        Deployment {
            env: env.clone(),
            code_id,
            salt,
            _phantom: PhantomData,
        }
    }

    pub fn pending_ctor<T: CallCodec>(self, args: T::Params) -> PendingCtor<A, T, E> {
        PendingCtor::new(self.env, self.code_id, self.salt, args)
    }
}

#[derive(Debug, Clone)]
pub struct Actor<A, E: GearEnv = GstdEnv> {
    env: E,
    id: ActorId,
    _phantom: PhantomData<A>,
}

impl<A, E: GearEnv> Actor<A, E> {
    pub fn new(env: E, id: ActorId) -> Self {
        Actor {
            env,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> ActorId {
        self.id
    }

    pub fn with_env<N: GearEnv>(self, env: &N) -> Actor<A, N> {
        let Self {
            env: _,
            id,
            _phantom: _,
        } = self;
        Actor {
            env: env.clone(),
            id,
            _phantom: PhantomData,
        }
    }

    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.id = actor_id;
        self
    }

    pub fn service<S>(&self, route: Route) -> Service<S, E> {
        Service::new(self.env.clone(), self.id, route)
    }
}

#[derive(Debug, Clone)]
pub struct Service<S, E: GearEnv = GstdEnv> {
    env: E,
    actor_id: ActorId,
    route: Route,
    _phantom: PhantomData<S>,
}

impl<S, E: GearEnv> Service<S, E> {
    pub fn new(env: E, actor_id: ActorId, route: Route) -> Self {
        Service {
            env,
            actor_id,
            route,
            _phantom: PhantomData,
        }
    }

    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    pub fn route(&self) -> Route {
        self.route
    }

    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = actor_id;
        self
    }

    pub fn pending_call<T: CallCodec>(&self, args: T::Params) -> PendingCall<T, E> {
        PendingCall::new(self.env.clone(), self.actor_id, self.route, args)
    }

    pub fn base_service<B>(&self) -> Service<B, E> {
        Service::new(self.env.clone(), self.actor_id, self.route)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn listener(&self) -> ServiceListener<S::Event, E>
    where
        S: ServiceWithEvents,
    {
        ServiceListener::new(self.env.clone(), self.actor_id, self.route)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait ServiceWithEvents {
    type Event: Event;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct ServiceListener<D: Event, E: GearEnv> {
    env: E,
    actor_id: ActorId,
    route: Route,
    _phantom: PhantomData<D>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: Event, E: GearEnv> ServiceListener<D, E> {
    pub fn new(env: E, actor_id: ActorId, route: Route) -> Self {
        ServiceListener {
            env,
            actor_id,
            route,
            _phantom: PhantomData,
        }
    }

    pub async fn listen(
        &self,
    ) -> Result<impl Stream<Item = (ActorId, D)> + Unpin, <E as GearEnv>::Error>
    where
        E: Listener<Error = <E as GearEnv>::Error>,
    {
        let self_id = self.actor_id;
        let prefix = self.route;
        self.env
            .listen(move |(actor_id, payload)| {
                if actor_id != self_id {
                    return None;
                }
                D::decode_event(prefix, payload).ok().map(|e| (actor_id, e))
            })
            .await
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCall<T: CallCodec, E: GearEnv> {
        env: E,
        destination: ActorId,
        route: Route,
        params: Option<E::Params>,
        args: Option<T::Params>,
        #[pin]
        state: Option<E::MessageState>
    }
}

impl<T: CallCodec, E: GearEnv> PendingCall<T, E> {
    pub fn new(env: E, destination: ActorId, route: Route, args: T::Params) -> Self {
        PendingCall {
            env,
            destination,
            route,
            params: None,
            args: Some(args),
            state: None,
        }
    }

    pub fn with_destination(mut self, actor_id: ActorId) -> Self {
        self.destination = actor_id;
        self
    }

    pub fn with_params(mut self, f: impl FnOnce(E::Params) -> E::Params) -> Self {
        self.params = Some(f(self.params.unwrap_or_default()));
        self
    }

    #[inline]
    fn take_encoded_args_and_params(&mut self) -> (Vec<u8>, E::Params) {
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        let payload = T::encode_params_with_prefix(self.route, &args);
        let params = self.params.take().unwrap_or_default();
        (payload, params)
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCtor<A, T: CallCodec, E: GearEnv> {
        env: E,
        code_id: CodeId,
        params: Option<E::Params>,
        salt: Option<Vec<u8>>,
        args: Option<T::Params>,
        _actor: PhantomData<A>,
        #[pin]
        state: Option<E::MessageState>,
        program_id: Option<ActorId>,
    }
}

impl<A, T: CallCodec, E: GearEnv> PendingCtor<A, T, E> {
    pub fn new(env: E, code_id: CodeId, salt: Vec<u8>, args: T::Params) -> Self {
        PendingCtor {
            env,
            code_id,
            params: None,
            salt: Some(salt),
            args: Some(args),
            _actor: PhantomData,
            state: None,
            program_id: None,
        }
    }

    pub fn with_params(mut self, f: impl FnOnce(E::Params) -> E::Params) -> Self {
        self.params = Some(f(self.params.unwrap_or_default()));
        self
    }
}

pub trait CallCodec {
    const ROUTE: Route;
    type Params: Encode;
    type Reply: Decode + 'static;

    fn encode_params(value: &Self::Params) -> Vec<u8> {
        let size = Encode::encoded_size(Self::ROUTE) + Encode::encoded_size(value);
        let mut result = Vec::with_capacity(size);
        Encode::encode_to(Self::ROUTE, &mut result);
        Encode::encode_to(value, &mut result);
        result
    }

    fn encode_params_with_prefix(prefix: Route, value: &Self::Params) -> Vec<u8> {
        let size = Encode::encoded_size(prefix)
            + Encode::encoded_size(Self::ROUTE)
            + Encode::encoded_size(value);
        let mut result = Vec::with_capacity(size);
        Encode::encode_to(prefix, &mut result);
        Encode::encode_to(Self::ROUTE, &mut result);
        Encode::encode_to(value, &mut result);
        result
    }

    fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply, parity_scale_codec::Error> {
        let mut value = payload.as_ref();
        if Self::is_empty_tuple::<Self::Reply>() {
            return Decode::decode(&mut value);
        }
        // Decode payload as `(String, Self::Reply)`
        let route = String::decode(&mut value)?;
        if route != Self::ROUTE {
            return Err("Invalid reply prefix".into());
        }
        Decode::decode(&mut value)
    }

    fn decode_reply_with_prefix(
        prefix: Route,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self::Reply, parity_scale_codec::Error> {
        let mut value = payload.as_ref();
        if Self::is_empty_tuple::<Self::Reply>() {
            return Decode::decode(&mut value);
        }
        // Decode payload as `(String, String, Self::Reply)`
        let route = String::decode(&mut value)?;
        if route != prefix {
            return Err("Invalid reply prefix".into());
        }
        let route = String::decode(&mut value)?;
        if route != Self::ROUTE {
            return Err("Invalid reply prefix".into());
        }
        Decode::decode(&mut value)
    }

    fn with_optimized_encode<R>(
        prefix: Route,
        value: &Self::Params,
        f: impl FnOnce(&[u8]) -> R,
    ) -> R {
        let size = Encode::encoded_size(prefix)
            + Encode::encoded_size(Self::ROUTE)
            + Encode::encoded_size(value);
        gcore::stack_buffer::with_byte_buffer(size, |buffer| {
            let mut buffer_writer = crate::utils::MaybeUninitBufferWriter::new(buffer);
            Encode::encode_to(prefix, &mut buffer_writer);
            Encode::encode_to(Self::ROUTE, &mut buffer_writer);
            Encode::encode_to(value, &mut buffer_writer);
            buffer_writer.with_buffer(f)
        })
    }

    fn is_empty_tuple<T: 'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<()>()
    }
}

#[macro_export]
macro_rules! params_struct_impl {
    (
        $env:ident,
        $name:ident { $( $(#[$attr:meta])* $vis:vis $field:ident: $ty:ty ),* $(,)?  }
    ) => {
        #[derive(Debug, Default)]
        pub struct $name {
            $(
                $(#[$attr])* $vis $field : Option< $ty >,
            )*
        }

        impl $name {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](mut self, $field: $ty) -> Self {
                        self.$field = Some($field);
                        self
                    }
                }
            )*
        }

        impl<A, T: CallCodec> PendingCtor<A, T, $env> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: CallCodec> PendingCall<T, $env> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! params_for_pending_impl {
    (
        $env:ident,
        $name:ident { $( $(#[$attr:meta])* $vis:vis $field:ident: $ty:ty ),* $(,)?  }
    ) => {
        impl $name {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](mut self, $field: $ty) -> Self {
                        self.$field = Some($field);
                        self
                    }
                }
            )*
        }

        impl<A, T: CallCodec> PendingCtor<A, T, $env> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: CallCodec> PendingCall<T, $env> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! io_struct_impl {
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty
    ) => {
        pub struct $name(());
        impl $name {
            pub fn encode_params($( $param: $ty, )* ) -> Vec<u8> {
                <$name as CallCodec>::encode_params(&( $( $param, )* ))
            }
            pub fn encode_params_with_prefix(prefix: Route, $( $param: $ty, )* ) -> Vec<u8> {
                <$name as CallCodec>::encode_params_with_prefix(prefix, &( $( $param, )* ))
            }
        }
        impl CallCodec for $name {
            const ROUTE: &'static str = stringify!($name);
            type Params = ( $( $ty, )* );
            type Reply = $reply;
        }
    };
}

#[allow(unused_macros)]
macro_rules! str_scale_encode {
    ($s:ident) => {{
        const S: &str = stringify!($s);
        assert!(S.len() <= 63, "Ident too long for encoding");
        const LEN: u8 = S.len() as u8;
        const BYTES: [u8; LEN as usize + 1] = {
            const fn to_array(s: &str) -> [u8; LEN as usize + 1] {
                let bytes = s.as_bytes();
                let mut out = [0u8; LEN as usize + 1];
                out[0] = LEN << 2;
                let mut i = 0;
                while i < LEN as usize {
                    out[i + 1] = bytes[i];
                    i += 1;
                }
                out
            }
            to_array(S)
        };
        BYTES.as_slice()
    }};
}

#[allow(async_fn_in_trait)]
pub trait Listener {
    type Error: Error;

    async fn listen<E, F: FnMut((ActorId, Vec<u8>)) -> Option<(ActorId, E)>>(
        &self,
        f: F,
    ) -> Result<impl Stream<Item = (ActorId, E)> + Unpin, Self::Error>;
}

#[cfg(not(target_arch = "wasm32"))]
pub trait Event: Decode {
    const EVENT_NAMES: &'static [Route];

    fn decode_event(
        prefix: Route,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self, parity_scale_codec::Error> {
        let mut payload = payload.as_ref();
        let route = String::decode(&mut payload)?;
        if route != prefix {
            return Err("Invalid event prefix".into());
        }
        let evt_name = String::decode(&mut payload)?;
        for (idx, &name) in Self::EVENT_NAMES.iter().enumerate() {
            if evt_name == name {
                let idx = idx as u8;
                let bytes = [&[idx], payload].concat();
                let mut event_bytes = &bytes[..];
                return Decode::decode(&mut event_bytes);
            }
        }
        Err("Invalid event name".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    io_struct_impl!(Add (value: u32) -> u32);
    io_struct_impl!(Value () -> u32);

    #[test]
    fn test_str_encode() {
        const ADD: &[u8] = str_scale_encode!(Add);
        assert_eq!(ADD, &[12, 65, 100, 100]);

        const VALUE: &[u8] = str_scale_encode!(Value);
        assert_eq!(VALUE, &[20, 86, 97, 108, 117, 101]);
    }

    #[test]
    fn test_io_struct_impl() {
        let add = Add::encode_params(42);
        assert_eq!(add, &[12, 65, 100, 100, 42, 0, 0, 0]);

        let value = Add::encode_params_with_prefix("Counter", 42);
        assert_eq!(
            value,
            &[
                28, 67, 111, 117, 110, 116, 101, 114, 12, 65, 100, 100, 42, 0, 0, 0
            ]
        );

        let value = Value::encode_params();
        assert_eq!(value, &[20, 86, 97, 108, 117, 101]);

        let value = Value::encode_params_with_prefix("Counter");
        assert_eq!(
            value,
            &[
                28, 67, 111, 117, 110, 116, 101, 114, 20, 86, 97, 108, 117, 101
            ]
        );
    }
}
