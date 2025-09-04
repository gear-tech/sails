use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

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
    fn visit_service(&mut self, service: &'ast Service) {
        let events_name = &format!("{}Events", self.service_name);
        let event_names = service
            .events()
            .iter()
            .map(|e| format!("\"{}\"", e.name()))
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

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $['\r'] $("}")
        };

        quote_in! { self.tokens =>
            impl EventDecode for $events_name {
                const EVENT_NAMES: &'static [Route] = &[$event_names];
            }

            impl ServiceEvent for $(self.service_name)Impl {
                type Event = $events_name;
            }
        }

        quote_in! { self.tokens =>
            $['\r'] $("}")
        };
    }

    fn visit_service_event(&mut self, event: &'ast ServiceEvent) {
        if let Some(type_decl) = event.type_decl().as_ref() {
            for doc in event.docs() {
                quote_in! { self.tokens =>
                    $['\r'] $("///") $doc
                };
            }

            let type_decl_code = crate::type_generators::generate_type_decl_code(type_decl);
            if type_decl_code.starts_with('{') {
                quote_in! { self.tokens =>
                    $['\r'] $(event.name()) $(type_decl_code),
                };
            } else {
                quote_in! { self.tokens =>
                    $['\r'] $(event.name())($(type_decl_code)) ,
                };
            }
        } else {
            quote_in! { self.tokens =>
                $['\r'] $(event.name()),
            };
        }
    }
}
