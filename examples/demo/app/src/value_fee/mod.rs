use sails_rs::gstd::{exec, msg, CommandReply};
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
    /// using special type `CommandReply<T>`
    pub fn do_something_and_take_fee(&mut self) -> CommandReply<bool> {
        let value = msg::value();
        if value == 0 {
            return false.into();
        }
        if value < self.fee {
            panic!("Not enough value");
        }
        self.notify_on(FeeEvents::Withheld(self.fee)).unwrap();
        let to_return = value - self.fee;
        if to_return < exec::env_vars().existential_deposit {
            // return zero value with reply
            true.into()
        } else {
            // return remaining value with reply
            CommandReply::new(true).with_value(to_return)
        }
    }
}
