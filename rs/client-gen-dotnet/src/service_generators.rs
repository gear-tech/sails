use crate::helpers::*;
use crate::type_generators::generate_type_decl_code;
use convert_case::{Case, Casing};
use csharp::Tokens;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

/// Generates a client that implements service trait
pub(crate) struct ServiceClientGenerator {
    service_name: String,
    interface_tokens: Tokens,
    class_tokens: Tokens,
}

impl ServiceClientGenerator {
    pub(crate) fn new(service_name: String) -> Self {
        Self {
            service_name,
            interface_tokens: Tokens::new(),
            class_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let name = &self.service_name;
        quote! {
            public interface I$name
            {
                $(self.interface_tokens)
            }

            public partial class $name : I$name
            {
                private readonly IRemoting remoting;

                public $name(IRemoting remoting)
                {
                    this.remoting = remoting;
                }

                $(self.class_tokens)
            }
        }
    }
}

// using quote_in instead of tokens.append
impl<'ast> Visitor<'ast> for ServiceClientGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        let name = &self.service_name;

        visitor::accept_service(service, self);
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name();
        let fn_name_snake = fn_name.to_case(Case::Snake);
    }
}
