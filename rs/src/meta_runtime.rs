extern crate alloc;

use crate::meta::{ExtendedInterface, ServiceMeta};
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use alloc::boxed::Box;
use alloc::vec::Vec;
#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use std::sync::OnceLock;

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
use crate::interface_id;

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
fn canonical_cache<S: ServiceMeta + 'static>() -> &'static (&'static [u8], u64) {
    static CACHE: OnceLock<(&'static [u8], u64)> = OnceLock::new();
    CACHE.get_or_init(|| {
        let document = interface_id::runtime::build_canonical_document::<S>()
            .expect("building canonical document should succeed");
        let bytes = document
            .to_bytes()
            .expect("canonical document serialization should succeed");
        let id = interface_id::compute_ids_from_bytes(&bytes);
        let leaked = Box::leak(bytes.into_boxed_slice());
        (leaked as &'static [u8], id)
    })
}

pub fn canonical_service<S: ServiceMeta + 'static>(precomputed: &'static [u8]) -> &'static [u8] {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        canonical_cache::<S>().0
    }
    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
    {
        precomputed
    }
}

pub fn interface_id<S: ServiceMeta + 'static>(fallback: u64) -> u64 {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        canonical_cache::<S>().1
    }
    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
    {
        fallback
    }
}

pub fn extends<S: ServiceMeta + 'static>(
    precomputed: &'static [ExtendedInterface],
    builder: fn(&mut Vec<ExtendedInterface>),
) -> &'static [ExtendedInterface] {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        static EXTENDS: OnceLock<&'static [ExtendedInterface]> = OnceLock::new();
        *EXTENDS.get_or_init(|| {
            let mut entries: Vec<ExtendedInterface> = Vec::new();
            builder(&mut entries);
            Box::leak(entries.into_boxed_slice())
        })
    }
    #[cfg(not(all(feature = "std", not(target_arch = "wasm32"))))]
    {
        let _ = builder;
        precomputed
    }
}
