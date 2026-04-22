use core::{
    any::TypeId,
    cmp::Ordering,
    fmt::{Debug, Error as FmtError, Formatter},
    hash::{Hash, Hasher},
};

use sails_idl_ast::{Type, TypeDecl};

use crate::registry::{Registry, TypeInfo};

/// Type-erased handle to a concrete [`TypeInfo`] implementation.
#[derive(Clone, Copy)]
pub struct MetaType {
    fn_type_decl: fn(&mut Registry) -> TypeDecl,
    fn_type_def: fn(&mut Registry) -> Option<Type>,
    fn_module_path: fn() -> &'static str,
    type_id: TypeId,
}

impl MetaType {
    /// Creates a `MetaType` for `T`.
    pub const fn new<T>() -> Self
    where
        T: TypeInfo + ?Sized,
    {
        Self {
            fn_type_decl: T::type_decl,
            fn_type_def: T::type_def,
            fn_module_path: T::module_path,
            type_id: TypeId::of::<T::Identity>(),
        }
    }

    /// Produces the use-site type declaration for the represented type.
    pub fn type_decl(&self, registry: &mut Registry) -> TypeDecl {
        (self.fn_type_decl)(registry)
    }

    /// Produces the stored nominal definition for the represented type, if any.
    pub fn type_def(&self, registry: &mut Registry) -> Option<Type> {
        (self.fn_type_def)(registry)
    }

    /// Returns the module path associated with the represented nominal type.
    pub fn module_path(&self) -> &'static str {
        (self.fn_module_path)()
    }

    /// Returns the unique identity of the represented type.
    pub const fn type_id(&self) -> TypeId {
        self.type_id
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
