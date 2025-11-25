use crate::ast::{self, IdlDoc, PrimitiveType};
use crate::visitor::{self, Visitor};
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::str::FromStr;

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
        bail!(error_messages.join("\n"));
    }

    Ok(())
}

struct Validator<'a> {
    // Counts of active type names in the current scope chain.
    active_names: HashMap<&'a str, u32>,
    // Stack of all visible type names, used for rewinding scopes.
    names_stack: Vec<&'a str>,
    errors: Vec<anyhow::Error>,
}

impl<'a> Validator<'a> {
    fn new() -> Self {
        Self {
            active_names: HashMap::new(),
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
            self.errors.push(anyhow::anyhow!("Unknown type '{}'", name));
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
                self.errors.push(anyhow::anyhow!(
                    "Mixing named and unnamed fields in a struct or enum variant is not allowed."
                ));
            }
        }

        visitor::accept_struct_def(struct_def, self);
    }
}
