#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]

use super::*;
use mem::MaybeUninit;

struct ServiceState {
    f1: u32,
    f2: Vec<u16>,
}

pub struct ProgramState {
    service_state: ServiceState,
    //new_field: u32,
}

macro_rules! write_field {
    ($state:expr, $($field:ident).+ , $value:expr) => {
        let instance: &mut mem::MaybeUninit<_> = $state;
        let pointer = instance.as_mut_ptr();
        unsafe {
            ptr::addr_of_mut!((*pointer).$($field).+).write($value);
        }
    };
}
// Would it be better to hide tracker as a static? Can't seem possible due to visibility/accessibility from different modules,
// but MigrationImpl should be implemented in one module, so potentially it could be a static.
// adopt(path: &str, value: &u16, new_state: &mut MaybeUninit<ProgramState>, tracker: &mut MigrationTracker) {
//     //if is_written!(tracker, new_state, service_state.f2)
//     if !tracker.is_written("service_state.f2") {
//        write_field!(new_state, service_state.f2, *value as u32);
//     }
//     else {
//        get_field_mut!(new_state, service_state.f2).push(*value as u16);
//     }
//     write_field!(new_state, service_state.f1, *value as u32, tracker);
// }
pub fn adopt(path: &str, value: &u16, new_state: &mut MaybeUninit<ProgramState>) {
    //let mut program_state = MaybeUninit::<ProgramState>::uninit();
    let value = *value as u32;
    write_field!(new_state, service_state.f1, value);
    write_field!(new_state, service_state.f2, vec![value as u16]);
}

pub fn finalize(new_state: &mut MaybeUninit<ProgramState>) {
    // written by developer
}
