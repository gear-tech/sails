use alloc::{collections::BTreeMap, vec::Vec};
use core::{any::TypeId, num::NonZeroU32};

use crate::ty::Type;

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

pub trait TypeInfo: 'static {
    type Identity: ?Sized + 'static;
    fn type_info(registry: &mut Registry) -> Type;
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    type_table: BTreeMap<TypeId, TypeRef>,
    types: Vec<Type>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_type<T: TypeInfo + ?Sized>(&mut self) -> TypeRef {
        self.register_meta_type(crate::meta_type::MetaType::new::<T>())
    }

    pub fn register_meta_type(&mut self, meta: crate::meta_type::MetaType) -> TypeRef {
        let type_id = meta.type_id();

        if let Some(&type_ref) = self.type_table.get(&type_id) {
            return type_ref;
        }

        let next_id = (self.types.len() as u32) + 1;
        let type_ref = TypeRef::new(next_id);

        self.type_table.insert(type_id, type_ref);
        self.types.push(Type::placeholder());

        let actual_type = meta.type_info(self);
        self.types[(next_id - 1) as usize] = actual_type;
        type_ref
    }

    pub fn register_type_def(&mut self, ty: Type) -> TypeRef {
        for (id, existing_ty) in self.types() {
            if existing_ty.name == ty.name && existing_ty.def == ty.def {
                return id;
            }
        }
        let next_id = (self.types.len() as u32) + 1;
        let type_ref = TypeRef::new(next_id);
        self.types.push(ty);
        type_ref
    }

    pub fn get_type(&self, type_ref: TypeRef) -> Option<&Type> {
        let index = (type_ref.get() as usize).checked_sub(1)?;
        self.types.get(index)
    }

    pub fn is_type<T: TypeInfo + ?Sized>(&self, type_ref: TypeRef) -> bool {
        let type_id = TypeId::of::<T::Identity>();
        self.type_table.get(&type_id) == Some(&type_ref)
    }

    pub fn types(&self) -> impl Iterator<Item = (TypeRef, &Type)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, t)| (TypeRef::new((i as u32) + 1), t))
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
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
    fn test_registry_deduplication_integrity() {
        use alloc::boxed::Box;
        use alloc::rc::Rc;

        let mut registry = Registry::new();

        // Register the same base type through different transparent wrappers
        let id1 = registry.register_type::<u32>();
        let id2 = registry.register_type::<Box<u32>>();
        let id3 = registry.register_type::<Rc<u32>>();
        let id4 = registry.register_type::<&'static u32>();

        // All IDs must be identical
        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
        assert_eq!(id1, id4);

        // The registry must contain EXACTLY one type
        assert_eq!(
            registry.len(),
            1,
            "Registry should deduplicate transparent wrappers into a single entry"
        );
    }
}
