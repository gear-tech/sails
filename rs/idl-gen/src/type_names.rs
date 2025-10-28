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
use quote::ToTokens;
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};
use gprimitives::*;
use scale_info::{
    Field, PortableType, Type, TypeDef, TypeDefArray, TypeDefPrimitive, TypeDefSequence,
    TypeDefTuple, TypeInfo, form::PortableForm,
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
    result::Result as StdResult,
    sync::OnceLock,
};

const INTERIM_VEC_TYPE_NAME: &str = "SailsInterimNameVec";
const INTERIM_BTREE_MAP_TYPE_NAME: &str = "SailsInterimNameBTreeMap";

pub(super) fn resolve<'a>(
    types: impl Iterator<Item = &'a PortableType>,
) -> Result<BTreeMap<u32, String>> {
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

    type_names.map(|type_names| {
        type_names
            .0
            .iter()
            .map(|(id, name)| (*id, name.as_string(false, &type_names.1)))
            .collect()
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

        format!("SailsInterimNameBTreeMap<{key_type_name}, {value_type_name}>")
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
        let field_type_names = self
            .field_type_names
            .iter()
            .map(|tn| tn.as_string(for_generic_param, by_path_type_names))
            .collect::<Vec<_>>();
        if field_type_names.len() == 1 {
            format!("({},)", field_type_names[0])
        } else {
            format!("({})", field_type_names.join(", "))
        }
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
        format!("SailsInterimNameVec<{item_type_name}>")
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

/// Type name with generics info.
///
/// Basically it's the string representation of how
/// the type was declared by the user, including generic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum RawNames {
    #[serde(rename = "enum")]
    Enum {
        own_name: String,
        fields: Vec<TypeFieldsOwner>,
    },
    #[serde(rename = "struct")]
    Struct(TypeFieldsOwner),
}

impl RawNames {
    fn new_struct(own_name: String) -> Self {
        RawNames::Struct(TypeFieldsOwner {
            own_name,
            fields: Vec::new(),
        })
    }

    fn new_enum(own_name: String) -> Self {
        RawNames::Enum {
            own_name,
            fields: Vec::new(),
        }
    }

    fn add_struct_field(&mut self, field_name: Option<String>, type_name: String) {
        match self {
            RawNames::Struct(owner) => owner.fields.push(TypeField {
                name: field_name,
                type_name,
            }),
            RawNames::Enum { .. } => {}
        }
    }

    fn add_enum_field(
        &mut self,
        variant_idx: usize,
        field_name: Option<String>,
        type_name: String,
    ) {
        let RawNames::Enum { fields, .. } = self else {
            return;
        };

        let Some(variant_owner) = fields.get_mut(variant_idx) else {
            panic!("internal error: variant not initialized");
        };

        variant_owner.fields.push(TypeField {
            name: field_name,
            type_name,
        });
    }

    fn initialize_enum_field(&mut self, variant_idx: usize, variant_name: String) {
        let RawNames::Enum { fields, .. } = self else {
            return;
        };

        let None = fields.get(variant_idx) else {
            panic!("internal error: variant already initialized");
        };

        if fields.len() != variant_idx {
            panic!("internal error: variant indices must be sequential");
        }

        let new_variant_owner = TypeFieldsOwner {
            own_name: variant_name,
            fields: Vec::new(),
        };

        fields.push(new_variant_owner);
    }

    #[cfg(test)]
    pub(crate) fn type_name(&self) -> &str {
        match self {
            RawNames::Enum { own_name: name, .. } => name,
            RawNames::Struct(owner) => &owner.own_name,
        }
    }

    #[cfg(test)]
    pub(crate) fn fields_type_names(&self) -> Vec<&str> {
        match self {
            RawNames::Enum { fields, .. } => fields
                .iter()
                .flat_map(|owner| owner.fields.iter().map(|field| field.type_name.as_str()))
                .collect(),
            RawNames::Struct(owner) => owner
                .fields
                .iter()
                .map(|field| field.type_name.as_str())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct TypeFieldsOwner {
    pub own_name: String,
    pub fields: Vec<TypeField>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct TypeField {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub type_name: String,
}

/// Build names for types defined by user, but with generic type params.
///
/// Takes types from the portable registry along with concrete (resolved) types names
/// and produces a map of type names with generic params as they were declared.
pub(crate) fn resolve_user_generic_type_names<'a>(
    types: impl Iterator<Item = &'a PortableType>,
    concrete_names: &BTreeMap<u32, String>,
    filter_out_types: &HashSet<u32>,
) -> Result<BTreeMap<u32, RawNames>> {
    let types = types.map(|t| (t.id, t));

    let mut generic_names = BTreeMap::new();

    // Iterate through all types and process their fields
    for (parent_type_id, parent_type_info) in types {
        if filter_out_types.contains(&parent_type_id) {
            continue;
        }

        let parent_type_info = &parent_type_info.ty;

        if parent_type_info.path.namespace().is_empty() {
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
        let parent_type_name = resolve_parent_type_name(
            concrete_names
                .get(&parent_type_id)
                .ok_or(Error::TypeIdIsUnknown(parent_type_id))?,
            &params_names,
        )?;

        // Construct set of param names for easier lookup
        let params_names = params_names.into_iter().collect::<HashSet<_>>();

        // Then insert fields
        let type_generic_name = match &parent_type_info.type_def {
            TypeDef::Composite(composite) => {
                let mut parent_generic_name = RawNames::new_struct(parent_type_name.clone());
                resolve_fields_types_names(
                    composite.fields.iter(),
                    concrete_names,
                    &params_names,
                    &mut parent_generic_name,
                    None,
                )?;

                parent_generic_name
            }
            TypeDef::Variant(variant) => {
                let mut parent_generic_name = RawNames::new_enum(parent_type_name.clone());
                for variant in variant.variants.iter() {
                    let variant_idx = variant.index as usize;
                    let variant_name = variant.name.to_string();

                    parent_generic_name.initialize_enum_field(variant_idx, variant_name);

                    // if there're no fields, then it's a unit variant
                    if !variant.fields.is_empty() {
                        resolve_fields_types_names(
                            variant.fields.iter(),
                            concrete_names,
                            &params_names,
                            &mut parent_generic_name,
                            Some(variant_idx),
                        )?;
                    }
                }

                parent_generic_name
            }
            _ => unreachable!("Must not be handled"),
        };

        if generic_names
            .insert(parent_type_id, type_generic_name)
            .is_some()
        {
            return Err(TypeNameResolutionError::MainTypeRepetition(parent_type_name).into());
        }
    }

    Ok(generic_names)
}

/// Construct parent type name declaration with generics.
///
/// Simply takes the concrete name and replaces concrete generics with generic param names,
/// by splitting at `<` and joining with provided type param names.
fn resolve_parent_type_name(concrete_name: &str, type_params: &[String]) -> Result<String> {
    let type_name_without_generics =
        concrete_name
            .split('<')
            .next()
            .ok_or(TypeNameResolutionError::UnexpectedValue(format!(
                "Expected struct/enum type with `<` symbol, got - {concrete_name}"
            )))?;

    Ok(if type_params.is_empty() {
        type_name_without_generics.to_string()
    } else {
        format!("{type_name_without_generics}<{}>", type_params.join(", "))
    })
}

fn resolve_fields_types_names<'a>(
    fields_iter: impl Iterator<Item = &'a Field<PortableForm>>,
    concrete_names: &BTreeMap<u32, String>,
    params_names: &HashSet<String>,
    parent_generic_name: &mut RawNames,
    variant_idx: Option<usize>,
) -> Result<()> {
    for field in fields_iter {
        let field_name = field.name.map(|name| name.to_string());
        let field_type_id = field.ty.id;
        let field_type_name = field.type_name.expect("field must have name set");

        let field_type_name =
            resolve_field_type_name(field_type_id, field_type_name, concrete_names, params_names)?;

        if let Some(variant_idx) = variant_idx {
            parent_generic_name.add_enum_field(variant_idx, field_name, field_type_name);
        } else {
            parent_generic_name.add_struct_field(field_name, field_type_name);
        }
    }

    Ok(())
}

fn resolve_field_type_name(
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
    let syn_field_type_name = syn::parse_str::<syn::Type>(field_type_name).map_err(|e| {
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

    Ok(resolved_type.format())
}

mod resolve_to_generic {
    use super::*;
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
    /// The algorithm works the following way:
    /// - If `initial` is just a generic parameter, then return it as is, else take concrete value,
    ///   because if values differ, then it's due to type resolution, and resolved names from concrete
    ///   must be taken.
    /// - If `initial` and `concrete` are complex types owning internally other types (like arrays, tuples and etc.)
    ///   then recursively resolve their inner types. The resolution is done based on instruction described above.
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
                concrete.format(),
                initial.format(),
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
                concrete.format(),
                initial.format(),
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

trait FormatSynType {
    fn format(&self) -> String;
}

impl<T: ToTokens> FormatSynType for T {
    fn format(&self) -> String {
        self
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
}

mod finalize_type_name {
    use super::*;
    use syn::{Type, PathArguments, GenericArgument, TypeArray, TypeParen, TypePath, TypeSlice, TypeTuple, TypeReference};

    pub(super) fn finalize(type_name: &str) -> String {
        let syn_type = syn::parse_str::<Type>(type_name).unwrap_or_else(|_| {
            panic!(
                "internal error: failed to parse type name during finalization: {type_name}"
            )
        });

        finalize_syn(&syn_type)
    }

    fn finalize_syn(t: &Type) -> String {
        match t {
            Type::Array(TypeArray { elem, len, .. }) => {
                format!("[{}; {}]", finalize_syn(elem), len.format())
            }
            Type::Slice(TypeSlice { elem, .. }) => format!("[{}]", finalize_syn(elem)),
            Type::Tuple(TypeTuple { elems, .. }) => {
                let elements = elems.iter().map(finalize_syn).collect::<Vec<_>>();
                if elements.len() == 1 {
                    format!("({},)", elements[0])
                } else {
                    format!("({})", elements.join(", "))
                }
            }
            // TODO: should references remain?
            Type::Reference(TypeReference { elem, .. }) => finalize_syn(elem),
            // No paren types in the final output. Only single value tuples
            Type::Paren(TypeParen { elem, .. }) => finalize_syn(elem),
            Type::Path(TypePath { path, .. }) => {
                let last_segment = path.segments.last().unwrap();
                let ident = last_segment.ident.to_string();

                if let PathArguments::AngleBracketed(syn_args) = &last_segment.arguments {
                    let args = (&syn_args.args).iter().map(finalize_type_inner).collect::<Vec<_>>().join(", ");

                    match ident.as_str() {
                        INTERIM_VEC_TYPE_NAME => {
                            if syn_args.args.len() != 1 {
                                panic!("Vec is accepted only with one generic argument");
                            }

                            format!("[{args}]")
                        }
                        INTERIM_BTREE_MAP_TYPE_NAME => {
                            if syn_args.args.len() != 2 {
                                panic!("BTreeMap is accepted only with two generic arguments");
                            }

                            format!("[({})]", args)
                        }
                        ident => format!("{ident}<{args}>"),
                    }
                } else {
                    ident
                }
            }
            _ => t.format(),
        }
    }

    fn finalize_type_inner(arg: &GenericArgument) -> String {
        match arg {
            GenericArgument::Type(t) => finalize_syn(t),
            _ => arg.format(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::{MetaType, PortableRegistry, Registry, TypeDefComposite, Variant};
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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

        let u32_vector_name = type_names.get(&u32_vector_id).unwrap();
        assert_eq!(u32_vector_name, "SailsInterimNameVec<u32>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(as_generic_param_name, "GenericStruct<SailsInterimNameVec<u32>>");
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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

        let btree_map_name = type_names.get(&btree_map_id).unwrap();
        assert_eq!(btree_map_name, "SailsInterimNameBTreeMap<u32, String>");
        let as_generic_param_name = type_names.get(&as_generic_param_id).unwrap();
        assert_eq!(
            as_generic_param_name,
            "GenericStruct<SailsInterimNameBTreeMap<u32, String>>"
        );
    }

    #[test]
    fn type_name_minification_works_for_types_with_the_same_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry.register_type(&MetaType::new::<mod_1::T1>()).id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T1>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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

            let type_names = resolve(portable_registry.types.iter()).unwrap();

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

        let type_names = resolve(portable_registry.types.iter()).unwrap();

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
    fn finalize_names() {
        let cases = vec![
            // Vec replacements
            ("SailsInterimNameVec<i32>", "[i32]"),
            ("SailsInterimNameVec<&str>", "[str]"),
            // Paren types are unwrapped
            ("(SailsInterimNameVec<String>)", "[String]"),
            ("SailsInterimNameVec<&'a mut u64>", "[u64]"),
            ("SailsInterimNameVec<(String, u8)>", "[(String, u8)]"),
            ("&SailsInterimNameVec<String>", "[String]"),
            ("&(SailsInterimNameVec<u32>)", "[u32]"),
            ("SailsInterimNameVec<SailsInterimNameVec<bool>>", "[[bool]]"),
            ("&(SailsInterimNameVec<&'static SailsInterimNameVec<i16>>)", "[[i16]]"),
            ("SailsInterimNameVec<[u32; 4]>", "[[u32; 4]]"),
            ("SailsInterimNameVec<(SailsInterimNameVec<i8>, f64)>", "[([i8], f64)]"),

            ("Result<SailsInterimNameVec<u8>, String>", "Result<[u8], String>"),
            ("Option<SailsInterimNameBTreeMap<i32, i32>>", "Option<[(i32, i32)]>"),
            ("(u32, SailsInterimNameVec<i64>, bool)", "(u32, [i64], bool)"),
            ("(SailsInterimNameBTreeMap<u8, u8>, (SailsInterimNameVec<u128>, char))", "([(u8, u8)], ([u128], char))"),

            ("SailsInterimNameBTreeMap<String, u32>", "[(String, u32)]"),
            ("SailsInterimNameBTreeMap<&'a str, i8>", "[(str, i8)]"),
            ("SailsInterimNameBTreeMap<u8, SailsInterimNameVec<bool>>", "[(u8, [bool])]"),
            ("&SailsInterimNameBTreeMap<i32, f32>", "[(i32, f32)]"),
            ("&(SailsInterimNameBTreeMap<&'static str, SailsInterimNameVec<f32>>)", "[(str, [f32])]"),
            ("SailsInterimNameBTreeMap<u8, SailsInterimNameBTreeMap<u8, u8>>", "[(u8, [(u8, u8)])]"),
            ("SailsInterimNameVec<SailsInterimNameBTreeMap<i32, i32>>", "[[(i32, i32)]]"),
            ("SailsInterimNameBTreeMap<(u8, u8), SailsInterimNameVec<(u8, u8)>>", "[((u8, u8), [(u8, u8)])]"),

            // Mixed/edge cases
            ("&'a mut SailsInterimNameBTreeMap<String, &'a T>", "[(String, T)]"),
            ("(SailsInterimNameVec<i32>, &'b SailsInterimNameBTreeMap<bool, bool>)", "([i32], [(bool, bool)])"),
            ("[SailsInterimNameBTreeMap<u8, SailsInterimNameVec<u8>>; 10]", "[[(u8, [u8])]; 10]"),
            ("Result<SailsInterimNameVec<SailsInterimNameBTreeMap<u8, SailsInterimNameVec<u8>>>, ()>", "Result<[[(u8, [u8])]], ()>"),

            // Double unwrap of a paren type
            ("((SailsInterimNameVec<u8>))", "[u8]"),

            // Single-element tuple in paren
            ("((SailsInterimNameVec<u8>),)", "([u8],)"),
            // Single-element tuple
            ("(SailsInterimNameVec<u8>,)", "([u8],)"),

            (
                "(Option<[SailsInterimNameBTreeMap<char, SailsInterimNameVec<u8>>; 4]>, &[SailsInterimNameVec<u8>])",
                "(Option<[[(char, [u8])]; 4]>, [[u8]])"
            ),
        ];
    

        for (input, expected) in cases {
            let finalized = finalize_type_name::finalize(input);
            assert_eq!(finalized, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn simple_cases_one_generic() {
        // Define helper types for the test
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct SimpleOneGenericStruct<T> {
            // Category 1: Simple generic usage
            concrete: u32,
            genericless_unit: GenericlessUnitStruct,
            genericless_tuple: GenericlessTupleStruct,
            genericless_named: GenericlessNamedStruct,
            genericless_enum: GenericlessEnum,
            genericless_variantless_enum: GenericlessVariantlessEnum,
            generic_value: T,
            tuple_generic: (String, T, T, u32),
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
            TupleGeneric(String, T, T, u32),
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
        struct GenericlessUnitStruct;

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct GenericlessTupleStruct(u32, String);

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct GenericlessNamedStruct {
            a: u32,
            b: String,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum GenericlessEnum {
            Unit,
            Tuple(u32, String),
            Named { a: u32, b: String },
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum GenericlessVariantlessEnum {}

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

        let genericless_unit_id = registry
            .register_type(&MetaType::new::<GenericlessUnitStruct>())
            .id;
        let genericless_tuple_id = registry
            .register_type(&MetaType::new::<GenericlessTupleStruct>())
            .id;
        let genericless_named_id = registry
            .register_type(&MetaType::new::<GenericlessNamedStruct>())
            .id;
        let genericless_enum_id = registry
            .register_type(&MetaType::new::<GenericlessEnum>())
            .id;
        let genericless_variantless_enum_id = registry
            .register_type(&MetaType::new::<GenericlessVariantlessEnum>())
            .id;

        let portable_registry = PortableRegistry::from(registry);

        // First get concrete names via existing resolve (unchanged)
        let concrete_names = resolve(portable_registry.types.iter()).unwrap();

        // Build user-generic names via new API; no types are filtered out in this test
        let filter_out_types: HashSet<u32> = HashSet::new();
        let generic_names = resolve_user_generic_type_names(
            portable_registry.types.iter(),
            &concrete_names,
            &filter_out_types,
        )
        .unwrap();

        // Check main types
        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "SimpleOneGenericStruct<u32>"
        );
        let struct_generic = generic_names
            .get(&struct_id)
            .expect("struct generic must exist");
        assert_eq!(struct_generic.type_name(), "SimpleOneGenericStruct<T>");

        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "SimpleOneGenericEnum<u32>"
        );
        let enum_generic = generic_names
            .get(&enum_id)
            .expect("enum generic must exist");
        assert_eq!(enum_generic.type_name(), "SimpleOneGenericEnum<T>");

        // For structs: check that expected generic field strings are present
        let s_fields = struct_generic.fields_type_names();

        let expect_struct_fields_type_names = vec![
            "u32",
            "GenericlessUnitStruct",
            "GenericlessTupleStruct",
            "GenericlessNamedStruct",
            "GenericlessEnum",
            "GenericlessVariantlessEnum",
            "T",
            "(String, T, T, u32)",
            "Option<T>",
            "Result<T, String>",
            "SailsInterimNameBTreeMap<String, T>",
            "GenericStruct<T>",
            "SimpleOneGenericEnum<T>",
            "Option<Option<T>>",
            "Result<Option<T>, String>",
            "SailsInterimNameBTreeMap<Option<T>, GenericStruct<T>>",
            "GenericStruct<Option<T>>",
            "SimpleOneGenericEnum<Result<T, String>>",
            "Option<Option<Option<T>>>",
            "Result<Option<Result<T, String>>, String>",
            "SailsInterimNameBTreeMap<Option<GenericStruct<T>>, Result<T, String>>",
            "GenericStruct<Option<Result<T, String>>>",
        ];

        for expected in expect_struct_fields_type_names {
            assert!(
                s_fields.contains(&expected),
                "struct {} missing generic field signature {}. All fields: {:#?}",
                struct_generic.type_name(),
                expected,
                s_fields
            );
        }

        // For enums: check the collected `fields` contains expected signatures and variant names
        let e_fields = &enum_generic.fields_type_names();

        // First let's check no fields variants
        let RawNames::Enum { fields, .. } = enum_generic else {
            panic!("Expected enum generic name");
        };

        let no_fields_variant = &fields[0];
        let no_fields2_variant = &fields[fields.len() - 1];

        assert_eq!(no_fields_variant.own_name, "NoFields");
        assert_eq!(no_fields2_variant.own_name, "NoFields2");
        assert!(no_fields_variant.fields.is_empty());
        assert!(no_fields2_variant.fields.is_empty());

        // expected generic strings for enum fields and nested types:
        let expect_enum_field_type_names = vec![
            "T",
            "String",
            "T",
            "T",
            "u32",
            "Option<T>",
            "Result<T, String>",
            "SailsInterimNameBTreeMap<String, T>",
            "GenericStruct<T>",
            "NestedGenericEnum<T>",
            "Option<Option<T>>",
            "Result<Option<T>, String>",
            "SailsInterimNameBTreeMap<Option<T>, GenericStruct<T>>",
            "GenericStruct<Option<T>>",
            "Option<Option<Option<T>>>",
            "Result<Option<Result<T, String>>, String>",
        ];

        for expected in expect_enum_field_type_names {
            assert!(
                e_fields.contains(&expected),
                "enum {} missing generic field signature {}. All enum fields/entries: {:#?}",
                enum_generic.type_name(),
                expected,
                e_fields
            );
        }

        // Also verify concrete_names for some representative fields to keep parity with original test spirit
        // Retrieve struct type to check underlying field concrete ids
        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();

        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            let generic_value = composite
                .fields
                .iter()
                .find(|f| f.name == Some("generic_value"))
                .unwrap();
            assert_eq!(concrete_names.get(&generic_value.ty.id).unwrap(), "u32");

            let tuple_generic = composite
                .fields
                .iter()
                .find(|f| f.name == Some("tuple_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&tuple_generic.ty.id).unwrap(),
                "(String, u32, u32, u32)"
            );

            let option_generic = composite
                .fields
                .iter()
                .find(|f| f.name == Some("option_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&option_generic.ty.id).unwrap(),
                "Option<u32>"
            );

            let btreemap_generic = composite
                .fields
                .iter()
                .find(|f| f.name == Some("btreemap_generic"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&btreemap_generic.ty.id).unwrap(),
                "SailsInterimNameBTreeMap<String, u32>"
            );
        } else {
            panic!("Expected composite type");
        }

        let genericless_unit = generic_names.get(&genericless_unit_id).unwrap();
        let RawNames::Struct(fields) = genericless_unit else {
            panic!("Expected struct generic name");
        };
        assert_eq!(fields.own_name.as_str(), "GenericlessUnitStruct");
        assert!(fields.fields.is_empty());

        let genericless_tuple = generic_names.get(&genericless_tuple_id).unwrap();
        let RawNames::Struct(fields) = genericless_tuple else {
            panic!("Expected struct generic name");
        };
        assert_eq!(fields.own_name.as_str(), "GenericlessTupleStruct");
        let expected_fields_value = vec![
            TypeField {
                name: None,
                type_name: "u32".to_string(),
            },
            TypeField {
                name: None,
                type_name: "String".to_string(),
            },
        ];
        assert_eq!(fields.fields, expected_fields_value);

        let genericless_named = generic_names.get(&genericless_named_id).unwrap();
        let RawNames::Struct(fields) = genericless_named else {
            panic!("Expected struct generic name");
        };
        assert_eq!(fields.own_name.as_str(), "GenericlessNamedStruct");
        let expected_fields_value = vec![
            TypeField {
                name: Some("a".to_string()),
                type_name: "u32".to_string(),
            },
            TypeField {
                name: Some("b".to_string()),
                type_name: "String".to_string(),
            },
        ];
        assert_eq!(fields.fields, expected_fields_value);

        let genericless_enum = generic_names.get(&genericless_enum_id).unwrap();
        let RawNames::Enum {
            own_name, fields, ..
        } = genericless_enum
        else {
            panic!("Expected enum generic name");
        };
        assert_eq!(own_name.as_str(), "GenericlessEnum");
        let expected_variants = vec![
            TypeFieldsOwner {
                own_name: "Unit".to_string(),
                fields: vec![],
            },
            TypeFieldsOwner {
                own_name: "Tuple".to_string(),
                fields: vec![
                    TypeField {
                        name: None,
                        type_name: "u32".to_string(),
                    },
                    TypeField {
                        name: None,
                        type_name: "String".to_string(),
                    },
                ],
            },
            TypeFieldsOwner {
                own_name: "Named".to_string(),
                fields: vec![
                    TypeField {
                        name: Some("a".to_string()),
                        type_name: "u32".to_string(),
                    },
                    TypeField {
                        name: Some("b".to_string()),
                        type_name: "String".to_string(),
                    },
                ],
            },
        ];
        assert_eq!(fields, &expected_variants);

        let genericless_variantless_enum =
            generic_names.get(&genericless_variantless_enum_id).unwrap();
        let RawNames::Enum {
            own_name, fields, ..
        } = genericless_variantless_enum
        else {
            panic!("Expected enum generic name");
        };
        assert_eq!(own_name.as_str(), "GenericlessVariantlessEnum");
        assert!(fields.is_empty());
    }

    #[test]
    fn complex_cases_one_generic() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct ComplexOneGenericStruct<T> {
            array_of_generic: [T; 10],
            tuple_complex: (T, Vec<T>, [T; 5]),
            array_of_tuple: [(T, T); 3],
            vec_of_array: Vec<[T; 8]>,

            array_of_option: [Option<T>; 5],
            tuple_of_result: (result::Result<T, String>, Option<T>),
            vec_of_struct: Vec<GenericStruct<T>>,
            array_of_btreemap: [BTreeMap<String, T>; 2],

            array_of_vec_of_option: [Vec<Option<T>>; 4],
            tuple_triple: (Option<Vec<T>>, result::Result<[T; 3], String>),
            vec_of_struct_of_option: Vec<GenericStruct<Option<T>>>,
            array_complex_triple: [BTreeMap<Option<T>, result::Result<T, String>>; 2],
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        #[allow(clippy::type_complexity)]
        enum ComplexOneGenericEnum<T> {
            ArrayOfGeneric([T; 10]),
            TupleComplex(T, Vec<T>, [T; 5]),
            ArrayOfTuple([(T, T); 3]),
            VecOfArray {
                vec: Vec<[T; 8]>,
            },

            ArrayOfOption([Option<T>; 5]),
            TupleOfResult {
                tuple: (result::Result<T, String>, Option<T>),
            },
            VecOfStruct(Vec<GenericStruct<T>>),
            ArrayOfBTreeMap {
                array: [BTreeMap<String, T>; 2],
            },

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

        // Register types
        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<ComplexOneGenericStruct<bool>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<ComplexOneGenericEnum<bool>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);

        // Compute concrete and generic type names
        let concrete_names = resolve(portable_registry.types.iter()).unwrap();
        let filter_out_types: HashSet<u32> = HashSet::new();
        let generic_names = resolve_user_generic_type_names(
            portable_registry.types.iter(),
            &concrete_names,
            &filter_out_types,
        )
        .unwrap();

        // Check top level resolved names
        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "ComplexOneGenericStruct<bool>"
        );
        let struct_generic = generic_names.get(&struct_id).unwrap();
        assert_eq!(struct_generic.type_name(), "ComplexOneGenericStruct<T>");

        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "ComplexOneGenericEnum<bool>"
        );
        let enum_generic = generic_names.get(&enum_id).unwrap();
        assert_eq!(enum_generic.type_name(), "ComplexOneGenericEnum<T>");

        // Validate Struct generics
        let struct_field_types = struct_generic.fields_type_names();
        let expect_struct_field_types = vec![
            "[T; 10]",
            "(T, SailsInterimNameVec<T>, [T; 5])",
            "[(T, T); 3]",
            "SailsInterimNameVec<[T; 8]>",
            "[Option<T>; 5]",
            "(Result<T, String>, Option<T>)",
            "SailsInterimNameVec<GenericStruct<T>>",
            "[SailsInterimNameBTreeMap<String, T>; 2]",
            "[SailsInterimNameVec<Option<T>>; 4]",
            "(Option<SailsInterimNameVec<T>>, Result<[T; 3], String>)",
            "SailsInterimNameVec<GenericStruct<Option<T>>>",
            "[SailsInterimNameBTreeMap<Option<T>, Result<T, String>>; 2]",
        ];

        for expected in expect_struct_field_types {
            assert!(
                struct_field_types.contains(&expected),
                "Struct {} missing field type {}.\n All: {:#?}",
                struct_generic.type_name(),
                expected,
                struct_field_types
            );
        }

        let enum_field_types = enum_generic.fields_type_names();
        let expect_enum_field_types = vec![
            "[T; 10]",
            "T",
            "SailsInterimNameVec<T>",
            "[T; 5]",
            "[(T, T); 3]",
            "SailsInterimNameVec<[T; 8]>",
            "[Option<T>; 5]",
            "(Result<T, String>, Option<T>)",
            "SailsInterimNameVec<GenericStruct<T>>",
            "[SailsInterimNameBTreeMap<String, T>; 2]",
            "[SailsInterimNameVec<Option<SailsInterimNameVec<T>>>; 4]",
            "Option<Option<SailsInterimNameVec<T>>>",
            "Result<Option<[T; 3]>, String>",
            "SailsInterimNameVec<GenericStruct<Option<T>>>",
            "[SailsInterimNameBTreeMap<SailsInterimNameBTreeMap<Option<T>, String>, Result<T, String>>; 2]",
        ];

        for expected in expect_enum_field_types {
            assert!(
                enum_field_types.contains(&expected),
                "Enum {} missing field type {}.\n All: {:#?}",
                enum_generic.type_name(),
                expected,
                enum_field_types
            );
        }
    }

    #[test]
    fn multiple_generics() {
        fn find_field_struct<'a>(
            composite: &'a TypeDefComposite<PortableForm>,
            name: &str,
        ) -> &'a Field<PortableForm> {
            composite
                .fields
                .iter()
                .find(|f| f.name == Some(name))
                .unwrap_or_else(|| {
                    panic!(
                        "Field `{}` not found. Fields: {:#?}",
                        name, composite.fields
                    )
                })
        }

        fn find_field_enum<'a>(
            variants: &'a [Variant<PortableForm>],
            name: &str,
        ) -> &'a Variant<PortableForm> {
            variants
                .iter()
                .find(|v| v.name == name)
                .unwrap_or_else(|| panic!("Variant `{name}` not found. Variants: {variants:#?}"))
        }

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
            TupleT2T3((T2, T3)),
            VecT3 {
                vec: Vec<T3>,
            },

            // Category 2: Mixed generics in complex types
            TupleMixed(T1, T2, T3),
            TupleRepeated((T1, T1, T2, T2, T3, T3)),
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

        // Register types and build portable registry
        let mut registry = Registry::new();
        let struct_id = registry
            .register_type(&MetaType::new::<MultiGenStruct<u32, String, H256>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<MultiGenEnum<u32, String, H256>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);

        let concrete_names = resolve(portable_registry.types.iter()).unwrap();
        let filter_out_types: HashSet<u32> = HashSet::new();
        let generic_names = resolve_user_generic_type_names(
            portable_registry.types.iter(),
            &concrete_names,
            &filter_out_types,
        )
        .unwrap();

        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "MultiGenStruct<u32, String, H256>"
        );
        let struct_generic = generic_names
            .get(&struct_id)
            .expect("struct generic must exist");
        assert_eq!(struct_generic.type_name(), "MultiGenStruct<T1, T2, T3>");

        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "MultiGenEnum<u32, String, H256>"
        );
        let enum_generic = generic_names
            .get(&enum_id)
            .expect("enum generic must exist");
        assert_eq!(enum_generic.type_name(), "MultiGenEnum<T1, T2, T3>");

        let struct_field_types = struct_generic.fields_type_names();
        let expect_struct_field_types = vec![
            "T1",
            "T2",
            "T3",
            "[T1; 8]",
            "(T2, T3)",
            "SailsInterimNameVec<T3>",
            "(T1, T2, T3)",
            "(T1, T1, T2, T2, T3, T3)",
            "[(T1, T2); 4]",
            "SailsInterimNameVec<[T3; 5]>",
            "SailsInterimNameBTreeMap<T1, T2>",
            "GenericStruct<T3>",
            "GenericEnum<T1, T2>",
            "Option<Result<T1, T2>>",
            "[Option<T2>; 6]",
            "SailsInterimNameVec<(T2, T3, T1)>",
            "(Result<T1, String>, Option<T2>)",
            "SailsInterimNameBTreeMap<Option<T1>, Result<T2, String>>",
            "GenericStruct<(T2, T3)>",
            "Option<Result<SailsInterimNameVec<T1>, T2>>",
            "[SailsInterimNameBTreeMap<T1, Option<T2>>; 3]",
            "SailsInterimNameVec<GenericStruct<Option<T3>>>",
            "[SailsInterimNameVec<(T1, T2)>; 2]",
            "(Option<SailsInterimNameVec<T1>>, Result<[T2; 4], T3>)",
            "SailsInterimNameVec<GenericStruct<Result<T1, T2>>>",
        ];

        for e in expect_struct_field_types {
            assert!(
                struct_field_types.contains(&e),
                "{} missing expected type signature `{}`. All entries: {:#?}",
                struct_generic.type_name(),
                e,
                struct_field_types
            );
        }

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();
        if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
            // quick concrete checks for representative fields
            let just_t1 = find_field_struct(composite, "just_t1");
            assert_eq!(concrete_names.get(&just_t1.ty.id).unwrap(), "u32");

            let tuple_t2_t3 = find_field_struct(composite, "tuple_t2_t3");
            assert_eq!(
                concrete_names.get(&tuple_t2_t3.ty.id).unwrap(),
                "(String, H256)"
            );

            let vec_t3 = find_field_struct(composite, "vec_t3");
            assert_eq!(concrete_names.get(&vec_t3.ty.id).unwrap(), "SailsInterimNameVec<H256>");

            let array_triple = find_field_struct(composite, "array_triple");
            assert_eq!(
                concrete_names.get(&array_triple.ty.id).unwrap(),
                "[SailsInterimNameBTreeMap<u32, Option<String>>; 3]"
            );
        } else {
            panic!("Expected composite type");
        }

        let enum_field_types = enum_generic.fields_type_names();
        let expect_enum_field_types = vec![
            "T1",
            "T2",
            "T3",
            "[T1; 8]",
            "(T2, T3)",
            "SailsInterimNameVec<T3>",
            "T1",
            "T2",
            "T3",
            "(T1, T1, T2, T2, T3, T3)",
            "[(T1, T2); 4]",
            "SailsInterimNameVec<[T3; 5]>",
            "SailsInterimNameBTreeMap<T1, T2>",
            "GenericStruct<T3>",
            "GenericEnum<T1, T2>",
            "Option<Result<T1, T2>>",
            "[Option<T2>; 6]",
            "SailsInterimNameVec<(T2, T3, T1)>",
            "Result<T1, String>",
            "Option<T2>",
            "SailsInterimNameBTreeMap<Option<T1>, Result<T2, String>>",
            "GenericStruct<(T2, T3)>",
            "Option<Result<SailsInterimNameVec<T1>, T2>>",
            "[SailsInterimNameBTreeMap<T1, Option<T2>>; 3]",
            "SailsInterimNameVec<GenericStruct<Option<T3>>>",
            "[SailsInterimNameVec<(T1, T2)>; 2]",
            "Option<SailsInterimNameVec<T1>>",
            "Result<[T2; 4], T3>",
            "SailsInterimNameVec<GenericStruct<Result<T1, T2>>>",
        ];

        for e in expect_enum_field_types {
            assert!(
                enum_field_types.contains(&e),
                "{} missing expected type signature `{}`. All entries: {:#?}",
                enum_generic.type_name(),
                e,
                enum_field_types
            );
        }

        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_id)
            .unwrap();
        if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
            // check a representative tuple-like variant concrete names
            let tuple_t2_t3_variant = find_field_enum(&variant.variants, "TupleT2T3");
            let f0 = &tuple_t2_t3_variant.fields[0];
            assert_eq!(concrete_names.get(&f0.ty.id).unwrap(), "(String, H256)");

            // check option/result shaped variant
            let tuple_of_result_variant = find_field_enum(&variant.variants, "TupleOfResult");
            let field1 = tuple_of_result_variant
                .fields
                .iter()
                .find(|f| f.name == Some("field1"))
                .unwrap();
            assert_eq!(
                concrete_names.get(&field1.ty.id).unwrap(),
                "Result<u32, String>"
            );
        } else {
            panic!("Expected variant type");
        }
    }

    // todo [sab]
    //     #[test]
    //     fn generic_const_with_generic_types() {
    //         #[allow(dead_code)]
    //         #[derive(TypeInfo)]
    //         struct ConstGenericStruct<const N: usize, T> {
    //             array: [T; N],
    //             value: T,
    //             vec: Vec<T>,
    //             option: Option<T>,
    //         }

    //         #[allow(dead_code)]
    //         #[derive(TypeInfo)]
    //         struct TwoConstGenericStruct<const N: usize, const M: usize, T1, T2> {
    //             array1: [T1; N],
    //             array2: [T2; M],
    //             tuple: (T1, T2),
    //             nested: GenericStruct<T1>,
    //             result: result::Result<T1, T2>,
    //         }

    //         #[allow(dead_code)]
    //         #[derive(TypeInfo)]
    //         enum ConstGenericEnum<const N: usize, T> {
    //             Array([T; N]),
    //             Value(T),
    //             Nested { inner: GenericStruct<T> },
    //         }

    //         let mut registry = Registry::new();

    //         // Register ConstGenericStruct with different N and T values
    //         let struct_n8_u32_id = registry
    //             .register_type(&MetaType::new::<ConstGenericStruct<8, u32>>())
    //             .id;
    //         let struct_n8_string_id = registry
    //             .register_type(&MetaType::new::<ConstGenericStruct<8, String>>())
    //             .id;

    //         let struct_n16_u32_id = registry
    //             .register_type(&MetaType::new::<ConstGenericStruct<16, u32>>())
    //             .id;

    //         assert_ne!(struct_n8_u32_id, struct_n8_string_id);
    //         assert_ne!(struct_n8_u32_id, struct_n16_u32_id);

    //         // Register TwoConstGenericStruct
    //         let two_const_id = registry
    //             .register_type(&MetaType::new::<TwoConstGenericStruct<4, 8, u64, H256>>())
    //             .id;

    //         // Register ConstGenericEnum
    //         let enum_n8_bool_id = registry
    //             .register_type(&MetaType::new::<ConstGenericEnum<8, bool>>())
    //             .id;

    //         let portable_registry = PortableRegistry::from(registry);
    //         let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

    //         // Check ConstGenericStruct with N=8, T=u32
    //         assert_eq!(
    //             concrete_names.get(&struct_n8_u32_id).unwrap(),
    //             "ConstGenericStruct1<u32>"
    //         );
    //         assert_eq!(
    //             concrete_names.get(&struct_n8_string_id).unwrap(),
    //             "ConstGenericStruct<String>"
    //         );
    //         assert_eq!(
    //             concrete_names.get(&struct_n16_u32_id).unwrap(),
    //             "ConstGenericStruct2<u32>"
    //         );
    //         assert_eq!(
    //             concrete_names.get(&two_const_id).unwrap(),
    //             "TwoConstGenericStruct<u64, H256>"
    //         );
    //         assert_eq!(
    //             concrete_names.get(&enum_n8_bool_id).unwrap(),
    //             "ConstGenericEnum<bool>"
    //         );

    //         assert_eq!(
    //             generic_names
    //                 .get(&FieldValue::ParentTy(struct_n8_u32_id))
    //                 .unwrap()
    //                 .as_deref(),
    //             Some("ConstGenericStruct1<T>")
    //         );
    //         assert_eq!(
    //             generic_names
    //                 .get(&FieldValue::ParentTy(two_const_id))
    //                 .unwrap()
    //                 .as_deref(),
    //             Some("TwoConstGenericStruct<T1, T2>")
    //         );
    //         assert_eq!(
    //             generic_names
    //                 .get(&FieldValue::ParentTy(enum_n8_bool_id))
    //                 .unwrap()
    //                 .as_deref(),
    //             Some("ConstGenericEnum<T>")
    //         );

    //         let struct_type = portable_registry
    //             .types
    //             .iter()
    //             .find(|t| t.id == struct_n8_u32_id)
    //             .unwrap();
    //         if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
    //             let array = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("array"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&array.ty.id).unwrap(), "[u32; 8]");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_n8_u32_id,
    //                 field_index: 0,
    //                 self_id: array.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("[T; 8]"));

    //             let value = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("value"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&value.ty.id).unwrap(), "u32");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_n8_u32_id,
    //                 field_index: 1,
    //                 self_id: value.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("T"));

    //             let vec = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("vec"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&vec.ty.id).unwrap(), "SailsInterimNameVec<u32>");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_n8_u32_id,
    //                 field_index: 2,
    //                 self_id: vec.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("SailsInterimNameVec<T>")
    //             );

    //             let option = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("option"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&option.ty.id).unwrap(), "Option<u32>");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_n8_u32_id,
    //                 field_index: 3,
    //                 self_id: option.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("Option<T>")
    //             );
    //         }

    //         let two_const_type = portable_registry
    //             .types
    //             .iter()
    //             .find(|t| t.id == two_const_id)
    //             .unwrap();
    //         if let TypeDef::Composite(composite) = &two_const_type.ty.type_def {
    //             let array1 = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("array1"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&array1.ty.id).unwrap(), "[u64; 4]");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: two_const_id,
    //                 field_index: 0,
    //                 self_id: array1.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("[T1; 4]"));

    //             let array2 = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("array2"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&array2.ty.id).unwrap(), "[H256; 8]");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: two_const_id,
    //                 field_index: 1,
    //                 self_id: array2.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("[T2; 8]"));

    //             let tuple = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("tuple"))
    //                 .unwrap();
    //             assert_eq!(concrete_names.get(&tuple.ty.id).unwrap(), "(u64, H256)");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: two_const_id,
    //                 field_index: 2,
    //                 self_id: tuple.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("(T1, T2)"));

    //             let nested = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("nested"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&nested.ty.id).unwrap(),
    //                 "GenericStruct<u64>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: two_const_id,
    //                 field_index: 3,
    //                 self_id: nested.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("GenericStruct<T1>")
    //             );

    //             let result = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("result"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&result.ty.id).unwrap(),
    //                 "Result<u64, H256>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: two_const_id,
    //                 field_index: 4,
    //                 self_id: result.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("Result<T1, T2>")
    //             );
    //         }

    //         let enum_type = portable_registry
    //             .types
    //             .iter()
    //             .find(|t| t.id == enum_n8_bool_id)
    //             .unwrap();
    //         if let TypeDef::Variant(variant) = &enum_type.ty.type_def {
    //             let array_variant = variant.variants.iter().find(|v| v.name == "Array").unwrap();
    //             let field = &array_variant.fields[0];
    //             assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "[bool; 8]");
    //             let id = FieldValue::EnumFields {
    //                 parent_ty: enum_n8_bool_id,
    //                 variant_index: 0,
    //                 field_index: 0,
    //                 self_id: field.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("[T; 8]"));

    //             let value_variant = variant.variants.iter().find(|v| v.name == "Value").unwrap();
    //             let field = &value_variant.fields[0];
    //             assert_eq!(concrete_names.get(&field.ty.id).unwrap(), "bool");
    //             let id = FieldValue::EnumFields {
    //                 parent_ty: enum_n8_bool_id,
    //                 variant_index: 1,
    //                 field_index: 0,
    //                 self_id: field.ty.id,
    //             };
    //             assert_eq!(generic_names.get(&id).unwrap().as_deref(), Some("T"));

    //             let nested_variant = variant
    //                 .variants
    //                 .iter()
    //                 .find(|v| v.name == "Nested")
    //                 .unwrap();
    //             let field = nested_variant
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("inner"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&field.ty.id).unwrap(),
    //                 "GenericStruct<bool>"
    //             );
    //             let id = FieldValue::EnumFields {
    //                 parent_ty: enum_n8_bool_id,
    //                 variant_index: 2,
    //                 field_index: 0,
    //                 self_id: field.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("GenericStruct<T>")
    //             );
    //         }
    //     }

    //     // Types for same_name_different_modules test
    //     #[allow(dead_code)]
    //     mod same_name_test {
    //         use super::*;

    //         pub mod module_a {
    //             use super::*;

    //             #[derive(TypeInfo)]
    //             pub struct SameName<T> {
    //                 pub value: T,
    //             }
    //         }

    //         pub mod module_b {
    //             use super::*;

    //             #[derive(TypeInfo)]
    //             pub struct SameName<T> {
    //                 pub value: T,
    //             }
    //         }

    //         pub mod module_c {
    //             use super::*;

    //             pub mod nested {
    //                 use super::*;

    //                 #[derive(TypeInfo)]
    //                 pub struct SameName<T> {
    //                     pub value: T,
    //                 }
    //             }
    //         }
    //     }

    //     #[test]
    //     fn same_name_different_mods_generic_names() {
    //         use same_name_test::*;

    //         #[allow(dead_code)]
    //         #[derive(TypeInfo)]
    //         struct TestStruct<T1, T2> {
    //             field_a: module_a::SameName<T1>,
    //             field_b: module_b::SameName<T2>,
    //             field_c: module_c::nested::SameName<T1>,
    //             generic_a: GenericStruct<module_a::SameName<T2>>,
    //             generic_b: GenericStruct<module_b::SameName<T1>>,
    //             vec_a: Vec<module_c::nested::SameName<T1>>,
    //             option_b: Option<module_b::SameName<T2>>,
    //             result_mix: result::Result<module_a::SameName<T1>, module_b::SameName<T2>>,
    //         }

    //         let mut registry = Registry::new();
    //         let struct_id = registry
    //             .register_type(&MetaType::new::<TestStruct<u32, bool>>())
    //             .id;

    //         let portable_registry = PortableRegistry::from(registry);
    //         let (concrete_names, generic_names) = resolve(portable_registry.types.iter()).unwrap();

    //         // Check main type
    //         assert_eq!(
    //             concrete_names.get(&struct_id).unwrap(),
    //             "TestStruct<u32, bool>"
    //         );
    //         assert_eq!(
    //             generic_names
    //                 .get(&FieldValue::ParentTy(struct_id))
    //                 .unwrap()
    //                 .as_deref(),
    //             Some("TestStruct<T1, T2>")
    //         );

    //         let struct_type = portable_registry
    //             .types
    //             .iter()
    //             .find(|t| t.id == struct_id)
    //             .unwrap();
    //         if let TypeDef::Composite(composite) = &struct_type.ty.type_def {
    //             // field_a: module_a::SameName<T1>
    //             let field_a = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("field_a"))
    //                 .unwrap();
    //             let name_a = concrete_names.get(&field_a.ty.id).unwrap();
    //             assert_eq!(name_a, "ModuleASameName<u32>");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 0,
    //                 self_id: field_a.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("ModuleASameName<T1>")
    //             );

    //             // field_b: module_b::SameName<T2>
    //             let field_b = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("field_b"))
    //                 .unwrap();
    //             let name_b = concrete_names.get(&field_b.ty.id).unwrap();
    //             assert_eq!(name_b, "ModuleBSameName<bool>");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 1,
    //                 self_id: field_b.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("ModuleBSameName<T2>")
    //             );

    //             // field_c: module_c::nested::SameName<T1>
    //             let field_c = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("field_c"))
    //                 .unwrap();
    //             let name_c = concrete_names.get(&field_c.ty.id).unwrap();
    //             assert_eq!(name_c, "NestedSameName<u32>");
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 2,
    //                 self_id: field_c.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("NestedSameName<T1>")
    //             );

    //             // Verify names are different
    //             assert_ne!(name_a, name_b);
    //             assert_ne!(name_a, name_c);
    //             assert_ne!(name_b, name_c);

    //             // generic_a: GenericStruct<module_a::SameName<T2>>
    //             let generic_a = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("generic_a"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&generic_a.ty.id).unwrap(),
    //                 "GenericStruct<ModuleASameName<bool>>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 3,
    //                 self_id: generic_a.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("GenericStruct<ModuleASameName<T2>>")
    //             );

    //             // generic_b: GenericStruct<module_b::SameName<T1>>
    //             let generic_b = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("generic_b"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&generic_b.ty.id).unwrap(),
    //                 "GenericStruct<ModuleBSameName<u32>>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 4,
    //                 self_id: generic_b.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("GenericStruct<ModuleBSameName<T1>>")
    //             );

    //             // vec_a: Vec<module_c::nested::SameName<T1>>
    //             let vec_a = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("vec_a"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&vec_a.ty.id).unwrap(),
    //                 "SailsInterimNameVec<NestedSameName<u32>>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 5,
    //                 self_id: vec_a.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("SailsInterimNameVec<NestedSameName<T1>>")
    //             );

    //             // option_b: Option<module_b::SameName<T2>>
    //             let option_b = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("option_b"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&option_b.ty.id).unwrap(),
    //                 "Option<ModuleBSameName<bool>>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 6,
    //                 self_id: option_b.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("Option<ModuleBSameName<T2>>")
    //             );

    //             // result_mix: result::Result<module_a::SameName<T1>, module_b::SameName<T2>>
    //             let result_mix = composite
    //                 .fields
    //                 .iter()
    //                 .find(|f| f.name.as_deref() == Some("result_mix"))
    //                 .unwrap();
    //             assert_eq!(
    //                 concrete_names.get(&result_mix.ty.id).unwrap(),
    //                 "Result<ModuleASameName<u32>, ModuleBSameName<bool>>"
    //             );
    //             let id = FieldValue::StructFields {
    //                 parent_ty: struct_id,
    //                 field_index: 7,
    //                 self_id: result_mix.ty.id,
    //             };
    //             assert_eq!(
    //                 generic_names.get(&id).unwrap().as_deref(),
    //                 Some("Result<ModuleASameName<T1>, ModuleBSameName<T2>>")
    //             );
    //         }
    //     }

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
            a3r: ReusableGenericStruct<(u64, H256)>,

            b1: ReusableGenericStruct<T2>,
            b1r: ReusableGenericStruct<H256>,

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
                field: ReusableGenericStruct<(u64, H256)>,
            },

            B1(ReusableGenericStruct<T2>),
            B1r(ReusableGenericStruct<H256>),

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

        let concrete_names = resolve(portable_registry.types.iter()).unwrap();
        let filter_out_types: HashSet<u32> = HashSet::new();
        let generic_names = resolve_user_generic_type_names(
            portable_registry.types.iter(),
            &concrete_names,
            &filter_out_types,
        )
        .unwrap();

        assert_eq!(
            concrete_names.get(&struct_id).unwrap(),
            "ReuseTestStruct<u64, H256>"
        );

        let struct_generic = generic_names.get(&struct_id).unwrap();
        assert_eq!(struct_generic.type_name(), "ReuseTestStruct<T1, T2>",);

        assert_eq!(
            concrete_names.get(&enum_id).unwrap(),
            "ReuseTestEnum<u64, H256>"
        );
        let enum_generic = generic_names.get(&enum_id).unwrap();
        assert_eq!(enum_generic.type_name(), "ReuseTestEnum<T1, T2>");

        let struct_field_types = struct_generic.fields_type_names();
        let expect_struct_field_types = vec![
            "ReusableGenericStruct<T1>",
            "ReusableGenericStruct<CodeId>",
            "ReusableGenericStruct<SailsInterimNameVec<T1>>",
            "ReusableGenericStruct<SailsInterimNameVec<bool>>",
            "ReusableGenericStruct<(T1, T2)>",
            "ReusableGenericStruct<(u64, H256)>",
            "ReusableGenericStruct<T2>",
            "ReusableGenericStruct<H256>",
            "ReusableGenericEnum<T1>",
            "ReusableGenericEnum<CodeId>",
            "ReusableGenericEnum<T2>",
            "ReusableGenericEnum<bool>",
            "ReusableGenericEnum<String>",
            "ReusableGenericEnum<[T1; 8]>",
            "GenericStruct<ReusableGenericStruct<T1>>",
            "GenericStruct<ReusableGenericStruct<T2>>",
            "GenericStruct<ReusableGenericStruct<u32>>",
            "SailsInterimNameVec<ReusableGenericStruct<T1>>",
            "[ReusableGenericEnum<T2>; 5]",
            "Option<ReusableGenericStruct<(T1, T2)>>",
            "Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>",
            "SailsInterimNameBTreeMap<T1, ReusableGenericStruct<T2>>",
            "SailsInterimNameBTreeMap<ReusableGenericEnum<T1>, String>",
            "SailsInterimNameBTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>",
            "SailsInterimNameBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>",
        ];

        for e in expect_struct_field_types {
            assert!(
                struct_field_types.contains(&e),
                "{} missing expected type signature `{}`. All entries: {:#?}",
                struct_generic.type_name(),
                e,
                struct_field_types
            );
        }

        let enum_field_types = enum_generic.fields_type_names();
        let expect_enum_field_types = vec![
            "ReusableGenericStruct<T1>",
            "ReusableGenericStruct<CodeId>",
            "ReusableGenericStruct<SailsInterimNameVec<T1>>",
            "ReusableGenericStruct<SailsInterimNameVec<bool>>",
            "ReusableGenericStruct<(T1, T2)>",
            "ReusableGenericStruct<(u64, H256)>",
            "ReusableGenericStruct<T2>",
            "ReusableGenericStruct<H256>",
            "ReusableGenericEnum<T1>",
            "ReusableGenericEnum<CodeId>",
            "ReusableGenericEnum<T2>",
            "ReusableGenericEnum<bool>",
            "ReusableGenericEnum<String>",
            "ReusableGenericEnum<[T1; 8]>",
            "GenericStruct<ReusableGenericStruct<T1>>",
            "GenericStruct<ReusableGenericStruct<T2>>",
            "GenericStruct<ReusableGenericStruct<u32>>",
            "SailsInterimNameVec<ReusableGenericStruct<T1>>",
            "[ReusableGenericEnum<T2>; 5]",
            "Option<ReusableGenericStruct<(T1, T2)>>",
            "Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>",
            "SailsInterimNameBTreeMap<T1, ReusableGenericStruct<T2>>",
            "SailsInterimNameBTreeMap<ReusableGenericEnum<T1>, String>",
            "SailsInterimNameBTreeMap<ReusableGenericStruct<T1>, ReusableGenericEnum<T2>>",
            "SailsInterimNameBTreeMap<ReusableGenericStruct<u64>, ReusableGenericEnum<H256>>",
        ];

        for e in expect_enum_field_types {
            assert!(
                enum_field_types.contains(&e),
                "{} missing expected type signature `{}`. All entries: {:#?}",
                enum_generic.type_name(),
                e,
                enum_field_types
            );
        }
    }
}
