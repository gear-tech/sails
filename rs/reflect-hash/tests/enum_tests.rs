//! Tests for #[derive(ReflectHash)] on enums

use keccak_const::Keccak256;
use sails_reflect_hash::ReflectHash;

#[derive(ReflectHash)]
#[allow(dead_code)]
enum SimpleEnum {
    Variant1,
    Variant2,
    Variant3,
}

#[derive(ReflectHash)]
#[allow(dead_code)]
enum MixedEnum {
    Unit,
    Tuple(u32, u64),
    Named { x: u32, y: u64 },
}

#[test]
fn test_simple_enum_derive() {
    // Variant hashes
    let v1 = Keccak256::new().update(b"Variant1").finalize();
    let v2 = Keccak256::new().update(b"Variant2").finalize();
    let v3 = Keccak256::new().update(b"Variant3").finalize();

    // Final enum hash
    let manual_hash = Keccak256::new()
        .update(&v1)
        .update(&v2)
        .update(&v3)
        .finalize();

    assert_eq!(SimpleEnum::HASH, manual_hash);
}

#[test]
fn test_mixed_enum_derive() {
    // Variant 1: Unit
    let v1 = Keccak256::new().update(b"Unit").finalize();

    // Variant 2: Tuple(u32, u64)
    let v2 = Keccak256::new()
        .update(b"Tuple")
        .update(&u32::HASH)
        .update(&u64::HASH)
        .finalize();

    // Variant 3: Named { x: u32, y: u64 }
    // Field names are NOT included - only types
    let v3 = Keccak256::new()
        .update(b"Named")
        .update(&u32::HASH)
        .update(&u64::HASH)
        .finalize();

    // Final enum hash
    let manual_hash = Keccak256::new()
        .update(&v1)
        .update(&v2)
        .update(&v3)
        .finalize();

    assert_eq!(MixedEnum::HASH, manual_hash);
}

#[derive(ReflectHash)]
#[allow(dead_code)]
enum TransferEvent {
    Transferred {
        from: gprimitives::ActorId,
        to: gprimitives::ActorId,
        amount: u128,
    },
    Approved(gprimitives::ActorId, u128),
    Paused,
}

#[test]
fn test_transfer_event_derive() {
    use gprimitives::ActorId;

    // Variant 1: Transferred { from, to, amount }
    // Field names are NOT included - only types
    let v1 = Keccak256::new()
        .update(b"Transferred")
        .update(&ActorId::HASH)
        .update(&ActorId::HASH)
        .update(&u128::HASH)
        .finalize();

    // Variant 2: Approved(ActorId, u128)
    let v2 = Keccak256::new()
        .update(b"Approved")
        .update(&ActorId::HASH)
        .update(&u128::HASH)
        .finalize();

    // Variant 3: Paused
    let v3 = Keccak256::new().update(b"Paused").finalize();

    // Final hash
    let manual_hash = Keccak256::new()
        .update(&v1)
        .update(&v2)
        .update(&v3)
        .finalize();

    assert_eq!(TransferEvent::HASH, manual_hash);
}

#[derive(ReflectHash)]
#[allow(dead_code)]
enum EmptyEnum {}

#[test]
fn test_empty_enum() {
    // Empty enum should have a hash (even though it's uninhabited)
    let manual_hash = Keccak256::new().finalize();
    assert_eq!(EmptyEnum::HASH, manual_hash);
}
