use cell::RefCell;
use sails_rtl::gstd::gservice;
use sails_rtl::prelude::*;

pub struct QuerySvc<'a> {
    value: &'a RefCell<u64>,
}

#[gservice]
impl<'a> QuerySvc<'a> {
    pub fn new(value: &'a RefCell<u64>) -> Self {
        Self { value }
    }

    pub fn current_value(&self) -> u64 {
        *self.value.borrow()
    }
}
