use crate::helpers::*;
use crate::type_generators::generate_type_decl_with_path;
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct IoModuleGenerator<'a> {
    path: &'a str,
    sails_path: &'a str,
    tokens: rust::Tokens,
}

impl<'a> IoModuleGenerator<'a> {
    pub(crate) fn new(path: &'a str, sails_path: &'a str) -> Self {
        Self {
            path,
            sails_path,
            tokens: rust::Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        quote!(
            pub mod io {
                use super::*;
                use $(self.sails_path)::calls::ActionIo;
                $(self.tokens)
            }
        )
    }
}

impl<'a, 'ast> Visitor<'ast> for IoModuleGenerator<'a> {
    fn visit_service(&mut self, service: &'ast Service) {
        visitor::accept_service(service, self);
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let fn_name = func.name();
        let (service_path_bytes, _) = path_bytes(self.path);
        let (route_bytes, _) = method_bytes(fn_name);

        let struct_tokens = generate_io_struct(
            fn_name,
            func.params(),
            Some(func.output()),
            format!("{service_path_bytes}{route_bytes}").as_str(),
        );

        quote_in! { self.tokens =>
            $struct_tokens
        };
    }
}

pub(crate) fn generate_io_struct(
    fn_name: &str,
    fn_params: &[FuncParam],
    fn_output: Option<&TypeDecl>,
    route_bytes: &str,
) -> rust::Tokens {
    let params_len = fn_params.len();
    let mut struct_param_tokens = rust::Tokens::new();
    let mut encode_call_args = rust::Tokens::new();
    let mut encode_call_names = rust::Tokens::new();
    for func_param in fn_params {
        let type_decl_code =
            generate_type_decl_with_path(func_param.type_decl(), "super".to_owned());
        quote_in! { struct_param_tokens =>
            $(&type_decl_code),
        };
        quote_in! { encode_call_args =>
            $(func_param.name()): $(&type_decl_code),
        };
        quote_in! { encode_call_names =>
            $(func_param.name()),
        };
    }

    let param_tokens = match params_len {
        0 => quote!(()),
        1 => quote!($(generate_type_decl_with_path(fn_params[0].type_decl(), "super".to_owned()))),
        _ => quote!(($struct_param_tokens)),
    };

    let func_output = fn_output.map_or("()".to_owned(), |output| {
        generate_type_decl_with_path(output, "super".to_owned())
    });

    let encode_call_tokens = match params_len {
        0 => quote!(
            impl $fn_name {
                #[allow(dead_code)]
                pub fn encode_call() -> Vec<u8> {
                    <$fn_name as ActionIo>::encode_call(&())
                }
            }
        ),
        1 => quote!(
            impl $fn_name {
                #[allow(dead_code)]
                pub fn encode_call($encode_call_args) -> Vec<u8> {
                    <$fn_name as ActionIo>::encode_call(&$(fn_params[0].name()))
                }
            }
        ),
        _ => quote!(
            impl $fn_name {
                #[allow(dead_code)]
                pub fn encode_call($encode_call_args) -> Vec<u8> {
                    <$fn_name as ActionIo>::encode_call(&($encode_call_names))
                }
            }
        ),
    };

    quote! {
        pub struct $fn_name (());

        $encode_call_tokens

        impl ActionIo for $fn_name {
            const ROUTE: &'static [u8] = &[$route_bytes];
            type Params = $param_tokens;
            type Reply = $func_output;
        }
    }
}
