use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::helpers::*;
use crate::type_generators::generate_type_decl_with_path;

pub(crate) struct IoModuleGenerator {
    path: String,
    tokens: rust::Tokens,
}

impl IoModuleGenerator {
    pub(crate) fn new(path: String) -> Self {
        Self {
            path,
            tokens: rust::Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for IoModuleGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        quote_in! { self.tokens =>
            pub mod io $("{")
                use super::*;
        };

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $("}")
        };
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let fn_name = func.name();

        quote_in! { self.tokens =>
            #[derive(Debug, Default, Clone, Copy)]
            pub struct $fn_name(());

            impl $fn_name $("{")
                #[allow(dead_code)]
                pub fn encode_call $("(")
        };

        visitor::accept_service_func(func, self);

        quote_in! { self.tokens =>
            $(")") -> Vec<u8> $("{")
        };

        let (service_path_bytes, service_path_encoded_length) = path_bytes(&self.path);
        let (route_bytes, route_encoded_length) = method_bytes(fn_name);

        let path_len = service_path_encoded_length + route_encoded_length;

        let args = encoded_args(func.params());

        quote_in! { self.tokens =>
            let args = $args;
            let mut result = Vec::with_capacity($path_len + args.encoded_size());
            result.extend_from_slice(&[$service_path_bytes]);
            result.extend_from_slice(&[$route_bytes]);
            args.encode_to(&mut result);
            result

            $("}")
        };

        let mut decode_reply_gen = DecodeReplyGenerator::default();
        decode_reply_gen.visit_service_func(func);

        self.tokens.extend(decode_reply_gen.tokens);

        quote_in! { self.tokens =>
            $("}")
        };
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code =
            generate_type_decl_with_path(func_param.type_decl(), "super".to_owned());

        quote_in! { self.tokens =>
            $(func_param.name()): $(type_decl_code),
        };
    }
}

#[derive(Default)]
struct DecodeReplyGenerator {
    tokens: rust::Tokens,
    is_unit: bool,
}

impl<'ast> Visitor<'ast> for DecodeReplyGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        visitor::accept_service(service, self);
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        quote_in! { self.tokens =>
            #[allow(dead_code)]
            pub fn decode_reply(mut reply: &[u8]) -> Result<
        };

        visitor::accept_service_func(func, self);

        let allow_rule = self
            .is_unit
            .then_some("#[allow(clippy::let_unit_value)]")
            .unwrap_or_default();

        let (route_bytes, route_encoded_length) = method_bytes(func.name());

        quote_in! { self.tokens =>
            , sails_rtl::errors::Error> {
                if !reply.starts_with(&[$route_bytes]) {
                    return Err(sails_rtl::errors::Error::Rtl(sails_rtl::errors::RtlError::ReplyPrefixMismatches));
                }

                reply = &reply[$route_encoded_length..];

                $allow_rule
                let result = Decode::decode(&mut reply).map_err(sails_rtl::errors::Error::Codec)?;
                Ok(result)
            }
        };
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        let type_decl_code = generate_type_decl_with_path(func_output, "super".to_owned());
        if type_decl_code == "()" {
            self.is_unit = true;
        }

        self.tokens.append(type_decl_code);
    }
}
