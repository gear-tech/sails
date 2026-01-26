use super::*;
use crate::type_resolver::TypeResolver;
use scale_info::*;

pub struct ProgramBuilder {
    registry: PortableRegistry,
    ctors_type_id: u32,
    service_expos: Vec<(&'static str, AnyServiceMeta)>,
}

impl ProgramBuilder {
    /// Build a new program builder with the constructors and services registered in metadata.
    pub fn new<P: ProgramMeta>() -> Self {
        let mut registry = Registry::new();
        let ctors = P::constructors();
        let ctors_type_id = registry.register_type(&ctors).id;
        let service_expos = P::services().collect();

        let registry = PortableRegistry::from(registry);
        Self {
            registry,
            ctors_type_id,
            service_expos,
        }
    }

    fn ctor_funcs(&self, resolver: &TypeResolver) -> Result<Vec<CtorFunc>> {
        any_funcs(&self.registry, self.ctors_type_id)?
            .map(|c| {
                if c.fields.len() != 1 {
                    Err(Error::MetaIsInvalid(format!(
                        "ctor `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type_id = c.fields[0].ty.id;
                    let params_type = &self
                        .registry
                        .resolve(params_type_id)
                        .ok_or(Error::TypeIdIsUnknown(params_type_id))?;
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        let params = params_type
                            .fields
                            .iter()
                            .map(|f| -> Result<_> {
                                let name = f.name.as_ref().ok_or_else(|| {
                                    Error::MetaIsInvalid(format!(
                                        "ctor `{}` param is missing a name",
                                        c.name
                                    ))
                                })?;
                                let type_decl = resolver
                                    .get(f.ty.id)
                                    .cloned()
                                    .ok_or(Error::TypeIdIsUnknown(f.ty.id))?;
                                Ok(FuncParam {
                                    name: name.to_string(),
                                    type_decl,
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Ok(CtorFunc {
                            name: c.name.to_string(),
                            params,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
                            annotations: vec![],
                        })
                    } else {
                        Err(Error::MetaIsInvalid(format!(
                            "ctor `{}` params type is not a composite",
                            c.name
                        )))
                    }
                }
            })
            .collect()
    }

    /// Assemble the final `ProgramUnit` from resolved constructors, types, and service exports.
    pub fn build(self, name: String, services: &[ServiceUnit]) -> Result<ProgramUnit> {
        let mut exclude = BTreeSet::new();
        exclude.insert(self.ctors_type_id);
        exclude.extend(any_funcs_ids(&self.registry, self.ctors_type_id)?);
        let resolver = TypeResolver::try_from(&self.registry, exclude)?;
        let ctors = self.ctor_funcs(&resolver)?;
        let types = resolver.into_types();

        if self.service_expos.len() > u8::MAX as usize {
            return Err(Error::MetaIsInvalid(
                "Too many services in program. Max: 255".to_string(),
            ));
        }

        let expos: Result<Vec<_>> = self
            .service_expos
            .into_iter()
            .enumerate()
            .map(|(idx, (route, meta))| {
                let interface_id = meta.interface_id();
                let Some(service) = services
                    .iter()
                    .find(|s| s.name.interface_id == Some(interface_id))
                else {
                    return Err(Error::MetaIsInvalid(format!(
                        "service `{route}@{interface_id}` not defined"
                    )));
                };
                let route = if service.name.name == route {
                    None
                } else {
                    Some(route.to_string())
                };

                Ok(ServiceExpo {
                    name: service.name.clone(),
                    route,
                    route_idx: (idx as u8) + 1,
                    docs: vec![],
                    annotations: vec![],
                })
            })
            .collect();

        Ok(ProgramUnit {
            name,
            ctors,
            services: expos?,
            types,
            docs: vec![],
            annotations: vec![],
        })
    }
}

fn any_funcs(
    registry: &PortableRegistry,
    func_type_id: u32,
) -> Result<impl Iterator<Item = &Variant<PortableForm>>> {
    let funcs = registry
        .resolve(func_type_id)
        .ok_or(Error::TypeIdIsUnknown(func_type_id))?;
    if let scale_info::TypeDef::Variant(variant) = &funcs.type_def {
        Ok(variant.variants.iter())
    } else {
        Err(Error::MetaIsInvalid(format!(
            "func type id {func_type_id} references a type that is not a variant"
        )))
    }
}

fn any_funcs_ids(registry: &PortableRegistry, func_type_id: u32) -> Result<Vec<u32>> {
    any_funcs(registry, func_type_id)?
        .map(|variant| {
            variant
                .fields
                .first()
                .map(|field| field.ty.id)
                .ok_or_else(|| {
                    Error::MetaIsInvalid(format!("func `{}` has no fields", variant.name))
                })
        })
        .collect::<Result<Vec<_>>>()
}

pub struct ServiceBuilder<'a> {
    name: &'a str,
    meta: &'a AnyServiceMeta,
    registry: PortableRegistry,
    commands_type_id: u32,
    queries_type_id: u32,
    events_type_id: u32,
}

impl<'a> ServiceBuilder<'a> {
    /// Create a builder for a single service name + metadata pair.
    pub fn new(name: &'a str, meta: &'a AnyServiceMeta) -> Self {
        let mut registry = Registry::new();
        let commands_type_id = registry.register_type(meta.commands()).id;
        let queries_type_id = registry.register_type(meta.queries()).id;
        let events_type_id = registry.register_type(meta.events()).id;
        let registry = PortableRegistry::from(registry);
        Self {
            name,
            meta,
            registry,
            commands_type_id,
            queries_type_id,
            events_type_id,
        }
    }

    /// Build this service (and its base services) into the shared service list.
    pub fn build(self, services: &mut Vec<ServiceUnit>) -> Result<ServiceIdent> {
        let mut visited = BTreeSet::new();
        self.build_inner(services, &mut visited)
    }

    fn build_inner(
        self,
        services: &mut Vec<ServiceUnit>,
        visited: &mut BTreeSet<u64>,
    ) -> core::result::Result<ServiceIdent, Error> {
        let interface_id = self.meta.interface_id();
        if let Some(service) = services
            .iter()
            .find(|s| s.name.interface_id == Some(interface_id))
        {
            // if service already built, return ident
            return Ok(service.name.clone());
        }

        // cycle detection for base service recursion
        let key = u64::from_le_bytes(interface_id.0);
        if !visited.insert(key) {
            return Err(Error::MetaIsInvalid(format!(
                "service `{}` has cyclic base services",
                ServiceIdent {
                    name: self.name.to_string(),
                    interface_id: Some(interface_id),
                }
            )));
        }

        let mut extends = Vec::new();
        for (name, meta) in self.meta.base_services() {
            let ident = ServiceBuilder::new(name, &meta).build_inner(services, visited)?;
            extends.push(ident);
        }

        let exclude = BTreeSet::from_iter(self.exclude_type_ids()?);
        let resolver = TypeResolver::try_from(&self.registry, exclude)?;
        let commands = self.commands(&resolver)?;
        let queries = self.queries(&resolver)?;
        let events = self.events(&resolver)?;
        let types = resolver.into_types();

        let ident = ServiceIdent {
            name: self.name.to_string(),
            interface_id: Some(interface_id),
        };
        let mut unit = ServiceUnit {
            name: ident.clone(),
            extends,
            funcs: [commands, queries].concat(),
            events,
            types,
            docs: vec![],
            annotations: vec![],
        };
        unit.normalize();
        // assert_eq!(unit.interface_id(), Ok(interface_id));
        services.push(unit);
        visited.remove(&key);
        Ok(ident)
    }

    fn exclude_type_ids(&self) -> Result<impl Iterator<Item = u32>> {
        let base = vec![
            self.commands_type_id,
            self.queries_type_id,
            self.events_type_id,
        ]
        .into_iter();
        let command_ids = any_funcs_ids(&self.registry, self.commands_type_id)?;
        let query_ids = any_funcs_ids(&self.registry, self.queries_type_id)?;
        Ok(base.chain(command_ids).chain(query_ids))
    }

    fn commands(&self, resolver: &TypeResolver) -> Result<Vec<ServiceFunc>> {
        any_funcs(&self.registry, self.commands_type_id)?
            .map(|c| {
                if c.fields.len() != 2 && c.fields.len() != 3 {
                    Err(Error::MetaIsInvalid(format!(
                        "command `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type_id = c.fields[0].ty.id;
                    let params_type = self
                        .registry
                        .resolve(params_type_id)
                        .ok_or(Error::TypeIdIsUnknown(params_type_id))?;
                    let output_type_id = c.fields[1].ty.id;
                    let output = resolver
                        .get(output_type_id)
                        .cloned()
                        .ok_or(Error::TypeIdIsUnknown(output_type_id))?;
                    let throws = if c.fields.len() == 3 {
                        let throws_type_id = c.fields[2].ty.id;
                        let throws = resolver
                            .get(throws_type_id)
                            .cloned()
                            .ok_or(Error::TypeIdIsUnknown(throws_type_id))?;
                        Some(throws)
                    } else {
                        None
                    };
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        let params = params_type
                            .fields
                            .iter()
                            .map(|f| -> Result<_> {
                                let name = f.name.as_ref().ok_or_else(|| {
                                    Error::MetaIsInvalid(format!(
                                        "command `{}` param is missing a name",
                                        c.name
                                    ))
                                })?;
                                let type_decl = resolver
                                    .get(f.ty.id)
                                    .cloned()
                                    .ok_or(Error::TypeIdIsUnknown(f.ty.id))?;
                                Ok(FuncParam {
                                    name: name.to_string(),
                                    type_decl,
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Ok(ServiceFunc {
                            name: c.name.to_string(),
                            params,
                            output,
                            throws,
                            kind: FunctionKind::Command,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
                            annotations: vec![],
                        })
                    } else {
                        Err(Error::MetaIsInvalid(format!(
                            "command `{}` params type is not a composite",
                            c.name
                        )))
                    }
                }
            })
            .collect()
    }

    fn queries(&self, resolver: &TypeResolver) -> Result<Vec<ServiceFunc>> {
        any_funcs(&self.registry, self.queries_type_id)?
            .map(|c| {
                if c.fields.len() != 2 && c.fields.len() != 3 {
                    Err(Error::MetaIsInvalid(format!(
                        "query `{}` has invalid number of fields",
                        c.name
                    )))
                } else {
                    let params_type_id = c.fields[0].ty.id;
                    let params_type = self
                        .registry
                        .resolve(params_type_id)
                        .ok_or(Error::TypeIdIsUnknown(params_type_id))?;
                    let output_type_id = c.fields[1].ty.id;
                    let output = resolver
                        .get(output_type_id)
                        .cloned()
                        .ok_or(Error::TypeIdIsUnknown(output_type_id))?;
                    let throws = if c.fields.len() == 3 {
                        let throws_type_id = c.fields[2].ty.id;
                        let throws = resolver
                            .get(throws_type_id)
                            .cloned()
                            .ok_or(Error::TypeIdIsUnknown(throws_type_id))?;
                        Some(throws)
                    } else {
                        None
                    };
                    if let scale_info::TypeDef::Composite(params_type) = &params_type.type_def {
                        let params = params_type
                            .fields
                            .iter()
                            .map(|f| -> Result<_> {
                                let name = f.name.as_ref().ok_or_else(|| {
                                    Error::MetaIsInvalid(format!(
                                        "query `{}` param is missing a name",
                                        c.name
                                    ))
                                })?;
                                let type_decl = resolver
                                    .get(f.ty.id)
                                    .cloned()
                                    .ok_or(Error::TypeIdIsUnknown(f.ty.id))?;
                                Ok(FuncParam {
                                    name: name.to_string(),
                                    type_decl,
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        Ok(ServiceFunc {
                            name: c.name.to_string(),
                            params,
                            output,
                            throws,
                            kind: FunctionKind::Query,
                            docs: c.docs.iter().map(|s| s.to_string()).collect(),
                            annotations: vec![("query".to_string(), None)],
                        })
                    } else {
                        Err(Error::MetaIsInvalid(format!(
                            "query `{}` params type is not a composite",
                            c.name
                        )))
                    }
                }
            })
            .collect()
    }

    fn events(&self, resolver: &TypeResolver) -> Result<Vec<ServiceEvent>> {
        any_funcs(&self.registry, self.events_type_id)?
            .map(|v| {
                let fields = v
                    .fields
                    .iter()
                    .map(|field| -> Result<_> {
                        let type_decl = resolver
                            .get(field.ty.id)
                            .cloned()
                            .ok_or(Error::TypeIdIsUnknown(field.ty.id))?;
                        Ok(StructField {
                            name: field.name.as_ref().map(|s| s.to_string()),
                            type_decl,
                            docs: field.docs.iter().map(|d| d.to_string()).collect(),
                            annotations: vec![],
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(ServiceEvent {
                    name: v.name.to_string(),
                    def: StructDef { fields },
                    docs: v.docs.iter().map(|d| d.to_string()).collect(),
                    annotations: vec![],
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::marker::PhantomData;
    use core::num::NonZeroU128;
    use gprimitives::{ActorId, CodeId, H160, H256, MessageId, NonZeroU256, U256};
    use scale_info::TypeInfo;

    mod utils {
        use super::*;

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum SimpleCtors {
            SimpleCtor(SimpleCtorParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) struct SimpleCtorParams {
            f1: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) struct SimpleFunctionParams {
            f1: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum NoCommands {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum NoQueries {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        pub(super) enum NoEvents {}
    }

    fn test_program_unit<T: StaticTypeInfo>() -> Result<ProgramUnit> {
        struct TestProgram<C: StaticTypeInfo>(PhantomData<C>);
        impl<C: StaticTypeInfo> ProgramMeta for TestProgram<C> {
            type ConstructorsMeta = C;
            const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[];
            const ASYNC: bool = false;
        }

        let services = Vec::new();
        ProgramBuilder::new::<TestProgram<T>>().build("TestProgram".to_string(), &services)
    }

    fn test_service_units<S: ServiceMeta>(service_name: &'static str) -> Result<Vec<ServiceUnit>> {
        let meta = AnyServiceMeta::new::<S>();
        let mut services = Vec::new();
        ServiceBuilder::new(service_name, &meta).build(&mut services)?;
        Ok(services)
    }

    // ------------------------------------------------------------------------------------
    // -------------------------- Program section related tests ---------------------------
    // ------------------------------------------------------------------------------------

    /// Test various constructor validation errors
    #[test]
    fn ctor_validation_errors() {
        // Define all constructor error test types
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NonCompositeArgsCtors {
            CtorWithInvalidArgTypes(u32), // u32 is not composite, should cause error
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NamelessFieldsCtors {
            CtorWithNamelessArgs(NamelessFieldParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NamelessFieldParams(u32, String);

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NoArgsCtors {
            CtorWithNoArgs,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum TooManyArgsCtors {
            CtorWithResult(ValidParams, String), // Should have exactly 1 field, not 2
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ValidParams {
            pub param1: u32,
        }

        // Helper function to test constructor validation errors
        fn test_ctor_error<T: StaticTypeInfo>(expected_error_msg: &str) {
            let result = test_program_unit::<T>();

            assert!(result.is_err());
            let Err(Error::MetaIsInvalid(msg)) = result else {
                panic!("Expected MetaIsInvalid error, got {result:?}");
            };
            assert_eq!(msg.as_str(), expected_error_msg);
        }

        // Test all error scenarios
        test_ctor_error::<NonCompositeArgsCtors>(
            "ctor `CtorWithInvalidArgTypes` params type is not a composite",
        );

        test_ctor_error::<NamelessFieldsCtors>(
            "ctor `CtorWithNamelessArgs` param is missing a name",
        );

        test_ctor_error::<NoArgsCtors>("func `CtorWithNoArgs` has no fields");

        test_ctor_error::<TooManyArgsCtors>("ctor `CtorWithResult` has invalid number of fields");
    }

    /// Test that returned program meta has result_ty == None for all constructors in program IDL section
    #[test]
    fn ctors_build_works() {
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ValidConstructors {
            Zero(ZeroParams),
            One(OneParam),
            Three(ThreeParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ZeroParams {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct OneParam {
            pub actor: ActorId,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ThreeParams {
            pub code: CodeId,
            pub name: String,
            pub num: NonZeroU256,
        }

        let meta = test_program_unit::<ValidConstructors>().expect("ProgramBuilder error");

        // Check that all constructors have parsed
        assert_eq!(meta.ctors.len(), 3);
    }

    /// Test successful creation with valid constructors and services
    #[test]
    fn ctor_simple_positive_test() {
        use TypeDecl::*;

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Ctors {
            Ctor(InitParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct InitParams {
            pub initial_value: u32,
        }

        let meta = test_program_unit::<Ctors>().expect("ProgramBuilder error");

        assert_eq!(
            meta.ctors,
            vec![CtorFunc {
                name: "Ctor".to_string(),
                params: vec![FuncParam {
                    name: "initial_value".to_string(),
                    type_decl: Primitive(PrimitiveType::U32)
                }],
                docs: vec![],
                annotations: vec![]
            }]
        );
    }

    #[test]
    fn program_has_services() {
        struct TestService;
        impl Identifiable for TestService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(1);
        }

        impl ServiceMeta for TestService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct TestProgram;
        impl ProgramMeta for TestProgram {
            type ConstructorsMeta = utils::SimpleCtors;
            const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
                ("TestService1", AnyServiceMeta::new::<TestService>),
                ("TestService2", AnyServiceMeta::new::<TestService>),
                ("TestService3", AnyServiceMeta::new::<TestService>),
            ];
            const ASYNC: bool = false;
        }

        let services =
            test_service_units::<TestService>("TestService1").expect("ServiceBuilder error");

        let meta = ProgramBuilder::new::<TestProgram>()
            .build("TestProgram".to_string(), &services)
            .expect("ProgramBuilder error");

        assert_eq!(meta.services.len(), 3);
        assert_eq!(
            meta.services[0],
            ServiceExpo {
                name: ServiceIdent {
                    name: "TestService1".to_string(),
                    interface_id: Some(InterfaceId::from_u64(1))
                },
                route: None,
                route_idx: 1,
                docs: vec![],
                annotations: vec![]
            }
        );
        assert_eq!(
            meta.services[1],
            ServiceExpo {
                name: ServiceIdent {
                    name: "TestService1".to_string(),
                    interface_id: Some(InterfaceId::from_u64(1))
                },
                route: Some("TestService2".to_string()),
                route_idx: 2,
                docs: vec![],
                annotations: vec![]
            }
        );
        assert_eq!(
            meta.services[2],
            ServiceExpo {
                name: ServiceIdent {
                    name: "TestService1".to_string(),
                    interface_id: Some(InterfaceId::from_u64(1))
                },
                route: Some("TestService3".to_string()),
                route_idx: 3,
                docs: vec![],
                annotations: vec![]
            }
        );
    }

    // #[test]
    // #[ignore = "TODO"]
    // fn program_has_same_name_services() {
    //     struct TestService;
    //     impl ServiceMeta for TestService {
    //         type CommandsMeta = utils::NoCommands;
    //         type QueriesMeta = utils::NoQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
    //         const ASYNC: bool = false;
    //
    //     }

    //     struct TestProgram;
    //     impl ProgramMeta for TestProgram {
    //         type ConstructorsMeta = utils::SimpleCtors;
    //         const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
    //             ("TestService", AnyServiceMeta::new::<TestService>),
    //             ("TestService", AnyServiceMeta::new::<TestService>),
    //         ];
    //         const ASYNC: bool = false;
    //     }

    //     let meta = ProgramBuilder::new::<TestProgram>()
    //         .build("TestProgram".to_string())
    //         .expect("ProgramBuilder error");

    //     assert_eq!(meta.services.len(), 2);
    //     assert_eq!(
    //         meta.services[0],
    //         ServiceExpo {
    //             name: "TestService".to_string(),
    //             route: None,
    //             docs: vec![],
    //             annotations: vec![]
    //         }
    //     );
    //     assert_eq!(
    //         meta.services[1],
    //         ServiceExpo {
    //             name: "TestService".to_string(),
    //             route: Some("TestService".to_string()),
    //             docs: vec![],
    //             annotations: vec![]
    //         }
    //     );
    // }

    #[test]
    fn program_section_has_types_section() {
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Ctors {
            Ctor1(Ctor1Params),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct Ctor1Params {
            pub param1: u32,
            pub param2: ActorId,
            pub param3: CtorType,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct CtorType(String);

        let meta = test_program_unit::<Ctors>().expect("ProgramBuilder error");

        assert_eq!(meta.types.len(), 1);
        assert!(matches!(
            meta.types.first(),
            Some(sails_idl_meta::Type { name, .. }) if name == "CtorType"
        ));
    }

    // ------------------------------------------------------------------------------------
    // -------------------- Extension and base services related tests ---------------------
    // ------------------------------------------------------------------------------------

    #[test]
    fn base_service_entities_doesnt_automatically_occur() {
        struct BaseService;
        impl Identifiable for BaseService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(1u64);
        }

        impl ServiceMeta for BaseService {
            type CommandsMeta = BaseServiceCommands;
            type QueriesMeta = BaseServiceQueries;
            type EventsMeta = BaseServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ExtendedService;
        impl Identifiable for ExtendedService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(2u64);
        }

        impl ServiceMeta for ExtendedService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = ExtendedServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("BaseService")];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BaseServiceCommands {
            BaseCmd(BaseServiceFunctionParams, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BaseServiceQueries {
            BaseQuery(BaseServiceFunctionParams, ActorId),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BaseServiceEvents {
            BaseEvent(NonZeroU128),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct BaseServiceFunctionParams {
            param: SomeBaseServiceType,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct SomeBaseServiceType(ActorId);

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ExtendedServiceEvents {
            ExtendedEvent(SomeExtendedServiceType),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct SomeExtendedServiceType(CodeId);

        let services =
            test_service_units::<ExtendedService>("ExtendedService").expect("ServiceBuilder error");

        assert_eq!(services.len(), 2);
        let base_service = &services[0];
        let extended_service = &services[1];

        assert_eq!(base_service.name.name, "BaseService");
        assert_eq!(extended_service.name.name, "ExtendedService");

        assert_eq!(
            extended_service.extends,
            vec![ServiceIdent {
                name: "BaseService".to_string(),
                interface_id: Some(<BaseService as Identifiable>::INTERFACE_ID)
            }]
        );

        assert_eq!(base_service.funcs.len(), 2);
        assert!(
            base_service
                .funcs
                .iter()
                .any(|f| f.kind == FunctionKind::Command && f.name == "BaseCmd")
        );
        assert!(
            base_service
                .funcs
                .iter()
                .any(|f| f.kind == FunctionKind::Query && f.name == "BaseQuery")
        );

        assert!(extended_service.funcs.is_empty());

        let base_events: Vec<&str> = base_service
            .events
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        let extended_events: Vec<&str> = extended_service
            .events
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(base_events, vec!["BaseEvent"]);
        assert_eq!(extended_events, vec!["ExtendedEvent"]);

        let base_types: Vec<&str> = base_service.types.iter().map(|t| t.name.as_str()).collect();
        let extended_types: Vec<&str> = extended_service
            .types
            .iter()
            .map(|t| t.name.as_str())
            .collect();
        assert_eq!(base_types, vec!["NonZeroU128", "SomeBaseServiceType"]);
        assert_eq!(extended_types, vec!["SomeExtendedServiceType"]);
    }

    #[test]
    fn service_extension_with_conflicting_names() {
        struct BaseService;
        impl Identifiable for BaseService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(10u64);
        }

        impl ServiceMeta for BaseService {
            type CommandsMeta = BaseServiceCommands;
            type QueriesMeta = BaseServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ExtendedService;
        impl Identifiable for ExtendedService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(11u64);
        }

        impl ServiceMeta for ExtendedService {
            type CommandsMeta = ExtendedServiceCommands;
            type QueriesMeta = ExtendedServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("BaseService")];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BaseServiceCommands {
            ConflictingCmd(utils::SimpleFunctionParams, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BaseServiceQueries {
            ConflictingQuery(utils::SimpleFunctionParams, u32),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ExtendedServiceCommands {
            ConflictingCmd(utils::SimpleFunctionParams, bool),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ExtendedServiceQueries {
            ConflictingQuery(utils::SimpleFunctionParams, String),
        }

        let services =
            test_service_units::<ExtendedService>("ExtendedService").expect("ServiceBuilder error");

        assert_eq!(services.len(), 2);
        let base_service = &services[0];
        let extended_service = &services[1];

        assert_eq!(base_service.name.name, "BaseService");
        assert_eq!(extended_service.name.name, "ExtendedService");

        let base_cmd = base_service
            .funcs
            .iter()
            .find(|f| f.kind == FunctionKind::Command && f.name == "ConflictingCmd")
            .expect("missing base command");
        let extended_cmd = extended_service
            .funcs
            .iter()
            .find(|f| f.kind == FunctionKind::Command && f.name == "ConflictingCmd")
            .expect("missing extended command");

        assert_eq!(base_cmd.output, TypeDecl::Primitive(PrimitiveType::String));
        assert_eq!(
            extended_cmd.output,
            TypeDecl::Primitive(PrimitiveType::Bool)
        );

        let base_query = base_service
            .funcs
            .iter()
            .find(|f| f.kind == FunctionKind::Query && f.name == "ConflictingQuery")
            .expect("missing base query");
        let extended_query = extended_service
            .funcs
            .iter()
            .find(|f| f.kind == FunctionKind::Query && f.name == "ConflictingQuery")
            .expect("missing extended query");

        assert_eq!(base_query.output, TypeDecl::Primitive(PrimitiveType::U32));
        assert_eq!(
            extended_query.output,
            TypeDecl::Primitive(PrimitiveType::String)
        );
    }

    #[test]
    fn service_extension_with_conflicting_events() {
        struct BaseService;
        impl Identifiable for BaseService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(20u64);
        }

        impl ServiceMeta for BaseService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = BaseServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ExtendedService;
        impl Identifiable for ExtendedService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(21u64);
        }

        impl ServiceMeta for ExtendedService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = ExtendedServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("BaseService")];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BaseServiceEvents {
            ConflictingEvent(u32),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ExtendedServiceEvents {
            ConflictingEvent(String),
        }

        let services =
            test_service_units::<ExtendedService>("ExtendedService").expect("ServiceBuilder error");

        assert_eq!(services.len(), 2);
        let base_service = &services[0];
        let extended_service = &services[1];

        assert_eq!(base_service.name.name, "BaseService");
        assert_eq!(extended_service.name.name, "ExtendedService");

        let base_event = base_service
            .events
            .iter()
            .find(|e| e.name == "ConflictingEvent")
            .expect("missing base event");
        let extended_event = extended_service
            .events
            .iter()
            .find(|e| e.name == "ConflictingEvent")
            .expect("missing extended event");

        assert_eq!(base_event.def.fields.len(), 1);
        assert_eq!(
            base_event.def.fields[0].type_decl,
            TypeDecl::Primitive(PrimitiveType::U32)
        );

        assert_eq!(extended_event.def.fields.len(), 1);
        assert_eq!(
            extended_event.def.fields[0].type_decl,
            TypeDecl::Primitive(PrimitiveType::String)
        );
    }

    #[test]
    fn service_extension_with_conflicting_types() {
        struct ServiceBase;
        impl Identifiable for ServiceBase {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(30u64);
        }

        impl ServiceMeta for ServiceBase {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = BaseServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[allow(unused)]
        #[derive(TypeInfo)]
        enum BaseServiceEvents {
            BaseEvent(GenericConstStruct<8>),
        }

        struct ExtensionService;
        impl Identifiable for ExtensionService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(31u64);
        }

        impl ServiceMeta for ExtensionService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = ExtendedServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<ServiceBase>("ServiceBase")];
            const ASYNC: bool = false;
        }

        #[allow(unused)]
        #[derive(TypeInfo)]
        enum ExtendedServiceEvents {
            ExtEvent(GenericConstStruct<16>),
        }

        #[allow(unused)]
        #[derive(TypeInfo)]
        struct GenericConstStruct<const N: usize>([u8; N]);

        let services = test_service_units::<ExtensionService>("ExtensionService")
            .expect("ServiceBuilder error");

        assert_eq!(services.len(), 2);
        let base_service = &services[0];
        let ext_service = &services[1];

        assert_eq!(base_service.types.len(), 1);
        assert_eq!(ext_service.types.len(), 1);

        let base_ty = &base_service.types[0];
        assert!(base_ty.name.starts_with("GenericConstStruct"));
        let sails_idl_meta::TypeDef::Struct(base_struct_def) = &base_ty.def else {
            panic!("expected struct type");
        };
        assert_eq!(base_struct_def.fields.len(), 1);
        assert_eq!(base_struct_def.fields[0].type_decl.to_string(), "[u8; 8]");

        let ext_ty = &ext_service.types[0];
        assert!(ext_ty.name.starts_with("GenericConstStruct"));
        let sails_idl_meta::TypeDef::Struct(ext_struct_def) = &ext_ty.def else {
            panic!("expected struct type");
        };
        assert_eq!(ext_struct_def.fields.len(), 1);
        assert_eq!(ext_struct_def.fields[0].type_decl.to_string(), "[u8; 16]");
    }

    #[test]
    fn service_extension_order() {
        struct ServiceA1;
        impl Identifiable for ServiceA1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(40u64);
        }

        impl ServiceMeta for ServiceA1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceA2;
        impl Identifiable for ServiceA2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(41u64);
        }

        impl ServiceMeta for ServiceA2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceB2;
        impl Identifiable for ServiceB2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(42u64);
        }

        impl ServiceMeta for ServiceB2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[
                BaseServiceMeta::new::<ServiceA1>("ServiceA1"),
                BaseServiceMeta::new::<ServiceA2>("ServiceA2"),
            ];
            const ASYNC: bool = false;
        }

        struct ServiceB1;
        impl Identifiable for ServiceB1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(43u64);
        }

        impl ServiceMeta for ServiceB1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceC;
        impl Identifiable for ServiceC {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(44u64);
        }

        impl ServiceMeta for ServiceC {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[
                BaseServiceMeta::new::<ServiceB1>("ServiceB1"),
                BaseServiceMeta::new::<ServiceB2>("ServiceB2"),
            ];
            const ASYNC: bool = false;
        }

        let services = test_service_units::<ServiceC>("ServiceC").expect("ServiceBuilder error");

        assert_eq!(services.len(), 5);
        let names: Vec<_> = services.iter().map(|s| s.name.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "ServiceB1",
                "ServiceA1",
                "ServiceA2",
                "ServiceB2",
                "ServiceC"
            ]
        );
    }

    #[test]
    fn no_repeated_base_services() {
        struct BaseService;
        impl Identifiable for BaseService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(50u64);
        }

        impl ServiceMeta for BaseService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service1;
        impl Identifiable for Service1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(51u64);
        }

        impl ServiceMeta for Service1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("BaseService")];
            const ASYNC: bool = false;
        }

        struct Service2;
        impl Identifiable for Service2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(52u64);
        }

        impl ServiceMeta for Service2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("BaseService")];
            const ASYNC: bool = false;
        }

        struct TestProgram;
        impl ProgramMeta for TestProgram {
            type ConstructorsMeta = utils::SimpleCtors;
            const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
                ("Service1", AnyServiceMeta::new::<Service1>),
                ("Service2", AnyServiceMeta::new::<Service2>),
            ];
            const ASYNC: bool = false;
        }

        let doc = build_program_ast::<TestProgram>(None).unwrap();
        assert_eq!(doc.services.len(), 3);
    }

    #[test]
    fn no_repeated_base_services_with_renaming() {
        struct BaseService;
        impl Identifiable for BaseService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(60u64);
        }

        impl ServiceMeta for BaseService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service1;
        impl Identifiable for Service1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(61u64);
        }

        impl ServiceMeta for Service1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("BaseService")];
            const ASYNC: bool = false;
        }

        struct Service2;
        impl Identifiable for Service2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(62u64);
        }

        impl ServiceMeta for Service2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] =
                &[BaseServiceMeta::new::<BaseService>("RenamedBaseService")];
            const ASYNC: bool = false;
        }

        struct TestProgram;
        impl ProgramMeta for TestProgram {
            type ConstructorsMeta = utils::SimpleCtors;
            const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
                ("Service1", AnyServiceMeta::new::<Service1>),
                ("Service2", AnyServiceMeta::new::<Service2>),
            ];
            const ASYNC: bool = false;
        }

        let doc = build_program_ast::<TestProgram>(None).unwrap();
        assert_eq!(doc.services.len(), 3);
    }

    // #[test]
    // fn base_services_cycle_detection() {
    //     struct ServiceA;
    //     impl Identifiable for ServiceA {
    // const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(70u64);
    // }
    // impl ServiceMeta for ServiceA {
    //         type CommandsMeta = utils::NoCommands;
    //         type QueriesMeta = utils::NoQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] =
    //             &[BaseServiceMeta::new::<ServiceB>("ServiceB")];
    //         const ASYNC: bool = false;
    //
    //     }

    //     struct ServiceB;
    //     impl Identifiable for ServiceB {
    // const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(71u64);
    // }
    // impl ServiceMeta for ServiceB {
    //         type CommandsMeta = utils::NoCommands;
    //         type QueriesMeta = utils::NoQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] =
    //             &[BaseServiceMeta::new::<ServiceA>("ServiceA")];
    //         const ASYNC: bool = false;
    //
    //     }
    //     let res = test_service_units::<ServiceA>("ServiceA");
    //     assert!(res.is_err());
    //     let Err(Error::MetaIsInvalid(msg)) = res else {
    //         panic!("Expected MetaIsInvalid error, got {res:?}");
    //     };
    //     assert!(msg.contains("cyclic base services"));
    // }

    // #[test]
    // #[ignore = "TODO [future]: Must be error when Sails binary protocol is implemented"]
    // fn no_same_service_in_base_services() {
    //     struct ServiceA;
    //     impl Identifiable for ServiceA {
    // const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(70u64);
    // }
    // impl ServiceMeta for ServiceA {
    //         type CommandsMeta = utils::NoCommands;
    //         type QueriesMeta = utils::NoQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
    //         const ASYNC: bool = false;
    //
    //     }

    //     struct ServiceB;
    //     impl Identifiable for ServiceB {
    // const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(71u64);
    // }
    // impl ServiceMeta for ServiceB {
    //         type CommandsMeta = utils::NoCommands;
    //         type QueriesMeta = utils::NoQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] = &[
    //             ("ServiceA", AnyServiceMeta::new::<ServiceA>),
    //             ("ServiceA", AnyServiceMeta::new::<ServiceA>),
    //         ];
    //         const ASYNC: bool = false;
    //
    //     }

    //     assert!(test_service_units::<ServiceB>("ServiceB").is_err());

    //     struct ServiceC;
    //     impl Identifiable for ServiceC {
    // const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(72u64);
    // }

    // impl ServiceMeta for ServiceC {
    //         type CommandsMeta = utils::NoCommands;
    //         type QueriesMeta = utils::NoQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] = &[
    //             ("ServiceA", AnyServiceMeta::new::<ServiceA>),
    //             ("RenamedServiceA", AnyServiceMeta::new::<ServiceA>),
    //         ];
    //         const ASYNC: bool = false;
    //
    //     }

    //     assert!(test_service_units::<ServiceC>("ServiceC").is_err());
    // }

    // ------------------------------------------------------------------------------------
    // ------------------------------ Events related tests --------------------------------
    // ------------------------------------------------------------------------------------

    #[test]
    fn invalid_events_type() {
        struct InvalidEventsService;
        impl Identifiable for InvalidEventsService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(80u64);
        }

        impl ServiceMeta for InvalidEventsService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = InvalidEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct InvalidEvents {
            pub field: u32,
        }

        let res = test_service_units::<InvalidEventsService>("InvalidEventsService");

        assert!(res.is_err());
        let Err(Error::MetaIsInvalid(msg)) = res else {
            panic!("Expected MetaIsInvalid error, got {res:?}");
        };
        assert!(msg.contains("references a type that is not a variant"));
    }

    #[test]
    fn service_events_positive_test() {
        use TypeDecl::*;

        struct EventService;
        impl Identifiable for EventService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(81u64);
        }

        impl ServiceMeta for EventService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventServiceEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum EventServiceEvents {
            Zero,
            One(u32),
            Two(EventTwoParams),
            Three { field1: ActorId, field2: String },
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct EventTwoParams {
            pub field1: ActorId,
            pub field2: String,
        }

        let services =
            test_service_units::<EventService>("EventService").expect("ServiceBuilder error");

        assert_eq!(services.len(), 1);
        let service = &services[0];

        assert_eq!(
            service.events,
            vec![
                ServiceEvent {
                    name: "One".to_string(),
                    def: StructDef {
                        fields: vec![StructField {
                            name: None,
                            type_decl: Primitive(PrimitiveType::U32),
                            docs: vec![],
                            annotations: vec![],
                        }],
                    },
                    docs: vec![],
                    annotations: vec![],
                },
                ServiceEvent {
                    name: "Three".to_string(),
                    def: StructDef {
                        fields: vec![
                            StructField {
                                name: Some("field1".to_string()),
                                type_decl: Primitive(PrimitiveType::ActorId),
                                docs: vec![],
                                annotations: vec![],
                            },
                            StructField {
                                name: Some("field2".to_string()),
                                type_decl: Primitive(PrimitiveType::String),
                                docs: vec![],
                                annotations: vec![],
                            },
                        ],
                    },
                    docs: vec![],
                    annotations: vec![],
                },
                ServiceEvent {
                    name: "Two".to_string(),
                    def: StructDef {
                        fields: vec![StructField {
                            name: None,
                            type_decl: TypeDecl::named("EventTwoParams".to_string()),
                            docs: vec![],
                            annotations: vec![],
                        }],
                    },
                    docs: vec![],
                    annotations: vec![],
                },
                ServiceEvent {
                    name: "Zero".to_string(),
                    def: StructDef { fields: vec![] },
                    docs: vec![],
                    annotations: vec![],
                },
            ]
        );

        assert_eq!(service.types.len(), 1);
        assert!(matches!(
            service.types.first(),
            Some(sails_idl_meta::Type { name, .. }) if name == "EventTwoParams"
        ));
    }

    // ------------------------------------------------------------------------------------
    // ----------------------------- Functions related tests ------------------------------
    // ------------------------------------------------------------------------------------

    /// Test error when commands/queries are not variant types
    #[test]
    fn service_functions_non_variant_error() {
        struct NotVariantCommandsService;
        impl Identifiable for NotVariantCommandsService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(90u64);
        }

        impl ServiceMeta for NotVariantCommandsService {
            type CommandsMeta = NotVariantCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct NotVariantQueriesService;
        impl Identifiable for NotVariantQueriesService {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(91u64);
        }

        impl ServiceMeta for NotVariantQueriesService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = NotVariantQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NotVariantCommands {
            pub field: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NotVariantQueries(u32);

        let internal_check = |result: Result<Vec<ServiceUnit>>| {
            assert!(result.is_err());
            let Err(Error::MetaIsInvalid(msg)) = result else {
                panic!("Expected MetaIsInvalid error, got {result:?}");
            };
            assert!(msg.contains("references a type that is not a variant"));
        };

        internal_check(test_service_units::<NotVariantCommandsService>(
            "TestService",
        ));
        internal_check(test_service_units::<NotVariantQueriesService>(
            "TestService",
        ));
    }

    /// Test error when service variant doesn't have exactly 2 fields
    #[test]
    fn service_variant_field_count_error() {
        struct InvalidCommandsService1;
        impl Identifiable for InvalidCommandsService1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(100u64);
        }

        impl ServiceMeta for InvalidCommandsService1 {
            type CommandsMeta = BadCommands1;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidCommandsService2;
        impl Identifiable for InvalidCommandsService2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(101u64);
        }

        impl ServiceMeta for InvalidCommandsService2 {
            type CommandsMeta = BadCommands2;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidQueriesService1;
        impl Identifiable for InvalidQueriesService1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(102u64);
        }

        impl ServiceMeta for InvalidQueriesService1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = BadQueries1;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidQueriesService2;
        impl Identifiable for InvalidQueriesService2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(103u64);
        }

        impl ServiceMeta for InvalidQueriesService2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = BadQueries2;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        // Commands/queries with wrong number of fields
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands1 {
            OneField(u32),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands2 {
            FourFields(u32, String, bool, u32),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadQueries1 {
            OneField(u32),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadQueries2 {
            FourFields(u32, String, bool, u32),
        }

        let internal_check = |result: Result<Vec<ServiceUnit>>, expected_msg: &str| {
            assert!(result.is_err());
            let Err(Error::MetaIsInvalid(msg)) = result else {
                panic!("Expected MetaIsInvalid error, got {result:?}");
            };
            assert_eq!(msg.as_str(), expected_msg);
        };

        internal_check(
            test_service_units::<InvalidCommandsService1>("TestService"),
            "command `OneField` has invalid number of fields",
        );
        internal_check(
            test_service_units::<InvalidQueriesService1>("TestService"),
            "query `OneField` has invalid number of fields",
        );
        internal_check(
            test_service_units::<InvalidCommandsService2>("TestService"),
            "command `FourFields` has invalid number of fields",
        );
        internal_check(
            test_service_units::<InvalidQueriesService2>("TestService"),
            "query `FourFields` has invalid number of fields",
        );
    }

    /// Test error when service method params are not composite
    #[test]
    fn service_params_non_composite_error() {
        struct TestServiceMeta;
        impl Identifiable for TestServiceMeta {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(110u64);
        }

        impl ServiceMeta for TestServiceMeta {
            type CommandsMeta = BadCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        // Commands where the first field (params) is not composite
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands {
            BadCmd(u32, String),
        }

        let result = test_service_units::<TestServiceMeta>("TestService");

        assert!(result.is_err());
        let Err(Error::MetaIsInvalid(msg)) = result else {
            panic!("Expected MetaIsInvalid error, got {result:?}");
        };
        assert_eq!(
            msg.as_str(),
            "command `BadCmd` params type is not a composite"
        );
    }

    /// Test error when service method params have nameless fields
    #[test]
    fn service_params_nameless_fields_error() {
        struct BadServiceMeta;
        impl Identifiable for BadServiceMeta {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(111u64);
        }

        impl ServiceMeta for BadServiceMeta {
            type CommandsMeta = BadCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands {
            BadCmd(NamelessParams, String),
        }

        // Tuple struct with nameless fields for params
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NamelessParams(u32, String);

        let result = test_service_units::<BadServiceMeta>("TestService");

        assert!(result.is_err());
        let Err(Error::MetaIsInvalid(msg)) = result else {
            panic!("Expected MetaIsInvalid error, got {result:?}");
        };
        assert_eq!(msg.as_str(), "command `BadCmd` param is missing a name");
    }

    // TODO: Result unwrapping
    // #[test]
    // fn service_fns_result_ty() {
    //     struct TestServiceMeta;
    //     impl Identifiable for TestServiceMeta {
    // const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(120u64);
    // }

    // impl ServiceMeta for TestServiceMeta {
    //         type CommandsMeta = TestCommands;
    //         type QueriesMeta = TestQueries;
    //         type EventsMeta = utils::NoEvents;
    //         const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
    //         const ASYNC: bool = false;
    //
    //     }

    //     #[derive(TypeInfo)]
    //     #[allow(unused)]
    //     enum TestCommands {
    //         Unit(utils::SimpleFunctionParams, ()),
    //         NonUnit(utils::SimpleFunctionParams, String),
    //         WithUnit(utils::SimpleFunctionParams, Result<(), u32>),
    //         Result(utils::SimpleFunctionParams, Result<u32, String>),
    //     }

    //     #[derive(TypeInfo)]
    //     #[allow(unused)]
    //     enum TestQueries {
    //         Unit(utils::SimpleFunctionParams, ()),
    //         NonUnit(utils::SimpleFunctionParams, u32),
    //         WithUnit(utils::SimpleFunctionParams, Result<(), u32>),
    //         Result(utils::SimpleFunctionParams, Result<u32, String>),
    //     }

    //     let services =
    //         test_service_units::<TestServiceMeta>("TestService").expect("ServiceBuilder error");
    //     assert_eq!(services.len(), 1);
    //     let service = &services[0];

    //     let get = |name: &str, kind: FunctionKind| -> &ServiceFunc {
    //         service
    //             .funcs
    //             .iter()
    //             .find(|f| f.name == name && f.kind == kind)
    //             .unwrap_or_else(|| panic!("missing {kind:?} {name}"))
    //     };

    //     assert_eq!(
    //         get("Unit", FunctionKind::Command).output,
    //         TypeDecl::Primitive(PrimitiveType::Void)
    //     );
    //     assert_eq!(get("Unit", FunctionKind::Command).throws, None);

    //     assert_eq!(
    //         get("NonUnit", FunctionKind::Command).output,
    //         TypeDecl::Primitive(PrimitiveType::String)
    //     );
    //     assert_eq!(get("NonUnit", FunctionKind::Command).throws, None);

    //     assert_eq!(
    //         get("WithUnit", FunctionKind::Command).output,
    //         TypeDecl::Primitive(PrimitiveType::Void)
    //     );
    //     assert_eq!(
    //         get("WithUnit", FunctionKind::Command).throws,
    //         Some(TypeDecl::Primitive(PrimitiveType::U32))
    //     );

    //     assert_eq!(
    //         get("Result", FunctionKind::Command).output,
    //         TypeDecl::Primitive(PrimitiveType::U32)
    //     );
    //     assert_eq!(
    //         get("Result", FunctionKind::Command).throws,
    //         Some(TypeDecl::Primitive(PrimitiveType::String))
    //     );

    //     assert_eq!(
    //         get("Unit", FunctionKind::Query).output,
    //         TypeDecl::Primitive(PrimitiveType::Void)
    //     );
    //     assert_eq!(get("Unit", FunctionKind::Query).throws, None);

    //     assert_eq!(
    //         get("NonUnit", FunctionKind::Query).output,
    //         TypeDecl::Primitive(PrimitiveType::U32)
    //     );
    //     assert_eq!(get("NonUnit", FunctionKind::Query).throws, None);

    //     assert_eq!(
    //         get("WithUnit", FunctionKind::Query).output,
    //         TypeDecl::Primitive(PrimitiveType::Void)
    //     );
    //     assert_eq!(
    //         get("WithUnit", FunctionKind::Query).throws,
    //         Some(TypeDecl::Primitive(PrimitiveType::U32))
    //     );

    //     assert_eq!(
    //         get("Result", FunctionKind::Query).output,
    //         TypeDecl::Primitive(PrimitiveType::U32)
    //     );
    //     assert_eq!(
    //         get("Result", FunctionKind::Query).throws,
    //         Some(TypeDecl::Primitive(PrimitiveType::String))
    //     );
    // }

    #[test]
    fn service_function_variations_positive_test() {
        struct ServiceWithOneCommand;
        impl Identifiable for ServiceWithOneCommand {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(130u64);
        }

        impl ServiceMeta for ServiceWithOneCommand {
            type CommandsMeta = OneFunction;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceWithOneQuery;
        impl Identifiable for ServiceWithOneQuery {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(131u64);
        }

        impl ServiceMeta for ServiceWithOneQuery {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = OneFunction;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceWithNoFunctions;
        impl Identifiable for ServiceWithNoFunctions {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(132u64);
        }

        impl ServiceMeta for ServiceWithNoFunctions {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum OneFunction {
            Fn1(utils::SimpleFunctionParams, String),
        }

        let internal_check = |service: &ServiceUnit,
                              expected_commands_count: usize,
                              expected_queries_count: usize| {
            let commands_count = service
                .funcs
                .iter()
                .filter(|f| f.kind == FunctionKind::Command)
                .count();
            let queries_count = service
                .funcs
                .iter()
                .filter(|f| f.kind == FunctionKind::Query)
                .count();

            assert_eq!(commands_count, expected_commands_count);
            assert_eq!(queries_count, expected_queries_count);

            if expected_commands_count > 0 {
                assert!(
                    service
                        .funcs
                        .iter()
                        .any(|f| f.kind == FunctionKind::Command && f.name == "Fn1")
                );
            }
            if expected_queries_count > 0 {
                assert!(
                    service
                        .funcs
                        .iter()
                        .any(|f| f.kind == FunctionKind::Query && f.name == "Fn1")
                );
            }
        };

        let svc = test_service_units::<ServiceWithOneCommand>("TestService").unwrap();
        internal_check(&svc[0], 1, 0);
        let svc = test_service_units::<ServiceWithOneQuery>("TestService").unwrap();
        internal_check(&svc[0], 0, 1);
        let svc = test_service_units::<ServiceWithNoFunctions>("TestService").unwrap();
        internal_check(&svc[0], 0, 0);

        struct Service;
        impl Identifiable for Service {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(133u64);
        }

        impl ServiceMeta for Service {
            type CommandsMeta = ServiceCommands;
            type QueriesMeta = ServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ServiceCommands {
            NoArgs(NoArgs, String),
            OneArg(OneArg, u32),
            MultiArgs(MultiArgs, bool),
            NoResult(OneArg, ()),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ServiceQueries {
            NoArgs(NoArgs, String),
            OneArg(OneArg, u32),
            MultiArgs(MultiArgs, bool),
            NoResult(OneArg, ()),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NoArgs;

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct OneArg {
            pub arg1: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct MultiArgs {
            pub arg1: u32,
            pub arg2: String,
            pub arg3: bool,
        }

        let service = test_service_units::<Service>("TestService").unwrap();
        let service = &service[0];

        let get = |name: &str, kind: FunctionKind| -> &ServiceFunc {
            service
                .funcs
                .iter()
                .find(|f| f.name == name && f.kind == kind)
                .unwrap_or_else(|| panic!("missing {kind:?} {name}"))
        };

        for kind in [FunctionKind::Command, FunctionKind::Query] {
            assert_eq!(get("NoArgs", kind).params.len(), 0);

            let one_arg = get("OneArg", kind);
            assert_eq!(one_arg.params.len(), 1);
            assert_eq!(one_arg.params[0].name, "arg1");

            let multi_args = get("MultiArgs", kind);
            assert_eq!(multi_args.params.len(), 3);
            assert_eq!(multi_args.params[0].name, "arg1");
            assert_eq!(multi_args.params[1].name, "arg2");
            assert_eq!(multi_args.params[2].name, "arg3");

            let no_result = get("NoResult", kind);
            assert_eq!(no_result.output, TypeDecl::Primitive(PrimitiveType::Void));
            assert_eq!(no_result.throws, None);
        }
    }

    // ------------------------------------------------------------------------------------
    // --------------------------- Types section related tests ----------------------------
    // ------------------------------------------------------------------------------------

    /// Test that services with only primitive/builtin types have empty types sections
    #[test]
    fn service_non_user_defined_types_excluded() {
        struct Service1;
        impl Identifiable for Service1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(140u64);
        }

        impl ServiceMeta for Service1 {
            type CommandsMeta = CommandsWithNonUserDefinedArgs;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service2;
        impl Identifiable for Service2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(141u64);
        }

        impl ServiceMeta for Service2 {
            type CommandsMeta = CommandWithUserDefinedArgs;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service3;
        impl Identifiable for Service3 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(142u64);
        }

        impl ServiceMeta for Service3 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = CommandsWithNonUserDefinedArgs;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service4;
        impl Identifiable for Service4 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(143u64);
        }

        impl ServiceMeta for Service4 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = CommandWithUserDefinedArgs;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service5;
        impl Identifiable for Service5 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(144u64);
        }

        impl ServiceMeta for Service5 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventsWithNonUserDefinedArgs;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service6;
        impl Identifiable for Service6 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(145u64);
        }

        impl ServiceMeta for Service6 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventsWithUserDefinedArgs;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum CommandWithUserDefinedArgs {
            Cmd1(UserDefinedParams, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct UserDefinedParams {
            pub arg1: NonUserDefinedArgs,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum CommandsWithNonUserDefinedArgs {
            Cmd1(NonUserDefinedArgs, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum EventsWithNonUserDefinedArgs {
            Event1 {
                number: u32,
                flag: bool,
                text: String,
                actor: ActorId,
                option_val: Option<u8>,
                result_val: Result<u16, String>,
                map: BTreeMap<String, u32>,
                code: CodeId,
                message: MessageId,
                h160: H160,
                h256: H256,
                u256: U256,
                // non_zero_u8: NonZeroU8,
                // non_zero_u16: NonZeroU16,
                // non_zero_u32: NonZeroU32,
                // non_zero_u64: NonZeroU64,
                // non_zero_u128: NonZeroU128,
                // non_zero_u256: NonZeroU256,
            },
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum EventsWithUserDefinedArgs {
            Event1(NonUserDefinedArgs),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NonUserDefinedArgs {
            pub number: u32,
            pub flag: bool,
            pub text: String,
            pub actor: ActorId,
            pub option_val: Option<u8>,
            pub result_val: Result<u16, String>,
            pub map: BTreeMap<String, u32>,
            pub code: CodeId,
            pub message: MessageId,
            pub h160: H160,
            pub h256: H256,
            pub u256: U256,
            // pub non_zero_u8: NonZeroU8,
            // pub non_zero_u16: NonZeroU16,
            // pub non_zero_u32: NonZeroU32,
            // pub non_zero_u64: NonZeroU64,
            // pub non_zero_u128: NonZeroU128,
            // pub non_zero_u256: NonZeroU256,
        }

        let check = |service: &ServiceUnit, expected_type_count: usize| {
            assert_eq!(service.types.len(), expected_type_count);
            if expected_type_count == 1 {
                assert!(matches!(
                    service.types.first(),
                    Some(sails_idl_meta::Type { name, .. }) if name == "NonUserDefinedArgs"
                ));
            }
        };

        check(&test_service_units::<Service1>("Service1").unwrap()[0], 0);
        check(&test_service_units::<Service2>("Service2").unwrap()[0], 1);

        check(&test_service_units::<Service3>("Service3").unwrap()[0], 0);
        check(&test_service_units::<Service4>("Service4").unwrap()[0], 1);

        check(&test_service_units::<Service5>("Service5").unwrap()[0], 0);
        check(&test_service_units::<Service6>("Service6").unwrap()[0], 1);
    }

    #[test]
    fn ctor_non_user_defined_types_excluded() {
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum CtorsWithNonUserDefinedArgs {
            Ctor1(NonUserDefinedCtorArgs),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum CtorsWithUserDefinedArgs {
            Ctor2(UserDefinedCtorArgs),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct NonUserDefinedCtorArgs {
            pub number: u32,
            pub flag: bool,
            pub text: String,
            pub actor: ActorId,
            pub option_val: Option<u8>,
            pub result_val: Result<u16, String>,
            pub map: BTreeMap<String, u32>,
            pub code: CodeId,
            pub message: MessageId,
            pub h160: H160,
            pub h256: H256,
            pub u256: U256,
            // pub non_zero_u8: NonZeroU8,
            // pub non_zero_u16: NonZeroU16,
            // pub non_zero_u32: NonZeroU32,
            // pub non_zero_u64: NonZeroU64,
            // pub non_zero_u128: NonZeroU128,
            // pub non_zero_u256: NonZeroU256,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct UserDefinedCtorArgs {
            pub custom: CustomType,
            pub number: u32,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct CustomType {
            pub value: String,
        }

        let meta1 =
            test_program_unit::<CtorsWithNonUserDefinedArgs>().expect("ProgramBuilder error");
        assert!(meta1.types.is_empty());

        let meta2 = test_program_unit::<CtorsWithUserDefinedArgs>().expect("ProgramBuilder error");
        assert_eq!(meta2.types.len(), 1);
        assert!(matches!(
            meta2.types.first(),
            Some(sails_idl_meta::Type { name, .. }) if name == "CustomType"
        ));
    }

    // --------------------------------------------------------------------------------
    // ------------------------------ Miscellaneous tests -----------------------------
    // --------------------------------------------------------------------------------

    #[test]
    fn shared_and_same_name_types_across_services() {
        struct Service1Meta;
        impl Identifiable for Service1Meta {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(150u64);
        }

        impl ServiceMeta for Service1Meta {
            type CommandsMeta = Service1Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service2Meta;
        impl Identifiable for Service2Meta {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(151u64);
        }

        impl ServiceMeta for Service2Meta {
            type CommandsMeta = Service2Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        // First service using both shared types
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Service1Commands {
            Cmd1(ServiceCommandParams, String),
            Cmd2(ServiceCommandParams, SharedCustomType),
        }

        // Second service using both shared types
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Service2Commands {
            Cmd3(ServiceCommandParams, String),
            Cmd4(ServiceCommandParams, SharedCustomType),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ServiceCommandParams {
            param1: SimpleFunctionParams,
            param2: utils::SimpleFunctionParams,
        }

        // Define SimpleFunctionParams in local scope
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct SimpleFunctionParams {
            f1: SharedCustomType,
        }

        // Define a custom type to be reused across services
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct SharedCustomType;

        let service_1 = &test_service_units::<Service1Meta>("Service1").unwrap()[0];
        let service_2 = &test_service_units::<Service2Meta>("Service2").unwrap()[0];

        assert_eq!(service_1.types.len(), 3);
        assert_eq!(service_2.types.len(), 3);

        let type_names = |svc: &ServiceUnit| -> BTreeSet<String> {
            svc.types.iter().map(|t| t.name.clone()).collect()
        };
        assert_eq!(type_names(service_1), type_names(service_2));
        assert!(type_names(service_1).contains("SharedCustomType"));

        let simple_params = service_1
            .types
            .iter()
            .filter(|t| t.name.contains("SimpleFunctionParams"))
            .collect::<Vec<_>>();
        assert_eq!(simple_params.len(), 2);

        let mut has_u32_field = false;
        let mut has_shared_custom_type_field = false;
        for ty in simple_params {
            let sails_idl_meta::TypeDef::Struct(def) = &ty.def else {
                continue;
            };
            if def
                .fields
                .iter()
                .any(|f| f.type_decl == TypeDecl::Primitive(PrimitiveType::U32))
            {
                has_u32_field = true;
            }
            if def.fields.iter().any(|f| {
                matches!(&f.type_decl, TypeDecl::Named { name, .. } if name == "SharedCustomType")
            }) {
                has_shared_custom_type_field = true;
            }
        }

        assert!(has_u32_field);
        assert!(has_shared_custom_type_field);
    }

    #[test]
    fn no_repeated_services() {
        struct Service1;
        impl Identifiable for Service1 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(160u64);
        }

        impl ServiceMeta for Service1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service2;
        impl Identifiable for Service2 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(161u64);
        }

        impl ServiceMeta for Service2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct Service3;
        impl Identifiable for Service3 {
            const INTERFACE_ID: InterfaceId = InterfaceId::from_u64(162u64);
        }

        impl ServiceMeta for Service3 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [BaseServiceMeta] = &[];
            const ASYNC: bool = false;
        }

        struct TestProgram;
        impl ProgramMeta for TestProgram {
            type ConstructorsMeta = utils::SimpleCtors;
            const SERVICES: &'static [(&'static str, AnyServiceMetaFn)] = &[
                ("Service11", AnyServiceMeta::new::<Service1>),
                ("Service12", AnyServiceMeta::new::<Service1>),
                ("Service13", AnyServiceMeta::new::<Service1>),
                ("Service21", AnyServiceMeta::new::<Service2>),
                ("Service22", AnyServiceMeta::new::<Service2>),
                ("Service31", AnyServiceMeta::new::<Service3>),
            ];
            const ASYNC: bool = false;
        }

        let doc = build_program_ast::<TestProgram>(None).unwrap();
        assert_eq!(doc.services.len(), 3);
    }
}
