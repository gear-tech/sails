use core::marker::PhantomData;
use sails_rs::{cell::RefCell, prelude::*};

pub trait MetadataStorage {}

impl MetadataStorage for u8 {}

// State shared by both services.
static mut GENERIC_DATA: Option<RefCell<u32>> = None;

#[allow(static_mut_refs)]
pub(crate) fn generic_data() -> &'static RefCell<u32> {
    unsafe {
        GENERIC_DATA
            .as_ref()
            .unwrap_or_else(|| panic!("`OverrideGeneric` data should be initialized first"))
    }
}

pub(crate) fn init_generic_data(initial: u32) {
    unsafe {
        GENERIC_DATA = Some(RefCell::new(initial));
    }
}

pub struct BaseService<T> {
    _marker: PhantomData<T>,
}

impl<T: MetadataStorage> BaseService<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[service]
impl<T: MetadataStorage> BaseService<T> {
    #[export]
    pub fn foo(&self) -> u32 {
        *generic_data().borrow()
    }

    #[export]
    pub fn set_value(&mut self, value: u32) {
        *generic_data().borrow_mut() = value;
    }
}

pub struct ChildService<T> {
    _marker: PhantomData<T>,
}

impl<T: MetadataStorage> ChildService<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> From<ChildService<T>> for BaseService<u8> {
    fn from(_: ChildService<T>) -> Self {
        BaseService {
            _marker: PhantomData,
        }
    }
}

#[service(extends = BaseService<u8>)]
impl<T: MetadataStorage> ChildService<T> {
    #[export(overrides = BaseService<u8>)]
    pub fn foo(&self) -> u32 {
        *generic_data().borrow() * 2
    }
}
