use sails_idl_meta::*;
use scale_info::{
    Field, PortableRegistry, PortableType, StaticTypeInfo, Type, TypeDef, TypeDefArray,
    TypeDefComposite, TypeDefPrimitive, TypeDefSequence, TypeDefTuple, TypeDefVariant, TypeInfo,
    form::PortableForm, *,
};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct TypeResolver<'a> {
    registry: &'a PortableRegistry,
    exclude: HashSet<u32>,
    map: HashMap<u32, TypeDecl>,
    user_defined: HashMap<String, UserDefinedEntry>,
}

pub struct UserDefinedEntry {
    pub ty: sails_idl_meta::Type,
    pub refs: HashSet<u32>,
}

impl<'a> TypeResolver<'a> {
    pub fn from_registry(registry: &'a PortableRegistry) -> Self {
        let mut resolver = Self {
            registry,
            exclude: HashSet::new(),
            map: HashMap::new(),
            user_defined: HashMap::new(),
        };
        resolver.build_type_decl_map();
        resolver
    }

    pub fn from(registry: &'a PortableRegistry, exclude: HashSet<u32>) -> Self {
        let mut resolver = Self {
            registry,
            exclude,
            map: HashMap::new(),
            user_defined: HashMap::new(),
        };
        resolver.build_type_decl_map();
        resolver
    }

    pub fn get(&self, key: u32) -> Option<&TypeDecl> {
        self.map.get(&key)
    }

    fn build_type_decl_map(&mut self) {
        for pt in &self.registry.types {
            let td = self.resolve_type_decl(&pt.ty);
            self.map.insert(pt.id, td);
        }
    }

    fn resolve_type_decl(&mut self, ty: &Type<PortableForm>) -> TypeDecl {
        match &ty.type_def {
            TypeDef::Composite(type_def_composite) => self
                .resolve_known_composite(ty, type_def_composite)
                .unwrap_or_else(|| self.resolve_user_defined(ty)),
            TypeDef::Variant(type_def_variant) => self
                .resolve_known_enum(ty, type_def_variant)
                .unwrap_or_else(|| self.resolve_user_defined(ty)),
            TypeDef::Sequence(type_def_sequence) => TypeDecl::Slice(Box::new(
                self.resolve_type_decl(
                    self.registry
                        .resolve(type_def_sequence.type_param.id)
                        .as_ref()
                        .unwrap(),
                ),
            )),
            TypeDef::Array(type_def_array) => TypeDecl::Array {
                item: Box::new(
                    self.resolve_type_decl(
                        self.registry
                            .resolve(type_def_array.type_param.id)
                            .as_ref()
                            .unwrap(),
                    ),
                ),
                len: type_def_array.len,
            },
            TypeDef::Tuple(type_def_tuple) => TypeDecl::Tuple(
                type_def_tuple
                    .fields
                    .iter()
                    .map(|f| self.registry.resolve(f.id).unwrap())
                    .map(|ty| self.resolve_type_decl(ty))
                    .collect(),
            ),
            TypeDef::Primitive(type_def_primitive) => {
                TypeDecl::Primitive(primitive_map(&type_def_primitive))
            }
            TypeDef::Compact(_) => unimplemented!("TypeDef::Compact is unimplemented"),
            TypeDef::BitSequence(_) => {
                unimplemented!("TypeDef::BitSequence is unimplemented")
            }
        }
    }

    fn resolve_user_defined(&mut self, ty: &Type<PortableForm>) -> TypeDecl {
        TypeDecl::UserDefined {
            path: ty.path.segments.last().unwrap().to_string(),
            generics: ty
                .type_params
                .iter()
                .map(|tp| {
                    self.resolve_type_decl(
                        self.registry
                            .resolve(tp.ty.as_ref().unwrap().id)
                            .as_ref()
                            .unwrap(),
                    )
                })
                .collect(),
        }
    }

    fn resolve_known_composite(
        &mut self,
        ty: &Type<PortableForm>,
        def: &TypeDefComposite<PortableForm>,
    ) -> Option<TypeDecl> {
        use PrimitiveType::*;
        use TypeDecl::*;

        if is_type::<gprimitives::H160>(ty) {
            Some(Primitive(H160))
        } else if is_type::<gprimitives::H256>(ty) {
            Some(Primitive(H256))
        } else if is_type::<gprimitives::U256>(ty) {
            Some(Primitive(U256))
        } else if is_type::<BTreeMap<(), ()>>(ty) {
            let key_ty = self
                .registry
                .resolve(ty.type_params[0].ty.unwrap().id)
                .unwrap();
            let key = self.resolve_type_decl(key_ty);
            let value_ty = self
                .registry
                .resolve(ty.type_params[1].ty.unwrap().id)
                .unwrap();
            let value = self.resolve_type_decl(value_ty);
            Some(Slice(Box::new(Tuple(vec![key, value]))))
        } else {
            None
        }
    }

    fn resolve_known_enum(
        &mut self,
        ty: &Type<PortableForm>,
        def: &TypeDefVariant<PortableForm>,
    ) -> Option<TypeDecl> {
        use TypeDecl::*;

        if is_type::<core::result::Result<(), ()>>(ty) {
            let ok_ty = self
                .registry
                .resolve(def.variants[0].fields[0].ty.id)
                .unwrap();
            let ok = self.resolve_type_decl(ok_ty);
            let err_ty = self
                .registry
                .resolve(def.variants[1].fields[0].ty.id)
                .unwrap();
            let err = self.resolve_type_decl(err_ty);
            Some(Result {
                ok: Box::new(ok),
                err: Box::new(err),
            })
        } else if is_type::<core::option::Option<()>>(ty) {
            let ty = self
                .registry
                .resolve(def.variants[1].fields[0].ty.id)
                .unwrap();
            let decl = self.resolve_type_decl(ty);
            Some(Option(Box::new(decl)))
        } else {
            None
        }
    }
}

fn is_type<T: StaticTypeInfo>(type_info: &Type<PortableForm>) -> bool {
    println!(
        "{:?} == {:?} : {}",
        T::type_info().path.segments,
        type_info.path.segments,
        T::type_info().path.segments == type_info.path.segments
    );
    T::type_info().path.segments == type_info.path.segments
}

fn primitive_map(type_def_primitive: &TypeDefPrimitive) -> PrimitiveType {
    use PrimitiveType::*;

    match type_def_primitive {
        TypeDefPrimitive::Bool => Bool,
        TypeDefPrimitive::Char => Char,
        TypeDefPrimitive::Str => String,
        TypeDefPrimitive::U8 => U8,
        TypeDefPrimitive::U16 => U16,
        TypeDefPrimitive::U32 => U32,
        TypeDefPrimitive::U64 => U64,
        TypeDefPrimitive::U128 => U128,
        TypeDefPrimitive::U256 => U256,
        TypeDefPrimitive::I8 => I8,
        TypeDefPrimitive::I16 => I16,
        TypeDefPrimitive::I32 => I32,
        TypeDefPrimitive::I64 => I64,
        TypeDefPrimitive::I128 => I128,
        TypeDefPrimitive::I256 => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::{MetaType, Registry, TypeInfo};

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

    #[test]
    fn type_resolver_h256() {
        let mut registry = Registry::new();
        let h256_id = registry
            .register_type(&MetaType::new::<gprimitives::H256>())
            .id;
        let h256_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<gprimitives::H256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);

        println!("{:#?}", portable_registry);
        let resolver = TypeResolver::from_registry(&portable_registry);
        println!("{:#?}", resolver.map);

        let h256_decl = resolver.get(h256_id).unwrap();
        assert_eq!(*h256_decl, TypeDecl::Primitive(PrimitiveType::H256));

        let generic_struct_decl = resolver.get(h256_as_generic_param_id).unwrap();
        assert_eq!(
            *generic_struct_decl,
            TypeDecl::UserDefined {
                path: "GenericStruct".to_string(),
                generics: vec![TypeDecl::Primitive(PrimitiveType::H256)]
            }
        );
        assert_eq!(generic_struct_decl.to_string(), "GenericStruct<H256>");
    }

    #[test]
    fn type_resolver_generic_struct() {
        let mut registry = Registry::new();
        let u32_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<u32>>())
            .id;
        let string_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<String>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);
        println!("{:#?}", resolver.map);

        let u32_struct = resolver.get(u32_struct_id).unwrap();
        assert_eq!(u32_struct.to_string(), "GenericStruct<u32>");

        let string_struct = resolver.get(string_struct_id).unwrap();
        assert_eq!(string_struct.to_string(), "GenericStruct<String>");
    }

    #[test]
    fn type_resolver_generic_enum() {
        let mut registry = Registry::new();
        let u32_string_enum_id = registry
            .register_type(&MetaType::new::<GenericEnum<u32, String>>())
            .id;
        let bool_u32_enum_id = registry
            .register_type(&MetaType::new::<GenericEnum<bool, u32>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let u32_string_enum = resolver.get(u32_string_enum_id).unwrap();
        assert_eq!(u32_string_enum.to_string(), "GenericEnum<u32, String>");

        let bool_u32_enum = resolver.get(bool_u32_enum_id).unwrap();
        assert_eq!(bool_u32_enum.to_string(), "GenericEnum<bool, u32>");
    }

    #[test]
    fn type_resolver_array_type() {
        let mut registry = Registry::new();
        let u32_array_id = registry.register_type(&MetaType::new::<[u32; 10]>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<[u32; 10]>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let u32_array = resolver.get(u32_array_id).unwrap();
        assert_eq!(u32_array.to_string(), "[u32; 10]");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<[u32; 10]>");
    }

    #[test]
    fn type_resolver_vector_type() {
        let mut registry = Registry::new();
        let u32_vector_id = registry.register_type(&MetaType::new::<Vec<u32>>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Vec<u32>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let u32_vector = resolver.get(u32_vector_id).unwrap();
        assert_eq!(u32_vector.to_string(), "[u32]");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<[u32]>");
    }

    #[test]
    fn type_resolver_result_type() {
        let mut registry = Registry::new();
        let u32_result_id = registry
            .register_type(&MetaType::new::<core::result::Result<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<
                GenericStruct<core::result::Result<u32, String>>,
            >())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let u32_result = resolver.get(u32_result_id).unwrap();
        assert_eq!(u32_result.to_string(), "Result<u32, String>");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(
            as_generic_param.to_string(),
            "GenericStruct<Result<u32, String>>"
        );
    }

    #[test]
    fn type_resolver_option_type() {
        let mut registry = Registry::new();
        let u32_option_id = registry.register_type(&MetaType::new::<Option<u32>>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Option<u32>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let u32_option = resolver.get(u32_option_id).unwrap();
        assert_eq!(u32_option.to_string(), "Option<u32>");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<Option<u32>>");
    }

    #[test]
    fn type_resolver_tuple_type() {
        let mut registry = Registry::new();
        let u32_str_tuple_id = registry.register_type(&MetaType::new::<(u32, String)>()).id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<(u32, String)>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let u32_str_tuple = resolver.get(u32_str_tuple_id).unwrap();
        assert_eq!(u32_str_tuple.to_string(), "(u32, String)");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<(u32, String)>");
    }

    #[test]
    fn type_resolver_btree_map_type() {
        let mut registry = Registry::new();
        let btree_map_id = registry
            .register_type(&MetaType::new::<BTreeMap<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<BTreeMap<u32, String>>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let btree_map = resolver.get(btree_map_id).unwrap();
        assert_eq!(btree_map.to_string(), "[(u32, String)]");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(
            as_generic_param.to_string(),
            "GenericStruct<[(u32, String)]>"
        );
    }
}
