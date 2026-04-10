use super::*;
use alloc::boxed::Box;
use convert_case::{Case, Casing};
use sails_type_registry::{
    Registry, TypeRef,
    trait_impls::StructuralEq,
    ty::{Field, GenericArg, Primitive, Type, TypeDef},
};

#[derive(Debug, Clone)]
pub struct TypeResolver<'a> {
    registry: &'a Registry,
    map: BTreeMap<TypeRef, TypeDecl>,
    user_defined: BTreeMap<String, UserDefinedEntry>,
}

#[derive(Debug, Clone)]
pub struct UserDefinedEntry {
    pub meta_type: sails_idl_meta::Type,
    pub ty: Type,
}

impl UserDefinedEntry {
    fn is_path_equals(&self, type_info: &Type) -> bool {
        self.ty.module_path == type_info.module_path && self.ty.name == type_info.name
    }

    fn is_fields_equal(&self, type_info: &Type, registry: &Registry) -> bool {
        Self::field_types(&self.ty).structurally_eq(&Self::field_types(type_info), registry)
    }

    fn field_types(type_info: &Type) -> Vec<TypeRef> {
        match &type_info.def {
            TypeDef::Composite(comp) => comp.fields.iter().map(|f| f.ty).collect(),
            TypeDef::Variant(var) => {
                let mut fields = Vec::new();
                for v in &var.variants {
                    fields.extend(v.fields.iter().map(|f| f.ty));
                }
                fields
            }
            _ => unreachable!(),
        }
    }

    #[cfg(test)]
    fn meta_fields(&self) -> Vec<StructField> {
        match &self.meta_type.def {
            sails_idl_meta::TypeDef::Struct(StructDef { fields }) => fields.clone(),
            sails_idl_meta::TypeDef::Enum(EnumDef { variants }) => {
                let mut fields = Vec::new();
                variants.iter().for_each(|v| {
                    fields.extend(v.def.fields.iter().cloned());
                });
                fields
            }
            sails_idl_meta::TypeDef::Alias(_) => Vec::new(),
        }
    }
}

impl<'a> TypeResolver<'a> {
    #[cfg(test)]
    pub fn from_registry(registry: &'a Registry) -> Self {
        TypeResolver::try_from(registry, BTreeSet::new()).unwrap()
    }

    pub fn try_from(registry: &'a Registry, exclude: BTreeSet<TypeRef>) -> Result<Self> {
        let mut resolver = Self {
            registry,
            map: BTreeMap::new(),
            user_defined: BTreeMap::new(),
        };
        resolver.build_type_decl_map(exclude)?;
        Ok(resolver)
    }

    pub fn into_types(self) -> Vec<sails_idl_meta::Type> {
        let mut vec: Vec<_> = self
            .user_defined
            .into_values()
            .map(|v| v.meta_type)
            .collect();
        vec.sort_by(|a, b| a.name.cmp(&b.name));
        vec
    }

    #[cfg(test)]
    pub fn get(&self, key: TypeRef) -> Option<&TypeDecl> {
        self.map.get(&key)
    }

    #[cfg(test)]
    pub fn get_user_defined(&self, name: &str) -> Option<&UserDefinedEntry> {
        self.user_defined.get(name)
    }

    fn build_type_decl_map(&mut self, exclude: BTreeSet<TypeRef>) -> Result<()> {
        let all_types: Vec<_> = self.registry.types().collect();
        for (id, _) in all_types {
            if exclude.contains(&id) {
                continue;
            }
            let type_decl = self.resolve_by_id(id)?;
            self.map.insert(id, type_decl);
        }
        Ok(())
    }

    pub fn resolve_by_id(&mut self, id: TypeRef) -> Result<TypeDecl> {
        self.resolve_by_id_inner(id, &BTreeMap::new())
    }

    pub(crate) fn resolve_by_id_inner(
        &mut self,
        id: TypeRef,
        type_args: &BTreeMap<String, TypeDecl>,
    ) -> Result<TypeDecl> {
        if type_args.is_empty()
            && let Some(decl) = self.map.get(&id)
        {
            return Ok(decl.clone());
        }

        let ty = self
            .registry
            .get_type(id)
            .ok_or(Error::TypeIdIsUnknown(id.get()))?;

        let type_decl = match &ty.def {
            TypeDef::Primitive(p) => TypeDecl::Primitive(primitive_map(p)?),
            TypeDef::GPrimitive(g) => TypeDecl::Primitive(gprimitive_map(g)?),
            TypeDef::Sequence(type_param) => TypeDecl::Slice {
                item: Box::new(self.resolve_by_id_inner(*type_param, type_args)?),
            },
            TypeDef::Array { len, type_param } => TypeDecl::Array {
                item: Box::new(self.resolve_by_id_inner(*type_param, type_args)?),
                len: *len,
            },
            TypeDef::Tuple(fields) => {
                if fields.is_empty() {
                    TypeDecl::Primitive(PrimitiveType::Void)
                } else {
                    let types = fields
                        .iter()
                        .map(|f| self.resolve_by_id_inner(*f, type_args))
                        .collect::<Result<Vec<_>>>()?;
                    TypeDecl::Tuple { types }
                }
            }
            TypeDef::Map { key, value } => {
                let key_decl = self.resolve_by_id_inner(*key, type_args)?;
                let value_decl = self.resolve_by_id_inner(*value, type_args)?;
                TypeDecl::Slice {
                    item: Box::new(TypeDecl::Tuple {
                        types: vec![key_decl, value_decl],
                    }),
                }
            }
            TypeDef::Option(type_param) => {
                let inner = self.resolve_by_id_inner(*type_param, type_args)?;
                TypeDecl::Named {
                    name: "Option".to_string(),
                    generics: vec![inner],
                }
            }
            TypeDef::Result { ok, err } => {
                let ok_decl = self.resolve_by_id_inner(*ok, type_args)?;
                let err_decl = self.resolve_by_id_inner(*err, type_args)?;
                TypeDecl::Named {
                    name: "Result".to_string(),
                    generics: vec![ok_decl, err_decl],
                }
            }
            TypeDef::Composite(_) | TypeDef::Variant(_) => {
                let name = self.register_user_defined(ty)?;
                let mut generics = Vec::new();
                for param in &ty.type_params {
                    if let GenericArg::Type(arg_id) = &param.arg {
                        generics.push(self.resolve_by_id_inner(*arg_id, type_args)?);
                    }
                }
                TypeDecl::Named { name, generics }
            }
            TypeDef::Parameter(name) => {
                if let Some(mapped) = type_args.get(name) {
                    mapped.clone()
                } else {
                    TypeDecl::Named {
                        name: name.clone(),
                        generics: vec![],
                    }
                }
            }
            TypeDef::Applied { base, args } => {
                self.registry
                    .get_type(*base)
                    .ok_or(Error::TypeIdIsUnknown(base.get()))?;

                let base_decl = self.resolve_by_id_inner(*base, &BTreeMap::new())?;
                if let TypeDecl::Named { name, .. } = base_decl {
                    let mut generics = Vec::new();
                    for arg in args {
                        generics.push(self.resolve_by_id_inner(*arg, type_args)?);
                    }
                    TypeDecl::Named { name, generics }
                } else {
                    base_decl
                }
            }
        };

        if type_args.is_empty() {
            self.map.insert(id, type_decl.clone());
        }
        Ok(type_decl)
    }

    fn register_user_defined(&mut self, ty: &Type) -> Result<String> {
        let name = match self.unique_type_name(ty) {
            Ok(name) => name,
            Err(exist) => return Ok(exist),
        };

        if self.user_defined.contains_key(&name) {
            return Ok(name);
        }

        self.user_defined.insert(
            name.clone(),
            UserDefinedEntry {
                meta_type: sails_idl_meta::Type {
                    name: name.clone(),
                    type_params: vec![],
                    def: sails_idl_meta::TypeDef::Struct(StructDef { fields: vec![] }),
                    docs: vec![],
                    annotations: vec![],
                },
                ty: ty.clone(),
            },
        );

        let empty_args = BTreeMap::new();

        let def = match &ty.def {
            TypeDef::Composite(comp) => {
                let fields = comp
                    .fields
                    .iter()
                    .map(|f| self.resolve_field_inner(f, &empty_args))
                    .collect::<Result<Vec<_>>>()?;
                sails_idl_meta::TypeDef::Struct(StructDef { fields })
            }
            TypeDef::Variant(var) => {
                let variants = var
                    .variants
                    .iter()
                    .map(|v| {
                        let fields = v
                            .fields
                            .iter()
                            .map(|f| self.resolve_field_inner(f, &empty_args))
                            .collect::<Result<Vec<_>>>()?;
                        Ok(EnumVariant {
                            name: v.name.to_string(),
                            def: StructDef { fields },
                            entry_id: 0,
                            docs: v.docs.iter().map(|d| d.to_string()).collect(),
                            annotations: v
                                .annotations
                                .iter()
                                .map(|a| (a.name.clone(), a.value.clone()))
                                .collect(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                sails_idl_meta::TypeDef::Enum(EnumDef { variants })
            }
            _ => unreachable!(),
        };

        let meta_type = sails_idl_meta::Type {
            name: name.clone(),
            type_params: ty
                .type_params
                .iter()
                .filter(|p| matches!(p.arg, GenericArg::Type(_)))
                .map(|p| sails_idl_meta::TypeParameter {
                    name: p.name.clone(),
                    ty: None,
                })
                .collect(),
            def,
            docs: ty.docs.iter().map(|d| d.to_string()).collect(),
            annotations: ty
                .annotations
                .iter()
                .map(|a| (a.name.clone(), a.value.clone()))
                .collect(),
        };

        self.user_defined.insert(
            name.clone(),
            UserDefinedEntry {
                meta_type,
                ty: ty.clone(),
            },
        );

        Ok(name)
    }

    fn resolve_field_inner(
        &mut self,
        field: &Field,
        type_args: &BTreeMap<String, TypeDecl>,
    ) -> Result<StructField> {
        let type_decl = self.resolve_by_id_inner(field.ty, type_args)?;
        Ok(StructField {
            name: field.name.clone(),
            type_decl,
            docs: field.docs.iter().map(|d| d.to_string()).collect(),
            annotations: field
                .annotations
                .iter()
                .map(|a| (a.name.clone(), a.value.clone()))
                .collect(),
        })
    }

    fn unique_type_name(&self, ty: &Type) -> Result<String, String> {
        let mut segments: Vec<&str> = ty
            .module_path
            .split("::")
            .filter(|s| !s.is_empty())
            .collect();
        segments.push(&ty.name);

        let mut base_name = String::new();
        let mut consts = Vec::new();
        for param in &ty.type_params {
            if let GenericArg::Const(val) = &param.arg {
                consts.push((param.name.clone(), val.clone()));
            }
        }
        consts.sort_by(|a, b| a.0.cmp(&b.0));
        let mut const_suffix = String::new();
        for (k, v) in consts {
            const_suffix.push_str(&k);
            const_suffix.push_str(&v);
        }

        for segment in segments.into_iter().rev() {
            base_name = segment.to_case(Case::Pascal) + &base_name;
            let name_with_consts = format!("{}{}", base_name, const_suffix);

            if let Some(exists) = self.user_defined.get(&name_with_consts) {
                if exists.is_path_equals(ty) && exists.is_fields_equal(ty, self.registry) {
                    return Err(name_with_consts);
                } else {
                    continue;
                }
            } else {
                return Ok(name_with_consts);
            }
        }

        let mut final_name = format!("{}{}", base_name, const_suffix);
        let mut i = 1;
        while self.user_defined.contains_key(&final_name) {
            final_name = format!("{}{}{}", base_name, const_suffix, i);
            i += 1;
        }
        Ok(final_name)
    }

    pub fn resolve_field_with_args(
        &mut self,
        field: &Field,
        type_args: &BTreeMap<String, TypeDecl>,
    ) -> Result<StructField> {
        self.resolve_field_inner(field, type_args)
    }
}

fn primitive_map(type_def_primitive: &Primitive) -> Result<PrimitiveType> {
    use PrimitiveType::*;
    Ok(match type_def_primitive {
        Primitive::Bool => Bool,
        Primitive::Char => Char,
        Primitive::Str => String,
        Primitive::U8 => U8,
        Primitive::U16 => U16,
        Primitive::U32 => U32,
        Primitive::U64 => U64,
        Primitive::U128 => U128,
        Primitive::I8 => I8,
        Primitive::I16 => I16,
        Primitive::I32 => I32,
        Primitive::I64 => I64,
        Primitive::I128 => I128,
    })
}

fn gprimitive_map(g: &sails_type_registry::ty::GPrimitive) -> Result<PrimitiveType> {
    use sails_type_registry::ty::GPrimitive::*;
    Ok(match g {
        U256 => PrimitiveType::U256,
        H160 => PrimitiveType::H160,
        H256 => PrimitiveType::H256,
        ActorId => PrimitiveType::ActorId,
        MessageId => PrimitiveType::MessageId,
        CodeId => PrimitiveType::CodeId,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};
    use gprimitives::NonZeroU256;
    use sails_idl_meta::TypeDef;
    use sails_type_registry::{Registry, TypeInfo};

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericStruct<T> {
        field: T,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    struct GenericConstStruct<const N: usize, const O: usize, T> {
        field: [T; N],
        field2: Option<[T; O]>,
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum GenericEnum<T1, T2> {
        Variant1(T1),
        Variant2(T2),
        Variant3(T1, Option<T2>),
        Variant4(Option<(T1, GenericStruct<T2>, u32)>),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum ManyVariants {
        One,
        Two(u32),
        Three(Option<Vec<gprimitives::U256>>),
        Four { a: u32, b: Option<u16> },
        Five(String, Vec<u8>),
        Six((u32,)),
        Seven(GenericEnum<u32, String>),
        Eight([BTreeMap<u32, String>; 10]),
        Nine(TupleVariantsDocs),
    }

    #[allow(dead_code)]
    #[derive(TypeInfo)]
    enum TupleVariantsDocs {
        /// Docs for no tuple docs 1
        NoTupleDocs1(u32, String),
        NoTupleDocs2(gprimitives::CodeId, Vec<u8>),
        /// Docs for tuple docs 1
        TupleDocs1(
            u32,
            /// This is the second field
            String,
        ),
        TupleDocs2(
            /// This is the first field
            u32,
            /// This is the second field
            String,
        ),
        /// Docs for struct docs
        StructDocs {
            /// This is field `a`
            a: u32,
            /// This is field `b`
            b: String,
        },
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
    fn type_resolver_h160_h256() {
        let mut registry = Registry::new();
        let _h160_id = registry.register_type::<gprimitives::H160>();
        let _h160_as_generic_param_id =
            registry.register_type::<GenericStruct<gprimitives::H160>>();

        let h256_id = registry.register_type::<gprimitives::H256>();
        let h256_as_generic_param_id = registry.register_type::<GenericStruct<gprimitives::H256>>();

        let resolver = TypeResolver::from_registry(&registry);

        let h256_decl = resolver.get(h256_id).unwrap();
        assert_eq!(*h256_decl, TypeDecl::Primitive(PrimitiveType::H256));

        let generic_struct_decl = resolver.get(h256_as_generic_param_id).unwrap();
        assert_eq!(
            *generic_struct_decl,
            TypeDecl::Named {
                name: "GenericStruct".to_string(),
                generics: vec![TypeDecl::Primitive(PrimitiveType::H256)]
            }
        );
        assert_eq!(generic_struct_decl.to_string(), "GenericStruct<H256>");
    }

    #[test]
    fn type_resolver_generic_struct() {
        let mut registry = Registry::new();
        let u32_struct_id = registry.register_type::<GenericStruct<u32>>();
        let string_struct_id = registry.register_type::<GenericStruct<String>>();
        let resolver = TypeResolver::from_registry(&registry);

        let u32_struct = resolver.get(u32_struct_id).unwrap();
        assert_eq!(u32_struct.to_string(), "GenericStruct<u32>");

        let string_struct = resolver.get(string_struct_id).unwrap();
        assert_eq!(string_struct.to_string(), "GenericStruct<String>");
    }

    #[test]
    fn type_resolver_generic_enum() {
        let mut registry = Registry::new();
        let u32_string_enum_id = registry.register_type::<GenericEnum<u32, String>>();
        let bool_u32_enum_id = registry.register_type::<GenericEnum<bool, u32>>();
        let resolver = TypeResolver::from_registry(&registry);

        let u32_string_enum = resolver.get(u32_string_enum_id).unwrap();
        assert_eq!(u32_string_enum.to_string(), "GenericEnum<u32, String>");

        let bool_u32_enum = resolver.get(bool_u32_enum_id).unwrap();
        assert_eq!(bool_u32_enum.to_string(), "GenericEnum<bool, u32>");
    }

    #[test]
    fn type_resolver_array_type() {
        let mut registry = Registry::new();
        let u32_array_id = registry.register_type::<[u32; 10]>();
        let as_generic_param_id = registry.register_type::<GenericStruct<[u32; 10]>>();
        let resolver = TypeResolver::from_registry(&registry);

        let u32_array = resolver.get(u32_array_id).unwrap();
        assert_eq!(u32_array.to_string(), "[u32; 10]");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<[u32; 10]>");
    }

    #[test]
    fn type_resolver_vector_type() {
        let mut registry = Registry::new();
        let u32_vector_id = registry.register_type::<Vec<u32>>();
        let as_generic_param_id = registry.register_type::<GenericStruct<Vec<u32>>>();
        let resolver = TypeResolver::from_registry(&registry);

        let u32_vector = resolver.get(u32_vector_id).unwrap();
        assert_eq!(u32_vector.to_string(), "[u32]");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<[u32]>");
    }

    #[test]
    fn type_resolver_result_type() {
        let mut registry = Registry::new();
        let u32_result_id = registry.register_type::<Result<u32, String>>();
        let as_generic_param_id = registry.register_type::<GenericStruct<Result<u32, String>>>();
        let resolver = TypeResolver::from_registry(&registry);

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
        let u32_option_id = registry.register_type::<Option<u32>>();
        let as_generic_param_id = registry.register_type::<GenericStruct<Option<u32>>>();
        let resolver = TypeResolver::from_registry(&registry);

        let u32_option = resolver.get(u32_option_id).unwrap();
        assert_eq!(u32_option.to_string(), "Option<u32>");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<Option<u32>>");
    }

    #[test]
    fn type_resolver_tuple_type() {
        let mut registry = Registry::new();
        let u32_str_tuple_id = registry.register_type::<(u32, String)>();
        let as_generic_param_id = registry.register_type::<GenericStruct<(u32, String)>>();
        let resolver = TypeResolver::from_registry(&registry);

        let u32_str_tuple = resolver.get(u32_str_tuple_id).unwrap();
        assert_eq!(u32_str_tuple.to_string(), "(u32, String)");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<(u32, String)>");
    }

    #[test]
    fn type_resolver_btree_map_type() {
        let mut registry = Registry::new();
        let btree_map_id = registry.register_type::<BTreeMap<u32, String>>();
        let as_generic_param_id = registry.register_type::<GenericStruct<BTreeMap<u32, String>>>();
        let resolver = TypeResolver::from_registry(&registry);

        let btree_map = resolver.get(btree_map_id).unwrap();
        assert_eq!(btree_map.to_string(), "[(u32, String)]");
        let as_generic_param = resolver.get(as_generic_param_id).unwrap();
        assert_eq!(
            as_generic_param.to_string(),
            "GenericStruct<[(u32, String)]>"
        );
    }

    #[test]
    fn type_resolver_enum_many_variants() {
        let mut registry = Registry::new();
        let id = registry.register_type::<ManyVariants>();
        let generic_id = registry.register_type::<GenericStruct<ManyVariants>>();
        let resolver = TypeResolver::from_registry(&registry);

        let ty = resolver.get(id).unwrap();
        assert_eq!(ty.to_string(), "ManyVariants");
        let as_generic_param = resolver.get(generic_id).unwrap();
        assert_eq!(as_generic_param.to_string(), "GenericStruct<ManyVariants>");
    }

    #[test]
    fn non_zero_types_name_resolution_works() {
        type Test = (
            NonZeroU8,
            NonZeroU16,
            NonZeroU32,
            NonZeroU64,
            NonZeroU128,
            NonZeroU256,
        );
        let mut registry = Registry::new();
        let id = registry.register_type::<Test>();
        let generic_id = registry.register_type::<GenericStruct<Test>>();
        let resolver = TypeResolver::from_registry(&registry);

        let ty = resolver.get(id).unwrap();
        assert_eq!(
            ty.to_string(),
            "(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroU256)"
        );

        let as_generic_param = resolver.get(generic_id).unwrap();
        assert_eq!(
            as_generic_param.to_string(),
            "GenericStruct<(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroU256)>"
        );
    }

    macro_rules! type_name_resolution_works {
        ($primitive:ty) => {
            let mut registry = Registry::new();
            let id = registry.register_type::<$primitive>();
            let generic_id = registry.register_type::<GenericStruct<$primitive>>();
            let resolver = TypeResolver::from_registry(&registry);

            let ty = resolver.get(id).unwrap();
            assert_eq!(ty.to_string(), stringify!($primitive));

            let as_generic_param = resolver.get(generic_id).unwrap();
            assert_eq!(
                as_generic_param.to_string(),
                format!("GenericStruct<{}>", stringify!($primitive))
            );
        };
    }

    #[test]
    fn actor_id_type_name_resolution_works() {
        use gprimitives::ActorId;
        type_name_resolution_works!(ActorId);
    }

    #[test]
    fn message_id_type_name_resolution_works() {
        use gprimitives::MessageId;
        type_name_resolution_works!(MessageId);
    }

    #[test]
    fn code_id_type_name_resolution_works() {
        use gprimitives::CodeId;
        type_name_resolution_works!(CodeId);
    }

    #[test]
    fn h160_type_name_resolution_works() {
        use gprimitives::H160;
        type_name_resolution_works!(H160);
    }

    #[test]
    fn h256_type_name_resolution_works() {
        use gprimitives::H256;
        type_name_resolution_works!(H256);
    }

    #[test]
    fn u256_type_name_resolution_works() {
        use gprimitives::U256;
        type_name_resolution_works!(U256);
    }

    #[test]
    fn type_name_minification_works_for_types_with_the_same_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry.register_type::<mod_1::T1>();
        let t2_id = registry.register_type::<mod_2::T1>();
        let resolver = TypeResolver::from_registry(&registry);

        let t1_name = resolver.get(t1_id).unwrap().to_string();
        assert_eq!(t1_name, "T1");

        let t2_name = resolver.get(t2_id).unwrap().to_string();
        assert_eq!(t2_name, "Mod2T1");
    }

    #[test]
    fn type_name_minification_works_for_types_with_different_mod_depth() {
        let mut registry = Registry::new();
        let t1_id = registry.register_type::<mod_1::mod_2::T2>();
        let t2_id = registry.register_type::<mod_2::T2>();
        let resolver = TypeResolver::from_registry(&registry);

        let t1_name = resolver.get(t1_id).unwrap().to_string();
        assert_eq!(t1_name, "T2");

        let t2_name = resolver.get(t2_id).unwrap().to_string();
        assert_eq!(t2_name, "Mod2T2");
    }

    #[test]
    fn generic_const_struct_type_name_resolution_works() {
        let mut registry = Registry::new();
        let n8_id = registry.register_type::<GenericConstStruct<8, 12, u8>>();
        let n8_id_2 = registry.register_type::<GenericConstStruct<8, 8, u8>>();
        let n32_id = registry.register_type::<GenericConstStruct<32, 8, u8>>();
        let n256_id = registry.register_type::<GenericConstStruct<256, 832, u8>>();
        let n32u256_id = registry.register_type::<GenericConstStruct<32, 8, gprimitives::U256>>();
        let resolver = TypeResolver::from_registry(&registry);

        let n8_name = resolver.get(n8_id).unwrap().to_string();
        let n8_name_2 = resolver.get(n8_id_2).unwrap().to_string();
        let n32_name = resolver.get(n32_id).unwrap().to_string();
        let n256_name = resolver.get(n256_id).unwrap().to_string();
        let n32u256_name = resolver.get(n32u256_id).unwrap().to_string();

        assert_eq!(n8_name, "GenericConstStructN8O12<u8>");
        assert_eq!(n8_name_2, "GenericConstStructN8O8<u8>");
        assert_eq!(n32_name, "GenericConstStructN32O8<u8>");
        assert_eq!(n256_name, "GenericConstStructN256O832<u8>");
        assert_eq!(n32u256_name, "GenericConstStructN32O8<U256>");
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
            result_generic: Result<T, String>,
            btreemap_generic: BTreeMap<String, T>,
            struct_generic: GenericStruct<T>,
            enum_generic: SimpleOneGenericEnum<T>,

            // Category 2: Two-level nested generics
            option_of_option: Option<Option<T>>,
            result_of_option: Result<Option<T>, String>,
            btreemap_nested: BTreeMap<Option<T>, GenericStruct<T>>,
            struct_of_option: GenericStruct<Option<T>>,
            enum_of_result: SimpleOneGenericEnum<Result<T, String>>,

            // Category 3: Triple-nested generics
            option_triple: Option<Option<Option<T>>>,
            result_triple: Result<Option<Result<T, String>>, String>,
            btreemap_triple: BTreeMap<Option<GenericStruct<T>>, Result<T, String>>,
            struct_triple: GenericStruct<Option<Result<T, String>>>,
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        enum SimpleOneGenericEnum<T> {
            // Category 1: Simple generic usage
            NoFields,
            GenericValue(T),
            TupleGeneric(String, T, T, u32),
            OptionGeneric(Option<T>),
            ResultGeneric(Result<T, String>),
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
                res: Result<Option<T>, String>,
            },
            DoubleNested {
                btree_map_nested: BTreeMap<Option<T>, GenericStruct<T>>,
                struct_nested: GenericStruct<Option<T>>,
            },

            // Category 3: Triple-nested generics
            TrippleNested {
                option_triple: Option<Option<Option<T>>>,
                result_triple: Result<Option<Result<T, String>>, String>,
            },
            OptionTriple(Option<Option<Option<T>>>),
            ResultTriple {
                res: Result<Option<Result<T, String>>, String>,
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

        let struct_id = registry.register_type::<SimpleOneGenericStruct<u32>>();
        let enum_id = registry.register_type::<SimpleOneGenericEnum<u32>>();

        let genericless_unit_id = registry.register_type::<GenericlessUnitStruct>();
        let genericless_tuple_id = registry.register_type::<GenericlessTupleStruct>();
        let genericless_named_id = registry.register_type::<GenericlessNamedStruct>();
        let genericless_enum_id = registry.register_type::<GenericlessEnum>();
        let genericless_variantless_enum_id =
            registry.register_type::<GenericlessVariantlessEnum>();

        let resolver = TypeResolver::from_registry(&registry);

        // Check main types
        assert_eq!(
            resolver.get(struct_id).unwrap().to_string(),
            "SimpleOneGenericStruct<u32>"
        );
        let struct_generic = resolver
            .get_user_defined("SimpleOneGenericStruct")
            .expect("struct generic must exist");

        assert_eq!(
            resolver.get(enum_id).unwrap().to_string(),
            "SimpleOneGenericEnum<u32>"
        );
        let enum_generic = resolver
            .get_user_defined("SimpleOneGenericEnum")
            .expect("enum generic must exist");

        // For structs: check that expected generic field strings are present
        let s_fields: Vec<_> = struct_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();

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
            "[(String, T)]",
            "GenericStruct<T>",
            "SimpleOneGenericEnum<T>",
            "Option<Option<T>>",
            "Result<Option<T>, String>",
            "[(Option<T>, GenericStruct<T>)]",
            "GenericStruct<Option<T>>",
            "SimpleOneGenericEnum<Result<T, String>>",
            "Option<Option<Option<T>>>",
            "Result<Option<Result<T, String>>, String>",
            "[(Option<GenericStruct<T>>, Result<T, String>)]",
            "GenericStruct<Option<Result<T, String>>>",
        ];

        for expected in expect_struct_fields_type_names {
            assert!(
                s_fields.contains(&expected.to_string()),
                "struct missing generic field {expected}, All fields: {s_fields:#?}"
            );
        }
        // For enums: check the collected `fields` contains expected signatures and variant names
        let e_fields: Vec<_> = enum_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();

        // First let's check no fields variants
        let TypeDef::Enum(EnumDef { variants }) = &enum_generic.meta_type.def else {
            panic!("Expected enum generic name");
        };

        let no_fields_variant = &variants[0];
        let no_fields2_variant = &variants[variants.len() - 1];

        assert_eq!(no_fields_variant.name, "NoFields");
        assert_eq!(no_fields2_variant.name, "NoFields2");
        assert!(no_fields_variant.def.fields.is_empty());
        assert!(no_fields2_variant.def.fields.is_empty());

        // expected generic strings for enum fields and nested types:
        let expect_enum_field_type_names = vec![
            "T",
            "String",
            "T",
            "T",
            "u32",
            "Option<T>",
            "Result<T, String>",
            "[(String, T)]",
            "GenericStruct<T>",
            "NestedGenericEnum<T>",
            "Option<Option<T>>",
            "Result<Option<T>, String>",
            "[(Option<T>, GenericStruct<T>)]",
            "GenericStruct<Option<T>>",
            "Option<Option<Option<T>>>",
            "Result<Option<Result<T, String>>, String>",
        ];

        for expected in expect_enum_field_type_names {
            assert!(
                e_fields.contains(&expected.to_string()),
                "enum missing generic field {expected}. All enum fields/entries: {e_fields:#?}"
            );
        }

        // Also verify concrete_names for some representative fields to keep parity with original test spirit
        // Retrieve struct type to check underlying field concrete ids
        let struct_type = registry
            .types()
            .find(|(id, _)| *id == struct_id)
            .map(|(_, ty)| ty)
            .unwrap();

        let struct_def = &struct_type.def;

        if let sails_type_registry::ty::TypeDef::Composite(_composite) = struct_def {
            let meta_fields = struct_generic.meta_fields();
            let find_field_name = |name: &str| {
                meta_fields
                    .iter()
                    .find(|f| f.name.as_ref().is_some_and(|s| s == name))
                    .map(|f| f.type_decl.to_string())
                    .unwrap()
            };

            assert_eq!(find_field_name("generic_value"), "T");
            assert_eq!(find_field_name("tuple_generic"), "(String, T, T, u32)");
            assert_eq!(find_field_name("option_generic"), "Option<T>");
            assert_eq!(find_field_name("btreemap_generic"), "[(String, T)]");
        } else {
            panic!("Expected composite type");
        }

        let genericless_unit = resolver.get(genericless_unit_id).unwrap();
        assert_eq!(genericless_unit.to_string(), "GenericlessUnitStruct");
        let genericless_unit_defined = resolver.get_user_defined("GenericlessUnitStruct").unwrap();
        assert!(genericless_unit_defined.meta_fields().is_empty());

        let genericless_tuple = resolver.get(genericless_tuple_id).unwrap();
        assert_eq!(genericless_tuple.to_string(), "GenericlessTupleStruct");
        let genericless_tuple_def = resolver.get_user_defined("GenericlessTupleStruct").unwrap();
        let fields = genericless_tuple_def.meta_fields();
        let expected_fields_value = vec![
            StructField {
                name: None,
                type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                docs: vec![],
                annotations: vec![],
            },
            StructField {
                name: None,
                type_decl: TypeDecl::Primitive(PrimitiveType::String),
                docs: vec![],
                annotations: vec![],
            },
        ];
        assert_eq!(fields, expected_fields_value);

        let genericless_named = resolver.get(genericless_named_id).unwrap();
        assert_eq!(genericless_named.to_string(), "GenericlessNamedStruct");
        let genericless_named_def = resolver.get_user_defined("GenericlessNamedStruct").unwrap();
        let fields = genericless_named_def.meta_fields();
        let expected_fields_value = vec![
            StructField {
                name: Some("a".to_string()),
                type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                docs: vec![],
                annotations: vec![],
            },
            StructField {
                name: Some("b".to_string()),
                type_decl: TypeDecl::Primitive(PrimitiveType::String),
                docs: vec![],
                annotations: vec![],
            },
        ];
        assert_eq!(fields, expected_fields_value);

        let genericless_enum = resolver.get(genericless_enum_id).unwrap();
        assert_eq!(genericless_enum.to_string(), "GenericlessEnum");
        let genericless_enum_def = resolver.get_user_defined("GenericlessEnum").unwrap();
        let TypeDef::Enum(EnumDef { variants }) = &genericless_enum_def.meta_type.def else {
            panic!("Expected enum");
        };

        let expected_variants = vec![
            EnumVariant {
                name: "Unit".to_string(),
                def: StructDef { fields: vec![] },
                entry_id: 0,
                docs: vec![],
                annotations: vec![],
            },
            EnumVariant {
                name: "Tuple".to_string(),
                def: StructDef {
                    fields: vec![
                        StructField {
                            name: None,
                            type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                            docs: vec![],
                            annotations: vec![],
                        },
                        StructField {
                            name: None,
                            type_decl: TypeDecl::Primitive(PrimitiveType::String),
                            docs: vec![],
                            annotations: vec![],
                        },
                    ],
                },
                entry_id: 0,
                docs: vec![],
                annotations: vec![],
            },
            EnumVariant {
                name: "Named".to_string(),
                def: StructDef {
                    fields: vec![
                        StructField {
                            name: Some("a".to_string()),
                            type_decl: TypeDecl::Primitive(PrimitiveType::U32),
                            docs: vec![],
                            annotations: vec![],
                        },
                        StructField {
                            name: Some("b".to_string()),
                            type_decl: TypeDecl::Primitive(PrimitiveType::String),
                            docs: vec![],
                            annotations: vec![],
                        },
                    ],
                },
                entry_id: 0,
                docs: vec![],
                annotations: vec![],
            },
        ];
        assert_eq!(variants, &expected_variants);

        let genericless_variantless_enum = resolver.get(genericless_variantless_enum_id).unwrap();
        assert_eq!(
            genericless_variantless_enum.to_string(),
            "GenericlessVariantlessEnum"
        );
        let genericless_variantless_enum_def = resolver
            .get_user_defined("GenericlessVariantlessEnum")
            .unwrap();
        let TypeDef::Enum(EnumDef { variants }) = &genericless_variantless_enum_def.meta_type.def
        else {
            panic!("Expected enum");
        };
        assert!(variants.is_empty());
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
            tuple_of_result: (Result<T, String>, Option<T>),
            vec_of_struct: Vec<GenericStruct<T>>,
            array_of_btreemap: [BTreeMap<String, T>; 2],

            array_of_vec_of_option: [Vec<Option<T>>; 4],
            tuple_triple: (Option<Vec<T>>, Result<[T; 3], String>),
            vec_of_struct_of_option: Vec<GenericStruct<Option<T>>>,
            array_complex_triple: [BTreeMap<Option<T>, Result<T, String>>; 2],
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
                tuple: (Result<T, String>, Option<T>),
            },
            VecOfStruct(Vec<GenericStruct<T>>),
            ArrayOfBTreeMap {
                array: [BTreeMap<String, T>; 2],
            },

            ArrayOfVecOfOption([Vec<Option<Vec<T>>>; 4]),
            TupleTriple {
                field1: Option<Option<Vec<T>>>,
                field2: Result<Option<[T; 3]>, String>,
            },
            VecOfStructOfOption(Vec<GenericStruct<Option<T>>>),
            ArrayComplexTriple([BTreeMap<BTreeMap<Option<T>, String>, Result<T, String>>; 2]),
        }

        // Register types
        let mut registry = Registry::new();
        let struct_id = registry.register_type::<ComplexOneGenericStruct<bool>>();
        let enum_id = registry.register_type::<ComplexOneGenericEnum<bool>>();

        let resolver = TypeResolver::from_registry(&registry);

        // Check top level resolved names
        let struct_complex = resolver.get(struct_id).unwrap();
        assert_eq!(struct_complex.to_string(), "ComplexOneGenericStruct<bool>");
        let struct_generic = resolver
            .get_user_defined("ComplexOneGenericStruct")
            .unwrap();
        // Validate Struct generics
        let struct_field_types: Vec<_> = struct_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expect_struct_field_types = vec![
            "[T; 10]",
            "(T, [T], [T; 5])",
            "[(T, T); 3]",
            "[[T; 8]]",
            "[Option<T>; 5]",
            "(Result<T, String>, Option<T>)",
            "[GenericStruct<T>]",
            "[[(String, T)]; 2]",
            "[[Option<T>]; 4]",
            "(Option<[T]>, Result<[T; 3], String>)",
            "[GenericStruct<Option<T>>]",
            "[[(Option<T>, Result<T, String>)]; 2]",
        ];

        for expected in expect_struct_field_types {
            assert!(
                struct_field_types.contains(&expected.to_string()),
                "Struct missing field type {expected}.\n All: {struct_field_types:#?}"
            );
        }

        let enum_complex = resolver.get(enum_id).unwrap();
        assert_eq!(enum_complex.to_string(), "ComplexOneGenericEnum<bool>");
        let enum_generic = resolver.get_user_defined("ComplexOneGenericEnum").unwrap();

        let enum_field_types: Vec<_> = enum_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expect_enum_field_types = vec![
            "[T; 10]",
            "T",
            "[T]",
            "[T; 5]",
            "[(T, T); 3]",
            "[[T; 8]]",
            "[Option<T>; 5]",
            "(Result<T, String>, Option<T>)",
            "[GenericStruct<T>]",
            "[[(String, T)]; 2]",
            "[[Option<[T]>]; 4]",
            "Option<Option<[T]>>",
            "Result<Option<[T; 3]>, String>",
            "[GenericStruct<Option<T>>]",
            "[[([(Option<T>, String)], Result<T, String>)]; 2]",
        ];

        for expected in expect_enum_field_types {
            assert!(
                enum_field_types.contains(&expected.to_string()),
                "Enum missing field type {expected}.\n All: {enum_field_types:#?}"
            );
        }
    }

    #[test]
    fn multiple_generics() {
        use gprimitives::H256;

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
            option_of_result: Option<Result<T1, T2>>,
            array_of_option: [Option<T2>; 6],
            vec_of_tuple: Vec<(T2, T3, T1)>,
            tuple_of_result: (Result<T1, String>, Option<T2>),
            btreemap_nested: BTreeMap<Option<T1>, Result<T2, String>>,
            struct_of_tuple: GenericStruct<(T2, T3)>,

            // Category 4: Triple-nested complex types with multiple generics
            option_triple: Option<Result<Vec<T1>, T2>>,
            array_triple: [BTreeMap<T1, Option<T2>>; 3],
            vec_of_struct_of_option: Vec<GenericStruct<Option<T3>>>,
            array_of_vec_of_tuple: [Vec<(T1, T2)>; 2],
            tuple_complex_triple: (Option<Vec<T1>>, Result<[T2; 4], T3>),
            vec_complex: Vec<GenericStruct<Result<T1, T2>>>,
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
            OptionOfResult(Option<Result<T1, T2>>),
            ArrayOfOption([Option<T2>; 6]),
            VecOfTuple(Vec<(T2, T3, T1)>),
            TupleOfResult {
                field1: Result<T1, String>,
                field2: Option<T2>,
            },
            BTreeMapNested {
                map: BTreeMap<Option<T1>, Result<T2, String>>,
            },
            StructOfTuple(GenericStruct<(T2, T3)>),

            // Category 4: Triple-nested complex types with multiple generics
            OptionTriple(Option<Result<Vec<T1>, T2>>),
            ArrayTriple([BTreeMap<T1, Option<T2>>; 3]),
            VecOfStructOfOption(Vec<GenericStruct<Option<T3>>>),
            ArrayOfVecOfTuple {
                array: [Vec<(T1, T2)>; 2],
            },
            TupleComplexTriple {
                field1: Option<Vec<T1>>,
                field2: Result<[T2; 4], T3>,
            },
            VecComplex(Vec<GenericStruct<Result<T1, T2>>>),
        }

        // Register types and build portable registry
        let mut registry = Registry::new();
        let struct_id = registry.register_type::<MultiGenStruct<u32, String, H256>>();
        let enum_id = registry.register_type::<MultiGenEnum<u32, String, H256>>();
        let resolver = TypeResolver::from_registry(&registry);

        assert_eq!(
            resolver.get(struct_id).unwrap().to_string(),
            "MultiGenStruct<u32, String, H256>"
        );
        assert_eq!(
            resolver.get(enum_id).unwrap().to_string(),
            "MultiGenEnum<u32, String, H256>"
        );

        let struct_generic = resolver
            .get_user_defined("MultiGenStruct")
            .expect("MultiGenStruct generic must exist");
        let struct_field_types: Vec<_> = struct_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expect_struct_field_types = vec![
            "T1",
            "T2",
            "T3",
            "[T1; 8]",
            "(T2, T3)",
            "[T3]",
            "(T1, T2, T3)",
            "(T1, T1, T2, T2, T3, T3)",
            "[(T1, T2); 4]",
            "[[T3; 5]]",
            "[(T1, T2)]",
            "GenericStruct<T3>",
            "GenericEnum<T1, T2>",
            "Option<Result<T1, T2>>",
            "[Option<T2>; 6]",
            "[(T2, T3, T1)]",
            "(Result<T1, String>, Option<T2>)",
            "[(Option<T1>, Result<T2, String>)]",
            "GenericStruct<(T2, T3)>",
            "Option<Result<[T1], T2>>",
            "[[(T1, Option<T2>)]; 3]",
            "[GenericStruct<Option<T3>>]",
            "[[(T1, T2)]; 2]",
            "(Option<[T1]>, Result<[T2; 4], T3>)",
            "[GenericStruct<Result<T1, T2>>]",
        ];
        for expected in expect_struct_field_types {
            assert!(
                struct_field_types.contains(&expected.to_string()),
                "MultiGenStruct missing field type {expected}.\n All: {struct_field_types:#?}"
            );
        }

        let enum_generic = resolver
            .get_user_defined("MultiGenEnum")
            .expect("MultiGenEnum generic must exist");
        let enum_field_types: Vec<_> = enum_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expect_enum_field_types = vec![
            "T1",
            "T2",
            "T3",
            "[T1; 8]",
            "(T2, T3)",
            "[T3]",
            "T1",
            "T2",
            "T3",
            "(T1, T1, T2, T2, T3, T3)",
            "[(T1, T2); 4]",
            "[[T3; 5]]",
            "[(T1, T2)]",
            "GenericStruct<T3>",
            "GenericEnum<T1, T2>",
            "Option<Result<T1, T2>>",
            "[Option<T2>; 6]",
            "[(T2, T3, T1)]",
            "Result<T1, String>",
            "Option<T2>",
            "[(Option<T1>, Result<T2, String>)]",
            "GenericStruct<(T2, T3)>",
            "Option<Result<[T1], T2>>",
            "[[(T1, Option<T2>)]; 3]",
            "[GenericStruct<Option<T3>>]",
            "[[(T1, T2)]; 2]",
            "Option<[T1]>",
            "Result<[T2; 4], T3>",
            "[GenericStruct<Result<T1, T2>>]",
        ];
        for expected in expect_enum_field_types {
            assert!(
                enum_field_types.contains(&expected.to_string()),
                "MultiGenEnum missing field type {expected}.\n All: {enum_field_types:#?}"
            );
        }

        let struct_type = registry
            .types()
            .find(|(id, _)| *id == struct_id)
            .map(|(_, ty)| ty)
            .unwrap();
        let struct_def = &struct_type.def;

        if let sails_type_registry::ty::TypeDef::Composite(_composite) = struct_def {
            let find_field_name = |name: &str| {
                struct_generic
                    .meta_fields()
                    .iter()
                    .find(|f| f.name.as_ref().is_some_and(|s| s == name))
                    .map(|f| f.type_decl.to_string())
                    .unwrap()
            };

            assert_eq!(find_field_name("just_t1"), "T1");
            assert_eq!(find_field_name("tuple_t2_t3"), "(T2, T3)");
            assert_eq!(find_field_name("vec_t3"), "[T3]");
            assert_eq!(find_field_name("array_triple"), "[[(T1, Option<T2>)]; 3]");
        } else {
            panic!("Expected composite type");
        }

        let enum_type = registry
            .types()
            .find(|(id, _)| *id == enum_id)
            .map(|(_, ty)| ty)
            .unwrap();
        let enum_def = &enum_type.def;

        if let sails_type_registry::ty::TypeDef::Variant(_variant) = enum_def {
            let enum_variants = match &enum_generic.meta_type.def {
                sails_idl_meta::TypeDef::Enum(e) => &e.variants,
                _ => panic!("Expected enum definition"),
            };
            let find_variant_field = |v_name: &str, f_idx: usize| {
                enum_variants
                    .iter()
                    .find(|v| v.name == v_name)
                    .unwrap()
                    .def
                    .fields
                    .get(f_idx)
                    .unwrap()
                    .type_decl
                    .to_string()
            };

            assert_eq!(find_variant_field("TupleT2T3", 0), "(T2, T3)");
            assert_eq!(find_variant_field("TupleOfResult", 0), "Result<T1, String>");
        } else {
            panic!("Expected variant type");
        }
    }

    #[test]
    fn generic_const_with_generic_types() {
        use gprimitives::H256;

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
            result: Result<T1, T2>,
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
        let struct_n8_u32_id = registry.register_type::<ConstGenericStruct<8, u32>>();
        let struct_n8_string_id = registry.register_type::<ConstGenericStruct<8, String>>();

        let struct_n16_u32_id = registry.register_type::<ConstGenericStruct<16, u32>>();

        assert_ne!(struct_n8_u32_id, struct_n8_string_id);
        assert_ne!(struct_n8_u32_id, struct_n16_u32_id);

        // Register TwoConstGenericStruct
        let two_const_id = registry.register_type::<TwoConstGenericStruct<4, 8, u64, H256>>();

        // Register ConstGenericEnum
        let enum_n8_bool_id = registry.register_type::<ConstGenericEnum<8, bool>>();

        let resolver = TypeResolver::from_registry(&registry);

        // Check ConstGenericStruct with N=8, T=u32
        let struct_n8_u32_decl = resolver.get(struct_n8_u32_id).unwrap().to_string();
        let struct_n8_string_decl = resolver.get(struct_n8_string_id).unwrap().to_string();
        let struct_n16_u32_decl = resolver.get(struct_n16_u32_id).unwrap().to_string();
        let two_const_decl = resolver.get(two_const_id).unwrap().to_string();
        let enum_n8_bool_decl = resolver.get(enum_n8_bool_id).unwrap().to_string();

        assert_eq!(struct_n8_u32_decl, "ConstGenericStructN8<u32>");
        assert_eq!(struct_n8_string_decl, "ConstGenericStructN8<String>");
        assert_eq!(struct_n16_u32_decl, "ConstGenericStructN16<u32>");
        assert_eq!(two_const_decl, "TwoConstGenericStructM8N4<u64, H256>");
        assert_eq!(enum_n8_bool_decl, "ConstGenericEnumN8<bool>");

        let TypeDecl::Named {
            name: struct_n8_u32_name,
            ..
        } = resolver.get(struct_n8_u32_id).unwrap()
        else {
            panic!("Expected named type")
        };
        let struct_n8_u32 = resolver.get_user_defined(struct_n8_u32_name).unwrap();
        let field_type_names: Vec<_> = struct_n8_u32
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expected_field_type_names = vec!["[T; 8]", "T", "[T]", "Option<T>"];
        for expected in expected_field_type_names {
            assert!(
                field_type_names.contains(&expected.to_string()),
                "ConstGenericStruct1<T> missing field type name `{expected}`. All: {field_type_names:#?}",
            );
        }

        let TypeDecl::Named {
            name: two_const_name,
            ..
        } = resolver.get(two_const_id).unwrap()
        else {
            panic!("Expected named type")
        };
        let two_const_generic = resolver.get_user_defined(two_const_name).unwrap();
        let field_type_names: Vec<_> = two_const_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expected_field_type_names = vec![
            "[T1; 4]",
            "[T2; 8]",
            "(T1, T2)",
            "GenericStruct<T1>",
            "Result<T1, T2>",
        ];
        for expected in expected_field_type_names {
            assert!(
                field_type_names.contains(&expected.to_string()),
                "TwoConstGenericStruct<T1, T2> missing field type name `{expected}`. All: {field_type_names:#?}",
            );
        }

        let TypeDecl::Named {
            name: enum_n8_bool_name,
            ..
        } = resolver.get(enum_n8_bool_id).unwrap()
        else {
            panic!("Expected named type")
        };
        let enum_generic = resolver.get_user_defined(enum_n8_bool_name).unwrap();
        let field_type_names: Vec<_> = enum_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expected_field_type_names = vec!["[T; 8]", "T", "GenericStruct<T>"];
        for expected in expected_field_type_names {
            assert!(
                field_type_names.contains(&expected.to_string()),
                "ConstGenericEnum<T> missing field type name `{expected}`. All: {field_type_names:#?}",
            );
        }
    }

    #[test]
    fn recursive_generic_transparent_wrapper_stays_flat() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct Recursive<T> {
            next: Option<Box<Recursive<T>>>,
            value: T,
        }

        let mut registry = Registry::new();
        let recursive_id = registry.register_type::<Recursive<u32>>();

        let resolver = TypeResolver::from_registry(&registry);

        let TypeDecl::Named { name, .. } = resolver.get(recursive_id).unwrap() else {
            panic!("Expected named type")
        };
        let recursive = resolver.get_user_defined(name).unwrap();

        let next_ty = recursive
            .meta_fields()
            .iter()
            .find(|f| f.name.as_deref() == Some("next"))
            .map(|f| f.type_decl.to_string())
            .unwrap();

        assert_eq!(next_ty, "Option<Recursive<T>>");
    }

    #[test]
    fn nested_const_generic_arguments_keep_consts_in_base_name() {
        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct Wrapper<T, const N: usize> {
            value: T,
            bytes: [u8; N],
        }

        #[allow(dead_code)]
        #[derive(TypeInfo)]
        struct Holder<T, const N: usize> {
            inner: Wrapper<T, N>,
            maybe: Option<Box<Wrapper<T, N>>>,
        }

        let mut registry = Registry::new();
        let holder_id = registry.register_type::<Holder<u32, 16>>();

        let resolver = TypeResolver::from_registry(&registry);

        let TypeDecl::Named { name, .. } = resolver.get(holder_id).unwrap() else {
            panic!("Expected named type")
        };
        let holder = resolver.get_user_defined(name).unwrap();

        let inner_ty = holder
            .meta_fields()
            .iter()
            .find(|f| f.name.as_deref() == Some("inner"))
            .map(|f| f.type_decl.to_string())
            .unwrap();
        let maybe_ty = holder
            .meta_fields()
            .iter()
            .find(|f| f.name.as_deref() == Some("maybe"))
            .map(|f| f.type_decl.to_string())
            .unwrap();

        assert_eq!(inner_ty, "WrapperN16<T>");
        assert_eq!(maybe_ty, "Option<WrapperN16<T>>");
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
            result_mix: Result<module_a::SameName<T1>, module_b::SameName<T2>>,
        }

        let mut registry = Registry::new();
        let struct_id = registry.register_type::<TestStruct<u32, bool>>();

        let resolver = TypeResolver::from_registry(&registry);

        // Check main type
        assert_eq!(
            resolver.get(struct_id).unwrap().to_string(),
            "TestStruct<u32, bool>"
        );
        let struct_generic = resolver
            .get_user_defined("TestStruct")
            .expect("TestStruct generic must exist");
        let struct_field_type_names: Vec<_> = struct_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expected_field_type_names = vec![
            "SameName<T1>",
            "ModuleBSameName<T2>",
            "NestedSameName<T1>",
            "GenericStruct<SameName<T2>>",
            "GenericStruct<ModuleBSameName<T1>>",
            "[NestedSameName<T1>]",
            "Option<ModuleBSameName<T2>>",
            "Result<SameName<T1>, ModuleBSameName<T2>>",
        ];

        for expected in expected_field_type_names {
            assert!(
                struct_field_type_names.contains(&expected.to_string()),
                "TestStruct<T1, T2> missing field type name `{expected}`. All: {struct_field_type_names:#?}",
            );
        }
    }

    #[test]
    fn type_names_concrete_generic_reuses() {
        use gprimitives::{CodeId, H256};

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
            c4: Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>,
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
            C4(Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>),
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
        let struct_id = registry.register_type::<ReuseTestStruct<u64, H256>>();
        let enum_id = registry.register_type::<ReuseTestEnum<u64, H256>>();

        let resolver = TypeResolver::from_registry(&registry);

        assert_eq!(
            resolver.get(struct_id).unwrap().to_string(),
            "ReuseTestStruct<u64, H256>"
        );
        assert_eq!(
            resolver.get(enum_id).unwrap().to_string(),
            "ReuseTestEnum<u64, H256>"
        );

        let struct_generic = resolver
            .get_user_defined("ReuseTestStruct")
            .expect("ReuseTestStruct generic must exist");
        let enum_generic = resolver
            .get_user_defined("ReuseTestEnum")
            .expect("ReuseTestEnum generic must exist");

        let struct_field_types: Vec<_> = struct_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expect_struct_field_types = vec![
            "ReusableGenericStruct<T1>",
            "ReusableGenericStruct<CodeId>",
            "ReusableGenericStruct<[T1]>",
            "ReusableGenericStruct<[bool]>",
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
            "[ReusableGenericStruct<T1>]",
            "[ReusableGenericEnum<T2>; 5]",
            "Option<ReusableGenericStruct<(T1, T2)>>",
            "Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>",
            "[(T1, ReusableGenericStruct<T2>)]",
            "[(ReusableGenericEnum<T1>, String)]",
            "[(ReusableGenericStruct<T1>, ReusableGenericEnum<T2>)]",
            "[(ReusableGenericStruct<u64>, ReusableGenericEnum<H256>)]",
        ];

        for e in expect_struct_field_types {
            assert!(
                struct_field_types.contains(&e.to_string()),
                "{} missing expected type signature `{}`. All entries: {:#?}",
                "ReuseTestStruct<T1, T2>",
                e,
                struct_field_types
            );
        }

        let enum_field_types: Vec<_> = enum_generic
            .meta_fields()
            .iter()
            .map(|f| f.type_decl.to_string())
            .collect();
        let expect_enum_field_types = vec![
            "ReusableGenericStruct<T1>",
            "ReusableGenericStruct<CodeId>",
            "ReusableGenericStruct<[T1]>",
            "ReusableGenericStruct<[bool]>",
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
            "[ReusableGenericStruct<T1>]",
            "[ReusableGenericEnum<T2>; 5]",
            "Option<ReusableGenericStruct<(T1, T2)>>",
            "Result<ReusableGenericEnum<T1>, ReusableGenericEnum<T2>>",
            "[(T1, ReusableGenericStruct<T2>)]",
            "[(ReusableGenericEnum<T1>, String)]",
            "[(ReusableGenericStruct<T1>, ReusableGenericEnum<T2>)]",
            "[(ReusableGenericStruct<u64>, ReusableGenericEnum<H256>)]",
        ];

        for e in expect_enum_field_types {
            assert!(
                enum_field_types.contains(&e.to_string()),
                "{} missing expected type signature `{}`. All entries: {:#?}",
                "ReuseTestEnum<T1, T2>",
                e,
                enum_field_types
            );
        }
    }
}
