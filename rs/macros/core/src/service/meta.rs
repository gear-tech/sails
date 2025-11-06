use super::*;
use proc_macro_error::emit_warning;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{ToTokens, quote};
use sails_interface_id::canonical::{
    CanonicalDocument, CanonicalExtendedInterface, CanonicalFunction, CanonicalHashMeta,
    CanonicalParam, CanonicalService, CanonicalType, FunctionKind,
};
use sails_interface_id::compute_ids_from_document;
use std::{collections::BTreeMap, env, fs, sync::OnceLock};

#[derive(Debug, Clone)]
struct CanonicalServiceInfo {
    interface_id: u64,
}

fn canonical_service_info(interface_path: &str) -> Option<&'static CanonicalServiceInfo> {
    static CACHE: OnceLock<Option<BTreeMap<String, CanonicalServiceInfo>>> = OnceLock::new();
    let cache = CACHE.get_or_init(load_canonical_cache);
    cache.as_ref().and_then(|map| map.get(interface_path))
}

fn load_canonical_cache() -> Option<BTreeMap<String, CanonicalServiceInfo>> {
    let path = env::var("SAILS_INTERFACE_CANONICAL").ok()?;
    let contents = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(err) => {
            emit_warning!(
                Span::call_site(),
                "failed to read canonical document `{}`: {}",
                path,
                err
            );
            return None;
        }
    };
    let canonical = match CanonicalDocument::from_json_str(&contents) {
        Ok(doc) => doc,
        Err(err) => {
            emit_warning!(
                Span::call_site(),
                "failed to parse canonical document `{}`: {}",
                path,
                err
            );
            return None;
        }
    };

    let mut cache = BTreeMap::new();
    let schema = canonical.canon_schema().to_owned();
    let version = canonical.canon_version().to_owned();
    let hash = canonical.hash().clone();
    let types = canonical.types().clone();

    for (name, service) in canonical.services() {
        let mut single_services = BTreeMap::new();
        single_services.insert(name.clone(), service.clone());
        let document = CanonicalDocument::from_parts(
            schema.clone(),
            version.clone(),
            hash.clone(),
            single_services,
            types.clone(),
        );
        let interface_id = compute_ids_from_document(&document);
        cache.insert(name.clone(), CanonicalServiceInfo { interface_id });
    }

    Some(cache)
}

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
                const INTERFACE_ID: u64 = #meta_module_ident::INTERFACE_ID;
                const EXTENDS: &'static [#sails_path::meta::ExtendedInterface] = #meta_module_ident::EXTENDS;

                fn command_entry_ids() -> #sails_path::Vec<u16> {
                    let mut ids = #sails_path::Vec::new();
                    ids.extend(#meta_module_ident::COMMAND_ENTRY_IDS.iter().copied());
                    #( ids.extend(<#base_types as #sails_path::meta::ServiceMeta>::command_entry_ids()); )*
                    ids
                }

                fn local_command_entry_ids() -> &'static [u16] {
                    #meta_module_ident::COMMAND_ENTRY_IDS
                }

                fn query_entry_ids() -> #sails_path::Vec<u16> {
                    let mut ids = #sails_path::Vec::new();
                    ids.extend(#meta_module_ident::QUERY_ENTRY_IDS.iter().copied());
                    #( ids.extend(<#base_types as #sails_path::meta::ServiceMeta>::query_entry_ids()); )*
                    ids
                }

                fn local_query_entry_ids() -> &'static [u16] {
                    #meta_module_ident::QUERY_ENTRY_IDS
                }

                fn event_entry_ids() -> #sails_path::Vec<u16> {
                    let mut ids = #meta_module_ident::event_entry_ids();
                    #( ids.extend(<#base_types as #sails_path::meta::ServiceMeta>::event_entry_ids()); )*
                    ids
                }

                fn local_event_entry_ids() -> #sails_path::Vec<u16> {
                    #meta_module_ident::event_entry_ids()
                }

                fn interface_id() -> u64 {
                    #meta_module_ident::interface_id()
                }

                fn extends() -> &'static [#sails_path::meta::ExtendedInterface] {
                    #meta_module_ident::extends()
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
        let command_entry_id_literals = self
            .service_handlers
            .iter()
            .filter(|fn_builder| !fn_builder.is_query())
            .map(|fn_builder| fn_builder.entry_id_literal());
        let query_entry_id_literals = self
            .service_handlers
            .iter()
            .filter(|fn_builder| fn_builder.is_query())
            .map(|fn_builder| fn_builder.entry_id_literal());
        let event_entry_ids_fn = self.events_type.map_or_else(
            || {
                quote! {
                    pub fn event_entry_ids() -> #sails_path::Vec<u16> {
                        #sails_path::Vec::new()
                    }
                }
            },
            |events_type| {
                quote! {
                    pub fn event_entry_ids() -> #sails_path::Vec<u16> {
                        <#events_type as #sails_path::meta::EventEntryIdMeta>::event_entry_ids()
                    }
                }
            },
        );

        let interface_path = self.type_path.to_token_stream().to_string();

        let canonical_extends = self
            .base_types
            .iter()
            .map(|base_type| CanonicalExtendedInterface {
                name: shared::remove_lifetimes(base_type)
                    .to_token_stream()
                    .to_string(),
                interface_id: 0,
                service: None,
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
                CanonicalFunction {
                    kind,
                    name: fn_builder.route.clone(),
                    route: None,
                    params,
                    returns: CanonicalType::Unit,
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

        let fallback_document = CanonicalDocument::from_parts(
            sails_interface_id::canonical::CANONICAL_SCHEMA,
            sails_interface_id::canonical::CANONICAL_VERSION,
            CanonicalHashMeta {
                algo: sails_interface_id::canonical::CANONICAL_HASH_ALGO.to_owned(),
                domain: sails_interface_id::INTERFACE_HASH_DOMAIN_STR.to_owned(),
            },
            canonical_services,
            BTreeMap::new(),
        );

        let fallback_interface_id = compute_ids_from_document(&fallback_document);
        let interface_id_value = if let Some(info) = canonical_service_info(&interface_path) {
            info.interface_id
        } else {
            fallback_interface_id
        };

        let interface_id_lit = Literal::u64_unsuffixed(interface_id_value);

        let extends_entries_const = self
            .base_types
            .iter()
            .map(|base_type| {
                let name = shared::remove_lifetimes(base_type)
                    .to_token_stream()
                    .to_string();
                let name_lit = Literal::string(&name);
                let base_type_no_lifetimes = shared::remove_lifetimes(base_type);
                quote! {
                    #sails_path::meta::ExtendedInterface {
                        name: #name_lit,
                        interface_id: <#base_type_no_lifetimes as #sails_path::meta::ServiceMeta>::INTERFACE_ID,
                    }
                }
            })
            .collect::<Vec<_>>();

        let extends_pushes = self.base_types.iter().map(|base_type| {
            let name = shared::remove_lifetimes(base_type)
                .to_token_stream()
                .to_string();
            let name_lit = Literal::string(&name);
            let base_type_no_lifetimes = shared::remove_lifetimes(base_type);
            quote! {
                entries.push(#sails_path::meta::ExtendedInterface {
                    name: #name_lit,
                    interface_id: <#base_type_no_lifetimes as #sails_path::meta::ServiceMeta>::interface_id(),
                });
            }
        });

        quote! {
            #[allow(unexpected_cfgs)]
            mod #meta_module_ident {
                use super::*;
                use #sails_path::{Decode, TypeInfo};
                use #sails_path::gstd::InvocationIo;
                use #sails_path::spin::Once;

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

                pub const INTERFACE_ID: u64 = #interface_id_lit;
                pub const EXTENDS: &[#sails_path::meta::ExtendedInterface] = &[ #( #extends_entries_const ),* ];

                pub const COMMAND_ENTRY_IDS: &[u16] = &[ #( #command_entry_id_literals ),* ];
                pub const QUERY_ENTRY_IDS: &[u16] = &[ #( #query_entry_id_literals ),* ];
                #event_entry_ids_fn

                impl #sails_path::meta::EventEntryIdMeta for #no_events_type {
                    fn event_entry_ids() -> #sails_path::Vec<u16> {
                        #sails_path::Vec::new()
                    }
                }

                #[cfg(feature = "std")]
                fn computed_interface_id() -> u64 {
                    static ID: Once<u64> = Once::new();
                    *ID.call_once(|| {
                        use #sails_path::interface_id;
                        let document = interface_id::runtime::build_canonical_document::<#service_type_path>()
                            .expect("building canonical document should succeed");
                        interface_id::compute_ids_from_document(&document)
                    })
                }

                pub fn interface_id() -> u64 {
                    #[cfg(feature = "std")]
                    {
                        computed_interface_id()
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        INTERFACE_ID
                    }
                }

                pub fn extends() -> &'static [#sails_path::meta::ExtendedInterface] {
                    #[cfg(feature = "std")]
                    {
                        static EXTENDS: Once<&'static [#sails_path::meta::ExtendedInterface]> = Once::new();
                        *EXTENDS.call_once(|| {
                            let mut entries: #sails_path::Vec<#sails_path::meta::ExtendedInterface> =
                                #sails_path::Vec::new();
                            #( #extends_pushes )*
                            #sails_path::boxed::Box::leak(entries.into_boxed_slice())
                        })
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        EXTENDS
                    }
                }
            }
        }
    }
}
