use crate::helpers::*;
use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct CtorGenerator<'a> {
    service_name: &'a str,
    sails_path: &'a str,
    ctor_tokens: Tokens,
    io_tokens: Tokens,
    trait_ctors_tokens: Tokens,
}

impl<'a> CtorGenerator<'a> {
    pub(crate) fn new(service_name: &'a str, sails_path: &'a str) -> Self {
        Self {
            service_name,
            sails_path,
            ctor_tokens: Tokens::new(),
            io_tokens: Tokens::new(),
            trait_ctors_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        quote! {
            pub trait $(self.service_name)Ctors {
                type Env: $(self.sails_path)::client::GearEnv;
                $(self.trait_ctors_tokens)
            }

            impl<E: $(self.sails_path)::client::GearEnv> $(self.service_name)Ctors for $(self.sails_path)::client::Deployment<E, $(self.service_name)Program> {
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

impl<'ast> Visitor<'ast> for CtorGenerator<'_> {
    fn visit_ctor(&mut self, ctor: &'ast Ctor) {
        visitor::accept_ctor(ctor, self);
    }

    fn visit_ctor_func(&mut self, func: &'ast CtorFunc) {
        let fn_name = func.name();
        let fn_name_snake = &fn_name.to_case(Case::Snake);

        let params_with_types = &fn_args_with_types(func.params());
        let args = &encoded_args(func.params());

        for doc in func.docs() {
            quote_in! { self.trait_ctors_tokens =>
                $['\r'] $("///") $doc
            };
        }

        if fn_name_snake == "new" {
            quote_in! {self.trait_ctors_tokens =>
                $['\r'] #[allow(clippy::new_ret_no_self)]
                $['\r'] #[allow(clippy::wrong_self_convention)]
            };
        }

        quote_in! { self.trait_ctors_tokens =>
            $['\r']
            fn $fn_name_snake (self, $params_with_types) -> $(self.sails_path)::client::PendingCtor<Self::Env, $(self.service_name)Program, io::$fn_name>;
        };

        quote_in! { self.ctor_tokens =>
            $['\r']
            fn $fn_name_snake (self, $params_with_types) -> $(self.sails_path)::client::PendingCtor<Self::Env, $(self.service_name)Program, io::$fn_name> {
                self.pending_ctor($args)
            }
        };

        let params_with_types_super = &fn_args_with_types_path(func.params(), "super");
        quote_in! { self.io_tokens =>
            $(self.sails_path)::io_struct_impl!($fn_name ($params_with_types_super) -> ());
        };
    }
}
