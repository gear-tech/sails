use super::*;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use sails_interface_id::canonical::{
    CanonicalDocument, CanonicalExtendedInterface, CanonicalFunction, CanonicalParam,
    CanonicalService, CanonicalType, FunctionKind,
};
use sails_interface_id::compute_ids_from_document;
use std::collections::BTreeMap;

impl ServiceBuilder<'_> {
    pub(super) fn meta_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();
        let meta_module_ident = &self.meta_module_ident;

        let base_services_meta = self.base_types.iter().map(|base_type| {
            let path_wo_lifetimes = shared::remove_lifetimes(base_type);
            quote! {
                #sails_path::meta::AnyServiceMeta::new::< #path_wo_lifetimes >
            }
        });
        let base_types = self.base_types;

        let has_async_handler = self
            .service_handlers
            .iter()
            .any(|fn_builder| fn_builder.is_async());

        let service_meta_asyncness = if has_async_handler {
            quote!(true)
        } else if self.base_types.is_empty() {
            quote!(false)
        } else {
            let base_asyncness = self.base_types.iter().map(|base_type| {
                let path_wo_lifetimes = shared::remove_lifetimes(base_type);
                quote! {
                    <#path_wo_lifetimes as #sails_path::meta::ServiceMeta>::ASYNC
                }
            });
            quote!(#( #base_asyncness )||*)
        };

        quote! {
            impl #generics #sails_path::meta::ServiceMeta for #service_type_path #service_type_constraints {
                type CommandsMeta = #meta_module_ident::CommandsMeta;
                type QueriesMeta = #meta_module_ident::QueriesMeta;
                type EventsMeta = #meta_module_ident::EventsMeta;
                const BASE_SERVICES: &'static [#sails_path::meta::AnyServiceMetaFn] = &[
                    #( #base_services_meta ),*
                ];
                const ASYNC: bool = #service_meta_asyncness ;
                const INTERFACE_PATH: &'static str = stringify!(#service_type_path);
                const INTERFACE_ID32: u32 = #meta_module_ident::INTERFACE_ID32;
                const INTERFACE_UID64: u64 = #meta_module_ident::INTERFACE_UID64;
                const EXTENDS: &'static [#sails_path::meta::ExtendedInterface] = #meta_module_ident::EXTENDS;

                fn command_opcodes() -> #sails_path::Vec<u16> {
                    let mut codes = #sails_path::Vec::new();
                    codes.extend(#meta_module_ident::COMMAND_OPCODES.iter().copied());
                    #( codes.extend(<#base_types as #sails_path::meta::ServiceMeta>::command_opcodes()); )*
                    codes
                }

                fn local_command_opcodes() -> &'static [u16] {
                    #meta_module_ident::COMMAND_OPCODES
                }

                fn query_opcodes() -> #sails_path::Vec<u16> {
                    let mut codes = #sails_path::Vec::new();
                    codes.extend(#meta_module_ident::QUERY_OPCODES.iter().copied());
                    #( codes.extend(<#base_types as #sails_path::meta::ServiceMeta>::query_opcodes()); )*
                    codes
                }

                fn local_query_opcodes() -> &'static [u16] {
                    #meta_module_ident::QUERY_OPCODES
                }

                fn event_codes() -> #sails_path::Vec<u16> {
                    let mut codes = #meta_module_ident::event_codes();
                    #( codes.extend(<#base_types as #sails_path::meta::ServiceMeta>::event_codes()); )*
                    codes
                }

                fn local_event_codes() -> #sails_path::Vec<u16> {
                    #meta_module_ident::event_codes()
                }

                fn canonical_service() -> &'static [u8] {
                    #meta_module_ident::canonical_service()
                }
            }
        }
    }

    pub(super) fn meta_module(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let scale_codec_path = &sails_paths::scale_codec_path(sails_path);
        let scale_info_path = &sails_paths::scale_info_path(sails_path);
        let meta_module_ident = &self.meta_module_ident;
        let service_type_path = self.type_path;

        let no_events_type = Path::from(Ident::new("NoEvents", Span::call_site()));
        let events_type = self.events_type.unwrap_or(&no_events_type);

        let invocation_params_structs = self
            .service_handlers
            .iter()
            .map(|fn_builder| fn_builder.params_struct(scale_codec_path, scale_info_path));
        let commands_meta_variants = self.service_handlers.iter().filter_map(|fn_builder| {
            (!fn_builder.is_query()).then_some(fn_builder.handler_meta_variant())
        });
        let queries_meta_variants = self.service_handlers.iter().filter_map(|fn_builder| {
            (fn_builder.is_query()).then_some(fn_builder.handler_meta_variant())
        });
        let command_opcode_literals = self
            .service_handlers
            .iter()
            .filter(|fn_builder| !fn_builder.is_query())
            .map(|fn_builder| fn_builder.opcode_literal());
        let query_opcode_literals = self
            .service_handlers
            .iter()
            .filter(|fn_builder| fn_builder.is_query())
            .map(|fn_builder| fn_builder.opcode_literal());
        let event_codes_fn = self.events_type.map_or_else(
            || {
                quote! {
                    pub fn event_codes() -> #sails_path::Vec<u16> {
                        #sails_path::Vec::new()
                    }
                }
            },
            |events_type| {
                quote! {
                    pub fn event_codes() -> #sails_path::Vec<u16> {
                        <#events_type as #sails_path::meta::EventCodeMeta>::event_codes()
                    }
                }
            },
        );

        let extends_entries = self
            .base_types
            .iter()
            .map(|base_type| {
                let name = base_type.to_token_stream().to_string();
                let name_lit = Literal::string(&name);
                quote! {
                    #sails_path::meta::ExtendedInterface {
                        name: #name_lit,
                        interface_id32: <#base_type as #sails_path::meta::ServiceMeta>::INTERFACE_ID32,
                        interface_uid64: <#base_type as #sails_path::meta::ServiceMeta>::INTERFACE_UID64,
                    }
                }
            })
            .collect::<Vec<_>>();

        let interface_path = self.type_path.to_token_stream().to_string();

        let canonical_extends = self
            .base_types
            .iter()
            .map(|base_type| CanonicalExtendedInterface {
                name: shared::remove_lifetimes(base_type)
                    .to_token_stream()
                    .to_string(),
                interface_id32: 0,
                interface_uid64: 0,
            })
            .collect::<Vec<_>>();

        let canonical_functions = self
            .service_handlers
            .iter()
            .map(|fn_builder| {
                let kind = if fn_builder.is_query() {
                    FunctionKind::Query
                } else {
                    FunctionKind::Command
                };
                let params = fn_builder
                    .params()
                    .map(|(ident, _)| CanonicalParam {
                        name: ident.to_string(),
                        ty: CanonicalType::Unit,
                    })
                    .collect::<Vec<_>>();
                let opcode = fn_builder
                    .opcode()
                    .expect("opcode assigned for exported method");
                CanonicalFunction {
                    kind,
                    name: fn_builder.route.clone(),
                    route: None,
                    params,
                    returns: CanonicalType::Unit,
                    message_id_override: Some(opcode),
                }
            })
            .collect::<Vec<_>>();

        let mut canonical_services = BTreeMap::new();
        canonical_services.insert(
            interface_path.clone(),
            CanonicalService {
                name: interface_path.clone(),
                extends: canonical_extends,
                functions: canonical_functions,
                events: Vec::new(),
            },
        );

        let canonical_document = CanonicalDocument {
            version: sails_interface_id::canonical::CANONICAL_VERSION.to_owned(),
            services: canonical_services,
            types: BTreeMap::new(),
        };

        let canonical_bytes = canonical_document
            .to_bytes()
            .expect("canonical document serialization should succeed");
        let (interface_id32, interface_uid64) = compute_ids_from_document(&canonical_document);
        let interface_id32_lit = Literal::u32_unsuffixed(interface_id32);
        let interface_uid64_lit = Literal::u64_unsuffixed(interface_uid64);
        let canonical_byte_literals = canonical_bytes
            .iter()
            .map(|byte| Literal::u8_unsuffixed(*byte))
            .collect::<Vec<_>>();

        let extends_builders = self.base_types.iter().map(|base_type| {
            let name = shared::remove_lifetimes(base_type)
                .to_token_stream()
                .to_string();
            let name_lit = Literal::string(&name);
            quote! {
                entries.push(#sails_path::meta::ExtendedInterface {
                    name: #name_lit,
                    interface_id32: <#base_type as #sails_path::meta::ServiceMeta>::INTERFACE_ID32,
                    interface_uid64: <#base_type as #sails_path::meta::ServiceMeta>::INTERFACE_UID64,
                });
            }
        });

        quote! {
            mod #meta_module_ident {
                use super::*;
                use #sails_path::{Decode, TypeInfo};
                use #sails_path::gstd::InvocationIo;

                #( #invocation_params_structs )*

                #[derive(TypeInfo)]
                #[scale_info(crate = #scale_info_path)]
                pub enum CommandsMeta {
                    #(#commands_meta_variants),*
                }

                #[derive(TypeInfo)]
                #[scale_info(crate = #scale_info_path)]
                pub enum QueriesMeta {
                    #(#queries_meta_variants),*
                }

                #[derive(TypeInfo)]
                #[scale_info(crate = #scale_info_path )]
                pub enum #no_events_type {}

                pub type EventsMeta = #events_type;

                pub const COMMAND_OPCODES: &[u16] = &[ #( #command_opcode_literals ),* ];
                pub const QUERY_OPCODES: &[u16] = &[ #( #query_opcode_literals ),* ];
                pub const INTERFACE_ID32: u32 = #interface_id32_lit;
                pub const INTERFACE_UID64: u64 = #interface_uid64_lit;
                pub const CANONICAL_BYTES: &[u8] = &[ #( #canonical_byte_literals ),* ];
                pub const EXTENDS: &[#sails_path::meta::ExtendedInterface] = &[ #( #extends_entries ),* ];

                #event_codes_fn

                impl #sails_path::meta::EventCodeMeta for #no_events_type {
                    fn event_codes() -> #sails_path::Vec<u16> {
                        #sails_path::Vec::new()
                    }
                }

                #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                fn canonical_cache() -> &'static (&'static [u8], u32, u64) {
                    static CACHE: #sails_path::spin::Once<(&'static [u8], u32, u64)> =
                        #sails_path::spin::Once::new();
                    CACHE.call_once(|| {
                        let document = #sails_path::interface_id::runtime::build_canonical_document::<#service_type_path>()
                            .expect("building canonical document should succeed");
                        let bytes = document
                            .to_bytes()
                            .expect("canonical document serialization should succeed");
                        let (id32, uid64) = #sails_path::interface_id::compute_ids_from_bytes(&bytes);
                        let leaked = #sails_path::boxed::Box::leak(bytes.into_boxed_slice());
                        (leaked as &'static [u8], id32, uid64)
                    })
                }

                pub fn canonical_service() -> &'static [u8] {
                    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                    {
                        let (bytes, _, _) = *canonical_cache();
                        bytes
                    }
                    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
                    {
                        CANONICAL_BYTES
                    }
                }

                pub fn interface_id32() -> u32 {
                    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                    {
                        let (_, id32, _) = *canonical_cache();
                        id32
                    }
                    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
                    {
                        INTERFACE_ID32
                    }
                }

                pub fn interface_uid64() -> u64 {
                    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                    {
                        let (_, _, uid64) = *canonical_cache();
                        uid64
                    }
                    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
                    {
                        INTERFACE_UID64
                    }
                }

                pub fn extends() -> &'static [#sails_path::meta::ExtendedInterface] {
                    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                    {
                        static EXTENDS: #sails_path::spin::Once<&'static [#sails_path::meta::ExtendedInterface]> =
                            #sails_path::spin::Once::new();
                        *EXTENDS.call_once(|| {
                            let mut entries: #sails_path::Vec<#sails_path::meta::ExtendedInterface> =
                                #sails_path::Vec::new();
                            #( #extends_builders )*
                            #sails_path::boxed::Box::leak(entries.into_boxed_slice())
                        })
                    }
                    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
                    {
                        EXTENDS
                    }
                }
            }
        }
    }
}
