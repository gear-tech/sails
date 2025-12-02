use sails_rs::CommandReply;
use sails_rs::prelude::*;

pub struct PayableBehaviorsService;

#[service]
impl PayableBehaviorsService {
    #[export]
    pub fn check_non_payable_no_return(&self, input_val: u32) -> u32 {
        input_val
    }

    #[export(payable)]
    pub fn check_payable_no_return(&self, input_val: u32) -> u32 {
        input_val
    }

    #[export]
    pub fn check_non_payable_with_return(&mut self, amount: u128) -> CommandReply<u128> {
        CommandReply::new(amount).with_value(amount)
    }

    #[export(payable)]
    pub fn check_payable_with_return(&mut self, amount: u128) -> CommandReply<u128> {
        CommandReply::new(amount).with_value(amount)
    }
}
