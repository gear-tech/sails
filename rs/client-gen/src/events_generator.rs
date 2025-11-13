use genco::prelude::*;
use sails_idl_parser_v2::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct EventsModuleGenerator<'a> {
    service_name: &'a str,
    sails_path: &'a str,
    tokens: rust::Tokens,
}

impl<'a> EventsModuleGenerator<'a> {
    pub(crate) fn new(service_name: &'a str, sails_path: &'a str) -> Self {
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

impl<'ast> Visitor<'ast> for EventsModuleGenerator<'_> {
    fn visit_service_unit(&mut self, service: &'ast ServiceUnit) {
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

    fn visit_enum_variant(&mut self, event: &'ast EnumVariant) {
        // In the new AST, event is just an enum variant.
        // We need to figure out if it has a payload.
        let has_payload = !event.def.fields.is_empty();

        for doc in &event.docs {
            quote_in! { self.tokens =>
                $['\r'] $("///") $doc
            };
        }

        if has_payload {
            // Assuming the payload is a single unnamed type for simplicity,
            // which was the case for service events in v1.
            // This might need adjustment if events can have complex struct-like payloads.
            if let Some(field) = event.def.fields.first() {
                let type_decl_code =
                    crate::type_generators::generate_type_decl_code(&field.type_decl);
                if type_decl_code.starts_with('{') {
                    quote_in! { self.tokens =>
                        $['\r'] $(&event.name) $(type_decl_code),
                    };
                } else {
                    quote_in! { self.tokens =>
                        $['\r'] $(&event.name)($(type_decl_code)) ,
                    };
                }
            }
        } else {
            quote_in! { self.tokens =>
                $['\r'] $(&event.name),
            };
        }
    }
}
