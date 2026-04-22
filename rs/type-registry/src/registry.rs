use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{any::TypeId, num::NonZeroU32};

use sails_idl_ast::{StructDef, Type, TypeDecl, TypeDef};

use crate::MetaType;

/// Stable reference to a named type stored in a [`Registry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TypeRef(NonZeroU32);

impl TypeRef {
    /// Creates a new non-zero type reference.
    pub fn new(id: u32) -> Self {
        Self(NonZeroU32::new(id).expect("Type ID must not be zero"))
    }

    /// Returns the raw numeric identifier.
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

/// Trait for exposing a Rust type as IDL metadata.
///
/// `type_decl` describes how the type appears at a use site. `type_def`
/// describes the stored named definition, when the type has one.
pub trait TypeInfo: 'static {
    /// Canonical identity used when caching a concrete Rust instantiation.
    type Identity: ?Sized + 'static;

    /// Type-erased handle to this type's metadata entry points.
    const META: MetaType = MetaType::new::<Self>();

    /// Builds the concrete use-site declaration of this type.
    fn type_decl(registry: &mut Registry) -> TypeDecl;

    /// Builds the stored named definition of this type, if it has one.
    fn type_def(_registry: &mut Registry) -> Option<Type> {
        None
    }

    /// Returns registry-only module metadata used for named-type disambiguation.
    fn module_path() -> &'static str {
        ""
    }
}

/// Key for one shared named definition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NamedKey {
    pub module_path: &'static str,
    pub base_name: String,
}

/// Concrete Rust instantiation bound to an interned named definition.
#[derive(Debug, Clone)]
pub struct ConcreteTypeBinding {
    pub type_ref: TypeRef,
    pub generics: Vec<TypeDecl>,
}

/// Stored named definition plus registry-only metadata.
#[derive(Debug, Clone)]
pub struct NamedEntry {
    pub key: NamedKey,
    pub unique_name: String,
    pub ty: Type,
}

struct NamedReservation {
    type_ref: TypeRef,
    key: NamedKey,
    final_name: String,
    needs_fill: bool,
}

/// Named-type interner plus concrete-use-site cache.
#[derive(Debug, Default, Clone)]
pub struct Registry {
    concrete_cache: BTreeMap<TypeId, ConcreteTypeBinding>,
    named_interner: BTreeMap<NamedKey, TypeRef>,
    used_names: BTreeMap<String, TypeRef>,
    named_entries: Vec<NamedEntry>,
}

impl Registry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a named type and returns its concrete use-site declaration.
    ///
    /// This is the common path for named types whose field dependencies were
    /// already lowered while building `generics`, or for named types without
    /// dependencies. The registry interns one shared named definition per
    /// `(module_path, base_name)` and caches this concrete `TypeId` separately
    /// with its applied generic arguments.
    pub fn register_named_type(
        &mut self,
        meta: MetaType,
        base_name: impl Into<String>,
        generics: Vec<TypeDecl>,
    ) -> TypeDecl {
        self.register_named_type_with_dependencies(meta, base_name, generics, |_| {})
    }

    /// Registers a named type and walks concrete dependencies after caching this use site.
    ///
    /// Derive-generated implementations use this for structs/enums because
    /// nested fields can refer back to the currently registering type. The
    /// concrete cache is populated before `register_dependencies` runs, which
    /// prevents recursive named-type graphs from re-entering the same concrete
    /// registration indefinitely.
    pub fn register_named_type_with_dependencies<F>(
        &mut self,
        meta: MetaType,
        base_name: impl Into<String>,
        generics: Vec<TypeDecl>,
        register_dependencies: F,
    ) -> TypeDecl
    where
        F: FnOnce(&mut Registry),
    {
        let base_name = base_name.into();
        let type_id = meta.type_id();
        if let Some(existing) = self.concrete_cache.get(&type_id) {
            return self.concrete_decl(existing);
        }

        let module_path = meta.module_path();
        let reservation = self.reserve_named_type(module_path, &base_name);

        self.cache_concrete_binding(type_id, reservation.type_ref, generics.clone());

        register_dependencies(self);

        if reservation.needs_fill {
            self.fill_named_entry(
                meta,
                reservation.type_ref,
                reservation.key,
                &reservation.final_name,
            );
        } else {
            #[cfg(debug_assertions)]
            {
                let mut candidate = meta.type_def(self).expect("named type must define itself");
                candidate.name = reservation.final_name.clone();
                self.assert_compatible_named_def(
                    reservation.type_ref,
                    &candidate,
                    module_path,
                    &base_name,
                );
            }
        }

        TypeDecl::named_with_generics(reservation.final_name, generics)
    }

    /// Registers a Rust type, returning a named ref only when the type has one.
    ///
    /// Structural and primitive types can still register nested named-type
    /// dependencies, but they do not get a `TypeRef` of their own.
    pub fn register_type<T: TypeInfo + ?Sized>(&mut self) -> Option<TypeRef> {
        let _ = T::type_decl(self);
        self.get_registered::<T>()
            .map(|registered| registered.type_ref)
    }

    /// Registers a type through its type-erased [`MetaType`] handle.
    ///
    /// This is intended for metadata paths that are known to be named. It
    /// panics when the represented type only lowers to a structural or primitive
    /// declaration.
    pub fn register_meta_type(&mut self, meta: MetaType) -> TypeRef {
        let _ = meta.type_decl(self);
        self.concrete_cache
            .get(&meta.type_id())
            .map(|registered| registered.type_ref)
            .expect("register_meta_type expects a named MetaType")
    }

    /// Returns the cached concrete binding for a raw [`TypeId`].
    ///
    /// The binding records both the shared named `TypeRef` and the concrete
    /// generic arguments observed at that use site.
    pub fn get_meta_binding(&self, type_id: &TypeId) -> Option<&ConcreteTypeBinding> {
        self.concrete_cache.get(type_id)
    }

    /// Returns the concrete declaration for `T`, registering dependencies as needed.
    pub fn decl_for<T: TypeInfo + ?Sized>(&mut self) -> TypeDecl {
        T::type_decl(self)
    }

    /// Returns the cached concrete binding for `T`.
    ///
    /// This is a pure lookup. Call [`Registry::decl_for`] or
    /// [`Registry::register_type`] first when the type might not have been
    /// lowered yet.
    pub fn get_registered<T: TypeInfo + ?Sized>(&self) -> Option<&ConcreteTypeBinding> {
        self.concrete_cache.get(&TypeId::of::<T::Identity>())
    }

    /// Returns the stored named entry for `type_ref`.
    ///
    /// Entries include registry-only metadata such as the original named key
    /// and final unique IDL name.
    pub fn get_entry(&self, type_ref: TypeRef) -> Option<&NamedEntry> {
        self.named_entries.get((type_ref.get() - 1) as usize)
    }

    /// Returns the stored named type for `type_ref`.
    pub fn get_type(&self, type_ref: TypeRef) -> Option<&Type> {
        self.get_entry(type_ref).map(|entry| &entry.ty)
    }

    /// Returns the named type reference with the given final IDL name.
    pub fn get_type_ref_by_name(&self, name: &str) -> Option<TypeRef> {
        self.used_names.get(name).copied()
    }

    /// Returns the stored named type with the given final IDL name.
    pub fn get_type_by_name(&self, name: &str) -> Option<(TypeRef, &Type)> {
        let type_ref = self.get_type_ref_by_name(name)?;
        self.get_type(type_ref).map(|ty| (type_ref, ty))
    }

    /// Returns the final unique name for an interned named definition.
    ///
    /// The name exists only after the named type has been registered. It is
    /// useful when external code has the original module path and base name but
    /// needs the collision-free IDL name.
    pub fn unique_name_for(&self, module_path: &'static str, base_name: &str) -> Option<String> {
        let key = NamedKey {
            module_path,
            base_name: base_name.to_string(),
        };
        self.named_interner
            .get(&key)
            .and_then(|type_ref| self.get_entry(*type_ref))
            .map(|entry| entry.unique_name.clone())
    }

    /// Returns `true` when `type_ref` points to the registered named identity of `T`.
    pub fn is_type<T: TypeInfo + ?Sized>(&self, type_ref: TypeRef) -> bool {
        self.get_registered::<T>()
            .is_some_and(|registered| registered.type_ref == type_ref)
    }

    /// Iterates over stored named types in stable insertion order.
    ///
    /// The yielded `TypeRef` values are derived from the one-based slot index
    /// used by the registry.
    pub fn types(&self) -> impl Iterator<Item = (TypeRef, &Type)> {
        self.named_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| (TypeRef::new((idx as u32) + 1), &entry.ty))
    }

    /// Returns the number of stored named type entries.
    pub fn len(&self) -> usize {
        self.named_entries.len()
    }

    /// Returns `true` when the registry has no stored named type entries.
    pub fn is_empty(&self) -> bool {
        self.named_entries.is_empty()
    }

    /// Rebuilds a concrete use-site declaration from a cached binding.
    fn concrete_decl(&self, binding: &ConcreteTypeBinding) -> TypeDecl {
        let name = self
            .get_type(binding.type_ref)
            .expect("ref valid")
            .name
            .clone();
        TypeDecl::named_with_generics(name, binding.generics.clone())
    }

    /// Returns an existing named-type slot or reserves a placeholder for a new one.
    ///
    /// The placeholder lets recursive type graphs obtain a stable `TypeRef`
    /// before the final `Type` definition has been built.
    fn reserve_named_type(
        &mut self,
        module_path: &'static str,
        base_name: &str,
    ) -> NamedReservation {
        let key = NamedKey {
            module_path,
            base_name: base_name.to_string(),
        };

        if let Some(&existing) = self.named_interner.get(&key) {
            let final_name = self.get_type(existing).expect("ref valid").name.clone();
            return NamedReservation {
                type_ref: existing,
                key,
                final_name,
                needs_fill: false,
            };
        }

        let final_name = self.reserve_unique_name(base_name, module_path);
        let type_ref = self.insert_placeholder_entry(key.clone(), final_name.clone());
        NamedReservation {
            type_ref,
            key,
            final_name,
            needs_fill: true,
        }
    }

    /// Inserts an empty named entry and indexes it by key and final name.
    fn insert_placeholder_entry(&mut self, key: NamedKey, final_name: String) -> TypeRef {
        let slot_idx = self.named_entries.len();
        let type_ref = TypeRef::new((slot_idx as u32) + 1);

        self.named_entries.push(NamedEntry {
            key: key.clone(),
            unique_name: final_name.clone(),
            ty: Type {
                name: final_name.clone(),
                type_params: Vec::new(),
                def: TypeDef::Struct(StructDef { fields: Vec::new() }),
                docs: Vec::new(),
                annotations: Vec::new(),
            },
        });
        self.named_interner.insert(key, type_ref);
        self.used_names.insert(final_name, type_ref);

        type_ref
    }

    /// Stores the concrete `TypeId` to named `TypeRef` plus generic arguments mapping.
    fn cache_concrete_binding(
        &mut self,
        type_id: TypeId,
        type_ref: TypeRef,
        generics: Vec<TypeDecl>,
    ) {
        self.concrete_cache
            .insert(type_id, ConcreteTypeBinding { type_ref, generics });
    }

    /// Replaces a placeholder entry with the final named definition.
    fn fill_named_entry(
        &mut self,
        meta: MetaType,
        type_ref: TypeRef,
        key: NamedKey,
        final_name: &str,
    ) {
        let mut ty = meta.type_def(self).expect("named type must define itself");
        ty.name = final_name.to_string();
        self.named_entries[(type_ref.get() - 1) as usize] = NamedEntry {
            key,
            unique_name: final_name.to_string(),
            ty,
        };
    }

    /// Chooses a collision-free IDL name for `base` using module-prefix fallback.
    fn reserve_unique_name(&self, base: &str, module_path: &str) -> String {
        let mut candidate = base.to_string();
        if self.name_is_available(&candidate) {
            return candidate;
        }

        for segment in module_path
            .rsplit("::")
            .filter(|segment| !segment.is_empty())
        {
            candidate = to_pascal_case(segment) + &candidate;
            if self.name_is_available(&candidate) {
                return candidate;
            }
        }

        let numeric_base = candidate.clone();
        let mut suffix = 1;
        while !self.name_is_available(&candidate) {
            candidate = format!("{numeric_base}{suffix}");
            suffix += 1;
        }
        candidate
    }

    /// Returns whether `name` is unused by any registered named entry.
    fn name_is_available(&self, name: &str) -> bool {
        !self.used_names.contains_key(name)
    }

    /// Verifies that a reused named key still produces the same abstract definition.
    ///
    /// This runs only in debug assertions and catches derive/implementation bugs
    /// where two concrete instantiations of the same named key disagree on the
    /// stored shared definition.
    fn assert_compatible_named_def(
        &self,
        existing_ref: TypeRef,
        candidate: &Type,
        module_path: &'static str,
        base_name: &str,
    ) {
        let existing = self.get_type(existing_ref).expect("ref valid");
        assert_eq!(
            existing.type_params, candidate.type_params,
            "conflicting type params for {module_path}::{base_name}",
        );
        assert_eq!(
            existing.def, candidate.def,
            "conflicting type definition for {module_path}::{base_name}",
        );
        assert_eq!(
            existing.docs, candidate.docs,
            "conflicting docs for {module_path}::{base_name}",
        );
        assert_eq!(
            existing.annotations, candidate.annotations,
            "conflicting annotations for {module_path}::{base_name}",
        );
    }
}

/// Builds a derive-owned name with const generic values encoded in a stable order.
pub fn const_suffixed_name(base: &str, mut consts: Vec<(String, String)>) -> String {
    consts.sort_by(|a, b| a.0.cmp(&b.0));

    let mut suffix = String::new();
    for (name, value) in &consts {
        suffix.push_str(name);
        suffix.push_str(value);
    }

    format!("{base}{suffix}")
}

fn to_pascal_case(value: &str) -> String {
    let mut out = String::new();
    let mut uppercase_next = true;

    for ch in value.chars() {
        if ch == '_' || ch == '-' {
            uppercase_next = true;
            continue;
        }

        if uppercase_next {
            out.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            out.push(ch);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn test_type_ref_niche_optimization() {
        assert_eq!(size_of::<TypeRef>(), 4);
        assert_eq!(size_of::<Option<TypeRef>>(), 4);
        assert_eq!(size_of::<Option<u32>>(), 8);
    }

    #[test]
    fn test_type_ref_behavior() {
        let t = TypeRef::new(1);
        assert_eq!(t.get(), 1);
    }

    #[test]
    #[should_panic(expected = "Type ID must not be zero")]
    fn test_type_ref_zero_panics() {
        let _ = TypeRef::new(0);
    }

    #[test]
    fn test_registry_initial_state() {
        let registry = Registry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn const_suffix_orders_params_by_name() {
        let name = const_suffixed_name(
            "Matrix",
            alloc::vec![
                ("ROWS".to_string(), "3".to_string()),
                ("COLS".to_string(), "4".to_string()),
            ],
        );

        assert_eq!(name, "MatrixCOLS4ROWS3");
    }
}
