use crate::{
    helpers::{doc_tokens, payload_type_expr, serialize_type, serialize_type_decl},
    naming::{escape_ident, to_camel},
    type_generator::TypeGenerator,
};
use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;

pub(crate) struct ServiceGenerator<'a> {
    type_gen: &'a TypeGenerator,
}

impl<'a> ServiceGenerator<'a> {
    pub(crate) fn new(type_gen: &'a TypeGenerator) -> Self {
        Self { type_gen }
    }

    pub(crate) fn render(
        &self,
        tokens: &mut Tokens,
        service_expo: &ast::ServiceExpo,
        service: &ast::ServiceUnit,
    ) {
        let class_name = service_expo.name.name.to_string();
        let interface_id = service
            .name
            .interface_id
            .expect("Service must have interface_id")
            .to_string();

        self.type_gen.render_all(tokens, &service.types);

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
            .map(|(entry_id, func)| self.render_func(func, entry_id as u16));

        let event_tokens = service
            .events
            .iter()
            .enumerate()
            .map(|(entry_id, event)| self.render_event(event, entry_id as u16));

        let extend_tokens = service.extends.iter().map(|base| {
            let base_class_name = base.name.clone();
            let accessor_name = escape_ident(&to_camel(&base.name));

            quote! {
                public get $(accessor_name)(): $(&base_class_name) {
                  return new $(&base_class_name)(this._api, this._programId, this._routeIdx);
                }
            }
        });
        let interface_id_type = &js::import("sails-js-parser-idl-v2", "InterfaceId");

        quote_in! { *tokens =>
            export class $class_name {
              private _typeResolver: $type_resolver;
              constructor(
                private _api: $gear_api,
                private _programId: $hex_string,
                private _routeIdx: number = 0,
              ) {
                this._typeResolver = new $type_resolver($resolver_types);
              }
              private get registry() {
                return this._typeResolver.registry;
              }
              public get interfaceId(): $interface_id_type {
                return $interface_id_type.from($(quoted(&interface_id)));
              }
              $(for ext in extend_tokens => $ext$['\n'])
              $(for func in func_tokens => $func$['\n'])
              $(for event in event_tokens => $event$['\n'])
            }
        };
    }

    fn render_func(&self, func: &ast::ServiceFunc, entry_id: u16) -> Tokens {
        let method_name = escape_ident(&to_camel(&func.name));

        let args = func.params.iter().map(|p| {
            let ident = escape_ident(&p.name);
            let ty = self.type_gen.ts_type_decl(&p.type_decl);
            quote!($(ident): $(ty))
        });

        let return_type = if let Some(throws) = &func.throws {
            let ok = self.type_gen.ts_type_decl(&func.output);
            let err = self.type_gen.ts_type_decl(throws);
            quote!({ ok: $(ok) } | { err: $(err) })
        } else {
            self.type_gen.ts_type_decl(&func.output)
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
        let message_header = &js::import("sails-js-parser-idl-v2", "SailsMessageHeader");

        match func.kind {
            ast::FunctionKind::Query => {
                quote! {
                    $doc_tokens
                    public $method_name($(for arg in args join (, ) => $arg)): $query_builder<$(&return_type)> {
                      return new $query_builder<$(&return_type)>(
                        this._api,
                        this.registry,
                        this._programId,
                        $message_header.v1(this.interfaceId, $(entry_id), this._routeIdx),
                        $params_expr,
                        $payload_type,
                        $return_type_scale,
                      );
                    }
                }
            }
            ast::FunctionKind::Command => {
                quote! {
                    $doc_tokens
                    public $method_name($(for arg in args join (, ) => $arg)): $tx_builder<$(&return_type)> {
                      return new $tx_builder<$(&return_type)>(
                        this._api,
                        this.registry,
                        $(quoted("send_message")),
                        $message_header.v1(this.interfaceId, $(entry_id), this._routeIdx),
                        $params_expr,
                        $payload_type,
                        $return_type_scale,
                        this._programId,
                      );
                    }
                }
            }
        }
    }

    fn render_event(&self, event: &ast::ServiceEvent, entry_id: u16) -> Tokens {
        let method_name = escape_ident(&format!("subscribeTo{}Event", event.name));
        let event_ts_type = self.type_gen.ts_struct_def_tokens(&event.def);
        let type_str = "`([u8; 16], ${typeStr})`";

        let zero_address = &js::import("sails-js", "ZERO_ADDRESS");
        let message_header = &js::import("sails-js-parser-idl-v2", "SailsMessageHeader");
        let struct_field = &js::import("sails-js-types", "IStructField");
        let doc_tokens = doc_tokens(&event.docs);

        quote! {
            $doc_tokens
            public $(method_name)<T = $(&event_ts_type)>(callback: (eventData: T) => void | Promise<void>): Promise<() => void> {
              const interfaceIdu64 = this.interfaceId.asU64();
              const eventFields = $(event.def.to_json_string().expect("StructDef should be serializable to JSON")).fields as $struct_field[];
              const typeStr = this._typeResolver.getStructDef(eventFields, {}, true);
              return this._api.gearEvents.subscribeToGearEvent("UserMessageSent", ({ data: { message } }) => {
                if (!message.source.eq(this._programId)) return;
                if (!message.destination.eq($zero_address)) return;

                const { ok, header } = $message_header.tryFromBytes(message.payload);
                if (ok && header.interfaceId.asU64() === interfaceIdu64 && header.entryId === $(entry_id)) {
                  callback(this.registry.createType($type_str, message.payload)[1].toJSON() as T);
                }
              });
            }
        }
    }
}
