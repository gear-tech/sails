use sails_rs::prelude::*;

#[sails_type]
#[derive(PartialEq, Clone, Debug)]
pub struct MyType {
    pub a: u32,
    pub b: String,
}

#[sails_type]
#[derive(PartialEq, Clone, Debug)]
pub enum MyEnum {
    Unit,
    Tuple(u32, String),
    Named { x: u32 },
}

#[sails_type(no_reflect_hash)]
#[derive(PartialEq, Clone, Debug)]
pub struct LegacyType {
    pub a: u32,
}
