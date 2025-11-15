use super::*;
use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};

impl ServiceBuilder<'_> {
    pub(super) fn canonical_module(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let meta_witness_ident = &self.meta_witness_ident;
        let module_ident = &self.canonical_module_ident;
        let service_name_literal = Literal::string(&self.service_ident.to_string());
        let async_match_arms = self.async_match_arms();

        let base_types: Vec<_> = self
            .base_types
            .iter()
            .map(shared::remove_lifetimes)
            .collect();
        let base_name_literals: Vec<_> = self
            .base_types
            .iter()
            .map(|path| {
                let name = path
                    .segments
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_else(|| path.to_token_stream().to_string());
                Literal::string(&name)
            })
            .collect();

        let parent_setup = if base_types.is_empty() {
            quote! {
                let context = CanonicalizationContext::default();
            }
        } else {
            let base_meta_inits =
                base_types
                    .iter()
                    .zip(base_name_literals.iter())
                    .map(|(path, name)| {
                        quote! {
                            {
                                let parent_meta = AnyServiceMeta::new::<#path>();
                                let parent_unit = build_service_unit(#name, &parent_meta)
                                    .expect("failed to build parent service AST");
                                parent_units.push((parent_unit, #path::interface_id()));
                            }
                        }
                    });

            quote! {
                let mut parent_units = Vec::new();
                #(#base_meta_inits)*

                let mut parent_interfaces = Vec::new();
                for (unit, interface_id) in parent_units.iter() {
                    parent_interfaces.push(ParentInterface::new(unit, *interface_id));
                }
                let context = CanonicalizationContext::with_parents(parent_interfaces.as_slice());
            }
        };

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

        let runtime_module = quote! {
            #[cfg(all(
                feature = "runtime-canonical",
                any(
                    not(feature = "sails-canonical"),
                    feature = "sails-meta-dump",
                    sails_canonical_dump
                )
            ))]
            mod #module_ident {
                use super::*;
                use #sails_path::prelude::{boxed::Box, vec::Vec};
                use #sails_path::meta::{
                    AnyServiceMeta,
                    CanonicalizationContext,
                    ParentInterface,
                    build_service_unit,
                    compute_interface_id,
                    interface::{build_entry_meta_with_async, EntryKind, EntryMeta},
                    CanonicalEntry,
                };
                use #sails_path::spin::Once;

                struct InterfaceMetadata {
                    interface_id: u64,
                    entry_meta: &'static [EntryMeta<'static>],
                    canonical_json: &'static [u8],
                }

                static META: Once<InterfaceMetadata> = Once::new();

                fn entry_is_async(entry: &CanonicalEntry) -> bool {
                    #match_body
                }

                fn compute() -> InterfaceMetadata {
                    let service_meta = AnyServiceMeta::new::<#meta_witness_ident>();
                    let service_unit = build_service_unit(#service_name_literal, &service_meta)
                        .expect("failed to build service AST");

                    #parent_setup

                    let result = compute_interface_id(&service_unit, &context)
                        .expect("canonicalization failed");

                    let entry_meta_vec = build_entry_meta_with_async(&result.envelope, entry_is_async)
                    .expect("entry metadata assignment failed");

                    let entry_meta = Box::leak(entry_meta_vec.into_boxed_slice());
                    let canonical_json = Box::leak(result.canonical_json.into_boxed_slice());

                    InterfaceMetadata {
                        interface_id: result.interface_id,
                        entry_meta,
                        canonical_json,
                    }
                }

                fn metadata() -> &'static InterfaceMetadata {
                    META.call_once(compute)
                }

                pub fn interface_id() -> u64 {
                    metadata().interface_id
                }

                pub fn entry_meta() -> &'static [EntryMeta<'static>] {
                    metadata().entry_meta
                }

                pub fn canonical_json() -> &'static [u8] {
                    metadata().canonical_json
                }

                #[cfg(feature = "sails-canonical")]
                pub fn __sails_entry_async_lookup(entry: &CanonicalEntry) -> bool {
                    entry_is_async(entry)
                }
            }
        };

        let const_module = quote! {
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
            #[cfg(all(
                not(feature = "runtime-canonical"),
                any(
                    not(feature = "sails-canonical"),
                    feature = "sails-meta-dump",
                    sails_canonical_dump
                )
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
            #runtime_module
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
