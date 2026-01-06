use crate::ast;
use crate::{
    ast::{IdlDoc, InterfaceId, PrimitiveType},
    error::{Error, Result},
    visitor::{self, Visitor},
};
use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::str::FromStr;

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

    // 1. Manually set up the program-level scope so it persists across all sibling service visits.
    let program_scope_start = validator.names_stack.len();
    if let Some(program) = &doc.program {
        for ty in &program.types {
            validator.add_type_to_current_scope(&ty.name);
        }
    }

    // 2. Traverse the program itself to validate its own nodes.
    if let Some(program) = &doc.program {
        validator.visit_program_unit(program);
    }

    // 3. Traverse each service. The program scope is still on the stack.
    for service in &doc.services {
        validator.visit_service_unit(service);
    }

    // 4. Manually pop the program scope after the entire traversal is complete.
    validator.unwind_scope(program_scope_start);

    if !validator.errors.is_empty() {
        let error_messages: Vec<String> = validator
            .errors
            .into_iter()
            .map(|e| e.to_string())
            .collect();
        return Err(Error::Validation(error_messages.join("\n")));
    }

    // 5. Compute `interface_id` for each service in doc
    let mut service_ids = ServiceInterfaceId::new(doc);
    service_ids.update_service_id()?;

    Ok(())
}

struct Validator<'a> {
    // Counts of active type names in the current scope chain.
    active_names: BTreeMap<&'a str, u32>,
    // Stack of all visible type names, used for rewinding scopes.
    names_stack: Vec<&'a str>,
    errors: Vec<Error>,
}

impl<'a> Validator<'a> {
    fn new() -> Self {
        Self {
            active_names: BTreeMap::new(),
            names_stack: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn unwind_scope(&mut self, start_index: usize) {
        // Remove types defined in this scope from the active set
        for name in self.names_stack.drain(start_index..) {
            if let Some(count) = self.active_names.get_mut(name) {
                *count -= 1;
                if *count == 0 {
                    self.active_names.remove(name);
                }
            }
        }
    }

    fn add_type_to_current_scope(&mut self, name: &'a str) {
        self.names_stack.push(name);
        *self.active_names.entry(name).or_insert(0) += 1;
    }

    fn is_type_known(&self, name: &str) -> bool {
        if ALLOWED_TYPES.contains(&name) {
            return true;
        }
        self.active_names.contains_key(name)
    }
}

impl<'a> visitor::Visitor<'a> for Validator<'a> {
    fn visit_service_unit(&mut self, service: &'a ast::ServiceUnit) {
        let scope_start = self.names_stack.len();
        for ty in &service.types {
            self.add_type_to_current_scope(&ty.name);
        }

        visitor::accept_service_unit(service, self);

        self.unwind_scope(scope_start);
    }

    fn visit_type(&mut self, ty: &'a ast::Type) {
        let scope_start = self.names_stack.len();
        for param in &ty.type_params {
            self.add_type_to_current_scope(&param.name);
        }

        // Now that generics are in scope, traverse the type's definition.
        visitor::accept_type(ty, self);

        self.unwind_scope(scope_start);
    }

    fn visit_named_type_decl(&mut self, name: &'a str, generics: &'a [ast::TypeDecl]) {
        if PrimitiveType::from_str(name).is_err() && !self.is_type_known(name) {
            self.errors
                .push(Error::Validation(format!("Unknown type '{name}'")));
        }

        for generic in generics {
            visitor::accept_type_decl(generic, self);
        }
    }

    fn visit_struct_def(&mut self, struct_def: &'a ast::StructDef) {
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
            self.compute_service_id(name.as_str())?;
        }
        if let Some(program) = &mut self.doc.program {
            for expo in &mut program.services {
                if let Some(id) = self.computed.get(&expo.name.name) {
                    expo.name.interface_id = Some(InterfaceId(id.0));
                } else {
                    return Err(Error::Validation(format!(
                        "service `{}`: `interface_id` is not copmputed",
                        expo.name.name
                    )));
                }
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
            let _ = self.compute_service_id(&base);
        }

        let service = &mut self.doc.services[idx];
        for ext in &mut service.extends {
            if let Some(id) = self.computed.get(&ext.name) {
                ext.interface_id = Some(InterfaceId(id.0));
            }
        }
        let id = service.interface_id().map_err(Error::Validation)?;
        if let Some(current_id) = service.name.interface_id
            && current_id != id
        {
            return Err(Error::Validation(format!(
                "service `{name}` computed interface_id {id} is not equal to {current_id} in IDL"
            )));
        }
        service.name.interface_id = Some(id);
        self.computed.insert(name.to_string(), id);
        Ok(id)
    }
}
