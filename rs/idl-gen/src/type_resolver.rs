use super::*;
use convert_case::{Case, Casing};
use scale_info::{
    Field, PortableRegistry, StaticTypeInfo, Type, TypeDef, TypeDefComposite, TypeDefPrimitive,
    TypeDefVariant,
};

#[derive(Debug, Clone)]
pub struct TypeResolver<'a> {
    registry: &'a PortableRegistry,
    map: BTreeMap<u32, TypeDecl>,
    user_defined: BTreeMap<String, UserDefinedEntry>,
}

#[derive(Debug, Clone)]
pub struct UserDefinedEntry {
    pub meta_type: sails_idl_meta::Type,
    pub ty: Type<PortableForm>,
}

impl UserDefinedEntry {
    fn is_path_equals(&self, type_info: &Type<PortableForm>) -> bool {
        self.ty.path == type_info.path
    }

    fn is_fields_equal(&self, type_info: &Type<PortableForm>) -> bool {
        let fs1 = Self::fields(&self.ty);
        let fs2 = Self::fields(type_info);
        fs1 == fs2
    }

    fn fields(type_info: &Type<PortableForm>) -> Vec<u32> {
        match &type_info.type_def {
            TypeDef::Composite(type_def_composite) => type_def_composite
                .fields
                .iter()
                .map(|f| f.ty.id)
                .collect::<Vec<_>>(),
            TypeDef::Variant(type_def_variant) => {
                let mut fields = Vec::new();
                type_def_variant.variants.iter().for_each(|v| {
                    fields.extend(v.fields.iter().map(|f| f.ty.id));
                });
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
    pub fn from_registry(registry: &'a PortableRegistry) -> Self {
        TypeResolver::try_from(registry, BTreeSet::new()).unwrap()
    }

    pub fn try_from(registry: &'a PortableRegistry, exclude: BTreeSet<u32>) -> Result<Self> {
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

    pub fn get(&self, key: u32) -> Option<&TypeDecl> {
        self.map.get(&key)
    }

    #[cfg(test)]
    pub fn get_user_defined(&self, name: &str) -> Option<&UserDefinedEntry> {
        self.user_defined.get(name)
    }

    fn build_type_decl_map(&mut self, exclude: BTreeSet<u32>) -> Result<()> {
        let filtered: Vec<_> = self
            .registry
            .types
            .iter()
            .filter(|pt| !exclude.contains(&pt.id))
            .collect();
        for pt in filtered {
            let type_decl = self.resolve_type_decl(&pt.ty)?;
            self.map.insert(pt.id, type_decl);
        }
        Ok(())
    }

    fn resolve_by_id(&mut self, id: u32) -> Result<TypeDecl> {
        if let Some(decl) = self.get(id) {
            return Ok(decl.clone());
        }
        let ty = self
            .registry
            .resolve(id)
            .ok_or(Error::TypeIdIsUnknown(id))?;
        let type_decl = self.resolve_type_decl(ty)?;
        self.map.insert(id, type_decl.clone());
        Ok(type_decl)
    }

    fn resolve_type_decl(&mut self, ty: &Type<PortableForm>) -> Result<TypeDecl> {
        let decl = match &ty.type_def {
            TypeDef::Composite(type_def_composite) => {
                if let Some(decl) = self.resolve_known_composite(ty, type_def_composite) {
                    decl
                } else {
                    let name = self.register_user_defined(ty)?;
                    self.resolve_user_defined(name, ty)?
                }
            }
            TypeDef::Variant(type_def_variant) => {
                if let Some(decl) = self.resolve_known_enum(ty, type_def_variant) {
                    decl
                } else {
                    let name = self.register_user_defined(ty)?;
                    self.resolve_user_defined(name, ty)?
                }
            }
            TypeDef::Sequence(type_def_sequence) => TypeDecl::Slice {
                item: Box::new(self.resolve_by_id(type_def_sequence.type_param.id)?),
            },
            TypeDef::Array(type_def_array) => TypeDecl::Array {
                item: Box::new(self.resolve_by_id(type_def_array.type_param.id)?),
                len: type_def_array.len,
            },
            TypeDef::Tuple(type_def_tuple) => {
                if type_def_tuple.fields.is_empty() {
                    TypeDecl::Primitive(PrimitiveType::Void)
                } else {
                    let types = type_def_tuple
                        .fields
                        .iter()
                        .map(|f| self.resolve_by_id(f.id))
                        .collect::<Result<Vec<_>>>()?;
                    TypeDecl::tuple(types)
                }
            }
            TypeDef::Primitive(type_def_primitive) => {
                TypeDecl::Primitive(primitive_map(type_def_primitive)?)
            }
            TypeDef::Compact(_) => {
                return Err(Error::TypeIsUnsupported(
                    "TypeDef::Compact is unsupported".to_string(),
                ));
            }
            TypeDef::BitSequence(_) => {
                return Err(Error::TypeIsUnsupported(
                    "TypeDef::BitSequence is unsupported".to_string(),
                ));
            }
        };
        Ok(decl)
    }

    fn register_user_defined(&mut self, ty: &Type<PortableForm>) -> Result<String> {
        let mut name = match self.unique_type_name(ty) {
            Ok(name) => name,
            Err(exist) => return Ok(exist),
        };

        let type_params = self.resolve_type_params(ty)?;
        let mut suffixes = BTreeSet::new();

        let def = match &ty.type_def {
            TypeDef::Composite(type_def_composite) => {
                let fields = type_def_composite
                    .fields
                    .iter()
                    .map(|f| self.resolve_field(f, &type_params, &mut suffixes))
                    .collect::<Result<Vec<_>>>()?;
                sails_idl_meta::TypeDef::Struct(StructDef { fields })
            }
            TypeDef::Variant(type_def_variant) => {
                let variants = type_def_variant
                    .variants
                    .iter()
                    .map(|v| {
                        let fields = v
                            .fields
                            .iter()
                            .map(|f| self.resolve_field(f, &type_params, &mut suffixes))
                            .collect::<Result<Vec<_>>>()?;
                        Ok(EnumVariant {
                            name: v.name.to_string(),
                            def: StructDef { fields },
                            docs: v.docs.iter().map(|d| d.to_string()).collect(),
                            annotations: vec![], // ("index".to_string(), Some(v.index.to_string()))
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                sails_idl_meta::TypeDef::Enum(EnumDef { variants })
            }
            _ => unreachable!(),
        };

        for suffix in suffixes {
            name.push_str(suffix.as_str());
        }

        if self.user_defined.contains_key(&name) {
            return Ok(name);
        }

        let meta_type = sails_idl_meta::Type {
            name: name.clone(),
            type_params,
            def,
            docs: ty.docs.iter().map(|d| d.to_string()).collect(),
            annotations: vec![], //("rust_type".to_string(), Some(ty.path.to_string()))
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

    pub(crate) fn resolve_type_params(
        &mut self,
        ty: &Type<PortableForm>,
    ) -> Result<Vec<sails_idl_meta::TypeParameter>> {
        ty.type_params
            .iter()
            .map(|tp| {
                let ty = match tp.ty {
                    Some(ref inner) => Some(self.resolve_by_id(inner.id)?),
                    None => None,
                };
                let name = tp.name.to_string();
                Ok(sails_idl_meta::TypeParameter { name, ty })
            })
            .collect()
    }

    fn unique_type_name(&self, ty: &Type<PortableForm>) -> Result<String, String> {
        for name in possible_names_by_path(ty) {
            if let Some(exists) = self.user_defined.get(&name) {
                if !exists.is_path_equals(ty) {
                    continue;
                } else if exists.is_fields_equal(ty) {
                    // type with exact fields already registered
                    return Err(name);
                } else {
                    return Ok(name);
                }
            }
            return Ok(name);
        }
        unreachable!();
    }

    fn resolve_user_defined(&mut self, name: String, ty: &Type<PortableForm>) -> Result<TypeDecl> {
        let generics = ty
            .type_params
            .iter()
            .map(|tp| {
                self.resolve_by_id(
                    tp.ty
                        .as_ref()
                        .ok_or(Error::TypeIsUnsupported(format!(
                            "Generic type parameter is unknown: {}",
                            tp.name
                        )))?
                        .id,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(TypeDecl::Named { name, generics })
    }

    fn resolve_field(
        &mut self,
        field: &Field<PortableForm>,
        type_params: &[sails_idl_meta::TypeParameter],
        suffixes: &mut BTreeSet<String>,
    ) -> Result<StructField> {
        let resolved = self.resolve_by_id(field.ty.id)?;
        let type_decl = if let Some(type_name) = field.type_name.as_ref()
            && &resolved.to_string() != type_name
        {
            let (td, suf) = crate::generic_resolver::resolve_generic_type_decl(
                &resolved,
                type_name,
                type_params,
            )?;
            suffixes.extend(suf);
            td
        } else {
            resolved
        };
        Ok(StructField {
            name: field.name.as_ref().map(|s| s.to_string()),
            type_decl,
            docs: field.docs.iter().map(|d| d.to_string()).collect(),
            annotations: vec![],
        })
    }

    fn resolve_known_composite(
        &mut self,
        ty: &Type<PortableForm>,
        _def: &TypeDefComposite<PortableForm>,
    ) -> Option<TypeDecl> {
        use PrimitiveType::*;
        use TypeDecl::*;

        if is_type::<gprimitives::H160>(ty) {
            Some(Primitive(H160))
        } else if is_type::<gprimitives::H256>(ty) {
            Some(Primitive(H256))
        } else if is_type::<gprimitives::U256>(ty) {
            Some(Primitive(U256))
        } else if is_type::<gprimitives::ActorId>(ty) {
            Some(Primitive(ActorId))
        } else if is_type::<gprimitives::CodeId>(ty) {
            Some(Primitive(CodeId))
        } else if is_type::<gprimitives::MessageId>(ty) {
            Some(Primitive(MessageId))
        } else if is_type::<Vec<()>>(ty)
            && let [vec_tp] = ty.type_params.as_slice()
            && let Some(ty) = vec_tp.ty
            && let Ok(ty) = self.resolve_by_id(ty.id)
        {
            Some(Slice { item: Box::new(ty) })
        } else if is_type::<BTreeMap<(), ()>>(ty)
            && let [key_tp, value_tp] = ty.type_params.as_slice()
            && let Some(key) = key_tp.ty
            && let Some(value) = value_tp.ty
            && let Ok(key) = self.resolve_by_id(key.id)
            && let Ok(value) = self.resolve_by_id(value.id)
        {
            Some(Slice {
                item: Box::new(TypeDecl::tuple(vec![key, value])),
            })
        } else {
            None
        }
    }

    fn resolve_known_enum(
        &mut self,
        ty: &Type<PortableForm>,
        def: &TypeDefVariant<PortableForm>,
    ) -> Option<TypeDecl> {
        if is_type::<core::result::Result<(), ()>>(ty)
            && let [ok_var, err_var] = def.variants.as_slice()
            && let [ok] = ok_var.fields.as_slice()
            && let [err] = err_var.fields.as_slice()
            && let Ok(ok) = self.resolve_by_id(ok.ty.id)
            && let Ok(err) = self.resolve_by_id(err.ty.id)
        {
            Some(TypeDecl::result(ok, err))
        } else if is_type::<core::option::Option<()>>(ty)
            && let [_, some_var] = def.variants.as_slice()
            && let [some] = some_var.fields.as_slice()
            && let Ok(decl) = self.resolve_by_id(some.ty.id)
        {
            Some(TypeDecl::option(decl))
        } else {
            None
        }
    }
}

fn is_type<T: StaticTypeInfo>(type_info: &Type<PortableForm>) -> bool {
    T::type_info().path.segments == type_info.path.segments
}

fn primitive_map(type_def_primitive: &TypeDefPrimitive) -> Result<PrimitiveType> {
    use PrimitiveType::*;

    let p = match type_def_primitive {
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
        TypeDefPrimitive::I256 => {
            return Err(Error::TypeIsUnsupported(
                "TypeDefPrimitive::I256 is unsupported".to_string(),
            ));
        }
    };
    Ok(p)
}

fn possible_names_by_path(ty: &Type<PortableForm>) -> impl Iterator<Item = String> + '_ {
    let mut name = String::default();
    ty.path.segments.iter().rev().map(move |segment| {
        name = segment.to_case(Case::Pascal) + &name;
        name.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};
    use gprimitives::NonZeroU256;
    use sails_idl_meta::TypeDef;
    use scale_info::{MetaType, Registry, TypeInfo};

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
        let _h160_id = registry
            .register_type(&MetaType::new::<gprimitives::H160>())
            .id;
        let _h160_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<gprimitives::H160>>())
            .id;

        let h256_id = registry
            .register_type(&MetaType::new::<gprimitives::H256>())
            .id;
        let h256_as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<gprimitives::H256>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);

        let resolver = TypeResolver::from_registry(&portable_registry);

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
        let u32_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<u32>>())
            .id;
        let string_struct_id = registry
            .register_type(&MetaType::new::<GenericStruct<String>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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
            .register_type(&MetaType::new::<Result<u32, String>>())
            .id;
        let as_generic_param_id = registry
            .register_type(&MetaType::new::<GenericStruct<Result<u32, String>>>())
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

    #[test]
    fn type_resolver_enum_many_variants() {
        let mut registry = Registry::new();
        let id = registry.register_type(&MetaType::new::<ManyVariants>()).id;
        let generic_id = registry
            .register_type(&MetaType::new::<GenericStruct<ManyVariants>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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
        let id = registry.register_type(&MetaType::new::<Test>()).id;
        let generic_id = registry
            .register_type(&MetaType::new::<GenericStruct<Test>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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
            let id = registry.register_type(&MetaType::new::<$primitive>()).id;
            let generic_id = registry
                .register_type(&MetaType::new::<GenericStruct<$primitive>>())
                .id;
            let portable_registry = PortableRegistry::from(registry);
            let resolver = TypeResolver::from_registry(&portable_registry);
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
        let t1_id = registry.register_type(&MetaType::new::<mod_1::T1>()).id;
        let t2_id = registry.register_type(&MetaType::new::<mod_2::T1>()).id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

        let t1_name = resolver.get(t1_id).unwrap().to_string();
        assert_eq!(t1_name, "T1");

        let t2_name = resolver.get(t2_id).unwrap().to_string();
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
        let resolver = TypeResolver::from_registry(&portable_registry);

        let t1_name = resolver.get(t1_id).unwrap().to_string();
        assert_eq!(t1_name, "T2");

        let t2_name = resolver.get(t2_id).unwrap().to_string();
        assert_eq!(t2_name, "Mod2T2");
    }

    #[test]
    fn generic_const_struct_type_name_resolution_works() {
        let mut registry = Registry::new();
        let n8_id = registry
            .register_type(&MetaType::new::<GenericConstStruct<8, 12, u8>>())
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
            .register_type(&MetaType::new::<GenericConstStruct<32, 8, gprimitives::U256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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
        let resolver = TypeResolver::from_registry(&portable_registry);

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
        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();

        if let scale_info::TypeDef::Composite(composite) = &struct_type.ty.type_def {
            let generic_value = composite
                .fields
                .iter()
                .find(|f| {
                    f.name
                        .as_ref()
                        .is_some_and(|s| s.to_string().eq("generic_value"))
                })
                .unwrap();
            assert_eq!(
                resolver.get(generic_value.ty.id).unwrap().to_string(),
                "u32"
            );

            let tuple_generic = composite
                .fields
                .iter()
                .find(|f| {
                    f.name
                        .as_ref()
                        .is_some_and(|s| s.to_string().eq("tuple_generic"))
                })
                .unwrap();
            assert_eq!(
                resolver.get(tuple_generic.ty.id).unwrap().to_string(),
                "(String, u32, u32, u32)"
            );

            let option_generic = composite
                .fields
                .iter()
                .find(|f| {
                    f.name
                        .as_ref()
                        .is_some_and(|s| s.to_string().eq("option_generic"))
                })
                .unwrap();
            assert_eq!(
                resolver.get(option_generic.ty.id).unwrap().to_string(),
                "Option<u32>"
            );

            let btreemap_generic = composite
                .fields
                .iter()
                .find(|f| {
                    f.name
                        .as_ref()
                        .is_some_and(|s| s.to_string().eq("btreemap_generic"))
                })
                .unwrap();
            assert_eq!(
                resolver.get(btreemap_generic.ty.id).unwrap().to_string(),
                "[(String, u32)]"
            );
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
        let struct_id = registry
            .register_type(&MetaType::new::<ComplexOneGenericStruct<bool>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<ComplexOneGenericEnum<bool>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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

        fn find_field_struct<'a>(
            composite: &'a TypeDefComposite<PortableForm>,
            name: &str,
        ) -> &'a Field<PortableForm> {
            composite
                .fields
                .iter()
                .find(|f| f.name.as_ref().is_some_and(|s| s.to_string().eq(name)))
                .unwrap_or_else(|| {
                    panic!("Field `{name}` not found. Fields: {:#?}", composite.fields)
                })
        }

        fn find_variant<'a>(
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
        let struct_id = registry
            .register_type(&MetaType::new::<MultiGenStruct<u32, String, H256>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<MultiGenEnum<u32, String, H256>>())
            .id;
        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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

        let struct_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == struct_id)
            .unwrap();
        if let scale_info::TypeDef::Composite(composite) = &struct_type.ty.type_def {
            let just_t1 = find_field_struct(composite, "just_t1");
            assert_eq!(resolver.get(just_t1.ty.id).unwrap().to_string(), "u32");

            let tuple_t2_t3 = find_field_struct(composite, "tuple_t2_t3");
            assert_eq!(
                resolver.get(tuple_t2_t3.ty.id).unwrap().to_string(),
                "(String, H256)"
            );

            let vec_t3 = find_field_struct(composite, "vec_t3");
            assert_eq!(resolver.get(vec_t3.ty.id).unwrap().to_string(), "[H256]");

            let array_triple = find_field_struct(composite, "array_triple");
            assert_eq!(
                resolver.get(array_triple.ty.id).unwrap().to_string(),
                "[[(u32, Option<String>)]; 3]"
            );
        } else {
            panic!("Expected composite type");
        }

        let enum_type = portable_registry
            .types
            .iter()
            .find(|t| t.id == enum_id)
            .unwrap();
        if let scale_info::TypeDef::Variant(variant) = &enum_type.ty.type_def {
            // check a representative tuple-like variant concrete names
            let tuple_t2_t3_variant = find_variant(&variant.variants, "TupleT2T3");
            let f0 = &tuple_t2_t3_variant.fields[0];
            assert_eq!(
                resolver.get(f0.ty.id).unwrap().to_string(),
                "(String, H256)"
            );

            // check option/result shaped variant
            let tuple_of_result_variant = find_variant(&variant.variants, "TupleOfResult");
            let field1 = tuple_of_result_variant
                .fields
                .iter()
                .find(|f| f.name.as_ref().is_some_and(|s| s.to_string().eq("field1")))
                .unwrap();
            assert_eq!(
                resolver.get(field1.ty.id).unwrap().to_string(),
                "Result<u32, String>"
            );
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
        let resolver = TypeResolver::from_registry(&portable_registry);

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
        let struct_id = registry
            .register_type(&MetaType::new::<TestStruct<u32, bool>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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
        let struct_id = registry
            .register_type(&MetaType::new::<ReuseTestStruct<u64, H256>>())
            .id;
        let enum_id = registry
            .register_type(&MetaType::new::<ReuseTestEnum<u64, H256>>())
            .id;

        let portable_registry = PortableRegistry::from(registry);
        let resolver = TypeResolver::from_registry(&portable_registry);

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
