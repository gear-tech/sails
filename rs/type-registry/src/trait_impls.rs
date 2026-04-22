use alloc::{
    borrow::Cow,
    borrow::ToOwned,
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

use sails_idl_ast::{PrimitiveType, Type, TypeDecl};

use crate::builder::TypeBuilder;
use crate::registry::{Registry, TypeInfo};

macro_rules! impl_primitive {
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

macro_rules! impl_transparent {
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

                fn module_path() -> &'static str {
                    T::module_path()
                }
            }
        )*
    };
}

macro_rules! impl_tuples {
    () => {};
    ($first:ident $(, $rest:ident)*) => {
        impl<$first: TypeInfo, $($rest: TypeInfo),*> TypeInfo for ($first, $($rest),*) {
            type Identity = Self;

            fn type_decl(registry: &mut Registry) -> TypeDecl {
                TypeDecl::Tuple {
                    types: vec![
                        <$first as TypeInfo>::type_decl(registry),
                        $( <$rest as TypeInfo>::type_decl(registry) ),*
                    ],
                }
            }
        }
        impl_tuples!($($rest),*);
    };
}

macro_rules! impl_non_zero {
    ( $( $t:ty : $inner:ty => $prim:ident ),* $(,)? ) => {
        $(
            impl TypeInfo for $t {
                type Identity = Self;

                fn type_decl(registry: &mut Registry) -> TypeDecl {
                    registry.register_named_type(
                        Self::META,
                        stringify!($t).into(),
                        Vec::new(),
                        |_| {},
                    )
                }

                fn type_def(_registry: &mut Registry) -> Option<Type> {
                    Some(
                        TypeBuilder::new()
                            .name(stringify!($t))
                            .composite()
                            .unnamed()
                            .ty(TypeDecl::Primitive(PrimitiveType::$prim))
                            .build(),
                    )
                }
            }
        )*
    };
}

impl_primitive! {
    bool => Bool,
    char => Char,
    str => String,
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

impl TypeInfo for () {
    type Identity = Self;

    fn type_decl(_registry: &mut Registry) -> TypeDecl {
        TypeDecl::Primitive(PrimitiveType::Void)
    }
}

impl TypeInfo for String {
    type Identity = str;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        <str as TypeInfo>::type_decl(registry)
    }
}

impl_tuples!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);

impl_transparent!(&'static T, &'static mut T, Box<T>, Rc<T>, Arc<T>);

impl<T: TypeInfo> TypeInfo for Option<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Named {
            name: "Option".into(),
            generics: vec![T::type_decl(registry)],
        }
    }
}

impl<T: TypeInfo, E: TypeInfo> TypeInfo for Result<T, E> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Named {
            name: "Result".into(),
            generics: vec![T::type_decl(registry), E::type_decl(registry)],
        }
    }
}

impl<T: TypeInfo> TypeInfo for [T] {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Slice {
            item: Box::new(T::type_decl(registry)),
        }
    }
}

impl<T: TypeInfo> TypeInfo for Vec<T> {
    type Identity = [T];

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        <[T] as TypeInfo>::type_decl(registry)
    }
}

impl<T: TypeInfo, const N: usize> TypeInfo for [T; N] {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Array {
            item: Box::new(T::type_decl(registry)),
            len: N as u32,
        }
    }
}

impl<K: TypeInfo, V: TypeInfo> TypeInfo for BTreeMap<K, V> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Slice {
            item: Box::new(TypeDecl::Tuple {
                types: vec![K::type_decl(registry), V::type_decl(registry)],
            }),
        }
    }
}

impl<T: TypeInfo> TypeInfo for BTreeSet<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Slice {
            item: Box::new(T::type_decl(registry)),
        }
    }
}

impl<T: TypeInfo> TypeInfo for VecDeque<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Slice {
            item: Box::new(T::type_decl(registry)),
        }
    }
}

impl<T: TypeInfo> TypeInfo for BinaryHeap<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        TypeDecl::Slice {
            item: Box::new(T::type_decl(registry)),
        }
    }
}

impl<T: TypeInfo + ToOwned + ?Sized + 'static> TypeInfo for Cow<'static, T> {
    type Identity = T::Identity;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        T::type_decl(registry)
    }

    fn type_def(registry: &mut Registry) -> Option<Type> {
        T::type_def(registry)
    }

    fn module_path() -> &'static str {
        T::module_path()
    }
}

impl<T: TypeInfo> TypeInfo for PhantomData<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let generics = vec![T::type_decl(registry)];
        registry.register_named_type(Self::META, "PhantomData".into(), generics, |_registry| {})
    }

    fn type_def(_registry: &mut Registry) -> Option<Type> {
        Some(
            TypeBuilder::new()
                .name("PhantomData")
                .param("T")
                .composite()
                .build(),
        )
    }
}

impl<T: TypeInfo> TypeInfo for Range<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let generics = vec![T::type_decl(registry)];
        registry.register_named_type(Self::META, "Range".into(), generics, |_registry| {})
    }

    fn type_def(_registry: &mut Registry) -> Option<Type> {
        let t = TypeDecl::Named {
            name: "T".into(),
            generics: Vec::new(),
        };
        Some(
            TypeBuilder::new()
                .name("Range")
                .param("T")
                .composite()
                .field("start")
                .ty(t.clone())
                .field("end")
                .ty(t)
                .build(),
        )
    }
}

impl<T: TypeInfo> TypeInfo for RangeInclusive<T> {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        let generics = vec![T::type_decl(registry)];
        registry.register_named_type(
            Self::META,
            "RangeInclusive".into(),
            generics,
            |_registry| {},
        )
    }

    fn type_def(_registry: &mut Registry) -> Option<Type> {
        let t = TypeDecl::Named {
            name: "T".into(),
            generics: Vec::new(),
        };
        Some(
            TypeBuilder::new()
                .name("RangeInclusive")
                .param("T")
                .composite()
                .field("start")
                .ty(t.clone())
                .field("end")
                .ty(t)
                .build(),
        )
    }
}

impl TypeInfo for Duration {
    type Identity = Self;

    fn type_decl(registry: &mut Registry) -> TypeDecl {
        registry.register_named_type(Self::META, "Duration".into(), Vec::new(), |_registry| {})
    }

    fn type_def(_registry: &mut Registry) -> Option<Type> {
        Some(
            TypeBuilder::new()
                .name("Duration")
                .composite()
                .field("secs")
                .ty(TypeDecl::Primitive(PrimitiveType::U64))
                .field("nanos")
                .ty(TypeDecl::Primitive(PrimitiveType::U32))
                .build(),
        )
    }
}

impl_non_zero! {
    NonZeroI8:   i8   => I8,
    NonZeroI16:  i16  => I16,
    NonZeroI32:  i32  => I32,
    NonZeroI64:  i64  => I64,
    NonZeroI128: i128 => I128,
    NonZeroU8:   u8   => U8,
    NonZeroU16:  u16  => U16,
    NonZeroU32:  u32  => U32,
    NonZeroU64:  u64  => U64,
    NonZeroU128: u128 => U128,
}

#[cfg(feature = "gprimitives")]
mod g_impls {
    use super::*;
    use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};

    macro_rules! impl_g_primitive {
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

    impl_g_primitive! {
        ActorId => ActorId,
        MessageId => MessageId,
        CodeId => CodeId,
        H160 => H160,
        H256 => H256,
        U256 => U256,
    }

    impl TypeInfo for NonZeroU256 {
        type Identity = Self;

        fn type_decl(registry: &mut Registry) -> TypeDecl {
            registry.register_named_type(
                Self::META,
                "NonZeroU256".into(),
                Vec::new(),
                |_registry| {},
            )
        }

        fn type_def(_registry: &mut Registry) -> Option<Type> {
            Some(
                TypeBuilder::new()
                    .name("NonZeroU256")
                    .composite()
                    .unnamed()
                    .ty(TypeDecl::Primitive(PrimitiveType::U256))
                    .build(),
            )
        }
    }
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
