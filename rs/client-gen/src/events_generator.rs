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

    fn visit_service_event(&mut self, event: &'ast ServiceEvent) {
        for doc in &event.docs {
            quote_in! { self.tokens =>
                $['\r'] $("///") $doc
            };
        }

        let variant_name = &event.name;

        if event.def.fields.is_empty() {
            // Unit variant: `Variant,`
            quote_in! { self.tokens =>
                $['\r'] $variant_name,
            };
            return;
        }

        let is_tuple = event.def.fields.iter().all(|f| f.name.is_none());
        let is_struct = event.def.fields.iter().all(|f| f.name.is_some());

        if !is_tuple && !is_struct {
            panic!(
                "Event variant '{}' has a mix of named and unnamed fields, which is not supported.",
                variant_name
            );
        }

        if is_tuple {
            // Tuple variant: `Variant(Type1, Type2),`
            let mut field_tokens = rust::Tokens::new();
            for (i, field) in event.def.fields.iter().enumerate() {
                if i > 0 {
                    field_tokens.append(", ");
                }
                let type_code =
                    crate::type_generators::generate_type_decl_with_path(&field.type_decl, "super".into());
                field_tokens.append(type_code);
            }
            quote_in! { self.tokens =>
                $['\r'] $variant_name($field_tokens),
            };
        } else {
            // Struct variant: `Variant { field1: Type1, ... },`
            let mut field_tokens = rust::Tokens::new();
            for field in &event.def.fields {
                for doc in &field.docs {
                    quote_in! { field_tokens =>
                        $['\r'] $("///") $doc
                    };
                }
                let field_name = field.name.as_ref().unwrap();
                let type_code =
                    crate::type_generators::generate_type_decl_with_path(&field.type_decl, "super".into());
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
