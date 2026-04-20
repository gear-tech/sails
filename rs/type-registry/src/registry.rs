use alloc::{collections::BTreeMap, vec::Vec};
use core::{any::TypeId, fmt, num::NonZeroU32};
use sails_idl_ast::{Type, TypeDecl};

use crate::MetaType;

/// Stable reference to a type stored in a [`Registry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TypeRef(NonZeroU32);

impl TypeRef {
    pub fn new(id: u32) -> Self {
        Self(NonZeroU32::new(id).expect("Type ID must not be zero"))
    }
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

/// Trait for exposing a Rust type as portable metadata.
pub trait TypeInfo: 'static {
    type Identity: ?Sized + 'static;
    const META: MetaType = MetaType::new::<Self>();

    /// Returns the type declaration (instance representation) used in fields.
    fn type_decl(registry: &mut Registry) -> TypeDecl;

    /// Returns the structural definition (template representation) if this is a custom struct/enum.
    fn type_def(_registry: &mut Registry) -> Option<Type> {
        None
    }
}

/// Deduplicated table of portable type metadata.
#[derive(Default, Debug, Clone)]
pub struct Registry {
    type_table: BTreeMap<TypeId, TypeRef>,
    type_defs: BTreeMap<TypeRef, Type>,
    type_decls: Vec<TypeDecl>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_type<T: TypeInfo + ?Sized>(&mut self) -> TypeRef {
        self.register_meta_type(crate::meta_type::MetaType::new::<T>())
    }

    pub fn register_meta_type(&mut self, meta: MetaType) -> TypeRef {
        let type_id = meta.type_id();

        if let Some(&type_ref) = self.type_table.get(&type_id) {
            return type_ref;
        }

        let next_id = (self.type_decls.len() as u32) + 1;
        let type_ref = TypeRef::new(next_id);

        self.type_table.insert(type_id, type_ref);
        // Placeholder to handle recursive structures
        self.type_decls.push(TypeDecl::Tuple { types: Vec::new() });

        let decl = meta.type_decl(self);
        self.type_decls[(next_id - 1) as usize] = decl;

        if let Some(def) = meta.type_def(self) {
            self.type_defs.insert(type_ref, def);
        }

        type_ref
    }

    pub fn get_type_decl(&self, type_ref: TypeRef) -> Option<&TypeDecl> {
        self.type_decls.get((type_ref.get() - 1) as usize)
    }

    /// Returns a `Type` by TypeRef.
    /// Returns the full struct/enum definition if one was registered.
    pub fn get_type(&self, type_ref: TypeRef) -> Option<&Type> {
        self.type_defs.get(&type_ref)
    }

    pub fn named_types(&self) -> impl Iterator<Item = &Type> {
        self.type_defs.values()
    }

    pub fn is_type<T: TypeInfo + ?Sized>(&self, type_ref: TypeRef) -> bool {
        let type_id = TypeId::of::<T::Identity>();
        self.type_table.get(&type_id) == Some(&type_ref)
    }

    /// Checks if a `TypeDecl` matches the expected Rust type.
    pub fn is_type_decl<T: TypeInfo + ?Sized>(&self, decl: &TypeDecl) -> bool {
        let type_id = TypeId::of::<T::Identity>();
        if let Some(&type_ref) = self.type_table.get(&type_id) {
            return self.get_type_decl(type_ref) == Some(decl);
        }
        let mut temp = Registry::new();
        decl == &T::type_decl(&mut temp)
    }

    pub fn types(&self) -> impl Iterator<Item = (TypeRef, &TypeDecl)> {
        self.type_decls
            .iter()
            .enumerate()
            .map(|(i, t)| (TypeRef::new((i as u32) + 1), t))
    }

    pub fn len(&self) -> usize {
        self.type_decls.len()
    }

    pub fn is_empty(&self) -> bool {
        self.type_decls.is_empty()
    }

    pub fn display(&self, type_ref: TypeRef) -> TypeDisplay<'_> {
        TypeDisplay {
            registry: self,
            type_ref,
        }
    }
}

pub struct TypeDisplay<'a> {
    registry: &'a Registry,
    type_ref: TypeRef,
}

impl fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(decl) = self.registry.get_type_decl(self.type_ref) {
            write!(f, "{}", decl)
        } else {
            write!(f, "<unknown>")
        }
    }
}
