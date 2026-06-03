use sails_storage::{
    FixedOpenAddressMap, StaticLayout, StaticOpenAddressTable, StaticRegion, TableError,
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
fn root_static_region_api_is_usable_by_consumers() {
    let region = StaticRegion::new(10, 4).unwrap();

    assert_eq!(region.base(), 10);
    assert_eq!(region.bytes(), 4);
    assert_eq!(region.end(), Ok(14));
    assert_eq!(
        StaticRegion::new(usize::MAX, 1),
        Err(TableError::InvalidLayout)
    );
}
