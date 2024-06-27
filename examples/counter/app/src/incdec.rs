use cell::RefCell;
use sails_rtl::gstd::gservice;
use sails_rtl::prelude::*;

pub struct IncDecSvc<'q> {
    value: &'q RefCell<u64>,
}

#[derive(Clone, sails_rtl::Encode, sails_rtl::TypeInfo)]
pub enum SvcEvents {
    Inc(u64),
    Dec(u64),
    Reset,
}

#[gservice(events = SvcEvents)]
impl<'q> IncDecSvc<'q> {
    pub fn new(value: &'q RefCell<u64>) -> Self {
        Self { value }
    }

    pub fn inc(&mut self, val: u64) -> u64 {
        let _ = self.notify_on(SvcEvents::Inc(val));
        self.value.replace_with(|&mut old| old + val)
    }

    pub fn dec(&mut self, val: u64) -> u64 {
        let _ = self.notify_on(SvcEvents::Dec(val));
        self.value.replace_with(|&mut old| old - val)
    }

    pub fn reset(&mut self) {
        let _ = self.notify_on(SvcEvents::Reset);
        self.value.replace(0);
    }
}
