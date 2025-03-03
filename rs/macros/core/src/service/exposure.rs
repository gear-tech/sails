use super::*;
use proc_macro2::TokenStream;
use quote::quote;

impl ServiceBuilder<'_> {
    pub(super) fn exposure_struct(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let lifetimes = (!self.base_types.is_empty())
            .then(|| shared::extract_lifetimes(self.type_args))
            .flatten();
        let exposure_lifetimes = lifetimes.map(|lifetimes| quote! { #( #lifetimes, )* });

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

        quote! {
            pub struct #exposure_ident<#exposure_lifetimes T> {
                #message_id_ident : #sails_path::MessageId,
                #route_ident : &'static [u8],
                #[cfg(not(target_arch = "wasm32"))]
                #inner_ident : Box<T>, // Ensure service is not movable
                #[cfg(not(target_arch = "wasm32"))]
                #inner_ptr_ident : *const T, // Prevent exposure being Send + Sync
                #[cfg(target_arch = "wasm32")]
                #inner_ident : T,
                #base_ident: ( #(#base_exposure_types,)* )
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
            impl<T> Drop for #exposure_ident <T> {
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

    pub(super) fn service_trait_impl(&self) -> TokenStream {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;
        let service_type_constraints = self.type_constraints();

        let lifetimes = (!self.base_types.is_empty())
            .then(|| shared::extract_lifetimes(self.type_args))
            .flatten();
        let exposure_lifetimes = lifetimes.map(|lifetimes| quote! { #( #lifetimes, )* });

        // Replace special "_" lifetimes with '_1', '_2' etc.
        let mut service_impl_generics = self.generics.clone();
        let mut service_impl_type_path = self.type_path.clone();
        let service_type_args = self.type_args.clone();
        if let PathArguments::AngleBracketed(mut type_args) = service_type_args {
            for (idx, a) in type_args.args.iter_mut().enumerate() {
                if let GenericArgument::Lifetime(lifetime) = a {
                    if lifetime.ident == "_" {
                        let ident = Ident::new(&format!("_{idx}"), Span::call_site());
                        lifetime.ident = ident;
                        service_impl_generics
                            .params
                            .push(GenericParam::Lifetime(LifetimeParam::new(lifetime.clone())));
                    }
                }
            }

            service_impl_type_path
                .path
                .segments
                .last_mut()
                .unwrap()
                .arguments = PathArguments::AngleBracketed(type_args);
        }

        let message_id_ident = &self.message_id_ident;
        let route_ident = &self.route_ident;
        let inner_ident = &self.inner_ident;
        let inner_ptr_ident = &self.inner_ptr_ident;
        let base_ident = &self.base_ident;

        let base_exposure_instantiations = self.base_types.iter().map(|base_type| {
            let path_wo_lifetimes = shared::remove_lifetimes(base_type);
            quote! {
                < #path_wo_lifetimes as Clone>::clone(AsRef::< #path_wo_lifetimes >::as_ref( #inner_ident )).expose( #message_id_ident , #route_ident )
            }
        });

        quote!(
            impl #service_impl_generics #sails_path::gstd::services::Service for #service_impl_type_path #service_type_constraints {
                type Exposure = #exposure_ident< #exposure_lifetimes Self>;

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
