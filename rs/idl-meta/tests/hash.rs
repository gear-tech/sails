use std::str::FromStr;

use sails_idl_meta::*;

mod fixture;

#[test]
fn hash_counter() {
    let s = fixture::counter_service();
    let inteface_id = s.inteface_id();
    assert_eq!(
        inteface_id,
        InterfaceId::from_str("0x579d6daba41b7d82").unwrap()
    )
}
