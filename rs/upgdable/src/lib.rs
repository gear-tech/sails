#![no_std]
#![allow(dead_code)]

//pub use program_v4::{handle as handle_v4, init as init_v4};
use sails_rs::prelude::*;

mod program_source;
//mod program_v2;
//mod program_v3;
//mod program_v4;
//mod program_v5;
//mod program_v6;
mod program_v10;
mod program_v11;
mod program_v12;
mod program_v13;
mod program_v14;
mod program_v15;
mod program_v7;
mod program_v8;
mod program_v9;
#[cfg(test)]
mod tests;

struct UpgradableService(());

#[sails_rs::service]
impl UpgradableService {
    pub fn new() -> Self {
        Self(())
    }

    // Service's method (command)
    pub fn do_something(&mut self) -> String {
        "Hello from Upgradable!".to_string()
    }

    // Service's query
    pub fn get_something(&self) -> String {
        "Hello from Upgradable!".to_string()
    }
}

pub struct UpgradableProgram(());

#[sails_rs::program]
impl UpgradableProgram {
    // Program's constructor
    pub fn new() -> Self {
        Self(())
    }

    // Exposed service
    pub fn upgradable(&self) -> UpgradableService {
        let _storage = self.__storage(); // Storage can be used for passing it down to the service, maybe wrapped into some other type/trait
        UpgradableService::new()
    }
}

impl UpgradableProgram {
    // This one is called by the new version of the program. Current program should be brought into paused state
    // before transferring the storage.
    pub fn __read_storage(&self, offset: u32, size: u32) -> (Vec<u8>, bool) {
        let storage = self.__storage();
        let encoded_storage = storage.encode();
        let encoded_storage_len = encoded_storage.len().try_into().unwrap();
        let end = cmp::min(offset.saturating_add(size), encoded_storage_len);
        let has_more = end < encoded_storage_len;
        (
            encoded_storage[offset as usize..end as usize].to_vec(),
            has_more,
        )
    }

    fn __storage(&self) -> &<Self as IUpgradableProgram>::StorageType {
        __PROGRAM_STORAGE.as_ref().unwrap()
    }
}

// RefCell with default initialization doesn't work as static requires const initializer
static __PROGRAM_STORAGE: Option<ProgramStorageV1> = None;

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
// Required for automated storage initialization. Doesn't seem like storage requires any specific initialization.
// It's initialization is a responsibility of the program ctor. The idea is to have an ability to specify
// storage type via the `program` macro. If `default` is too strict, then storage initialization can be specified
// as a func
#[derive(Default)]
struct ProgramStorageV1 {
    a: u32,
}

trait IUpgradableProgram {
    type PrevStorageType: Decode;
    type StorageType: Encode;

    fn migrate(&self, prev_storage: Self::PrevStorageType) -> Self::StorageType;
}

impl IUpgradableProgram for UpgradableProgram {
    type PrevStorageType = ();
    type StorageType = ProgramStorageV1;

    fn migrate(&self, _prev_storage: Self::PrevStorageType) -> Self::StorageType {
        ProgramStorageV1::default()
    }
}
