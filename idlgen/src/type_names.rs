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
    form::PortableForm, PortableRegistry, Type, TypeDef, TypeDefPrimitive, TypeDefTuple, TypeInfo,
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
        TypeDef::Sequence(sequence_def) => array_type_name(
            type_registry,
            sequence_def.type_param.id,
            resolved_type_names,
        )?,
        TypeDef::Composite(_) => type_name_by_path(type_info)?,
        TypeDef::Variant(_) => {
            let result_type_info = std::result::Result::<(), ()>::type_info();
            let option_type_info = std::option::Option::<()>::type_info();
            if result_type_info.path.segments == type_info.path.segments {
                result_type_name(type_registry, type_info, resolved_type_names)?
            } else if option_type_info.path.segments == type_info.path.segments {
                option_type_name(type_registry, type_info, resolved_type_names)?
            } else {
                type_name_by_path(type_info)?
            }
        }
        _ => {
            return Err(Error::UnsupprotedType(format!("{type_info:?}")));
        }
    };

    resolved_type_names.insert(type_id, type_name.clone());
    Ok(type_name)
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
    Ok(format!("({ok_type_name}, {err_type_name})"))
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

fn type_name_by_path(type_info: &Type<PortableForm>) -> Result<String> {
    let type_name = type_info
        .path
        .segments
        .iter()
        .map(|segment| segment.to_case(Case::Pascal))
        .collect::<Vec<_>>()
        .join("");
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
        .join("; ");
    if fields.is_empty() {
        Ok("null".into()) // For the () type
    } else {
        Ok(format!("record {{ {} }}", fields))
    }
}

fn array_type_name(
    type_registry: &PortableRegistry,
    item_type_id: u32,
    resolved_type_names: &mut BTreeMap<u32, String>,
) -> Result<String> {
    let item_type_name = resolve_type_name(type_registry, item_type_id, resolved_type_names)?;
    Ok(format!("vec {}", item_type_name))
}

fn primitive_type_name(type_def: &TypeDefPrimitive) -> Result<String> {
    match type_def {
        TypeDefPrimitive::Bool => Ok("bool".into()),
        TypeDefPrimitive::Char => Ok("char".into()), // Candid doesn't have it. Do we want to support it? If such it will require a definition
        TypeDefPrimitive::Str => Ok("text".into()),
        TypeDefPrimitive::U8 => Ok("nat8".into()),
        TypeDefPrimitive::U16 => Ok("nat16".into()),
        TypeDefPrimitive::U32 => Ok("nat32".into()),
        TypeDefPrimitive::U64 => Ok("nat64".into()),
        TypeDefPrimitive::U128 => Ok("nat128".into()), // Candid doesn't have it
        TypeDefPrimitive::U256 => Err(Error::UnsupprotedType("u256".into())), // Rust doesn't have it
        TypeDefPrimitive::I8 => Ok("int8".into()),
        TypeDefPrimitive::I16 => Ok("int16".into()),
        TypeDefPrimitive::I32 => Ok("int32".into()),
        TypeDefPrimitive::I64 => Ok("int64".into()),
        TypeDefPrimitive::I128 => Ok("int128".into()), // Candid doesn't have it
        TypeDefPrimitive::I256 => Err(Error::UnsupprotedType("i256".into())), // Rust doesn't have it
    }
}
