use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::{any::TypeId, num::NonZeroU32};

use crate::ty::{FieldType, Type, TypeDef};

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

        // --- TWO CHAIRS PATTERN ---
        // 1. Create a clone for the expanded (monomorphized) definition
        let mut expanded_type = actual_type.clone();
        self.real_expand_type_fields(&mut expanded_type);

        // 2. Save the expanded definition side-by-side with the original template
        actual_type.expanded_def = Some(expanded_type.def);

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

    pub fn expand_aliases(&self, field_type: &FieldType) -> FieldType {
        // Called by macro. Keep it as a pass-through to preserve template for idl-gen.
        field_type.clone()
    }

    fn real_expand_type_fields(&self, ty: &mut Type) {
        match &mut ty.def {
            TypeDef::Composite(comp) => {
                for field in &mut comp.fields {
                    field.ty = self.real_expand_aliases(&field.ty);
                }
            }
            TypeDef::Variant(var) => {
                for variant in &mut var.variants {
                    for field in &mut variant.fields {
                        field.ty = self.real_expand_aliases(&field.ty);
                    }
                }
            }
            _ => {}
        }
    }

    fn real_expand_aliases(&self, field_type: &FieldType) -> FieldType {
        match field_type {
            FieldType::Id(id) => {
                let Some(actual_ty) = self.get_type(*id) else {
                    return field_type.clone();
                };
                match &actual_ty.def {
                    TypeDef::Sequence(inner) | TypeDef::Option(inner) => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![self.real_expand_aliases(&FieldType::Id(*inner))],
                    },
                    TypeDef::Result { ok, err } => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![
                            self.real_expand_aliases(&FieldType::Id(*ok)),
                            self.real_expand_aliases(&FieldType::Id(*err)),
                        ],
                    },
                    TypeDef::Map { key, value } => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![
                            self.real_expand_aliases(&FieldType::Id(*key)),
                            self.real_expand_aliases(&FieldType::Id(*value)),
                        ],
                    },
                    TypeDef::Tuple(elems) => FieldType::Tuple {
                        id: *id,
                        elems: elems
                            .iter()
                            .map(|e| self.real_expand_aliases(&FieldType::Id(*e)))
                            .collect(),
                    },
                    TypeDef::Array { len, type_param } => FieldType::Array {
                        id: *id,
                        elem: alloc::boxed::Box::new(
                            self.real_expand_aliases(&FieldType::Id(*type_param)),
                        ),
                        len: crate::ty::ArrayLen::Static(*len),
                    },
                    _ => {
                        let args = self.expand_args(&actual_ty.type_params, &BTreeMap::new());

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

                let expanded_args: Vec<FieldType> = args
                    .iter()
                    .map(|arg| self.real_expand_aliases(arg))
                    .collect();

                let mut mapping = BTreeMap::new();
                let mut provided_idx = 0;
                for param in &actual_ty.type_params {
                    if let crate::ty::GenericArg::Type(_) = &param.arg
                        && provided_idx < expanded_args.len()
                    {
                        mapping.insert(param.name.clone(), expanded_args[provided_idx].clone());
                        provided_idx += 1;
                    }
                }

                match &actual_ty.def {
                    TypeDef::Result { ok, err } => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![
                            self.expand_and_substitute(*ok, &mapping),
                            self.expand_and_substitute(*err, &mapping),
                        ],
                    },
                    TypeDef::Option(inner) | TypeDef::Sequence(inner) => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![self.expand_and_substitute(*inner, &mapping)],
                    },
                    TypeDef::Map { key, value } => FieldType::Parameterized {
                        id: *id,
                        args: alloc::vec![
                            self.expand_and_substitute(*key, &mapping),
                            self.expand_and_substitute(*value, &mapping),
                        ],
                    },
                    TypeDef::Tuple(elems) => FieldType::Tuple {
                        id: *id,
                        elems: elems
                            .iter()
                            .map(|e| self.expand_and_substitute(*e, &mapping))
                            .collect(),
                    },
                    TypeDef::Array { len, type_param } => FieldType::Array {
                        id: *id,
                        elem: alloc::boxed::Box::new(
                            self.expand_and_substitute(*type_param, &mapping),
                        ),
                        len: crate::ty::ArrayLen::Static(*len),
                    },
                    _ => {
                        let mut final_args = self.expand_args(&actual_ty.type_params, &mapping);

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
                elem: alloc::boxed::Box::new(self.real_expand_aliases(elem)),
                len: len.clone(),
            },
            FieldType::Tuple { id, elems } => FieldType::Tuple {
                id: *id,
                elems: elems.iter().map(|e| self.real_expand_aliases(e)).collect(),
            },
            _ => field_type.clone(),
        }
    }

    fn substitute_params(&self, ty: FieldType, mapping: &BTreeMap<String, FieldType>) -> FieldType {
        match ty {
            FieldType::Parameter(name) => mapping
                .get(&name)
                .cloned()
                .unwrap_or(FieldType::Parameter(name)),
            FieldType::Parameterized { id, args } => FieldType::Parameterized {
                id,
                args: args
                    .into_iter()
                    .map(|arg| self.substitute_params(arg, mapping))
                    .collect(),
            },
            FieldType::Array { id, elem, len } => FieldType::Array {
                id,
                elem: alloc::boxed::Box::new(self.substitute_params(*elem, mapping)),
                len,
            },
            FieldType::Tuple { id, elems } => FieldType::Tuple {
                id,
                elems: elems
                    .into_iter()
                    .map(|e| self.substitute_params(e, mapping))
                    .collect(),
            },
            _ => ty,
        }
    }

    #[inline]
    fn expand_and_substitute(
        &self,
        id: TypeRef,
        mapping: &BTreeMap<String, FieldType>,
    ) -> FieldType {
        let expanded = self.real_expand_aliases(&FieldType::Id(id));
        self.substitute_params(expanded, mapping)
    }

    fn expand_args(
        &self,
        params: &[crate::ty::TypeParameter],
        mapping: &BTreeMap<String, FieldType>,
    ) -> Vec<FieldType> {
        let mut args = Vec::new();
        for param in params {
            if let crate::ty::GenericArg::Type(arg_id) = &param.arg {
                if let Some(mapped) = mapping.get(&param.name) {
                    args.push(mapped.clone());
                } else {
                    args.push(self.real_expand_aliases(&FieldType::Id(*arg_id)));
                }
            }
        }
        args
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
