#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use super::*;
use sails_rs::collections::BTreeMap;
use sails_rs::mem::MaybeUninit;

//#[derive(CompositeMigration)] Generates CompositeMigration impl for SomeStruct and trait SomeStructMigration
#[derive(Decode)]
#[codec(crate = sails_rs::scale_codec)]
pub struct SomeStruct {
    f1: u32,
    f2: Vec<String>,
}

//#[derive(CompositeMigration)]
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

//#[source_program_state(migration = MigrationV1V2)]
pub struct SourceProgram {
    //#[composite]
    svc1_state: Service1State,
    svc2_state: u32,
    svc3_state: Vec<String>,
    //#[composite]
    svc4_state: SomeStruct,
    //#[collection(chunk_size = 200)]
    svc5_state: Vec<SomeStruct>,
    //#[composite]
    svc6_state: SomeStruct,
}

pub struct MigrationV1V2 {
    target: Program,
}

pub mod sails {
    use super::*;
    type Error = String;

    pub trait RemotingMigrator {
        async fn read_from<T>(&mut self, actor_id: ActorId, path: &str) -> Result<T, Error>;

        async fn read_chunk_from<T>(
            &mut self,
            actor_id: ActorId,
            path: &str,
            offset: usize,
            limit: usize,
        ) -> Result<Vec<T>, Error>;
    }

    pub trait CompositeMigration<R, T> {
        async fn transfer_from(
            remoting: &mut R,
            target: &mut T,
            source_actor_id: ActorId,
            path: &str,
        );
    }
}

pub mod generated {
    use super::{sails::RemotingMigrator, *};

    // This trait generated based on #[composite] and #[collection] attributes
    // - composite - skipped for trait
    // - collection - generates methods with `offset` parameter
    pub trait SourceProgramMigration {
        fn visit_svc2_state(&mut self, path: &str, value: u64);
        fn visit_svc3_state(&mut self, path: &str, value: Vec<String>);
        fn visit_svc5_state(&mut self, path: &str, value: Vec<SomeStruct>, offset: usize);
    }

    pub trait Service1StateMigration {
        fn visit_a(&mut self, path: &str, value: u32);
        fn visit_b(&mut self, path: &str, value: SomeStruct);
        fn visit_c(&mut self, path: &str, value: Vec<u16>, offset: usize);
        fn visit_d(&mut self, path: &str, value: Vec<(u32, String)>, offset: usize);
    }

    pub trait SomeStructMigration {
        fn visit_f1(&mut self, path: &str, value: u32);
        fn visit_f2(&mut self, path: &str, value: Vec<String>);
    }

    // This impl generated based on atrributes
    impl<R, T> sails::CompositeMigration<R, T> for Service1State
    where
        R: RemotingMigrator,
        T: Service1StateMigration,
    {
        async fn transfer_from(
            remoting: &mut R,
            target: &mut T,
            source_actor_id: ActorId,
            path: &str,
        ) {
            {
                let value = remoting
                    .read_from::<u32>(source_actor_id, &format!("{path}/a"))
                    .await
                    .unwrap();
                <T as Service1StateMigration>::visit_a(target, path, value);
            }
            {
                let value = remoting
                    .read_from::<SomeStruct>(source_actor_id, &format!("{path}/b"))
                    .await
                    .unwrap();
                // let value = SomeStruct { f1: 0, f2: vec![] };
                <T as Service1StateMigration>::visit_b(target, path, value);
            }
            {
                let mut offset = 0;
                while let Some(value) = remoting
                    .read_chunk_from(source_actor_id, &format!("{path}/c"), offset, 100)
                    .await
                    .ok()
                {
                    let len = value.len();
                    <T as Service1StateMigration>::visit_c(target, path, value, offset);
                    offset += len;
                }
            }
            {
                let mut offset = 0;
                while let Some(value) = remoting
                    .read_chunk_from::<(u32, String)>(
                        source_actor_id,
                        &format!("{path}/d"),
                        offset,
                        100,
                    )
                    .await
                    .ok()
                {
                    let len = value.len();
                    target.visit_d(path, value, offset);
                    offset += len;
                }
            }
        }
    }

    impl<R, T> sails::CompositeMigration<R, T> for SomeStruct
    where
        R: RemotingMigrator,
        T: SomeStructMigration,
    {
        async fn transfer_from(
            remoting: &mut R,
            target: &mut T,
            source_actor_id: ActorId,
            path: &str,
        ) {
            {
                //let value = read_from::<u32>(source_actor_id, &format!("{path}/f1"));
                let value = 0u32;
                target.visit_f1(path, value);
            }
            {
                //let value = read_from::<Vec<String>>(source_actor_id, &format!("{path}/f2"));
                let value = vec![];
                target.visit_f2(path, value);
            }
        }
    }

    impl<R> sails::CompositeMigration<R, MigrationV1V2> for SourceProgram
    where
        R: RemotingMigrator,
        // + trait restriction for each #[composite] field
        // (!) Works only with two-level hierarchy
        // Self::Target: SourceProgramMigration + Service1StateMigration + SomeStructMigration,
    {
        async fn transfer_from(
            remoting: &mut R,
            target: &mut MigrationV1V2,
            source_actor_id: ActorId,
            path: &str,
        ) {
            <Service1State as sails::CompositeMigration<R, MigrationV1V2>>::transfer_from(
                remoting,
                target,
                source_actor_id,
                &format!("{path}/svc1_state"),
            )
            .await;
            {
                //let value = read_from::<u64>(source_actor_id, &format!("{path}/svc2_state"));
                let value = 0u64;
                target.visit_svc2_state(path, value);
            }
            {
                //let value = read_from::<Vec<String>>(source_actor_id, &format!("{path}/svc3_state"));
                let value = vec![];
                target.visit_svc3_state(path, value);
            }
            <SomeStruct as sails::CompositeMigration<R, MigrationV1V2>>::transfer_from(
                remoting,
                target,
                source_actor_id,
                &format!("{path}/svc4_state"),
            )
            .await;
            {
                let mut offset = 0;
                while let Some(value) = remoting
                    .read_chunk_from(source_actor_id, &format!("{path}/d"), offset, 200)
                    .await
                    .ok()
                {
                    let len = value.len();
                    target.visit_svc5_state(path, value, offset);
                    offset += len;
                }
            }
        }
    }
}

// All traits must be implemented by user for Migration
impl generated::SourceProgramMigration for MigrationV1V2 {
    fn visit_svc2_state(&mut self, path: &str, value: u64) {
        // let pointer = self.target.as_mut_ptr();
        // unsafe {
        //     ptr::addr_of_mut!((*pointer).svc2_state).write(value);
        // }
        self.target.svc2_state = value;
    }

    fn visit_svc3_state(&mut self, path: &str, value: Vec<String>) {
        todo!()
    }

    fn visit_svc5_state(&mut self, path: &str, value: Vec<SomeStruct>, offset: usize) {
        todo!()
    }
}

impl generated::Service1StateMigration for MigrationV1V2 {
    fn visit_a(&mut self, path: &str, value: u32) {
        todo!()
    }

    fn visit_b(&mut self, path: &str, value: SomeStruct) {
        todo!()
    }

    fn visit_c(&mut self, path: &str, value: Vec<u16>, offset: usize) {
        todo!()
    }

    fn visit_d(&mut self, path: &str, value: Vec<(u32, String)>, offset: usize) {
        todo!()
    }
}

impl generated::SomeStructMigration for MigrationV1V2 {
    fn visit_f1(&mut self, path: &str, value: u32) {
        todo!()
    }

    fn visit_f2(&mut self, path: &str, value: Vec<String>) {
        todo!()
    }
}

//#[program_state(composite)]
struct Program {
    //#[composite]
    svc1_state: Service1State,
    svc2_state: u64,
    svc3_state: Vec<String>,
    //#[composite]
    svc4_state: SomeStruct,
    //#[collection]
    svc5_state: Vec<SomeStruct>,
}

//#[program(upgradable)]
impl Program {
    pub fn new() -> Self {
        Self {
            svc1_state: Service1State {
                a: 0,
                b: SomeStruct { f1: 0, f2: vec![] },
                c: vec![],
                d: BTreeMap::new(),
            },
            svc2_state: 0,
            svc3_state: vec![],
            svc4_state: SomeStruct { f1: 0, f2: vec![] },
            svc5_state: vec![],
        }
    }
}
