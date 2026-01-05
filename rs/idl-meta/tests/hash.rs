use std::str::FromStr;

use sails_idl_meta::*;

mod fixture;

#[test]
fn hash_counter() {
    let s = fixture::counter_service();
    let interface_id = s.interface_id().unwrap();
    assert_eq!(
        interface_id,
        InterfaceId::from_str("0x579d6daba41b7d82").unwrap()
    )
}
