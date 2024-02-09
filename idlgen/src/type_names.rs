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
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

pub(super) fn resolve_type_names(
    type_registry: &PortableRegistry,
) -> Result<BTreeMap<u32, String>> {
    let type_names = type_registry.types.iter().try_fold(
        (
            BTreeMap::<u32, RcTypeName>::new(),
            HashMap::<(String, Vec<u32>), u32>::new(),
        ),
        |mut type_names, ty| {
            resolve_type_name(type_registry, ty.id, &mut type_names.0, &mut type_names.1)
                .map(|_| type_names)
        },
    );
    type_names.map(|type_names| {
        type_names
            .0
            .iter()
            .map(|(id, name)| (*id, name.as_string(&type_names.1)))
            .collect()
    })
}

fn resolve_type_name(
    type_registry: &PortableRegistry,
    type_id: u32,
    resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
    by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
) -> Result<RcTypeName> {
    if let Some(type_name) = resolved_type_names.get(&type_id) {
        return Ok(type_name.clone());
    }

    let type_info = type_registry
        .resolve(type_id)
        .ok_or_else(|| Error::UnknownType(type_id))?;

    let type_name: RcTypeName = match &type_info.type_def {
        TypeDef::Tuple(tuple_def) => Rc::new(TupleTypeName::new(
            type_registry,
            tuple_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Sequence(vector_def) => Rc::new(VectorTypeName::new(
            type_registry,
            vector_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Array(array_def) => Rc::new(ArrayTypeName::new(
            type_registry,
            array_def,
            resolved_type_names,
            by_path_type_names,
        )?),
        TypeDef::Composite(_) => {
            if BTreeMapTypeName::is_btree_map_type(type_info) {
                Rc::new(BTreeMapTypeName::new(
                    type_registry,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else {
                Rc::new(ByPathTypeName::new(
                    type_registry,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            }
        }
        TypeDef::Variant(_) => {
            if ResultTypeName::is_result_type(type_info) {
                Rc::new(ResultTypeName::new(
                    type_registry,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else if OptionTypeName::is_option_type(type_info) {
                Rc::new(OptionTypeName::new(
                    type_registry,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            } else {
                Rc::new(ByPathTypeName::new(
                    type_registry,
                    type_info,
                    resolved_type_names,
                    by_path_type_names,
                )?)
            }
        }
        TypeDef::Primitive(primitive_def) => Rc::new(PrimitiveTypeName::new(primitive_def)?),
        _ => {
            return Err(Error::UnsupprotedType(format!("{type_info:?}")));
        }
    };

    resolved_type_names.insert(type_id, type_name.clone());
    Ok(type_name)
}

type RcTypeName = Rc<dyn TypeName>;

trait TypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String; // Make returning &str + use OnceCell to cache the result
}

/// By path type name resolution.
struct ByPathTypeName {
    possible_names: Vec<(String, Vec<u32>)>,
    type_param_type_names: Vec<RcTypeName>,
}

impl ByPathTypeName {
    pub fn new(
        type_registry: &PortableRegistry,
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
                    .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
                    .id;
                let type_param_type_name = resolve_type_name(
                    type_registry,
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

        let possible_names =
            Self::possible_names(type_info).fold(Vec::new(), |mut possible_names, name| {
                possible_names.push((name.clone(), type_params.0.clone()));
                let name_ref_count = by_path_type_names
                    .entry((name, type_params.0.clone()))
                    .or_default();
                *name_ref_count += 1;
                possible_names
            });
        if possible_names.is_empty() {
            return Err(Error::UnsupprotedType(format!("{type_info:?}")));
        }

        Ok(Self {
            possible_names,
            type_param_type_names: type_params.1,
        })
    }

    fn possible_names(type_info: &Type<PortableForm>) -> impl Iterator<Item = String> + '_ {
        let mut name = String::default();
        type_info.path.segments.iter().rev().map(move |segment| {
            name = segment.to_case(Case::Pascal) + &name;
            name.clone()
        })
    }
}

impl TypeName for ByPathTypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        let name = self
            .possible_names
            .iter()
            .find(|possible_name| {
                by_path_type_names
                    .get(possible_name)
                    .map_or(false, |ref_count| *ref_count == 1)
            })
            .unwrap_or_else(|| self.possible_names.last().unwrap());
        if self.type_param_type_names.is_empty() {
            name.0.clone()
        } else {
            let type_param_names = self
                .type_param_type_names
                .iter()
                .map(|tn| tn.as_string(by_path_type_names).to_case(Case::Pascal))
                .collect::<Vec<_>>()
                .join("And");
            format!("{}For{}", name.0, type_param_names)
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
        type_registry: &PortableRegistry,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
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
        let key_type_name = resolve_type_name(
            type_registry,
            key_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        let value_type_name = resolve_type_name(
            type_registry,
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
        let btree_map_type_info = BTreeMap::<u32, ()>::type_info();
        btree_map_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for BTreeMapTypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        format!(
            "map ({}, {})",
            self.key_type_name.as_string(by_path_type_names),
            self.value_type_name.as_string(by_path_type_names)
        )
    }
}

/// Result type name resolution.
struct ResultTypeName {
    ok_type_name: RcTypeName,
    err_type_name: RcTypeName,
}

impl ResultTypeName {
    pub fn new(
        type_registry: &PortableRegistry,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
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
        let ok_type_name = resolve_type_name(
            type_registry,
            ok_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        let err_type_name = resolve_type_name(
            type_registry,
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
        let result_type_info = std::result::Result::<(), ()>::type_info();
        result_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for ResultTypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        format!(
            "result ({}, {})",
            self.ok_type_name.as_string(by_path_type_names),
            self.err_type_name.as_string(by_path_type_names)
        )
    }
}

/// Option type name resolution.
struct OptionTypeName {
    some_type_name: RcTypeName,
}

impl OptionTypeName {
    pub fn new(
        type_registry: &PortableRegistry,
        type_info: &Type<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let some_type_id = type_info
            .type_params
            .iter()
            .find(|param| param.name == "T")
            .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?
            .ty
            .ok_or_else(|| Error::UnsupprotedType(format!("{type_info:?}")))?;
        let some_type_name = resolve_type_name(
            type_registry,
            some_type_id.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self { some_type_name })
    }

    pub fn is_option_type(type_info: &Type<PortableForm>) -> bool {
        let option_type_info = std::option::Option::<()>::type_info();
        option_type_info.path.segments == type_info.path.segments
    }
}

impl TypeName for OptionTypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        format!("opt {}", self.some_type_name.as_string(by_path_type_names))
    }
}

/// Tuple type name resolution.
struct TupleTypeName {
    field_type_names: Vec<RcTypeName>,
}

impl TupleTypeName {
    pub fn new(
        type_registry: &PortableRegistry,
        tuple_def: &TypeDefTuple<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let field_type_names = tuple_def
            .fields
            .iter()
            .map(|field| {
                resolve_type_name(
                    type_registry,
                    field.id,
                    resolved_type_names,
                    by_path_type_names,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { field_type_names })
    }
}

impl TypeName for TupleTypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        if self.field_type_names.is_empty() {
            "null".into()
        } else {
            format!(
                "struct {{ {} }}",
                self.field_type_names
                    .iter()
                    .map(|tn| tn.as_string(by_path_type_names))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

/// Vector type name resolution.
struct VectorTypeName {
    item_type_name: RcTypeName,
}

impl VectorTypeName {
    pub fn new(
        type_registry: &PortableRegistry,
        vector_def: &TypeDefSequence<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let item_type_name = resolve_type_name(
            type_registry,
            vector_def.type_param.id,
            resolved_type_names,
            by_path_type_names,
        )?;
        Ok(Self { item_type_name })
    }
}

impl TypeName for VectorTypeName {
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        format!("vec {}", self.item_type_name.as_string(by_path_type_names))
    }
}

/// Array type name resolution.
struct ArrayTypeName {
    item_type_name: RcTypeName,
    len: u32,
}

impl ArrayTypeName {
    pub fn new(
        type_registry: &PortableRegistry,
        array_def: &TypeDefArray<PortableForm>,
        resolved_type_names: &mut BTreeMap<u32, RcTypeName>,
        by_path_type_names: &mut HashMap<(String, Vec<u32>), u32>,
    ) -> Result<Self> {
        let item_type_name = resolve_type_name(
            type_registry,
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
    fn as_string(&self, by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        format!(
            "[{}, {}]",
            self.item_type_name.as_string(by_path_type_names),
            self.len
        )
    }
}

struct PrimitiveTypeName {
    name: &'static str,
}

impl PrimitiveTypeName {
    pub fn new(type_def: &TypeDefPrimitive) -> Result<Self> {
        let name = match type_def {
            TypeDefPrimitive::Bool => Ok("bool"),
            TypeDefPrimitive::Char => Ok("char"),
            TypeDefPrimitive::Str => Ok("str"),
            TypeDefPrimitive::U8 => Ok("u8"),
            TypeDefPrimitive::U16 => Ok("u16"),
            TypeDefPrimitive::U32 => Ok("u32"),
            TypeDefPrimitive::U64 => Ok("u64"),
            TypeDefPrimitive::U128 => Ok("u128"),
            TypeDefPrimitive::U256 => Err(Error::UnsupprotedType("u256".into())), // Rust doesn't have it
            TypeDefPrimitive::I8 => Ok("i8"),
            TypeDefPrimitive::I16 => Ok("i16"),
            TypeDefPrimitive::I32 => Ok("i32"),
            TypeDefPrimitive::I64 => Ok("i64"),
            TypeDefPrimitive::I128 => Ok("i128"),
            TypeDefPrimitive::I256 => Err(Error::UnsupprotedType("i256".into())), // Rust doesn't have it
        }?;
        Ok(Self { name })
    }
}

impl TypeName for PrimitiveTypeName {
    fn as_string(&self, _by_path_type_names: &HashMap<(String, Vec<u32>), u32>) -> String {
        self.name.to_string()
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
        assert_eq!(u32_struct_name, "GenericStructForU32");

        let string_struct_name = type_names.get(&string_struct_id).unwrap();
        assert_eq!(string_struct_name, "GenericStructForStr");
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
        assert_eq!(u32_string_enum_name, "GenericEnumForU32AndStr");

        let bool_u32_enum_name = type_names.get(&bool_u32_enum_id).unwrap();
        assert_eq!(bool_u32_enum_name, "GenericEnumForBoolAndU32");
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

    #[test]
    fn type_name_minification_works_for_types_with_the_same_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry.register_type(&MetaType::new::<mod_1::T1>()).id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T1>()).id;
        let portable_registry = PortableRegistry::from(registry);

        let type_names = resolve_type_names(&portable_registry).unwrap();

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

        let type_names = resolve_type_names(&portable_registry).unwrap();

        let t1_name = type_names.get(&t1_id).unwrap();
        assert_eq!(t1_name, "Mod1Mod2T2");

        let t2_name = type_names.get(&t2_id).unwrap();
        assert_eq!(t2_name, "TestsMod2T2");
    }
}
