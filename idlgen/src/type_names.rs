// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Type names resolution.

use crate::errors::{Error, Result};
use convert_case::{Case, Casing};
use scale_info::{
    form::PortableForm, PortableRegistry, Type, TypeDef, TypeDefArray, TypeDefPrimitive,
    TypeDefSequence, TypeDefTuple, TypeInfo,
};
use std::collections::BTreeMap;

pub(super) fn resolve_type_names(
    type_registry: &PortableRegistry,
) -> Result<BTreeMap<u32, String>> {
    type_registry.types.iter().try_fold(
        BTreeMap::<u32, String>::new(),
        |mut resolved_type_names, ty| {
            resolve_type_name(type_registry, ty.id, &mut resolved_type_names)
                .map(|_| resolved_type_names)
        },
    )
}

fn resolve_type_name(
    type_registry: &PortableRegistry,
    type_id: u32,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    if let Some(type_name) = resolved_type_names.get(&type_id) {
        return Ok(type_name.clone());
    }

    let type_info = type_registry
        .resolve(type_id)
        .ok_or_else(|| Error::UnknownType(type_id))?;

    let type_name = match &type_info.type_def {
        TypeDef::Primitive(primitive_def) => primitive_type_name(primitive_def)?,
        TypeDef::Tuple(tuple_def) => {
            tuple_type_name(type_registry, tuple_def, resolved_type_names)?
        }
        TypeDef::Sequence(vector_def) => {
            vector_type_name(type_registry, vector_def, resolved_type_names)?
        }
        TypeDef::Array(array_def) => {
            array_type_name(type_registry, array_def, resolved_type_names)?
        }
        TypeDef::Composite(_) => {
            let btree_map_type_info = BTreeMap::<u32, ()>::type_info();
            if btree_map_type_info.path.segments == type_info.path.segments {
                btree_map_type_name(type_registry, type_info, resolved_type_names)?
            } else {
                type_name_by_path(type_registry, type_info, resolved_type_names)?
            }
        }
        TypeDef::Variant(_) => {
            let result_type_info = std::result::Result::<(), ()>::type_info();
            let option_type_info = std::option::Option::<()>::type_info();
            if result_type_info.path.segments == type_info.path.segments {
                result_type_name(type_registry, type_info, resolved_type_names)?
            } else if option_type_info.path.segments == type_info.path.segments {
                option_type_name(type_registry, type_info, resolved_type_names)?
            } else {
                type_name_by_path(type_registry, type_info, resolved_type_names)?
            }
        }
        _ => {
            return Err(Error::UnsupprotedType(format!("{type_info:?}")));
        }
    };

    resolved_type_names.insert(type_id, type_name.clone());
    Ok(type_name)
}

fn btree_map_type_name(
    type_registry: &PortableRegistry,
    type_info: &Type<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let key_type_id = type_info
        .type_params
        .iter()
        .find(|param| param.name == "K")
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
        .ty
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?;
    let value_type_id = type_info
        .type_params
        .iter()
        .find(|param| param.name == "V")
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
        .ty
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?;
    let key_type_name = resolve_type_name(type_registry, key_type_id.id, resolved_type_names)?;
    let value_type_name = resolve_type_name(type_registry, value_type_id.id, resolved_type_names)?;
    Ok(format!("map ({}, {})", key_type_name, value_type_name))
}

fn result_type_name(
    type_registry: &PortableRegistry,
    type_info: &Type<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let ok_type_id = type_info
        .type_params
        .iter()
        .find(|param| param.name == "T")
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
        .ty
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?;
    let err_type_id = type_info
        .type_params
        .iter()
        .find(|param| param.name == "E")
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
        .ty
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?;
    let ok_type_name = resolve_type_name(type_registry, ok_type_id.id, resolved_type_names)?;
    let err_type_name = resolve_type_name(type_registry, err_type_id.id, resolved_type_names)?;
    Ok(format!("result ({ok_type_name}, {err_type_name})"))
}

fn option_type_name(
    type_registry: &PortableRegistry,
    type_info: &Type<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let some_type_id = type_info
        .type_params
        .iter()
        .find(|param| param.name == "T")
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
        .ty
        .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?;
    let some_type_name = resolve_type_name(type_registry, some_type_id.id, resolved_type_names)?;
    Ok(format!("opt {}", some_type_name))
}

fn type_name_by_path(
    type_registry: &PortableRegistry,
    type_info: &Type<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let type_name = type_info
        .path
        .segments
        .iter()
        .map(|segment| segment.to_case(Case::Pascal))
        .collect::<Vec<_>>()
        .join("");
    let type_param_names = type_info
        .type_params
        .iter()
        .map(|type_param| {
            let type_param_id = type_param
                .ty
                .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
                .id;
            resolve_type_name(type_registry, type_param_id, resolved_type_names)
                .map(|type_name| type_name.to_case(Case::Pascal))
        })
        .collect::<Result<Vec<_>>>()?
        .join("And");
    let type_name = if type_param_names.is_empty() {
        type_name
    } else {
        format!("{}For{}", type_name, type_param_names)
    };
    if type_name.is_empty() {
        Err(Error::UnsupprotedType(format!("{type_info:?}")))
    } else {
        Ok(type_name)
    }
}

fn tuple_type_name(
    type_registry: &PortableRegistry,
    tuple_def: &TypeDefTuple<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let fields = tuple_def
        .fields
        .iter()
        .map(|field| resolve_type_name(type_registry, field.id, resolved_type_names))
        .collect::<Result<Vec<_>>>()?
        .join(", ");
    if fields.is_empty() {
        Ok("null".into()) // For the () type
    } else {
        Ok(format!("struct {{ {} }}", fields))
    }
}

fn vector_type_name(
    type_registry: &PortableRegistry,
    vector_def: &TypeDefSequence<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let item_type_name =
        resolve_type_name(type_registry, vector_def.type_param.id, resolved_type_names)?;
    Ok(format!("vec {}", item_type_name))
}

fn array_type_name(
    type_registry: &PortableRegistry,
    array_def: &TypeDefArray<PortableForm>,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let item_type_name =
        resolve_type_name(type_registry, array_def.type_param.id, resolved_type_names)?;
    Ok(format!("[{}, {}]", item_type_name, array_def.len))
}

fn primitive_type_name(type_def: &TypeDefPrimitive) -> Result<String> {
    match type_def {
        TypeDefPrimitive::Bool => Ok("bool".into()),
        TypeDefPrimitive::Char => Ok("char".into()),
        TypeDefPrimitive::Str => Ok("str".into()),
        TypeDefPrimitive::U8 => Ok("u8".into()),
        TypeDefPrimitive::U16 => Ok("u16".into()),
        TypeDefPrimitive::U32 => Ok("u32".into()),
        TypeDefPrimitive::U64 => Ok("u64".into()),
        TypeDefPrimitive::U128 => Ok("u128".into()),
        TypeDefPrimitive::U256 => Err(Error::UnsupprotedType("u256".into())), // Rust doesn't have it
        TypeDefPrimitive::I8 => Ok("i8".into()),
        TypeDefPrimitive::I16 => Ok("i16".into()),
        TypeDefPrimitive::I32 => Ok("i32".into()),
        TypeDefPrimitive::I64 => Ok("i64".into()),
        TypeDefPrimitive::I128 => Ok("i128".into()),
        TypeDefPrimitive::I256 => Err(Error::UnsupprotedType("i256".into())), // Rust doesn't have it
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::{MetaType, Registry};

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericStruct<T> {
        field: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
    }

    #[test]
    fn generic_struct_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<u32>>())
            .id;
        let string_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<String>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let type_names = resolve_type_names(&portable_registry).unwrap();

        let u32_struct_name = type_names.get(&u32_struct_id).unwrap();
        assert_eq!(
            u32_struct_name,
            "SailsIdlgenTypeNamesTestsGenericStructForU32"
        );

        let string_struct_name = type_names.get(&string_struct_id).unwrap();
        assert_eq!(
            string_struct_name,
            "SailsIdlgenTypeNamesTestsGenericStructForStr"
        );
    }

    #[test]
    fn generic_variant_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_string_enum_id = registry
            .register_type(&MetaType::new::<GenericEnum<u32, String>>())
            .id;
        let bool_u32_enum_id = registry
            .register_type(&MetaType::new::<GenericEnum<bool, u32>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let type_names = resolve_type_names(&portable_registry).unwrap();

        let u32_string_enum_name = type_names.get(&u32_string_enum_id).unwrap();
        assert_eq!(
            u32_string_enum_name,
            "SailsIdlgenTypeNamesTestsGenericEnumForU32AndStr"
        );

        let bool_u32_enum_name = type_names.get(&bool_u32_enum_id).unwrap();
        assert_eq!(
            bool_u32_enum_name,
            "SailsIdlgenTypeNamesTestsGenericEnumForBoolAndU32"
        );
    }

    #[test]
    fn array_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_array_id = registry.register_type(&MetaType::new::<[u32; 10]>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let type_names = resolve_type_names(&portable_registry).unwrap();

        let u32_array_name = type_names.get(&u32_array_id).unwrap();
        assert_eq!(u32_array_name, "[u32, 10]");
    }

    #[test]
    fn btree_map_name_resolution_works() {
        let mut registry = Registry::new();
        let btree_map_id = registry
            .register_type(&MetaType::new::<BTreeMap<u32, String>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let type_names = resolve_type_names(&portable_registry).unwrap();

        let btree_map_name = type_names.get(&btree_map_id).unwrap();
        assert_eq!(btree_map_name, "map (u32, str)");
    }
}
