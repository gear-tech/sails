use crate::{gstd::EventEmitter, meta::InterfaceId};

#[macro_export]
macro_rules! hash_fn {
    // accept: command ...
    (
        command $name:ident ( $( $ty:ty ),* $(,)? ) -> $reply:ty $(| $throws:ty )?
    ) => {
        $crate::hash_fn!(@impl "command" $name ( $( $ty ),* ) -> $reply $(| $throws )?)
    };

    // accept: query ...
    (
        query $name:ident ( $( $ty:ty ),* $(,)? ) -> $reply:ty $(| $throws:ty )?
    ) => {
        $crate::hash_fn!(@impl "query" $name ( $( $ty ),* ) -> $reply $(| $throws )?)
    };

    (@impl $kind:literal
        $name:ident ( $( $ty:ty ),* ) -> $reply:ty $(| $throws:ty )?
    ) => {{
        let mut fn_hash = $crate::keccak_const::Keccak256::new();
        fn_hash = fn_hash.update($kind.as_bytes()).update(stringify!($name).as_bytes());
        $( fn_hash = fn_hash.update(&<$ty as $crate::sails_reflect_hash::ReflectHash>::HASH); )*
        fn_hash = fn_hash.update(b"res").update(&<$reply as $crate::sails_reflect_hash::ReflectHash>::HASH);
        $( fn_hash = fn_hash.update(b"throws").update(&<$throws as $crate::sails_reflect_hash::ReflectHash>::HASH); )?
        fn_hash.finalize()
    }};
}

pub trait Service: Sized {
    type Exposure: Exposure;

    fn expose(self, route_idx: u8) -> Self::Exposure;
}

pub trait Exposure {
    fn route_idx(&self) -> u8;
    fn check_asyncness(interface_id: InterfaceId, entry_id: u16) -> Option<bool>;
}

pub trait ExposureWithEvents: Exposure {
    type Events;

    fn emitter(&self) -> EventEmitter<Self::Events> {
        let route_idx = self.route_idx();
        EventEmitter::new(route_idx)
    }
}

pub struct ServiceExposure<T> {
    route_idx: u8,
    inner: T,
}

impl<T: Service> ServiceExposure<T> {
    pub fn new(route_idx: u8, inner: T) -> Self {
        Self { route_idx, inner }
    }

    fn route_idx(&self) -> u8 {
        self.route_idx
    }
}

impl<T> core::ops::Deref for ServiceExposure<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> core::ops::DerefMut for ServiceExposure<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
