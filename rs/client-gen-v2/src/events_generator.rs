use genco::prelude::*;
use sails_idl_parser_v2::{ast, visitor, visitor::Visitor};

use crate::helpers::generate_doc_comments;

pub(crate) struct EventsModuleGenerator<'ast> {
    service_name: &'ast str,
    sails_path: &'ast str,
    tokens: rust::Tokens,
}

impl<'ast> EventsModuleGenerator<'ast> {
    pub(crate) fn new(service_name: &'ast str, sails_path: &'ast str) -> Self {
        Self {
            service_name,
            sails_path,
            tokens: rust::Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for EventsModuleGenerator<'ast> {
    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        let events_name = &format!("{}Events", self.service_name);

        quote_in! { self.tokens =>
            $['\n']
            #[cfg(not(target_arch = "wasm32"))]
            pub mod events $("{")
                use super::*;
                #[$(self.sails_path)::sails_type(crate = $(self.sails_path))]
                #[derive(PartialEq, Debug)]
                pub enum $events_name $("{")
        };

        visitor::accept_service_unit(service, self);

        quote_in! { self.tokens =>
            $['\r'] $("}")

            $['\r']
            impl $events_name {
                pub fn entry_id(&self) -> u16 {
                    match self {
                        $(for event in &service.events join ($['\r']) =>
                            Self::$(&event.name) { .. } => $(event.entry_id),
                        )
                    }
                }
            }

            impl $(self.sails_path)::client::Event for $events_name {
                fn decode_event(
                    route: &$(self.sails_path)::client::RouteIdx,
                    payload: impl AsRef<[u8]>,
                ) -> Result<Self, $(self.sails_path)::scale_codec::Error> {
                    $(self.sails_path)::client::decode_event_v2::<Self>(route.0, payload)
                }
            }

            impl $(self.sails_path)::client::Identifiable for $events_name {
                const INTERFACE_ID: $(self.sails_path)::InterfaceId = <$(self.service_name)Impl as $(self.sails_path)::client::Identifiable>::INTERFACE_ID;
            }

            impl $(self.sails_path)::client::ServiceWithEvents for $(self.service_name)Impl {
                type Event = $events_name;
            }
        }

        quote_in! { self.tokens =>
            $['\r'] $("}")
        };
    }

    fn visit_service_event(&mut self, event: &'ast ast::ServiceEvent) {
        generate_doc_comments(&mut self.tokens, &event.docs);

        let variant_name = &event.name;
        let entry_id = event.entry_id;

        if event.def.is_unit() {
            quote_in! { self.tokens =>
                $['\r']
                #[codec(index = $entry_id)]
                $variant_name,
            };
        } else if event.def.is_tuple() {
            let mut field_tokens = rust::Tokens::new();
            for field in &event.def.fields {
                let type_code =
                    crate::type_generators::generate_type_decl_with_path(&field.type_decl, "");
                field_tokens.append(type_code);
                field_tokens.append(", ");
            }
            quote_in! { self.tokens =>
                $['\r']
                #[codec(index = $entry_id)]
                $variant_name($field_tokens),
            };
        } else {
            let mut field_tokens = rust::Tokens::new();
            for field in &event.def.fields {
                generate_doc_comments(&mut field_tokens, &field.docs);
                let field_name = field.name.as_ref().unwrap();
                let type_code =
                    crate::type_generators::generate_type_decl_with_path(&field.type_decl, "");
                quote_in! { field_tokens =>
                    $['\r'] $field_name: $type_code,
                };
            }
            quote_in! { self.tokens =>
                $['\r']
                #[codec(index = $entry_id)]
                $variant_name {
                    $(field_tokens)
                $['\r'] },
            };
        }
    }
}
