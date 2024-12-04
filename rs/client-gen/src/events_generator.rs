use crate::helpers::{method_bytes, path_bytes};
use genco::prelude::*;
use sails_idl_parser::{ast::visitor, ast::visitor::Visitor, ast::*};

pub(crate) struct EventsModuleGenerator<'a> {
    service_name: &'a str,
    path: &'a str,
    sails_path: &'a str,
    tokens: rust::Tokens,
}

impl<'a> EventsModuleGenerator<'a> {
    pub(crate) fn new(service_name: &'a str, path: &'a str, sails_path: &'a str) -> Self {
        Self {
            service_name,
            path,
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
        let events_name = format!("{}Events", self.service_name);
        let (service_path_bytes, _) = path_bytes(self.path);
        let event_names_bytes = service
            .events()
            .iter()
            .map(|e| method_bytes(e.name()).0)
            .collect::<Vec<_>>()
            .join("], &[");

        quote_in! { self.tokens =>
            $['\n']
            #[allow(dead_code)]
            #[cfg(not(target_arch = "wasm32"))]
            pub mod events $("{")
                use super::*;
                use $(self.sails_path)::events::*;
                #[derive(PartialEq, Debug, Encode, Decode)]
                #[codec(crate = $(self.sails_path)::scale_codec)]
                pub enum $(&events_name) $("{")
        };

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $['\r'] $("}")
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
