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
pub use sails_idl_meta::InterfaceId;

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

    pub fn pending_ctor<T: ServiceCall>(self, args: T::Params) -> PendingCtor<A, T, E> {
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

    pub fn service<S>(&self, route_idx: u8) -> Service<S, E> {
        Service::new(self.env.clone(), self.id, route_idx)
    }
}

pub trait Identifiable {
    const INTERFACE_ID: InterfaceId;
}

#[derive(Debug, Clone)]
pub struct Service<S, E: GearEnv = GstdEnv> {
    env: E,
    actor_id: ActorId,
    route_idx: u8,
    _phantom: PhantomData<S>,
}

impl<S, E: GearEnv> Service<S, E> {
    pub fn new(env: E, actor_id: ActorId, route_idx: u8) -> Self {
        Service {
            env,
            actor_id,
            route_idx,
            _phantom: PhantomData,
        }
    }

    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    pub fn interface_id(&self) -> InterfaceId
    where
        S: Identifiable,
    {
        S::INTERFACE_ID
    }

    pub fn route_idx(&self) -> u8 {
        self.route_idx
    }

    pub fn with_actor_id(mut self, actor_id: ActorId) -> Self {
        self.actor_id = actor_id;
        self
    }

    pub fn pending_call<T: ServiceCall>(&self, args: T::Params) -> PendingCall<T, E> {
        PendingCall::new(self.env.clone(), self.actor_id, self.route_idx, args)
    }

    pub fn base_service<B>(&self) -> Service<B, E> {
        Service::new(self.env.clone(), self.actor_id, self.route_idx)
    }

    pub fn decode_reply<T: ServiceCall>(
        &self,
        payload: impl AsRef<[u8]>,
    ) -> Result<T::Reply, parity_scale_codec::Error> {
        T::decode_reply_with_header(self.route_idx, payload)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn decode_event<Ev: Event>(
        &self,
        payload: impl AsRef<[u8]>,
    ) -> Result<Ev, parity_scale_codec::Error> {
        Ev::decode_event(self.route_idx, payload)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn listener(&self) -> ServiceListener<S::Event, E>
    where
        S: ServiceWithEvents,
    {
        ServiceListener::new(self.env.clone(), self.actor_id, self.route_idx)
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
    route_idx: u8,
    _phantom: PhantomData<D>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<D: Event, E: GearEnv> ServiceListener<D, E> {
    pub fn new(env: E, actor_id: ActorId, route_idx: u8) -> Self {
        ServiceListener {
            env,
            actor_id,
            route_idx,
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
        let route_idx = self.route_idx;
        self.env
            .listen(move |(actor_id, payload)| {
                if actor_id != self_id {
                    return None;
                }
                D::decode_event(route_idx, payload)
                    .ok()
                    .map(|e| (actor_id, e))
            })
            .await
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCall<T: ServiceCall, E: GearEnv> {
        env: E,
        destination: ActorId,
        route_idx: u8,
        params: Option<E::Params>,
        args: Option<T::Params>,
        #[pin]
        state: Option<E::MessageState>
    }
}

impl<T: ServiceCall, E: GearEnv> PendingCall<T, E> {
    pub fn new(env: E, destination: ActorId, route_idx: u8, args: T::Params) -> Self {
        PendingCall {
            env,
            destination,
            route_idx,
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
    fn take_encoded_args_and_params(&mut self) -> (Vec<u8>, E::Params) {
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CALL_INVALID_STATE}"));
        let payload = T::encode_params_with_header(self.route_idx, &args);
        let params = self.params.take().unwrap_or_default();
        (payload, params)
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCtor<A, T: ServiceCall, E: GearEnv> {
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

impl<A, T: ServiceCall, E: GearEnv> PendingCtor<A, T, E> {
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

    pub fn encode_params(mut self) -> Vec<u8> {
        let args = self
            .args
            .take()
            .unwrap_or_else(|| panic!("{PENDING_CTOR_INVALID_STATE}"));
        T::encode_params(&args)
    }
}

pub trait CallCodec {
    const ENTRY_ID: u16;
    type Params: Encode;
    type Reply: Decode + 'static;

    fn encode_params(value: &Self::Params) -> Vec<u8> {
        value.encode()
    }

    fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply, parity_scale_codec::Error> {
        let mut value = payload.as_ref();
        if Self::is_empty_tuple::<Self::Reply>() {
            return Decode::decode(&mut value);
        }
        let header = SailsMessageHeader::decode(&mut value)?;
        if header.entry_id() != Self::ENTRY_ID {
            return Err("Invalid reply entry_id".into());
        }
        Decode::decode(&mut value)
    }

    fn is_empty_tuple<T: 'static>() -> bool {
        TypeId::of::<T>() == TypeId::of::<()>()
    }
}

pub trait ServiceCall: CallCodec + Identifiable {
    fn encode_params_with_header(route_idx: u8, value: &Self::Params) -> Vec<u8> {
        let header = SailsMessageHeader::new(
            crate::meta::Version::v1(),
            crate::meta::HeaderLength::new(crate::meta::MINIMAL_HLEN).unwrap(),
            Self::INTERFACE_ID,
            route_idx,
            Self::ENTRY_ID,
        );
        let mut result = header.to_bytes();
        value.encode_to(&mut result);
        result
    }

    fn decode_reply_with_header(
        route_idx: u8,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self::Reply, parity_scale_codec::Error> {
        let mut value = payload.as_ref();
        if Self::is_empty_tuple::<Self::Reply>() {
            return Decode::decode(&mut value);
        }
        let header = SailsMessageHeader::decode(&mut value)?;
        if header.interface_id() != Self::INTERFACE_ID {
            return Err("Invalid reply interface_id".into());
        }
        if header.route_id() != route_idx {
            return Err("Invalid reply route_idx".into());
        }
        if header.entry_id() != Self::ENTRY_ID {
            return Err("Invalid reply entry_id".into());
        }
        Decode::decode(&mut value)
    }

    fn with_optimized_encode<R>(
        route_idx: u8,
        value: &Self::Params,
        f: impl FnOnce(&[u8]) -> R,
    ) -> R {
        let header = SailsMessageHeader::new(
            crate::meta::Version::v1(),
            crate::meta::HeaderLength::new(crate::meta::MINIMAL_HLEN).unwrap(),
            Self::INTERFACE_ID,
            route_idx,
            Self::ENTRY_ID,
        );
        let size = (crate::meta::MINIMAL_HLEN as usize) + Encode::encoded_size(value);
        gcore::stack_buffer::with_byte_buffer(size, |buffer| {
            let mut buffer_writer = crate::utils::MaybeUninitBufferWriter::new(buffer);
            header.encode_to(&mut buffer_writer);
            Encode::encode_to(value, &mut buffer_writer);
            buffer_writer.with_buffer(f)
        })
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

        impl<A, T: ServiceCall> PendingCtor<A, T, $env> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: ServiceCall> PendingCall<T, $env> {
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

        impl<A, T: ServiceCall> PendingCtor<A, T, $env> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: ServiceCall> PendingCall<T, $env> {
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
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty, $entry_id:expr, $interface_id:expr
    ) => {
        pub struct $name(());
        impl $name {
            /// Encodes the parameters only (without Sails header).
            pub fn encode_params($( $param: $ty, )* ) -> Vec<u8> {
                <$name as CallCodec>::encode_params(&( $( $param, )* ))
            }
            /// Encodes the full call with the correct Sails header (Interface ID + Route Index + Entry ID).
            pub fn encode_call(route_idx: u8, $( $param: $ty, )* ) -> Vec<u8> {
                <$name as ServiceCall>::encode_params_with_header(route_idx, &( $( $param, )* ))
            }
            /// Decodes the reply checking against the correct Sails header (Interface ID + Route Index + Entry ID).
            pub fn decode_reply(route_idx: u8, payload: impl AsRef<[u8]>) -> Result<$reply, $crate::scale_codec::Error> {
                <$name as ServiceCall>::decode_reply_with_header(route_idx, payload)
            }
        }
        impl Identifiable for $name {
            const INTERFACE_ID: InterfaceId = $interface_id;
        }
        impl CallCodec for $name {
            const ENTRY_ID: u16 = $entry_id;
            type Params = ( $( $ty, )* );
            type Reply = $reply;
        }
        impl ServiceCall for $name {}
    };
    (
        $name:ident ( $( $param:ident : $ty:ty ),* ) -> $reply:ty, $entry_id:expr
    ) => {
        pub struct $name(());
        impl $name {
            /// Encodes the parameters only (without Sails header).
            pub fn encode_params($( $param: $ty, )* ) -> Vec<u8> {
                <$name as CallCodec>::encode_params(&( $( $param, )* ))
            }
        }
        impl CallCodec for $name {
            const ENTRY_ID: u16 = $entry_id;
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

#[cfg(not(target_arch = "wasm32"))]
pub trait Event: Decode + Identifiable {
    fn decode_event(
        route_idx: u8,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self, parity_scale_codec::Error> {
        let mut payload = payload.as_ref();

        let header = SailsMessageHeader::decode(&mut payload)?;
        if header.interface_id() != Self::INTERFACE_ID {
            return Err("Invalid event interface_id".into());
        }
        if header.route_id() != route_idx {
            return Err("Invalid event route_idx".into());
        }

        let entry_id = header.entry_id();
        if entry_id > 255 {
            return Err("Entry ID exceeds u8 limit for SCALE enum".into());
        }

        // Reconstruct the standard SCALE enum encoding.
        // The network payload only contains the event data (arguments), omitting the variant index
        // because the `entry_id` in the header already serves as the identifier.
        // However, the standard Rust `Decode` implementation for enums expects a leading index byte.
        // Therefore, we use a custom Input to prepend the `entry_id` (as u8) to the payload without allocation.
        let variant_index = entry_id as u8;
        let mut input = EventInput {
            idx: variant_index,
            payload,
            first: true,
        };
        Decode::decode(&mut input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Define Add with InterfaceId to test 3-arg macro (Service mode)
    io_struct_impl!(Add (value: u32) -> u32, 0, InterfaceId::from_bytes_8([1, 2, 3, 4, 5, 6, 7, 8]));
    // Define Value with 2-arg macro (Ctor/Legacy mode)
    io_struct_impl!(Value () -> u32, 1);

    #[test]
    fn test_io_struct_impl() {
        // Add is now a "Service" method, so encode takes route_idx
        let route_idx = 5;
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

        // Decode uses the new helper method
        let decoded = Add::decode_reply(route_idx, &reply_with_header).unwrap();
        assert_eq!(decoded, 42);

        // Add::encode_params should return raw bytes without header even for service methods
        let add_params_only = Add::encode_params(42);
        assert_eq!(add_params_only, expected_add_payload);

        // Value is "Ctor" mode, encode takes no route_idx and returns raw bytes
        let value_encoded = Value::encode_params();
        assert_eq!(value_encoded, Vec::<u8>::new());
    }
}
