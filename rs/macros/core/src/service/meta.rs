use super::*;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;

impl ServiceBuilder<'_> {
    pub(super) fn meta_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();
        let meta_module_ident = &self.meta_module_ident;

        // TODO [future]: remove the duplicates check for the Sails binary protocol
        let mut base_names = HashSet::new();
        let base_services_meta = self.base_types.iter().map(|base_type| {
            let path_wo_lifetimes = shared::remove_lifetimes(base_type);
            let base_type_pathless_name = path_wo_lifetimes
                .segments
                .last()
                .expect("Base service path should have at least one segment")
                .ident
                .to_string();

            if !base_names.insert(base_type_pathless_name.clone()) {
                abort!(
                    base_type, "Base service with the same name was defined - `{}`",
                    base_type_pathless_name
                );
            }

            quote! {
                (#base_type_pathless_name , #sails_path::meta::AnyServiceMeta::new::< #path_wo_lifetimes >)
            }
        });

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

        let interface_id_computation = self.generate_interface_id();

        quote! {
            impl #generics #sails_path::meta::ServiceMeta for #service_type_path #service_type_constraints {
                type CommandsMeta = #meta_module_ident::CommandsMeta;
                type QueriesMeta = #meta_module_ident::QueriesMeta;
                type EventsMeta = #meta_module_ident::EventsMeta;
                const BASE_SERVICES: &'static [(&'static str, #sails_path::meta::AnyServiceMetaFn)] = &[
                    #( #base_services_meta ),*
                ];
                const ASYNC: bool = #service_meta_asyncness ;
                const INTERFACE_ID: #sails_path::meta::InterfaceId = #interface_id_computation;
            }
        }
    }

    pub(super) fn meta_module(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let scale_codec_path = &sails_paths::scale_codec_path(sails_path);
        let scale_info_path = &sails_paths::scale_info_path(sails_path);
        let meta_module_ident = &self.meta_module_ident;

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
            }
        }
    }

    fn generate_interface_id(&self) -> TokenStream {
        let sails_path = self.sails_path;

        // Sort handlers by name for deterministic ordering
        let (mut commands, mut queries): (Vec<_>, Vec<_>) =
            self.service_handlers.iter().partition(|h| !h.is_query());

        commands.sort_by_key(|h| h.route.to_lowercase());
        queries.sort_by_key(|h| h.route.to_lowercase());

        // TODO: #1124
        let fn_hash_computations: Vec<_> = commands
            .into_iter()
            .chain(queries)
            .map(|handler| {
                let fn_type = if handler.is_query() { "query" } else { "command" };
                let fn_name = &handler.route;

                // Generate ARG_HASH = REFLECT_HASH for each argument
                let arg_hash_computations = handler.params().map(|(_, ty)| {
                    quote!(fn_hash = fn_hash.update(&<#ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH);)
                });

                // Generate RES_HASH - check if result type is Result<T, E>
                let result_type = handler.result_type_with_static_lifetime();
                let result_hash = if let Type::Path(ref tp) = result_type && let Some((ok_ty, err_ty)) = shared::extract_result_types(tp) {
                    // Result type: RES_HASH = b"res" || T::HASH || b"throws" || E::HASH
                    quote! {
                        fn_hash = fn_hash.update(b"res");
                        fn_hash = fn_hash.update(&<#ok_ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH);
                        fn_hash = fn_hash.update(b"throws");
                        fn_hash = fn_hash.update(&<#err_ty as #sails_path::sails_reflect_hash::ReflectHash>::HASH);
                    }
                } else {
                    // Other types: RES_HASH = b"res" || REFLECT_HASH
                    quote! {
                        fn_hash = fn_hash.update(b"res");
                        fn_hash = fn_hash.update(&<#result_type as #sails_path::sails_reflect_hash::ReflectHash>::HASH);
                    }
                };

                // FN_HASH = hash(bytes(FN_TYPE) || bytes(FN_NAME) || ARGS_REFLECT_HASH || RES_HASH)
                // RES_HASH = (b"res" || REFLECT_HASH) | (b"res" || T_REFLECT_HASH || bytes("throws") || E_REFLECT_HASH)
                quote! {
                    {
                        let mut fn_hash = #sails_path::keccak_const::Keccak256::new();
                        fn_hash = fn_hash.update(#fn_type.as_bytes());
                        fn_hash = fn_hash.update(#fn_name.as_bytes());
                        #(#arg_hash_computations)*
                        #result_hash
                        final_hash = final_hash.update(&fn_hash.finalize());
                    }
                }
            })
            .collect();

        // Handle events if present
        let events_hash = if let Some(events_type) = self.events_type {
            quote!(final_hash = final_hash.update(&<#events_type as #sails_path::sails_reflect_hash::ReflectHash>::HASH);)
        } else {
            quote!()
        };

        // Handle base services if present
        let base_services_hash = if !self.base_types.is_empty() {
            let mut base_services = self
                .base_types
                .iter()
                .map(shared::remove_lifetimes)
                .collect::<Vec<_>>();
            base_services.sort_by_key(|base_type_no_lifetime| {
                base_type_no_lifetime
                    .segments
                    .last()
                    .expect("Base service path should have at least one segment")
                    .ident
                    .to_string()
                    .to_lowercase()
            });

            let base_service_ids = base_services.into_iter().map(|base_type_no_lifetime| {
                quote!(final_hash = final_hash.update(&<#base_type_no_lifetime as #sails_path::meta::ServiceMeta>::INTERFACE_ID.0);)
            });

            quote!(#(#base_service_ids)*)
        } else {
            Default::default()
        };

        quote! {
            {
                let mut final_hash = #sails_path::keccak_const::Keccak256::new();

                // Hash all functions
                #(#fn_hash_computations)*

                // Hash events if present
                #events_hash

                // Hash base services if present
                #base_services_hash

                let hash = final_hash.finalize();
                #sails_path::meta::InterfaceId::from_bytes_32(hash)
            }
        }
    }
}
