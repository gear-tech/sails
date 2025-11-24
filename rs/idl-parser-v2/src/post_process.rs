use crate::ast::{self, IdlDoc, PrimitiveType};
use crate::visitor::{self, Visitor};
use anyhow::{Result, bail};
use std::collections::HashSet;
use std::str::FromStr;

const CORE_CONTAINERS: &[&str] = &["Option", "Result", "Vec"];

const COLLECTIONS: &[&str] = &[
    "BTreeMap",
    "BTreeSet",
    "HashMap",
    "HashSet",
    "VecDeque",
    "LinkedList",
    "BinaryHeap",
];

const NUM_TYPES: &[&str] = &[
    "NonZeroU8",
    "NonZeroU16",
    "NonZeroU32",
    "NonZeroU64",
    "NonZeroU128",
    "NonZeroU256",
];

const SAILS_TYPES: &[&str] = &["ActorId", "CodeId", "MessageId", "H160", "H256", "U256"];

pub fn validate_and_post_process(doc: &mut IdlDoc) -> Result<()> {
    let mut validator = Validator::new();

    // 1. Manually set up the program-level scope so it persists across all sibling service visits.
    if let Some(program) = &doc.program {
        validator.push_scope(); // Program scope
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
    if doc.program.is_some() {
        validator.pop_scope();
    }

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
    scopes: Vec<HashSet<&'a str>>,
    errors: Vec<anyhow::Error>,
}

impl<'a> Validator<'a> {
    fn new() -> Self {
        let global_scope: HashSet<&str> = [CORE_CONTAINERS, COLLECTIONS, NUM_TYPES, SAILS_TYPES]
            .concat()
            .iter()
            .copied()
            .collect();

        Self {
            scopes: vec![global_scope],
            errors: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn add_type_to_current_scope(&mut self, name: &'a str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    fn is_type_known(&self, name: &str) -> bool {
        // Check from innermost scope to outermost (from top of the stack to bottom).
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }
}

impl<'a> visitor::Visitor<'a> for Validator<'a> {
    fn visit_service_unit(&mut self, service: &'a ast::ServiceUnit) {
        self.push_scope();
        for ty in &service.types {
            self.add_type_to_current_scope(&ty.name);
        }

        visitor::accept_service_unit(service, self);

        self.pop_scope();
    }

    fn visit_type(&mut self, ty: &'a ast::Type) {
        self.push_scope();
        for param in &ty.type_params {
            self.add_type_to_current_scope(&param.name);
        }

        // Now that generics are in scope, traverse the type's definition.
        visitor::accept_type(ty, self);

        self.pop_scope();
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
