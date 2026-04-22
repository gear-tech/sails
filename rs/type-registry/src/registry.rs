use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{any::TypeId, num::NonZeroU32};

use sails_idl_ast::{StructDef, Type, TypeDecl, TypeDef};

use crate::MetaType;

/// Stable reference to a nominal type stored in a [`Registry`].
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
/// describes the stored nominal definition, when the type has one.
pub trait TypeInfo: 'static {
    /// Canonical identity used when caching a concrete Rust instantiation.
    type Identity: ?Sized + 'static;

    /// Type-erased handle to this type's metadata entry points.
    const META: MetaType = MetaType::new::<Self>();

    /// Builds the concrete use-site declaration of this type.
    fn type_decl(registry: &mut Registry) -> TypeDecl;

    /// Builds the stored nominal definition of this type, if it has one.
    fn type_def(_registry: &mut Registry) -> Option<Type> {
        None
    }

    /// Returns registry-only module metadata used for nominal-name disambiguation.
    fn module_path() -> &'static str {
        ""
    }
}

/// Key for one shared nominal definition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NominalKey {
    pub module_path: &'static str,
    pub base_name: String,
}

/// Concrete Rust instantiation bound to an interned nominal definition.
#[derive(Debug, Clone)]
pub struct ConcreteTypeBinding {
    pub type_ref: TypeRef,
    pub generics: Vec<TypeDecl>,
}

/// Stored nominal definition plus registry-only metadata.
#[derive(Debug, Clone)]
pub struct NominalEntry {
    pub key: NominalKey,
    pub unique_name: String,
    pub ty: Type,
}

struct NominalReservation {
    type_ref: TypeRef,
    final_name: String,
    needs_fill: bool,
    check_existing: bool,
}

/// Nominal-type interner plus concrete-use-site cache.
#[derive(Debug, Default, Clone)]
pub struct Registry {
    concrete_cache: BTreeMap<TypeId, ConcreteTypeBinding>,
    nominal_interner: BTreeMap<NominalKey, TypeRef>,
    used_names: BTreeMap<String, TypeRef>,
    nominal_entries: Vec<NominalEntry>,
}

impl Registry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a nominal type, returning its concrete use-site declaration.
    pub fn register_named_type<F>(
        &mut self,
        meta: MetaType,
        base_name: String,
        generics: Vec<TypeDecl>,
        register_dependencies: F,
    ) -> TypeDecl
    where
        F: FnOnce(&mut Registry),
    {
        let type_id = meta.type_id();
        if let Some(existing) = self.concrete_cache.get(&type_id) {
            return self.concrete_decl(existing);
        }

        let key = NominalKey {
            module_path: meta.module_path(),
            base_name: base_name.clone(),
        };
        let reservation = self.reserve_nominal(key.clone(), &base_name, meta.module_path());

        self.cache_concrete_binding(type_id, reservation.type_ref, generics.clone());

        register_dependencies(self);

        if reservation.check_existing {
            #[cfg(debug_assertions)]
            {
                let mut candidate = meta
                    .type_def(self)
                    .expect("nominal type must define itself");
                candidate.name = reservation.final_name.clone();
                self.assert_compatible_named_def(
                    reservation.type_ref,
                    &candidate,
                    meta.module_path(),
                    &base_name,
                );
            }
        }

        if reservation.needs_fill {
            self.fill_nominal_entry(meta, reservation.type_ref, key, &reservation.final_name);
        }

        named_decl(reservation.final_name, generics)
    }

    /// Registers a Rust type, returning a nominal ref only when the type has one.
    pub fn register_type<T: TypeInfo + ?Sized>(&mut self) -> Option<TypeRef> {
        let _ = T::type_decl(self);
        self.get_registered::<T>()
            .map(|registered| registered.type_ref)
    }

    /// Registers a type through its type-erased [`MetaType`] handle.
    pub fn register_meta_type(&mut self, meta: MetaType) -> TypeRef {
        let _ = meta.type_decl(self);
        self.concrete_cache
            .get(&meta.type_id())
            .map(|registered| registered.type_ref)
            .expect("register_meta_type expects a nominal MetaType")
    }

    /// Returns the cached concrete binding for a type_id.
    pub fn get_meta_binding(&self, type_id: &TypeId) -> Option<&ConcreteTypeBinding> {
        self.concrete_cache.get(type_id)
    }

    /// Returns the concrete declaration for `T`.
    pub fn decl_for<T: TypeInfo + ?Sized>(&mut self) -> TypeDecl {
        T::type_decl(self)
    }

    /// Returns the cached concrete binding for `T`.
    pub fn get_registered<T: TypeInfo + ?Sized>(&self) -> Option<&ConcreteTypeBinding> {
        self.concrete_cache.get(&TypeId::of::<T::Identity>())
    }

    /// Returns the stored nominal entry for `type_ref`.
    pub fn get_entry(&self, type_ref: TypeRef) -> Option<&NominalEntry> {
        self.nominal_entries.get((type_ref.get() - 1) as usize)
    }

    /// Returns the stored nominal type for `type_ref`.
    pub fn get_type(&self, type_ref: TypeRef) -> Option<&Type> {
        self.get_entry(type_ref).map(|entry| &entry.ty)
    }

    /// Returns the nominal type reference with the given final IDL name.
    pub fn get_type_ref_by_name(&self, name: &str) -> Option<TypeRef> {
        self.used_names.get(name).copied()
    }

    /// Returns the stored nominal type with the given final IDL name.
    pub fn get_type_by_name(&self, name: &str) -> Option<(TypeRef, &Type)> {
        let type_ref = self.get_type_ref_by_name(name)?;
        self.get_type(type_ref).map(|ty| (type_ref, ty))
    }

    /// Returns the final unique name for an interned nominal definition.
    pub fn unique_name_for(&self, module_path: &'static str, base_name: &str) -> Option<String> {
        let key = NominalKey {
            module_path,
            base_name: base_name.to_string(),
        };
        self.nominal_interner
            .get(&key)
            .and_then(|type_ref| self.get_entry(*type_ref))
            .map(|entry| entry.unique_name.clone())
    }

    /// Returns `true` when `type_ref` points to the registered nominal identity of `T`.
    pub fn is_type<T: TypeInfo + ?Sized>(&self, type_ref: TypeRef) -> bool {
        self.get_registered::<T>()
            .is_some_and(|registered| registered.type_ref == type_ref)
    }

    /// Iterates over stored nominal types in insertion order.
    pub fn types(&self) -> impl Iterator<Item = (TypeRef, &Type)> {
        self.nominal_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| (TypeRef::new((idx as u32) + 1), &entry.ty))
    }

    /// Returns the number of stored nominal type entries.
    pub fn len(&self) -> usize {
        self.nominal_entries.len()
    }

    /// Returns `true` when the registry has no stored nominal type entries.
    pub fn is_empty(&self) -> bool {
        self.nominal_entries.is_empty()
    }

    fn concrete_decl(&self, binding: &ConcreteTypeBinding) -> TypeDecl {
        let name = self
            .get_type(binding.type_ref)
            .expect("ref valid")
            .name
            .clone();
        named_decl(name, binding.generics.clone())
    }

    fn reserve_nominal(
        &mut self,
        key: NominalKey,
        base_name: &str,
        module_path: &'static str,
    ) -> NominalReservation {
        if let Some(&existing) = self.nominal_interner.get(&key) {
            let final_name = self.get_type(existing).expect("ref valid").name.clone();
            return NominalReservation {
                type_ref: existing,
                final_name,
                needs_fill: false,
                check_existing: true,
            };
        }

        let final_name = self.reserve_unique_name(base_name, module_path);
        let type_ref = self.insert_placeholder_entry(key, final_name.clone());
        NominalReservation {
            type_ref,
            final_name,
            needs_fill: true,
            check_existing: false,
        }
    }

    fn insert_placeholder_entry(&mut self, key: NominalKey, final_name: String) -> TypeRef {
        let slot_idx = self.nominal_entries.len();
        let type_ref = TypeRef::new((slot_idx as u32) + 1);

        self.nominal_entries.push(NominalEntry {
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
        self.nominal_interner.insert(key, type_ref);
        self.used_names.insert(final_name, type_ref);

        type_ref
    }

    fn cache_concrete_binding(
        &mut self,
        type_id: TypeId,
        type_ref: TypeRef,
        generics: Vec<TypeDecl>,
    ) {
        self.concrete_cache
            .insert(type_id, ConcreteTypeBinding { type_ref, generics });
    }

    fn fill_nominal_entry(
        &mut self,
        meta: MetaType,
        type_ref: TypeRef,
        key: NominalKey,
        final_name: &str,
    ) {
        let mut ty = meta
            .type_def(self)
            .expect("nominal type must define itself");
        ty.name = final_name.to_string();
        self.nominal_entries[(type_ref.get() - 1) as usize] = NominalEntry {
            key,
            unique_name: final_name.to_string(),
            ty,
        };
    }

    fn reserve_unique_name(&self, base: &str, module_path: &str) -> String {
        let mut candidate = base.to_string();
        if !self.used_names.contains_key(&candidate) {
            return candidate;
        }

        for segment in module_path
            .rsplit("::")
            .filter(|segment| !segment.is_empty())
        {
            candidate = to_pascal_case(segment) + &candidate;
            if !self.used_names.contains_key(&candidate) {
                return candidate;
            }
        }

        let numeric_base = candidate.clone();
        let mut suffix = 1;
        while self.used_names.contains_key(&candidate) {
            candidate = format!("{numeric_base}{suffix}");
            suffix += 1;
        }
        candidate
    }

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

fn named_decl(name: String, generics: Vec<TypeDecl>) -> TypeDecl {
    TypeDecl::Named { name, generics }
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
