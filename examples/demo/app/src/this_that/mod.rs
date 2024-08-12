use sails_rs::{gstd::debug, prelude::*};

#[derive(Default)]
pub struct MyService(());

#[service]
impl MyService {
    // This is a service command as it works over `&mut self`
    pub async fn do_this(
        &mut self,
        p1: u32,
        p2: String,
        p3: (Option<H160>, NonZeroU8),
        p4: TupleStruct,
    ) -> (String, u32) {
        debug!("Handling 'do_this': {}, {}, {:?}, {:?}", p1, p2, p3, p4);
        (p2, p1)
    }

    // This is another service command
    pub fn do_that(&mut self, param: DoThatParam) -> Result<(ActorId, NonZeroU32), (String,)> {
        debug!("Handling 'do_that': {:?}", param);
        Ok((param.p2, param.p1))
    }

    pub fn noop(&mut self) {
        debug!("Handling 'noop'");
    }

    // This is a service query as it works over `&self`
    pub fn this(&self) -> u32 {
        debug!("Handling 'this'");
        42
    }

    // This is another service query
    pub fn that(&self) -> Result<String, String> {
        debug!("Handling 'that'");
        Ok("Forty two".into())
    }
}

#[allow(dead_code)]
#[derive(Debug, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct TupleStruct(bool);

#[derive(Debug, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct DoThatParam {
    pub p1: NonZeroU32,
    pub p2: ActorId,
    pub p3: ManyVariants,
}

#[derive(Debug, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum ManyVariants {
    One,
    Two(u32),
    Three(Option<U256>),
    Four { a: u32, b: Option<u16> },
    Five(String, H256),
    Six((u32,)),
}
