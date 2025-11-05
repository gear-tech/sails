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

use crate::{
    FunctionArgumentIdl, FunctionIdl, FunctionResultIdl, FunctionsSection, ProgramIdlSection,
    ServiceSection,
    errors::{Error, Result},
    type_names::{self, FinalizedName, FinalizedRawName, ResultTypeName},
};
use gprimitives::*;
use sails_idl_meta::*;
use scale_info::{MetaType, PortableRegistry, Registry, TypeDef, Variant, form::PortableForm};
use std::{
    collections::HashSet,
    num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128},
};

#[derive(Debug, serde::Serialize)]
pub(crate) struct ExpandedProgramMeta {
    pub(crate) program: Option<ProgramIdlSection>,
    pub(crate) services: Vec<ServiceSection>,
}

impl ExpandedProgramMeta {
    pub fn new(
        ctors: Option<(String, MetaType)>,
        services: impl Iterator<Item = (&'static str, AnyServiceMeta)>,
    ) -> Result<Self> {
        let mut program_section = if let Some((program_name, ctors_meta)) = ctors {
            let program_meta_registry = ProgramMetaRegistry::new(ctors_meta)?;
            let concrete_names = program_meta_registry.concrete_names().to_vec();
            let types = program_meta_registry.types().to_vec();
            let ctors = program_meta_registry.ctor_fns;

            // If there're no constructors, don't generate program section.
            (!ctors.is_empty()).then_some({
                ProgramIdlSection {
                    name: program_name,
                    concrete_names,
                    types,
                    ctors,
                    services: Default::default(),
                }
            })
        } else {
            None
        };

        let mut services_section = Vec::new();
        for (name, service_meta) in services {
            let service_registry = ServiceMetaRegistry::new(service_meta)?;

            let types = service_registry.types().to_vec();
            let concrete_names = service_registry.concrete_names().to_vec();
            let functions = FunctionsSection {
                commands: service_registry.commands_fns,
                queries: service_registry.queries_fns,
            };
            let events = service_registry.events;

            let service_section = ServiceSection {
                name: name.to_string(),
                types,
                concrete_names,
                extends: Default::default(),
                events,
                functions,
            };
            services_section.push(service_section);
        }

        if let Some(ref mut program_section) = program_section {
            program_section.services = services_section
                .iter()
                .map(|svc| svc.name.clone())
                .collect();
        }

        Ok(Self {
            program: program_section,
            services: services_section,
        })
    }
}

struct ProgramMetaRegistry {
    portable_registry: IdlPortableTypesRegistry,
    ctor_fns: Vec<FunctionIdl>,
}

impl ProgramMetaRegistry {
    fn new(ctors: MetaType) -> Result<Self> {
        let mut idl_registry = IdlTypesRegistry::new();

        let ctors_type_id = idl_registry.register_function(&ctors);
        let idl_portable_registry: IdlPortableTypesRegistry = idl_registry.try_into()?;

        let ctor_fns = idl_portable_registry.resolve_functions(ctors_type_id, true)?;

        Ok(Self {
            portable_registry: idl_portable_registry,
            ctor_fns,
        })
    }

    fn types(&self) -> &[FinalizedRawName] {
        self.portable_registry.raw_names()
    }

    fn concrete_names(&self) -> &[FinalizedName] {
        self.portable_registry.concrete_names()
    }
}

struct ServiceMetaRegistry {
    portable_registry: IdlPortableTypesRegistry,
    commands_fns: Vec<FunctionIdl>,
    queries_fns: Vec<FunctionIdl>,
    events: Vec<Variant<PortableForm>>,
}

impl ServiceMetaRegistry {
    pub fn new(service_meta: AnyServiceMeta) -> Result<Self> {
        let mut idl_registry = IdlTypesRegistry::new();
        let registered_service_meta = idl_registry.register_service_meta(&service_meta);

        let idl_portable_registry: IdlPortableTypesRegistry = idl_registry.try_into()?;

        let commands_fns = Self::commands_idl_data(
            &idl_portable_registry,
            registered_service_meta.commands_type_ids,
        )?;
        let queries_fns = Self::queries_idl_data(
            &idl_portable_registry,
            registered_service_meta.queries_type_ids,
        )?;
        let events = Self::events_idl_data(
            &idl_portable_registry,
            registered_service_meta.events_type_ids,
        )?;

        Ok(Self {
            portable_registry: idl_portable_registry,
            commands_fns,
            queries_fns,
            events,
        })
    }

    fn commands_idl_data(
        idl_portable_registry: &IdlPortableTypesRegistry,
        mut commands_ids: Vec<u32>,
    ) -> Result<Vec<FunctionIdl>> {
        if commands_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Contract: the first id in `commands_ids` is the main one.
        let main_commands_ids = commands_ids.remove(0);
        let base_commands_ids = commands_ids;

        let mut commands_idl_data =
            idl_portable_registry.resolve_functions(main_commands_ids, false)?;

        for base_commands_type_id in base_commands_ids {
            let mut base_commands_idl_data =
                idl_portable_registry.resolve_functions(base_commands_type_id, false)?;

            // Override any existing function.
            // The latest ("most extended") one always generated.
            base_commands_idl_data.retain(|base_f| {
                !commands_idl_data
                    .iter()
                    .any(|existing_f| existing_f.name == base_f.name)
            });

            commands_idl_data.append(&mut base_commands_idl_data);
        }

        Ok(commands_idl_data)
    }

    fn queries_idl_data(
        idl_portable_registry: &IdlPortableTypesRegistry,
        mut queries_ids: Vec<u32>,
    ) -> Result<Vec<FunctionIdl>> {
        if queries_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Contract: the first id in `queries_ids` is the main one.
        let main_queries_ids = queries_ids.remove(0);
        let base_queries_ids = queries_ids;

        let mut queries_idl_data =
            idl_portable_registry.resolve_functions(main_queries_ids, false)?;

        for base_queries_type_id in base_queries_ids {
            let mut base_queries_idl_data =
                idl_portable_registry.resolve_functions(base_queries_type_id, false)?;

            // Override any existing function.
            // The latest ("most extended") one always generated.
            base_queries_idl_data.retain(|base_f| {
                !queries_idl_data
                    .iter()
                    .any(|existing_f| existing_f.name == base_f.name)
            });

            queries_idl_data.append(&mut base_queries_idl_data);
        }

        Ok(queries_idl_data)
    }

    fn events_idl_data(
        idl_portable_registry: &IdlPortableTypesRegistry,
        events_ids: Vec<u32>,
    ) -> Result<Vec<Variant<PortableForm>>> {
        let mut events: Vec<Variant<PortableForm>> = Vec::new();

        for events_type_id in events_ids {
            let svc_events = idl_portable_registry.resolve_events(events_type_id)?;
            for svc_event in svc_events {
                // Override any existing event.
                // The latest ("most extended") one always generated.
                if events
                    .iter()
                    .any(|existing_v| existing_v.name == svc_event.name)
                {
                    return Err(Error::EventMetaIsAmbiguous(format!(
                        "event `{}` is defined multiple times in the service inheritance chain",
                        svc_event.name
                    )));
                }

                events.push(svc_event);
            }
        }

        Ok(events)
    }

    fn types(&self) -> &[FinalizedRawName] {
        self.portable_registry.raw_names()
    }

    fn concrete_names(&self) -> &[FinalizedName] {
        self.portable_registry.concrete_names()
    }
}

struct IdlTypesRegistry {
    non_type_section_ids: HashSet<u32>,
    unit_type_id: u32,
    functions_ids: HashSet<u32>,
    registry: Registry,
}

impl IdlTypesRegistry {
    fn new() -> Self {
        let mut registry = Registry::new();
        let non_type_section_ids = registry
            .register_types([
                MetaType::new::<ActorId>(),
                MetaType::new::<CodeId>(),
                MetaType::new::<MessageId>(),
                MetaType::new::<H160>(),
                MetaType::new::<H256>(),
                MetaType::new::<U256>(),
                MetaType::new::<NonZeroU8>(),
                MetaType::new::<NonZeroU16>(),
                MetaType::new::<NonZeroU32>(),
                MetaType::new::<NonZeroU64>(),
                MetaType::new::<NonZeroU128>(),
                MetaType::new::<NonZeroU256>(),
            ])
            .into_iter()
            .map(|t| t.id)
            .collect::<HashSet<_>>();

        let unit_type_id = registry.register_type(&MetaType::new::<()>()).id;

        Self {
            registry,
            non_type_section_ids,
            functions_ids: HashSet::new(),
            unit_type_id,
        }
    }

    fn register_service_meta(&mut self, service_meta: &AnyServiceMeta) -> RegisteredServiceMeta {
        let commands_type_ids = Self::flat_meta(service_meta, |meta| meta.commands())
            .into_iter()
            .map(|fn_meta| self.register_function(fn_meta))
            .collect();
        let queries_type_ids = Self::flat_meta(service_meta, |meta| meta.queries())
            .into_iter()
            .map(|fn_meta| self.register_function(fn_meta))
            .collect();
        let events_type_ids = Self::flat_meta(service_meta, |meta| meta.events())
            .into_iter()
            .map(|fn_meta| self.register_event(fn_meta))
            .collect();

        RegisteredServiceMeta {
            commands_type_ids,
            queries_type_ids,
            events_type_ids,
        }
    }

    fn flat_meta(
        service_meta: &AnyServiceMeta,
        f: fn(&AnyServiceMeta) -> &MetaType,
    ) -> Vec<&MetaType> {
        let mut metas = vec![f(service_meta)];
        for base_service_meta in service_meta.base_services() {
            metas.extend(Self::flat_meta(base_service_meta, f));
        }

        metas
    }

    fn register_function(&mut self, fn_type: &MetaType) -> u32 {
        let ret = self.registry.register_type(fn_type).id;
        self.non_type_section_ids.insert(ret);
        self.functions_ids.insert(ret);

        ret
    }

    fn register_event(&mut self, event_type: &MetaType) -> u32 {
        let ret = self.registry.register_type(event_type).id;
        self.non_type_section_ids.insert(ret);

        ret
    }
}

// TODO: if extensions are implemented, this separation can be useful to distinguish on IDL the owner of fn/event
struct RegisteredServiceMeta {
    commands_type_ids: Vec<u32>,
    queries_type_ids: Vec<u32>,
    events_type_ids: Vec<u32>,
}

#[derive(Debug)]
struct IdlPortableTypesRegistry {
    portable_registry: PortableRegistry,
    unit_type_id: u32,
    concrete_type_names: Vec<FinalizedName>,
    raw_type_names: Vec<FinalizedRawName>,
}

impl TryFrom<IdlTypesRegistry> for IdlPortableTypesRegistry {
    type Error = Error;

    fn try_from(idl_registry: IdlTypesRegistry) -> Result<Self> {
        let IdlTypesRegistry {
            registry,
            mut non_type_section_ids,
            functions_ids,
            unit_type_id,
        } = idl_registry;

        let portable_registry = PortableRegistry::from(registry);

        // Mark `__*Params` structs of functions as non type section types.
        for func_type_id in &functions_ids {
            for fn_meta in Self::fns_meta_iter(&portable_registry, *func_type_id)? {
                if fn_meta.fields.is_empty() || fn_meta.fields.len() > 2 {
                    return Err(Error::FuncMetaIsInvalid(format!(
                        "function `{}` has invalid signature: expected at least args or/and result",
                        fn_meta.name
                    )));
                }

                let fn_args_type_id = fn_meta.fields[0].ty.id;
                non_type_section_ids.insert(fn_args_type_id);
            }
        }

        let (concrete_type_names, raw_type_names) =
            type_names::resolve(portable_registry.types.iter(), &non_type_section_ids)?;
        let concrete_type_names = concrete_type_names.into_values().collect();
        let raw_type_names = raw_type_names.into_values().collect();
        Ok(Self {
            portable_registry,
            unit_type_id,
            concrete_type_names,
            raw_type_names,
        })
    }
}

impl IdlPortableTypesRegistry {
    fn raw_names(&self) -> &[FinalizedRawName] {
        &self.raw_type_names
    }

    fn concrete_names(&self) -> &[FinalizedName] {
        &self.concrete_type_names
    }

    fn resolve_events(&self, events_type_id: u32) -> Result<Vec<Variant<PortableForm>>> {
        let event_type = self
            .portable_registry
            .resolve(events_type_id)
            .ok_or(Error::TypeIdIsUnknown(events_type_id))?;

        let TypeDef::Variant(ref event_type_def) = event_type.type_def else {
            return Err(Error::FuncMetaIsInvalid(
                "Event type is not a variant".to_string(),
            ));
        };

        Ok(event_type_def.variants.clone())
    }

    fn resolve_functions(&self, fns_type_id: u32, is_ctor: bool) -> Result<Vec<FunctionIdl>> {
        let mut ret = Vec::new();

        for fn_meta in Self::fns_meta_iter(&self.portable_registry, fns_type_id)? {
            let expected_fields_len = if is_ctor { 1 } else { 2 };
            if fn_meta.fields.len() != expected_fields_len {
                let msg = if expected_fields_len == 1 {
                    format!(
                        "function `{}` has invalid signature: expected only args",
                        fn_meta.name
                    )
                } else {
                    format!(
                        "function `{}` has invalid signature: expected args and result",
                        fn_meta.name
                    )
                };
                return Err(Error::FuncMetaIsInvalid(msg));
            }

            let args_type_id = fn_meta.fields[0].ty.id;
            let args_meta = self
                .portable_registry
                .resolve(args_type_id)
                .ok_or(Error::TypeIdIsUnknown(args_type_id))?;
            let TypeDef::Composite(args_meta_type_def) = &args_meta.type_def else {
                return Err(Error::FuncMetaIsInvalid(format!(
                    "function `{}` args type is not a composite",
                    fn_meta.name
                )));
            };

            // Construct args vector by taking fields of `__*Params` struct.
            let args = args_meta_type_def
                .fields
                .iter()
                .map(|arg_meta| -> Result<FunctionArgumentIdl> {
                    let name = arg_meta
                        .name
                        .as_ref()
                        .map(|s| s.to_string())
                        .ok_or_else(|| {
                            Error::FuncMetaIsInvalid(format!(
                                "function `{}` has nameless argument",
                                fn_meta.name
                            ))
                        })?;

                    Ok(FunctionArgumentIdl {
                        name,
                        ty: arg_meta.ty.id,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let result_ty = self.resolve_result_ty(fn_meta)?;

            ret.push(FunctionIdl {
                name: fn_meta.name.to_string(),
                args,
                result_ty,
                docs: fn_meta.docs.iter().map(|s| s.to_string()).collect(),
            });
        }

        Ok(ret)
    }

    /// The function takes the result field and builds FunctionResultIdl from it.
    ///
    /// If function's meta variant types has 2 field, it means one of them is for args, and the other is for result.
    ///
    /// If the result type is a Result<T, E>, then FunctionResultIdl will have both `res` and `err` fields populated,
    /// otherwise only `res` field will be populated. The `res` gets the type id of the stored in `fn_meta` result type,
    /// unless it is unit type `()`, in which case it will be None.
    fn resolve_result_ty(
        &self,
        fn_meta: &Variant<PortableForm>,
    ) -> Result<Option<FunctionResultIdl>> {
        let Some(res_field) = fn_meta.fields.get(1) else {
            return Ok(None);
        };

        let res_type_id = res_field.ty.id;
        let res_type_meta = self
            .portable_registry
            .resolve(res_type_id)
            .ok_or(Error::TypeIdIsUnknown(res_type_id))?;

        let res = if ResultTypeName::is_result_type(res_type_meta) {
            let TypeDef::Variant(result_variants) = &res_type_meta.type_def else {
                return Err(Error::TypeIsUnsupported(format!(
                    "Expected Result type to be a variant, got {:?}",
                    res_type_meta.type_def
                )));
            };

            let result_variants = &result_variants.variants;
            if result_variants.len() != 2 {
                return Err(Error::TypeIsUnsupported(format!(
                    "Expected Result type to have 2 variants, got {}",
                    result_variants.len()
                )));
            }

            let ok_variant_type_id = {
                let ok_variant = &result_variants[0];
                if ok_variant.fields.len() != 1 {
                    return Err(Error::TypeIsUnsupported(format!(
                        "Expected Result::Ok variant to have 1 field, got {}",
                        ok_variant.fields.len()
                    )));
                }

                ok_variant.fields[0].ty.id
            };
            let err_variant_type_id = {
                let err_variant = &result_variants[1];
                if err_variant.fields.len() != 1 {
                    return Err(Error::TypeIsUnsupported(format!(
                        "Expected Result::Err variant to have 1 field, got {}",
                        err_variant.fields.len()
                    )));
                }

                err_variant.fields[0].ty.id
            };

            FunctionResultIdl {
                res: (ok_variant_type_id != self.unit_type_id).then_some(ok_variant_type_id),
                err: Some(err_variant_type_id),
            }
        } else {
            FunctionResultIdl {
                res: (res_type_id != self.unit_type_id).then_some(res_type_id),
                err: None,
            }
        };

        Ok(Some(res))
    }

    // Creates iterator over functions metadata by accessing it as a variant
    // of enum functions wrapper under `funcs_type_id`.
    fn fns_meta_iter(
        registry: &PortableRegistry,
        funcs_type_id: u32,
    ) -> Result<impl Iterator<Item = &Variant<PortableForm>>> {
        let fns_meta_type = registry
            .resolve(funcs_type_id)
            .ok_or(Error::TypeIdIsUnknown(funcs_type_id))?;

        let TypeDef::Variant(ref fns_meta_type_def) = fns_meta_type.type_def else {
            return Err(Error::FuncMetaIsInvalid(
                "Functions wrapper type is not a variant".to_string(),
            ));
        };

        Ok(fns_meta_type_def.variants.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::TypeInfo;
    use std::{collections::BTreeMap, iter};

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
        fn test_ctor_error<T: TypeInfo + 'static>(expected_error_msg: &str) {
            let result = ExpandedProgramMeta::new(
                Some(("TestProgram".to_string(), MetaType::new::<T>())),
                iter::empty(),
            );

            assert!(result.is_err());
            let Err(Error::FuncMetaIsInvalid(msg)) = result else {
                panic!("Expected FuncMetaIsInvalid error, got {result:?}");
            };
            assert_eq!(msg.as_str(), expected_error_msg);
        }

        // Test all error scenarios
        test_ctor_error::<NonCompositeArgsCtors>(
            "function `CtorWithInvalidArgTypes` args type is not a composite",
        );

        test_ctor_error::<NamelessFieldsCtors>(
            "function `CtorWithNamelessArgs` has nameless argument",
        );

        test_ctor_error::<NoArgsCtors>(
            "function `CtorWithNoArgs` has invalid signature: expected at least args or/and result",
        );

        test_ctor_error::<TooManyArgsCtors>(
            "function `CtorWithResult` has invalid signature: expected only args",
        );
    }

    /// Test that returned program meta has result_ty == None for all constructors in program IDL section
    #[test]
    fn ctors_result_ty_none() {
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

        let result = ExpandedProgramMeta::new(
            Some((
                "TestProgram".to_string(),
                MetaType::new::<ValidConstructors>(),
            )),
            iter::empty(),
        );

        assert!(result.is_ok());
        let meta = result.unwrap();

        // Check that all constructors have result_ty == None
        let ctors = &meta.program.as_ref().unwrap().ctors;
        assert_eq!(ctors.len(), 3);

        for ctor in ctors {
            assert!(
                ctor.result_ty.is_none(),
                "Constructor '{}' should have result_ty == None, but got {:?}",
                ctor.name,
                ctor.result_ty
            );
        }
    }

    /// Test successful creation with valid constructors and services
    #[test]
    fn ctor_simple_positive_test() {
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

        let result = ExpandedProgramMeta::new(
            Some(("TestProgram".to_string(), MetaType::new::<Ctors>())),
            std::iter::empty(),
        );

        assert!(result.is_ok());
        let meta = result.unwrap();

        let ctors = &meta.program.as_ref().unwrap().ctors;
        let ctors_json = serde_json::to_value(ctors).expect("Should serialize to JSON");
        assert_eq!(
            ctors_json,
            serde_json::json!([
                {
                    "name": "Ctor",
                    "args": [
                        {
                            "name": "initial_value",
                            "type": ctors[0].args[0].ty,
                        }
                    ],
                    "docs": [],
                }
            ])
        );
    }

    #[test]
    fn program_has_services() {
        struct TestService;
        impl sails_idl_meta::ServiceMeta for TestService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        let meta = ExpandedProgramMeta::new(
            Some((
                "TestProgram".to_string(),
                MetaType::new::<utils::SimpleCtors>(),
            )),
            vec![
                ("TestService1", AnyServiceMeta::new::<TestService>()),
                ("TestService2", AnyServiceMeta::new::<TestService>()),
                ("TestService3", AnyServiceMeta::new::<TestService>()),
            ]
            .into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        let program_section = meta.program.unwrap();
        assert_eq!(program_section.services.len(), 3);
        assert_eq!(program_section.services[0], "TestService1");
        assert_eq!(program_section.services[1], "TestService2");
        assert_eq!(program_section.services[2], "TestService3");
    }

    // ------------------------------------------------------------------------------------
    // -------------------- Extension and base services related tests ---------------------
    // ------------------------------------------------------------------------------------

    #[test]
    fn base_service_entities_occur() {
        struct BaseServiceMeta;
        impl sails_idl_meta::ServiceMeta for BaseServiceMeta {
            type CommandsMeta = BaseServiceCommands;
            type QueriesMeta = BaseServiceQueries;
            type EventsMeta = BaseServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ExtendedServiceMeta;
        impl sails_idl_meta::ServiceMeta for ExtendedServiceMeta {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = ExtendedServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] =
                &[AnyServiceMeta::new::<BaseServiceMeta>];
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

        let meta = ExpandedProgramMeta::new(
            None,
            vec![(
                "ExtendedService",
                AnyServiceMeta::new::<ExtendedServiceMeta>(),
            )]
            .into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        assert_eq!(meta.services.len(), 1);
        let service = &meta.services[0];

        // Currently service extended section is not filled.
        assert!(service.extends.is_empty());

        // Check that base service functions are inherited
        let function_check = |fns: &[FunctionIdl], expected_base_fn_name: &str| {
            assert_eq!(
                fns.len(),
                1,
                "Expected exactly one function in extended service"
            );

            let actual_base_fn_name = &fns[0].name;

            assert_eq!(
                actual_base_fn_name, expected_base_fn_name,
                "Unexpected base function name - {actual_base_fn_name}"
            );
        };

        function_check(&service.functions.commands, "BaseCmd");
        function_check(&service.functions.queries, "BaseQuery");

        // Check that events from base service are included
        let events: Vec<_> = service.events.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            events.len(),
            2,
            "Expected exactly two events in extended service"
        );
        assert_eq!(events, vec!["ExtendedEvent", "BaseEvent"]);

        // Check that types from base service are included
        let types: Vec<&str> = service.types.iter().map(|t| t.type_name()).collect();
        assert_eq!(
            types.len(),
            2,
            "Expected exactly two types in extended service"
        );
        assert_eq!(
            types,
            vec!["SomeBaseServiceType", "SomeExtendedServiceType"],
        );
    }

    #[test]
    fn service_extension_with_conflicting_names() {
        struct BaseServiceMeta;
        impl sails_idl_meta::ServiceMeta for BaseServiceMeta {
            type CommandsMeta = BaseServiceCommands;
            type QueriesMeta = BaseServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ExtendedServiceMeta;
        impl sails_idl_meta::ServiceMeta for ExtendedServiceMeta {
            type CommandsMeta = ExtendedServiceCommands;
            type QueriesMeta = ExtendedServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] =
                &[AnyServiceMeta::new::<BaseServiceMeta>];
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

        let meta = ExpandedProgramMeta::new(
            None,
            vec![(
                "ExtendedService",
                AnyServiceMeta::new::<ExtendedServiceMeta>(),
            )]
            .into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        assert_eq!(meta.services.len(), 1, "Expected one service");
        let service = &meta.services[0];

        // Check that extended service has only its own method
        let cmd_names: Vec<(&str, &str)> = service
            .functions
            .commands
            .iter()
            .map(|c| {
                let fn_result = c.result_ty.as_ref().unwrap();
                let fn_res_idx = fn_result.res.unwrap();
                (
                    c.name.as_str(),
                    service.concrete_names[fn_res_idx as usize].0.as_str(),
                )
            })
            .collect();
        assert_eq!(
            cmd_names.len(),
            1,
            "Expected one command in extended service"
        );
        assert_eq!(
            cmd_names[0].0, "ConflictingCmd",
            "Expected command name to be 'ConflictingCmd'"
        );
        assert_eq!(
            cmd_names[0].1, "bool",
            "Expected command result type to be 'bool'"
        );

        // Check that extended service has only its own query
        let query_names: Vec<(&str, &str)> = service
            .functions
            .queries
            .iter()
            .map(|q| {
                let fn_result = q.result_ty.as_ref().unwrap();
                let fn_res_idx = fn_result.res.unwrap();
                (
                    q.name.as_str(),
                    service.concrete_names[fn_res_idx as usize].0.as_str(),
                )
            })
            .collect();
        assert_eq!(
            query_names.len(),
            1,
            "Expected one query in extended service"
        );
        assert_eq!(
            query_names[0].0, "ConflictingQuery",
            "Expected query name to be 'ConflictingQuery'"
        );
        assert_eq!(
            query_names[0].1, "String",
            "Expected query result type to be 'String'"
        );
    }

    #[test]
    fn service_extension_with_conflicting_events() {
        struct BaseService;
        impl sails_idl_meta::ServiceMeta for BaseService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = BaseServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ExtendedService;
        impl sails_idl_meta::ServiceMeta for ExtendedService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = ExtendedServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] =
                &[AnyServiceMeta::new::<BaseService>];
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

        let res = ExpandedProgramMeta::new(
            None,
            vec![("ExtendedService", AnyServiceMeta::new::<ExtendedService>())].into_iter(),
        );

        assert!(res.is_err());
        let Err(Error::EventMetaIsAmbiguous(msg_err)) = res else {
            panic!("Expected EventNameConflict error, got {res:?}");
        };
        assert_eq!(
            msg_err.as_str(),
            "event `ConflictingEvent` is defined multiple times in the service inheritance chain"
        );
    }

    #[test]
    fn service_extension_with_conflicting_types() {
        struct ServiceBase;
        impl sails_idl_meta::ServiceMeta for ServiceBase {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = BaseServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[allow(unused)]
        #[derive(TypeInfo)]
        enum BaseServiceEvents {
            BaseEvent(GenericConstStruct<8>),
        }

        struct ExtensionService;
        impl sails_idl_meta::ServiceMeta for ExtensionService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = ExtendedServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] =
                &[AnyServiceMeta::new::<ServiceBase>];
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

        let meta = ExpandedProgramMeta::new(
            None,
            vec![(
                "ExtensionService",
                AnyServiceMeta::new::<ExtensionService>(),
            )]
            .into_iter(),
        )
        .unwrap();

        assert_eq!(meta.services.len(), 1);

        let types = &meta.services[0].types;
        assert_eq!(types.len(), 2);
        let type_names = types
            .iter()
            .map(|t| (t.type_name(), t.fields_type_names()))
            .collect::<Vec<_>>();
        assert_eq!(
            type_names,
            vec![
                ("GenericConstStruct1", vec!["[u8; 16]"]),
                ("GenericConstStruct2", vec!["[u8; 8]"])
            ]
        );
    }

    // ------------------------------------------------------------------------------------
    // ------------------------------ Events related tests --------------------------------
    // ------------------------------------------------------------------------------------
    #[test]
    fn invalid_events_type() {
        struct InvalidEventsService;
        impl sails_idl_meta::ServiceMeta for InvalidEventsService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = InvalidEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct InvalidEvents {
            pub field: u32,
        }

        let res = ExpandedProgramMeta::new(
            None,
            vec![(
                "InvalidEventsService",
                AnyServiceMeta::new::<InvalidEventsService>(),
            )]
            .into_iter(),
        );

        assert!(res.is_err());
        let Err(Error::FuncMetaIsInvalid(msg)) = res else {
            panic!("Expected FuncMetaIsInvalid error, got {res:?}");
        };
        assert_eq!(msg.as_str(), "Event type is not a variant");
    }

    #[test]
    fn service_events_positive_test() {
        struct EventService;
        impl sails_idl_meta::ServiceMeta for EventService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventServiceEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
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

        let meta = ExpandedProgramMeta::new(
            None,
            vec![("EventService", AnyServiceMeta::new::<EventService>())].into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        assert_eq!(meta.services.len(), 1);
        let service = &meta.services[0];
        let event_names: Vec<String> = service
            .events
            .iter()
            .map(|e| {
                if e.fields.is_empty() {
                    e.name.to_string()
                } else {
                    let fields = e
                        .fields
                        .iter()
                        .map(|f| service.concrete_names[f.ty.id as usize].0.clone())
                        .collect::<Vec<_>>()
                        .join(", ");

                    format!("{}: {}", e.name, fields)
                }
            })
            .collect();
        assert_eq!(
            event_names,
            vec![
                "Zero",
                "One: u32",
                "Two: EventTwoParams",
                "Three: ActorId, String"
            ]
        );
    }

    // ------------------------------------------------------------------------------------
    // ----------------------------- Functions related tests ------------------------------
    // ------------------------------------------------------------------------------------

    /// Test error when commands/queries are not variant types
    #[test]
    fn service_functions_non_variant_error() {
        struct NotVariantCommandsService;
        impl sails_idl_meta::ServiceMeta for NotVariantCommandsService {
            type CommandsMeta = NotVariantCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct NotVariantQueriesService;
        impl sails_idl_meta::ServiceMeta for NotVariantQueriesService {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = NotVariantQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
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

        let internal_check = |service: AnyServiceMeta| {
            let result = ExpandedProgramMeta::new(None, vec![("TestService", service)].into_iter());
            assert!(result.is_err());
            let Err(Error::FuncMetaIsInvalid(msg)) = result else {
                panic!("Expected FuncMetaIsInvalid error, got {result:?}");
            };
            assert_eq!(msg.as_str(), "Functions wrapper type is not a variant");
        };

        internal_check(AnyServiceMeta::new::<NotVariantCommandsService>());
        internal_check(AnyServiceMeta::new::<NotVariantQueriesService>());
    }

    /// Test error when service variant doesn't have exactly 2 fields
    #[test]
    fn service_variant_field_count_error() {
        struct InvalidCommandsService1;
        impl sails_idl_meta::ServiceMeta for InvalidCommandsService1 {
            type CommandsMeta = BadCommands1;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidCommandsService2;
        impl sails_idl_meta::ServiceMeta for InvalidCommandsService2 {
            type CommandsMeta = BadCommands2;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidQueriesService1;
        impl sails_idl_meta::ServiceMeta for InvalidQueriesService1 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = BadQueries1;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct InvalidQueriesService2;
        impl sails_idl_meta::ServiceMeta for InvalidQueriesService2 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = BadQueries2;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        // Commands with wrong number of fields
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands1 {
            OneField(u32),                                 // Should have 2 fields (params, result)
            ValidCmd(utils::SimpleFunctionParams, String), // Valid command for control
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands2 {
            ThreeFields(u32, String, bool), // Should have 2 fields (params, result)
            ValidCmd(utils::SimpleFunctionParams, String), // Valid command for control
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadQueries1 {
            OneField(u32), // Should have 2 fields (params, result)
            ValidQuery(utils::SimpleFunctionParams, String), // Valid query for control
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadQueries2 {
            ThreeFields(u32, String, bool), // Should have 2 fields (params, result)
            ValidQuery(utils::SimpleFunctionParams, String), // Valid query for control
        }

        let internal_check = |service: AnyServiceMeta, expected_msg: &str| {
            let result = ExpandedProgramMeta::new(None, vec![("TestService", service)].into_iter());

            assert!(result.is_err());
            let Err(Error::FuncMetaIsInvalid(msg)) = result else {
                panic!("Expected FuncMetaIsInvalid error, got {result:?}");
            };
            assert_eq!(msg.as_str(), expected_msg);
        };

        internal_check(
            AnyServiceMeta::new::<InvalidCommandsService1>(),
            "function `OneField` has invalid signature: expected args and result",
        );

        internal_check(
            AnyServiceMeta::new::<InvalidQueriesService1>(),
            "function `OneField` has invalid signature: expected args and result",
        );

        internal_check(
            AnyServiceMeta::new::<InvalidCommandsService2>(),
            "function `ThreeFields` has invalid signature: expected at least args or/and result",
        );

        internal_check(
            AnyServiceMeta::new::<InvalidQueriesService2>(),
            "function `ThreeFields` has invalid signature: expected at least args or/and result",
        );
    }

    /// Test error when service method params are not composite
    #[test]
    fn service_params_non_composite_error() {
        struct TestServiceMeta;
        impl sails_idl_meta::ServiceMeta for TestServiceMeta {
            type CommandsMeta = BadCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        // Commands where the first field (params) is not composite
        #[derive(TypeInfo)]
        #[allow(unused)]
        enum BadCommands {
            BadCmd(u32, String), // First field should be composite (struct), not primitive
        }

        let result = ExpandedProgramMeta::new(
            None,
            vec![("TestService", AnyServiceMeta::new::<TestServiceMeta>())].into_iter(),
        );

        assert!(result.is_err());
        let Err(Error::FuncMetaIsInvalid(msg)) = result else {
            panic!("Expected FuncMetaIsInvalid error, got {result:?}");
        };
        assert_eq!(
            msg.as_str(),
            "function `BadCmd` args type is not a composite"
        );
    }

    /// Test error when service method params have nameless fields
    #[test]
    fn service_params_nameless_fields_error() {
        struct BadServiceMeta;
        impl sails_idl_meta::ServiceMeta for BadServiceMeta {
            type CommandsMeta = BadCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
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

        let result = ExpandedProgramMeta::new(
            None,
            vec![("TestService", AnyServiceMeta::new::<BadServiceMeta>())].into_iter(),
        );

        assert!(result.is_err());
        let Err(Error::FuncMetaIsInvalid(msg)) = result else {
            panic!("Expected FuncMetaIsInvalid error, got {result:?}");
        };
        assert_eq!(msg.as_str(), "function `BadCmd` has nameless argument");
    }

    #[test]
    fn service_fns_result_ty() {
        struct TestServiceMeta;
        impl sails_idl_meta::ServiceMeta for TestServiceMeta {
            type CommandsMeta = TestCommands;
            type QueriesMeta = TestQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum TestCommands {
            Unit(utils::SimpleFunctionParams, ()), // Returns unit type - no result_ty
            NonUnit(utils::SimpleFunctionParams, String), // Returns non-unit type
            WithUnit(utils::SimpleFunctionParams, Result<(), u32>), // Returns value containing unit type
            Result(utils::SimpleFunctionParams, Result<u32, String>), // Returns Result with non-unit type
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum TestQueries {
            Unit(utils::SimpleFunctionParams, ()), // Returns unit type - no result_ty
            NonUnit(utils::SimpleFunctionParams, u32), // Returns non-unit type
            WithUnit(utils::SimpleFunctionParams, Result<(), u32>), // Returns value containing unit type
            Result(utils::SimpleFunctionParams, Result<u32, String>), // Returns Result with non-unit type
        }

        let meta = ExpandedProgramMeta::new(
            None,
            vec![("TestService", AnyServiceMeta::new::<TestServiceMeta>())].into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        let service = &meta.services[0];

        // Check commands
        let check_fn_result_ty = |fns: &[FunctionIdl]| {
            for f in fns {
                match f.name.as_str() {
                    "Unit" => {
                        assert!(
                            matches!(
                                f.result_ty,
                                Some(FunctionResultIdl {
                                    res: None,
                                    err: None
                                })
                            ),
                            "Command returning () should have result_ty == None"
                        );
                    }
                    "NonUnit" => {
                        assert!(
                            matches!(
                                f.result_ty,
                                Some(FunctionResultIdl {
                                    res: Some(_),
                                    err: None
                                })
                            ),
                            "Command returning non-unit should have result_ty == Some"
                        );
                    }
                    "WithUnit" => {
                        assert!(
                            matches!(
                                f.result_ty,
                                Some(FunctionResultIdl {
                                    res: None,
                                    err: Some(_)
                                })
                            ),
                            "Command returning Result<(), T> should have result_ty == Some"
                        );
                    }
                    "Result" => {
                        assert!(
                            matches!(
                                f.result_ty,
                                Some(FunctionResultIdl {
                                    res: Some(_),
                                    err: Some(_)
                                })
                            ),
                            "Command returning Result<T, E> should have result_ty == Some"
                        );
                    }
                    _ => unimplemented!("Unexpected function name: {}", f.name),
                }
            }
        };

        check_fn_result_ty(&service.functions.commands);
        check_fn_result_ty(&service.functions.queries);

        // Test JSON serialization - result_ty should not appear in JSON when None
        let json = serde_json::to_value(service).expect("Should serialize to JSON");
        let (commands, queries) = json
            .get("functions")
            .and_then(|v| {
                v.get("commands")
                    .and_then(|c| v.get("queries").map(|q| (c, q)))
            })
            .and_then(|(c, q)| {
                q.as_array()
                    .and_then(|q_arr| c.as_array().map(|c_arr| (c_arr, q_arr)))
            })
            .unwrap();

        let check_json_result_ty = |fns_json: &[serde_json::Value]| {
            for fn_json in fns_json {
                let name = fn_json.get("name").unwrap().as_str().unwrap();
                match name {
                    "Unit" => {
                        assert_eq!(
                            fn_json.get("result_ty").unwrap(),
                            &serde_json::json!({}),
                            "{name} returning () should have result_ty as empty dict in JSON",
                        );
                    }
                    "NonUnit" => {
                        assert!(
                            fn_json.get("result_ty").is_some(),
                            "{name} returning non-unit should have result_ty in JSON",
                        );
                    }
                    "WithUnit" => {
                        assert!(
                            fn_json.get("result_ty").is_some(),
                            "{name} returning Result<(), T> should have result_ty in JSON",
                        );
                    }
                    "Result" => {
                        assert!(
                            fn_json.get("result_ty").is_some(),
                            "{name} returning Result<T, E> should have result_ty in JSON",
                        );
                    }
                    _ => unimplemented!("Unexpected function name in JSON: {name}"),
                }
            }
        };

        check_json_result_ty(commands);
        check_json_result_ty(queries);
    }

    #[test]
    fn service_function_variations_positive_test() {
        struct ServiceWithOneCommand;
        impl sails_idl_meta::ServiceMeta for ServiceWithOneCommand {
            type CommandsMeta = OneFunction;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceWithOneQuery;
        impl sails_idl_meta::ServiceMeta for ServiceWithOneQuery {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = OneFunction;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct ServiceWithNoFunctions;
        impl sails_idl_meta::ServiceMeta for ServiceWithNoFunctions {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum OneFunction {
            Fn1(utils::SimpleFunctionParams, String),
        }

        let internal_check = |service: AnyServiceMeta,
                              expected_commands_count: usize,
                              expected_queries_count: usize| {
            let meta = ExpandedProgramMeta::new(None, vec![("TestService", service)].into_iter())
                .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

            let service_meta = &meta.services[0];
            assert_eq!(
                service_meta.functions.commands.len(),
                expected_commands_count,
                "Service should have {expected_commands_count} command(s)",
            );
            assert_eq!(
                service_meta.functions.queries.len(),
                expected_queries_count,
                "Service should have {expected_queries_count} query(s)",
            );

            if !service_meta.functions.commands.is_empty() {
                let cmd = &service_meta.functions.commands[0];
                assert_eq!(cmd.name, "Fn1", "Command name should be 'Fn1'");
            } else if !service_meta.functions.queries.is_empty() {
                let query = &service_meta.functions.queries[0];
                assert_eq!(query.name, "Fn1", "Query name should be 'Fn1'");
            }
        };

        internal_check(AnyServiceMeta::new::<ServiceWithOneCommand>(), 1, 0);
        internal_check(AnyServiceMeta::new::<ServiceWithOneQuery>(), 0, 1);
        internal_check(AnyServiceMeta::new::<ServiceWithNoFunctions>(), 0, 0);

        struct Service;
        impl sails_idl_meta::ServiceMeta for Service {
            type CommandsMeta = ServiceCommands;
            type QueriesMeta = ServiceQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
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

        // One argument
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct OneArg {
            pub arg1: u32,
        }

        // Multiple arguments
        #[derive(TypeInfo)]
        #[allow(unused)]
        struct MultiArgs {
            pub arg1: u32,
            pub arg2: String,
            pub arg3: bool,
        }

        let meta = ExpandedProgramMeta::new(
            None,
            vec![("TestService", AnyServiceMeta::new::<Service>())].into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        assert_eq!(meta.services.len(), 1);
        let service = &meta.services[0];

        let internal_check = |fns: &[FunctionIdl]| {
            for f in fns {
                match f.name.as_str() {
                    "NoArgs" => {
                        assert_eq!(f.args.len(), 0, "{} should have no arguments", f.name);
                    }
                    "OneArg" => {
                        assert_eq!(f.args.len(), 1, "{} should have one argument", f.name);
                        assert_eq!(f.args[0].name, "arg1", "Argument name should be 'arg1'");
                    }
                    "MultiArgs" => {
                        assert_eq!(f.args.len(), 3, "{} should have three arguments", f.name);
                        assert_eq!(
                            f.args[0].name, "arg1",
                            "First argument name should be 'arg1'"
                        );
                        assert_eq!(
                            f.args[1].name, "arg2",
                            "Second argument name should be 'arg2'"
                        );
                        assert_eq!(
                            f.args[2].name, "arg3",
                            "Third argument name should be 'arg3'"
                        );
                    }
                    "NoResult" => {
                        assert!(
                            matches!(
                                f.result_ty,
                                Some(FunctionResultIdl {
                                    res: None,
                                    err: None
                                })
                            ),
                            "{} should have no result type",
                            f.name
                        );
                    }
                    _ => unimplemented!("Unexpected function name: {}", f.name),
                }
            }
        };

        internal_check(&service.functions.commands);
        internal_check(&service.functions.queries);
    }

    // ------------------------------------------------------------------------------------
    // --------------------------- Types section related tests ----------------------------
    // ------------------------------------------------------------------------------------

    /// Test that services with only primitive/builtin types have empty types sections
    #[test]
    fn service_non_user_defined_types_excluded() {
        struct Service1;
        impl sails_idl_meta::ServiceMeta for Service1 {
            type CommandsMeta = CommandsWithNonUserDefinedArgs;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service2;
        impl sails_idl_meta::ServiceMeta for Service2 {
            type CommandsMeta = CommandWithUserDefinedArgs;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service3;
        impl sails_idl_meta::ServiceMeta for Service3 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = CommandsWithNonUserDefinedArgs;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service4;
        impl sails_idl_meta::ServiceMeta for Service4 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = CommandWithUserDefinedArgs;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service5;
        impl sails_idl_meta::ServiceMeta for Service5 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventsWithNonUserDefinedArgs;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service6;
        impl sails_idl_meta::ServiceMeta for Service6 {
            type CommandsMeta = utils::NoCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = EventsWithUserDefinedArgs;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
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
                non_zero_u8: NonZeroU8,
                non_zero_u16: NonZeroU16,
                non_zero_u32: NonZeroU32,
                non_zero_u64: NonZeroU64,
                non_zero_u128: NonZeroU128,
                non_zero_u256: NonZeroU256,
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
            pub non_zero_u8: NonZeroU8,
            pub non_zero_u16: NonZeroU16,
            pub non_zero_u32: NonZeroU32,
            pub non_zero_u64: NonZeroU64,
            pub non_zero_u128: NonZeroU128,
            pub non_zero_u256: NonZeroU256,
        }

        let internal_check = |service1: AnyServiceMeta, service2: AnyServiceMeta| {
            let meta = ExpandedProgramMeta::new(
                None,
                vec![("Service1", service1), ("Service2", service2)].into_iter(),
            )
            .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

            assert_eq!(meta.services.len(), 2);

            let service_1 = &meta.services[0];
            assert_eq!(service_1.name, "Service1");
            assert_eq!(
                service_1.types.len(),
                0,
                "Service with only primitive/builtin types should have empty types section"
            );

            let service_2 = &meta.services[1];
            assert_eq!(service_2.name, "Service2");
            assert_eq!(
                service_2.types.len(),
                1,
                "Service with user-defined types should have non-empty types section"
            );
            assert_eq!(service_2.types[0].type_name(), "NonUserDefinedArgs");
        };

        internal_check(
            AnyServiceMeta::new::<Service1>(),
            AnyServiceMeta::new::<Service2>(),
        );

        internal_check(
            AnyServiceMeta::new::<Service3>(),
            AnyServiceMeta::new::<Service4>(),
        );

        internal_check(
            AnyServiceMeta::new::<Service5>(),
            AnyServiceMeta::new::<Service6>(),
        );
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
            pub non_zero_u8: NonZeroU8,
            pub non_zero_u16: NonZeroU16,
            pub non_zero_u32: NonZeroU32,
            pub non_zero_u64: NonZeroU64,
            pub non_zero_u128: NonZeroU128,
            pub non_zero_u256: NonZeroU256,
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

        let meta1 = ExpandedProgramMeta::new(
            Some((
                "TestProgram1".to_string(),
                MetaType::new::<CtorsWithNonUserDefinedArgs>(),
            )),
            iter::empty(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        let user_defined_types = meta1.program.unwrap().types;
        assert!(user_defined_types.is_empty());

        let meta2 = ExpandedProgramMeta::new(
            Some((
                "TestProgram2".to_string(),
                MetaType::new::<CtorsWithUserDefinedArgs>(),
            )),
            iter::empty(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        let user_defined_types_2 = meta2.program.unwrap().types;
        assert_eq!(user_defined_types_2.len(), 1);
        let type_name = user_defined_types_2[0].type_name();
        assert_eq!(type_name, "CustomType");
    }

    // --------------------------------------------------------------------------------
    // ------------------------------ Miscellaneous tests -----------------------------
    // --------------------------------------------------------------------------------
    #[test]
    fn shared_and_same_name_types_across_services() {
        struct Service1Meta;
        impl sails_idl_meta::ServiceMeta for Service1Meta {
            type CommandsMeta = Service1Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct Service2Meta;
        impl sails_idl_meta::ServiceMeta for Service2Meta {
            type CommandsMeta = Service2Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
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

        let meta = ExpandedProgramMeta::new(
            None,
            vec![
                ("Service1", AnyServiceMeta::new::<Service1Meta>()),
                ("Service2", AnyServiceMeta::new::<Service2Meta>()),
            ]
            .into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        assert_eq!(meta.services.len(), 2, "Expected two services");

        // Helper to check type names in a service
        let check_service_types = |service: &ServiceSection, expected_types: &[&str]| {
            let actual_types = service
                .types
                .iter()
                .map(|t| t.type_name())
                .collect::<HashSet<_>>();
            for expected_type in expected_types {
                assert!(
                    actual_types.contains(expected_type),
                    "Service '{}' should contain type '{}'. Available types: {:?}",
                    service.name,
                    expected_type,
                    actual_types
                );
            }
        };

        check_service_types(
            &meta.services[0],
            &[
                "TestsSimpleFunctionParams",
                "UtilsSimpleFunctionParams",
                "SharedCustomType",
            ],
        );
        check_service_types(
            &meta.services[1],
            &[
                "TestsSimpleFunctionParams",
                "UtilsSimpleFunctionParams",
                "SharedCustomType",
            ],
        );
    }

    /// Test that flat_meta returns metadata in correct order: current service first, then base services in declaration order
    #[test]
    fn flat_meta_order() {
        // Create a service inheritance hierarchy:
        // ExtendedService extends BaseService1 and BaseService2 (in that order)
        // BaseService1 extends GrandBaseService1
        // BaseService2 extends GrandBaseService2

        struct GrandBaseService1;
        impl sails_idl_meta::ServiceMeta for GrandBaseService1 {
            type CommandsMeta = GrandBase1Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct GrandBaseService2;
        impl sails_idl_meta::ServiceMeta for GrandBaseService2 {
            type CommandsMeta = GrandBase2Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        struct BaseService1;
        impl sails_idl_meta::ServiceMeta for BaseService1 {
            type CommandsMeta = Base1Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] =
                &[AnyServiceMeta::new::<GrandBaseService1>];
            const ASYNC: bool = false;
        }

        struct BaseService2;
        impl sails_idl_meta::ServiceMeta for BaseService2 {
            type CommandsMeta = Base2Commands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] =
                &[AnyServiceMeta::new::<GrandBaseService2>];
            const ASYNC: bool = false;
        }

        struct ExtendedService;
        impl sails_idl_meta::ServiceMeta for ExtendedService {
            type CommandsMeta = ExtendedCommands;
            type QueriesMeta = utils::NoQueries;
            type EventsMeta = utils::NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[
                AnyServiceMeta::new::<BaseService1>,
                AnyServiceMeta::new::<BaseService2>,
            ];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ExtendedCommands {
            ExtendedCmd(utils::SimpleFunctionParams, String),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Base1Commands {
            Base1Cmd(utils::SimpleFunctionParams, u32),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum Base2Commands {
            Base2Cmd(utils::SimpleFunctionParams, bool),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum GrandBase1Commands {
            GrandBase1Cmd(utils::SimpleFunctionParams, u64),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum GrandBase2Commands {
            GrandBase2Cmd(utils::SimpleFunctionParams, u8),
        }

        let meta = ExpandedProgramMeta::new(
            None,
            vec![("ExtendedService", AnyServiceMeta::new::<ExtendedService>())].into_iter(),
        )
        .unwrap_or_else(|e| panic!("Failed to create expanded meta: {e:?}"));

        assert_eq!(meta.services.len(), 1);
        let service = &meta.services[0];

        // Commands should appear in the order: Extended, Base1, GrandBase1, Base2, GrandBase2
        let cmd_names: Vec<&str> = service
            .functions
            .commands
            .iter()
            .map(|c| c.name.as_str())
            .collect();

        assert_eq!(
            cmd_names.len(),
            5,
            "Expected 5 commands from service hierarchy"
        );

        assert_eq!(
            cmd_names,
            vec![
                "ExtendedCmd",   // Current service first
                "Base1Cmd",      // First base service
                "GrandBase1Cmd", // Base of first base service
                "Base2Cmd",      // Second base service
                "GrandBase2Cmd", // Base of second base service
            ],
            "Commands should appear in order: current service, then base services in declaration order (depth-first)"
        );
    }
}
