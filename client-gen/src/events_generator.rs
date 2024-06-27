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
        let (service_path_bytes, _) = path_bytes(&self.path);
        let event_names_bytes = service
            .events()
            .iter()
            .map(|e| method_bytes(e.name()).0)
            .collect::<Vec<_>>()
            .join("], &[");

        quote_in! { self.tokens =>
            pub mod events $("{")
                use super::*;
                #[derive(PartialEq, Debug, Encode, Decode)]
                #[codec(crate = sails_rtl::scale_codec)]
                pub enum $(&self.service_name)Events $("{")
        };

        visitor::accept_service(service, self);

        quote_in! { self.tokens =>
            $("}")
        };

        quote_in! { self.tokens =>
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
                pub fn new(remoting: &R) -> Self {
                    Self {
                        remoting: remoting.clone(),
                        _phantom: PhantomData,
                    }
                }
            }

            impl<R, A> traits::$(&self.service_name)Listener for Listener<R, A>
            where
                R: Remoting<A> + Clone + EventSubscriber,
                A: Default,
            {
                fn listener(self) -> impl Subscribe<$(&self.service_name)Events> {
                    RemotingSubscribe::new(
                        self.remoting,
                        &[$service_path_bytes],
                        &[&[$event_names_bytes]],
                    )
                }
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
            pub trait $(&self.service_name)Listener {
                fn listener(self) -> impl Subscribe<$(name)::events::$(&self.service_name)Events>;
            }
        }
    }
}
