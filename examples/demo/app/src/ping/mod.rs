use sails_rs::prelude::*;

#[derive(Default)]
pub struct PingService(());

#[service]
impl PingService {
    // This is a service command as it works over `&mut self`
    pub fn ping(&mut self, input: String) -> Result<String, String> {
        if input != "ping" {
            Err("Invalid input".into())
        } else {
            Ok("pong".into())
        }
    }

    pub fn bit_vec_query(&self) -> BitVec<u8, Lsb0> {
        let slice = &[1u8, 1, 2, 3, 5];
        BitVec::<_, Lsb0>::from_slice(slice)
    }
}
