use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    rc::Rc,
    string::String,
    sync::Arc,
    vec,
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

use crate::builder::TypeBuilder;
use crate::registry::{Registry, TypeInfo};
use sails_idl_ast::{PrimitiveType, Type, TypeDecl};

macro_rules! impl_type_info_primitive {
    ($($t:ty => $p:ident),* $(,)?) => {
        $(
            impl TypeInfo for $t {
                type Identity = Self;
                fn type_decl(_registry: &mut Registry) -> TypeDecl {
                    TypeDecl::Primitive(PrimitiveType::$p)
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
                fn type_decl(_registry: &mut Registry) -> TypeDecl {
                    TypeDecl::named(::core::stringify!($t).into())
                }
                fn type_def(registry: &mut Registry) -> Option<Type> {
                    let inner_ref = registry.register_type::<$inner>();
                    let ty_decl = registry.get_type_decl(inner_ref).cloned().unwrap_or(TypeDecl::named("<unknown>".into()));
                    Some(TypeBuilder::new()
                        .name(::core::stringify!($t))
                        .composite()
                        .unnamed().ty(ty_decl)
                        .build())
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
            fn type_decl(registry: &mut Registry) -> TypeDecl {
                let fields = vec![
                    registry.register_type::<$first>(),
                    $(registry.register_type::<$rest>()),*
                ];
                let types = fields.into_iter()
                    .filter_map(|id| registry.get_type_decl(id).cloned())
                    .collect();
                TypeDecl::tuple(types)
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
                fn type_decl(registry: &mut Registry) -> TypeDecl {
                    T::type_decl(registry)
                }
                fn type_def(registry: &mut Registry) -> Option<Type> {
                    T::type_def(registry)
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
                    fn type_decl(_registry: &mut Registry) -> TypeDecl {
                        TypeDecl::Primitive(PrimitiveType::$p)
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

    #[cfg(feature = "alloy-primitives")]
    mod alloy_impls {
        use super::*;

        impl TypeInfo for alloy_primitives::Address {
            type Identity = Self;
            fn type_decl(_registry: &mut Registry) -> TypeDecl {
                TypeDecl::Primitive(PrimitiveType::H160)
            }
        }

        impl TypeInfo for alloy_primitives::B256 {
            type Identity = Self;
            fn type_decl(_registry: &mut Registry) -> TypeDecl {
                TypeDecl::Primitive(PrimitiveType::H256)
            }
        }
    }

    impl TypeInfo for NonZeroU256 {
        type Identity = Self;
        fn type_decl(_registry: &mut Registry) -> TypeDecl {
            TypeDecl::named("NonZeroU256".into())
        }
        fn type_def(registry: &mut Registry) -> Option<Type> {
            let inner_ref = registry.register_type::<U256>();
            let ty_decl = registry
                .get_type_decl(inner_ref)
                .cloned()
                .unwrap_or(TypeDecl::named("<unknown>".into()));
            Some(
                TypeBuilder::new()
                    .name("NonZeroU256")
                    .composite()
                    .unnamed()
                    .ty(ty_decl)
                    .build(),
            )
        }
    }
}

impl TypeInfo for () {
    type Identity = Self;
    fn type_decl(_registry: &mut Registry) -> TypeDecl {
        TypeDecl::Primitive(PrimitiveType::Void)
    }
}

impl TypeInfo for str {
    type Identity = Self;
    fn type_decl(_registry: &mut Registry) -> TypeDecl {
        TypeDecl::Primitive(PrimitiveType::String)
    }
}

impl TypeInfo for String {
    type Identity = str;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        str::type_decl(registry)
    }
}

impl<T: TypeInfo> TypeInfo for Option<T> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let t_id = registry.register_type::<T>();
        let inner = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::option(inner)
    }
}

impl<T: TypeInfo, E: TypeInfo> TypeInfo for Result<T, E> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let ok_id = registry.register_type::<T>();
        let err_id = registry.register_type::<E>();
        let ok = registry
            .get_type_decl(ok_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        let err = registry
            .get_type_decl(err_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::result(ok, err)
    }
}

impl<T: TypeInfo> TypeInfo for [T] {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let t_id = registry.register_type::<T>();
        let inner = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::Slice {
            item: Box::new(inner),
        }
    }
}

impl<T: TypeInfo> TypeInfo for Vec<T> {
    type Identity = [T];
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        <[T]>::type_decl(registry)
    }
}

impl<T: TypeInfo, const N: usize> TypeInfo for [T; N] {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let t_id = registry.register_type::<T>();
        let inner = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::Array {
            item: Box::new(inner),
            len: N as u32,
        }
    }
}

impl<K: TypeInfo, V: TypeInfo> TypeInfo for BTreeMap<K, V> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let key_id = registry.register_type::<K>();
        let val_id = registry.register_type::<V>();
        let key = registry
            .get_type_decl(key_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        let val = registry
            .get_type_decl(val_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::Slice {
            item: Box::new(TypeDecl::Tuple {
                types: vec![key, val],
            }),
        }
    }
}

impl<T: TypeInfo> TypeInfo for BTreeSet<T> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        <[T]>::type_decl(registry)
    }
}

impl<T: TypeInfo> TypeInfo for VecDeque<T> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        <[T]>::type_decl(registry)
    }
}

impl<T: TypeInfo> TypeInfo for BinaryHeap<T> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        <[T]>::type_decl(registry)
    }
}

impl<T: TypeInfo> TypeInfo for PhantomData<T> {
    type Identity = PhantomData<T>;
    fn type_decl(_registry: &mut Registry) -> TypeDecl {
        TypeDecl::named("PhantomData".into())
    }
    fn type_def(_registry: &mut Registry) -> Option<Type> {
        Some(TypeBuilder::new().name("PhantomData").composite().build())
    }
}

impl<T: TypeInfo> TypeInfo for Range<T> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let t_id = registry.register_type::<T>();
        let inner = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::Named {
            name: "Range".into(),
            generics: vec![inner],
            param: None,
        }
    }
    fn type_def(registry: &mut Registry) -> Option<Type> {
        let t_id = registry.register_type::<T>();
        let t_decl = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        Some(
            TypeBuilder::new()
                .name("Range")
                .composite()
                .field("start")
                .ty(t_decl.clone())
                .field("end")
                .ty(t_decl)
                .build(),
        )
    }
}

impl<T: TypeInfo> TypeInfo for RangeInclusive<T> {
    type Identity = Self;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let t_id = registry.register_type::<T>();
        let inner = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        TypeDecl::Named {
            name: "RangeInclusive".into(),
            generics: vec![inner],
            param: None,
        }
    }
    fn type_def(registry: &mut Registry) -> Option<Type> {
        let t_id = registry.register_type::<T>();
        let t_decl = registry
            .get_type_decl(t_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        Some(
            TypeBuilder::new()
                .name("RangeInclusive")
                .composite()
                .field("start")
                .ty(t_decl.clone())
                .field("end")
                .ty(t_decl)
                .build(),
        )
    }
}

impl TypeInfo for Duration {
    type Identity = Self;
    fn type_decl(_registry: &mut Registry) -> TypeDecl {
        TypeDecl::named("Duration".into())
    }
    fn type_def(registry: &mut Registry) -> Option<Type> {
        let u64_id = registry.register_type::<u64>();
        let u32_id = registry.register_type::<u32>();
        let u64_decl = registry
            .get_type_decl(u64_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        let u32_decl = registry
            .get_type_decl(u32_id)
            .cloned()
            .unwrap_or(TypeDecl::named("<unknown>".into()));
        Some(
            TypeBuilder::new()
                .name("Duration")
                .composite()
                .field("secs")
                .ty(u64_decl)
                .field("nanos")
                .ty(u32_decl)
                .build(),
        )
    }
}

impl<T: TypeInfo + Clone + 'static> TypeInfo for Cow<'static, T> {
    type Identity = T::Identity;
    fn type_decl(registry: &mut Registry) -> TypeDecl {
        T::type_decl(registry)
    }
    fn type_def(registry: &mut Registry) -> Option<Type> {
        T::type_def(registry)
    }
}
