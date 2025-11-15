use super::*;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn canonical_module(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let module_ident = &self.canonical_module_ident;
        let async_match_arms = self.async_match_arms();

        let match_body = if async_match_arms.is_empty() {
            quote!(false)
        } else {
            quote! {
                match (entry.name.as_str(), entry.kind) {
                    #(#async_match_arms),*,
                    _ => false,
                }
            }
        };

        let const_module = quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(all(
                feature = "sails-canonical",
                not(feature = "sails-meta-dump"),
                not(sails_canonical_dump)
            ))]
            mod #module_ident {
                use super::*;
                use #sails_path::meta::{CanonicalEntry, EntryKind, EntryMeta};

                fn entry_is_async(entry: &CanonicalEntry) -> bool {
                    #match_body
                }

                pub fn interface_id() -> u64 {
                    super::INTERFACE_ID
                }

                pub fn entry_meta() -> &'static [EntryMeta<'static>] {
                    super::ENTRY_META
                }

                pub fn canonical_json() -> &'static [u8] {
                    super::CANONICAL_INTERFACE_JSON
                }

                pub fn __sails_entry_async_lookup(entry: &CanonicalEntry) -> bool {
                    entry_is_async(entry)
                }
            }
        };

        let stub_module = quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(any(
                not(feature = "sails-canonical"),
                feature = "sails-meta-dump",
                sails_canonical_dump
            ))]
            mod #module_ident {
                use super::*;
                use #sails_path::meta::{CanonicalEntry, EntryKind, EntryMeta};

                static ENTRY_META_STUB: [EntryMeta<'static>; 0] = [];
                static CANONICAL_JSON_STUB: [u8; 0] = [];

                fn entry_is_async(entry: &CanonicalEntry) -> bool {
                    #match_body
                }

                pub fn interface_id() -> u64 {
                    0
                }

                pub fn entry_meta() -> &'static [EntryMeta<'static>] {
                    &ENTRY_META_STUB
                }

                pub fn canonical_json() -> &'static [u8] {
                    &CANONICAL_JSON_STUB
                }

                pub fn __sails_entry_async_lookup(entry: &CanonicalEntry) -> bool {
                    entry_is_async(entry)
                }
            }
        };

        quote! {
            #const_module
            #stub_module
        }
    }

    pub(super) fn canonical_api_impl(&self) -> TokenStream {
        let module_ident = &self.canonical_module_ident;
        let sails_path = self.sails_path;
        let service_type_path = self.type_path;
        let generics = &self.generics;
        let type_constraints = self.type_constraints();

        quote! {
            impl #generics #service_type_path #type_constraints {
                pub fn interface_id() -> u64 {
                    #module_ident::interface_id()
                }

                pub fn entry_meta() -> &'static [#sails_path::meta::EntryMeta<'static>] {
                    #module_ident::entry_meta()
                }

                pub fn canonical_interface_json() -> &'static [u8] {
                    #module_ident::canonical_json()
                }

                #[cfg(feature = "sails-canonical")]
                pub fn __sails_entry_async(entry: &#sails_path::meta::CanonicalEntry) -> bool {
                    #module_ident::__sails_entry_async_lookup(entry)
                }
            }
        }
    }

    pub(super) fn canonical_include(&self) -> TokenStream {
        let consts_rel_path = format!(
            "/sails_interface_consts/{}.rs",
            self.service_ident.to_string()
        );
        let consts_literal = Literal::string(&consts_rel_path);
        quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(all(
                feature = "sails-canonical",
                not(feature = "sails-meta-dump"),
                not(sails_canonical_dump)
            ))]
            include!(concat!(env!("OUT_DIR"), #consts_literal));
        }
    }

    fn async_match_arms(&self) -> Vec<TokenStream> {
        self.service_handlers
            .iter()
            .map(|handler| {
                let route_literal = Literal::string(&handler.route);
                let entry_kind = if handler.is_query() {
                    quote!(EntryKind::Query)
                } else {
                    quote!(EntryKind::Command)
                };
                let is_async = handler.is_async();
                quote! {
                    (#route_literal, #entry_kind) => #is_async
                }
            })
            .collect()
    }
}
