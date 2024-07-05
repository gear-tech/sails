use convert_case::{Case, Casing};
use genco::prelude::*;
use rust::Tokens;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::{helpers::*, type_generators::generate_type_decl_code};

pub(crate) struct CtorFactoryGenerator {
    service_name: String,
    tokens: Tokens,
}

impl CtorFactoryGenerator {
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

impl<'ast> Visitor<'ast> for CtorFactoryGenerator {
    fn visit_ctor(&mut self, ctor: &'ast Ctor) {
        quote_in! {self.tokens =>
            pub struct $(&self.service_name)Factory<R, A> {
                remoting: R,
                _phantom: PhantomData<A>,
            }

            impl<R: Remoting<A>, A> $(&self.service_name)Factory<R, A> {
                #[allow(unused)]
                pub fn new(remoting: R) -> Self {
                    Self {
                        remoting,
                        _phantom: PhantomData,
                    }
                }
            }

            impl<R: Remoting<A> + Clone, A: Default> traits::$(&self.service_name)Factory<A> for $(&self.service_name)Factory<R, A> $("{")
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
        let route_bytes = path_bytes(fn_name).0;

        quote_in! { self.tokens =>
            $(")") -> impl Activation<A> {
                RemotingAction::new(self.remoting.clone(), &[$route_bytes], $args)
            }
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
            pub trait $(&self.service_name)Factory<A> $("{")
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
            $(")") -> impl Activation<A>;
        };
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());

        quote_in! { self.tokens =>
            $(func_param.name()): $(type_decl_code),
        };
    }
}
