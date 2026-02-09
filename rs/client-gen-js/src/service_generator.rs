use crate::{
    helpers::{doc_tokens, payload_type_expr, serialize_type, serialize_type_decl, ts_type_decl},
    naming::{escape_ident, to_camel},
    type_generator::TypeGenerator,
};
use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;

pub(crate) struct ServiceGenerator<'a> {
    type_gen: &'a TypeGenerator<'a>,
}

impl<'a> ServiceGenerator<'a> {
    pub(crate) fn new(type_gen: &'a TypeGenerator<'a>) -> Self {
        Self { type_gen }
    }

    pub(crate) fn render(
        &self,
        tokens: &mut Tokens,
        service_expo: &ast::ServiceExpo,
        service: &ast::ServiceUnit,
    ) {
        let class_name = format!("{}", service_expo.name.name);
        let interface_id = service
            .name
            .interface_id
            .expect("Service must have interface_id")
            .to_string();

        for ty in &service.types {
            self.type_gen.render_type(tokens, ty);
            tokens.push();
        }

        let mut all_types = Vec::new();
        for ty in &service.types {
            all_types.push(serialize_type(ty));
        }
        let resolver_types = format!("[{}]", all_types.join(", "));

        let gear_api = &js::import("@gear-js/api", "GearApi");
        let hex_string = &js::import("@gear-js/api", "HexString");
        let type_resolver = &js::import("sails-js", "TypeResolver");

        let func_tokens = service
            .funcs
            .iter()
            .enumerate()
            .map(|(entry_id, func)| self.render_func(func, entry_id as u16, &interface_id));

        quote_in! { *tokens =>
            export class $(class_name) {
              private _typeResolver: $type_resolver;
              constructor(
                private _api: $gear_api,
                private _programId: $hex_string,
                private _routeIdx: number = 0,
              ) {
                this._typeResolver = new $type_resolver($(resolver_types));
              }
              private get registry() {
                return this._typeResolver.registry;
              }
              $(for func in func_tokens => $func$['\n'])
            }
        };
    }

    fn render_func(&self, func: &ast::ServiceFunc, entry_id: u16, interface_id: &str) -> Tokens {
        let method_name = escape_ident(&to_camel(&func.name));

        let args = func
            .params
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    escape_ident(&p.name),
                    ts_type_decl(&p.type_decl)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let return_type = if let Some(throws) = &func.throws {
            format!(
                "{{ ok: {} }} | {{ err: {} }}",
                ts_type_decl(&func.output),
                ts_type_decl(throws)
            )
        } else {
            ts_type_decl(&func.output)
        };

        let payload_type = payload_type_expr(&func.params, "this._typeResolver");

        let return_type_scale = format!(
            "this._typeResolver.getTypeDeclString({})",
            serialize_type_decl(&func.output)
        );

        let params_expr = if func.params.is_empty() {
            "null".to_string()
        } else if func.params.len() == 1 {
            escape_ident(&func.params[0].name)
        } else {
            format!(
                "[{}]",
                func.params
                    .iter()
                    .map(|p| escape_ident(&p.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let doc_tokens = doc_tokens(&func.docs);

        let query_builder = &js::import("sails-js", "QueryBuilderWithHeader");
        let tx_builder = &js::import("sails-js", "TransactionBuilderWithHeader");
        let interface_id_type = &js::import("sails-js-parser-idl-v2", "InterfaceId");
        let message_header = &js::import("sails-js-parser-idl-v2", "SailsMessageHeader");

        match func.kind {
            ast::FunctionKind::Query => {
                quote! {
                    $doc_tokens
                    public $(method_name)($(args)): $query_builder<$(&return_type)> {
                      return new $query_builder<$(&return_type)>(
                        this._api,
                        this.registry,
                        this._programId,
                        $message_header.v1($interface_id_type.from($(quoted(interface_id))), $(entry_id), this._routeIdx),
                        $(params_expr),
                        $(payload_type),
                        $(return_type_scale),
                      );
                    }
                }
            }
            ast::FunctionKind::Command => {
                quote! {
                    $doc_tokens
                    public $(method_name)($(args)): $tx_builder<$(&return_type)> {
                      return new $tx_builder<$(&return_type)>(
                        this._api,
                        this.registry,
                        $(quoted("send_message")),
                        $message_header.v1($interface_id_type.from($(quoted(interface_id))), $(entry_id), this._routeIdx),
                        $(params_expr),
                        $(payload_type),
                        $(return_type_scale),
                        this._programId,
                      );
                    }
                }
            }
        }
    }

}
