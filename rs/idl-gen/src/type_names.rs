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

use crate::{
    TypeNameResolutionError,
    errors::{Error, Result},
};
use convert_case::{Case, Casing};
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};
use gprimitives::*;
use scale_info::{
    form::PortableForm, Field, PortableType, Type, TypeDef, TypeDefArray, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeInfo
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
    result::Result as StdResult,
    sync::OnceLock,
};

pub(super) fn resolve<'a>(
    types: impl Iterator<Item = &'a PortableType>,
) -> Result<(BTreeMap<u32, String>, BTreeMap<TypeRegistryId, String>)> {
    let types = types
        .map(|t| (t.id, t))
        .collect::<BTreeMap<u32, &PortableType>>();
    let type_names = types.iter().try_fold(
        (
            BTreeMap::<u32, RcTypeName>::new(),
            HashMap::<(String, Vec<u32>), u32>::new(),
        ),
        |mut type_names, ty| {
            resolve_type_name(&types, *ty.0, &mut type_names.0, &mut type_names.1)
                .map(|_| type_names)
        },
    );

    type_names
        .map(|type_names| {
            type_names
                .0
                .iter()
                .map(|(id, name)| (*id, name.as_string(false, &type_names.1)))
                .collect()
        })
        .and_then(|concrete_names| {
            build_generic_names(&types, &concrete_names)
                .map(|generic_names| (concrete_names, generic_names))
        })
}

fn resolve_type_name(
    types: &BTreeMap<u32, &PortableType>,
    type_id: u32,
    resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
    by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
) -> Result<RcTypeName> {
    if let Some(type_name) = resolved_type_names.get(&type_id) {
        return Ok(type_name.clone());
    }

    let type_info = types
        .get(&type_id)
        .map(|t| &t.ty)
        .ok_or_else(|| Error::TypeIdIsUnknown(type_id))?;

    let type_name: RcTypeName = match &type_info.type_def {
        TypeDef::Tuple(tuple_def) => Rc::new(TupleTypeName::new(
            types,
            tuple_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Sequence(vector_def) => Rc::new(VectorTypeName::new(
            types,
            vector_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Array(array_def) => Rc::new(ArrayTypeName::new(
            types,
            array_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Composite(_) => {
            if BTreeMapTypeName::is_btree_map_type(type_info) {
                Rc::new(BTreeMapTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else if actor_id::TypeNameImpl::is_type(type_info) {
                Rc::new(actor_id::TypeNameImpl::new())
            } else if message_id::TypeNameImpl::is_type(type_info) {
                Rc::new(message_id::TypeNameImpl::new())
            } else if code_id::TypeNameImpl::is_type(type_info) {
                Rc::new(code_id::TypeNameImpl::new())
            } else if h160::TypeNameImpl::is_type(type_info) {
                Rc::new(h160::TypeNameImpl::new())
            } else if h256::TypeNameImpl::is_type(type_info) {
                Rc::new(h256::TypeNameImpl::new())
            } else if u256::TypeNameImpl::is_type(type_info) {
                Rc::new(u256::TypeNameImpl::new())
            } else if nat8::TypeNameImpl::is_type(type_info) {
                Rc::new(nat8::TypeNameImpl::new())
            } else if nat16::TypeNameImpl::is_type(type_info) {
                Rc::new(nat16::TypeNameImpl::new())
            } else if nat32::TypeNameImpl::is_type(type_info) {
                Rc::new(nat32::TypeNameImpl::new())
            } else if nat64::TypeNameImpl::is_type(type_info) {
                Rc::new(nat64::TypeNameImpl::new())
            } else if nat128::TypeNameImpl::is_type(type_info) {
                Rc::new(nat128::TypeNameImpl::new())
            } else if nat256::TypeNameImpl::is_type(type_info) {
                Rc::new(nat256::TypeNameImpl::new())
            } else {
                Rc::new(ByPathTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            }
        }
        TypeDef::Variant(_) => {
            if ResultTypeName::is_result_type(type_info) {
                Rc::new(ResultTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else if OptionTypeName::is_option_type(type_info) {
                Rc::new(OptionTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else {
                Rc::new(ByPathTypeName::new(
                    types,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            }
        }
        TypeDef::Primitive(primitive_def) => Rc::new(PrimitiveTypeName::new(primitive_def)?),
        _ => {
            return Err(Error::TypeIsUnsupported(format!("{type_info:?}")));
        }
    };

    resolved_type_names.insert(type_id, type_name.clone());
    Ok(type_name)
}

type RcTypeName = Rc<dyn TypeName>;

trait TypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String; // Make returning &str + use OnceCell to cache the result
}

/// By path type name resolution.
struct ByPathTypeName {
    possible_names: Vec<(String, Vec<u32>)>,
    type_param_type_names: Vec<RcTypeName>,
}

impl ByPathTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let type_params = type_info.type_params.iter().try_fold(
            (
                Vec::with_capacity(type_info.type_params.len()),
                Vec::with_capacity(type_info.type_params.len()),
            ),
            |(mut type_param_ids, mut type_param_type_names), type_param| {
                let type_param_id = type_param
                    .ty
                    .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
                    .id;
                let type_param_type_name = resolve_type_name(
                    types,
                    type_param_id,
                    resolved_type_names,
                    by_path_type_names,
                )?;
                type_param_ids.push(type_param_id);
                type_param_type_names.push(type_param_type_name);
                Ok::<(Vec<u32>, Vec<Rc<dyn TypeName>>), Error>((
                    type_param_ids,
                    type_param_type_names,
                ))
            },
        )?;

        let mut possible_names = Self::possible_names_by_path(type_info).fold(
            Vec::with_capacity(type_info.path.segments.len() + 1),
            |mut possible_names, name| {
                let possible_name = (name.clone(), type_params.0.clone());
                possible_names.push(possible_name.clone());
                let name_ref_count = by_path_type_names
                    .entry((name.clone(), type_params.0.clone()))
                    .or_default();
                *name_ref_count += 1;
                possible_names
            },
        );
        if let Some(first_name) = possible_names.first() {
            // add numbered type name like `TypeName1`, `TypeName2` as last name
            // to solve name conflict with const generic parameters `<const N: size>`
            let name_ref_count = by_path_type_names.get(first_name).unwrap_or(&0);
            let name = format!("{}{}", first_name.0, name_ref_count);
            let possible_name = (name.clone(), first_name.1.clone());
            possible_names.push(possible_name);
            let name_ref_count = by_path_type_names
                .entry((name.clone(), type_params.0.clone()))
                .or_default();
            *name_ref_count += 1;
        } else {
            return Err(Error::TypeIsUnsupported(format!("{type_info:?}")));
        }

        Ok(Self {
            possible_names,
            type_param_type_names: type_params.1,
        })
    }

    fn possible_names_by_path(type_info: &Type<PortableForm>) -> impl Iterator<Item = String> + '_ {
        let mut name = String::default();
        type_info.path.segments.iter().rev().map(move |segment| {
            name = segment.to_case(Case::Pascal) + &name;
            name.clone()
        })
    }
}

impl TypeName for ByPathTypeName {
    fn as_string(
        &self,
        _for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let name = self
            .possible_names
            .iter()
            .find(|possible_name| {
                by_path_type_names
                    .get(possible_name)
                    .is_some_and(|ref_count| *ref_count == 1)
            })
            .unwrap_or_else(|| self.possible_names.last().unwrap());
        if self.type_param_type_names.is_empty() {
            name.0.clone()
        } else {
            let type_param_names = self
                .type_param_type_names
                .iter()
                .map(|tn| tn.as_string(true, by_path_type_names))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name.0, type_param_names)
        }
    }
}

/// BTreeMap type name resolution.
struct BTreeMapTypeName {
    key_type_name: RcTypeName,
    value_type_name: RcTypeName,
}

impl BTreeMapTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let key_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "K")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let value_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "V")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let key_type_name = resolve_type_name(
            types,
            key_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        let value_type_name = resolve_type_name(
            types,
            value_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self {
            key_type_name,
            value_type_name,
        })
    }

    pub fn is_btree_map_type(type_info: &Type<PortableForm>) -> bool {
        static BTREE_MAP_TYPE_INFO: OnceLock<Type> = OnceLock::new();
        let btree_map_type_info = BTREE_MAP_TYPE_INFO.get_or_init(BTreeMap::<u32, ()>::type_info);
        btree_map_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for BTreeMapTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let key_type_name = self
            .key_type_name
            .as_string(for_generic_param, by_path_type_names);
        let value_type_name = self
            .value_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("SailsBTreeMap<{key_type_name}, {value_type_name}>")
    }
}

/// Result type name resolution.
pub(crate) struct ResultTypeName {
    ok_type_name: RcTypeName,
    err_type_name: RcTypeName,
}

impl ResultTypeName {
    fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let ok_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "T")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let err_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "E")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let ok_type_name = resolve_type_name(
            types,
            ok_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        let err_type_name = resolve_type_name(
            types,
            err_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self {
            ok_type_name,
            err_type_name,
        })
    }

    pub fn is_result_type(type_info: &Type<PortableForm>) -> bool {
        static RESULT_TYPE_INFO: OnceLock<Type> = OnceLock::new();
        let result_type_info = RESULT_TYPE_INFO.get_or_init(StdResult::<(), ()>::type_info);
        result_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for ResultTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let ok_type_name = self
            .ok_type_name
            .as_string(for_generic_param, by_path_type_names);
        let err_type_name = self
            .err_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("Result<{ok_type_name}, {err_type_name}>")
    }
}

/// Option type name resolution.
struct OptionTypeName {
    some_type_name: RcTypeName,
}

impl OptionTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let some_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "T")
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::TypeIsUnsupported(format!("{type_info:?}")))?;
        let some_type_name = resolve_type_name(
            types,
            some_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self { some_type_name })
    }

    pub fn is_option_type(type_info: &Type<PortableForm>) -> bool {
        static OPTION_TYPE_INFO: OnceLock<Type> = OnceLock::new();
        let option_type_info = OPTION_TYPE_INFO.get_or_init(Option::<()>::type_info);
        option_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for OptionTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let some_type_name = self
            .some_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("Option<{some_type_name}>")
    }
}

/// Tuple type name resolution.
struct TupleTypeName {
    field_type_names: Vec<RcTypeName>,
}

impl TupleTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        tuple_def: &TypeDefTuple<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let field_type_names = tuple_def
            .fields
            .iter()
            .map(|field| {
                resolve_type_name(types, field.id, resolved_type_names, by_path_type_names)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { field_type_names })
    }
}

impl TypeName for TupleTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        format!(
            "({})",
            self.field_type_names
                .iter()
                .map(|tn| tn.as_string(for_generic_param, by_path_type_names))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Vector type name resolution.
struct VectorTypeName {
    item_type_name: RcTypeName,
}

impl VectorTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        vector_def: &TypeDefSequence<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let item_type_name = resolve_type_name(
            types,
            vector_def.type_param.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self { item_type_name })
    }
}

impl TypeName for VectorTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let item_type_name = self
            .item_type_name
            .as_string(for_generic_param, by_path_type_names);
        format!("SailsVec<{item_type_name}>")
    }
}

/// Array type name resolution.
struct ArrayTypeName {
    item_type_name: RcTypeName,
    len: u32,
}

impl ArrayTypeName {
    pub fn new(
        types: &BTreeMap<u32, &PortableType>,
        array_def: &TypeDefArray<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let item_type_name = resolve_type_name(
            types,
            array_def.type_param.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self {
            item_type_name,
            len: array_def.len,
        })
    }
}

impl TypeName for ArrayTypeName {
    fn as_string(
        &self,
        for_generic_param: bool,
        by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        let item_type_name = self
            .item_type_name
            .as_string(for_generic_param, by_path_type_names);

        format!("[{item_type_name}; {len}]", len = self.len)
    }
}

/// Primitive type name resolution.
struct PrimitiveTypeName {
    name: &'static str,
}

impl PrimitiveTypeName {
    pub fn new(type_def: &TypeDefPrimitive) -> Result<Self> {
        let name = match type_def {
            TypeDefPrimitive::Bool => Ok("bool"),
            TypeDefPrimitive::Char => Ok("char"),
            TypeDefPrimitive::Str => Ok("String"),
            TypeDefPrimitive::U8 => Ok("u8"),
            TypeDefPrimitive::U16 => Ok("u16"),
            TypeDefPrimitive::U32 => Ok("u32"),
            TypeDefPrimitive::U64 => Ok("u64"),
            TypeDefPrimitive::U128 => Ok("u128"),
            TypeDefPrimitive::U256 => Err(Error::TypeIsUnsupported("u256".into())), // Rust doesn't have it
            TypeDefPrimitive::I8 => Ok("i8"),
            TypeDefPrimitive::I16 => Ok("i16"),
            TypeDefPrimitive::I32 => Ok("i32"),
            TypeDefPrimitive::I64 => Ok("i64"),
            TypeDefPrimitive::I128 => Ok("i128"),
            TypeDefPrimitive::I256 => Err(Error::TypeIsUnsupported("i256".into())), // Rust doesn't have it
        }?;
        Ok(Self { name })
    }
}

impl TypeName for PrimitiveTypeName {
    fn as_string(
        &self,
        _for_generic_param: bool,
        _by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
    ) -> String {
        self.name.to_string()
    }
}

macro_rules! impl_primitive_alias_type_name {
    ($mod_name:ident, $primitive:ident) => {
        impl_primitive_alias_type_name!($mod_name, $primitive, $primitive);
    };

    ($mod_name:ident, $primitive:ident, $alias:ident) => {
        mod $mod_name {
            use super::*;

            pub(super) struct TypeNameImpl;

            impl TypeNameImpl {
                pub fn new() -> Self {
                    Self
                }

                pub fn is_type(type_info: &Type<PortableForm>) -> bool {
                    static TYPE_INFO: OnceLock<Type> = OnceLock::new();
                    let info = TYPE_INFO.get_or_init($primitive::type_info);
                    info.path.segments == type_info.path.segments
                }
            }

            impl TypeName for TypeNameImpl {
                fn as_string(
                    &self,
                    _for_generic_param: bool,
                    _by_path_type_names: &HashMap<(String, Vec<u32>), u32>,
                ) -> String {
                    stringify!($alias).into()
                }
            }
        }
    };
}

impl_primitive_alias_type_name!(actor_id, ActorId);
impl_primitive_alias_type_name!(message_id, MessageId);
impl_primitive_alias_type_name!(code_id, CodeId);
impl_primitive_alias_type_name!(h160, H160);
impl_primitive_alias_type_name!(h256, H256);
impl_primitive_alias_type_name!(u256, U256, u256);
impl_primitive_alias_type_name!(nat8, NonZeroU8);
impl_primitive_alias_type_name!(nat16, NonZeroU16);
impl_primitive_alias_type_name!(nat32, NonZeroU32);
impl_primitive_alias_type_name!(nat64, NonZeroU64);
impl_primitive_alias_type_name!(nat128, NonZeroU128);
impl_primitive_alias_type_name!(nat256, NonZeroU256);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeRegistryId {
    ParentTy(u32),
    StructField {
        parent_ty: u32,
        field_index: usize,
        self_id: u32,
    },
    EnumField {
        parent_ty: u32,
        // Variant index and name
        variant_data: (u8, String),
        field_index: usize,
        self_id: u32,
    },
}

impl TypeRegistryId {
    fn parent(id: u32) -> Self {
        TypeRegistryId::ParentTy(id)
    }

    fn field(&self, field_index: usize, self_id: u32, variant_data: Option<(u8, String)>) -> Self {
        match self {
            TypeRegistryId::ParentTy(parent_ty) => {
                if let Some(variant_data) = variant_data {
                    TypeRegistryId::EnumField {
                        parent_ty: *parent_ty,
                        variant_data,
                        field_index,
                        self_id,
                    }
                } else {
                    TypeRegistryId::StructField {
                        parent_ty: *parent_ty,
                        field_index,
                        self_id,
                    }
                }
            }
            _ => panic!("Cannot get field of a field type"),
        }
    }
}

/// Build generic names for types with generics
///
/// Takes types from the portable registry along with concrete (resolved) types names
/// and produces a map of type names with generic params as they were declared.
///
/// The map is used later to generate `types` section of the IDL, which consists of
/// user defined types.
fn build_generic_names(
    types: &BTreeMap<u32, &PortableType>,
    concrete_names: &BTreeMap<u32, String>,
) -> Result<BTreeMap<TypeRegistryId, Option<String>>> {
    let mut generic_names = BTreeMap::new();

    // Iterate through all types and process their fields
    for (&parent_type_id, parent_type_info) in types.iter() {
        let parent_type_info = &parent_type_info.ty;

        // Skip types without generics
        if parent_type_info.type_params.is_empty() {
            continue;
        }

        // Skip non-composite and non-variant types, as these are only
        // types that can be declared with generics by the user.
        //
        // Also this check allows to skip Vec<T>, as it's `TypeInfo` impl
        // defines it as a sequence, not a composite.
        if !matches!(
            parent_type_info.type_def,
            TypeDef::Composite(_) | TypeDef::Variant(_)
        ) {
            continue;
        }

        // Also skip known generic types, as there are no user-defined generics.
        if ResultTypeName::is_result_type(parent_type_info)
            || OptionTypeName::is_option_type(parent_type_info)
            || BTreeMapTypeName::is_btree_map_type(parent_type_info)
        {
            continue;
        }

        // Take generics names
        let params_names = parent_type_info
            .type_params
            .iter()
            .map(|param| param.name.to_string())
            .collect::<Vec<_>>();

        // First insert main type
        let main_type_name = build_parent_type_name(
            concrete_names
                .get(&parent_type_id)
                .ok_or(Error::TypeIdIsUnknown(parent_type_id))?,
            &params_names,
        )?;

        let parent_registry_id = TypeRegistryId::parent(parent_type_id);
        if generic_names
            .insert(parent_registry_id, Some(main_type_name.clone()))
            .is_some()
        {
            return Err(
                TypeNameResolutionError::MainTypeRepetition(format!("{main_type_name}")).into(),
            );
        }

        // Construct set of param names for easier lookup
        let params_names = params_names.into_iter().collect::<HashSet<_>>();

        // Then insert fields
        match &parent_type_info.type_def {
            TypeDef::Composite(composite) => build_fields_types_names(
                composite.fields.iter(),
                concrete_names,
                &params_names,
                None,
                parent_registry_id,
                &mut generic_names
            )?,
            TypeDef::Variant(variant) => {
                for variant in variant.variants.iter() {
                    let variant_data = Some((variant.index, variant.name));
                    if variant.fields.is_empty() {
                        // Unit variant like `Option::None`
                        // `field_registry_id` is unique, because variant index and names are unique within enum
                        let field_registry_id = parent_registry_id.field(0, 0, variant_data);
                        let field_name = None;

                        if let Some(field_name) = generic_names.insert(field_registry_id, field_name) {
                            return Err(
                                TypeNameResolutionError::FieldTypeRepetition(field_name).into(),
                            );
                        }
                    } else {
                        build_fields_types_names(
                            variant.fields.iter(),
                            concrete_names,
                            &params_names,
                            variant_data,
                            parent_registry_id,
                            &mut generic_names
                        )?;
                    }
                }
            }
            _ => unreachable!("Must not be handled"),
        };
    }

    Ok(generic_names)
}

/// Construct parent type name declaration with generics.
///
/// Simply takes the concrete name and replaces concrete generics with generic param names,
/// by splitting at `<` and joining with provided type param names.
fn build_parent_type_name(concrete_name: &str, type_params: &[String]) -> Result<String> {
    let type_name_without_generics =
        concrete_name
            .split('<')
            .next()
            .ok_or(TypeNameResolutionError::UnexpectedValue(format!(
                "Expected struct/enum type with `<` symbol, got - {concrete_name}"
            )))?;

    Ok(format!(
        "{type_name_without_generics}<{}>",
        type_params.join(", ")
    ))
}

fn build_fields_types_names<'a>(
    fields_iter: impl Iterator<Item = &'a Field<PortableForm>>,
    concrete_names: &BTreeMap<u32, String>,
    params_names: &HashSet<String>,
    variant_data: Option<(u8, String)>,
    parent_registry_id: TypeRegistryId,
    generic_names: &mut BTreeMap<TypeRegistryId, String>,
) -> Result<()> {
    for (field_index, field) in fields_iter.enumerate() {
        let field_type_id = field.ty.id;
        let field_type_name = field.type_name
            .expect("field must have name set");

        let field_registry_id = parent_registry_id.field(field_index, field_type_id, variant_data);
        let field_generic_name = build_field_type_name(
            field_type_id,
            field_type_name,
            concrete_names,
            params_names,
        )?;

        if let Some(field_generic_name) =
            generic_names.insert(field_registry_id, Some(field_generic_name))
        {
            return Err(TypeNameResolutionError::FieldTypeRepetition(
                field_generic_name,
            )
            .into());
        }
    }

    Ok(())
}

fn build_field_type_name(
    field_type_id: u32,
    field_type_name: &str,
    concrete_names: &BTreeMap<u32, String>,
    params_names: &HashSet<String>,
) -> Result<String> {
    let concrete_type_name = concrete_names
        .get(&field_type_id)
        .ok_or(Error::TypeIdIsUnknown(field_type_id))?;

    if field_type_name == concrete_type_name {
        return Ok(field_type_name.to_string());
    }

    // Type names differ either due to monomorphization or type name resolution, or both
    let syn_field_type_name = syn::parse_str::<syn::Type>(&field_type_name).map_err(|e| {
        TypeNameResolutionError::UnexpectedValue(format!(
            "Failed to parse field type name `{field_type_name}`: {e}"
        ))
    })?;
    let syn_concrete_type_name = syn::parse_str::<syn::Type>(concrete_type_name).map_err(|e| {
        TypeNameResolutionError::UnexpectedValue(format!(
            "Failed to parse concrete type name `{concrete_type_name}`: {e}"
        ))
    })?;

    let resolved_type =
        resolve_to_generic::resolve(&syn_concrete_type_name, &syn_field_type_name, params_names)?;

    Ok(fmt_string(resolved_type))
}

fn fmt_string(syn_ty: syn::Type) -> String {
    use quote::ToTokens;

    syn_ty
        .to_token_stream()
        .to_string()
        .replace(" < ", "<")
        .replace(" >", ">")
        .replace(" , ", ", ")
        .replace(" ( ", "(")
        .replace(" ) ", ")")
        .replace(" [ ", "[")
        .replace(" ] ", "]")
        .replace(" ; ", "; ")
}

mod resolve_to_generic {
    use super::*;
    use quote::ToTokens;
    use syn::{
        GenericArgument, PathArguments, Type as SynType, TypeArray, TypeGroup, TypeParen, TypePath,
        TypeReference, TypeTuple, punctuated::Punctuated,
    };

    /// Resolves a concrete type name to a generic one by replacing concrete arguments by generic ones.
    ///
    /// The function is a bit complex, because `initial` contains identifiers of types that were previously
    /// resolved by type names resolution algo above, which takes into account if there are types with same
    /// names and etc.
    ///
    /// The algorithm works the following way (todo [sab])
    pub(super) fn resolve(
        concrete: &SynType,
        initial: &SynType,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        match initial {
            // Check if we have `initial` to be just a generic parameter.
            SynType::Path(TypePath { qself: None, path }) if path.get_ident().is_some() => {
                let ident = path
                    .get_ident()
                    .expect("path type must have identifier")
                    .to_string();

                if generics.contains(&ident) {
                    return Ok(initial.clone());
                }
            }
            _ => {}
        }

        match (concrete, initial) {
            (SynType::Path(cp), SynType::Path(gp)) => resolve_type_path(cp, gp, generics),
            (SynType::Tuple(ct), SynType::Tuple(gt)) => resolve_type_tuple(ct, gt, generics),
            (SynType::Array(ca), SynType::Array(ga)) => resolve_type_array(ca, ga, generics),
            (SynType::Paren(cp), SynType::Paren(gp)) => resolve_type_paren(cp, gp, generics),
            (SynType::Group(cg), SynType::Group(gg)) => resolve_type_group(cg, gg, generics),
            (SynType::Reference(cr), SynType::Reference(gr)) => resolve_type_ref(cr, gr, generics),
            _ => unreachable!(
                "Unexpected type combination in into generic resolution method: concrete {}, initial {}",
                concrete.to_token_stream().to_string(),
                initial.to_token_stream().to_string()
            ),
        }
    }

    fn resolve_type_path(
        concrete: &TypePath,
        initial: &TypePath,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        // The resolution is done base on concrete value, as it's elements are changed to generics.
        let mut ret = concrete.clone();

        let concrete_path = concrete.path.segments.last();
        let initial_path = initial.path.segments.last();
        let Some((concrete_path, initial_path)) = concrete_path.zip(initial_path) else {
            return Err(TypeNameResolutionError::UnexpectedValue(format!(
                "Mismatched type paths during generic resolution: concrete {}, initial {}",
                concrete.to_token_stream().to_string(),
                initial.to_token_stream().to_string()
            ))
            .into());
        };

        if let (
            PathArguments::AngleBracketed(concrete_type_args),
            PathArguments::AngleBracketed(initial_type_args),
        ) = (&concrete_path.arguments, &initial_path.arguments)
        {
            let mut new_args = Punctuated::new();

            let args_iter = concrete_type_args
                .args
                .iter()
                .zip(initial_type_args.args.iter());
            for (concrete_argument, initial_argument) in args_iter {
                // Just take the argument or resolve from concrete to generic.
                let resolved_arg = match (concrete_argument, initial_argument) {
                    (GenericArgument::Type(c), GenericArgument::Type(i)) => {
                        GenericArgument::Type(resolve(c, i, generics)?)
                    }
                    _ => concrete_argument.clone(),
                };

                new_args.push(resolved_arg);
            }

            let last = ret.path.segments.last_mut().expect("checked");
            last.arguments = PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                args: new_args,
                ..concrete_type_args.clone()
            });
        }

        // Else means, that concrete path type doesn't have generic argument, so does not the initial one.
        // In this case, we just take a value of `concrete`.
        // Case when initial has generics and the concrete doesn't is not possible.

        Ok(SynType::Path(ret))
    }

    fn resolve_type_tuple(
        concrete: &TypeTuple,
        initial: &TypeTuple,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        let mut ret = concrete.clone();
        ret.elems = concrete
            .elems
            .iter()
            .zip(&initial.elems)
            .map(|(c, i)| resolve(c, i, generics))
            .collect::<Result<Vec<SynType>>>()?
            .into_iter()
            .collect();

        Ok(SynType::Tuple(ret))
    }

    fn resolve_type_array(
        concrete: &TypeArray,
        initial: &TypeArray,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        let mut ret = concrete.clone();
        ret.elem = Box::new(resolve(&concrete.elem, &initial.elem, generics)?);

        Ok(SynType::Array(ret))
    }

    fn resolve_type_paren(
        concrete: &TypeParen,
        initial: &TypeParen,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        let mut ret = concrete.clone();
        ret.elem = Box::new(resolve(&concrete.elem, &initial.elem, generics)?);

        Ok(SynType::Paren(ret))
    }

    fn resolve_type_group(
        concrete: &TypeGroup,
        initial: &TypeGroup,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        let mut ret = concrete.clone();
        ret.elem = Box::new(resolve(&concrete.elem, &initial.elem, generics)?);

        Ok(SynType::Group(ret))
    }

    fn resolve_type_ref(
        concrete: &TypeReference,
        initial: &TypeReference,
        generics: &HashSet<String>,
    ) -> Result<SynType> {
        let mut ret = concrete.clone();
        ret.elem = Box::new(resolve(&concrete.elem, &initial.elem, generics)?);

        Ok(SynType::Reference(ret))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::{MetaType, PortableRegistry, Registry};
    use std::result;

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericStruct<T> {
        field: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericConstStruct<const N: usize, const M: usize, T> {
        field: [T; N],
        field2: [T; M],
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
    }

    #[allow(dead_code)]
    mod mod_1 {
        use super::*;

        #[derive(TypeInfo)]
        pub struct T1 {}

        pub mod mod_2 {
            use super::*;

            #[derive(TypeInfo)]
            pub struct T2 {}
        }
    }

    #[allow(dead_code)]
    mod mod_2 {
        use super::*;

        #[derive(TypeInfo)]
        pub struct T1 {}

        #[derive(TypeInfo)]
        pub struct T2 {}
    }

    #[test]
    fn h256_u256_type_name_resolution_works() {
        let mut registry = Registry::new();
        let h256_id = registry.register_type(&MetaType::new::<H256>()).id;
        let h256_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<H256>>())
            .id;
        let u256_id = registry.register_type(&MetaType::new::<U256>()).id;
        let u256_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<U256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let h256_name = type_names.get(&h256_id).unwrap();
        assert_eq!(h256_name, "H256");
        let as_generic_param_name = type_names.get(&h256_as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<H256>");
        let u256_name = type_names.get(&u256_id).unwrap();
        assert_eq!(u256_name, "u256");
        let as_generic_param_name = type_names.get(&u256_as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<u256>");
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

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_struct_name = type_names.get(&u32_struct_id).unwrap();
        assert_eq!(u32_struct_name, "GenericStruct<u32>");

        let string_struct_name = type_names.get(&string_struct_id).unwrap();
        assert_eq!(string_struct_name, "GenericStruct<String>");
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

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_string_enum_name = type_names.get(&u32_string_enum_id).unwrap();
        assert_eq!(u32_string_enum_name, "GenericEnum<u32, String>");

        let bool_u32_enum_name = type_names.get(&bool_u32_enum_id).unwrap();
        assert_eq!(bool_u32_enum_name, "GenericEnum<bool, u32>");
    }

    #[test]
    fn array_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_array_id = registry.register_type(&MetaType::new::<[u32; 10]>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<[u32; 10]>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_array_name = type_names.get(&u32_array_id).unwrap();
        assert_eq!(u32_array_name, "[u32; 10]");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<[u32; 10]>");
    }

    #[test]
    // todo [sab] SailsVec to []
    fn vector_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_vector_id = registry.register_type(&MetaType::new::<Vec<u32>>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Vec<u32>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_vector_name = type_names.get(&u32_vector_id).unwrap();
        assert_eq!(u32_vector_name, "SailsVec<u32>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<SailsVec<u32>>");
    }

    #[test]
    fn result_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_result_id = registry
            .register_type(&MetaType::new::<result::Result<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<result::Result<u32, String>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_result_name = type_names.get(&u32_result_id).unwrap();
        assert_eq!(u32_result_name, "Result<u32, String>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<Result<u32, String>>");
    }

    #[test]
    fn option_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_option_id = registry.register_type(&MetaType::new::<Option<u32>>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Option<u32>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_option_name = type_names.get(&u32_option_id).unwrap();
        assert_eq!(u32_option_name, "Option<u32>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<Option<u32>>");
    }

    #[test]
    fn tuple_type_name_resolution_works() {
        let mut registry = Registry::new();
        let u32_str_tuple_id = registry.register_type(&MetaType::new::<(u32, String)>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<(u32, String)>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let u32_str_tuple_name = type_names.get(&u32_str_tuple_id).unwrap();
        assert_eq!(u32_str_tuple_name, "(u32, String)");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<(u32, String)>");
    }

    #[test]
    // todo [sab] SailsBTreeMap to BTreeMap
    fn btree_map_type_name_resolution_works() {
        let mut registry = Registry::new();
        let btree_map_id = registry
            .register_type(&MetaType::new::<BTreeMap<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<BTreeMap<u32, String>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let btree_map_name = type_names.get(&btree_map_id).unwrap();
        assert_eq!(btree_map_name, "SailsBTreeMap<u32, String>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(
            as_generic_param_name,
            "GenericStruct<SailsBTreeMap<u32, String>>"
        );
    }

    #[test]
    fn type_name_minification_works_for_types_with_the_same_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry.register_type(&MetaType::new::<mod_1::T1>()).id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T1>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let t1_name = type_names.get(&t1_id).unwrap();
        assert_eq!(t1_name, "Mod1T1");

        let t2_name = type_names.get(&t2_id).unwrap();
        assert_eq!(t2_name, "Mod2T1");
    }

    #[test]
    fn type_name_minification_works_for_types_with_different_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry
            .register_type(&MetaType::new::<mod_1::mod_2::T2>())
            .id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T2>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        let t1_name = type_names.get(&t1_id).unwrap();
        assert_eq!(t1_name, "Mod1Mod2T2");

        let t2_name = type_names.get(&t2_id).unwrap();
        assert_eq!(t2_name, "TestsMod2T2");
    }

    macro_rules! type_name_resolution_works {
        ($primitive:ident) => {
            let mut registry = Registry::new();
            let id = registry.register_type(&MetaType::new::<$primitive>()).id;
            let as_generic_param_id = registry
                .register_type(&MetaType::new::<GenericStruct<$primitive>>())
                .id;
            let portable_registry = PortableRegistry::from(registry);

            let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

            let name = type_names.get(&id).unwrap();
            assert_eq!(name, stringify!($primitive));
            let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
            assert_eq!(
                as_generic_param_name,
                concat!("GenericStruct<", stringify!($primitive), ">")
            );
        };
    }

    #[test]
    fn actor_id_type_name_resolution_works() {
        type_name_resolution_works!(ActorId);
    }

    #[test]
    fn message_id_type_name_resolution_works() {
        type_name_resolution_works!(MessageId);
    }

    #[test]
    fn code_id_type_name_resolution_works() {
        type_name_resolution_works!(CodeId);
    }

    #[test]
    fn h160_type_name_resolution_works() {
        type_name_resolution_works!(H160);
    }

    #[test]
    fn nonzero_u8_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU8);
    }

    #[test]
    fn nonzero_u16_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU16);
    }

    #[test]
    fn nonzero_u32_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU32);
    }

    #[test]
    fn nonzero_u64_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU64);
    }

    #[test]
    fn nonzero_u128_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU128);
    }

    #[test]
    fn nonzero_u256_type_name_resolution_works() {
        type_name_resolution_works!(NonZeroU256);
    }

    #[test]
    fn generic_const_struct_type_name_resolution_works() {
        let mut registry = Registry::new();
        let n8_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<8, 8, u8>>())
            .id;
        let n8_id_2 = registry
            .register_type(&MetaType::new::<GenericConstStruct<8, 8, u8>>())
            .id;
        let n32_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<32, 8, u8>>())
            .id;
        let n256_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<256, 832, u8>>())
            .id;
        let n32u256_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<32, 8, U256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        let (type_names, _) = resolve(portable_registry.types.iter()).unwrap();

        assert_eq!(n8_id, n8_id_2);
        assert_ne!(n8_id, n32_id);
        assert_ne!(n8_id, n256_id);
        assert_eq!(type_names.get(&n8_id).unwrap(), "GenericConstStruct1<u8>");
        assert_eq!(type_names.get(&n32_id).unwrap(), "GenericConstStruct2<u8>");
        assert_eq!(type_names.get(&n256_id).unwrap(), "GenericConstStruct3<u8>");
        assert_eq!(
            type_names.get(&n32u256_id).unwrap(),
            "GenericConstStruct<u256>"
        );
    }

    #[test]
    fn simple_cases_one_generic() {
        // Define helper types for the test
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct SimpleOneGenericStruct<T> {
            // Category 1: Simple generic usage
            generic_value: T,
            tuple_generic: (T, T),
            option_generic: Option<T>,
            result_generic: result::Result<T, String>,
            btreemap_generic: BTreeMap<String, T>,
            struct_generic: GenericStruct<T>,
            enum_generic: SimpleOneGenericEnum<T>,

            // Category 2: Two-level nested generics
            option_of_option: Option<Option<T>>,
            result_of_option: result::Result<Option<T>, String>,
            btreemap_nested: BTreeMap<Option<T>, GenericStruct<T>>,
            struct_of_option: GenericStruct<Option<T>>,
            enum_of_result: SimpleOneGenericEnum<result::Result<T, String>>,

            // Category 3: Triple-nested generics
            option_triple: Option<Option<Option<T>>>,
            result_triple: result::Result<Option<result::Result<T, String>>, String>,
            btreemap_triple: BTreeMap<Option<GenericStruct<T>>, result::Result<T, String>>,
            struct_triple: GenericStruct<Option<result::Result<T, String>>>,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum SimpleOneGenericEnum<T> {
            // Category 1: Simple generic usage
            NoFields,
            GenericValue(T),
            TupleGeneric(T, T),
            OptionGeneric(Option<T>),
            ResultGeneric(result::Result<T, String>),
            BTreeMapGeneric {
                map: BTreeMap<String, T>,
            },
            StructGeneric {
                inner: GenericStruct<T>,
            },
            NestedEnum(NestedGenericEnum<T>),

            // Category 2: Two-level nested generics
            OptionOfOption(Option<Option<T>>),
            ResultOfOption {
                res: result::Result<Option<T>, String>,
            },
            DoubleNested {
                btree_map_nested: BTreeMap<Option<T>, GenericStruct<T>>,
                struct_nested: GenericStruct<Option<T>>,
            },

            // Category 3: Triple-nested generics
            TrippleNested {
                option_triple: Option<Option<Option<T>>>,
                result_triple: result::Result<Option<result::Result<T, String>>, String>,
            },
            OptionTriple(Option<Option<Option<T>>>),
            ResultTriple {
                res: result::Result<Option<result::Result<T, String>>, String>,
            },
            NoFields2,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum NestedGenericEnum<T> {
            First(T),
            Second(Vec<T>),
        }

        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<SimpleOneGenericStruct<u32>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<SimpleOneGenericEnum<u32>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check main types
        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "SimpleOneGenericStruct<u32>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(struct_id))
                .unwrap(),
            "SimpleOneGenericStruct<T>"
        );
        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "SimpleOneGenericEnum<u32>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(enum_id))
                .unwrap(),
            "SimpleOneGenericEnum<T>"
        );

        // Get the struct to check fields
        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();
        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            // Category 1: Simple generic usage
            let generic_value = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("generic_value"))
                .unwrap();
            assert_eq!(concrete_names.get(&generic_value.ty.id).unwrap(), "u32");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 0,
                self_id: generic_value.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T");

            let tuple_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_generic.ty.id).unwrap(),
                "(u32, u32)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 1,
                self_id: tuple_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "(T, T)");

            let option_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_generic.ty.id).unwrap(),
                "Option<u32>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 2,
                self_id: option_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<T>");

            let result_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("result_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&result_generic.ty.id).unwrap(),
                "Result<u32, String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 3,
                self_id: result_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Result<T, String>");

            let btreemap_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("btreemap_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btreemap_generic.ty.id).unwrap(),
                "SailsBTreeMap<String, u32>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 4,
                self_id: btreemap_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsBTreeMap<String, T>");

            let struct_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("struct_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&struct_generic.ty.id).unwrap(),
                "GenericStruct<u32>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 5,
                self_id: struct_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<T>");

            let enum_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("enum_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&enum_generic.ty.id).unwrap(),
                "SimpleOneGenericEnum<u32>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 6,
                self_id: enum_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SimpleOneGenericEnum<T>");

            // Category 2: Two-level nested
            let option_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_of_option.ty.id).unwrap(),
                "Option<Option<u32>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 7,
                self_id: option_of_option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Option<T>>");

            let result_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("result_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&result_of_option.ty.id).unwrap(),
                "Result<Option<u32>, String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 8,
                self_id: result_of_option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Result<Option<T>, String>");

            let btreemap_nested = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("btreemap_nested"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btreemap_nested.ty.id).unwrap(),
                "SailsBTreeMap<Option<u32>, GenericStruct<u32>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 9,
                self_id: btreemap_nested.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<Option<T>, GenericStruct<T>>"
            );

            let struct_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("struct_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&struct_of_option.ty.id).unwrap(),
                "GenericStruct<Option<u32>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 10,
                self_id: struct_of_option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<Option<T>>");

            let enum_of_result = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("enum_of_result"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&enum_of_result.ty.id).unwrap(),
                "SimpleOneGenericEnum<Result<u32, String>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 11,
                self_id: enum_of_result.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SimpleOneGenericEnum<Result<T, String>>"
            );

            // Category 3: Triple-nested
            let option_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_triple.ty.id).unwrap(),
                "Option<Option<Option<u32>>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 12,
                self_id: option_triple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Option<Option<T>>>");

            let result_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("result_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&result_triple.ty.id).unwrap(),
                "Result<Option<Result<u32, String>>, String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 13,
                self_id: result_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Result<Option<Result<T, String>>, String>"
            );

            let btreemap_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("btreemap_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btreemap_triple.ty.id).unwrap(),
                "SailsBTreeMap<Option<GenericStruct<u32>>, Result<u32, String>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 14,
                self_id: btreemap_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<Option<GenericStruct<T>>, Result<T, String>>"
            );

            let struct_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("struct_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&struct_triple.ty.id).unwrap(),
                "GenericStruct<Option<Result<u32, String>>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 15,
                self_id: struct_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<Option<Result<T, String>>>"
            );
        } else {
            panic!("Expected composite type");
        }

        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_id)
            .unwrap();
        if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
            let no_field_generic_name = generic_names.get(&TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: (0, "NoFields".to_string()),
                self_id: 0,
                field_index: 0,
            });
            assert!(no_field_generic_name.is_none());

            // Category 1: Simple generic usage
            let generic_value = variant
                .variants
                .iter()
                .find(|v| v.name == "GenericValue")
                .unwrap();
            let field = generic_value.fields.iter().next().unwrap();
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "u32");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 1,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T");

            let tuple_generic = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleGeneric")
                .unwrap();
            let tuple_generic_field1 = &tuple_generic.fields[0];
            assert_eq!(
                concrete_names.get(&tuple_generic_field1.ty.id).unwrap(),
                "u32"
            );
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 2,
                field_index: 0,
                self_id: tuple_generic_field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "T");
            let tuple_generic_field2 = &tuple_generic.fields[1];
            assert_eq!(
                concrete_names.get(&tuple_generic_field2.ty.id).unwrap(),
                "u32"
            );
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 2,
                field_index: 1,
                self_id: tuple_generic_field2.ty.id,
            };
            assert_eq!(generic_names.get(&id2).unwrap(), "T");

            let option_generic = variant
                .variants
                .iter()
                .find(|v| v.name == "OptionGeneric")
                .unwrap();
            let field = &option_generic.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "Option<u32>");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 3,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<T>");

            let result_generic = variant
                .variants
                .iter()
                .find(|v| v.name == "ResultGeneric")
                .unwrap();
            let field = &result_generic.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Result<u32, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 4,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Result<T, String>");

            let btreemap_generic = variant
                .variants
                .iter()
                .find(|v| v.name == "BTreeMapGeneric")
                .unwrap();
            let field = btreemap_generic
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("map"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<String, u32>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 5,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsBTreeMap<String, T>");

            let struct_generic = variant
                .variants
                .iter()
                .find(|v| v.name == "StructGeneric")
                .unwrap();
            let field = struct_generic
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("inner"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<u32>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 6,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<T>");

            let nested_enum = variant
                .variants
                .iter()
                .find(|v| v.name == "NestedEnum")
                .unwrap();
            let field = &nested_enum.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "NestedGenericEnum<u32>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "NestedGenericEnum<T>");

            // Category 2: Two-level nested generics
            let option_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "OptionOfOption")
                .unwrap();
            let field = &option_of_option.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Option<Option<u32>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 8,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Option<T>>");

            let result_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "ResultOfOption")
                .unwrap();
            let field = result_of_option
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("res"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Result<Option<u32>, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 9,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Result<Option<T>, String>");

            let double_nested = variant
                .variants
                .iter()
                .find(|v| v.name == "DoubleNested")
                .unwrap();
            let btree_field = double_nested
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("btree_map_nested"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btree_field.ty.id).unwrap(),
                "SailsBTreeMap<Option<u32>, GenericStruct<u32>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 10,
                field_index: 0,
                self_id: btree_field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<Option<T>, GenericStruct<T>>"
            );
            let struct_field = double_nested
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("struct_nested"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&struct_field.ty.id).unwrap(),
                "GenericStruct<Option<u32>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 10,
                field_index: 1,
                self_id: struct_field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<Option<T>>");

            // Category 3: Triple-nested generics
            let tripple_nested = variant
                .variants
                .iter()
                .find(|v| v.name == "TrippleNested")
                .unwrap();
            let option_field = tripple_nested
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_field.ty.id).unwrap(),
                "Option<Option<Option<u32>>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 11,
                field_index: 0,
                self_id: option_field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Option<Option<T>>>");
            let result_field = tripple_nested
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("result_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&result_field.ty.id).unwrap(),
                "Result<Option<Result<u32, String>>, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 11,
                field_index: 1,
                self_id: result_field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Result<Option<Result<T, String>>, String>"
            );

            let option_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "OptionTriple")
                .unwrap();
            let field = &option_triple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Option<Option<Option<u32>>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 12,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Option<Option<T>>>");

            let result_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "ResultTriple")
                .unwrap();
            let field = result_triple
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("res"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Result<Option<Result<u32, String>>, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 13,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Result<Option<Result<T, String>>, String>"
            );

            let no_field_2_generic_name = generic_names.get(&TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: (14, "NoFields2".to_string()),
                self_id: 0,
                field_index: 0,
            });
            assert!(no_field_2_generic_name.is_none());
        } else {
            panic!("Expected variant type");
        }
    }

    #[test]
    fn complex_cases_one_generic() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct ComplexOneGenericStruct<T> {
            // Category 1: Complex types containing generics
            array_of_generic: [T; 10],
            tuple_complex: (T, Vec<T>, [T; 5]),
            array_of_tuple: [(T, T); 3],
            vec_of_array: Vec<[T; 8]>,

            // Category 2: Two-level nested complex types
            array_of_option: [Option<T>; 5],
            tuple_of_result: (result::Result<T, String>, Option<T>),
            vec_of_struct: Vec<GenericStruct<T>>,
            array_of_btreemap: [BTreeMap<String, T>; 2],

            // Category 3: Triple-nested complex types
            array_of_vec_of_option: [Vec<Option<T>>; 4],
            tuple_triple: (Option<Vec<T>>, result::Result<[T; 3], String>),
            vec_of_struct_of_option: Vec<GenericStruct<Option<T>>>,
            array_complex_triple: [BTreeMap<Option<T>, result::Result<T, String>>; 2],
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum ComplexOneGenericEnum<T> {
            // Category 1: Complex types containing generics
            ArrayOfGeneric([T; 10]),
            TupleComplex(T, Vec<T>, [T; 5]),
            ArrayOfTuple([(T, T); 3]),
            VecOfArray {
                vec: Vec<[T; 8]>,
            },

            // Category 2: Two-level nested complex types
            ArrayOfOption([Option<T>; 5]),
            TupleOfResult {
                tuple: (result::Result<T, String>, Option<T>),
            },
            VecOfStruct(Vec<GenericStruct<T>>),
            ArrayOfBTreeMap {
                array: [BTreeMap<String, T>; 2],
            },

            // Category 3: Triple-nested complex types
            ArrayOfVecOfOption([Vec<Option<Vec<T>>>; 4]),
            TupleTriple {
                field1: Option<Option<Vec<T>>>,
                field2: result::Result<Option<[T; 3]>, String>,
            },
            VecOfStructOfOption(Vec<GenericStruct<Option<T>>>),
            ArrayComplexTriple(
                [BTreeMap<BTreeMap<Option<T>, String>, result::Result<T, String>>; 2],
            ),
        }

        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<ComplexOneGenericStruct<bool>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<ComplexOneGenericEnum<bool>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check main types
        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "ComplexOneGenericStruct<bool>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(struct_id))
                .unwrap(),
            "ComplexOneGenericStruct<T>"
        );
        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "ComplexOneGenericEnum<bool>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(enum_id))
                .unwrap(),
            "ComplexOneGenericEnum<T>"
        );

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();

        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            // Category 1: Complex types containing generics
            let array_of_generic = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_generic.ty.id).unwrap(),
                "[bool; 10]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 0,
                self_id: array_of_generic.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T; 10]");

            let tuple_complex = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_complex"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_complex.ty.id).unwrap(),
                "(bool, SailsVec<bool>, [bool; 5])"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 1,
                self_id: tuple_complex.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "(T, SailsVec<T>, [T; 5])");

            let array_of_tuple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_tuple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_tuple.ty.id).unwrap(),
                "[(bool, bool); 3]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 2,
                self_id: array_of_tuple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[(T, T); 3]");

            let vec_of_array = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_of_array"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_of_array.ty.id).unwrap(),
                "SailsVec<[bool; 8]>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 3,
                self_id: vec_of_array.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<[T; 8]>");

            // Category 2: Two-level nested complex types
            let array_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_option.ty.id).unwrap(),
                "[Option<bool>; 5]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 4,
                self_id: array_of_option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[Option<T>; 5]");

            let tuple_of_result = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_of_result"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_of_result.ty.id).unwrap(),
                "(Result<bool, String>, Option<bool>)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 5,
                self_id: tuple_of_result.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "(Result<T, String>, Option<T>)"
            );

            let vec_of_struct = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_of_struct"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_of_struct.ty.id).unwrap(),
                "SailsVec<GenericStruct<bool>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 6,
                self_id: vec_of_struct.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<T>>"
            );

            let array_of_btreemap = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_btreemap"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_btreemap.ty.id).unwrap(),
                "[SailsBTreeMap<String, bool>; 2]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 7,
                self_id: array_of_btreemap.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsBTreeMap<String, T>; 2]"
            );

            // Category 3: Triple-nested complex types
            let array_of_vec_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_vec_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_vec_of_option.ty.id).unwrap(),
                "[SailsVec<Option<bool>>; 4]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 8,
                self_id: array_of_vec_of_option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[SailsVec<Option<T>>; 4]");

            let tuple_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_triple.ty.id).unwrap(),
                "(Option<SailsVec<bool>>, Result<[bool; 3], String>)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 9,
                self_id: tuple_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "(Option<SailsVec<T>>, Result<[T; 3], String>)"
            );

            let vec_of_struct_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_of_struct_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_of_struct_of_option.ty.id).unwrap(),
                "SailsVec<GenericStruct<Option<bool>>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 10,
                self_id: vec_of_struct_of_option.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<Option<T>>>"
            );

            let array_complex_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_complex_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_complex_triple.ty.id).unwrap(),
                "[SailsBTreeMap<Option<bool>, Result<bool, String>>; 2]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 11,
                self_id: array_complex_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsBTreeMap<Option<T>, Result<T, String>>; 2]"
            );
        } else {
            panic!("Expected composite type");
        }

        // Check enum variants
        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_id)
            .unwrap();
        if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
            // Category 1: Complex types containing generics
            let array_of_generic = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfGeneric")
                .unwrap();
            let field = &array_of_generic.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "[bool; 10]");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 0,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T; 10]");

            let tuple_complex = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleComplex")
                .unwrap();
            let field0 = &tuple_complex.fields[0];
            assert_eq!(concrete_names.get(&field0.ty.id).unwrap(), "bool");
            let id0 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 1,
                field_index: 0,
                self_id: field0.ty.id,
            };
            assert_eq!(generic_names.get(&id0).unwrap(), "T");
            let field1 = &tuple_complex.fields[1];
            assert_eq!(concrete_names.get(&field1.ty.id).unwrap(), "SailsVec<bool>");
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 1,
                field_index: 1,
                self_id: field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "SailsVec<T>");
            let field2 = &tuple_complex.fields[2];
            assert_eq!(concrete_names.get(&field2.ty.id).unwrap(), "[bool; 5]");
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 1,
                field_index: 2,
                self_id: field2.ty.id,
            };
            assert_eq!(generic_names.get(&id2).unwrap(), "[T; 5]");

            let array_of_tuple = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfTuple")
                .unwrap();
            let field = &array_of_tuple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[(bool, bool); 3]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 2,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[(T, T); 3]");

            let vec_of_array = variant
                .variants
                .iter()
                .find(|v| v.name == "VecOfArray")
                .unwrap();
            let field = vec_of_array
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<[bool; 8]>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 3,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<[T; 8]>");

            // Category 2: Two-level nested complex types
            let array_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfOption")
                .unwrap();
            let field = &array_of_option.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[Option<bool>; 5]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 4,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[Option<T>; 5]");

            let tuple_of_result = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleOfResult")
                .unwrap();
            let field = tuple_of_result
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "(Result<bool, String>, Option<bool>)"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 5,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "(Result<T, String>, Option<T>)"
            );

            let vec_of_struct = variant
                .variants
                .iter()
                .find(|v| v.name == "VecOfStruct")
                .unwrap();
            let field = &vec_of_struct.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<GenericStruct<bool>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 6,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<T>>"
            );

            let array_of_btreemap = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfBTreeMap")
                .unwrap();
            let field = array_of_btreemap
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[SailsBTreeMap<String, bool>; 2]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsBTreeMap<String, T>; 2]"
            );

            // Category 3: Triple-nested complex types
            let array_of_vec_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfVecOfOption")
                .unwrap();
            let field = &array_of_vec_of_option.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[SailsVec<Option<SailsVec<bool>>>; 4]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 8,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsVec<Option<SailsVec<T>>>; 4]"
            );

            let tuple_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleTriple")
                .unwrap();
            let field1 = tuple_triple
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field1.ty.id).unwrap(),
                "Option<Option<SailsVec<bool>>>"
            );
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 9,
                field_index: 0,
                self_id: field1.ty.id,
            };
            assert_eq!(
                generic_names.get(&id1).unwrap(),
                "Option<Option<SailsVec<T>>>"
            );
            let field2 = tuple_triple
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field2.ty.id).unwrap(),
                "Result<Option<[bool; 3]>, String>"
            );
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 9,
                field_index: 1,
                self_id: field2.ty.id,
            };
            assert_eq!(
                generic_names.get(&id2).unwrap(),
                "Result<Option<[T; 3]>, String>"
            );

            let vec_of_struct_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "VecOfStructOfOption")
                .unwrap();
            let field = &vec_of_struct_of_option.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<GenericStruct<Option<bool>>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 10,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<Option<T>>>"
            );

            let array_complex_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayComplexTriple")
                .unwrap();
            let field = &array_complex_triple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[SailsBTreeMap<SailsBTreeMap<Option<bool>, String>, Result<bool, String>>; 2]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 11,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsBTreeMap<SailsBTreeMap<Option<T>, String>, Result<T, String>>; 2]"
            );
        } else {
            panic!("Expected variant type");
        }
    }

    #[test]
    fn multiple_generics() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct MultiGenStruct<T1, T2, T3> {
            // Category 1: Simple and complex types with single generics
            just_t1: T1,
            just_t2: T2,
            just_t3: T3,
            array_t1: [T1; 8],
            tuple_t2_t3: (T2, T3),
            vec_t3: Vec<T3>,

            // Category 2: Mixed generics in complex types
            tuple_mixed: (T1, T2, T3),
            tuple_repeated: (T1, T1, T2, T2, T3, T3),
            array_of_tuple: [(T1, T2); 4],
            vec_of_array: Vec<[T3; 5]>,
            btreemap_t1_t2: BTreeMap<T1, T2>,
            struct_of_t3: GenericStruct<T3>,
            enum_mixed: GenericEnum<T1, T2>,

            // Category 3: Two-level nested with multiple generics
            option_of_result: Option<result::Result<T1, T2>>,
            array_of_option: [Option<T2>; 6],
            vec_of_tuple: Vec<(T2, T3, T1)>,
            tuple_of_result: (result::Result<T1, String>, Option<T2>),
            btreemap_nested: BTreeMap<Option<T1>, result::Result<T2, String>>,
            struct_of_tuple: GenericStruct<(T2, T3)>,

            // Category 4: Triple-nested complex types with multiple generics
            option_triple: Option<result::Result<Vec<T1>, T2>>,
            array_triple: [BTreeMap<T1, Option<T2>>; 3],
            vec_of_struct_of_option: Vec<GenericStruct<Option<T3>>>,
            array_of_vec_of_tuple: [Vec<(T1, T2)>; 2],
            tuple_complex_triple: (Option<Vec<T1>>, result::Result<[T2; 4], T3>),
            vec_complex: Vec<GenericStruct<result::Result<T1, T2>>>,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum MultiGenEnum<T1, T2, T3> {
            // Category 1: Simple and complex types with single generics
            JustT1(T1),
            JustT2(T2),
            JustT3(T3),
            ArrayT1([T1; 8]),
            TupleT2T3(T2, T3),
            VecT3 {
                vec: Vec<T3>,
            },

            // Category 2: Mixed generics in complex types
            TupleMixed(T1, T2, T3),
            TupleRepeated(T1, T1, T2, T2, T3, T3),
            ArrayOfTuple([(T1, T2); 4]),
            VecOfArray {
                vec: Vec<[T3; 5]>,
            },
            BTreeMapT1T2 {
                map: BTreeMap<T1, T2>,
            },
            StructOfT3(GenericStruct<T3>),
            EnumMixed {
                inner: GenericEnum<T1, T2>,
            },

            // Category 3: Two-level nested with multiple generics
            OptionOfResult(Option<result::Result<T1, T2>>),
            ArrayOfOption([Option<T2>; 6]),
            VecOfTuple(Vec<(T2, T3, T1)>),
            TupleOfResult {
                field1: result::Result<T1, String>,
                field2: Option<T2>,
            },
            BTreeMapNested {
                map: BTreeMap<Option<T1>, result::Result<T2, String>>,
            },
            StructOfTuple(GenericStruct<(T2, T3)>),

            // Category 4: Triple-nested complex types with multiple generics
            OptionTriple(Option<result::Result<Vec<T1>, T2>>),
            ArrayTriple([BTreeMap<T1, Option<T2>>; 3]),
            VecOfStructOfOption(Vec<GenericStruct<Option<T3>>>),
            ArrayOfVecOfTuple {
                array: [Vec<(T1, T2)>; 2],
            },
            TupleComplexTriple {
                field1: Option<Vec<T1>>,
                field2: result::Result<[T2; 4], T3>,
            },
            VecComplex(Vec<GenericStruct<result::Result<T1, T2>>>),
        }

        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<MultiGenStruct<u32, String, H256>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<MultiGenEnum<u32, String, H256>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check main types
        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "MultiGenStruct<u32, String, H256>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(struct_id))
                .unwrap(),
            "MultiGenStruct<T1, T2, T3>"
        );

        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "MultiGenEnum<u32, String, H256>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(enum_id))
                .unwrap(),
            "MultiGenEnum<T1, T2, T3>"
        );

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();

        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            // Category 1: Simple and complex types with single generics
            let just_t1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("just_t1"))
                .unwrap();
            assert_eq!(concrete_names.get(&just_t1.ty.id).unwrap(), "u32");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 0,
                self_id: just_t1.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T1");

            let just_t2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("just_t2"))
                .unwrap();
            assert_eq!(concrete_names.get(&just_t2.ty.id).unwrap(), "String");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 1,
                self_id: just_t2.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T2");

            let just_t3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("just_t3"))
                .unwrap();
            assert_eq!(concrete_names.get(&just_t3.ty.id).unwrap(), "H256");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 2,
                self_id: just_t3.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T3");

            let array_t1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_t1"))
                .unwrap();
            assert_eq!(concrete_names.get(&array_t1.ty.id).unwrap(), "[u32; 8]");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 3,
                self_id: array_t1.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T1; 8]");

            let tuple_t2_t3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_t2_t3"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_t2_t3.ty.id).unwrap(),
                "(String, H256)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 4,
                self_id: tuple_t2_t3.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "(T2, T3)");

            let vec_t3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_t3"))
                .unwrap();
            assert_eq!(concrete_names.get(&vec_t3.ty.id).unwrap(), "SailsVec<H256>");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 5,
                self_id: vec_t3.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<T3>");

            // Category 2: Mixed generics in complex types
            let tuple_mixed = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_mixed"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_mixed.ty.id).unwrap(),
                "(u32, String, H256)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 6,
                self_id: tuple_mixed.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "(T1, T2, T3)");

            let tuple_repeated = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_repeated"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_repeated.ty.id).unwrap(),
                "(u32, u32, String, String, H256, H256)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 7,
                self_id: tuple_repeated.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "(T1, T1, T2, T2, T3, T3)");

            let array_of_tuple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_tuple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_tuple.ty.id).unwrap(),
                "[(u32, String); 4]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 8,
                self_id: array_of_tuple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[(T1, T2); 4]");

            let vec_of_array = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_of_array"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_of_array.ty.id).unwrap(),
                "SailsVec<[H256; 5]>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 9,
                self_id: vec_of_array.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<[T3; 5]>");

            let btreemap_t1_t2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("btreemap_t1_t2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btreemap_t1_t2.ty.id).unwrap(),
                "SailsBTreeMap<u32, String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 10,
                self_id: btreemap_t1_t2.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsBTreeMap<T1, T2>");

            let struct_of_t3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("struct_of_t3"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&struct_of_t3.ty.id).unwrap(),
                "GenericStruct<H256>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 11,
                self_id: struct_of_t3.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<T3>");

            let enum_mixed = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("enum_mixed"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&enum_mixed.ty.id).unwrap(),
                "GenericEnum<u32, String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 12,
                self_id: enum_mixed.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericEnum<T1, T2>");

            // Category 3: Two-level nested with multiple generics
            let option_of_result = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_of_result"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_of_result.ty.id).unwrap(),
                "Option<Result<u32, String>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 13,
                self_id: option_of_result.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Result<T1, T2>>");

            let array_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_option.ty.id).unwrap(),
                "[Option<String>; 6]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 14,
                self_id: array_of_option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[Option<T2>; 6]");

            let vec_of_tuple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_of_tuple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_of_tuple.ty.id).unwrap(),
                "SailsVec<(String, H256, u32)>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 15,
                self_id: vec_of_tuple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<(T2, T3, T1)>");

            let tuple_of_result = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_of_result"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_of_result.ty.id).unwrap(),
                "(Result<u32, String>, Option<String>)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 16,
                self_id: tuple_of_result.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "(Result<T1, String>, Option<T2>)"
            );

            let btreemap_nested = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("btreemap_nested"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btreemap_nested.ty.id).unwrap(),
                "SailsBTreeMap<Option<u32>, Result<String, String>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 17,
                self_id: btreemap_nested.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<Option<T1>, Result<T2, String>>"
            );

            let struct_of_tuple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("struct_of_tuple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&struct_of_tuple.ty.id).unwrap(),
                "GenericStruct<(String, H256)>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 18,
                self_id: struct_of_tuple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<(T2, T3)>");

            // Category 4: Triple-nested complex types with multiple generics
            let option_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_triple.ty.id).unwrap(),
                "Option<Result<SailsVec<u32>, String>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 19,
                self_id: option_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Option<Result<SailsVec<T1>, T2>>"
            );

            let array_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_triple.ty.id).unwrap(),
                "[SailsBTreeMap<u32, Option<String>>; 3]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 20,
                self_id: array_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsBTreeMap<T1, Option<T2>>; 3]"
            );

            let vec_of_struct_of_option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_of_struct_of_option"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_of_struct_of_option.ty.id).unwrap(),
                "SailsVec<GenericStruct<Option<H256>>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 21,
                self_id: vec_of_struct_of_option.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<Option<T3>>>"
            );

            let array_of_vec_of_tuple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array_of_vec_of_tuple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&array_of_vec_of_tuple.ty.id).unwrap(),
                "[SailsVec<(u32, String)>; 2]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 22,
                self_id: array_of_vec_of_tuple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[SailsVec<(T1, T2)>; 2]");

            let tuple_complex_triple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple_complex_triple"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_complex_triple.ty.id).unwrap(),
                "(Option<SailsVec<u32>>, Result<[String; 4], H256>)"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 23,
                self_id: tuple_complex_triple.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "(Option<SailsVec<T1>>, Result<[T2; 4], T3>)"
            );

            let vec_complex = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_complex"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_complex.ty.id).unwrap(),
                "SailsVec<GenericStruct<Result<u32, String>>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 24,
                self_id: vec_complex.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<Result<T1, T2>>>"
            );
        } else {
            panic!("Expected composite type");
        }

        // Check enum variants
        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_id)
            .unwrap();
        if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
            // Category 1: Simple and complex types with single generics
            let just_t1 = variant
                .variants
                .iter()
                .find(|v| v.name == "JustT1")
                .unwrap();
            let field = &just_t1.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "u32");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 0,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T1");

            let just_t2 = variant
                .variants
                .iter()
                .find(|v| v.name == "JustT2")
                .unwrap();
            let field = &just_t2.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "String");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 1,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T2");

            let just_t3 = variant
                .variants
                .iter()
                .find(|v| v.name == "JustT3")
                .unwrap();
            let field = &just_t3.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "H256");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 2,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T3");

            let array_t1 = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayT1")
                .unwrap();
            let field = &array_t1.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "[u32; 8]");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 3,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T1; 8]");

            let tuple_t2_t3 = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleT2T3")
                .unwrap();
            let field0 = &tuple_t2_t3.fields[0];
            assert_eq!(concrete_names.get(&field0.ty.id).unwrap(), "String");
            let id0 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 4,
                field_index: 0,
                self_id: field0.ty.id,
            };
            assert_eq!(generic_names.get(&id0).unwrap(), "T2");
            let field1 = &tuple_t2_t3.fields[1];
            assert_eq!(concrete_names.get(&field1.ty.id).unwrap(), "H256");
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 4,
                field_index: 1,
                self_id: field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "T3");

            let vec_t3 = variant.variants.iter().find(|v| v.name == "VecT3").unwrap();
            let field = vec_t3
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec"))
                .unwrap();
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "SailsVec<H256>");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 5,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<T3>");

            // Category 2: Mixed generics in complex types
            let tuple_mixed = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleMixed")
                .unwrap();
            let field0 = &tuple_mixed.fields[0];
            assert_eq!(concrete_names.get(&field0.ty.id).unwrap(), "u32");
            let id0 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 6,
                field_index: 0,
                self_id: field0.ty.id,
            };
            assert_eq!(generic_names.get(&id0).unwrap(), "T1");
            let field1 = &tuple_mixed.fields[1];
            assert_eq!(concrete_names.get(&field1.ty.id).unwrap(), "String");
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 6,
                field_index: 1,
                self_id: field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "T2");
            let field2 = &tuple_mixed.fields[2];
            assert_eq!(concrete_names.get(&field2.ty.id).unwrap(), "H256");
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 6,
                field_index: 2,
                self_id: field2.ty.id,
            };
            assert_eq!(generic_names.get(&id2).unwrap(), "T3");

            let tuple_repeated = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleRepeated")
                .unwrap();
            let field0 = &tuple_repeated.fields[0];
            assert_eq!(concrete_names.get(&field0.ty.id).unwrap(), "u32");
            let id0 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 0,
                self_id: field0.ty.id,
            };
            assert_eq!(generic_names.get(&id0).unwrap(), "T1");
            let field1 = &tuple_repeated.fields[1];
            assert_eq!(concrete_names.get(&field1.ty.id).unwrap(), "u32");
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 1,
                self_id: field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "T1");
            let field2 = &tuple_repeated.fields[2];
            assert_eq!(concrete_names.get(&field2.ty.id).unwrap(), "String");
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 2,
                self_id: field2.ty.id,
            };
            assert_eq!(generic_names.get(&id2).unwrap(), "T2");
            let field3 = &tuple_repeated.fields[3];
            assert_eq!(concrete_names.get(&field3.ty.id).unwrap(), "String");
            let id3 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 3,
                self_id: field3.ty.id,
            };
            assert_eq!(generic_names.get(&id3).unwrap(), "T2");
            let field4 = &tuple_repeated.fields[4];
            assert_eq!(concrete_names.get(&field4.ty.id).unwrap(), "H256");
            let id4 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 4,
                self_id: field4.ty.id,
            };
            assert_eq!(generic_names.get(&id4).unwrap(), "T3");
            let field5 = &tuple_repeated.fields[5];
            assert_eq!(concrete_names.get(&field5.ty.id).unwrap(), "H256");
            let id5 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 5,
                self_id: field5.ty.id,
            };
            assert_eq!(generic_names.get(&id5).unwrap(), "T3");

            let array_of_tuple = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfTuple")
                .unwrap();
            let field = &array_of_tuple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[(u32, String); 4]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 8,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[(T1, T2); 4]");

            let vec_of_array = variant
                .variants
                .iter()
                .find(|v| v.name == "VecOfArray")
                .unwrap();
            let field = vec_of_array
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<[H256; 5]>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 9,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<[T3; 5]>");

            let btreemap_t1_t2 = variant
                .variants
                .iter()
                .find(|v| v.name == "BTreeMapT1T2")
                .unwrap();
            let field = btreemap_t1_t2
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("map"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<u32, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 10,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsBTreeMap<T1, T2>");

            let struct_of_t3 = variant
                .variants
                .iter()
                .find(|v| v.name == "StructOfT3")
                .unwrap();
            let field = &struct_of_t3.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<H256>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 11,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<T3>");

            let enum_mixed = variant
                .variants
                .iter()
                .find(|v| v.name == "EnumMixed")
                .unwrap();
            let field = enum_mixed
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("inner"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericEnum<u32, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 12,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericEnum<T1, T2>");

            // Category 3: Two-level nested with multiple generics
            let option_of_result = variant
                .variants
                .iter()
                .find(|v| v.name == "OptionOfResult")
                .unwrap();
            let field = &option_of_result.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Option<Result<u32, String>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 13,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<Result<T1, T2>>");

            let array_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfOption")
                .unwrap();
            let field = &array_of_option.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[Option<String>; 6]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 14,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[Option<T2>; 6]");

            let vec_of_tuple = variant
                .variants
                .iter()
                .find(|v| v.name == "VecOfTuple")
                .unwrap();
            let field = &vec_of_tuple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<(String, H256, u32)>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 15,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<(T2, T3, T1)>");

            let tuple_of_result = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleOfResult")
                .unwrap();
            let field1 = tuple_of_result
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field1.ty.id).unwrap(),
                "Result<u32, String>"
            );
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 16,
                field_index: 0,
                self_id: field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "Result<T1, String>");
            let field2 = tuple_of_result
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field2"))
                .unwrap();
            assert_eq!(concrete_names.get(&field2.ty.id).unwrap(), "Option<String>");
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 16,
                field_index: 1,
                self_id: field2.ty.id,
            };
            assert_eq!(generic_names.get(&id2).unwrap(), "Option<T2>");

            let btreemap_nested = variant
                .variants
                .iter()
                .find(|v| v.name == "BTreeMapNested")
                .unwrap();
            let field = btreemap_nested
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("map"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<Option<u32>, Result<String, String>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 17,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<Option<T1>, Result<T2, String>>"
            );

            let struct_of_tuple = variant
                .variants
                .iter()
                .find(|v| v.name == "StructOfTuple")
                .unwrap();
            let field = &struct_of_tuple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<(String, H256)>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 18,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<(T2, T3)>");

            // Category 4: Triple-nested complex types with multiple generics
            let option_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "OptionTriple")
                .unwrap();
            let field = &option_triple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Option<Result<SailsVec<u32>, String>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 19,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Option<Result<SailsVec<T1>, T2>>"
            );

            let array_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayTriple")
                .unwrap();
            let field = &array_triple.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[SailsBTreeMap<u32, Option<String>>; 3]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 20,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[SailsBTreeMap<T1, Option<T2>>; 3]"
            );

            let vec_of_struct_of_option = variant
                .variants
                .iter()
                .find(|v| v.name == "VecOfStructOfOption")
                .unwrap();
            let field = &vec_of_struct_of_option.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<GenericStruct<Option<H256>>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 21,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<Option<T3>>>"
            );

            let array_of_vec_of_tuple = variant
                .variants
                .iter()
                .find(|v| v.name == "ArrayOfVecOfTuple")
                .unwrap();
            let field = array_of_vec_of_tuple
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[SailsVec<(u32, String)>; 2]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 22,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[SailsVec<(T1, T2)>; 2]");

            let tuple_complex_triple = variant
                .variants
                .iter()
                .find(|v| v.name == "TupleComplexTriple")
                .unwrap();
            let field1 = tuple_complex_triple
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field1.ty.id).unwrap(),
                "Option<SailsVec<u32>>"
            );
            let id1 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 23,
                field_index: 0,
                self_id: field1.ty.id,
            };
            assert_eq!(generic_names.get(&id1).unwrap(), "Option<SailsVec<T1>>");
            let field2 = tuple_complex_triple
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field2.ty.id).unwrap(),
                "Result<[String; 4], H256>"
            );
            let id2 = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 23,
                field_index: 1,
                self_id: field2.ty.id,
            };
            assert_eq!(generic_names.get(&id2).unwrap(), "Result<[T2; 4], T3>");

            let vec_complex = variant
                .variants
                .iter()
                .find(|v| v.name == "VecComplex")
                .unwrap();
            let field = &vec_complex.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<GenericStruct<Result<u32, String>>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 24,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<GenericStruct<Result<T1, T2>>>"
            );
        } else {
            panic!("Expected variant type");
        }
    }

    #[test]
    fn generic_const_with_generic_types() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct ConstGenericStruct<const N: usize, T> {
            array: [T; N],
            value: T,
            vec: Vec<T>,
            option: Option<T>,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct TwoConstGenericStruct<const N: usize, const M: usize, T1, T2> {
            array1: [T1; N],
            array2: [T2; M],
            tuple: (T1, T2),
            nested: GenericStruct<T1>,
            result: result::Result<T1, T2>,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum ConstGenericEnum<const N: usize, T> {
            Array([T; N]),
            Value(T),
            Nested { inner: GenericStruct<T> },
        }

        let mut registry = Registry::new();

        // Register ConstGenericStruct with different N and T values
        let struct_n8_u32_id = registry
            .register_type(&MetaType::new::<ConstGenericStruct<8, u32>>())
            .id;
        let struct_n8_string_id = registry
            .register_type(&MetaType::new::<ConstGenericStruct<8, String>>())
            .id;

        let struct_n16_u32_id = registry
            .register_type(&MetaType::new::<ConstGenericStruct<16, u32>>())
            .id;

        assert_ne!(struct_n8_u32_id, struct_n8_string_id);
        assert_ne!(struct_n8_u32_id, struct_n16_u32_id);

        // Register TwoConstGenericStruct
        let two_const_id = registry
            .register_type(&MetaType::new::<TwoConstGenericStruct<4, 8, u64, H256>>())
            .id;

        // Register ConstGenericEnum
        let enum_n8_bool_id = registry
            .register_type(&MetaType::new::<ConstGenericEnum<8, bool>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check ConstGenericStruct with N=8, T=u32
        assert_eq!(
            concrete_names.get(&struct_n8_u32_id).unwrap(),
            "ConstGenericStruct1<u32>"
        );
        assert_eq!(
            concrete_names.get(&struct_n8_string_id).unwrap(),
            "ConstGenericStruct<String>"
        );
        assert_eq!(
            concrete_names.get(&struct_n16_u32_id).unwrap(),
            "ConstGenericStruct2<u32>"
        );
        assert_eq!(
            concrete_names.get(&two_const_id).unwrap(),
            "TwoConstGenericStruct<u64, H256>"
        );
        assert_eq!(
            concrete_names.get(&enum_n8_bool_id).unwrap(),
            "ConstGenericEnum<bool>"
        );

        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(struct_n8_u32_id))
                .unwrap(),
            "ConstGenericStruct1<T>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(two_const_id))
                .unwrap(),
            "TwoConstGenericStruct<T1, T2>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(enum_n8_bool_id))
                .unwrap(),
            "ConstGenericEnum<T>"
        );

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_n8_u32_id)
            .unwrap();
        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            let array = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array"))
                .unwrap();
            assert_eq!(concrete_names.get(&array.ty.id).unwrap(), "[u32; 8]");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_n8_u32_id,
                field_index: 0,
                self_id: array.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T; 8]");

            let value = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("value"))
                .unwrap();
            assert_eq!(concrete_names.get(&value.ty.id).unwrap(), "u32");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_n8_u32_id,
                field_index: 1,
                self_id: value.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T");

            let vec = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec"))
                .unwrap();
            assert_eq!(concrete_names.get(&vec.ty.id).unwrap(), "SailsVec<u32>");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_n8_u32_id,
                field_index: 2,
                self_id: vec.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "SailsVec<T>");

            let option = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option"))
                .unwrap();
            assert_eq!(concrete_names.get(&option.ty.id).unwrap(), "Option<u32>");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_n8_u32_id,
                field_index: 3,
                self_id: option.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Option<T>");
        }

        let two_const_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == two_const_id)
            .unwrap();
        if let TypeDef::Composite(composite) = &two_const_type.ty.type_def {
            let array1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array1"))
                .unwrap();
            assert_eq!(concrete_names.get(&array1.ty.id).unwrap(), "[u64; 4]");
            let id = TypeRegistryId::StructField {
                parent_ty: two_const_id,
                field_index: 0,
                self_id: array1.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T1; 4]");

            let array2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("array2"))
                .unwrap();
            assert_eq!(concrete_names.get(&array2.ty.id).unwrap(), "[H256; 8]");
            let id = TypeRegistryId::StructField {
                parent_ty: two_const_id,
                field_index: 1,
                self_id: array2.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T2; 8]");

            let tuple = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("tuple"))
                .unwrap();
            assert_eq!(concrete_names.get(&tuple.ty.id).unwrap(), "(u64, H256)");
            let id = TypeRegistryId::StructField {
                parent_ty: two_const_id,
                field_index: 2,
                self_id: tuple.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "(T1, T2)");

            let nested = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("nested"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&nested.ty.id).unwrap(),
                "GenericStruct<u64>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: two_const_id,
                field_index: 3,
                self_id: nested.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<T1>");

            let result = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("result"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&result.ty.id).unwrap(),
                "Result<u64, H256>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: two_const_id,
                field_index: 4,
                self_id: result.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "Result<T1, T2>");
        }

        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_n8_bool_id)
            .unwrap();
        if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
            let array_variant = variant.variants.iter().find(|v| v.name == "Array").unwrap();
            let field = &array_variant.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "[bool; 8]");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_n8_bool_id,
                variant_data: 0,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "[T; 8]");

            let value_variant = variant.variants.iter().find(|v| v.name == "Value").unwrap();
            let field = &value_variant.fields[0];
            assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "bool");
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_n8_bool_id,
                variant_data: 1,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "T");

            let nested_variant = variant
                .variants
                .iter()
                .find(|v| v.name == "Nested")
                .unwrap();
            let field = nested_variant
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("inner"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<bool>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_n8_bool_id,
                variant_data: 2,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "GenericStruct<T>");
        }
    }

    // Types for same_name_different_modules test
    #[allow(dead_code)]
    mod same_name_test {
        use super::*;

        pub mod module_a {
            use super::*;

            #[derive(TypeInfo)]
            pub struct SameName<T> {
                pub value: T,
            }
        }

        pub mod module_b {
            use super::*;

            #[derive(TypeInfo)]
            pub struct SameName<T> {
                pub value: T,
            }
        }

        pub mod module_c {
            use super::*;

            pub mod nested {
                use super::*;

                #[derive(TypeInfo)]
                pub struct SameName<T> {
                    pub value: T,
                }
            }
        }
    }

    #[test]
    fn same_name_different_mods_generic_names() {
        use same_name_test::*;

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct TestStruct<T1, T2> {
            field_a: module_a::SameName<T1>,
            field_b: module_b::SameName<T2>,
            field_c: module_c::nested::SameName<T1>,
            generic_a: GenericStruct<module_a::SameName<T2>>,
            generic_b: GenericStruct<module_b::SameName<T1>>,
            vec_a: Vec<module_c::nested::SameName<T1>>,
            option_b: Option<module_b::SameName<T2>>,
            result_mix: result::Result<module_a::SameName<T1>, module_b::SameName<T2>>,
        }

        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<TestStruct<u32, bool>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        // Check main type
        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "TestStruct<u32, bool>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(struct_id))
                .unwrap(),
            "TestStruct<T1, T2>"
        );

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();
        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            // field_a: module_a::SameName<T1>
            let field_a = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field_a"))
                .unwrap();
            let name_a = concrete_names.get(&field_a.ty.id).unwrap();
            assert_eq!(name_a, "ModuleASameName<u32>");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 0,
                self_id: field_a.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ModuleASameName<T1>");

            // field_b: module_b::SameName<T2>
            let field_b = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field_b"))
                .unwrap();
            let name_b = concrete_names.get(&field_b.ty.id).unwrap();
            assert_eq!(name_b, "ModuleBSameName<bool>");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 1,
                self_id: field_b.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ModuleBSameName<T2>");

            // field_c: module_c::nested::SameName<T1>
            let field_c = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field_c"))
                .unwrap();
            let name_c = concrete_names.get(&field_c.ty.id).unwrap();
            assert_eq!(name_c, "NestedSameName<u32>");
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 2,
                self_id: field_c.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "NestedSameName<T1>");

            // Verify names are different
            assert_ne!(name_a, name_b);
            assert_ne!(name_a, name_c);
            assert_ne!(name_b, name_c);

            // generic_a: GenericStruct<module_a::SameName<T2>>
            let generic_a = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("generic_a"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&generic_a.ty.id).unwrap(),
                "GenericStruct<ModuleASameName<bool>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 3,
                self_id: generic_a.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ModuleASameName<T2>>"
            );

            // generic_b: GenericStruct<module_b::SameName<T1>>
            let generic_b = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("generic_b"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&generic_b.ty.id).unwrap(),
                "GenericStruct<ModuleBSameName<u32>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 4,
                self_id: generic_b.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ModuleBSameName<T1>>"
            );

            // vec_a: Vec<module_c::nested::SameName<T1>>
            let vec_a = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("vec_a"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&vec_a.ty.id).unwrap(),
                "SailsVec<NestedSameName<u32>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 5,
                self_id: vec_a.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<NestedSameName<T1>>"
            );

            // option_b: Option<module_b::SameName<T2>>
            let option_b = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("option_b"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_b.ty.id).unwrap(),
                "Option<ModuleBSameName<bool>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 6,
                self_id: option_b.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Option<ModuleBSameName<T2>>"
            );

            // result_mix: result::Result<module_a::SameName<T1>, module_b::SameName<T2>>
            let result_mix = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("result_mix"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&result_mix.ty.id).unwrap(),
                "Result<ModuleASameName<u32>, ModuleBSameName<bool>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 7,
                self_id: result_mix.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Result<ModuleASameName<T1>, ModuleBSameName<T2>>"
            );
        }
    }

    #[test]
    fn type_names_concrete_generic_reuses() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct ReuseTestStruct<T1, T2> {
            // Same type with different generic instantiations
            a1: ReusableGenericStruct<T1>,
            a1r: ReusableGenericStruct<CodeId>,

            a2: ReusableGenericStruct<Vec<T1>>,
            a2r: ReusableGenericStruct<Vec<bool>>,

            a3: ReusableGenericStruct<(T1, T2)>,
            a3r: ReusableGenericStruct<(u32, bool)>,

            b1: ReusableGenericStruct<T2>,
            b1r: ReusableGenericStruct<bool>,

            // Same enum with different instantiations
            e1: ReusableGenericEnum<T1>,
            e1r: ReusableGenericEnum<CodeId>,

            e2: ReusableGenericEnum<T2>,
            e2r: ReusableGenericEnum<bool>,

            e3: ReusableGenericEnum<String>,
            e3r: ReusableGenericEnum<[T1; 8]>,

            // Nested reuses
            n1: GenericStruct<ReusableGenericStruct<T1>>,
            n2: GenericStruct<ReusableGenericStruct<T2>>,
            n3: GenericStruct<ReusableGenericStruct<u32>>,

            // Complex reuses
            c1: Vec<ReusableGenericStruct<T1>>,
            c2: [ReusableGenericEnum<T2>; 5],
            c3: Option<ReusableGenericStruct<(T1, T2)>>,
            c4: result::Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>,
            c5: BTreeMap<T1, ReusableGenericStruct<T2>>,
            c6: BTreeMap<ReusableGenericEnum<T1>, String>,
            c7: BTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>,
            c8: BTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum ReuseTestEnum<T1, T2> {
            // Same type with different generic instantiations
            A1(ReusableGenericStruct<T1>),
            A1r(ReusableGenericStruct<CodeId>),

            A2(ReusableGenericStruct<Vec<T1>>),
            A2r(ReusableGenericStruct<Vec<bool>>),

            A3 {
                field: ReusableGenericStruct<(T1, T2)>,
            },
            A3r {
                field: ReusableGenericStruct<(u32, bool)>,
            },

            B1(ReusableGenericStruct<T2>),
            B1r(ReusableGenericStruct<bool>),

            // Same enum with different instantiations
            E1(ReusableGenericEnum<T1>),
            E1r(ReusableGenericEnum<CodeId>),

            E2(ReusableGenericEnum<T2>),
            E2r(ReusableGenericEnum<bool>),

            E3 {
                field: ReusableGenericEnum<String>,
            },
            E3r {
                field: ReusableGenericEnum<[T1; 8]>,
            },

            // Nested reuses
            N1(GenericStruct<ReusableGenericStruct<T1>>),
            N2(GenericStruct<ReusableGenericStruct<T2>>),
            N3(GenericStruct<ReusableGenericStruct<u32>>),

            // Complex reuses
            C1(Vec<ReusableGenericStruct<T1>>),
            C2 {
                field: [ReusableGenericEnum<T2>; 5],
            },
            C3(Option<ReusableGenericStruct<(T1, T2)>>),
            C4(result::Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>),
            C5 {
                field: BTreeMap<T1, ReusableGenericStruct<T2>>,
            },
            C6(BTreeMap<ReusableGenericEnum<T1>, String>),
            C7(BTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>),
            C8(BTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>),
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct ReusableGenericStruct<T> {
            data: T,
            count: u32,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum ReusableGenericEnum<T> {
            Some(T),
            None,
        }

        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<ReuseTestStruct<u64, H256>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<ReuseTestEnum<u64, H256>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "ReuseTestStruct<u64, H256>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(struct_id))
                .unwrap(),
            "ReuseTestStruct<T1, T2>"
        );
        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "ReuseTestEnum<u64, H256>"
        );
        assert_eq!(
            generic_names
                .get(&TypeRegistryId::ParentTy(enum_id))
                .unwrap(),
            "ReuseTestEnum<T1, T2>"
        );

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();

        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            // a1: ReusableGenericStruct<T1>
            let a1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("a1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&a1.ty.id).unwrap(),
                "ReusableGenericStruct<u64>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 0,
                self_id: a1.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericStruct<T1>");

            // a1r: ReusableGenericStruct<CodeId> - concrete type
            let a1r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("a1r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&a1r.ty.id).unwrap(),
                "ReusableGenericStruct<CodeId>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 1,
                self_id: a1r.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<CodeId>"
            );

            // a2: ReusableGenericStruct<Vec<T1>>
            let a2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("a2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&a2.ty.id).unwrap(),
                "ReusableGenericStruct<SailsVec<u64>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 2,
                self_id: a2.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<SailsVec<T1>>"
            );

            // a2r: ReusableGenericStruct<Vec<bool>> - concrete type
            let a2r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("a2r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&a2r.ty.id).unwrap(),
                "ReusableGenericStruct<SailsVec<bool>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 3,
                self_id: a2r.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<SailsVec<bool>>"
            );

            // a3: ReusableGenericStruct<(T1, T2)>
            let a3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("a3"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&a3.ty.id).unwrap(),
                "ReusableGenericStruct<(u64, H256)>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 4,
                self_id: a3.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<(T1, T2)>"
            );

            // a3r: ReusableGenericStruct<(u32, bool)> - concrete type
            let a3r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("a3r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&a3r.ty.id).unwrap(),
                "ReusableGenericStruct<(u32, bool)>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 5,
                self_id: a3r.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<(u32, bool)>"
            );

            // b1: ReusableGenericStruct<T2>
            let b1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("b1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&b1.ty.id).unwrap(),
                "ReusableGenericStruct<H256>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 6,
                self_id: b1.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericStruct<T2>");

            // b1r: ReusableGenericStruct<bool> - concrete type
            let b1r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("b1r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&b1r.ty.id).unwrap(),
                "ReusableGenericStruct<bool>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 7,
                self_id: b1r.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<bool>"
            );

            // e1: ReusableGenericEnum<T1>
            let e1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("e1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&e1.ty.id).unwrap(),
                "ReusableGenericEnum<u64>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 8,
                self_id: e1.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericEnum<T1>");

            // e1r: ReusableGenericEnum<CodeId> - concrete type
            let e1r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("e1r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&e1r.ty.id).unwrap(),
                "ReusableGenericEnum<CodeId>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 9,
                self_id: e1r.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericEnum<CodeId>"
            );

            // e2: ReusableGenericEnum<T2>
            let e2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("e2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&e2.ty.id).unwrap(),
                "ReusableGenericEnum<H256>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 10,
                self_id: e2.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericEnum<T2>");

            // e2r: ReusableGenericEnum<bool> - concrete type
            let e2r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("e2r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&e2r.ty.id).unwrap(),
                "ReusableGenericEnum<bool>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 11,
                self_id: e2r.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericEnum<bool>");

            // e3: ReusableGenericEnum<String> - concrete type
            let e3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("e3"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&e3.ty.id).unwrap(),
                "ReusableGenericEnum<String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 12,
                self_id: e3.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericEnum<String>"
            );

            // e3r: ReusableGenericEnum<[T1; 8]>
            let e3r = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("e3r"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&e3r.ty.id).unwrap(),
                "ReusableGenericEnum<[u64; 8]>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 13,
                self_id: e3r.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericEnum<[T1; 8]>"
            );

            // n1: GenericStruct<ReusableGenericStruct<T1>>
            let n1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("n1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&n1.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u64>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 14,
                self_id: n1.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ReusableGenericStruct<T1>>"
            );

            // n2: GenericStruct<ReusableGenericStruct<T2>>
            let n2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("n2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&n2.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<H256>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 15,
                self_id: n2.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ReusableGenericStruct<T2>>"
            );

            // n3: GenericStruct<ReusableGenericStruct<u32>> - concrete
            let n3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("n3"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&n3.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u32>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 16,
                self_id: n3.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u32>>"
            );

            // c1: Vec<ReusableGenericStruct<T1>>
            let c1 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c1.ty.id).unwrap(),
                "SailsVec<ReusableGenericStruct<u64>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 17,
                self_id: c1.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<ReusableGenericStruct<T1>>"
            );

            // c2: [ReusableGenericEnum<T2>; 5]
            let c2 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c2"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c2.ty.id).unwrap(),
                "[ReusableGenericEnum<H256>; 5]"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 18,
                self_id: c2.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[ReusableGenericEnum<T2>; 5]"
            );

            // c3: Option<ReusableGenericStruct<(T1, T2)>>
            let c3 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c3"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c3.ty.id).unwrap(),
                "Option<ReusableGenericStruct<(u64, H256)>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 19,
                self_id: c3.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Option<ReusableGenericStruct<(T1, T2)>>"
            );

            // c4: result::Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>
            let c4 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c4"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c4.ty.id).unwrap(),
                "Result<ReusableGenericEnum<u64>, ReusableGenericEnum<H256>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 20,
                self_id: c4.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>"
            );

            // c5: BTreeMap<T1, ReusableGenericStruct<T2>>
            let c5 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c5"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c5.ty.id).unwrap(),
                "SailsBTreeMap<u64, ReusableGenericStruct<H256>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 21,
                self_id: c5.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<T1, ReusableGenericStruct<T2>>"
            );

            // c6: BTreeMap<ReusableGenericEnum<T1>, String>
            let c6 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c6"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c6.ty.id).unwrap(),
                "SailsBTreeMap<ReusableGenericEnum<u64>, String>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 22,
                self_id: c6.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<ReusableGenericEnum<T1>, String>"
            );

            // c7: BTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>
            let c7 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c7"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c7.ty.id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 23,
                self_id: c7.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>"
            );

            // c8: BTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>> - concrete
            let c8 = composite
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("c8"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&c8.ty.id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>"
            );
            let id = TypeRegistryId::StructField {
                parent_ty: struct_id,
                field_index: 24,
                self_id: c8.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>"
            );
        } else {
            panic!("Expected composite type");
        }

        // Check enum variants
        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_id)
            .unwrap();
        if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
            // A1: ReusableGenericStruct<T1>
            let a1 = variant.variants.iter().find(|v| v.name == "A1").unwrap();
            let field = &a1.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<u64>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 0,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericStruct<T1>");

            // A1r: ReusableGenericStruct<CodeId> - concrete
            let a1r = variant.variants.iter().find(|v| v.name == "A1r").unwrap();
            let field = &a1r.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<CodeId>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 1,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<CodeId>"
            );

            // A2: ReusableGenericStruct<Vec<T1>>
            let a2 = variant.variants.iter().find(|v| v.name == "A2").unwrap();
            let field = &a2.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<SailsVec<u64>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 2,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<SailsVec<T1>>"
            );

            // A2r: ReusableGenericStruct<Vec<bool>> - concrete
            let a2r = variant.variants.iter().find(|v| v.name == "A2r").unwrap();
            let field = &a2r.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<SailsVec<bool>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 3,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<SailsVec<bool>>"
            );

            // A3: ReusableGenericStruct<(T1, T2)>
            let a3 = variant.variants.iter().find(|v| v.name == "A3").unwrap();
            let field = a3
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<(u64, H256)>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 4,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<(T1, T2)>"
            );

            // A3r: ReusableGenericStruct<(u32, bool)> - concrete
            let a3r = variant.variants.iter().find(|v| v.name == "A3r").unwrap();
            let field = a3r
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<(u32, bool)>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 5,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<(u32, bool)>"
            );

            // B1: ReusableGenericStruct<T2>
            let b1 = variant.variants.iter().find(|v| v.name == "B1").unwrap();
            let field = &b1.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<H256>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 6,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericStruct<T2>");

            // B1r: ReusableGenericStruct<bool> - concrete
            let b1r = variant.variants.iter().find(|v| v.name == "B1r").unwrap();
            let field = &b1r.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericStruct<bool>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 7,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericStruct<bool>"
            );

            // E1: ReusableGenericEnum<T1>
            let e1 = variant.variants.iter().find(|v| v.name == "E1").unwrap();
            let field = &e1.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericEnum<u64>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 8,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericEnum<T1>");

            // E1r: ReusableGenericEnum<CodeId> - concrete
            let e1r = variant.variants.iter().find(|v| v.name == "E1r").unwrap();
            let field = &e1r.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericEnum<CodeId>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 9,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericEnum<CodeId>"
            );

            // E2: ReusableGenericEnum<T2>
            let e2 = variant.variants.iter().find(|v| v.name == "E2").unwrap();
            let field = &e2.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericEnum<H256>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 10,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericEnum<T2>");

            // E2r: ReusableGenericEnum<bool> - concrete
            let e2r = variant.variants.iter().find(|v| v.name == "E2r").unwrap();
            let field = &e2r.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericEnum<bool>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 11,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(generic_names.get(&id).unwrap(), "ReusableGenericEnum<bool>");

            // E3: ReusableGenericEnum<String> - concrete
            let e3 = variant.variants.iter().find(|v| v.name == "E3").unwrap();
            let field = e3
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericEnum<String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 12,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericEnum<String>"
            );

            // E3r: ReusableGenericEnum<[T1; 8]>
            let e3r = variant.variants.iter().find(|v| v.name == "E3r").unwrap();
            let field = e3r
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "ReusableGenericEnum<[u64; 8]>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 13,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "ReusableGenericEnum<[T1; 8]>"
            );

            // N1: GenericStruct<ReusableGenericStruct<T1>>
            let n1 = variant.variants.iter().find(|v| v.name == "N1").unwrap();
            let field = &n1.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u64>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 14,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ReusableGenericStruct<T1>>"
            );

            // N2: GenericStruct<ReusableGenericStruct<T2>>
            let n2 = variant.variants.iter().find(|v| v.name == "N2").unwrap();
            let field = &n2.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<H256>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 15,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ReusableGenericStruct<T2>>"
            );

            // N3: GenericStruct<ReusableGenericStruct<u32>> - concrete
            let n3 = variant.variants.iter().find(|v| v.name == "N3").unwrap();
            let field = &n3.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u32>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 16,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "GenericStruct<ReusableGenericStruct<u32>>"
            );

            // C1: Vec<ReusableGenericStruct<T1>>
            let c1 = variant.variants.iter().find(|v| v.name == "C1").unwrap();
            let field = &c1.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsVec<ReusableGenericStruct<u64>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 17,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsVec<ReusableGenericStruct<T1>>"
            );

            // C2: [ReusableGenericEnum<T2>; 5]
            let c2 = variant.variants.iter().find(|v| v.name == "C2").unwrap();
            let field = c2
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "[ReusableGenericEnum<H256>; 5]"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 18,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "[ReusableGenericEnum<T2>; 5]"
            );

            // C3: Option<ReusableGenericStruct<(T1, T2)>>
            let c3 = variant.variants.iter().find(|v| v.name == "C3").unwrap();
            let field = &c3.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Option<ReusableGenericStruct<(u64, H256)>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 19,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Option<ReusableGenericStruct<(T1, T2)>>"
            );

            // C4: result::Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>
            let c4 = variant.variants.iter().find(|v| v.name == "C4").unwrap();
            let field = &c4.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "Result<ReusableGenericEnum<u64>, ReusableGenericEnum<H256>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 20,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>"
            );

            // C5: BTreeMap<T1, ReusableGenericStruct<T2>>
            let c5 = variant.variants.iter().find(|v| v.name == "C5").unwrap();
            let field = c5
                .fields
                .iter()
                .find(|f| f.name.as_deref() == Some("field"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<u64, ReusableGenericStruct<H256>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 21,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<T1, ReusableGenericStruct<T2>>"
            );

            // C6: BTreeMap<ReusableGenericEnum<T1>, String>
            let c6 = variant.variants.iter().find(|v| v.name == "C6").unwrap();
            let field = &c6.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<ReusableGenericEnum<u64>, String>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 22,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<ReusableGenericEnum<T1>, String>"
            );

            // C7: BTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>
            let c7 = variant.variants.iter().find(|v| v.name == "C7").unwrap();
            let field = &c7.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 23,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>"
            );

            // C8: BTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>> - concrete
            let c8 = variant.variants.iter().find(|v| v.name == "C8").unwrap();
            let field = &c8.fields[0];
            assert_eq!(
                concrete_names.get(&field.ty.id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>"
            );
            let id = TypeRegistryId::EnumField {
                parent_ty: enum_id,
                variant_data: 24,
                field_index: 0,
                self_id: field.ty.id,
            };
            assert_eq!(
                generic_names.get(&id).unwrap(),
                "SailsBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>"
            );
        } else {
            panic!("Expected variant type");
        }
    }
}
