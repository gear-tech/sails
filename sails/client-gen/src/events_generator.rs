use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

use crate::helpers::{method_bytes, path_bytes};

pub(crate) struct EventsModuleGenerator {
    service_name: String,
    path: String,
    tokens: rust::Tokens,
}

impl EventsModuleGenerator {
    pub(crate) fn new(service_name: String, path: String) -> Self {
        Self {
            service_name,
            path,
            tokens: rust::Tokens::new(),
        }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        self.tokens
    }
}

impl<'ast> Visitor<'ast> for EventsModuleGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        let events_name = format!("{}Events", self.service_name);
        let (service_path_bytes, _) = path_bytes(&self.path);
        let event_names_bytes = service
            .events()
            .iter()
            .map(|e| method_bytes(e.name()).0)
            .collect::<Vec<_>>()
            .join("], &[");

        quote_in! { self.tokens =>
            #[allow(dead_code)]
            #[cfg(not(target_arch = "wasm32"))]
            pub mod events $("{")
                use super::*;
                use sails_rs::events::*;
                #[derive(PartialEq, Debug, Encode, Decode)]
                #[codec(crate = sails_rs::scale_codec)]
                pub enum $(&events_name) $("{")
        };

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $("}")
        };

        quote_in! { self.tokens =>
            impl EventIo for $(&events_name) {
                const ROUTE: &'static [u8] = &[$service_path_bytes];
                const EVENT_NAMES: &'static [&'static [u8]] = &[&[$event_names_bytes]];
                type Event = Self;
            }

            pub fn listener<R: Listener<Vec<u8>>>(remoting: R) -> impl Listener<$(&events_name)> {
                RemotingListener::<_, $(&events_name)>::new(remoting)
            }
        }

        quote_in! { self.tokens =>
            $("}")
        };
    }

    fn visit_service_event(&mut self, event: &'ast ServiceEvent) {
        if let Some(type_decl) = event.type_decl().as_ref() {
            let type_decl_code = crate::type_generators::generate_type_decl_code(type_decl);
            if type_decl_code.starts_with('{') {
                quote_in! { self.tokens =>
                    $(event.name()) $(type_decl_code),
                };
            } else {
                quote_in! { self.tokens =>
                    $(event.name())($(type_decl_code)) ,
                };
            }
        } else {
            quote_in! { self.tokens =>
                $(event.name()),
            };
        }
    }
}
