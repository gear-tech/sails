use core::{
    any::TypeId,
    cmp::Ordering,
    fmt::{Debug, Error as FmtError, Formatter},
    hash::{Hash, Hasher},
};

use crate::registry::{Registry, TypeInfo, TypeRef};
use crate::ty::Type;

#[derive(Clone, Copy)]
pub struct MetaType {
    fn_type_info: fn(&mut Registry) -> Type,
    type_id: TypeId,
}

impl MetaType {
    pub fn new<T>() -> Self
    where
        T: TypeInfo + ?Sized,
    {
        Self {
            fn_type_info: T::type_info,
            type_id: TypeId::of::<T::Identity>(),
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn type_info(&self, registry: &mut Registry) -> Type {
        (self.fn_type_info)(registry)
    }

    pub fn register(&self, registry: &mut Registry) -> TypeRef {
        registry.register_meta_type(*self)
    }
}

impl PartialEq for MetaType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for MetaType {}

impl PartialOrd for MetaType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetaType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl Hash for MetaType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl Debug for MetaType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.type_id.fmt(f)
    }
}
