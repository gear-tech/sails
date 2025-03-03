use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn exposure_struct(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let message_id_ident = &self.message_id_ident;
        let route_ident = &self.route_ident;
        let inner_ident = &self.inner_ident;
        let inner_ptr_ident = &self.inner_ptr_ident;
        let base_ident = &self.base_ident;

        quote! {
            pub struct #exposure_ident<T: #sails_path::gstd::services::Service> {
                #message_id_ident : #sails_path::MessageId,
                #route_ident : &'static [u8],
                #[cfg(not(target_arch = "wasm32"))]
                #inner_ident : Box<T>, // Ensure service is not movable
                #[cfg(not(target_arch = "wasm32"))]
                #inner_ptr_ident : *const T, // Prevent exposure being Send + Sync
                #[cfg(target_arch = "wasm32")]
                #inner_ident : T,
                #base_ident: T::BaseExposures,
            }

            impl<T: #sails_path::gstd::services::Service> #sails_path::gstd::services::Exposure for #exposure_ident<T> {
                fn message_id(&self) -> #sails_path::MessageId {
                    self. #message_id_ident
                }

                fn route(&self) -> &'static [u8] {
                    self. #route_ident
                }
            }
        }
    }

    pub(super) fn exposure_set_event_listener(&self) -> Option<TokenStream> {
        let sails_path = self.sails_path;

        self.events_type.map(|events_type| {
            // get non conflicting lifetime name
            let lifetimes = shared::extract_lifetime_names(self.type_args);
            let mut lt = "__elg".to_owned();
            while lifetimes.contains(&lt) {
                lt = format!("_{}", lt);
            }
            let lifetime = Lifetime::new(&format!("'{0}", lt), Span::call_site());            

            quote! {
                #[cfg(not(target_arch = "wasm32"))]
                // Immutable so one can set it via AsRef when used with extending
                pub fn set_event_listener<#lifetime>(
                    &self,
                    listener: impl FnMut(& #events_type ) + #lifetime,
                ) -> #sails_path::gstd::events::EventListenerGuard<#lifetime> {
                    if core::mem::size_of_val(self.inner.as_ref()) == 0 {
                        panic!("setting event listener on a zero-sized service is not supported for now");
                    }
                    let service_ptr = self.inner_ptr as usize;
                    let listener: Box<dyn FnMut(& #events_type )> = Box::new(listener);
                    let listener = Box::new(listener);
                    let listener_ptr = Box::into_raw(listener) as usize;
                    #sails_path::gstd::events::EventListenerGuard::new(service_ptr, listener_ptr)
                }
            }
        })
    }

    pub(super) fn exposure_drop(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;

        quote!(
            #[cfg(not(target_arch = "wasm32"))]
            impl<T: #sails_path::gstd::services::Service> Drop for #exposure_ident <T> {
                fn drop(&mut self) {
                    let service_ptr = self.inner_ptr as usize;
                    let mut event_listeners = #sails_path::gstd::events::event_listeners().lock();
                    if event_listeners.remove(&service_ptr).is_some() {
                        panic!("there should be no any event listeners left by this time");
                    }
                }
            }
        )
    }

    pub(super) fn exposure_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        let base_ident = &self.base_ident;

        // We propagate only known attributes as we don't know the consequences of unknown ones
        let exposure_allow_attrs = self
            .service_impl
            .attrs
            .iter()
            .filter(|attr| matches!(attr.path().get_ident(), Some(ident) if ident == "allow"));

        let exposure_funcs = self
            .service_handlers
            .iter()
            .map(|fn_builder| fn_builder.exposure_func(&self.inner_ident));

        let base_exposure_accessors =
            self
                .base_types
                .iter()
                .enumerate()
                .map(|(idx, base_type)| {
                    let as_base_ident = Ident::new(&format!("as_base_{}", idx), Span::call_site());
                    let idx = Literal::usize_unsuffixed(idx);
                    quote! {
                        pub fn #as_base_ident (&self) -> &< #base_type as #sails_path::gstd::services::Service>::Exposure {
                            &self. #base_ident . #idx
                        }
                    }
                });

        let exposure_set_event_listener_code = self.exposure_set_event_listener();
        let try_handle_impl = self.try_handle_impl();
        // ethexe
        let try_handle_solidity_impl = self.try_handle_solidity_impl(base_ident);

        quote! {
            #( #exposure_allow_attrs )*
            impl #generics #exposure_ident< #service_type_path > #service_type_constraints {
                #( #exposure_funcs )*

                #( #base_exposure_accessors )*

                #try_handle_impl

                #try_handle_solidity_impl

                #exposure_set_event_listener_code
            }
        }
    }

    pub(super) fn try_handle_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let base_ident = &self.base_ident;
        let input_ident = &self.input_ident;

        let invocation_dispatches = self.service_handlers.iter().map(|fn_builder| {
            fn_builder.try_handle_branch_impl(&self.meta_module_ident, input_ident)
        });
        // Base Services, as tuple in exposure `base: (...)`
        let base_exposure_invocations = self.base_types.iter().enumerate().map(|(idx, _)| {
            let idx = Literal::usize_unsuffixed(idx);
            quote! {
                if let Some((output, value)) = self. #base_ident . #idx .try_handle(#input_ident).await {
                    return Some((output, value));
                }
            }
        });

        quote! {
            pub async fn try_handle(&mut self, #input_ident : &[u8]) -> Option<(Vec<u8>, u128)> {
                use #sails_path::gstd::InvocationIo;
                #( #invocation_dispatches )*
                #( #base_exposure_invocations )*
                None
            }
        }
    }

    pub(super) fn service_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();

        let message_id_ident = &self.message_id_ident;
        let route_ident = &self.route_ident;
        let inner_ident = &self.inner_ident;
        let inner_ptr_ident = &self.inner_ptr_ident;
        let base_ident = &self.base_ident;

        let base_exposure_types = self.base_types.iter().map(|base_type| {
            quote! {
                < #base_type as #sails_path::gstd::services::Service>::Exposure
            }
        });
        let base_exposure_instantiations = self.base_types.iter().map(|base_type| {
            let path_wo_lifetimes = shared::remove_lifetimes(base_type);
            quote! {
                < #path_wo_lifetimes as Clone>::clone(AsRef::< #path_wo_lifetimes >::as_ref( #inner_ident )).expose( #message_id_ident , #route_ident )
            }
        });

        quote!(
            impl #generics #sails_path::gstd::services::Service for #service_type_path #service_type_constraints {
                type Exposure = #exposure_ident<Self>;
                type BaseExposures = ( #( #base_exposure_types, )* );

                fn expose(self, #message_id_ident : #sails_path::MessageId, #route_ident : &'static [u8]) -> Self::Exposure {
                    #[cfg(not(target_arch = "wasm32"))]
                    let inner_box = Box::new(self);
                    #[cfg(not(target_arch = "wasm32"))]
                    let #inner_ident = inner_box.as_ref();
                    #[cfg(target_arch = "wasm32")]
                    let #inner_ident = &self;
                    Self::Exposure {
                        #message_id_ident ,
                        #route_ident ,
                        #base_ident: ( #( #base_exposure_instantiations, )* ),
                        #[cfg(not(target_arch = "wasm32"))]
                        #inner_ptr_ident : inner_box.as_ref() as *const Self,
                        #[cfg(not(target_arch = "wasm32"))]
                        #inner_ident : inner_box ,
                        #[cfg(target_arch = "wasm32")]
                        #inner_ident : self,
                    }
                }
            }
        )
    }
}
