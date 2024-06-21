use cell::RefCell;
use sails_rtl::gstd::gservice;
use sails_rtl::prelude::*;

pub struct IncSvc<'a> {
    value: &'a RefCell<u64>,
}

#[derive(Clone, sails_rtl::Encode, sails_rtl::TypeInfo)]
pub enum IncEvents {
    Inc(u64)
}

#[gservice(events = IncEvents)]
impl<'a> IncSvc<'a> {
    pub fn new(value: &'a RefCell<u64>) -> Self {
        Self { value }
    }

    pub fn op(&mut self, val: u64) -> u64 {
        let _ = self.notify_on(IncEvents::Inc(val));
        self.value.replace_with(|&mut old| old + val)
    }
}
