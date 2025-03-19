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

    pub(super) fn exposure_listen_and_drop(&self) -> Option<TokenStream> {
        let sails_path = self.sails_path;
        let exposure_ident = &self.exposure_ident;

        self.events_type.map(|events_type| {
            quote! {
                #[cfg(not(target_arch = "wasm32"))]
                const _: () = {
                    type ServiceEventsMap = #sails_path::collections::BTreeMap<usize, #sails_path::Vec< #events_type >>;
                    type Mutex<T> = #sails_path::spin::Mutex<T>;

                    // Impl for `Exposure<T>` with concrete events type for cleanup via Drop
                    impl<T: #sails_path::gstd::services::Service> #exposure_ident <T> {
                        pub fn take_events(&mut self) -> Vec< #events_type > {
                            if core::mem::size_of_val(self.inner.as_ref()) == 0 {
                                panic!("setting event listener on a zero-sized service is not supported for now");
                            }

                            let service_ptr = self.inner_ptr as usize;
                            let mut map = Self::events_map();
                            map.remove(&service_ptr).unwrap_or_default()
                        }

                        fn notify_on(svc: &mut T, event: #events_type) -> #sails_path::errors::Result<()> {
                            let service_ptr = svc as *const _ as *const () as usize;
                            let mut map = Self::events_map();
                            map.entry(service_ptr).or_default().push(event);
                            Ok(())
                        }

                        fn events_map() -> impl core::ops::DerefMut<Target = ServiceEventsMap> {
                            static MAP: Mutex<ServiceEventsMap> = Mutex::new(ServiceEventsMap::new());
                            MAP.lock()
                        }
                    }

                    impl<T: #sails_path::gstd::services::Service> Drop for #exposure_ident <T> {
                        fn drop(&mut self) {
                            let service_ptr = self.inner_ptr as usize;
                            let mut map = Self::events_map();
                            _ = map.remove(&service_ptr);
                        }
                    }
                };
            }
        })
    }

    pub(super) fn service_with_events_impls(&self) -> Option<TokenStream> {
        let sails_path = self.sails_path;
        let generics = &self.generics;
        let service_type_path = self.type_path;
        let service_type_constraints = self.type_constraints();
        let exposure_ident = &self.exposure_ident;

        self.events_type.map(|events_type| {
            quote! {
                impl #generics #service_type_path #service_type_constraints {
                    fn notify_on(&mut self, event: #events_type ) -> #sails_path::errors::Result<()>  {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            #exposure_ident::<Self>::notify_on(self, event)
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            #sails_path::gstd::__notify_on(event)
                        }
                    }
                }
            }
        })
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
