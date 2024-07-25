use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::{
    helpers::*, io_generators::generate_io_struct, type_generators::generate_type_decl_code,
};

pub(crate) struct CtorFactoryGenerator {
    service_name: String,
    tokens: Tokens,
    io_tokens: Tokens,
}

impl CtorFactoryGenerator {
    pub(crate) fn new(service_name: String) -> Self {
        Self {
            service_name,
            tokens: Tokens::new(),
            io_tokens: Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> Tokens {
        let service_name_snake = self.service_name.to_case(Case::Snake);
        quote! {
            $(self.tokens)
            pub mod $(service_name_snake)_factory {
                use super::*;
                pub mod io {
                    use super::*;
                    use sails_rs::calls::ActionIo;
                    $(self.io_tokens)
                }
            }
        }
    }
}

impl<'ast> Visitor<'ast> for CtorFactoryGenerator {
    fn visit_ctor(&mut self, ctor: &'ast Ctor) {
        quote_in! {self.tokens =>
            pub struct $(&self.service_name)Factory<R> {
                #[allow(dead_code)]
                remoting: R,
            }

            impl<R> $(&self.service_name)Factory<R> {
                #[allow(unused)]
                pub fn new(remoting: R) -> Self {
                    Self {
                        remoting,
                    }
                }
            }

            impl<R: Remoting + Clone> traits::$(&self.service_name)Factory for $(&self.service_name)Factory<R> $("{")
                type Args = R::Args;
        };

        visitor::accept_ctor(ctor, self);

        quote_in! { self.tokens =>
            $("}")
        };
    }

    fn visit_ctor_func(&mut self, func: &'ast CtorFunc) {
        let fn_name = func.name();
        let fn_name_snake = fn_name.to_case(Case::Snake);
        let fn_name_snake = fn_name_snake.as_str();

        quote_in! { self.tokens =>
            fn $fn_name_snake$("(")&self,
        };

        visitor::accept_ctor_func(func, self);

        let args = encoded_args(func.params());

        let service_name_snake = self.service_name.to_case(Case::Snake);
        let params_type = format!("{service_name_snake}_factory::io::{fn_name}");

        quote_in! { self.tokens =>
            $(")") -> impl Activation<Args = R::Args> {
                RemotingAction::<_, $params_type>::new(self.remoting.clone(), $args)
            }
        };

        let route_bytes = path_bytes(fn_name).0;
        let struct_tokens = generate_io_struct(fn_name, func.params(), None, route_bytes.as_str());

        quote_in! { self.io_tokens =>
            $struct_tokens
        };
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());
        quote_in! { self.tokens =>
            $(func_param.name()): $(type_decl_code),
        };
    }
}

pub(crate) struct CtorTraitGenerator {
    service_name: String,
    tokens: Tokens,
}

impl CtorTraitGenerator {
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

impl<'ast> Visitor<'ast> for CtorTraitGenerator {
    fn visit_ctor(&mut self, ctor: &'ast Ctor) {
        quote_in! {self.tokens =>
            #[allow(dead_code)]
            pub trait $(&self.service_name)Factory $("{")
                type Args;
        };

        visitor::accept_ctor(ctor, self);

        quote_in! {self.tokens =>
            $("}")
        };
    }

    fn visit_ctor_func(&mut self, func: &'ast CtorFunc) {
        let fn_name = func.name();
        let fn_name_snake = fn_name.to_case(Case::Snake);
        let fn_name_snake = fn_name_snake.as_str();

        if fn_name_snake == "new" {
            quote_in! {self.tokens =>
                #[allow(clippy::new_ret_no_self)]
                #[allow(clippy::wrong_self_convention)]
            };
        }

        quote_in! {self.tokens =>
            fn $fn_name_snake$("(")&self,
        };

        visitor::accept_ctor_func(func, self);

        quote_in! {self.tokens =>
            $(")") -> impl Activation<Args = Self::Args>;
        };
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());

        quote_in! { self.tokens =>
            $(func_param.name()): $(type_decl_code),
        };
    }
}
