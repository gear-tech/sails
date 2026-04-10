use core::marker::PhantomData;
use sails_rs::{cell::RefCell, prelude::*};

pub trait MetadataStorage {}

impl MetadataStorage for u8 {}
impl MetadataStorage for u16 {}

static mut GENERIC_DATA: Option<RefCell<u32>> = None;

#[allow(static_mut_refs)]
pub(crate) fn generic_data() -> &'static RefCell<u32> {
    unsafe {
        GENERIC_DATA
            .as_ref()
            .unwrap_or_else(|| panic!("`OverrideGenerics` data should be initialized first"))
    }
}

pub(crate) fn init_generic_data(initial: u32) {
    unsafe {
        GENERIC_DATA = Some(RefCell::new(initial));
    }
}

pub struct BaseService<T = u8> {
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

pub struct ChildService<T = u16> {
    _marker: PhantomData<T>,
}

impl<T: MetadataStorage> ChildService<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> From<ChildService<T>> for BaseService<T> {
    fn from(_: ChildService<T>) -> Self {
        BaseService {
            _marker: PhantomData,
        }
    }
}

#[service(extends = BaseService<T>)]
impl<T: MetadataStorage> ChildService<T> {
    /// The generic service type should use either a concrete type or the default type.
    ///
    /// ```rust
    /// #[export(overrides = BaseService)]
    /// ```
    /// ```rust
    /// #[export(overrides = BaseService<u16>)]
    /// ```
    #[export(overrides = BaseService<T>)]
    pub fn foo(&self) -> u32 {
        *generic_data().borrow() * 2
    }
}
