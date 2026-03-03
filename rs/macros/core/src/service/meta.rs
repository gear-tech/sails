use super::*;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::BTreeSet;

impl ServiceBuilder<'_> {
    pub(super) fn meta_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        // TODO [future]: remove the duplicates check for the Sails binary protocol
        let mut base_names = BTreeSet::new();
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
                    base_type,
                    "Base service with the same name was defined - `{}`",
                    base_type_pathless_name
                );
            }

            quote! {
                #sails_path::meta::BaseServiceMeta::new::< super:: #path_wo_lifetimes >( #base_type_pathless_name )
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
                    <super:: #path_wo_lifetimes as #sails_path::meta::ServiceMeta>::ASYNC
                }
            });
            quote!(#( #base_asyncness )||*)
        };

        let interface_id_computation = self.generate_interface_id();
        let methods_meta = self.generate_methods_meta();

        let override_validations = self.generate_override_validations();

        quote! {
            impl #generics #sails_path::meta::Identifiable for super:: #service_type_path #service_type_constraints {
                const INTERFACE_ID: #sails_path::meta::InterfaceId = #interface_id_computation;
            }

            impl #generics #sails_path::meta::ServiceMeta for super:: #service_type_path #service_type_constraints {
                type CommandsMeta = CommandsMeta;
                type QueriesMeta = QueriesMeta;
                type EventsMeta = EventsMeta;
                const BASE_SERVICES: &'static [#sails_path::meta::BaseServiceMeta] = &[
                    #( #base_services_meta ),*
                ];
                const METHODS: &'static [#sails_path::meta::MethodMetadata] = &[
                    #( #methods_meta ),*
                ];
                const ASYNC: bool = #service_meta_asyncness ;
            }

            #override_validations
        }
    }

    pub(super) fn meta_module(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let scale_codec_path = &sails_paths::scale_codec_path(sails_path);
        let scale_info_path = &sails_paths::scale_info_path(sails_path);
        let meta_module_ident = &self.meta_module_ident;

        let no_events_type = Path::from(Ident::new("NoEvents", Span::call_site()));
        let events_type = self.events_type.unwrap_or(&no_events_type);

        let invocation_params_structs = self.service_handlers.iter().map(|fn_builder| {
            fn_builder.params_struct(self.type_path, scale_codec_path, scale_info_path)
        });
        let commands_meta_variants = self
            .service_handlers
            .iter()
            .filter(|h| h.overrides.is_none())
            .filter_map(|fn_builder| {
                (!fn_builder.is_query()).then_some(fn_builder.handler_meta_variant())
            });
        let queries_meta_variants = self
            .service_handlers
            .iter()
            .filter(|h| h.overrides.is_none())
            .filter_map(|fn_builder| {
                (fn_builder.is_query()).then_some(fn_builder.handler_meta_variant())
            });

        let meta_trait_impl = self.meta_trait_impl();

        quote! {
            mod #meta_module_ident {
                use super::*;

                #meta_trait_impl

                #( #invocation_params_structs )*

                #[derive(#sails_path::TypeInfo)]
                #[scale_info(crate = #scale_info_path)]
                pub enum CommandsMeta {
                    #(#commands_meta_variants),*
                }

                #[derive(#sails_path::TypeInfo)]
                #[scale_info(crate = #scale_info_path)]
                pub enum QueriesMeta {
                    #(#queries_meta_variants),*
                }

                #[derive(#sails_path::TypeInfo)]
                #[scale_info(crate = #scale_info_path )]
                pub enum #no_events_type {}

                pub type EventsMeta = #events_type;
            }
        }
    }

    fn generate_interface_id(&self) -> TokenStream {
        let sails_path = self.sails_path;

        let fn_hash_computations: Vec<_> = self
            .service_handlers
            .iter()
            .filter(|h| h.overrides.is_none())
            .map(|handler| {
                let fn_hash = FnHashBuilder::from_handler(handler, sails_path).build();
                quote! {
                    final_hash = final_hash.update(& #fn_hash);
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
            let base_service_ids = self.base_types.iter().map(|base_type| {
                let path_wo_lifetimes = shared::remove_lifetimes(base_type);
                quote!(final_hash = final_hash.update(&<super:: #path_wo_lifetimes as #sails_path::meta::Identifiable>::INTERFACE_ID.0);)
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

    fn generate_methods_meta(&self) -> Vec<TokenStream> {
        let sails_path = self.sails_path;
        self.service_handlers
            .iter()
            .filter(|h| h.overrides.is_none())
            .map(|handler| {
                let name = &handler.route;
                let entry_id = handler.entry_id;
                let fn_hash = FnHashBuilder::from_handler(handler, sails_path).build();
                let is_async = handler.is_async();

                quote! {
                    #sails_path::meta::MethodMetadata {
                        name: #name,
                        entry_id: #entry_id,
                        hash: #fn_hash,
                        is_async: #is_async,
                    }
                }
            })
            .collect()
    }

    fn generate_override_validations(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let validations = self.service_handlers.iter().filter_map(|handler| {
            handler.overrides.as_ref().map(|base_path| {
                let name = &handler.route;
                let entry_id_arg = if let Some(id) = handler.override_entry_id {
                    quote! { Some(#id) }
                } else {
                    quote! { None }
                };

                let base_path_wo_lifetimes = shared::remove_lifetimes(base_path);
                let current_fn_hash = FnHashBuilder::from_handler(handler, sails_path)
                    .with_override_name(quote!(base_name))
                    .build();

                quote! {
                    const _: () = {
                        let base_methods = <super::#base_path_wo_lifetimes as #sails_path::meta::ServiceMeta>::METHODS;

                        if let Some(method) = #sails_path::meta::find_method_data(base_methods, #name, #entry_id_arg) {
                            let base_name = method.name;
                            if !#sails_path::meta::bytes32_eq(&method.hash, &#current_fn_hash) {
                                core::panic!(concat!("Override signature mismatch for method `", #name, "`"));
                            }
                        } else {
                            core::panic!(concat!("Method `", #name, "` not found in base service"));
                        }
                    };
                }
            })
        });

        quote! { #( #validations )* }
    }
}

struct FnHashBuilder<'a> {
    sails_path: &'a Path,
    is_query: bool,
    route: &'a str,
    arg_types: &'a [&'a Type],
    result_type: Type,
    unwrap_result: bool,
    override_name: Option<TokenStream>,
}

impl<'a> FnHashBuilder<'a> {
    fn from_handler(handler: &'a FnBuilder<'a>, sails_path: &'a Path) -> Self {
        let original_result_type = shared::result_type(&handler.impl_fn.sig);
        let static_result_type = shared::replace_any_lifetime_with_static(original_result_type);
        let (result_type, _) =
            if let Some(ty) = shared::extract_reply_type_with_value(&static_result_type) {
                (ty.clone(), true)
            } else {
                (static_result_type, false)
            };

        Self {
            sails_path,
            is_query: handler.is_query(),
            route: &handler.route,
            arg_types: handler.params_types(),
            result_type,
            unwrap_result: handler.unwrap_result,
            override_name: None,
        }
    }

    fn with_override_name(mut self, name: TokenStream) -> Self {
        self.override_name = Some(name);
        self
    }

    fn build(self) -> TokenStream {
        let sails_path = self.sails_path;

        let arg_types = self.arg_types;

        // Generate RES_HASH - check if result type is Result<T, E>
        // TODO: We only use 'flat' hashing (with | separator) if unwrap_result is true.
        // This is to stay in sync with idl-gen which only uses 'throws' if metadata has 3 fields.
        // If we want truly unified throws, we need a hashing scheme that is independent of syntactic unfolding.
        let result_tokens = if self.unwrap_result
            && let Type::Path(ref tp) = self.result_type
            && let Some((ok_ty, err_ty)) = shared::extract_result_types(tp)
        {
            // Result type: RES_HASH = b"res" || T::HASH || b"throws" || E::HASH
            quote!( -> #ok_ty | #err_ty )
        } else {
            let result_type = &self.result_type;
            // Other types: RES_HASH = b"res" || REFLECT_HASH
            quote!( -> #result_type )
        };

        if let Some(name_expr) = self.override_name {
            let kind = if self.is_query { "query" } else { "command" };
            quote! {
                #sails_path::hash_fn_raw!( #kind #name_expr, ( #( #arg_types ),* ) #result_tokens )
            }
        } else {
            let kind_ident = if self.is_query {
                quote!(query)
            } else {
                quote!(command)
            };
            let name_ident = Ident::new(self.route, Span::call_site());

            // FN_HASH = hash(bytes(FN_TYPE) || bytes(FN_NAME) || ARGS_REFLECT_HASH || RES_HASH)
            // RES_HASH = (b"res" || REFLECT_HASH) | (b"res" || T_REFLECT_HASH || bytes("throws") || E_REFLECT_HASH)
            quote! {
                #sails_path::hash_fn!( #kind_ident #name_ident ( #( #arg_types ),* ) #result_tokens )
            }
        }
    }
}
