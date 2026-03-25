use sails_rs::prelude::*;

#[event]
#[derive(Clone, Debug, PartialEq, Encode, TypeInfo, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[type_info(crate = sails_rs::type_info)]
#[reflect_hash(crate = sails_rs)]
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
    /// Return `Ok(())` if fee taken and remain value,
    /// using special type `CommandReply<T>`
    #[export(unwrap_result)]
    pub fn do_something_and_take_fee(&mut self) -> Result<CommandReply<()>, String> {
        let value = Syscall::message_value();
        if value == 0 {
            return Ok(().into());
        }
        if value < self.fee {
            return Err("Not enough value".to_string());
        }
        self.emit_event(FeeEvents::Withheld(self.fee)).unwrap();
        let to_return = value - self.fee;
        if to_return < Syscall::env_vars().existential_deposit {
            // return zero value with reply
            Ok(().into())
        } else {
            // return remaining value with reply
            Ok(CommandReply::new(()).with_value(to_return))
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
        let mut fee_service = FeeService::new(100).expose(1); // fee = 100

        // Act: invoke fee service.
        let (_, value) = fee_service.do_something_and_take_fee().unwrap().to_tuple();

        // Assert: should return `Ok(())` with no extra value.
        assert_eq!(value, 0);
    }

    #[test]
    #[should_panic(expected = "Not enough value")]
    fn test_insufficient_value() {
        // Arrange: simulate call with insufficient transferred value.
        Syscall::with_message_value(100);

        let mut fee_service = FeeService::new(200).expose(1); // fee = 200

        // Act: this should panic because transferred value < fee.
        let _ = fee_service.do_something_and_take_fee().unwrap();
    }

    #[test]
    fn test_fee_taken_with_zero_remaining() {
        // Arrange
        Syscall::with_message_value(100);
        let mut fee_service = FeeService::new(100).expose(1);

        // Act: fee is taken and remaining value is too small.
        let (_, value) = fee_service.do_something_and_take_fee().unwrap().to_tuple();

        // Assert: reply `Ok(())` but without carrying extra value.
        assert_eq!(value, 0);
    }

    #[test]
    fn test_fee_taken_with_remaining_value() {
        // Arrange: simulate a call where transferred value is high enough so that the remainder is
        // at or above the existential deposit.
        let message_value = 200 + Syscall::env_vars().existential_deposit;
        let fee = 100;
        Syscall::with_message_value(message_value);
        let mut fee_service = FeeService::new(fee).expose(1);

        // Act: fee is taken and the remaining value is passed along.
        let (_, value) = fee_service.do_something_and_take_fee().unwrap().to_tuple();

        // Assert: reply `Ok(())` and carries the remaining value (message_value - fee)
        // in its value field.
        assert_eq!(value, message_value - fee);
    }
}
