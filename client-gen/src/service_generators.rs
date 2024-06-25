use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::helpers::*;
use crate::type_generators::generate_type_decl_code;

/// Generates a trait with service methods
pub(crate) struct ServiceTraitGenerator {
    service_name: String,
    tokens: Tokens,
}

impl ServiceTraitGenerator {
    pub(crate) fn new(service_name: String) -> Self {
        Self {
            service_name,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for ServiceTraitGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        quote_in! { self.tokens =>
            pub trait $(&self.service_name)<TCallArgs> $("{")
        };

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $("}")
        };
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name().to_case(Case::Snake);

        quote_in! { self.tokens=>
            #[allow(clippy::type_complexity)]
            fn $fn_name $("(")&$mutability self,
        };

        visitor::accept_service_func(func, self);
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());
        quote_in! { self.tokens =>
            $(func_param.name()): $(type_decl_code),
        };
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        let type_decl_code = generate_type_decl_code(func_output);
        quote_in! { self.tokens =>
            $(")") -> impl Call<TCallArgs, $type_decl_code>;
        };
    }
}

/// Generates a client that implements service trait
pub(crate) struct ServiceClientGenerator {
    service_name: String,
    path: String,
    tokens: Tokens,
}

impl ServiceClientGenerator {
    pub(crate) fn new(service_name: String, path: String) -> Self {
        Self {
            service_name,
            path,
            tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        self.tokens
    }
}

// using quote_in instead of tokens.append
impl<'ast> Visitor<'ast> for ServiceClientGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        let name = &self.service_name;

        quote_in! {self.tokens =>
            #[derive(Clone)]
            pub struct $name<R, A> where
                R: Remoting<A>,
                A: Default,
            {
                remoting: R,
                _phantom: PhantomData<A>,
            }

            impl<A: Default, R: Remoting<A>> $name<R, A> {
                pub fn new(remoting: R) -> Self {
                    Self { remoting, _phantom: PhantomData }
                }
            }

            impl<R, A> traits::$name<A> for $name<R, A>
            where
                R: Remoting<A> + Clone,
                A: Default,
            $("{")
        };

        visitor::accept_service(service, self);

        quote_in! {self.tokens =>
            $("}")
        };
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name();
        let fn_name_snake = fn_name.to_case(Case::Snake);
        let service_name = self.service_name.to_case(Case::Snake);

        quote_in! {self.tokens =>
            fn $fn_name_snake $("(")&$mutability self,
        };

        visitor::accept_service_func(func, self);

        let args = encoded_fn_args(func.params());

        let (service_path_bytes, _service_path_encoded_length) = path_bytes(&self.path);
        let (route_bytes, _route_encoded_length) = method_bytes(fn_name);

        quote_in! {self.tokens =>
            {
                RemotingAction::new(self.remoting.clone(), &[$service_path_bytes $route_bytes], $(service_name)_io::$fn_name::encode_call($args))
            }
        };
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());

        quote_in! {self.tokens =>
            $(func_param.name()): $(type_decl_code),
        };
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        let type_decl_code = generate_type_decl_code(func_output);

        quote_in! {self.tokens =>
            $(")") -> impl Call<A, $type_decl_code>
        };
    }
}
