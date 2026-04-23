use crate::{
    error::{Error, Result},
    visitor::{self, Visitor},
};
use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::str::FromStr;
use sails_idl_ast::*;

const ALLOWED_TYPES: &[&str] = &[
    "Option",
    "Result",
    "NonZeroU8",
    "NonZeroU16",
    "NonZeroU32",
    "NonZeroU64",
    "NonZeroU128",
    "NonZeroU256",
];

pub fn validate_and_post_process(doc: &mut IdlDoc) -> Result<()> {
    let mut validator = Validator::new();

    // 1. Program types are added to the root scope so they remain visible to all services.
    if let Some(program) = &doc.program {
        for ty in &program.types {
            validator.add_type(&ty.name);
        }
        // 2. Validate the program unit (ctors, type references, field consistency).
        validator.visit_program_unit(program);
    }

    // 3. Validate each service unit (funcs, events, types, field consistency).
    for service in &doc.services {
        validator.visit_service_unit(service);
    }

    // 4. Collect and return any validation errors found above.
    if !validator.errors.is_empty() {
        let error_messages: Vec<String> = validator
            .errors
            .into_iter()
            .map(|e| e.to_string())
            .collect();
        return Err(Error::Validation(error_messages.join("\n")));
    }

    // 5. Validate entry_ids: check uniqueness and that @partial services have explicit @entry_id.
    validate_entry_ids(doc)?;

    // 6. Compute and assign `interface_id` for each service.
    let mut service_ids = ServiceInterfaceId::new(doc);
    service_ids.update_service_id()?;

    Ok(())
}

struct Validator<'a> {
    scopes: Vec<Vec<&'a str>>,
    errors: Vec<Error>,
}

impl<'a> Validator<'a> {
    fn new() -> Self {
        Self {
            scopes: vec![vec![]],
            errors: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn add_type(&mut self, name: &'a str) {
        self.scopes.last_mut().unwrap().push(name);
    }

    fn is_type_known(&self, name: &str) -> bool {
        ALLOWED_TYPES.contains(&name) || self.scopes.iter().any(|s| s.contains(&name))
    }
}

impl<'a> visitor::Visitor<'a> for Validator<'a> {
    fn visit_service_unit(&mut self, service: &'a ServiceUnit) {
        self.push_scope();
        for ty in &service.types {
            self.add_type(&ty.name);
        }
        visitor::accept_service_unit(service, self);
        self.pop_scope();
    }

    fn visit_type(&mut self, ty: &'a Type) {
        self.push_scope();
        for param in &ty.type_params {
            self.add_type(&param.name);
        }
        visitor::accept_type(ty, self);
        self.pop_scope();
    }

    fn visit_named_type_decl(&mut self, name: &'a str, generics: &'a [TypeDecl]) {
        if PrimitiveType::from_str(name).is_err() && !self.is_type_known(name) {
            self.errors
                .push(Error::Validation(format!("Unknown type '{name}'")));
        }

        for generic in generics {
            visitor::accept_type_decl(generic, self);
        }
    }

    fn visit_struct_def(&mut self, struct_def: &'a StructDef) {
        if !struct_def.fields.is_empty() {
            let first_field_is_named = struct_def.fields[0].name.is_some();
            if !struct_def
                .fields
                .iter()
                .all(|f| f.name.is_some() == first_field_is_named)
            {
                self.errors.push(Error::Validation(
                    "Mixing named and unnamed fields in a struct or enum variant is not allowed."
                        .to_string(),
                ));
            }
        }

        visitor::accept_struct_def(struct_def, self);
    }
}

struct ServiceInterfaceId<'a> {
    doc: &'a mut IdlDoc,
    service_idx: BTreeMap<String, usize>,
    computed: BTreeMap<String, InterfaceId>,
}

impl<'a> ServiceInterfaceId<'a> {
    fn new(doc: &'a mut IdlDoc) -> Self {
        let service_index = doc
            .services
            .iter()
            .enumerate()
            .map(|(idx, s)| (s.name.name.to_string(), idx))
            .collect();
        Self {
            doc,
            service_idx: service_index,
            computed: BTreeMap::new(),
        }
    }

    fn update_service_id(&mut self) -> Result<()> {
        let names: Vec<_> = self
            .doc
            .services
            .iter()
            .map(|s| s.name.name.to_string())
            .collect();
        for name in names {
            _ = self.compute_service_id(name.as_str())?;
        }

        let mut seen_ids = BTreeMap::new();
        for service in &self.doc.services {
            let id = service.name.interface_id.expect("interface_id must be set");
            if let Some(other_name) = seen_ids.insert(id.as_u64(), &service.name.name) {
                return Err(Error::Validation(format!(
                    "duplicate interface_id {id} found for services `{}` and `{}`",
                    other_name, service.name.name
                )));
            }
        }

        if let Some(program) = &mut self.doc.program {
            for expo in &mut program.services {
                let id = self.computed.get(&expo.name.name).ok_or_else(|| {
                    Error::Validation(format!(
                        "service `{}`: `interface_id` is not computed",
                        expo.name.name
                    ))
                })?;
                expo.name.interface_id = Some(*id);
            }
        }
        Ok(())
    }

    fn compute_service_id(&mut self, name: &str) -> Result<InterfaceId> {
        if let Some(id) = self.computed.get(name) {
            return Ok(*id);
        }
        let &idx = self
            .service_idx
            .get(name)
            .ok_or_else(|| Error::Validation(format!("service `{name}` not found in IDL")))?;

        let base_names: Vec<String> = self.doc.services[idx]
            .extends
            .iter()
            .map(|base| base.name.clone())
            .collect();

        for base in base_names {
            _ = self.compute_service_id(&base)?;
        }

        let service = &mut self.doc.services[idx];
        for ext in &mut service.extends {
            let id = self.computed.get(&ext.name).ok_or_else(|| {
                Error::Validation(format!(
                    "service `{}`: `interface_id` is not computed",
                    ext.name
                ))
            })?;
            ext.interface_id = Some(*id);
        }

        let id = if service.is_partial() {
            service.name.interface_id.ok_or_else(|| {
                Error::Validation(format!(
                    "service `{name}` is marked as `@partial` but does not have an explicit `interface_id` (e.g. `service {name}@0x...`)"
                ))
            })?
        } else {
            let id = service.interface_id().map_err(Error::Validation)?;
            if let Some(current_id) = service.name.interface_id
                && current_id != id
            {
                return Err(Error::Validation(format!(
                    "service `{name}` computed interface_id {id} is not equal to {current_id} in IDL"
                )));
            }
            id
        };

        service.name.interface_id = Some(id);
        self.computed.insert(name.to_string(), id);
        Ok(id)
    }
}

fn validate_entry_ids(doc: &IdlDoc) -> Result<()> {
    for service in &doc.services {
        let is_partial = service.is_partial();

        for func in &service.funcs {
            validate_entry_id_annotation(
                "service",
                &service.name.name,
                "function",
                &func.name,
                &func.annotations,
                is_partial,
            )?;
        }
        for event in &service.events {
            validate_entry_id_annotation(
                "service",
                &service.name.name,
                "event",
                &event.name,
                &event.annotations,
                is_partial,
            )?;
        }

        // entry_ids must be unique within funcs and within events
        let mut seen = alloc::collections::BTreeSet::new();
        for func in &service.funcs {
            if !seen.insert(func.entry_id) {
                return Err(Error::Validation(format!(
                    "service `{}`: duplicate entry_id {} among functions",
                    service.name.name, func.entry_id
                )));
            }
        }
        seen.clear();
        for event in &service.events {
            if !seen.insert(event.entry_id) {
                return Err(Error::Validation(format!(
                    "service `{}`: duplicate entry_id {} among events",
                    service.name.name, event.entry_id
                )));
            }
        }
    }

    if let Some(program) = &doc.program {
        let mut seen = alloc::collections::BTreeSet::new();
        for ctor in &program.ctors {
            validate_entry_id_annotation(
                "program",
                &program.name,
                "constructor",
                &ctor.name,
                &ctor.annotations,
                false,
            )?;
            if !seen.insert(ctor.entry_id) {
                return Err(Error::Validation(format!(
                    "program `{}`: duplicate entry_id {} among constructors",
                    program.name, ctor.entry_id
                )));
            }
        }
    }

    Ok(())
}

fn validate_entry_id_annotation(
    owner_kind: &str,
    owner_name: &str,
    item_kind: &str,
    item_name: &str,
    annotations: &[(String, Option<String>)],
    required: bool,
) -> Result<()> {
    let Some((_, value)) = annotations.iter().find(|(k, _)| k == "entry_id") else {
        if required {
            return Err(Error::Validation(format!(
                "{owner_kind} `{owner_name}`: {item_kind} `{item_name}` is missing `@entry_id` annotation (required for @partial services)"
            )));
        }
        return Ok(());
    };

    let Some(value) = value.as_deref() else {
        return Err(Error::Validation(format!(
            "{owner_kind} `{owner_name}`: {item_kind} `{item_name}` has invalid `@entry_id` value (expected a u16)"
        )));
    };

    value.parse::<u16>().map_err(|_| {
        Error::Validation(format!(
            "{owner_kind} `{owner_name}`: {item_kind} `{item_name}` has invalid `@entry_id` value `{value}` (expected a u16)"
        ))
    })?;

    Ok(())
}
