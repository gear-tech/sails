use crate::meta::SailsMessageHeader;
use crate::prelude::*;
use core::{
    any::TypeId,
    error::Error,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;
pub use sails_idl_meta::{Identifiable, InterfaceId, MethodMeta};

#[cfg(all(feature = "gtest", not(target_arch = "wasm32")))]
mod gtest_env;
#[cfg(all(feature = "gtest", not(target_arch = "wasm32")))]
pub use gtest_env::{BlockRunMode, GtestEnv, GtestError, GtestParams};

#[cfg(all(feature = "gclient", not(target_arch = "wasm32")))]
mod gclient_env;
#[cfg(all(feature = "gclient", not(target_arch = "wasm32")))]
pub use gclient_env::{GclientEnv, GclientError, GclientParams};

mod gstd_env;
pub use gstd_env::{GstdEnv, GstdParams};

pub(crate) const PENDING_CALL_INVALID_STATE: &str =
    "PendingCall polled after completion or invalid state";
pub(crate) const PENDING_CTOR_INVALID_STATE: &str =
    "PendingCtor polled after completion or invalid state";

// --- Route header typestate ---

/// Alias for v1 service route strings (OLD IDL protocol).
pub type Route = &'static str;

/// Typestate marker for route-header encoding.
pub trait RouteHeader: Clone + core::fmt::Debug {}

/// v2 (NEW IDL) binary-header route: numeric route index (default).
#[derive(Debug, Clone, Copy)]
pub struct RouteIdx(pub u8);
impl RouteHeader for RouteIdx {}

/// v1 (OLD IDL) SCALE-string route: carries the service route string at runtime.
/// Ctor services use `RouteName("")` (no service prefix).
#[derive(Debug, Clone, Copy)]
pub struct RouteName(pub Route);
impl RouteHeader for RouteName {}

// --- GearEnv and related ---

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

    /// v2 (NEW IDL, default) ctor.
    pub fn pending_ctor<T: ServiceCall>(self, args: T::Params) -> PendingCtor<A, T, E> {
        PendingCtor::new(self.env, self.code_id, self.salt, RouteIdx(0), args)
    }

    /// v1 (OLD IDL) ctor. Ctors have no service prefix — uses `RouteName("")`.
    pub fn pending_ctor_v1<T: ServiceCall<RouteName>>(
        self,
        args: T::Params,
    ) -> PendingCtor<A, T, E, RouteName> {
        PendingCtor::new(self.env, self.code_id, self.salt, RouteName(""), args)
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

    /// v2 (NEW IDL, default): service identified by numeric route index.
    pub fn service<S>(&self, route_idx: u8) -> Service<S, E> {
        Service::new(self.env.clone(), self.id, RouteIdx(route_idx))
    }

    /// v1 (OLD IDL): service identified by its IDL name (empty string for anonymous).
    pub fn service_v1<S>(&self, name: Route) -> Service<S, E, RouteName> {
        Service::new(self.env.clone(), self.id, RouteName(name))
    }
}

#[derive(Debug, Clone)]
pub struct Service<S, E: GearEnv = GstdEnv, R: RouteHeader = RouteIdx> {
    env: E,
    actor_id: ActorId,
    route: R,
    _phantom: PhantomData<S>,
}

impl<S, E: GearEnv, R: RouteHeader> Service<S, E, R> {
    pub fn new(env: E, actor_id: ActorId, route: R) -> Self {
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

    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = actor_id;
        self
    }

    pub fn pending_call<T: ServiceCall<R>>(&self, args: T::Params) -> PendingCall<T, E, R> {
        PendingCall::new(self.env.clone(), self.actor_id, self.route.clone(), args)
    }

    pub fn base_service<B>(&self) -> Service<B, E, R> {
        Service::new(self.env.clone(), self.actor_id, self.route.clone())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn decode_event<Ev: Event<R>>(
        &self,
        payload: impl AsRef<[u8]>,
    ) -> Result<Ev, parity_scale_codec::Error> {
        Ev::decode_event(&self.route, payload)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn listener(&self) -> ServiceListener<S::Event, E, R>
    where
        S: ServiceWithEvents<R>,
    {
        ServiceListener::new(self.env.clone(), self.actor_id, self.route.clone())
    }
}

// v2-specific id accessors only on RouteIdx
impl<S, E: GearEnv> Service<S, E, RouteIdx> {
    pub fn interface_id(&self) -> InterfaceId
    where
        S: Identifiable,
    {
        S::INTERFACE_ID
    }

    pub fn route_idx(&self) -> u8 {
        self.route.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait ServiceWithEvents<R: RouteHeader = RouteIdx> {
    type Event: Event<R>;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct ServiceListener<D, E: GearEnv, R: RouteHeader = RouteIdx>
where
    D: Event<R>,
{
    env: E,
    actor_id: ActorId,
    route: R,
    _phantom: PhantomData<D>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<D, E: GearEnv, R: RouteHeader> ServiceListener<D, E, R>
where
    D: Event<R>,
{
    pub fn new(env: E, actor_id: ActorId, route: R) -> Self {
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
        let route = self.route.clone();
        self.env
            .listen(move |(actor_id, payload)| {
                if actor_id != self_id {
                    return None;
                }
                D::decode_event(&route, payload).ok().map(|e| (actor_id, e))
            })
            .await
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCall<T: ServiceCall<R>, E: GearEnv, R: RouteHeader = RouteIdx> {
        env: E,
        destination: ActorId,
        route: R,
        params: Option<E::Params>,
        args: Option<T::Params>,
        #[pin]
        state: Option<E::MessageState>
    }
}

impl<T: ServiceCall<R>, E: GearEnv, R: RouteHeader> PendingCall<T, E, R> {
    pub fn new(env: E, destination: ActorId, route: R, args: T::Params) -> Self {
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

    pub fn encode_call(mut self) -> Vec<u8> {
        let (payload, _) = self.take_encoded_args_and_params();
        payload
    }

    #[inline]
    pub(crate) fn take_encoded_args_and_params(&mut self) -> (Vec<u8>, E::Params) {
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        let payload = T::encode_call(&self.route, &args);
        let params = self.params.take().unwrap_or_default();
        (payload, params)
    }
}

pub trait PendingCtorOutput<A, E: GearEnv> {
    type Output;

    fn map_result(self, env: E, id: ActorId) -> Self::Output;

    fn actor(output: Self::Output) -> Actor<A, E>;
}

impl<A, E: GearEnv> PendingCtorOutput<A, E> for () {
    type Output = Actor<A, E>;

    fn map_result(self, env: E, id: ActorId) -> Self::Output {
        Actor::new(env, id)
    }

    fn actor(output: Self::Output) -> Actor<A, E> {
        output
    }
}

impl<A, E: GearEnv, Err: core::fmt::Debug> PendingCtorOutput<A, E> for Result<(), Err> {
    type Output = Result<Actor<A, E>, Err>;

    fn map_result(self, env: E, id: ActorId) -> Self::Output {
        self.map(|_| Actor::new(env, id))
    }

    fn actor(output: Self::Output) -> Actor<A, E> {
        output.expect("PendingCtor output is not Ok")
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCtor<A, T: ServiceCall<R>, E: GearEnv, R: RouteHeader = RouteIdx> {
        env: E,
        code_id: CodeId,
        route: R,
        params: Option<E::Params>,
        salt: Option<Vec<u8>>,
        args: Option<T::Params>,
        _actor: PhantomData<A>,
        #[pin]
        state: Option<E::MessageState>,
        program_id: Option<ActorId>,
    }
}

impl<A, T: ServiceCall<R>, E: GearEnv, R: RouteHeader> PendingCtor<A, T, E, R> {
    pub fn new(env: E, code_id: CodeId, salt: Vec<u8>, route: R, args: T::Params) -> Self {
        PendingCtor {
            env,
            code_id,
            route,
            params: None,
            salt: Some(salt),
            args: Some(args),
            state: None,
            program_id: None,
            _actor: PhantomData,
        }
    }

    pub fn with_params(mut self, f: impl FnOnce(E::Params) -> E::Params) -> Self {
        self.params = Some(f(self.params.unwrap_or_default()));
        self
    }
}

/// Unified service-call codec parameterized by route header.
///
/// `R = RouteIdx` (default) → v2 binary SailsHeader encoding.
/// `R = RouteName` → v1 SCALE-string encoding; the route string is read from
/// the `RouteName(route)` instance passed at call time.
pub trait ServiceCall<R: RouteHeader = RouteIdx> {
    type Params;
    type Reply;
    /// Application-level error type (IDL `throws`). Always `()` for v1.
    type Throws;
    type Output;

    fn encode_call(route: &R, value: &Self::Params) -> Vec<u8>;

    fn decode_reply(
        route: &R,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self::Output, parity_scale_codec::Error>;

    fn decode_error(
        route: &R,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self::Output, parity_scale_codec::Error>;
}

// --- Standalone helpers (previously on the ServiceCall trait) ---

/// v2-specific: decode a reply payload that has a SailsMessageHeader prefix.
/// Used internally by `io_struct_impl!`.
pub fn decode_with_header<T: Decode + 'static, M: MethodMeta + Identifiable>(
    route_idx: u8,
    payload: impl AsRef<[u8]>,
) -> Result<T, parity_scale_codec::Error> {
    let mut value = payload.as_ref();
    if is_empty_tuple::<T>() {
        return Decode::decode(&mut value);
    }
    let header = SailsMessageHeader::decode(&mut value)?;
    if header.interface_id() != M::INTERFACE_ID {
        return Err("Invalid reply interface_id".into());
    }
    if header.route_id() != route_idx {
        return Err("Invalid reply route_idx".into());
    }
    if header.entry_id() != M::ENTRY_ID {
        return Err("Invalid reply entry_id".into());
    }
    Decode::decode(&mut value)
}

/// Returns true if `T` is the unit type `()`.
pub fn is_empty_tuple<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<()>()
}

/// v2-specific: encode a call using a stack buffer (zero-copy, requires `gcore`).
/// Used in wasm32 on-chain dispatch code via the `io_struct_impl!` inherent methods.
pub fn encode_call_optimized<T, R>(
    route_idx: u8,
    value: &T::Params,
    f: impl FnOnce(&[u8]) -> R,
) -> R
where
    T: ServiceCall<RouteIdx> + MethodMeta + Identifiable,
    T::Params: Encode,
{
    encode_call_optimized_with_id::<T, R>(T::INTERFACE_ID, T::ENTRY_ID, route_idx, value, f)
}

pub fn encode_call_optimized_with_id<T, R>(
    interface_id: InterfaceId,
    entry_id: u16,
    route_idx: u8,
    value: &T::Params,
    f: impl FnOnce(&[u8]) -> R,
) -> R
where
    T: ServiceCall<RouteIdx>,
    T::Params: Encode,
{
    let header = SailsMessageHeader::new(
        crate::meta::Version::v1(),
        crate::meta::HeaderLength::new(crate::meta::MINIMAL_HLEN).unwrap(),
        interface_id,
        route_idx,
        entry_id,
    );
    let size = (crate::meta::MINIMAL_HLEN as usize) + Encode::encoded_size(value);
    gcore::stack_buffer::with_byte_buffer(size, |buffer| {
        let mut buffer_writer = crate::utils::MaybeUninitBufferWriter::new(buffer);
        header.encode_to(&mut buffer_writer);
        Encode::encode_to(value, &mut buffer_writer);
        buffer_writer.with_buffer(f)
    })
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

        impl<A, T: $crate::client::ServiceCall<R>, R: $crate::client::RouteHeader> $crate::client::PendingCtor<A, T, $env, R> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: $crate::client::ServiceCall<R>, R: $crate::client::RouteHeader> $crate::client::PendingCall<T, $env, R> {
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

        impl<A, T: $crate::client::ServiceCall<R>, R: $crate::client::RouteHeader> $crate::client::PendingCtor<A, T, $env, R> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: $crate::client::ServiceCall<R>, R: $crate::client::RouteHeader> $crate::client::PendingCall<T, $env, R> {
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
    // 3-arg form: service method with entry_id + interface_id (no throws)
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty, $entry_id:expr, $interface_id:expr
    ) => {
        $crate::io_struct_impl!(@impl_base $name ( $( $param : $ty ),* ) -> $reply, $entry_id, $interface_id);

        impl $crate::client::ServiceCall<$crate::client::RouteIdx> for $name {
            type Params = ( $( $ty, )* );
            type Reply = $reply;
            type Throws = ();
            type Output = $reply;

            fn encode_call(route: &$crate::client::RouteIdx, value: &Self::Params) -> Vec<u8> {
                let header = $crate::meta::SailsMessageHeader::new(
                    $crate::meta::Version::v1(),
                    $crate::meta::HeaderLength::new($crate::meta::MINIMAL_HLEN).unwrap(),
                    <$name as $crate::client::Identifiable>::INTERFACE_ID,
                    route.0,
                    <$name as $crate::client::MethodMeta>::ENTRY_ID,
                );
                let mut result = header.to_bytes();
                $crate::prelude::Encode::encode_to(value, &mut result);
                result
            }

            fn decode_reply(
                route: &$crate::client::RouteIdx,
                payload: impl AsRef<[u8]>,
            ) -> Result<Self::Output, $crate::scale_codec::Error> {
                $crate::client::decode_with_header::<Self::Reply, $name>(route.0, payload)
            }

            fn decode_error(
                _route: &$crate::client::RouteIdx,
                _payload: impl AsRef<[u8]>,
            ) -> Result<Self::Output, $crate::scale_codec::Error> {
                Err("Throws type is `()`".into())
            }
        }
    };
    // 3-arg form with throws
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty | $throws:ty, $entry_id:expr, $interface_id:expr
    ) => {
        $crate::io_struct_impl!(@impl_base $name ( $( $param : $ty ),* ) -> $reply, $entry_id, $interface_id);

        impl $crate::client::ServiceCall<$crate::client::RouteIdx> for $name {
            type Params = ( $( $ty, )* );
            type Reply = $reply;
            type Throws = $throws;
            type Output = Result<$reply, $throws>;

            fn encode_call(route: &$crate::client::RouteIdx, value: &Self::Params) -> Vec<u8> {
                let header = $crate::meta::SailsMessageHeader::new(
                    $crate::meta::Version::v1(),
                    $crate::meta::HeaderLength::new($crate::meta::MINIMAL_HLEN).unwrap(),
                    <$name as $crate::client::Identifiable>::INTERFACE_ID,
                    route.0,
                    <$name as $crate::client::MethodMeta>::ENTRY_ID,
                );
                let mut result = header.to_bytes();
                $crate::prelude::Encode::encode_to(value, &mut result);
                result
            }

            fn decode_reply(
                route: &$crate::client::RouteIdx,
                payload: impl AsRef<[u8]>,
            ) -> Result<Self::Output, $crate::scale_codec::Error> {
                Ok(Ok($crate::client::decode_with_header::<Self::Reply, $name>(route.0, payload)?))
            }

            fn decode_error(
                route: &$crate::client::RouteIdx,
                payload: impl AsRef<[u8]>,
            ) -> Result<Self::Output, $crate::scale_codec::Error> {
                Ok(Err($crate::client::decode_with_header::<Self::Throws, $name>(route.0, payload)?))
            }
        }
    };
    // 2-arg form: ctor method (InterfaceId::zero)
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty $( | $throws:ty )?, $entry_id:expr
    ) => {
        $crate::io_struct_impl!($name ( $( $param : $ty ),* ) -> $reply $( | $throws )?, $entry_id, $crate::meta::InterfaceId::zero());
    };
    // @impl_base: struct + Identifiable + MethodMeta + convenience inherent methods
    (
        @impl_base $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty, $entry_id:expr, $interface_id:expr
    ) => {
        pub struct $name(());
        impl $name {
            /// Encodes the full call with the correct Sails header.
            pub fn encode_call(route_idx: u8, $( $param: $ty, )* ) -> Vec<u8> {
                <$name as $crate::client::ServiceCall<$crate::client::RouteIdx>>::encode_call(
                    &$crate::client::RouteIdx(route_idx), &( $( $param, )* )
                )
            }
            /// Decodes the reply checking against the correct Sails header.
            pub fn decode_reply(route_idx: u8, payload: impl AsRef<[u8]>) -> Result<$reply, $crate::scale_codec::Error> {
                $crate::client::decode_with_header::<$reply, $name>(route_idx, payload)
            }
        }
        impl $crate::client::Identifiable for $name {
            const INTERFACE_ID: $crate::meta::InterfaceId = $interface_id;
        }
        impl $crate::client::MethodMeta for $name {
            const ENTRY_ID: u16 = $entry_id;
        }
    };
}

/// v1 (OLD IDL, SCALE-string encoded) service/ctor IO struct.
///
/// `ServiceCall<RouteName>` uses the route string from the `RouteName` instance
/// passed at call time (set by `Actor::service_v1(name)` / `pending_ctor_v1`).
///
/// v1 has NO `throws` concept — `Throws` is always `()` and `Output = Reply`.
///
/// Encoding: `SCALE(route.0) + SCALE(stringify!($name)) + SCALE(params)`
/// If `route.0` is empty (ctor / anonymous service), the service prefix is omitted.
#[macro_export]
macro_rules! io_struct_impl_v1 {
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty
    ) => {
        pub struct $name(());

        impl $name {
            pub fn encode_call(route: $crate::client::Route, $( $param: $ty, )* ) -> Vec<u8> {
                <$name as $crate::client::ServiceCall<$crate::client::RouteName>>::encode_call(
                    &$crate::client::RouteName(route), &( $( $param, )* )
                )
            }
        }

        impl $crate::client::ServiceCall<$crate::client::RouteName> for $name {
            type Params = ( $( $ty, )* );
            type Reply = $reply;
            // v1 has no `throws` concept — always unit, Output == Reply.
            type Throws = ();
            type Output = $reply;

            fn encode_call(
                route: &$crate::client::RouteName,
                value: &Self::Params,
            ) -> Vec<u8> {
                use $crate::prelude::Encode as _;
                let mut result = Vec::new();
                if !route.0.is_empty() {
                    route.0.encode_to(&mut result);
                }
                stringify!($name).encode_to(&mut result);
                value.encode_to(&mut result);
                result
            }

            fn decode_reply(
                route: &$crate::client::RouteName,
                payload: impl AsRef<[u8]>,
            ) -> Result<Self::Output, $crate::scale_codec::Error> {
                use $crate::prelude::Decode as _;
                let mut value = payload.as_ref();
                if $crate::client::is_empty_tuple::<$reply>() {
                    return $crate::prelude::Decode::decode(&mut value);
                }
                if !route.0.is_empty() {
                    let svc = String::decode(&mut value)?;
                    if svc != route.0 {
                        return Err("Invalid reply service route".into());
                    }
                }
                let method = String::decode(&mut value)?;
                if method != stringify!($name) {
                    return Err("Invalid reply method route".into());
                }
                $crate::prelude::Decode::decode(&mut value)
            }

            fn decode_error(
                _route: &$crate::client::RouteName,
                _payload: impl AsRef<[u8]>,
            ) -> Result<Self::Output, $crate::scale_codec::Error> {
                Err("Throws type is `()` for v1".into())
            }
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

struct EventInput<'a> {
    idx: u8,
    payload: &'a [u8],
    first: bool,
}

impl<'a> parity_scale_codec::Input for EventInput<'a> {
    fn remaining_len(&mut self) -> Result<Option<usize>, parity_scale_codec::Error> {
        Ok(Some(1 + self.payload.len()))
    }

    fn read(&mut self, into: &mut [u8]) -> Result<(), parity_scale_codec::Error> {
        if into.is_empty() {
            return Ok(());
        }
        let (head, tail) = into.split_at_mut(if self.first { 1 } else { 0 });
        if self.first {
            head[0] = self.idx;
            self.first = false;
        }
        if tail.is_empty() {
            return Ok(());
        }
        if tail.len() > self.payload.len() {
            return Err("Not enough data to fill buffer".into());
        }
        tail.copy_from_slice(&self.payload[..tail.len()]);
        self.payload = &self.payload[tail.len()..];
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, parity_scale_codec::Error> {
        if self.first {
            self.first = false;
            Ok(self.idx)
        } else {
            let b = *self.payload.first().ok_or("Not enough data to read byte")?;
            self.payload = &self.payload[1..];
            Ok(b)
        }
    }
}

// --- Event traits ---

/// v1 (OLD IDL): compile-time list of event variant names.
/// Implemented by `client-gen-v1`-generated event enums.
#[cfg(not(target_arch = "wasm32"))]
pub trait EventNames {
    const EVENT_NAMES: &'static [Route];
}

/// v2-specific: decode an event payload using SailsMessageHeader.
/// Used by v2-generated `impl Event<RouteIdx>` blocks.
#[cfg(not(target_arch = "wasm32"))]
pub fn decode_event_v2<Ev: Decode + Identifiable>(
    route_idx: u8,
    payload: impl AsRef<[u8]>,
) -> Result<Ev, parity_scale_codec::Error> {
    let mut payload = payload.as_ref();

    let header = SailsMessageHeader::decode(&mut payload)?;
    if header.interface_id() != Ev::INTERFACE_ID {
        return Err("Invalid event interface_id".into());
    }
    if header.route_id() != route_idx {
        return Err("Invalid event route_idx".into());
    }

    let entry_id = header.entry_id();
    if entry_id > 255 {
        return Err("Entry ID exceeds u8 limit for SCALE enum".into());
    }

    let variant_index = entry_id as u8;
    let mut input = EventInput {
        idx: variant_index,
        payload,
        first: true,
    };
    Decode::decode(&mut input)
}

/// v1-specific: decode an event payload encoded as `SCALE(variant_name) + SCALE(data)`.
#[cfg(not(target_arch = "wasm32"))]
pub fn decode_event_v1<Ev: Decode + EventNames>(
    payload: impl AsRef<[u8]>,
) -> Result<Ev, parity_scale_codec::Error> {
    let mut payload = payload.as_ref();
    let name = String::decode(&mut payload)?;
    let idx = <Ev as EventNames>::EVENT_NAMES
        .iter()
        .position(|n| *n == name)
        .ok_or("Unknown v1 event name")? as u8;
    let mut input = EventInput {
        idx,
        payload,
        first: true,
    };
    Decode::decode(&mut input)
}

/// Event codec parameterized by route header.
/// `R = RouteIdx` → v2 SailsHeader-based decoding.
/// `R = RouteName` → v1 SCALE-string variant-name decoding.
#[cfg(not(target_arch = "wasm32"))]
pub trait Event<R: RouteHeader = RouteIdx>: Decode + Sized {
    fn decode_event(
        route: &R,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self, parity_scale_codec::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    // Define Add with InterfaceId to test 3-arg macro (Service mode)
    io_struct_impl!(Add (value: u32) -> u32, 0, InterfaceId::from_bytes_8([1, 2, 3, 4, 5, 6, 7, 8]));
    // Define Value with 2-arg macro (Ctor/Legacy mode)
    io_struct_impl!(Value () -> u32, 1);
    // Define Sub with `throws` type
    io_struct_impl!(Sub (value: u32) -> u32 | String, 0, InterfaceId::from_bytes_8([1, 2, 3, 4, 5, 6, 7, 8]));

    #[test]
    fn test_io_struct_impl() {
        // Add is a "Service" method, encode takes RouteIdx
        let route_idx = 5u8;
        let add_specific = Add::encode_call(route_idx, 42);

        let expected_header_add = [
            0x47, 0x4D, // magic ("GM")
            1,    // version
            16,   // hlen
            1, 2, 3, 4, 5, 6, 7, 8, // interface_id
            0, 0, // entry_id (0 for Add)
            5, // route_id
            0, // reserved
        ];
        let expected_add_payload = [42, 0, 0, 0]; // 42u32 LE

        let mut expected_add_specific = Vec::new();
        expected_add_specific.extend_from_slice(&expected_header_add);
        expected_add_specific.extend_from_slice(&expected_add_payload);

        assert_eq!(add_specific, expected_add_specific);

        let reply_payload = [42, 0, 0, 0];
        let mut reply_with_header = expected_add_specific.clone();
        reply_with_header.truncate(16);
        reply_with_header.extend_from_slice(&reply_payload);

        let decoded = Add::decode_reply(route_idx, &reply_with_header).unwrap();
        assert_eq!(decoded, 42);

        // Value is "Ctor" mode, it uses InterfaceId::zero()
        let value_encoded = Value::encode_call(0);
        let expected_header_value = [
            0x47, 0x4D, 1, 16, // magic, version, hlen
            0, 0, 0, 0, 0, 0, 0, 0, // interface_id (zero)
            1, 0, // entry_id (1 for Value)
            0, 0, // route_id 0 and reserved 0
        ];
        assert_eq!(value_encoded, expected_header_value);

        // Decode reply for Value
        let mut value_reply = expected_header_value.to_vec();
        value_reply.extend_from_slice(&[123, 0, 0, 0]); // payload 123u32
        let decoded_value = Value::decode_reply(0, &value_reply).unwrap();
        assert_eq!(decoded_value, 123);
    }

    #[test]
    fn test_io_struct_impl_throws() {
        let route_idx = 5u8;
        let sub_specific = Sub::encode_call(route_idx, 42);

        let expected_header_sub = [
            0x47, 0x4D, // magic ("GM")
            1,    // version
            16,   // hlen
            1, 2, 3, 4, 5, 6, 7, 8, // interface_id
            0, 0, // entry_id (0 for Sub)
            5, // route_id
            0, // reserved
        ];
        let expected_sub_payload = [42, 0, 0, 0]; // 42u32 LE

        let mut expected_sub_specific = Vec::new();
        expected_sub_specific.extend_from_slice(&expected_header_sub);
        expected_sub_specific.extend_from_slice(&expected_sub_payload);

        assert_eq!(sub_specific, expected_sub_specific);

        let mut success_reply = expected_header_sub.to_vec();
        success_reply.extend_from_slice(&expected_sub_payload);

        let decoded_success =
            <Sub as ServiceCall<RouteIdx>>::decode_reply(&RouteIdx(route_idx), &success_reply)
                .unwrap();
        assert_eq!(decoded_success, Ok(42));
        assert_eq!(Sub::decode_reply(route_idx, &success_reply).unwrap(), 42);

        let error_message = String::from("boom");
        let mut error_reply = expected_header_sub.to_vec();
        error_reply.extend_from_slice(&error_message.encode());

        let decoded_error =
            <Sub as ServiceCall<RouteIdx>>::decode_error(&RouteIdx(route_idx), &error_reply)
                .unwrap();
        assert_eq!(decoded_error, Err(error_message));
    }

    #[test]
    fn test_io_struct_impl_v1() {
        io_struct_impl_v1!(DoThis (value: u32) -> u32);

        // Encoding: SCALE("MyService") + SCALE("DoThis") + SCALE(42u32)
        let encoded = DoThis::encode_call("MyService", 42u32);
        let mut expected = Vec::new();
        "MyService".encode_to(&mut expected);
        "DoThis".encode_to(&mut expected);
        42u32.encode_to(&mut expected);
        assert_eq!(encoded, expected);

        // Decoding
        let decoded =
            <DoThis as ServiceCall<RouteName>>::decode_reply(&RouteName("MyService"), &encoded)
                .unwrap();
        assert_eq!(decoded, 42u32);
    }
}
