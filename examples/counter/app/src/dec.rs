use cell::RefCell;
use sails_rtl::gstd::gservice;
use sails_rtl::prelude::*;

pub struct DecSvc<'a> {
    value: &'a RefCell<u64>,
}

#[gservice]
impl<'a> DecSvc<'a> {
    pub fn new(value: &'a RefCell<u64>) -> Self {
        Self { value }
    }

    pub fn op(&mut self, val: u64) -> u64 {
        self.value.replace_with(|&mut old| old - val)
    }
}
