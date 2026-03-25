use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    rc::Rc,
    string::String,
    sync::Arc,
    vec::Vec,
};
use core::{
    marker::PhantomData,
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU128,
    },
    ops::{Range, RangeInclusive},
    time::Duration,
};

use crate::registry::{Registry, TypeInfo};
use crate::ty::{Primitive, Type};

macro_rules! impl_type_info_primitive {
    ($($t:ty => $p:ident),* $(,)?) => {
        $(
            impl TypeInfo for $t {
                type Identity = Self;
                fn type_info(_registry: &mut Registry) -> Type {
                    Type::builder().primitive(Primitive::$p)
                }
            }
        )*
    };
}

macro_rules! impl_for_non_zero {
    ( $( $t: ty: $inner: ty ),* $(,)? ) => {
        $(
            impl TypeInfo for $t {
                type Identity = Self;
                fn type_info(registry: &mut Registry) -> Type {
                    Type::builder()
                        .name(::core::stringify!($t))
                        .composite()
                        .unnamed().ty(registry.register_type::<$inner>())
                        .build()
                }
            }
        )*
    };
}

macro_rules! impl_type_info_for_tuples {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: TypeInfo, $($rest: TypeInfo),*> TypeInfo for ($first, $($rest),*) {
            type Identity = Self;
            fn type_info(registry: &mut Registry) -> Type {
                let fields = alloc::vec![
                    registry.register_type::<$first>(),
                    $(registry.register_type::<$rest>()),*
                ];
                Type::builder().tuple(fields)
            }
        }
        impl_type_info_for_tuples!($($rest),*);
    };
}

macro_rules! impl_type_info_transparent {
    ($($t:ty),* $(,)?) => {
        $(
            impl<T: TypeInfo + ?Sized + 'static> TypeInfo for $t {
                type Identity = T::Identity;
                fn type_info(registry: &mut Registry) -> Type {
                    T::type_info(registry)
                }
            }
        )*
    };
}

impl_type_info_for_tuples!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

impl_type_info_transparent!(&'static T, &'static mut T, Box<T>, Rc<T>, Arc<T>);

impl_type_info_primitive! {
    bool => Bool,
    char => Char,
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
}

impl_for_non_zero!(
    NonZeroI8: i8,
    NonZeroI16: i16,
    NonZeroI32: i32,
    NonZeroI64: i64,
    NonZeroI128: i128,
    NonZeroU8: u8,
    NonZeroU16: u16,
    NonZeroU32: u32,
    NonZeroU64: u64,
    NonZeroU128: u128,
);

#[cfg(feature = "gprimitives")]
mod g_impls {
    use super::*;
    use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};

    macro_rules! impl_type_info_gprimitive {
        ($($t:ty => $p:ident),* $(,)?) => {
            $(
                impl TypeInfo for $t {
                    type Identity = Self;
                    fn type_info(_registry: &mut Registry) -> Type {
                        Type::builder().gprimitive(crate::ty::GPrimitive::$p)
                    }
                }
            )*
        };
    }

    impl_type_info_gprimitive! {
        ActorId => ActorId,
        MessageId => MessageId,
        CodeId => CodeId,
        H160 => H160,
        H256 => H256,
        U256 => U256,
    }

    impl TypeInfo for NonZeroU256 {
        type Identity = Self;
        fn type_info(registry: &mut Registry) -> Type {
            Type::builder()
                .name("NonZeroU256")
                .composite()
                .unnamed()
                .ty(registry.register_type::<U256>())
                .build()
        }
    }
}

impl TypeInfo for () {
    type Identity = Self;
    fn type_info(_registry: &mut Registry) -> Type {
        Type::builder().tuple(alloc::vec![])
    }
}

impl TypeInfo for str {
    type Identity = Self;
    fn type_info(_registry: &mut Registry) -> Type {
        Type::builder().primitive(Primitive::Str)
    }
}

impl TypeInfo for String {
    type Identity = str;
    fn type_info(registry: &mut Registry) -> Type {
        str::type_info(registry)
    }
}

impl<T: TypeInfo> TypeInfo for Option<T> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().option(registry.register_type::<T>())
    }
}

impl<T: TypeInfo, E: TypeInfo> TypeInfo for Result<T, E> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().result(registry.register_type::<T>(), registry.register_type::<E>())
    }
}

impl<T: TypeInfo> TypeInfo for [T] {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().sequence(registry.register_type::<T>())
    }
}

impl<T: TypeInfo> TypeInfo for Vec<T> {
    type Identity = [T];
    fn type_info(registry: &mut Registry) -> Type {
        <[T]>::type_info(registry)
    }
}

impl<T: TypeInfo, const N: usize> TypeInfo for [T; N] {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().array(registry.register_type::<T>(), N as u32)
    }
}

impl<K: TypeInfo, V: TypeInfo> TypeInfo for BTreeMap<K, V> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().map(registry.register_type::<K>(), registry.register_type::<V>())
    }
}

impl<T: TypeInfo> TypeInfo for BTreeSet<T> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().sequence(registry.register_type::<T>())
    }
}

impl<T: TypeInfo> TypeInfo for VecDeque<T> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().sequence(registry.register_type::<T>())
    }
}

impl<T: TypeInfo> TypeInfo for BinaryHeap<T> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder().sequence(registry.register_type::<T>())
    }
}

impl<T: TypeInfo> TypeInfo for PhantomData<T> {
    type Identity = PhantomData<T>;
    fn type_info(_registry: &mut Registry) -> Type {
        Type::builder().name("PhantomData").tuple(alloc::vec![])
    }
}

impl<T: TypeInfo> TypeInfo for Range<T> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder()
            .name("Range")
            .composite()
            .field("start")
            .ty(registry.register_type::<T>())
            .field("end")
            .ty(registry.register_type::<T>())
            .build()
    }
}

impl<T: TypeInfo> TypeInfo for RangeInclusive<T> {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder()
            .name("RangeInclusive")
            .composite()
            .field("start")
            .ty(registry.register_type::<T>())
            .field("end")
            .ty(registry.register_type::<T>())
            .build()
    }
}

impl TypeInfo for Duration {
    type Identity = Self;
    fn type_info(registry: &mut Registry) -> Type {
        Type::builder()
            .name("Duration")
            .composite()
            .field("secs")
            .ty(registry.register_type::<u64>())
            .field("nanos")
            .ty(registry.register_type::<u32>())
            .build()
    }
}

impl<T: TypeInfo + Clone + 'static> TypeInfo for Cow<'static, T> {
    type Identity = T::Identity;
    fn type_info(registry: &mut Registry) -> Type {
        T::type_info(registry)
    }
}
