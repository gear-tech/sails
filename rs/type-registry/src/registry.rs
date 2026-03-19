use alloc::{collections::BTreeMap, vec::Vec};
use core::{any::TypeId, num::NonZeroU32};

use crate::ty::{FieldType, Type, TypeDef, TypeDefinitionKind};

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

        let mut actual_type = meta.type_info(self);

        self.expand_type_fields(&mut actual_type);

        self.types[(next_id - 1) as usize] = actual_type;
        type_ref
    }

    pub fn register_type_def(&mut self, ty: Type) -> TypeRef {
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

    pub fn types(&self) -> Types<'_> {
        Types {
            iter: self.types.iter().enumerate(),
        }
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    fn expand_type_fields(&self, ty: &mut Type) {
        if let TypeDef::Definition(def) = &mut ty.def {
            match &mut def.kind {
                TypeDefinitionKind::Composite(comp) => {
                    for field in &mut comp.fields {
                        field.ty = self.expand_aliases(&field.ty);
                    }
                }
                TypeDefinitionKind::Variant(var) => {
                    for variant in &mut var.variants {
                        for field in &mut variant.fields {
                            field.ty = self.expand_aliases(&field.ty);
                        }
                    }
                }
            }
        }
    }

    pub fn expand_aliases(&self, field_type: &FieldType) -> FieldType {
        match field_type {
            FieldType::Id(id) => {
                let Some(actual_ty) = self.get_type(*id) else {
                    return field_type.clone();
                };
                match &actual_ty.def {
                    TypeDef::Sequence(inner) | TypeDef::Option(inner) => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![self.expand_aliases(&FieldType::Id(*inner))],
                    },
                    TypeDef::Result { ok, err } => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![
                            self.expand_aliases(&FieldType::Id(*ok)),
                            self.expand_aliases(&FieldType::Id(*err)),
                        ],
                    },
                    TypeDef::Map { key, value } => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![
                            self.expand_aliases(&FieldType::Id(*key)),
                            self.expand_aliases(&FieldType::Id(*value)),
                        ],
                    },
                    TypeDef::Tuple(elems) => FieldType::Tuple {
                        id: *id,
                        elems: elems
                            .iter()
                            .map(|e| self.expand_aliases(&FieldType::Id(*e)))
                            .collect(),
                    },
                    TypeDef::Array { len, type_param } => FieldType::Array {
                        id: *id,
                        elem: alloc::boxed::Box::new(
                            self.expand_aliases(&FieldType::Id(*type_param)),
                        ),
                        len: crate::ty::ArrayLen::Static(*len),
                    },
                    _ => {
                        let args: Vec<_> = actual_ty
                            .type_params
                            .iter()
                            .filter_map(|p| {
                                if let crate::ty::GenericArg::Type(t) = &p.arg {
                                    Some(self.expand_aliases(&FieldType::Id(*t)))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !args.is_empty() {
                            FieldType::Parameterized { id: *id, args }
                        } else {
                            field_type.clone()
                        }
                    }
                }
            }
            FieldType::Parameterized { id, args } => {
                let Some(actual_ty) = self.get_type(*id) else {
                    return field_type.clone();
                };

                let expanded_args: Vec<FieldType> =
                    args.iter().map(|arg| self.expand_aliases(arg)).collect();

                match &actual_ty.def {
                    TypeDef::Result { ok, err } => {
                        let arg1 = expanded_args
                            .first()
                            .cloned()
                            .unwrap_or_else(|| self.expand_aliases(&FieldType::Id(*ok)));
                        let arg2 = expanded_args
                            .get(1)
                            .cloned()
                            .unwrap_or_else(|| self.expand_aliases(&FieldType::Id(*err)));
                        FieldType::Parameterized {
                            id: *id,
                            args: alloc::vec![arg1, arg2],
                        }
                    }
                    TypeDef::Option(inner) | TypeDef::Sequence(inner) => {
                        let arg1 = expanded_args
                            .first()
                            .cloned()
                            .unwrap_or_else(|| self.expand_aliases(&FieldType::Id(*inner)));
                        FieldType::Parameterized {
                            id: *id,
                            args: alloc::vec![arg1],
                        }
                    }
                    TypeDef::Map { key, value } => {
                        let arg1 = expanded_args
                            .first()
                            .cloned()
                            .unwrap_or_else(|| self.expand_aliases(&FieldType::Id(*key)));
                        let arg2 = expanded_args
                            .get(1)
                            .cloned()
                            .unwrap_or_else(|| self.expand_aliases(&FieldType::Id(*value)));
                        FieldType::Parameterized {
                            id: *id,
                            args: alloc::vec![arg1, arg2],
                        }
                    }
                    TypeDef::Tuple(elems) => FieldType::Tuple {
                        id: *id,
                        elems: elems
                            .iter()
                            .map(|e| self.expand_aliases(&FieldType::Id(*e)))
                            .collect(),
                    },
                    TypeDef::Array { len, type_param } => FieldType::Array {
                        id: *id,
                        elem: alloc::boxed::Box::new(
                            self.expand_aliases(&FieldType::Id(*type_param)),
                        ),
                        len: crate::ty::ArrayLen::Static(*len),
                    },
                    _ => {
                        let mut final_args = Vec::new();
                        let mut provided_idx = 0;

                        for param in &actual_ty.type_params {
                            if let crate::ty::GenericArg::Type(arg_id) = &param.arg {
                                if provided_idx < expanded_args.len() {
                                    final_args.push(expanded_args[provided_idx].clone());
                                    provided_idx += 1;
                                } else {
                                    final_args.push(self.expand_aliases(&FieldType::Id(*arg_id)));
                                }
                            }
                        }

                        if final_args.is_empty() && !expanded_args.is_empty() {
                            final_args = expanded_args;
                        }

                        FieldType::Parameterized {
                            id: *id,
                            args: final_args,
                        }
                    }
                }
            }
            FieldType::Array { id, elem, len } => FieldType::Array {
                id: *id,
                elem: alloc::boxed::Box::new(self.expand_aliases(elem)),
                len: len.clone(),
            },
            FieldType::Tuple { id, elems } => FieldType::Tuple {
                id: *id,
                elems: elems.iter().map(|e| self.expand_aliases(e)).collect(),
            },
            _ => field_type.clone(),
        }
    }
}

pub struct Types<'a> {
    iter: core::iter::Enumerate<core::slice::Iter<'a, Type>>,
}

impl<'a> Iterator for Types<'a> {
    type Item = (TypeRef, &'a Type);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(i, t)| (TypeRef::new((i as u32) + 1), t))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for Types<'a> {}

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
