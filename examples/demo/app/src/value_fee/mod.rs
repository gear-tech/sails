use sails_rs::prelude::*;

#[event]
#[derive(Clone, Debug, PartialEq, Encode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum FeeEvents {
    Withheld(ValueUnit),
}

pub struct FeeService {
    fee: ValueUnit,
}
impl FeeService {
    pub fn new(fee: ValueUnit) -> Self {
        Self { fee }
    }
}

#[service(events = FeeEvents)]
impl FeeService {
    /// Return flag if fee taken and remain value,
    /// using special type `CommandReply<T>`
    #[export]
    pub fn do_something_and_take_fee(&mut self) -> CommandReply<bool> {
        let value = Syscall::message_value();
        if value == 0 {
            return false.into();
        }
        if value < self.fee {
            panic!("Not enough value");
        }
        self.emit_event(FeeEvents::Withheld(self.fee)).unwrap();
        let to_return = value - self.fee;
        if to_return < Syscall::env_vars().existential_deposit {
            // return zero value with reply
            true.into()
        } else {
            // return remaining value with reply
            CommandReply::new(true).with_value(to_return)
        }
    }
}

#[cfg(test)]
mod tests {
    use sails_rs::gstd::services::Service;

    use super::*;

    #[test]
    fn test_zero_value() {
        // Arrange: simulate call with zero transferred value.
        Syscall::with_message_value(0);
        let mut fee_service = FeeService::new(100).expose(&[]); // fee = 100

        // Act: invoke fee service.
        let (data, value) = fee_service.do_something_and_take_fee().to_tuple();

        // Assert: should return false with no extra value.
        assert!(!data);
        assert_eq!(value, 0);
    }

    #[test]
    #[should_panic(expected = "Not enough value")]
    fn test_insufficient_value() {
        // Arrange: simulate call with insufficient transferred value.
        Syscall::with_message_value(100);

        let mut fee_service = FeeService::new(200).expose(&[]); // fee = 200

        // Act: this should panic because transferred value < fee.
        let _ = fee_service.do_something_and_take_fee();
    }

    #[test]
    fn test_fee_taken_with_zero_remaining() {
        // Arrange
        Syscall::with_message_value(100);
        let mut fee_service = FeeService::new(100).expose(&[]);

        // Act: fee is taken and remaining value is too small.
        let (data, value) = fee_service.do_something_and_take_fee().to_tuple();

        // Assert: reply indicates success (true) but without carrying extra value.
        assert!(data);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_fee_taken_with_remaining_value() {
        // Arrange: simulate a call where transferred value is high enough so that the remainder is
        // at or above the existential deposit.
        let message_value = 200 + Syscall::env_vars().existential_deposit;
        let fee = 100;
        Syscall::with_message_value(message_value);
        let mut fee_service = FeeService::new(fee).expose(&[]);

        // Act: fee is taken and the remaining value is passed along.
        let (data, value) = fee_service.do_something_and_take_fee().to_tuple();

        // Assert: reply indicates success (true) and carries the remaining value (message_value - fee)
        // in its value field.
        assert!(data);
        assert_eq!(value, message_value - fee);
    }
}
