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
    fn interface_id() -> InterfaceId;
    fn route_idx(&self) -> u8;
    fn check_asyncness(interface_id: InterfaceId, entry_id: u16) -> Option<bool>;
}

pub trait ExposureWithEvents: Exposure {
    type Events;

    fn emitter(&self) -> EventEmitter<Self::Events> {
        EventEmitter::new(Self::interface_id(), self.route_idx())
    }
}
