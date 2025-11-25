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
        let event_names = service
            .events
            .iter()
            .map(|e| format!("\"{}\"", e.name))
            .collect::<Vec<_>>()
            .join(", ");

        quote_in! { self.tokens =>
            $['\n']
            #[cfg(not(target_arch = "wasm32"))]
            pub mod events $("{")
                use super::*;
                #[derive(PartialEq, Debug, Encode, Decode)]
                #[codec(crate = $(self.sails_path)::scale_codec)]
                pub enum $events_name $("{")
        };

        visitor::accept_service_unit(service, self);

        quote_in! { self.tokens =>
            $['\r'] $("}")
        };

        quote_in! { self.tokens =>
            impl $(self.sails_path)::client::Event for $events_name {
                const EVENT_NAMES: &'static [Route] = &[$event_names];
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

        if event.def.is_unit() {
            // Unit variant: `Variant,`
            quote_in! { self.tokens =>
                $['\r'] $variant_name,
            };
        } else if event.def.is_tuple() {
            // Tuple variant: `Variant(Type1, Type2),`
            let mut field_tokens = rust::Tokens::new();
            for field in &event.def.fields {
                let type_code =
                    crate::type_generators::generate_type_decl_with_path(&field.type_decl, "");
                field_tokens.append(type_code);
                field_tokens.append(", ");
            }
            quote_in! { self.tokens =>
                $['\r'] $variant_name($field_tokens),
            };
        } else {
            // Struct variant: `Variant { field1: Type1, ... },`
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
                $['\r'] $variant_name {
                    $(field_tokens)
                $['\r'] },
            };
        }
    }
}
