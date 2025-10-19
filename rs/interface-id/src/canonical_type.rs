#![allow(clippy::result_large_err)]

extern crate alloc;

use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};

use crate::canonical::{CanonicalEnumVariant, CanonicalStructField, CanonicalType};
use scale_info::{PortableRegistry, TypeDef, TypeDefPrimitive, Variant, form::PortableForm};

/// Errors that can occur while resolving canonical types from SCALE metadata.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CanonicalTypeError {
    #[error("could not resolve type id `{0}` in registry")]
    UnknownType(u32),
}

pub type CanonicalTypeResult<T> = core::result::Result<T, CanonicalTypeError>;

/// Resolves a type registered in the SCALE registry into its canonical representation.
pub fn canonical_type(
    registry: &PortableRegistry,
    type_id: u32,
) -> CanonicalTypeResult<CanonicalType> {
    let mut visited = BTreeSet::new();
    canonical_type_inner(registry, type_id, &mut visited)
}

fn canonical_type_inner(
    registry: &PortableRegistry,
    type_id: u32,
    visited: &mut BTreeSet<u32>,
) -> CanonicalTypeResult<CanonicalType> {
    if visited.contains(&type_id) {
        return Ok(named_type(registry, type_id));
    }
    visited.insert(type_id);

    let ty = registry
        .resolve(type_id)
        .ok_or(CanonicalTypeError::UnknownType(type_id))?;

    use CanonicalType::*;

    let result = match &ty.type_def {
        TypeDef::Composite(def) => {
            if def.fields.is_empty() {
                CanonicalType::Unit
            } else if def.fields.iter().all(|f| f.name.is_some()) {
                Struct {
                    fields: def
                        .fields
                        .iter()
                        .map(|field| {
                            Ok(CanonicalStructField {
                                name: field.name.as_ref().map(|value| value.to_string()),
                                ty: canonical_type_inner(registry, field.ty.id, visited)?,
                            })
                        })
                        .collect::<CanonicalTypeResult<Vec<_>>>()?,
                }
            } else {
                Tuple {
                    items: def
                        .fields
                        .iter()
                        .map(|field| canonical_type_inner(registry, field.ty.id, visited))
                        .collect::<CanonicalTypeResult<Vec<_>>>()?,
                }
            }
        }
        TypeDef::Tuple(def) => {
            if def.fields.is_empty() {
                CanonicalType::Unit
            } else {
                Tuple {
                    items: def
                        .fields
                        .iter()
                        .map(|field| canonical_type_inner(registry, field.id, visited))
                        .collect::<CanonicalTypeResult<Vec<_>>>()?,
                }
            }
        }
        TypeDef::Variant(def) => Enum {
            variants: def
                .variants
                .iter()
                .map(|variant| canonical_enum_variant(registry, variant, visited))
                .collect::<CanonicalTypeResult<Vec<_>>>()?,
        },
        TypeDef::Sequence(def) => Vector {
            item: Box::new(canonical_type_inner(registry, def.type_param.id, visited)?),
        },
        TypeDef::Array(def) => Array {
            item: Box::new(canonical_type_inner(registry, def.type_param.id, visited)?),
            len: def.len,
        },
        TypeDef::Primitive(primitive) => Primitive {
            name: primitive_name(primitive),
        },
        TypeDef::Compact(compact) => {
            canonical_type_inner(registry, compact.type_param.id, visited)?
        }
        TypeDef::BitSequence(_) => named_type(registry, type_id),
    };

    visited.remove(&type_id);
    Ok(result)
}

fn canonical_enum_variant(
    registry: &PortableRegistry,
    variant: &Variant<PortableForm>,
    visited: &mut BTreeSet<u32>,
) -> CanonicalTypeResult<CanonicalEnumVariant> {
    let payload = match variant.fields.len() {
        0 => None,
        1 => Some(canonical_type_inner(
            registry,
            variant.fields[0].ty.id,
            visited,
        )?),
        _ if variant.fields.iter().all(|f| f.name.is_some()) => Some(CanonicalType::Struct {
            fields: variant
                .fields
                .iter()
                .map(|field| {
                    Ok(CanonicalStructField {
                        name: field.name.as_ref().map(|value| value.to_string()),
                        ty: canonical_type_inner(registry, field.ty.id, visited)?,
                    })
                })
                .collect::<CanonicalTypeResult<Vec<_>>>()?,
        }),
        _ => Some(CanonicalType::Tuple {
            items: variant
                .fields
                .iter()
                .map(|field| canonical_type_inner(registry, field.ty.id, visited))
                .collect::<CanonicalTypeResult<Vec<_>>>()?,
        }),
    };

    Ok(CanonicalEnumVariant {
        name: variant.name.to_string(),
        payload,
    })
}

/// Resolves a type by its registry path, falling back to an autogenerated identifier.
pub fn named_type(registry: &PortableRegistry, type_id: u32) -> CanonicalType {
    let ty = registry
        .resolve(type_id)
        .expect("type id should exist while building canonical document");
    let name = ty.path.segments.join("::");
    if name.is_empty() {
        CanonicalType::Named {
            name: format!("type_{}", type_id),
        }
    } else {
        CanonicalType::Named { name }
    }
}

/// Returns the canonical name for a primitive scale-info definition.
pub fn primitive_name(primitive: &TypeDefPrimitive) -> String {
    match primitive {
        TypeDefPrimitive::Bool => "bool",
        TypeDefPrimitive::Char => "char",
        TypeDefPrimitive::Str => "str",
        TypeDefPrimitive::U8 => "u8",
        TypeDefPrimitive::U16 => "u16",
        TypeDefPrimitive::U32 => "u32",
        TypeDefPrimitive::U64 => "u64",
        TypeDefPrimitive::U128 => "u128",
        TypeDefPrimitive::U256 => "u256",
        TypeDefPrimitive::I8 => "i8",
        TypeDefPrimitive::I16 => "i16",
        TypeDefPrimitive::I32 => "i32",
        TypeDefPrimitive::I64 => "i64",
        TypeDefPrimitive::I128 => "i128",
        TypeDefPrimitive::I256 => "i256",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_scale_codec::Compact;
    use scale_info::{
        Field, MetaType, Path, PortableRegistryBuilder, Registry, Type, TypeDefBitSequence,
        TypeDefComposite, TypeDefPrimitive, form::PortableForm,
    };

    #[allow(dead_code)]
    #[derive(scale_info::TypeInfo)]
    enum TupleEnum {
        Unit,
        Tuple(u32, i64),
    }

    #[allow(dead_code)]
    #[derive(scale_info::TypeInfo)]
    struct CompactWrapper {
        value: Compact<u32>,
    }

    #[allow(dead_code)]
    #[derive(scale_info::TypeInfo)]
    struct RecursiveNode {
        value: u32,
        next: Option<Box<RecursiveNode>>,
    }

    #[allow(dead_code)]
    #[derive(scale_info::TypeInfo)]
    struct NestedVecArray {
        values: Vec<[u32; 4]>,
    }

    fn portable_registry_for<T: scale_info::TypeInfo + 'static>() -> (PortableRegistry, u32) {
        let mut registry = Registry::new();
        let id = registry.register_type(&MetaType::new::<T>()).id;
        (PortableRegistry::from(registry), id)
    }

    #[test]
    fn enum_with_unnamed_fields_is_tuple_payload() {
        let (registry, id) = portable_registry_for::<TupleEnum>();
        let ty = canonical_type(&registry, id).unwrap();
        match ty {
            CanonicalType::Enum { variants } => {
                let tuple_variant = variants
                    .into_iter()
                    .find(|variant| variant.name == "Tuple")
                    .expect("Tuple variant is present");
                match tuple_variant.payload.expect("payload exists for Tuple") {
                    CanonicalType::Tuple { items } => {
                        assert_eq!(items.len(), 2);
                    }
                    other => panic!("expected tuple payload, got {other:?}"),
                }
            }
            other => panic!("expected enum, got {other:?}"),
        }
    }

    #[test]
    fn compact_type_flattens_to_inner() {
        let (registry, id) = portable_registry_for::<CompactWrapper>();
        let ty = canonical_type(&registry, id).unwrap();
        match ty {
            CanonicalType::Struct { fields } => {
                assert_eq!(fields.len(), 1);
                match &fields[0].ty {
                    CanonicalType::Primitive { name } => assert_eq!(name, "u32"),
                    other => panic!("expected primitive payload, got {other:?}"),
                }
            }
            other => panic!("expected struct, got {other:?}"),
        }
    }

    #[test]
    fn self_referential_struct_uses_named_cycle_break() {
        let (registry, id) = portable_registry_for::<RecursiveNode>();
        let ty = canonical_type(&registry, id).unwrap();
        match ty {
            CanonicalType::Struct { fields } => {
                assert_eq!(fields.len(), 2);
                let next_field = fields
                    .into_iter()
                    .find(|field| field.name.as_deref() == Some("next"))
                    .expect("next field exists");
                // Recursive traversal should not overflow the stack.
                assert!(matches!(
                    next_field.ty,
                    CanonicalType::Enum { .. } | CanonicalType::Named { .. }
                ));
            }
            other => panic!("expected struct, got {other:?}"),
        }
    }

    #[test]
    fn bitsequence_resolves_to_named_type() {
        let mut builder = PortableRegistryBuilder::new();

        let store_id = builder.register_type(Type::new(
            Path::<PortableForm>::from_segments_unchecked(vec!["Store".into()]),
            vec![],
            TypeDefPrimitive::U32,
            vec![],
        ));
        let order_id = builder.register_type(Type::new(
            Path::<PortableForm>::from_segments_unchecked(vec!["Order".into()]),
            vec![],
            TypeDefPrimitive::U8,
            vec![],
        ));
        let bitseq_id = builder.register_type(Type::new(
            Path::<PortableForm>::from_segments_unchecked(vec!["test".into(), "BitSeq".into()]),
            vec![],
            TypeDefBitSequence::new_portable(store_id.into(), order_id.into()),
            vec![],
        ));
        let wrapper_id = builder.register_type(Type::new(
            Path::<PortableForm>::from_segments_unchecked(vec![
                "tests".into(),
                "BitSequenceWrapper".into(),
            ]),
            vec![],
            TypeDefComposite::new(vec![Field::<PortableForm>::new(
                Some("bits".into()),
                bitseq_id.into(),
                None,
                vec![],
            )]),
            vec![],
        ));

        let registry = builder.finish();
        let ty = canonical_type(&registry, wrapper_id).unwrap();
        let CanonicalType::Struct { fields } = ty else {
            panic!("expected struct, got {ty:?}");
        };
        assert_eq!(fields.len(), 1);
        match &fields[0].ty {
            CanonicalType::Named { name } => {
                assert!(
                    name.contains("BitSeq"),
                    "expected BitSeq in name, got {name}"
                );
            }
            other => panic!("expected named type, got {other:?}"),
        }
    }

    #[test]
    fn nested_vectors_and_arrays_resolve_recursively() {
        let (registry, id) = portable_registry_for::<NestedVecArray>();
        let ty = canonical_type(&registry, id).unwrap();
        match ty {
            CanonicalType::Struct { fields } => {
                assert_eq!(fields.len(), 1);
                match &fields[0].ty {
                    CanonicalType::Vector { item } => match **item {
                        CanonicalType::Array { len, ref item } => {
                            assert_eq!(len, 4);
                            assert!(matches!(**item, CanonicalType::Primitive { .. }));
                        }
                        ref other => panic!("expected array field, got {other:?}"),
                    },
                    other => panic!("expected vector field, got {other:?}"),
                }
            }
            other => panic!("expected struct, got {other:?}"),
        }
    }
}
