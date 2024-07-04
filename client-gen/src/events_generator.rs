use convert_case::{Case, Casing};
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
        let (service_path_bytes, service_path_length) = path_bytes(&self.path);
        let event_names_bytes = service
            .events()
            .iter()
            .map(|e| method_bytes(e.name()).0)
            .collect::<Vec<_>>()
            .join("], &[");

        quote_in! { self.tokens =>
            #[cfg(not(target_arch = "wasm32"))]
            pub mod events $("{")
                use super::*;
                use sails_rtl::event_listener::{EventSubscriber, RemotingSubscribe, Subscribe};
                #[derive(PartialEq, Debug, Encode, Decode)]
                #[codec(crate = sails_rtl::scale_codec)]
                pub enum $(&events_name) $("{")
        };

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $("}")
        };

        quote_in! { self.tokens =>
            const SERVICE_ROUTE: &[u8] = &[$service_path_bytes];
            const EVENT_NAMES: &[&[u8]] = &[&[$event_names_bytes]];

            #[derive(Clone)]
            pub struct Listener<R, A>
            where
                R: Remoting<A> + Clone + EventSubscriber,
                A: Default,
            {
                remoting: R,
                _phantom: PhantomData<A>,
            }

            impl<A: Default, R: Remoting<A> + Clone + EventSubscriber> Listener<R, A> {
                pub fn new(remoting: R) -> Self {
                    Self {
                        remoting,
                        _phantom: PhantomData,
                    }
                }
            }

            impl<R, A> traits::$(&self.service_name)Listener for Listener<R, A>
            where
                R: Remoting<A> + Clone + EventSubscriber,
                A: Default,
            {
                fn listener(self) -> impl Subscribe<$(&events_name)> {
                    RemotingSubscribe::new(
                        self.remoting,
                        SERVICE_ROUTE,
                        EVENT_NAMES,
                    )
                }
            }

            #[allow(dead_code)]
            pub fn decode_event(payload: &[u8]) -> Result<$(&events_name), sails_rtl::errors::Error> {
                if !payload.starts_with(SERVICE_ROUTE) {
                    Err(sails_rtl::errors::RtlError::EventPrefixMismatches)?;
                }
                let event_bytes = &payload[$(service_path_length)..];
                for (idx, name) in EVENT_NAMES.iter().enumerate() {
                    if event_bytes.starts_with(name) {
                        let idx = idx as u8;
                        let bytes = [&[idx], &event_bytes[name.len()..]].concat();
                        let mut event_bytes = &bytes[..];
                        return Ok($(&events_name)::decode(&mut event_bytes)?);
                    }
                }
                Err(sails_rtl::errors::RtlError::EventNameIsNotFound)?
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

pub(crate) struct EventsTraitGenerator {
    service_name: String,
}

impl EventsTraitGenerator {
    pub(crate) fn new(service_name: String) -> Self {
        Self { service_name }
    }

    pub(crate) fn finalize(self) -> rust::Tokens {
        let name = self.service_name.to_case(Case::Snake);
        quote! {
            #[cfg(not(target_arch = "wasm32"))]
            pub trait $(&self.service_name)Listener {
                fn listener(self) -> impl sails_rtl::event_listener::Subscribe<$(name)::events::$(&self.service_name)Events>;
            }
        }
    }
}
