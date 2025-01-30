#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use super::*;
use sails_rs::collections::BTreeMap;
use sails_rs::mem::MaybeUninit;

//#[derive(CompositeState)]
#[derive(Decode)]
#[codec(crate = sails_rs::scale_codec)]
struct SomeStruct {
    f1: u32,
    f2: Vec<String>,
}

//#[derive(CompositeState)] Structs marked with this attribute can't have fields marked as #[composite], only simple or #[collection] fields are allowed.
#[derive(Decode)]
#[codec(crate = sails_rs::scale_codec)]
struct Service1State {
    a: u32,
    b: SomeStruct,
    //#[collection(chunk_size = 100)]
    c: Vec<u16>,
    //#[collection]
    d: BTreeMap<u32, String>,
}

//#[source_program_state(composite, target = Program)]
struct SourceProgram {
    //#[composite]
    svc1_state: Service1State,
    svc2_state: u32,
    svc3_state: Vec<String>,
    //#[composite]
    svc4_state: SomeStruct,
    //#[collection(chunk_size = 200)]
    svc5_state: Vec<SomeStruct>,
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

impl TargetProgramState<u32> for Program {
    fn adopt(target: &mut MaybeUninit<Program>, path: &str, value: u32) {
        if path == "svc2_state" {
            let value = value as u64;
            write_field!(target, svc2_state, value);
        } else if path == "svc4_state/f1" {
            write_field!(target, svc4_state.f1, value);
        } else if path == "svc1_state/a" {
            write_field!(target, svc1_state.a, value);
        } else {
            panic!("Unexpected path: {}", path);
        }
    }
}

impl TargetProgramState<Vec<String>> for Program {
    fn adopt(target: &mut MaybeUninit<Program>, path: &str, value: Vec<String>) {
        if path == "svc3_state" {
            write_field!(target, svc3_state, value);
        } else if path == "svc4_state/f2" {
            write_field!(target, svc4_state.f2, value);
        } else {
            panic!("Unexpected path: {}", path);
        }
    }
}

impl TargetProgramState<SomeStruct> for Program {
    fn adopt(target: &mut MaybeUninit<Program>, path: &str, value: SomeStruct) {
        if path == "svc1_state/b" {
            write_field!(target, svc1_state.b, value);
        } else {
            panic!("Unexpected path: {}", path);
        }
    }
}

impl TargetProgramState<Vec<u16>> for Program {
    fn adopt(target: &mut MaybeUninit<Program>, path: &str, value: Vec<u16>) {
        if path == "svc1_state/c" {
            // Here we need to know whether we have already written first chunk of data or not
            // If not, we write the first chunk, otherwise we append the new chunk to the existing data
            // We can either introduce a tracker which will be updated by the write_field! macro or
            // we can introduce a separate trait for adopting collections which will accept chunk offset
            // as an argument which we can judge upon.
            // Another option is to have some fieild in the target program state which is marked as not a part of the state
            // , i.e. it is not migrated, which implements Default so we can initialize it in the generated code straight after
            // we create MaybeUninit<Program> instance.
            write_field!(target, svc1_state.c, value);
        } else {
            panic!("Unexpected path: {}", path);
        }
    }
}

impl TargetProgramState<BTreeMap<u32, String>> for Program {
    fn adopt(target: &mut MaybeUninit<Program>, path: &str, value: BTreeMap<u32, String>) {
        if path == "svc1_state/d" {
            // Here we need to know whether we have already written first chunk of data or not
            // If not, we write the first chunk, otherwise we append the new chunk to the existing data
            // We can either introduce a tracker which will be updated by the write_field! macro or
            // we can introduce a separate trait for adopting collections which will accept chunk offset
            // as an argument which we can judge upon.
            // Another option is to have some fieild in the target program state which is marked as not a part of the state
            // , i.e. it is not migrated, which implements Default so we can initialize it in the generated code straight after
            // we create MaybeUninit<Program> instance.
            write_field!(target, svc1_state.d, value);
        } else {
            panic!("Unexpected path: {}", path);
        }
    }
}

impl CollectionState1<Program> for Vec<SomeStruct> {
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut MaybeUninit<Program>,
    ) {
        // Do custom reads from source_actor_id using custom path or other custom rules
        // which are supported by the source program.

        // Apply read values to the target program state
    }
}

// ---- Generated code ----

// -- By #[derive(CompositeState)] over SomeStruct
impl<TProgramState> CompositeState1<TProgramState> for SomeStruct
where
    TProgramState: TargetProgramState<u32>,
    TProgramState: TargetProgramState<Vec<String>>,
{
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        target: &mut MaybeUninit<TProgramState>,
    ) {
        // Coz f1 is a simple state in SomeStruct
        {
            let path = format!("{path}/f1");
            let f1 = 0; // Get from source_actor_id using path and decode
            <TProgramState as TargetProgramState<u32>>::adopt(target, &path, f1);
        }

        // Coz f2 is a simple state in SomeStruct
        {
            let path = format!("{path}/f2");
            let f2 = vec![]; // Get from source_actor_id using path and decode
            <TProgramState as TargetProgramState<Vec<String>>>::adopt(target, &path, f2);
        }
    }
}

// -- By #[derive(CompositeState)] over Service1State
impl<TProgramState> CompositeState1<TProgramState> for Service1State
where
    TProgramState: TargetProgramState<u32>,
    TProgramState: TargetProgramState<SomeStruct>,
    TProgramState: TargetProgramState<Vec<u16>>,
    TProgramState: TargetProgramState<BTreeMap<u32, String>>,
{
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        target: &mut MaybeUninit<TProgramState>,
    ) {
        // Coz a is a simple state in Service1State
        {
            let path = format!("{path}/a");
            let value = 0; // Get from source_actor_id using path and decode
            <TProgramState as TargetProgramState<u32>>::adopt(target, &path, value);
        }

        // Coz b is a simple state in Service1State
        {
            let path = format!("{path}/b");
            let value = SomeStruct { f1: 0, f2: vec![] }; // Get from source_actor_id using path and decode
            <TProgramState as TargetProgramState<SomeStruct>>::adopt(target, &path, value);
        }

        // Coz c is a collection state in Service1State
        {
            let path = format!("{path}/c");
            <Vec<u16> as CollectionState1<TProgramState>>::transfer_from(
                source_actor_id,
                &path,
                Some(100),
                target,
            );
        }

        // Coz d is a collection state in Service1State
        {
            let path = format!("{path}/d");
            <BTreeMap<u32, String> as CollectionState1<TProgramState>>::transfer_from(
                source_actor_id,
                &path,
                None,
                target,
            );
        }
    }
}

// -- By #[program(upgradable)]
async fn part_of_handle_responsible_for_migrating_state(source_actor_id: ActorId) -> Program {
    let mut partially_migrated_state = MaybeUninit::<Program>::uninit();
    // Atm we assume that program state is always a composite state. If one wants to migrate
    // entire program state in one go, they can pack it into some struct so the program state
    // comprises a single field.
    // Another option is using some attribute like it is done in the case of `#[source_program_state(composite)]`
    <Program as CompositeState1<Program>>::transfer_from(
        source_actor_id,
        "",
        &mut partially_migrated_state,
    );
    let migrated_state = unsafe { partially_migrated_state.assume_init() };
    migrated_state
}

// -- #[source_program_state(composite, target = Program)]
impl CompositeState1<Program> for Program {
    fn transfer_from(source_actor_id: ActorId, path: &str, target: &mut MaybeUninit<Program>) {
        // Coz svc1_state is a composite state in SourceProgram
        {
            let path = format!("{path}/svc1_state");
            <Service1State as CompositeState1<Program>>::transfer_from(
                source_actor_id,
                &path,
                target,
            );
        }

        // Coz svc2_state is a simple state in SourceProgram
        {
            let path = format!("{path}/svc2_state");
            let svc2_state = 0; // Get from source_actor_id using path and decode
            <Program as TargetProgramState<u32>>::adopt(target, &path, svc2_state);
        }

        // Coz svc3_state is a simple state in SourceProgram
        {
            let path = format!("{path}/svc3_state");
            let svc3_state = vec![]; // Get from source_actor_id using path and decode
            <Program as TargetProgramState<Vec<String>>>::adopt(target, &path, svc3_state);
        }

        // Coz svc4_state is a composite state in SourceProgram
        {
            let path = format!("{path}/svc4_state");
            <SomeStruct as CompositeState1<Program>>::transfer_from(source_actor_id, &path, target);
        }

        // Coz svc5_state is a collection state in SourceProgram
        {
            let path = format!("{path}/svc5_state");
            <Vec<SomeStruct> as CollectionState1<Program>>::transfer_from(
                source_actor_id,
                &path,
                Some(200),
                target,
            );
        }
    }
}

////////////////////////////// sails-rs //////////////////////////////
trait CompositeState<TMigration> {
    fn transfer_from(source_actor_id: ActorId, path: &str, target: &mut TMigration);
}

trait CompositeState1<TProgramState> {
    fn transfer_from(source_actor_id: ActorId, path: &str, target: &mut MaybeUninit<TProgramState>);
}

trait CollectionState<TMigration> {
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut TMigration,
    );
}

trait CollectionState1<TProgramState> {
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut MaybeUninit<TProgramState>,
    );
}

trait TargetMigration<TValue> {
    fn adopt(&mut self, path: &str, value: TValue);
}

trait TargetProgramState<TValue> {
    fn adopt(target: &mut MaybeUninit<Self>, path: &str, value: TValue)
    where
        Self: Sized;
}

impl<TMigration, TItem> CollectionState<TMigration> for Vec<TItem>
where
    TItem: Decode,
    TMigration: TargetMigration<Vec<TItem>>,
{
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut TMigration,
    ) {
        let source_chunk = Vec::<TItem>::new();
        <TMigration as TargetMigration<Vec<TItem>>>::adopt(target, path, source_chunk);
    }
}

impl<TProgramState, TItem> CollectionState1<TProgramState> for Vec<TItem>
where
    TItem: Decode,
    TProgramState: TargetProgramState<Vec<TItem>>,
{
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut MaybeUninit<TProgramState>,
    ) {
        let source_chunk = Vec::<TItem>::new();
        <TProgramState as TargetProgramState<Vec<TItem>>>::adopt(target, path, source_chunk);
    }
}

impl<TMigration, TKey, TValue> CollectionState<TMigration> for BTreeMap<TKey, TValue>
where
    TKey: Decode + Ord,
    TValue: Decode,
    TMigration: TargetMigration<BTreeMap<TKey, TValue>>,
{
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut TMigration,
    ) {
        let source_chunk = BTreeMap::<TKey, TValue>::new();
        <TMigration as TargetMigration<BTreeMap<TKey, TValue>>>::adopt(target, path, source_chunk);
    }
}

impl<TProgramState, TKey, TValue> CollectionState1<TProgramState> for BTreeMap<TKey, TValue>
where
    TKey: Decode + Ord,
    TValue: Decode,
    TProgramState: TargetProgramState<BTreeMap<TKey, TValue>>,
{
    fn transfer_from(
        source_actor_id: ActorId,
        path: &str,
        chunk_size: Option<u32>,
        target: &mut MaybeUninit<TProgramState>,
    ) {
        let source_chunk = BTreeMap::<TKey, TValue>::new();
        <TProgramState as TargetProgramState<BTreeMap<TKey, TValue>>>::adopt(
            target,
            path,
            source_chunk,
        );
    }
}

#[macro_export]
macro_rules! write_field {
    ($state:expr, $($field:ident).+ , $value:expr) => {
        let instance: &mut mem::MaybeUninit<_> = $state;
        let pointer = instance.as_mut_ptr();
        unsafe {
            ptr::addr_of_mut!((*pointer).$($field).+).write($value);
        }
    };
}
