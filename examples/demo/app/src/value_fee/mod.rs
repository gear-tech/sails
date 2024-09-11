use sails_rs::gstd::{exec, msg};
use sails_rs::prelude::*;

#[derive(Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
enum FeeEvents {
    Withheld(ValueUnit),
}

pub struct FeeService {
    fee: ValueUnit,
}

#[service(events = FeeEvents)]
impl FeeService {
    pub fn new(fee: ValueUnit) -> Self {
        Self { fee }
    }

    /// Return flag if fee taken and remain value,
    /// using special tuple syntax `(T, ValueUnit)`
    pub fn do_something_and_take_fee(&mut self) -> (bool, ValueUnit) {
        let value = msg::value();
        if value == 0 {
            return (false, value);
        }
        if value < self.fee {
            panic!("Not enough value");
        }
        self.notify_on(FeeEvents::Withheld(self.fee)).unwrap();
        let to_return = value - self.fee;
        if to_return < exec::env_vars().existential_deposit {
            (true, 0)
        } else {
            // return remaining value with reply
            (true, to_return)
        }
    }
}
