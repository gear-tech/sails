#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use super::*;
use sails_rs::collections::BTreeMap;
use sails_rs::mem::MaybeUninit;

//#[derive(CompositeState)]
#[derive(Encode, Decode)]
#[codec(crate = sails_rs::scale_codec)]
pub struct SomeStruct {
    f1: u32,
    f2: Vec<String>,
}

//#[derive(CompositeState)]
#[derive(Decode)]
#[codec(crate = sails_rs::scale_codec)]
pub struct Service1State {
    a: u32,
    b: SomeStruct,
    //#[collection(chunk_size = 100)]
    c: Vec<u16>,
    //#[collection]
    d: BTreeMap<u32, String>,
}

//#[derive(CompositeState)]
pub struct SourceProgram {
    //#[composite]
    svc1_state: Service1State,
    svc2_state: u32,
    svc3_state: Vec<String>,
    //#[composite]
    svc4_state: SomeStruct,
    //#[collection(chunk_size = 200)]
    svc5_state: Vec<SomeStruct>,
}

pub mod sails {
    use core::str::FromStr;

    use sails_rs::collections::HashMap;

    use super::*;
    type Error = String;

    pub struct Range {
        start: u32,
        len: u32,
    }

    impl FromStr for Range {
        type Err = num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (start_s, len_s) = s.split_once(',').unwrap_or((s, "100"));
            let start = u32::from_str(start_s)?;
            let len = u32::from_str(len_s)?;
            Ok(Range { start, len })
        }
    }

    pub trait CompositeState {
        const PATHS: &[&'static str];
        fn extract(&self, path: &str) -> Vec<u8>;
    }

    impl<T> CompositeState for Vec<T>
    where
        T: Encode,
    {
        const PATHS: &[&'static str] = &[];

        fn extract(&self, path: &str) -> Vec<u8> {
            let range = Range::from_str(path).unwrap();
            self[range.start as usize..range.len as usize].encode()
        }
    }

    impl<K, V> CompositeState for BTreeMap<K, V>
    where
        K: Encode,
        V: Encode,
    {
        const PATHS: &[&'static str] = &[];

        fn extract(&self, path: &str) -> Vec<u8> {
            let range = Range::from_str(path).unwrap();

            self.iter()
                .skip(range.start as usize)
                .take(range.len as usize)
                .collect::<Vec<_>>()
                .encode()
        }
    }

    impl<K, V> CompositeState for HashMap<K, V>
    where
        K: Encode,
        V: Encode,
    {
        const PATHS: &[&'static str] = &[];

        fn extract(&self, path: &str) -> Vec<u8> {
            let range = Range::from_str(path).unwrap();

            self.iter()
                .skip(range.start as usize)
                .take(range.len as usize)
                .collect::<Vec<_>>()
                .encode()
        }
    }
}

pub mod generated {
    use super::{sails::CompositeState, *};

    impl CompositeState for SourceProgram {
        const PATHS: &[&'static str] = &[
            "svc1_state",
            "svc2_state",
            "svc3_state",
            "svc4_state",
            "svc5_state",
        ];

        fn extract(&self, path: &str) -> Vec<u8> {
            let (first, next) = path.split_once('/').unwrap_or((path, ""));
            //let [first, next @ ..] = path.split_first().unwrap();
            match first {
                "svc1_state" => self.svc1_state.extract(next),
                "svc2_state" => self.svc2_state.encode(),
                "svc3_state" => self.svc3_state.encode(),
                "svc4_state" => self.svc1_state.extract(next),
                "svc5_state" => self.svc1_state.extract(next),
                _ => unreachable!(),
            }
        }
    }

    impl CompositeState for Service1State {
        const PATHS: &[&'static str] = &["a", "b", "c", "d"];

        fn extract(&self, path: &str) -> Vec<u8> {
            let (first, next) = path.split_once('/').unwrap_or((path, ""));
            match first {
                "a" => self.a.encode(),
                "b" => self.b.encode(),
                "c" => self.c.extract(next),
                "d" => self.d.extract(next),
                _ => unreachable!(),
            }
        }
    }

    impl CompositeState for SomeStruct {
        const PATHS: &[&'static str] = &["f1", "f2"];

        fn extract(&self, path: &str) -> Vec<u8> {
            let (first, next) = path.split_once('/').unwrap_or((path, ""));
            match first {
                "f1" => self.f1.encode(),
                "f2" => self.f2.encode(),
                _ => unreachable!(),
            }
        }
    }
}
