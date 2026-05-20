use gprimitives::{ActorId, U256};
use sails_storage::{
    ACTOR_ID_U256_SLOT_SIZE, ACTOR_PAIR_U256_SLOT_SIZE, FixedOpenAddressMap, StaticActorIdU256Map,
    StaticActorPairU256Map, StaticLayout, StaticOpenAddressTable, TableError,
};

#[test]
fn root_fixed_and_static_table_api_is_usable_by_consumers() {
    let mut fixed = FixedOpenAddressMap::<1, 1, 2>::new();
    assert_eq!(fixed.insert([1], [10]), Ok(None));
    assert_eq!(fixed.get(&[1]), Ok(Some([10])));
    assert_eq!(fixed.remove(&[1]), Ok(Some([10])));

    let mut memory = vec![0u8; StaticOpenAddressTable::<1, 1>::bytes_len(2).unwrap()];
    let table =
        unsafe { StaticOpenAddressTable::<1, 1>::new(memory.as_mut_ptr() as usize, 2).unwrap() };
    assert_eq!(table.insert(&[1], &[10]), Ok(None));
    assert_eq!(table.get(&[1]), Ok(Some([10])));

    let mut layout = StaticLayout::new(1024, 4096).unwrap();
    let region = layout.reserve_table::<1, 1>(2).unwrap();
    assert_eq!(region.base(), 1024);
    assert_eq!(region.slots(), 2);
}

#[test]
fn root_actor_static_maps_preserve_layout_and_basic_operations() {
    assert_eq!(
        StaticActorIdU256Map::<2>::slot_size(),
        ACTOR_ID_U256_SLOT_SIZE
    );
    assert_eq!(
        StaticActorPairU256Map::<2>::slot_size(),
        ACTOR_PAIR_U256_SLOT_SIZE
    );
    assert_eq!(StaticActorIdU256Map::<2>::slots(), Ok(4));
    assert_eq!(StaticActorPairU256Map::<2>::slots(), Ok(4));

    let mut actor_memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
    let actors =
        unsafe { StaticActorIdU256Map::<2>::new(actor_memory.as_mut_ptr() as usize).unwrap() };
    let account = ActorId::from(1u64);

    assert_eq!(actors.get_actor_u256(&account), Ok(None));
    assert_eq!(actors.insert_actor_u256(account, U256::from(10)), Ok(None));
    assert_eq!(actors.get_actor_u256(&account), Ok(Some(U256::from(10))));
    assert_eq!(
        actors.insert_actor_u256(account, U256::zero()),
        Ok(Some(U256::from(10)))
    );
    assert_eq!(actors.get_actor_u256(&account), Ok(None));

    let mut pair_memory = vec![0u8; StaticActorPairU256Map::<2>::bytes_len().unwrap()];
    let pairs =
        unsafe { StaticActorPairU256Map::<2>::new(pair_memory.as_mut_ptr() as usize).unwrap() };
    let left = ActorId::from(2u64);
    let right = ActorId::from(3u64);

    assert_eq!(pairs.get_actor_pair_u256(&left, &right), Ok(None));
    assert_eq!(
        pairs.insert_actor_pair_u256(left, right, U256::from(20)),
        Ok(None)
    );
    assert_eq!(
        pairs.get_actor_pair_u256(&left, &right),
        Ok(Some(U256::from(20)))
    );
    assert_eq!(
        pairs.remove_actor_pair_u256(&left, &right),
        Ok(Some(U256::from(20)))
    );
}

#[test]
fn root_actor_maps_reject_zero_actor_mutation() {
    let mut actor_memory = vec![0u8; StaticActorIdU256Map::<2>::bytes_len().unwrap()];
    let actors =
        unsafe { StaticActorIdU256Map::<2>::new(actor_memory.as_mut_ptr() as usize).unwrap() };
    assert_eq!(
        actors.insert_actor_u256(ActorId::zero(), U256::from(1)),
        Err(TableError::InvalidKey)
    );

    let mut pair_memory = vec![0u8; StaticActorPairU256Map::<2>::bytes_len().unwrap()];
    let pairs =
        unsafe { StaticActorPairU256Map::<2>::new(pair_memory.as_mut_ptr() as usize).unwrap() };
    assert_eq!(
        pairs.insert_actor_pair_u256(ActorId::zero(), ActorId::from(1u64), U256::from(1)),
        Err(TableError::InvalidKey)
    );
}
