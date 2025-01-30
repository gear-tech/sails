#![allow(dead_code)]
#![allow(unused_variables)]
use super::*;

//#[atomic_state_part]
#[derive(Decode)]
#[codec(crate = sails_rs::scale_codec)]
struct SomeItem {
    a: u32,
    b: String,
}

// Generated for `SomeItem`
impl<TMigration> Migratable<TMigration> for SomeItem
where
    TMigration: DocumentMigration<SomeItem>, // Because it is atomic
{
    fn migrate(source: ActorId, name: &str, migration: &mut TMigration) {
        let document = read_document::<SomeItem>(source, name);
        migration.extend(name, &document);
    }
}

//#[composite_state_part]
#[derive(Decode)]
#[codec(crate = sails_rs::scale_codec)]
struct SomeOtherItem {
    a: u32,
    //#[nested_state_part]
    some_item: SomeItem,
}

// Generated for `SomeOtherItem`
impl<TMigration> Migratable<TMigration> for SomeOtherItem
where
    TMigration: DocumentMigration<u32>, // Because it is not nested
    TMigration: DocumentMigration<SomeItem>, // ??? Not good
{
    fn migrate(source: ActorId, name: &str, migration: &mut TMigration) {
        <u32 as Migratable<TMigration>>::migrate(source, &format!("{}/a", name), migration);
        <SomeItem as Migratable<TMigration>>::migrate(
            source,
            &format!("{}/some_item", name),
            migration,
        );
    }
}

//#[composite_state_part]
struct YetAnotherItem {
    some_other_item: SomeOtherItem,
}

// Generated for `YetAnotherItem`
// impl<TMigration> Migratable<TMigration> for YetAnotherItem {
//     fn migrate(source: ActorId, name: &str, migration: &mut TMigration) {
//         <SomeOtherItem as Migratable<TMigration>>::migrate(
//             source,
//             &format!("{}/some_other_item", name),
//             migration,
//         );
//     }
// }

//////////////////////////////////////////////////////////////////////////
trait DocumentMigration<T> {
    fn extend(&mut self, name: &str, document: &T);
}

trait Migratable<TMigration> {
    fn migrate(source: ActorId, name: &str, migration: &mut TMigration);
}

impl<TMigration> Migratable<TMigration> for u32
where
    TMigration: DocumentMigration<u32>,
{
    fn migrate(source: ActorId, name: &str, migration: &mut TMigration) {
        let document = read_document::<u32>(source, name);
        migration.extend(name, &document);
    }
}

fn read_document<T>(actor_id: ActorId, name: &str) -> T
where
    T: Decode,
{
    let document_bytes: Vec<u8> = vec![]; // read bytes from another contract
    T::decode(&mut &document_bytes[..]).unwrap()
}
