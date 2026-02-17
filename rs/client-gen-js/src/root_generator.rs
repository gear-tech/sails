use crate::{
    helpers::{doc_tokens, payload_type_expr, serialize_type},
    naming::{escape_ident, to_camel},
    service_generator::ServiceGenerator,
    type_generator::TypeGenerator,
};
use genco::prelude::*;
use js::Tokens;
use sails_idl_parser_v2::ast;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct RootGenerator<'a> {
    type_gen: &'a TypeGenerator<'a>,
}

impl<'a> RootGenerator<'a> {
    pub(crate) fn new(type_gen: &'a TypeGenerator<'a>) -> Self {
        Self { type_gen }
    }

    pub(crate) fn generate(&mut self, doc: &ast::IdlDoc) -> String {
        let mut tokens = Tokens::new();
        quote_in! { tokens =>
            /* eslint-disable */
        };
        tokens.push();

        let mut service_index: BTreeMap<u64, &ast::ServiceUnit> = BTreeMap::new();
        let mut service_name_index: BTreeMap<&str, &ast::ServiceUnit> = BTreeMap::new();
        for service in &doc.services {
            service_name_index.insert(service.name.name.as_str(), service);
            if let Some(interface_id) = service.name.interface_id {
                service_index.insert(interface_id.as_u64(), service);
            }
        }

        if let Some(program) = &doc.program {
            self.type_gen.render_all(&mut tokens, &program.types);
            if !program.types.is_empty() {
                tokens.push();
            }

            let gear_api = &js::import("@gear-js/api", "GearApi");
            let hex_string = &js::import("@gear-js/api", "HexString");
            let program_id_ts = "`0x${string}`".to_string();
            let type_resolver = &js::import("sails-js", "TypeResolver");
            let tx_builder = &js::import("sails-js", "TransactionBuilderWithHeader");
            let interface_id_type = &js::import("sails-js-parser-idl-v2", "InterfaceId");
            let message_header = &js::import("sails-js-parser-idl-v2", "SailsMessageHeader");

            let mut all_types = Vec::new();
            for ty in &program.types {
                all_types.push(serialize_type(ty));
            }
            let resolver_types = format!("[{}]", all_types.join(", "));
            let program_class_name = if program.name.is_empty() {
                "SailsProgram".to_string()
            } else {
                program.name.clone()
            };

            let service_getters = program.services.iter().map(|service_expo| {
                let class_name = service_expo.name.name.clone();
                let class_ctor_name = class_name.clone();
                let getter_name = escape_ident(&to_camel(&service_expo.name.name));
                let route_idx = service_expo.route_idx;

                quote! {
                    public get $(getter_name)(): $(class_name) {
                      return new $(class_ctor_name)(this.api, this.programId, $(route_idx));
                    }
                }
            });

            let ctor_methods = program
                .ctors
                .iter()
                .enumerate()
                .flat_map(|(entry_id, ctor)| {
                    render_ctor_methods(
                        self.type_gen,
                        ctor,
                        entry_id as u16,
                        tx_builder,
                        message_header,
                        interface_id_type,
                    )
                });

            quote_in! { tokens =>
                export class $(program_class_name) {
                  private _typeResolver: $type_resolver;
                  constructor(
                    public api: $gear_api,
                    private _programId?: $(program_id_ts),
                  ) {
                    this._typeResolver = new $type_resolver($(resolver_types));
                  }

                  private get registry() {
                    return this._typeResolver.registry;
                  }

                  public get programId(): $hex_string {
                    if (!this._programId) throw new Error("Program ID is not set");
                    return this._programId;
                  }

                  $(for ctor_method in ctor_methods => $ctor_method$['\n'])

                  $(for getter in service_getters => $getter$['\n'])
                }
            };
            tokens.push();
        }

        if let Some(program) = &doc.program {
            let service_gen = ServiceGenerator::new(self.type_gen);
            let mut rendered = BTreeSet::new();
            for service_expo in &program.services {
                let Some(interface_id) = service_expo.name.interface_id else {
                    continue;
                };
                if let Some(service) = service_index.get(&interface_id.as_u64()) {
                    render_service_recursive(
                        &mut tokens,
                        &service_gen,
                        &service_index,
                        &service_name_index,
                        service_expo,
                        service,
                        &mut rendered,
                    );
                }
            }
        }

        tokens.to_file_string().unwrap_or_default()
    }
}

fn render_service_recursive<'a>(
    tokens: &mut Tokens,
    service_gen: &ServiceGenerator<'a>,
    service_index: &BTreeMap<u64, &'a ast::ServiceUnit>,
    service_name_index: &BTreeMap<&str, &'a ast::ServiceUnit>,
    service_expo: &ast::ServiceExpo,
    service: &'a ast::ServiceUnit,
    rendered: &mut BTreeSet<String>,
) {
    let current_name = service.name.name.clone();

    if rendered.contains(&current_name) {
        return;
    }

    for base in &service.extends {
        let base_service = if let Some(base_id) = base.interface_id.map(|id| id.as_u64()) {
            service_index.get(&base_id).copied()
        } else {
            service_name_index.get(base.name.as_str()).copied()
        };

        let Some(base_service) = base_service else {
            continue;
        };

        let base_expo = ast::ServiceExpo {
            name: base.clone(),
            route: None,
            route_idx: service_expo.route_idx,
            docs: vec![],
            annotations: vec![],
        };

        render_service_recursive(
            tokens,
            service_gen,
            service_index,
            service_name_index,
            &base_expo,
            base_service,
            rendered,
        );
    }

    service_gen.render(tokens, service_expo, service);
    tokens.push();
    rendered.insert(current_name);
}

fn render_ctor_methods(
    type_gen: &TypeGenerator<'_>,
    ctor: &ast::CtorFunc,
    entry_id: u16,
    tx_builder: &js::Import,
    message_header: &js::Import,
    interface_id_type: &js::Import,
) -> Vec<Tokens> {
    let base_name = format!("{}Ctor", to_camel(&ctor.name));
    let from_code_name = escape_ident(&format!("{base_name}FromCode"));
    let from_code_id_name = escape_ident(&format!("{base_name}FromCodeId"));

    let args_sig = ctor
        .params
        .iter()
                .map(|p| {
                    format!(
                        "{}: {}",
                        escape_ident(&p.name),
                        type_gen.ts_type_decl(&p.type_decl)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");

    let code_arg = "code: Uint8Array | Buffer | HexString".to_string();
    let code_id_arg = "codeId: `0x${string}`".to_string();
    let from_code_sig = if args_sig.is_empty() {
        code_arg
    } else {
        format!("{code_arg}, {args_sig}")
    };
    let from_code_id_sig = if args_sig.is_empty() {
        code_id_arg
    } else {
        format!("{code_id_arg}, {args_sig}")
    };

    let payload_type = payload_type_expr(&ctor.params, "this._typeResolver");

    let params_expr = if ctor.params.is_empty() {
        "null".to_string()
    } else if ctor.params.len() == 1 {
        escape_ident(&ctor.params[0].name)
    } else {
        format!(
            "[{}]",
            ctor.params
                .iter()
                .map(|p| escape_ident(&p.name))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let docs = doc_tokens(&ctor.docs);
    let docs_for_second = docs.clone();
    let params_expr_for_second = params_expr.clone();
    let payload_type_for_second = payload_type.clone();

    let from_code = quote! {
        $docs
        public $(from_code_name)($(from_code_sig)): $tx_builder<null> {
          const builder = new $tx_builder<null>(
            this.api,
            this.registry,
            "upload_program",
            $message_header.v1($interface_id_type.zero(), $(entry_id), 0),
            $(params_expr),
            $(payload_type),
            this._typeResolver.getTypeDeclString("String"),
            code,
          );
          this._programId = builder.programId;
          return builder;
        }
    };

    let from_code_id = quote! {
        $docs_for_second
        public $(from_code_id_name)($(from_code_id_sig)): $tx_builder<null> {
          const builder = new $tx_builder<null>(
            this.api,
            this.registry,
            "create_program",
            $message_header.v1($interface_id_type.zero(), $(entry_id), 0),
            $(params_expr_for_second),
            $(payload_type_for_second),
            this._typeResolver.getTypeDeclString("String"),
            codeId,
          );
          this._programId = builder.programId;
          return builder;
        }
    };

    vec![from_code, from_code_id]
}
