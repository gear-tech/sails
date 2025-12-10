use super::*;
use convert_case::{Case, Casing};
use scale_info::{
    Field, Path, PortableRegistry, StaticTypeInfo, Type, TypeDef, TypeDefComposite,
    TypeDefPrimitive, TypeDefVariant, form::PortableForm,
};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TypeResolver<'a> {
    registry: &'a PortableRegistry,
    map: HashMap<u32, TypeDecl>,
    user_defined: HashMap<String, UserDefinedEntry>,
}

#[derive(Debug, Clone)]
pub struct UserDefinedEntry {
    pub meta_type: sails_idl_meta::Type,
    pub path: Path<PortableForm>,
}

impl UserDefinedEntry {
    fn is_type(&self, type_info: &Type<PortableForm>) -> bool {
        self.path == type_info.path
    }
}

impl<'a> TypeResolver<'a> {
    #[cfg(test)]
    pub fn from_registry(registry: &'a PortableRegistry) -> Self {
        TypeResolver::try_from(registry, HashSet::new()).unwrap()
    }

    pub fn try_from(registry: &'a PortableRegistry, exclude: HashSet<u32>) -> Result<Self> {
        let mut resolver = Self {
            registry,
            map: HashMap::new(),
            user_defined: HashMap::new(),
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

    fn build_type_decl_map(&mut self, exclude: HashSet<u32>) -> Result<()> {
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
        let name = match self.unique_type_name(ty) {
            Ok(name) => name,
            Err(exist) => return Ok(exist),
        };

        let type_params = self.resolve_type_params(ty)?;

        let def = match &ty.type_def {
            TypeDef::Composite(type_def_composite) => {
                let fields = type_def_composite
                    .fields
                    .iter()
                    .map(|f| self.resolve_field(f, &type_params))
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
                            .map(|f| self.resolve_field(f, &type_params))
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

        let meta_type = sails_idl_meta::Type {
            name: name.clone(),
            type_params,
            def,
            docs: ty.docs.iter().map(|d| d.to_string()).collect(),
            annotations: vec![], //("rust_type".to_string(), Some(ty.path.to_string()))
        };
        let path = ty.path.clone();
        self.user_defined
            .insert(name.clone(), UserDefinedEntry { meta_type, path });
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
                if exists.is_type(ty) {
                    // type already registered
                    return Err(name);
                } else {
                    continue;
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
        type_params: &Vec<sails_idl_meta::TypeParameter>,
    ) -> Result<StructField> {
        let resolved = self.resolve_by_id(field.ty.id)?;
        let type_decl = if let Some(type_name) = field.type_name.as_ref()
            && &resolved.to_string() != type_name
            && !type_params.is_empty()
        {
            crate::generic_resolver::resolve_generic_type_decl(&resolved, type_name, type_params)?
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
    use gprimitives::NonZeroU256;
    use scale_info::{MetaType, Registry, TypeInfo};
    use std::num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128};

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
        println!("{resolver:#?}");

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
        println!("{resolver:#?}");

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
        println!("{resolver:#?}");

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
        println!("{resolver:#?}");

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
        println!("{resolver:#?}");

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
        // println!("{:#?}", resolver);

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
}
