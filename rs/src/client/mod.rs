use crate::prelude::*;
use core::{
    any::TypeId,
    error::Error,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{Stream, StreamExt as _};

#[cfg(not(target_arch = "wasm32"))]
mod mock_env;
#[cfg(not(target_arch = "wasm32"))]
pub use mock_env::{MockEnv, MockParams};

#[cfg(feature = "gtest")]
#[cfg(not(target_arch = "wasm32"))]
mod gtest_env;
#[cfg(feature = "gtest")]
#[cfg(not(target_arch = "wasm32"))]
pub use gtest_env::{BlockRunMode, GtestEnv, GtestParams};

#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
mod gclient_env;
#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
pub use gclient_env::{GclientEnv, GclientParams};

#[cfg(feature = "gstd")]
mod gstd_env;
#[cfg(feature = "gstd")]
pub use gstd_env::{GstdEnv, GstdParams};

pub(crate) const PENDING_CALL_INVALID_STATE: &str =
    "PendingCall polled after completion or invalid state";
pub(crate) const PENDING_CTOR_INVALID_STATE: &str =
    "PendingCtor polled after completion or invalid state";

pub trait GearEnv: Clone {
    type Params: Default;
    type Error: Error;
    type MessageState;
}

pub trait Program: Sized {
    fn deploy<E: GearEnv>(env: E, code_id: CodeId, salt: Vec<u8>) -> Deployment<E, Self> {
        Deployment::new(env, code_id, salt)
    }

    fn client<E: GearEnv>(env: E, program_id: ActorId) -> Actor<E, Self> {
        Actor::new(env, program_id)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type DefaultEnv = MockEnv;

pub type Route = &'static str;

pub struct Deployment<E: GearEnv, A> {
    env: E,
    code_id: CodeId,
    salt: Vec<u8>,
    _phantom: PhantomData<A>,
}

impl<E: GearEnv, A> Deployment<E, A> {
    pub fn new(env: E, code_id: CodeId, salt: Vec<u8>) -> Self {
        Deployment {
            env,
            code_id,
            salt,
            _phantom: PhantomData,
        }
    }

    pub fn with_env<N: GearEnv>(self, env: N) -> Deployment<N, A> {
        let Self {
            env: _,
            code_id,
            salt,
            _phantom: _,
        } = self;
        Deployment {
            env,
            code_id,
            salt,
            _phantom: PhantomData,
        }
    }

    pub fn pending_ctor<T: CallEncodeDecode>(self, args: T::Params) -> PendingCtor<E, A, T> {
        PendingCtor::new(self.env, self.code_id, self.salt, args)
    }
}

pub struct Actor<E: GearEnv, A> {
    env: E,
    id: ActorId,
    _phantom: PhantomData<A>,
}

impl<E: GearEnv, A> Actor<E, A> {
    pub fn new(env: E, id: ActorId) -> Self {
        Actor {
            env,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn with_env<N: GearEnv>(self, env: N) -> Actor<N, A> {
        let Self {
            env: _,
            id,
            _phantom: _,
        } = self;
        Actor {
            env,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn service<S>(&self, route: Route) -> Service<E, S> {
        Service::new(self.env.clone(), self.id, route)
    }
}

pub struct Service<E: GearEnv, S> {
    env: E,
    actor_id: ActorId,
    route: Route,
    _phantom: PhantomData<S>,
}

impl<E: GearEnv, S> Service<E, S> {
    pub fn new(env: E, actor_id: ActorId, route: Route) -> Self {
        Service {
            env,
            actor_id,
            route,
            _phantom: PhantomData,
        }
    }

    pub fn pending_call<T: CallEncodeDecode>(&self, args: T::Params) -> PendingCall<E, T> {
        PendingCall::new(self.env.clone(), self.actor_id, self.route, args)
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCall<E: GearEnv, T: CallEncodeDecode> {
        env: E,
        destination: ActorId,
        route: Option<Route>,
        params: Option<E::Params>,
        args: Option<T::Params>,
        #[pin]
        state: Option<E::MessageState>
    }
}

impl<E: GearEnv, T: CallEncodeDecode> PendingCall<E, T> {
    pub fn new(env: E, destination: ActorId, route: Route, args: T::Params) -> Self {
        PendingCall {
            env,
            destination,
            route: Some(route),
            params: None,
            args: Some(args),
            state: None,
        }
    }

    pub fn with_params(mut self, f: impl FnOnce(E::Params) -> E::Params) -> Self {
        self.params = Some(f(self.params.unwrap_or_default()));
        self
    }
}

pin_project_lite::pin_project! {
    pub struct PendingCtor<E: GearEnv, A, T: CallEncodeDecode> {
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

impl<E: GearEnv, A, T: CallEncodeDecode> PendingCtor<E, A, T> {
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

    fn encode_ctor(&self) -> Vec<u8> {
        if let Some(args) = &self.args {
            T::encode_params(args)
        } else {
            vec![]
        }
    }
}

pub trait CallEncodeDecode {
    const ROUTE: Route;
    type Params: Encode;
    type Reply: Decode + 'static;

    fn encode_params(value: &Self::Params) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::ROUTE.len() + Encode::size_hint(value));
        Encode::encode_to(Self::ROUTE, &mut result);
        Encode::encode_to(value, &mut result);
        result
    }

    fn encode_params_with_prefix(prefix: Route, value: &Self::Params) -> Vec<u8> {
        let mut result = Vec::with_capacity(Self::ROUTE.len() + Encode::size_hint(value));
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

        impl<A, T: CallEncodeDecode> PendingCtor<$env, A, T> {
            $(
                paste::paste! {
                    $(#[$attr])*
                    pub fn [<with_ $field>](self, $field: $ty) -> Self {
                        self.with_params(|params| params.[<with_ $field>]($field))
                    }
                }
            )*
        }

        impl<T: CallEncodeDecode> PendingCall<$env, T> {
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
                <$name as CallEncodeDecode>::encode_params(&( $( $param, )* ))
            }
            pub fn encode_params_with_prefix(prefix: Route, $( $param: $ty, )* ) -> Vec<u8> {
                <$name as CallEncodeDecode>::encode_params_with_prefix(prefix, &( $( $param, )* ))
            }
        }
        impl CallEncodeDecode for $name {
            const ROUTE: &'static str = stringify!($name);
            type Params = ( $( $ty, )* );
            type Reply = $reply;
        }
    };
}

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

// impl<R: Listener<Vec<u8>> + GearEnv, S, E: EventDecode> Listener<E::Event> for Service<R, S> {
//     type Error = parity_scale_codec::Error;

//     async fn listen(
//         &mut self,
//     ) -> Result<impl Stream<Item = (ActorId, E::Event)> + Unpin, Self::Error> {
//         let stream = self.env.listen().await?;
//         let map = stream.filter_map(move |(actor_id, payload)| async move {
//             E::decode_event(self.route, payload)
//                 .ok()
//                 .map(|e| (actor_id, e))
//         });
//         Ok(Box::pin(map))
//     }
// }

pub trait EventDecode {
    const EVENT_NAMES: &'static [&'static [u8]];
    type Event: Decode;

    fn decode_event(
        prefix: Route,
        payload: impl AsRef<[u8]>,
    ) -> Result<Self::Event, parity_scale_codec::Error> {
        let mut payload = payload.as_ref();
        let route = String::decode(&mut payload)?;
        if route != prefix {
            return Err("Invalid event prefix".into());
        }

        for (idx, name) in Self::EVENT_NAMES.iter().enumerate() {
            if payload.starts_with(name) {
                let idx = idx as u8;
                let bytes = [&[idx], &payload[name.len()..]].concat();
                let mut event_bytes = &bytes[..];
                return Decode::decode(&mut event_bytes);
            }
        }
        Err("Invalid event name".into())
    }
}

// mod client {

//     use super::service::Service;
//     use super::*;

//     pub struct MyServiceImpl;

//     pub trait MyService<E: GearEnv> {
//         fn mint(&mut self, to: ActorId, amount: u128) -> PendingCall<E, bool>;
//         fn burn(&mut self, from: ActorId) -> PendingCall<E, u8>;
//         fn total(&self) -> PendingCall<E, u128>;
//     }

//     impl<E: GearEnv> MyService<E> for Service<MyServiceImpl, E> {
//         fn mint(&mut self, to: ActorId, amount: u128) -> PendingCall<E, bool> {
//             self.pending_call("Mint", (to, amount))
//         }

//         fn burn(&mut self, from: ActorId) -> PendingCall<E, u8> {
//             self.pending_call("Burn", (from,))
//         }

//         fn total(&self) -> PendingCall<E, u128> {
//             self.pending_call("Total", ())
//         }
//     }

//     #[cfg(feature = "mockall")]
//     #[cfg(not(target_arch = "wasm32"))]
//     mockall::mock! {
//         pub MyService<E: GearEnv> {}

//         impl<E: GearEnv> MyService<E> for MyService<E> {
//             fn mint(&mut self, to: ActorId, amount: u128) -> PendingCall<E, bool>;
//             fn burn(&mut self, from: ActorId) -> PendingCall<E, u8>;
//             fn total(&self) -> PendingCall<E, u128>;
//         }
//     }
// }

// #[cfg(feature = "mockall")]
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn sample() -> Result<(), Box<dyn Error>> {
//         use client::*;

//         let mut my_service = MockMyService::new();
//         my_service.expect_total().returning(move || 137.into());
//         my_service.expect_mint().returning(move |_, _| true.into());

//         assert_eq!(my_service.total().await?, 137);

//         let mut my_service = my_service;

//         assert!(my_service.mint(ActorId::from(137), 1_000).await?);

//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    io_struct_impl!(Add (value: u32) -> u32);
    io_struct_impl!(Value () -> u32);

    #[test]
    fn test_str_encode() {
        const ADD: &'static [u8] = str_scale_encode!(Add);
        assert_eq!(ADD, &[12, 65, 100, 100]);

        const VALUE: &'static [u8] = str_scale_encode!(Value);
        assert_eq!(VALUE, &[20, 86, 97, 108, 117, 101]);
    }

    #[test]
    fn test_io_struct_impl() {
        let add = Add::encode_params(42);
        assert_eq!(add, &[12, 65, 100, 100, 42, 0, 0, 0]);

        let value = Value::encode_params();
        assert_eq!(value, &[20, 86, 97, 108, 117, 101]);
    }
}
