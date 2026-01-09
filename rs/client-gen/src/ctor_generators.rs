use crate::helpers::{encoded_args, fn_args_with_types_path, generate_doc_comments};
use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser_v2::{ast, visitor::Visitor};
use std::collections::HashMap;

pub(crate) struct CtorGenerator<'ast> {
    program_name: &'ast str,
    sails_path: &'ast str,
    ctor_tokens: Tokens,
    io_tokens: Tokens,
    trait_ctors_tokens: Tokens,
    entry_ids: HashMap<&'ast str, u16>,
}

impl<'ast> CtorGenerator<'ast> {
    pub(crate) fn new(program_name: &'ast str, sails_path: &'ast str) -> Self {
        Self {
            program_name,
            sails_path,
            ctor_tokens: Tokens::new(),
            io_tokens: Tokens::new(),
            trait_ctors_tokens: Tokens::new(),
            entry_ids: HashMap::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote! {
            pub trait $(self.program_name)Ctors {
                type Env: $(self.sails_path)::client::GearEnv;
                $(self.trait_ctors_tokens)
            }

            impl<E: $(self.sails_path)::client::GearEnv> $(self.program_name)Ctors for $(self.sails_path)::client::Deployment<$(self.program_name)Program, E> {
                type Env = E;
                $(self.ctor_tokens)
            }

            $['\n']
            pub mod io {
                use super::*;
                $(self.io_tokens)
            }
        }
    }
}

impl<'ast> Visitor<'ast> for CtorGenerator<'ast> {
    fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
        for (idx, ctor) in program.ctors.iter().enumerate() {
            self.entry_ids.insert(&ctor.name, idx as u16);
        }

        sails_idl_parser_v2::visitor::accept_program_unit(program, self);
    }

    fn visit_ctor_func(&mut self, func: &'ast ast::CtorFunc) {
        let fn_name = &func.name;
        let fn_name_snake = &fn_name.to_case(Case::Snake);

        let params_with_types = &fn_args_with_types_path(&func.params, "");
        let args = &encoded_args(&func.params);

        generate_doc_comments(&mut self.trait_ctors_tokens, &func.docs);

        if fn_name_snake == "new" {
            quote_in! {self.trait_ctors_tokens =>
                $['\r'] #[allow(clippy::new_ret_no_self)]
                $['\r'] #[allow(clippy::wrong_self_convention)]
            };
        }

        quote_in! { self.trait_ctors_tokens =>
            $['\r']
            fn $fn_name_snake (self, $params_with_types) -> $(self.sails_path)::client::PendingCtor<$(self.program_name)Program, io::$fn_name, Self::Env>;
        };

        quote_in! { self.ctor_tokens =>
            $['\r']
            fn $fn_name_snake (self, $params_with_types) -> $(self.sails_path)::client::PendingCtor<$(self.program_name)Program, io::$fn_name, Self::Env> {
                self.pending_ctor($args)
            }
        };

        let params_with_types_super = &fn_args_with_types_path(&func.params, "super");
        let entry_id = self.entry_ids.get(func.name.as_str()).copied().unwrap_or(0);
        quote_in! { self.io_tokens =>
            $(self.sails_path)::io_struct_impl!($fn_name ($params_with_types_super) -> (), $entry_id);
        };
    }
}
