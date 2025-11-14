use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn meta_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();
        let meta_module_ident = &self.meta_module_ident;
        let base_services_meta = self.base_services_meta_tokens();
        let service_meta_asyncness = self.service_meta_asyncness_tokens();

        quote! {
            impl #generics #sails_path::meta::ServiceMeta for #service_type_path #service_type_constraints {
                type CommandsMeta = #meta_module_ident::CommandsMeta;
                type QueriesMeta = #meta_module_ident::QueriesMeta;
                type EventsMeta = #meta_module_ident::EventsMeta;
                const BASE_SERVICES: &'static [#sails_path::meta::AnyServiceMetaFn] = &[
                    #( #base_services_meta ),*
                ];
                const ASYNC: bool = #service_meta_asyncness ;
            }
        }
    }

    pub(super) fn meta_witness_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let meta_module_ident = &self.meta_module_ident;
        let meta_witness_ident = &self.meta_witness_ident;
        let base_services_meta = self.base_services_meta_tokens();
        let service_meta_asyncness = self.service_meta_asyncness_tokens();

        quote! {
            #[doc(hidden)]
            pub struct #meta_witness_ident;

            impl #sails_path::meta::ServiceMeta for #meta_witness_ident {
                type CommandsMeta = #meta_module_ident::CommandsMeta;
                type QueriesMeta = #meta_module_ident::QueriesMeta;
                type EventsMeta = #meta_module_ident::EventsMeta;
                const BASE_SERVICES: &'static [#sails_path::meta::AnyServiceMetaFn] = &[
                    #( #base_services_meta ),*
                ];
                const ASYNC: bool = #service_meta_asyncness;
            }
        }
    }

    pub(super) fn meta_helper_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let service_type_path = self.type_path;
        let generics = &self.generics;
        let service_type_constraints = self.type_constraints();
        let meta_witness_ident = &self.meta_witness_ident;

        quote! {
            impl #generics #service_type_path #service_type_constraints {
                #[doc(hidden)]
                pub fn __sails_any_service_meta() -> #sails_path::meta::AnyServiceMeta {
                    #sails_path::meta::AnyServiceMeta::new::<#meta_witness_ident>()
                }
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

    fn base_services_meta_tokens(&self) -> Vec<TokenStream> {
        let sails_path = self.sails_path;
        self.base_types
            .iter()
            .map(|base_type| {
                let path_wo_lifetimes = shared::remove_lifetimes(base_type);
                quote! {
                    #sails_path::meta::AnyServiceMeta::new::< #path_wo_lifetimes >
                }
            })
            .collect()
    }

    fn service_meta_asyncness_tokens(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let has_async_handler = self
            .service_handlers
            .iter()
            .any(|fn_builder| fn_builder.is_async());

        if has_async_handler {
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
        }
    }
}
